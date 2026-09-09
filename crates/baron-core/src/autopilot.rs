//! Bounded Autopilot learning over Baron Core authorities.
//!
//! Autopilot records observations and proposes changes. It never makes a
//! proposal current merely because it was repeated, scored highly, observed by
//! a hook, or agreed on by multiple adapters. Durable project decisions go
//! through the existing Product Harness decision authority after an explicit,
//! correlated approval.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Local, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::harness::record_decision;
use crate::intent::intent_status;
use crate::operation::OperationContext;
use crate::safe_io::{acquire_project_lock, read_text, replace_text};
use crate::vault::VaultContext;

const AUTOPILOT_SCHEMA_VERSION: u32 = 1;
const MAX_CANDIDATES: usize = 256;
const MAX_PROVENANCE: usize = 64;
const MAX_RESPONSE_RECORDS: usize = 256;
const DEFER_COOLDOWN_HOURS: i64 = 24;
const STALE_CANDIDATE_DAYS: i64 = 30;
const STATE_RELATIVE: &str = "docs/baron/autopilot/STATE.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopilotReview {
    pub candidate_count: usize,
    pub candidate_ids: Vec<String>,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub approval_required: bool,
    pub observed_automation: Vec<String>,
    pub resume_sources: Vec<String>,
    pub status: String,
    pub readiness: String,
    pub scope: String,
    pub provenance: Vec<String>,
}

/// Project-scoped evidence used to correlate observations and approvals.
/// Adapter identity is retained as provenance but is never a trust authority.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopilotCorrelation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

/// A candidate is deliberately richer than the legacy Markdown line while
/// retaining a `trusted` flag that is always false for Autopilot records.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutopilotCandidate {
    pub id: String,
    pub summary: String,
    pub scope: String,
    pub impact: String,
    pub status: String,
    pub trusted: bool,
    pub evidence_count: usize,
    pub provenance: Vec<String>,
    pub observed_automation: Vec<String>,
    pub resume_sources: Vec<String>,
    pub contradiction: Option<String>,
    pub readiness: String,
    pub created_at: String,
    pub updated_at: String,
    pub correlation: AutopilotCorrelation,
    pub suppressed_until: Option<String>,
    pub replacement_for: Option<String>,
    pub decision_recorded: bool,
    #[serde(default)]
    pub proposal_key: String,
    #[serde(default)]
    pub semantic_key: String,
    #[serde(default)]
    pub evidence_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopilotApproval {
    pub candidate_id: String,
    pub status: String,
    pub changed: bool,
    pub promotion: String,
    pub message: String,
    pub child_promoted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopilotHousekeeping {
    pub deduplicated: usize,
    pub expired: usize,
    pub suppression_reopened: usize,
    pub pruned: usize,
    pub pending: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AutopilotState {
    #[serde(default = "schema_version")]
    schema_version: u32,
    #[serde(default)]
    project_id: String,
    #[serde(default)]
    candidates: Vec<AutopilotCandidate>,
    #[serde(default)]
    responses: Vec<AutopilotResponseRecord>,
    #[serde(default)]
    archived: Vec<AutopilotCandidate>,
}

impl AutopilotState {
    fn new(project_id: &str) -> Self {
        Self {
            schema_version: AUTOPILOT_SCHEMA_VERSION,
            project_id: project_id.to_string(),
            candidates: Vec::new(),
            responses: Vec::new(),
            archived: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct AutopilotResponseRecord {
    fingerprint: String,
    candidate_id: String,
    status: String,
    promotion: String,
    child_promoted: bool,
    correlation: AutopilotCorrelation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ApprovalAction {
    Approve,
    Reject,
    Defer,
    Correct(String),
}

pub fn review_after_task(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<AutopilotReview> {
    review_after_task_inner(repo_root.as_ref(), vault, summary, None)
}

/// Record a task observation while retaining explicit adapter/session/request
/// provenance. The operation identity is never used to change trust.
pub fn review_after_task_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
    operation: &OperationContext,
) -> Result<AutopilotReview> {
    review_after_task_inner(repo_root.as_ref(), vault, summary, Some(operation))
}

fn review_after_task_inner(
    repo_root: &Path,
    vault: &VaultContext,
    summary: &str,
    operation: Option<&OperationContext>,
) -> Result<AutopilotReview> {
    let summary = one_line(summary);
    let summary = if summary.is_empty() {
        "post-task review captured with no summary".to_string()
    } else {
        summary
    };
    let observed_automation = observed_automation(vault);
    let resume_sources = resume_sources(repo_root);
    let scope = project_scope(vault);
    let correlation = correlation(operation, &summary, vault);
    let _lock = acquire_project_lock(repo_root)?;
    let mut state = load_state(repo_root, vault)?;
    let mut changed = housekeeping_state(&mut state);
    let proposal_key = normalize_proposal(&summary);
    let semantic = semantic_key(&proposal_key);
    let impact = infer_impact(&summary);
    let observation_key = observation_key(&scope, &summary, &correlation, &observed_automation);
    if let Some(candidate) = state.archived.iter().find(|candidate| {
        candidate.scope == scope
            && candidate.proposal_key == proposal_key
            && matches!(
                candidate.status.as_str(),
                "approved" | "rejected" | "superseded"
            )
    }) {
        if changed {
            save_state(repo_root, vault, &state)?;
        }
        return Ok(AutopilotReview {
            candidate_count: 1,
            candidate_ids: vec![candidate.id.clone()],
            repo_path: repo_root.join("docs/baron/autopilot/CANDIDATES.md"),
            vault_path: vault.project_root.join("Autopilot/CANDIDATES.md"),
            approval_required: false,
            observed_automation,
            resume_sources,
            status: candidate.status.clone(),
            readiness: candidate.readiness.clone(),
            scope: candidate.scope.clone(),
            provenance: candidate.provenance.clone(),
        });
    }
    let candidate_index = state.candidates.iter().position(|candidate| {
        candidate.scope == scope
            && candidate.proposal_key == proposal_key
            && candidate.status != "superseded"
    });
    let now = now();
    let index = if let Some(index) = candidate_index {
        let candidate = &mut state.candidates[index];
        if candidate
            .evidence_keys
            .iter()
            .all(|key| key != &observation_key)
            && !is_permanently_suppressed(candidate)
        {
            let observation_provenance = format!("observation:{observation_key}");
            candidate.evidence_keys.push(observation_key);
            candidate.provenance.push(observation_provenance);
            candidate.evidence_count = candidate.evidence_keys.len();
            candidate.updated_at = now.clone();
            merge_observation(
                candidate,
                &observed_automation,
                &resume_sources,
                &correlation,
            );
            changed = true;
        }
        index
    } else {
        let id = candidate_id(&vault.project_id, &scope, &proposal_key, &impact);
        state.candidates.push(new_candidate(
            id,
            summary.clone(),
            scope.clone(),
            impact,
            proposal_key,
            semantic,
            observation_key,
            observed_automation.clone(),
            resume_sources.clone(),
            correlation.clone(),
            &now,
        ));
        changed = true;
        state.candidates.len() - 1
    };

    let intent_changed = apply_explicit_intent_authority(repo_root, &mut state.candidates[index]);
    changed |= intent_changed;
    changed |= refresh_conflicts(&mut state.candidates);
    update_readiness(&mut state.candidates);
    if changed {
        save_state(repo_root, vault, &state)?;
    }
    let candidate = &state.candidates[index];
    let approval_required = approval_required(candidate);
    Ok(AutopilotReview {
        candidate_count: 1,
        candidate_ids: vec![candidate.id.clone()],
        repo_path: repo_root.join("docs/baron/autopilot/CANDIDATES.md"),
        vault_path: vault.project_root.join("Autopilot/CANDIDATES.md"),
        approval_required,
        observed_automation,
        resume_sources,
        status: candidate.status.clone(),
        readiness: candidate.readiness.clone(),
        scope: candidate.scope.clone(),
        provenance: candidate.provenance.clone(),
    })
}

/// Load project-scoped candidates from the additive ledger. Legacy Markdown is
/// imported when no ledger exists, and all returned records remain untrusted.
pub fn candidate_records(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> Result<Vec<AutopilotCandidate>> {
    Ok(load_state(repo_root.as_ref(), vault)?.candidates)
}

/// Maintain bounded candidate metadata without changing trust or runtime
/// behavior. This is safe to call from lifecycle hooks.
pub fn housekeep_candidates(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> Result<AutopilotHousekeeping> {
    let repo_root = repo_root.as_ref();
    let has_state = repo_state_path(repo_root).exists()
        || vault_state_path(vault).exists()
        || repo_root
            .join("docs/baron/autopilot/CANDIDATES.md")
            .exists()
        || vault.project_root.join("Autopilot/CANDIDATES.md").exists();
    if !has_state {
        return Ok(AutopilotHousekeeping {
            deduplicated: 0,
            expired: 0,
            suppression_reopened: 0,
            pruned: 0,
            pending: 0,
        });
    }
    let _lock = acquire_project_lock(repo_root)?;
    let mut state = load_state(repo_root, vault)?;
    let report = housekeeping_state_report(&mut state);
    save_state(repo_root, vault, &state)?;
    Ok(report)
}

/// Resolve a natural-language response against the current project pending
/// approval. No internal candidate ID is required for ordinary use.
pub fn respond_to_pending_approval(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    response: &str,
) -> Result<AutopilotApproval> {
    respond_to_pending_approval_inner(repo_root.as_ref(), vault, response, None)
}

/// The adapter is only correlation/provenance; Codex and Claude can resolve the
/// same project-scoped approval.
pub fn respond_to_pending_approval_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    response: &str,
    operation: &OperationContext,
) -> Result<AutopilotApproval> {
    respond_to_pending_approval_inner(repo_root.as_ref(), vault, response, Some(operation))
}

fn respond_to_pending_approval_inner(
    repo_root: &Path,
    vault: &VaultContext,
    response: &str,
    operation: Option<&OperationContext>,
) -> Result<AutopilotApproval> {
    let action = parse_approval_response(response)?;
    let _lock = acquire_project_lock(repo_root)?;
    let mut state = load_state(repo_root, vault)?;
    housekeeping_state(&mut state);
    let scope = project_scope(vault);
    let normalized_response = normalize_proposal(response);
    if let Some(previous) = find_previous_response(&state, &normalized_response, &scope) {
        return Ok(AutopilotApproval {
            candidate_id: previous.candidate_id.clone(),
            status: previous.status.clone(),
            changed: false,
            promotion: previous.promotion.clone(),
            message: "The same correlated approval response was already applied.".to_string(),
            child_promoted: previous.child_promoted,
        });
    }
    let pending_index = resolve_pending_candidate(&state.candidates, operation, &action)
        .context("No pending Autopilot approval is available for this project")?;
    let pending_id = state.candidates[pending_index].id.clone();
    let fingerprint = response_fingerprint(&pending_id, response, &scope);
    if let Some(previous) = state
        .responses
        .iter()
        .find(|record| record.fingerprint == fingerprint)
    {
        return Ok(AutopilotApproval {
            candidate_id: previous.candidate_id.clone(),
            status: previous.status.clone(),
            changed: false,
            promotion: previous.promotion.clone(),
            message: "The same correlated approval response was already applied.".to_string(),
            child_promoted: previous.child_promoted,
        });
    }

    // Remove the pending item temporarily so a correction can append a
    // replacement without holding two mutable borrows into the ledger.
    let mut candidate = state.candidates.remove(pending_index);
    if candidate.status == "conflict" && matches!(action, ApprovalAction::Approve) {
        bail!(
            "Autopilot candidate `{}` has contradictory evidence; clarify or correct it before approval",
            candidate.id
        );
    }
    let response_correlation = correlation(operation, &candidate.summary, vault);
    let outcome = match action {
        ApprovalAction::Approve => approve_state_candidate(repo_root, vault, &mut candidate)?,
        ApprovalAction::Reject => {
            candidate.status = "rejected".to_string();
            candidate.suppressed_until = Some("never".to_string());
            candidate.updated_at = now();
            candidate.readiness =
                "rejected; prompt suppressed until a new explicit proposal is made".to_string();
            AutopilotApproval {
                candidate_id: candidate.id.clone(),
                status: candidate.status.clone(),
                changed: true,
                promotion: "none; rejection is not a fact or policy".to_string(),
                message: "Rejected and suppressed repeat prompts for this proposal.".to_string(),
                child_promoted: false,
            }
        }
        ApprovalAction::Defer => {
            candidate.status = "deferred".to_string();
            candidate.suppressed_until = Some(
                (Local::now() + Duration::hours(DEFER_COOLDOWN_HOURS))
                    .to_rfc3339_opts(SecondsFormat::Secs, false),
            );
            candidate.updated_at = now();
            candidate.readiness =
                "deferred; prompt cooldown is bounded and approval remains pending".to_string();
            AutopilotApproval {
                candidate_id: candidate.id.clone(),
                status: candidate.status.clone(),
                changed: true,
                promotion: "none; deferred candidates remain untrusted".to_string(),
                message: "Deferred; Baron will not nag during the bounded cooldown.".to_string(),
                child_promoted: false,
            }
        }
        ApprovalAction::Correct(corrected) => {
            let replacement = create_correction(vault, &mut candidate, &corrected, &mut state)?;
            AutopilotApproval {
                candidate_id: replacement,
                status: "corrected".to_string(),
                changed: true,
                promotion: "none; corrected proposal requires its own explicit approval"
                    .to_string(),
                message: "Recorded the correction and replaced the broad proposal.".to_string(),
                child_promoted: false,
            }
        }
    };
    state.candidates.insert(pending_index, candidate);
    state.responses.push(AutopilotResponseRecord {
        fingerprint,
        candidate_id: outcome.candidate_id.clone(),
        status: outcome.status.clone(),
        promotion: outcome.promotion.clone(),
        child_promoted: outcome.child_promoted,
        correlation: response_correlation,
    });
    bound_responses(&mut state.responses);
    update_readiness(&mut state.candidates);
    save_state(repo_root, vault, &state)?;
    Ok(outcome)
}

/// Legacy explicit-ID approval remains available as a diagnostic command. It
/// uses the same authority and safety path as conversational approval.
pub fn approve_candidate(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    candidate_id: &str,
) -> Result<()> {
    decide_candidate_by_id(
        repo_root.as_ref(),
        vault,
        candidate_id,
        ApprovalAction::Approve,
    )
}

pub fn reject_candidate(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    candidate_id: &str,
) -> Result<()> {
    decide_candidate_by_id(
        repo_root.as_ref(),
        vault,
        candidate_id,
        ApprovalAction::Reject,
    )
}

fn decide_candidate_by_id(
    repo_root: &Path,
    vault: &VaultContext,
    candidate_id: &str,
    action: ApprovalAction,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let mut state = load_state(repo_root, vault)?;
    let index = state
        .candidates
        .iter()
        .position(|candidate| candidate.id == candidate_id.trim())
        .context("Candidate not found")?;
    let candidate = &mut state.candidates[index];
    if matches!(action, ApprovalAction::Approve)
        && candidate.status == "approved"
        && candidate.decision_recorded
    {
        return Ok(());
    }
    if matches!(action, ApprovalAction::Reject) && candidate.status == "rejected" {
        return Ok(());
    }
    match action {
        ApprovalAction::Approve => {
            let outcome = approve_state_candidate(repo_root, vault, candidate)?;
            let fingerprint =
                response_fingerprint(candidate_id, "legacy approve", &project_scope(vault));
            state.responses.push(AutopilotResponseRecord {
                fingerprint,
                candidate_id: outcome.candidate_id,
                status: outcome.status,
                promotion: outcome.promotion,
                child_promoted: outcome.child_promoted,
                correlation: AutopilotCorrelation::default(),
            });
        }
        ApprovalAction::Reject => {
            candidate.status = "rejected".to_string();
            candidate.suppressed_until = Some("never".to_string());
            candidate.updated_at = now();
        }
        ApprovalAction::Defer | ApprovalAction::Correct(_) => {
            bail!("This diagnostic operation only accepts approve or reject")
        }
    }
    bound_responses(&mut state.responses);
    update_readiness(&mut state.candidates);
    save_state(repo_root, vault, &state)
}

pub fn autopilot_status(repo_root: impl AsRef<Path>, vault: &VaultContext) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let state = load_state(repo_root, vault)?;
    let candidate_count = state.candidates.len();
    let open_count = state
        .candidates
        .iter()
        .filter(|candidate| approval_required(candidate))
        .count();
    let observed = observed_automation(vault);
    let continuity = fs::read_to_string(repo_root.join("docs/baron/continuity/CURRENT.md"))
        .unwrap_or_else(|_| {
            "# Baron Continuity Resume\n\n- Status: no checkpoint recorded\n- Next action: inspect current context before editing\n".to_string()
        });
    let mut output = String::new();
    output.push_str("# Baron Autopilot Status\n\n");
    output.push_str(&format!(
        "- Candidate count: {}\n- Open candidates: {}\n",
        candidate_count, open_count
    ));
    output.push_str("- Trusted fact policy: `candidates are not facts until approved by an existing authority`\n");
    output.push_str(&format!(
        "- Observed automation: {}\n",
        values_or_none(&observed)
    ));
    output.push_str(&format!(
        "- Candidate file: `{}`\n",
        repo_root
            .join("docs/baron/autopilot/CANDIDATES.md")
            .display()
    ));
    output.push_str(&format!(
        "- Vault candidate file: `{}`\n\n",
        vault.project_root.join("Autopilot/CANDIDATES.md").display()
    ));
    for candidate in state.candidates.iter().take(8) {
        output.push_str(&format!(
            "- `{}` status=`{}` scope=`{}` impact=`{}` evidence=`{}` readiness=`{}`\n  summary: {}\n  provenance: {}\n",
            candidate.id,
            candidate.status,
            candidate.scope,
            candidate.impact,
            candidate.evidence_count,
            candidate.readiness,
            candidate.summary,
            values_or_none(&candidate.provenance)
        ));
    }
    output.push_str("\n## Resume Point\n\n");
    output.push_str(&truncate(&continuity, 1_200));
    output.push_str("\n\n## Rules\n\n");
    output.push_str(
        "- Do not infer completion from silence, shutdown, quota exhaustion, or network loss.\n",
    );
    output.push_str("- Treat candidates as untrusted evidence until an explicit approval reaches the existing authority.\n");
    output.push_str("- Runtime-affecting changes require their own explicit managed transaction and approval metadata.\n");
    Ok(output)
}

pub fn render_autopilot_context_summary(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> String {
    match autopilot_status(repo_root, vault) {
        Ok(status) => format!(
            "## Autopilot Learning And Resume\n\n{}\n\n",
            truncate(&status, 1_600)
        ),
        Err(error) => format!(
            "## Autopilot Learning And Resume\n\n- approval state unavailable: {error}; learning remains untrusted\n\n"
        ),
    }
}

/// Existing PreparePacket v1 can surface an approval-needed warning without a
/// schema change. Corrupt optional state is reported as unknown and never
/// treated as permission to promote anything.
pub fn pending_approval_warning(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> Option<String> {
    match candidate_records(repo_root.as_ref(), vault) {
        Ok(candidates) => {
            let pending = candidates
                .iter()
                .filter(|candidate| approval_required(candidate))
                .count();
            (pending > 0).then(|| {
                format!(
                    "{} untrusted Autopilot proposal(s) await explicit project-scoped approval; routing and task truth are unchanged",
                    pending
                )
            })
        }
        Err(_) => {
            let path = repo_root.as_ref().join(STATE_RELATIVE);
            path.exists().then(|| {
                "Autopilot approval state is unreadable; preserve learning as unknown and do not promote it".to_string()
            })
        }
    }
}

fn repo_state_path(repo_root: &Path) -> PathBuf {
    repo_root.join(STATE_RELATIVE)
}

fn vault_state_path(vault: &VaultContext) -> PathBuf {
    vault.project_root.join("Autopilot/STATE.json")
}

fn load_state(repo_root: &Path, vault: &VaultContext) -> Result<AutopilotState> {
    let mut sources = Vec::new();
    for path in [repo_state_path(repo_root), vault_state_path(vault)] {
        if let Some(content) = read_text(&path)? {
            sources.push(parse_state(&content, &path, &vault.project_id)?);
        }
    }
    if sources.is_empty() {
        let mut state = AutopilotState::new(&vault.project_id);
        for path in [
            repo_root.join("docs/baron/autopilot/CANDIDATES.md"),
            vault.project_root.join("Autopilot/CANDIDATES.md"),
        ] {
            if let Some(content) = read_text(&path)? {
                merge_legacy_candidates(
                    &mut state,
                    parse_legacy_candidates(&content, &project_scope(vault)),
                );
            }
        }
        update_readiness(&mut state.candidates);
        return Ok(state);
    }
    let mut merged = AutopilotState::new(&vault.project_id);
    for source in sources {
        merge_state(&mut merged, source)?;
    }
    update_readiness(&mut merged.candidates);
    Ok(merged)
}

fn parse_state(content: &str, path: &Path, project_id: &str) -> Result<AutopilotState> {
    let mut state: AutopilotState = serde_json::from_str(content)
        .with_context(|| format!("Could not parse Autopilot state: {}", path.display()))?;
    if state.schema_version == 0 {
        state.schema_version = AUTOPILOT_SCHEMA_VERSION;
    }
    if state.schema_version > AUTOPILOT_SCHEMA_VERSION {
        bail!(
            "Autopilot state schema {} is newer than supported schema {}",
            state.schema_version,
            AUTOPILOT_SCHEMA_VERSION
        );
    }
    if state.project_id.is_empty() {
        state.project_id = project_id.to_string();
    }
    if state.project_id != project_id {
        bail!("Autopilot state belongs to a different project identity");
    }
    for candidate in &mut state.candidates {
        normalize_candidate(candidate, project_id);
    }
    Ok(state)
}

fn merge_state(target: &mut AutopilotState, source: AutopilotState) -> Result<()> {
    if source.project_id != target.project_id {
        bail!("Autopilot state mirrors disagree about project identity");
    }
    for candidate in source.candidates {
        if let Some(existing) = target
            .candidates
            .iter_mut()
            .find(|item| item.id == candidate.id)
        {
            merge_candidate(existing, candidate);
        } else {
            target.candidates.push(candidate);
        }
    }
    for response in source.responses {
        if target
            .responses
            .iter()
            .all(|item| item.fingerprint != response.fingerprint)
        {
            target.responses.push(response);
        }
    }
    for candidate in source.archived {
        if target.archived.iter().all(|item| item.id != candidate.id) {
            target.archived.push(candidate);
        }
    }
    bound_responses(&mut target.responses);
    bound_candidates(&mut target.archived);
    Ok(())
}

fn merge_legacy_candidates(target: &mut AutopilotState, candidates: Vec<AutopilotCandidate>) {
    for candidate in candidates {
        if let Some(existing) = target.candidates.iter_mut().find(|item| {
            item.scope == candidate.scope && item.proposal_key == candidate.proposal_key
        }) {
            merge_candidate(existing, candidate);
        } else {
            target.candidates.push(candidate);
        }
    }
}

fn merge_candidate(target: &mut AutopilotCandidate, source: AutopilotCandidate) {
    let mut provenance = target.provenance.iter().cloned().collect::<BTreeSet<_>>();
    provenance.extend(source.provenance);
    target.provenance = provenance.into_iter().take(MAX_PROVENANCE).collect();
    let mut evidence = target
        .evidence_keys
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    evidence.extend(source.evidence_keys);
    target.evidence_keys = evidence.into_iter().collect();
    target.evidence_count = target
        .evidence_keys
        .len()
        .max(target.evidence_count)
        .max(source.evidence_count);
    target.observed_automation =
        merge_values(&target.observed_automation, &source.observed_automation);
    target.resume_sources = merge_values(&target.resume_sources, &source.resume_sources);
    if status_rank(&source.status) > status_rank(&target.status) {
        target.status = source.status;
    }
    target.decision_recorded |= source.decision_recorded;
    target.suppressed_until = target.suppressed_until.clone().or(source.suppressed_until);
    target.updated_at = target.updated_at.clone().max(source.updated_at);
    if target.correlation.adapter.is_none() {
        target.correlation.adapter = source.correlation.adapter;
    }
    if target.correlation.session_id.is_none() {
        target.correlation.session_id = source.correlation.session_id;
    }
    if target.correlation.request_id.is_none() {
        target.correlation.request_id = source.correlation.request_id;
    }
    if target.correlation.task_id.is_none() {
        target.correlation.task_id = source.correlation.task_id;
    }
}

fn normalize_candidate(candidate: &mut AutopilotCandidate, project_id: &str) {
    candidate.trusted = false;
    if candidate.scope.is_empty() {
        candidate.scope = format!("project:{project_id}");
    }
    if candidate.summary.is_empty() {
        candidate.summary = "legacy Autopilot candidate".to_string();
    }
    if candidate.status.is_empty() {
        candidate.status = "candidate".to_string();
    }
    if candidate.created_at.is_empty() {
        candidate.created_at = now();
    }
    if candidate.updated_at.is_empty() {
        candidate.updated_at = candidate.created_at.clone();
    }
    if candidate.proposal_key.is_empty() {
        candidate.proposal_key = normalize_proposal(&candidate.summary);
    }
    if candidate.semantic_key.is_empty() {
        candidate.semantic_key = semantic_key(&candidate.proposal_key);
    }
    if candidate.id.is_empty() {
        candidate.id = candidate_id(
            project_id,
            &candidate.scope,
            &candidate.proposal_key,
            &infer_impact(&candidate.summary),
        );
    }
    if candidate.evidence_keys.is_empty() {
        candidate
            .evidence_keys
            .push(hash(&format!("legacy|{}", candidate.id)));
    }
    candidate.evidence_count = candidate
        .evidence_count
        .max(candidate.evidence_keys.len())
        .max(1);
    if candidate.provenance.is_empty() {
        candidate.provenance.push("legacy:autopilot".to_string());
    }
    candidate.readiness = if candidate.readiness.is_empty() {
        "candidate; explicit review required".to_string()
    } else {
        candidate.readiness.clone()
    };
}

fn parse_legacy_candidates(content: &str, scope: &str) -> Vec<AutopilotCandidate> {
    let mut records = Vec::new();
    let mut current: Option<AutopilotCandidate> = None;
    for line in content.lines() {
        if let Some(id) = line.strip_prefix("## ") {
            if let Some(candidate) = current.take() {
                records.push(candidate);
            }
            let id = id.trim().to_string();
            current = Some(AutopilotCandidate {
                id,
                summary: String::new(),
                scope: scope.to_string(),
                impact: "unknown".to_string(),
                status: "candidate".to_string(),
                trusted: false,
                evidence_count: 1,
                provenance: vec!["legacy:autopilot".to_string()],
                observed_automation: Vec::new(),
                resume_sources: Vec::new(),
                contradiction: None,
                readiness: "legacy candidate; explicit review required".to_string(),
                created_at: now(),
                updated_at: now(),
                correlation: AutopilotCorrelation::default(),
                suppressed_until: None,
                replacement_for: None,
                decision_recorded: false,
                proposal_key: String::new(),
                semantic_key: String::new(),
                evidence_keys: Vec::new(),
            });
        } else if let Some(candidate) = current.as_mut() {
            if let Some(value) = line.strip_prefix("- Summary: ") {
                candidate.summary = one_line(value);
                candidate.impact = infer_impact(&candidate.summary);
                candidate.proposal_key = normalize_proposal(&candidate.summary);
                candidate.semantic_key = semantic_key(&candidate.proposal_key);
            } else if let Some(value) = line.strip_prefix("- Status: `") {
                candidate.status = value.trim_end_matches('`').to_string();
            } else if let Some(value) = line.strip_prefix("- Created: ") {
                candidate.created_at = value.trim().to_string();
                candidate.updated_at = candidate.created_at.clone();
            } else if let Some(value) = line.strip_prefix("- Observed automation: ") {
                candidate.observed_automation = split_values(value);
            } else if let Some(value) = line.strip_prefix("- Resume sources: ") {
                candidate.resume_sources = split_values(value);
            }
        }
    }
    if let Some(candidate) = current {
        records.push(candidate);
    }
    for candidate in &mut records {
        candidate.evidence_keys = vec![hash(&format!("legacy|{}", candidate.id))];
        normalize_candidate(candidate, "legacy");
    }
    records
}

fn save_state(repo_root: &Path, vault: &VaultContext, state: &AutopilotState) -> Result<()> {
    let mut state = state.clone();
    state.schema_version = AUTOPILOT_SCHEMA_VERSION;
    state.project_id = vault.project_id.clone();
    state.candidates.truncate(MAX_CANDIDATES);
    bound_responses(&mut state.responses);
    bound_candidates(&mut state.archived);
    let json = format!("{}\n", serde_json::to_string_pretty(&state)?);
    replace_text(repo_state_path(repo_root), &json)?;
    replace_text(vault_state_path(vault), &json)?;
    let rendered_candidates = state
        .candidates
        .iter()
        .chain(state.archived.iter())
        .cloned()
        .collect::<Vec<_>>();
    let candidates = render_candidates(&rendered_candidates);
    replace_text(
        repo_root.join("docs/baron/autopilot/CANDIDATES.md"),
        &candidates,
    )?;
    replace_text(
        vault.project_root.join("Autopilot/CANDIDATES.md"),
        &candidates,
    )?;
    let approved = render_approved(&rendered_candidates);
    replace_text(
        repo_root.join("docs/baron/autopilot/APPROVED.md"),
        &approved,
    )?;
    replace_text(vault.project_root.join("Autopilot/APPROVED.md"), &approved)
}

fn render_candidates(candidates: &[AutopilotCandidate]) -> String {
    let mut output = String::from(
        "---\nconfidence: candidate\nstatus: candidate\ntags: [autopilot, candidate, untrusted]\n---\n# Baron Autopilot Learning Candidates\n\nCandidates are untrusted evidence. They require explicit approval through the existing authority before any durable behavior change.\n\n",
    );
    for candidate in candidates {
        output.push_str(&format!(
            "## {}\n\n- Status: `{}`\n- Trusted fact: `no`\n- Approval required: `{}`\n- Created: {}\n- Updated: {}\n- Summary: {}\n- Scope: `{}`\n- Impact: `{}`\n- Evidence count: `{}`\n- Readiness: {}\n- Provenance: {}\n- Observed automation: {}\n- Resume sources: {}\n- Contradiction: {}\n- Suppressed until: {}\n- Replacement for: {}\n- Decision recorded: `{}`\n- Safe action: preserve as untrusted evidence; do not rewrite skills, agents, memory facts, routing, proof policy, or runtime assets from this item alone.\n\n",
            candidate.id,
            candidate.status,
            if approval_required(candidate) { "yes" } else { "no" },
            candidate.created_at,
            candidate.updated_at,
            candidate.summary,
            candidate.scope,
            candidate.impact,
            candidate.evidence_count,
            candidate.readiness,
            values_or_none(&candidate.provenance),
            values_or_none(&candidate.observed_automation),
            values_or_none(&candidate.resume_sources),
            candidate.contradiction.as_deref().unwrap_or("none"),
            candidate.suppressed_until.as_deref().unwrap_or("none"),
            candidate.replacement_for.as_deref().unwrap_or("none"),
            if candidate.decision_recorded { "yes" } else { "no" },
        ));
    }
    output
}

fn render_approved(candidates: &[AutopilotCandidate]) -> String {
    let mut output = String::from(
        "---\nconfidence: candidate\nstatus: candidate\ntags: [autopilot, approval-evidence, untrusted]\n---\n# Baron Approved Autopilot Learning\n\nApproval evidence remains separate from facts. Only the existing destination authority can make a durable project record current.\n\n",
    );
    for candidate in candidates.iter().filter(|candidate| {
        matches!(
            candidate.status.as_str(),
            "approved" | "rejected" | "deferred" | "superseded"
        )
    }) {
        output.push_str(&format!(
            "## {}\n\n- Status: `{}`\n- Approved candidate: `{}`\n- Scope: `{}`\n- Trusted fact: `no`\n- Decision authority: `{}`\n- Summary: {}\n\n",
            candidate.id,
            candidate.status,
            candidate.id,
            candidate.scope,
            if candidate.decision_recorded { "project Product Harness decision" } else { "none" },
            candidate.summary
        ));
    }
    output
}

#[allow(clippy::too_many_arguments)]
fn new_candidate(
    id: String,
    summary: String,
    scope: String,
    impact: String,
    proposal_key: String,
    semantic_key: String,
    observation_key: String,
    observed_automation: Vec<String>,
    resume_sources: Vec<String>,
    correlation: AutopilotCorrelation,
    now: &str,
) -> AutopilotCandidate {
    let mut provenance = vec![format!("task-review:{}", hash(&summary))];
    provenance.push("source:user-task".to_string());
    provenance.push("source:repo-review".to_string());
    provenance.push("review:post-task".to_string());
    if let Some(adapter) = correlation.adapter.as_deref() {
        provenance.push(format!("adapter:{adapter}"));
    }
    provenance.extend(
        observed_automation
            .iter()
            .map(|event| format!("automation:{event}")),
    );
    provenance.extend(
        resume_sources
            .iter()
            .map(|source| format!("resume:{source}")),
    );
    provenance.sort();
    provenance.dedup();
    AutopilotCandidate {
        id,
        summary,
        scope,
        impact,
        status: "candidate".to_string(),
        trusted: false,
        evidence_count: 1,
        provenance,
        observed_automation,
        resume_sources,
        contradiction: None,
        readiness: "candidate; one observation requires explicit review".to_string(),
        created_at: now.to_string(),
        updated_at: now.to_string(),
        correlation,
        suppressed_until: None,
        replacement_for: None,
        decision_recorded: false,
        proposal_key,
        semantic_key,
        evidence_keys: vec![observation_key],
    }
}

fn approve_state_candidate(
    repo_root: &Path,
    vault: &VaultContext,
    candidate: &mut AutopilotCandidate,
) -> Result<AutopilotApproval> {
    if candidate.status == "superseded" {
        bail!("Candidate was superseded by a correction or explicit intent")
    }
    if explicit_intent_conflicts(repo_root, candidate) {
        candidate.status = "superseded".to_string();
        candidate.contradiction = Some("current explicit intent is authoritative".to_string());
        candidate.updated_at = now();
        return Ok(AutopilotApproval {
            candidate_id: candidate.id.clone(),
            status: candidate.status.clone(),
            changed: true,
            promotion: "none; current explicit intent remains authoritative".to_string(),
            message: "The current explicit intent supersedes this older proposal.".to_string(),
            child_promoted: false,
        });
    }
    let marker = format!("Autopilot approval: {}", candidate.summary);
    if !decision_exists(repo_root, vault, &marker) {
        record_decision(repo_root, vault, &marker)?;
    }
    candidate.status = "approved".to_string();
    candidate.trusted = false;
    candidate.decision_recorded = true;
    candidate
        .provenance
        .push("source:user-approval".to_string());
    candidate.provenance.sort();
    candidate.provenance.dedup();
    candidate.provenance.truncate(MAX_PROVENANCE);
    candidate.suppressed_until = None;
    candidate.updated_at = now();
    candidate.readiness =
        "approved through project decision authority; candidate remains non-trusted evidence"
            .to_string();
    Ok(AutopilotApproval {
        candidate_id: candidate.id.clone(),
        status: candidate.status.clone(),
        changed: true,
        promotion: "project decision authority recorded in Product Harness; no runtime or Core asset mutation".to_string(),
        message: "Approved with an explicit project-scoped decision; no automatic runtime promotion occurred.".to_string(),
        child_promoted: false,
    })
}

fn create_correction(
    vault: &VaultContext,
    candidate: &mut AutopilotCandidate,
    corrected: &str,
    state: &mut AutopilotState,
) -> Result<String> {
    let corrected = one_line(corrected);
    if corrected.is_empty() {
        bail!("Correction must include the replacement proposal")
    }
    let proposal_key = normalize_proposal(&corrected);
    if proposal_key == candidate.proposal_key {
        bail!("Correction must change the proposal or its scope")
    }
    let impact = infer_impact(&corrected);
    let scope = candidate.scope.clone();
    let semantic = semantic_key(&proposal_key);
    let id = candidate_id(&vault.project_id, &scope, &proposal_key, &impact);
    let original_id = candidate.id.clone();
    candidate.status = "superseded".to_string();
    candidate.suppressed_until = Some("never".to_string());
    candidate.updated_at = now();
    candidate.readiness =
        "superseded by an explicit correction; historical evidence retained".to_string();
    if let Some(existing) = state.candidates.iter_mut().find(|item| item.id == id) {
        existing.status = "candidate".to_string();
        existing.suppressed_until = None;
        existing.replacement_for = Some(original_id);
        existing.updated_at = now();
        return Ok(existing.id.clone());
    }
    let now = now();
    let mut replacement = new_candidate(
        id.clone(),
        corrected.clone(),
        scope,
        impact,
        proposal_key,
        semantic,
        hash(&format!("correction|{}|{}", original_id, corrected)),
        candidate.observed_automation.clone(),
        candidate.resume_sources.clone(),
        candidate.correlation.clone(),
        &now,
    );
    replacement.replacement_for = Some(original_id.clone());
    replacement
        .provenance
        .push(format!("correction-of:{original_id}"));
    state.candidates.push(replacement);
    Ok(id)
}

fn resolve_pending_candidate(
    candidates: &[AutopilotCandidate],
    operation: Option<&OperationContext>,
    action: &ApprovalAction,
) -> Option<usize> {
    let mut matching = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            approval_required(candidate)
                || matches!(action, ApprovalAction::Correct(_))
                    && matches!(
                        candidate.status.as_str(),
                        "deferred" | "rejected" | "conflict"
                    )
        })
        .filter(|(_, candidate)| {
            operation.is_none_or(|operation| correlation_matches(&candidate.correlation, operation))
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matching.len() == 1 {
        matching.pop()
    } else {
        // A natural-language response is safe only when exactly one project
        // candidate is correlated. Never guess between unrelated proposals.
        None
    }
}

fn parse_approval_response(response: &str) -> Result<ApprovalAction> {
    let normalized = one_line(response).to_lowercase();
    if normalized.is_empty() {
        bail!("Approval response is empty")
    }
    if normalized.starts_with("change ")
        || normalized.starts_with("correct ")
        || normalized.starts_with("instead ")
        || normalized.starts_with("only ")
        || normalized.contains("not always")
    {
        let corrected = normalized
            .trim_start_matches("change it to ")
            .trim_start_matches("change to ")
            .trim_start_matches("correct it to ")
            .trim_start_matches("correct to ")
            .trim_start_matches("instead ")
            .trim_start_matches("only ")
            .trim_start_matches("not always ")
            .trim();
        return Ok(ApprovalAction::Correct(corrected.to_string()));
    }
    if normalized.contains("not now")
        || normalized.contains("later")
        || normalized.starts_with("defer")
        || normalized.starts_with("ask me later")
    {
        return Ok(ApprovalAction::Defer);
    }
    if normalized == "yes"
        || normalized == "y"
        || normalized.starts_with("yes ")
        || normalized.starts_with("yes,")
        || normalized.starts_with("yes;")
        || normalized.starts_with("approve")
        || normalized.starts_with("do that")
        || normalized.starts_with("make that")
        || normalized.starts_with("make it the rule")
    {
        return Ok(ApprovalAction::Approve);
    }
    if normalized == "no"
        || normalized == "n"
        || normalized.starts_with("no ")
        || normalized.starts_with("no,")
        || normalized.starts_with("no;")
        || normalized.starts_with("reject")
        || normalized.starts_with("never")
    {
        return Ok(ApprovalAction::Reject);
    }
    bail!("Approval response is ambiguous; answer yes, no, not now, or provide a correction")
}

fn correlation(
    operation: Option<&OperationContext>,
    summary: &str,
    vault: &VaultContext,
) -> AutopilotCorrelation {
    AutopilotCorrelation {
        adapter: operation.map(|item| item.adapter.as_str().to_string()),
        session_id: operation.and_then(|item| item.session_id.clone()),
        request_id: operation.and_then(|item| item.request_id.clone()),
        task_id: Some(hash(&format!(
            "{}|{}",
            vault.project_id,
            normalize_proposal(summary)
        ))),
    }
}

fn correlation_matches(candidate: &AutopilotCorrelation, operation: &OperationContext) -> bool {
    let session_matches = candidate
        .session_id
        .as_ref()
        .zip(operation.session_id.as_ref())
        .map(|(left, right)| left == right)
        .unwrap_or(true);
    let request_matches = candidate
        .request_id
        .as_ref()
        .zip(operation.request_id.as_ref())
        .map(|(left, right)| left == right)
        .unwrap_or(true);
    session_matches && request_matches
}

fn apply_explicit_intent_authority(repo_root: &Path, candidate: &mut AutopilotCandidate) -> bool {
    if explicit_intent_conflicts(repo_root, candidate) && candidate.status != "approved" {
        candidate.status = "superseded".to_string();
        candidate.contradiction = Some("current explicit intent is authoritative".to_string());
        candidate.suppressed_until = Some("never".to_string());
        candidate.updated_at = now();
        return true;
    }
    false
}

fn explicit_intent_conflicts(repo_root: &Path, candidate: &AutopilotCandidate) -> bool {
    let source = match intent_status(repo_root) {
        Ok(source) => source,
        Err(_) => return false,
    };
    let intent = normalize_proposal(&source);
    if intent.is_empty() || !intent.contains(&candidate.semantic_key) {
        return false;
    }
    let candidate_positive = positive_polarity(&candidate.proposal_key);
    let intent_positive = positive_polarity(&intent);
    candidate_positive.is_some()
        && intent_positive.is_some()
        && candidate_positive != intent_positive
}

fn positive_polarity(value: &str) -> Option<bool> {
    let tokens = value.split_whitespace().collect::<Vec<_>>();
    if tokens
        .iter()
        .any(|token| matches!(*token, "never" | "avoid" | "not" | "without"))
    {
        Some(false)
    } else if tokens
        .iter()
        .any(|token| matches!(*token, "always" | "prefer" | "use" | "adopt"))
    {
        Some(true)
    } else {
        None
    }
}

fn refresh_conflicts(candidates: &mut [AutopilotCandidate]) -> bool {
    let mut groups = BTreeMap::<(String, String), BTreeSet<String>>::new();
    for candidate in candidates.iter() {
        if matches!(
            candidate.status.as_str(),
            "approved" | "superseded" | "expired"
        ) {
            continue;
        }
        groups
            .entry((candidate.scope.clone(), candidate.semantic_key.clone()))
            .or_default()
            .insert(candidate.proposal_key.clone());
    }
    let mut changed = false;
    for candidate in candidates.iter_mut() {
        let key = (candidate.scope.clone(), candidate.semantic_key.clone());
        let conflict = groups.get(&key).is_some_and(|values| values.len() > 1);
        if conflict {
            if candidate.status != "conflict" {
                candidate.status = "conflict".to_string();
                changed = true;
            }
            candidate.contradiction =
                Some("equivalent scope has contradictory proposals".to_string());
        } else if candidate.status == "conflict" {
            candidate.status = if candidate.evidence_count > 1 {
                "ready"
            } else {
                "candidate"
            }
            .to_string();
            candidate.contradiction = None;
            changed = true;
        }
    }
    changed
}

fn update_readiness(candidates: &mut [AutopilotCandidate]) {
    for candidate in candidates {
        candidate.trusted = false;
        candidate.evidence_count = candidate
            .evidence_count
            .max(candidate.evidence_keys.len())
            .max(1);
        if candidate.status == "candidate" && candidate.evidence_count > 1 {
            candidate.status = "ready".to_string();
        }
        candidate.readiness = match candidate.status.as_str() {
            "ready" => {
                "ready for explicit review; repeated evidence is still untrusted".to_string()
            }
            "approved" => {
                "approved through an existing authority; candidate remains non-trusted evidence"
                    .to_string()
            }
            "rejected" => "rejected; repeat prompts suppressed".to_string(),
            "deferred" => "deferred; bounded cooldown before another prompt".to_string(),
            "conflict" => {
                "blocked by contradictory evidence; correction or authoritative decision required"
                    .to_string()
            }
            "superseded" => "superseded; retained for provenance only".to_string(),
            "expired" => "expired weak evidence; retained for audit only".to_string(),
            _ => "candidate; explicit review required".to_string(),
        };
    }
}

fn housekeeping_state(state: &mut AutopilotState) -> bool {
    let before = state.candidates.clone();
    let report = housekeeping_state_report(state);
    report.deduplicated > 0
        || report.expired > 0
        || report.suppression_reopened > 0
        || report.pruned > 0
        || before != state.candidates
}

fn housekeeping_state_report(state: &mut AutopilotState) -> AutopilotHousekeeping {
    let mut report = AutopilotHousekeeping {
        deduplicated: 0,
        expired: 0,
        suppression_reopened: 0,
        pruned: 0,
        pending: 0,
    };
    let mut merged: Vec<AutopilotCandidate> = Vec::new();
    for candidate in state.candidates.drain(..) {
        if let Some(existing) = merged.iter_mut().find(|item| {
            item.scope == candidate.scope && item.proposal_key == candidate.proposal_key
        }) {
            merge_candidate(existing, candidate);
            report.deduplicated += 1;
        } else {
            merged.push(candidate);
        }
    }
    state.candidates = merged;
    let current = Utc::now();
    for candidate in &mut state.candidates {
        if candidate.status == "deferred" {
            if let Some(until) = candidate.suppressed_until.as_deref().and_then(parse_time) {
                if until <= current {
                    candidate.status = "candidate".to_string();
                    candidate.suppressed_until = None;
                    candidate.updated_at = now();
                    report.suppression_reopened += 1;
                }
            }
        }
        if matches!(candidate.status.as_str(), "candidate" | "ready")
            && candidate.evidence_count <= 1
            && parse_time(&candidate.updated_at).is_some_and(|updated| {
                current.signed_duration_since(updated).num_days() >= STALE_CANDIDATE_DAYS
            })
        {
            candidate.status = "expired".to_string();
            candidate.updated_at = now();
            report.expired += 1;
        }
    }
    let mut active = Vec::with_capacity(state.candidates.len());
    for candidate in state.candidates.drain(..) {
        if matches!(
            candidate.status.as_str(),
            "approved" | "rejected" | "superseded" | "expired"
        ) {
            state.archived.push(candidate);
            report.pruned += 1;
        } else {
            active.push(candidate);
        }
    }
    state.candidates = active;
    bound_candidates(&mut state.archived);
    report.pending = state
        .candidates
        .iter()
        .filter(|candidate| approval_required(candidate))
        .count();
    update_readiness(&mut state.candidates);
    report
}

fn approval_required(candidate: &AutopilotCandidate) -> bool {
    matches!(
        candidate.status.as_str(),
        "candidate" | "ready" | "conflict"
    ) && !is_suppressed(candidate)
}

fn is_suppressed(candidate: &AutopilotCandidate) -> bool {
    match candidate.suppressed_until.as_deref() {
        None => false,
        Some("never") => true,
        Some(value) => parse_time(value).is_none_or(|until| until > Utc::now()),
    }
}

fn is_permanently_suppressed(candidate: &AutopilotCandidate) -> bool {
    matches!(candidate.suppressed_until.as_deref(), Some("never"))
}

fn decision_exists(repo_root: &Path, vault: &VaultContext, marker: &str) -> bool {
    [
        repo_root.join("docs/baron/harness/DECISIONS.md"),
        vault.project_root.join("ProductHarness/DECISIONS.md"),
    ]
    .iter()
    .filter_map(|path| fs::read_to_string(path).ok())
    .any(|content| content.contains(marker))
}

fn response_fingerprint(candidate_id: &str, response: &str, scope: &str) -> String {
    hash(&format!(
        "{scope}|{candidate_id}|{}",
        normalize_proposal(response)
    ))
}

fn bound_responses(responses: &mut Vec<AutopilotResponseRecord>) {
    if responses.len() > MAX_RESPONSE_RECORDS {
        let remove = responses.len() - MAX_RESPONSE_RECORDS;
        responses.drain(..remove);
    }
}

fn bound_candidates(candidates: &mut Vec<AutopilotCandidate>) {
    if candidates.len() > MAX_CANDIDATES {
        let remove = candidates.len() - MAX_CANDIDATES;
        candidates.drain(..remove);
    }
}

fn find_previous_response<'a>(
    state: &'a AutopilotState,
    response: &str,
    scope: &str,
) -> Option<&'a AutopilotResponseRecord> {
    state.responses.iter().find(|record| {
        state
            .candidates
            .iter()
            .chain(state.archived.iter())
            .any(|candidate| {
                response_fingerprint(&candidate.id, response, scope) == record.fingerprint
            })
    })
}

fn candidate_id(project_id: &str, scope: &str, proposal: &str, impact: &str) -> String {
    format!(
        "candidate-{}-{}",
        &hash(&format!("{project_id}|{scope}|{proposal}"))[..16],
        impact_slug(impact)
    )
}

fn impact_slug(impact: &str) -> &'static str {
    match impact {
        "workflow" => "workflow",
        "routing" => "routing",
        "verification" => "verification",
        "runtime_asset" => "runtime",
        "memory" => "memory",
        "project_decision" => "decision",
        _ => "unknown",
    }
}

fn infer_impact(summary: &str) -> String {
    let lower = summary.to_lowercase();
    if contains_any(&lower, &["skill", "agent", "workflow", "instruction"]) {
        "workflow".to_string()
    } else if contains_any(&lower, &["routing", "route", "profile"]) {
        "routing".to_string()
    } else if contains_any(&lower, &["proof", "trace", "verification", "test", "gate"]) {
        "verification".to_string()
    } else if contains_any(&lower, &["runtime", "asset", "hook", "managed"]) {
        "runtime_asset".to_string()
    } else if contains_any(&lower, &["memory", "remember", "recall", "fact"]) {
        "memory".to_string()
    } else {
        "project_decision".to_string()
    }
}

fn project_scope(vault: &VaultContext) -> String {
    format!("project:{}", vault.project_id)
}

fn observation_key(
    scope: &str,
    summary: &str,
    correlation: &AutopilotCorrelation,
    observed: &[String],
) -> String {
    hash(&format!(
        "{scope}|{}|{}|{}|{}|{}",
        summary,
        correlation.adapter.as_deref().unwrap_or(""),
        correlation.session_id.as_deref().unwrap_or(""),
        correlation.request_id.as_deref().unwrap_or(""),
        observed.join(",")
    ))
}

fn merge_observation(
    candidate: &mut AutopilotCandidate,
    observed: &[String],
    resume_sources: &[String],
    correlation: &AutopilotCorrelation,
) {
    candidate
        .provenance
        .extend(observed.iter().map(|event| format!("automation:{event}")));
    candidate.provenance.extend(
        resume_sources
            .iter()
            .map(|source| format!("resume:{source}")),
    );
    if let Some(adapter) = correlation.adapter.as_deref() {
        candidate.provenance.push(format!("adapter:{adapter}"));
    }
    if candidate.evidence_count > 1 {
        candidate.provenance.push("repeated-behavior".to_string());
    }
    candidate.provenance.sort();
    candidate.provenance.dedup();
    candidate.provenance.truncate(MAX_PROVENANCE);
    candidate.observed_automation = merge_values(&candidate.observed_automation, observed);
    candidate.resume_sources = merge_values(&candidate.resume_sources, resume_sources);
    candidate.correlation.adapter = candidate
        .correlation
        .adapter
        .clone()
        .or_else(|| correlation.adapter.clone());
}

fn normalize_proposal(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character.is_whitespace() || character == '_' {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn semantic_key(proposal: &str) -> String {
    proposal
        .split_whitespace()
        .filter(|token| {
            !matches!(
                *token,
                "always"
                    | "never"
                    | "prefer"
                    | "avoid"
                    | "use"
                    | "adopt"
                    | "do"
                    | "not"
                    | "without"
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn status_rank(status: &str) -> u8 {
    match status {
        "approved" => 8,
        "rejected" => 7,
        "superseded" => 6,
        "deferred" => 5,
        "conflict" => 4,
        "ready" => 3,
        "candidate" => 2,
        "expired" => 1,
        _ => 0,
    }
}

fn observed_automation(vault: &VaultContext) -> Vec<String> {
    let path = vault
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    let mut events = fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|value| {
            value
                .get("event_kind")
                .or_else(|| value.get("event"))
                .and_then(Value::as_str)
                .map(pretty_event)
        })
        .collect::<Vec<_>>();
    events.sort();
    events.dedup();
    events
}

fn pretty_event(event: &str) -> String {
    event
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn resume_sources(repo_root: &Path) -> Vec<String> {
    [
        "docs/baron/continuity/CURRENT.md",
        "docs/baron/continuity/CURRENT_RECOVERY.md",
        "docs/baron/plans/CURRENT.md",
        "docs/baron/harness/CURRENT.md",
        "docs/baron/harness/CURRENT_INTENT.md",
        "docs/baron/proofs/INDEX.md",
        "docs/baron/traces/INDEX.md",
    ]
    .iter()
    .filter(|relative| repo_root.join(relative).exists())
    .map(|relative| (*relative).to_string())
    .collect()
}

fn merge_values(left: &[String], right: &[String]) -> Vec<String> {
    let values = left
        .iter()
        .chain(right.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    values.into_iter().take(MAX_PROVENANCE).collect()
}

fn split_values(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.is_empty() || value == "none" {
        Vec::new()
    } else {
        value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect()
    }
}

fn values_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn one_line(value: &str) -> String {
    value
        .replace(['\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn hash(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn parse_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn schema_version() -> u32 {
    AUTOPILOT_SCHEMA_VERSION
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
