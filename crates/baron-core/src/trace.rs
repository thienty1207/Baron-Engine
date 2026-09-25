use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use anyhow::{bail, Context, Result};
use chrono::{Local, NaiveDate, SecondsFormat, TimeZone};
use serde::{Deserialize, Serialize};

use crate::control_plane::gate_evidence_status_strict_for_operation;
use crate::harness::{current_harness_risk, current_harness_title};
use crate::operation::OperationContext;
use crate::plan::{active_plan_authority, ActivePlanAuthority};
use crate::proof::{
    latest_proof, proof_by_id, proof_has_current_receipt, proof_operation_binding,
    proof_satisfies_risk, ProofRecord,
};
use crate::risk::RiskLane;
use crate::safe_io::{
    acquire_project_lock, append_text, artifact_instance_id, create_new_text, read_text,
    replace_text,
};
use crate::vault::VaultContext;

const SCORE_START: &str = "<!-- BARON:TRACE-SCORE:START -->";
const SCORE_END: &str = "<!-- BARON:TRACE-SCORE:END -->";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceOutcome {
    Completed,
    Partial,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceTier {
    Incomplete,
    Minimal,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceRecord {
    pub id: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub proof_id: Option<String>,
    pub binding: Option<TraceOperationBinding>,
}

/// Exact operation identity carried by a correctness-sensitive trace. The
/// proof ID is part of the binding so a trace cannot silently use a newer or
/// unrelated proof for the same repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceOperationBinding {
    pub task_id: String,
    pub operation_id: String,
    pub adapter: String,
    pub session_id: String,
    pub request_id: String,
    pub proof_id: String,
}

impl TraceOperationBinding {
    pub fn from_operation(operation: &OperationContext, proof_id: &str) -> Result<Self> {
        let binding = Self {
            task_id: operation.task_id.clone().unwrap_or_default(),
            operation_id: operation.operation_id.clone().unwrap_or_default(),
            adapter: operation.adapter.as_str().to_string(),
            session_id: operation.session_id.clone().unwrap_or_default(),
            request_id: operation.request_id.clone().unwrap_or_default(),
            proof_id: proof_id.trim().to_string(),
        };
        binding.validate()?;
        Ok(binding)
    }

    fn validate(&self) -> Result<()> {
        for (field, value) in [
            ("task_id", self.task_id.as_str()),
            ("operation_id", self.operation_id.as_str()),
            ("adapter", self.adapter.as_str()),
            ("session_id", self.session_id.as_str()),
            ("request_id", self.request_id.as_str()),
            ("proof_id", self.proof_id.as_str()),
        ] {
            if value.trim().is_empty() {
                bail!("trace operation binding field `{field}` is missing");
            }
        }
        Ok(())
    }

    fn matches_identity(&self, other: &Self) -> bool {
        self.task_id == other.task_id
            && self.operation_id == other.operation_id
            && self.adapter == other.adapter
            && self.session_id == other.session_id
            && self.request_id == other.request_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceScore {
    pub trace_id: String,
    pub achieved: TraceTier,
    pub required: TraceTier,
    pub passed: bool,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub proof_id: Option<String>,
    pub binding: Option<TraceOperationBinding>,
}

pub fn record_trace(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
    outcome: TraceOutcome,
) -> Result<TraceRecord> {
    record_trace_internal(
        repo_root.as_ref(),
        vault,
        summary,
        outcome,
        None,
        None,
        None,
    )
}

/// Record a trace bound to one exact proof and operation. This path never
/// consults repository-global newest-proof selection.
pub fn record_trace_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
    outcome: TraceOutcome,
    binding: &TraceOperationBinding,
) -> Result<TraceRecord> {
    binding.validate()?;
    let repo_root = repo_root.as_ref();
    let proof = proof_by_id(repo_root, &binding.proof_id)?
        .context("operation-bound trace proof is missing")?;
    let proof_binding = proof_operation_binding(&proof)
        .context("operation-bound trace proof has no complete operation binding")?;
    if proof_binding.gate_kind.trim() != "proof"
        || proof_binding.task_id != binding.task_id
        || proof_binding.operation_id != binding.operation_id
        || proof_binding.adapter != binding.adapter
        || proof_binding.session_id != binding.session_id
        || proof_binding.request_id != binding.request_id
    {
        bail!("operation-bound trace proof binding does not match the trace operation");
    }
    let plan_authority = active_plan_authority(repo_root)?;
    if plan_authority.is_none() && repo_root.join("docs/baron/plans/CURRENT.md").exists() {
        bail!("operation-bound trace requires a validated active plan");
    }
    let risk = plan_authority
        .as_ref()
        .map(|authority| authority.risk)
        .unwrap_or_else(|| current_plan_risk(repo_root));
    if risk != RiskLane::Low && !proof_has_current_receipt(repo_root, &proof)? {
        bail!("operation-bound trace requires a current trusted receipt for medium/high risk");
    }
    if !proof_satisfies_risk(&proof.summary, risk) {
        bail!("operation-bound trace proof does not satisfy plan risk requirements");
    }
    record_trace_internal(
        repo_root,
        vault,
        summary,
        outcome,
        Some(binding),
        Some(&proof),
        plan_authority.as_ref(),
    )
}

fn record_trace_internal(
    repo_root: &Path,
    vault: &VaultContext,
    summary: &str,
    outcome: TraceOutcome,
    binding: Option<&TraceOperationBinding>,
    bound_proof: Option<&ProofRecord>,
    plan_authority: Option<&ActivePlanAuthority>,
) -> Result<TraceRecord> {
    // Git status is diagnostic trace context and may spawn a child process;
    // collect it before entering the project mutation critical section.
    let files = changed_files(repo_root);
    let _lock = acquire_project_lock(repo_root)?;
    if let Some(binding) = binding {
        let current_proof = proof_by_id(repo_root, &binding.proof_id)?
            .context("operation-bound trace proof disappeared before publication")?;
        if Some(&current_proof) != bound_proof {
            bail!(
                "operation-bound trace proof changed during validation; refusing stale trace publication"
            );
        }
        let current_plan = active_plan_authority(repo_root)?;
        if current_plan.as_ref() != plan_authority {
            bail!(
                "operation-bound trace plan authority changed during validation; refusing stale trace publication"
            );
        }
        if let Some(proof) = bound_proof {
            if proof.receipt_id.is_some() && !proof_has_current_receipt(repo_root, proof)? {
                bail!(
                    "operation-bound trace proof receipt became stale during validation; refusing trace publication"
                );
            }
        }
    }
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let id = artifact_instance_id(&date)?;
    let risk = if binding.is_some() {
        plan_authority
            .map(|authority| authority.risk)
            .unwrap_or_else(|| current_plan_risk(repo_root))
    } else if repo_root.join("docs/baron/harness/CURRENT.md").exists() {
        current_harness_risk(repo_root)
    } else {
        current_plan_risk(repo_root)
    };
    let story = current_harness_title(repo_root);
    let plan = if binding.is_some() {
        plan_authority
            .map(|authority| authority.title.clone())
            .or_else(|| current_plan_title(repo_root))
    } else {
        current_plan_title(repo_root)
    };
    let proof = match bound_proof {
        Some(proof) => Some(proof.clone()),
        None => latest_proof(repo_root)?,
    };
    let repo_path = repo_root
        .join("docs/baron/traces")
        .join(&date)
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("Traces")
        .join(&date)
        .join(format!("{id}.md"));
    let content = render_trace(TraceView {
        id: &id,
        summary,
        outcome,
        risk,
        plan: plan.as_deref(),
        story: story.as_deref(),
        proof: proof.as_ref().map(|value| value.summary.as_str()),
        proof_id: proof.as_ref().map(|value| value.id.as_str()),
        binding,
        capability_gate_passed: proof
            .as_ref()
            .map(|value| value.capability_gate_passed)
            .unwrap_or(true),
        capability_warnings: proof
            .as_ref()
            .map(|value| value.capability_warnings.as_slice())
            .unwrap_or(&[]),
        files: &files,
    });
    create_new_text(&repo_path, &content)?;
    create_new_text(&vault_path, &content)?;
    append(
        &repo_root.join("docs/baron/traces/INDEX.md"),
        "# Baron Trace Index\n\n",
        &format!("- `{id}` - {} - {}", outcome.as_str(), summary.trim()),
    )?;
    append(
        &vault.project_root.join("Traces/INDEX.md"),
        "# Baron Trace Index\n\n",
        &format!("- `{id}` - {} - {}", outcome.as_str(), summary.trim()),
    )?;
    Ok(TraceRecord {
        id,
        repo_path,
        vault_path,
        proof_id: proof.map(|value| value.id),
        binding: binding.cloned(),
    })
}

pub fn score_trace(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    trace_id: Option<&str>,
) -> Result<TraceScore> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let repo_path = find_trace(repo_root, trace_id)?;
    let content = fs::read_to_string(&repo_path)?;
    let score = evaluate_trace_score(repo_root, &content)?;
    let updated = replace_score(&content, &score);
    write(&repo_path, &updated)?;
    let relative = repo_path
        .strip_prefix(repo_root.join("docs/baron/traces"))
        .unwrap_or(&repo_path);
    let vault_path = vault.project_root.join("Traces").join(relative);
    write(&vault_path, &updated)?;
    let outcome = trace_field(&content, "- Outcome: `").unwrap_or_else(|| "unknown".to_string());
    let summary = trace_summary(&content);
    update_trace_index(
        &repo_root.join("docs/baron/traces/INDEX.md"),
        &score.trace_id,
        &outcome,
        &summary,
        &score,
    )?;
    update_trace_index(
        &vault.project_root.join("Traces/INDEX.md"),
        &score.trace_id,
        &outcome,
        &summary,
        &score,
    )?;
    Ok(score)
}

/// Recompute trace authority from the current trace, proof, receipt, gate, and
/// capability state. The persisted score block is intentionally not read here;
/// it is a display/history cache written by [`score_trace`].
fn evaluate_trace_score(repo_root: &Path, content: &str) -> Result<TraceScore> {
    let risk = parse_risk(content);
    let trace_id = trace_field(content, "- Trace ID: `").unwrap_or_else(|| "unknown".to_string());
    let binding = parse_trace_binding(content)?;
    let proof_id = trace_field(content, "- Proof ID: `");
    let mut missing = Vec::new();
    if !content.contains("## Task Summary\n\n") || content.contains("## Task Summary\n\n\n") {
        missing.push("task summary".to_string());
    }
    if content.contains("- Outcome: `unknown`") {
        missing.push("outcome".to_string());
    }
    let has_plan = !content.contains("- Current plan: `missing`");
    let has_story = !content.contains("- Current story: `missing`");
    let bound_proof = if let Some(binding) = binding.as_ref() {
        match proof_id.as_deref() {
            Some(proof_id) => match proof_by_id(repo_root, proof_id)? {
                Some(proof) => {
                    let matches = proof_operation_binding(&proof)
                        .map(|proof_binding| {
                            proof_binding.gate_kind.trim() == "proof"
                                && proof_binding.task_id == binding.task_id
                                && proof_binding.operation_id == binding.operation_id
                                && proof_binding.adapter == binding.adapter
                                && proof_binding.session_id == binding.session_id
                                && proof_binding.request_id == binding.request_id
                        })
                        .unwrap_or(false);
                    if !matches || proof.id != binding.proof_id {
                        missing.push("bound proof binding".to_string());
                        None
                    } else {
                        Some(proof)
                    }
                }
                None => {
                    missing.push("bound proof".to_string());
                    None
                }
            },
            None => {
                missing.push("bound proof".to_string());
                None
            }
        }
    } else {
        None
    };
    let has_proof = if binding.is_some() {
        bound_proof.is_some()
    } else {
        !content.contains("- Proof: `missing`")
    };
    let has_files = content
        .split("## Files Changed")
        .nth(1)
        .map(|value| value.lines().any(|line| line.starts_with("- `")))
        .unwrap_or(false);

    let achieved = if missing.is_empty() && has_plan && has_story && has_proof && has_files {
        TraceTier::Detailed
    } else if missing.is_empty() && has_plan && has_proof {
        TraceTier::Standard
    } else if missing.is_empty() {
        TraceTier::Minimal
    } else {
        TraceTier::Incomplete
    };
    let required = required_tier(risk);
    if required >= TraceTier::Standard && !has_plan {
        missing.push("current plan".to_string());
    }
    if required >= TraceTier::Standard && !has_proof {
        missing.push("proof".to_string());
    }
    if required == TraceTier::Detailed && !has_story {
        missing.push("current story".to_string());
    }
    if required == TraceTier::Detailed && !has_files {
        missing.push("files changed".to_string());
    }
    let proof_valid = if risk == RiskLane::High {
        high_risk_proof_present(content)
    } else {
        true
    };
    if !proof_valid {
        missing.push("security/data-impact proof".to_string());
    }
    if let Some(proof) = bound_proof.as_ref() {
        if !proof_satisfies_risk(&proof.summary, risk) {
            missing.push("proof does not satisfy risk requirements".to_string());
        }
        if !proof.capability_gate_passed {
            missing.push("required capability execution evidence".to_string());
        }
        if risk != RiskLane::Low && !proof_has_current_receipt(repo_root, proof)? {
            missing.push("current trusted execution receipt".to_string());
        }
    }
    if risk != RiskLane::Low {
        if let Some(binding) = binding.as_ref() {
            let required_agents = [
                "code-reviewer".to_string(),
                "security-auditor".to_string(),
                "test-engineer".to_string(),
            ];
            let gate_status = gate_evidence_status_strict_for_operation(
                repo_root,
                &required_agents,
                &binding.task_id,
                &binding.operation_id,
                &binding.adapter,
                Some(&binding.session_id),
                Some(&binding.request_id),
            )?;
            if !gate_status.passed {
                missing.push("trusted quality-gate receipts".to_string());
            }
        }
    }
    if content.contains("- Capability gate: `failed`") {
        missing.push("required capability execution evidence".to_string());
    }
    let warnings = trace_list_field(content, "- Capability warnings: ");
    missing.sort();
    missing.dedup();
    let passed = achieved >= required && missing.is_empty();
    Ok(TraceScore {
        trace_id,
        achieved,
        required,
        passed,
        missing_fields: missing,
        warnings,
        proof_id,
        binding,
    })
}

pub fn latest_trace_score(repo_root: impl AsRef<Path>) -> Result<Option<TraceScore>> {
    let path = match find_trace(repo_root.as_ref(), None) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    let content = fs::read_to_string(path)?;
    let Some(_section) = content.split(SCORE_START).nth(1) else {
        return Ok(None);
    };
    Ok(Some(parse_score(&content)?))
}

/// Find a scored trace for one exact operation and proof. Global newest trace
/// selection is intentionally not used for completion authority.
pub fn latest_trace_score_for_operation(
    repo_root: &Path,
    expected: &TraceOperationBinding,
) -> Result<Option<TraceScore>> {
    let Some(trace) = trace_for_operation(repo_root, expected)? else {
        return Ok(None);
    };
    let content = fs::read_to_string(trace.repo_path)?;
    let score = evaluate_trace_score(repo_root, &content)?;
    if score.binding.as_ref() == Some(expected)
        && score.proof_id.as_deref() == Some(expected.proof_id.as_str())
    {
        Ok(Some(score))
    } else {
        Ok(None)
    }
}

/// Return the newest trace whose complete binding matches the expected
/// operation and proof ID.
pub fn trace_for_operation(
    repo_root: &Path,
    expected: &TraceOperationBinding,
) -> Result<Option<TraceRecord>> {
    expected.validate()?;
    let mut paths = trace_paths(repo_root)?;
    paths.sort_by_key(|path| artifact_sort_key(path));
    for path in paths.into_iter().rev() {
        let content = fs::read_to_string(&path)?;
        let Some(binding) = parse_trace_binding(&content)? else {
            continue;
        };
        if binding.matches_identity(expected) && binding.proof_id == expected.proof_id {
            let id = trace_field(&content, "- Trace ID: `").unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into()
            });
            return Ok(Some(TraceRecord {
                id,
                repo_path: path,
                vault_path: PathBuf::new(),
                proof_id: Some(binding.proof_id.clone()),
                binding: Some(binding),
            }));
        }
    }
    Ok(None)
}

fn render_trace(view: TraceView<'_>) -> String {
    let mut content = format!(
        "# Baron Execution Trace\n\n\
- Trace ID: `{}`\n\
- Recorded: {}\n\
- Risk: `{}`\n\
- Outcome: `{}`\n\
- Current plan: `{}`\n\
- Current story: `{}`\n\
- Proof: `{}`\n\
- Proof ID: `{}`\n\
{}\n\
- Capability gate: `{}`\n\
- Capability warnings: {}\n\
- Score status: `unscored`\n\n\
## Task Summary\n\n{}\n\n\
## Files Changed\n\n",
        view.id,
        Local::now().to_rfc3339_opts(SecondsFormat::Secs, false),
        view.risk.as_str(),
        view.outcome.as_str(),
        view.plan.unwrap_or("missing"),
        view.story.unwrap_or("missing"),
        view.proof.unwrap_or("missing"),
        view.proof_id.unwrap_or("missing"),
        render_binding(view.binding),
        if view.capability_gate_passed {
            "passed"
        } else {
            "failed"
        },
        if view.capability_warnings.is_empty() {
            "none".to_string()
        } else {
            view.capability_warnings.join(", ")
        },
        view.summary.trim()
    );
    if view.files.is_empty() {
        content.push_str("- none detected\n");
    } else {
        for file in view.files {
            content.push_str(&format!("- `{file}`\n"));
        }
    }
    content
}

struct TraceView<'a> {
    id: &'a str,
    summary: &'a str,
    outcome: TraceOutcome,
    risk: RiskLane,
    plan: Option<&'a str>,
    story: Option<&'a str>,
    proof: Option<&'a str>,
    proof_id: Option<&'a str>,
    binding: Option<&'a TraceOperationBinding>,
    capability_gate_passed: bool,
    capability_warnings: &'a [String],
    files: &'a [String],
}

fn render_binding(binding: Option<&TraceOperationBinding>) -> String {
    binding
        .map(|binding| {
            format!(
                "- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n",
                binding.task_id,
                binding.operation_id,
                binding.adapter,
                binding.session_id,
                binding.request_id,
            )
        })
        .unwrap_or_default()
}

fn find_trace(repo_root: &Path, trace_id: Option<&str>) -> Result<PathBuf> {
    let mut files = trace_paths(repo_root)?;
    files.sort_by_key(|path| artifact_sort_key(path));
    if let Some(id) = trace_id {
        return files
            .into_iter()
            .find(|path| path.file_stem().and_then(|value| value.to_str()) == Some(id))
            .with_context(|| format!("Trace not found: {id}"));
    }
    files.pop().context("No Baron trace found")
}

fn trace_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let root = repo_root.join("docs/baron/traces");
    let mut files = Vec::new();
    collect_markdown(&root, &mut files)?;
    files.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("INDEX.md"));
    Ok(files)
}

fn artifact_sort_key(path: &Path) -> (i128, String) {
    let timestamp = path
        .file_stem()
        .and_then(|value| value.to_str())
        .and_then(parse_artifact_timestamp)
        .or_else(|| {
            fs::metadata(path)
                .ok()?
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|value| value.as_millis() as i128)
        })
        .unwrap_or_default();
    (timestamp, path.to_string_lossy().into_owned())
}

fn parse_artifact_timestamp(stem: &str) -> Option<i128> {
    let mut parts = stem.split('-');
    let date = parts.next()?;
    if let Some(millis) = parts.next() {
        if date.len() == 8
            && millis.len() == 20
            && date.chars().all(|value| value.is_ascii_digit())
            && millis.chars().all(|value| value.is_ascii_digit())
        {
            return millis.parse().ok();
        }
    }
    let prefix = stem.chars().take(17).collect::<String>();
    if prefix.chars().count() != 17 || !prefix.chars().all(|value| value.is_ascii_digit()) {
        return None;
    }
    let year = prefix.get(0..4)?.parse().ok()?;
    let month = prefix.get(4..6)?.parse().ok()?;
    let day = prefix.get(6..8)?.parse().ok()?;
    let hour = prefix.get(8..10)?.parse().ok()?;
    let minute = prefix.get(10..12)?.parse().ok()?;
    let second = prefix.get(12..14)?.parse().ok()?;
    let millis = prefix.get(14..17)?.parse().ok()?;
    let legacy = NaiveDate::from_ymd_opt(year, month, day)?
        .and_hms_milli_opt(hour, minute, second, millis)?;
    Local
        .from_local_datetime(&legacy)
        .single()
        .map(|value| value.timestamp_millis() as i128)
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}

fn current_plan_title(repo_root: &Path) -> Option<String> {
    let content = fs::read_to_string(repo_root.join("docs/baron/plans/CURRENT.md")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("- Title: "))
        .map(str::to_string)
}

fn current_plan_risk(repo_root: &Path) -> RiskLane {
    let content =
        fs::read_to_string(repo_root.join("docs/baron/plans/CURRENT.md")).unwrap_or_default();
    if content.contains("- Risk: `high`") {
        RiskLane::High
    } else if content.contains("- Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

fn changed_files(repo_root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(repo_root)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !is_baron_managed_path(line))
        .map(str::to_string)
        .collect()
}

fn is_baron_managed_path(path: &str) -> bool {
    let path = path.replace('\\', "/");
    path.starts_with(".baron/")
        || path.starts_with(".codex/")
        || path.starts_with(".claude/")
        || path.starts_with("docs/baron/")
        || matches!(path.as_str(), "AGENTS.md" | "CLAUDE.md")
}

fn parse_risk(content: &str) -> RiskLane {
    if content.contains("- Risk: `high`") {
        RiskLane::High
    } else if content.contains("- Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

fn required_tier(risk: RiskLane) -> TraceTier {
    match risk {
        RiskLane::Low => TraceTier::Minimal,
        RiskLane::Medium => TraceTier::Standard,
        RiskLane::High => TraceTier::Detailed,
    }
}

fn high_risk_proof_present(content: &str) -> bool {
    let lower = content.to_lowercase();
    let verification = ["passed", "verified", "test"]
        .iter()
        .any(|term| lower.contains(term));
    let impact = [
        "security",
        "authorization",
        "permission",
        "tenant",
        "rls",
        "migration",
        "data impact",
        "payment",
        "upload",
    ]
    .iter()
    .any(|term| lower.contains(term));
    verification && impact
}

fn replace_score(content: &str, score: &TraceScore) -> String {
    let missing = if score.missing_fields.is_empty() {
        "none".to_string()
    } else {
        score.missing_fields.join(", ")
    };
    let warnings = if score.warnings.is_empty() {
        "none".to_string()
    } else {
        score.warnings.join(", ")
    };
    let block = format!(
        "{SCORE_START}\n## Trace Quality Score\n\n- Achieved: `{}`\n- Required: `{}`\n- Passed: `{}`\n- Missing: {missing}\n- Warnings: {warnings}\n{SCORE_END}\n",
        score.achieved.as_str(),
        score.required.as_str(),
        if score.passed { "yes" } else { "no" }
    );
    match (content.find(SCORE_START), content.find(SCORE_END)) {
        (Some(start), Some(end)) if end >= start => {
            let end = end + SCORE_END.len();
            format!("{}{}{}", &content[..start], block, &content[end..])
        }
        _ => format!("{}\n\n{}", content.trim_end(), block),
    }
}

fn trace_list_field(content: &str, prefix: &str) -> Vec<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or("none")
        .split(", ")
        .filter(|value| *value != "none")
        .map(str::to_string)
        .collect()
}

fn trace_field(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.strip_suffix('`'))
        .map(str::to_string)
}

fn trace_summary(content: &str) -> String {
    content
        .split("## Task Summary")
        .nth(1)
        .and_then(|value| value.split("## Files Changed").next())
        .unwrap_or("unknown")
        .trim()
        .replace(['\r', '\n'], " ")
}

fn update_trace_index(
    path: &Path,
    id: &str,
    outcome: &str,
    summary: &str,
    score: &TraceScore,
) -> Result<()> {
    let row = format!(
        "- `{id}` - {outcome} - score: `{}/{}` - passed: `{}` - {summary}",
        score.achieved.as_str(),
        score.required.as_str(),
        if score.passed { "yes" } else { "no" }
    );
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Trace Index\n\n".to_string());
    let prefix = format!("- `{id}` -");
    let mut replaced = false;
    let mut lines = content
        .lines()
        .map(|line| {
            if line.starts_with(&prefix) {
                replaced = true;
                row.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if !replaced {
        lines.push(row);
    }
    content = lines.join("\n");
    content.push('\n');
    write(path, &content)
}

fn parse_score(content: &str) -> Result<TraceScore> {
    let section = content
        .split(SCORE_START)
        .nth(1)
        .context("Trace quality score is missing")?;
    let missing = section
        .lines()
        .find_map(|line| line.strip_prefix("- Missing: "))
        .unwrap_or("none")
        .split(", ")
        .filter(|value| *value != "none")
        .map(str::to_string)
        .collect();
    let warnings = section
        .lines()
        .find_map(|line| line.strip_prefix("- Warnings: "))
        .unwrap_or("none")
        .split(", ")
        .filter(|value| *value != "none")
        .map(str::to_string)
        .collect();
    Ok(TraceScore {
        trace_id: trace_field(content, "- Trace ID: `").unwrap_or_else(|| "unknown".to_string()),
        achieved: parse_tier_line(section, "- Achieved: `"),
        required: parse_tier_line(section, "- Required: `"),
        passed: section.contains("- Passed: `yes`"),
        missing_fields: missing,
        warnings,
        proof_id: trace_field(content, "- Proof ID: `"),
        binding: parse_trace_binding(content)?,
    })
}

fn parse_trace_binding(content: &str) -> Result<Option<TraceOperationBinding>> {
    let fields = [
        trace_field(content, "- Task ID: `"),
        trace_field(content, "- Operation ID: `"),
        trace_field(content, "- Adapter: `"),
        trace_field(content, "- Session ID: `"),
        trace_field(content, "- Request ID: `"),
        trace_field(content, "- Proof ID: `"),
    ];
    let identity_present = fields[..5].iter().filter(|value| value.is_some()).count();
    if identity_present == 0 {
        return Ok(None);
    }
    if identity_present != 5 || fields[5].is_none() {
        bail!("trace operation binding is incomplete");
    }
    let [task_id, operation_id, adapter, session_id, request_id, proof_id] = fields;
    let binding = TraceOperationBinding {
        task_id: task_id.expect("checked task ID"),
        operation_id: operation_id.expect("checked operation ID"),
        adapter: adapter.expect("checked adapter"),
        session_id: session_id.expect("checked session ID"),
        request_id: request_id.expect("checked request ID"),
        proof_id: proof_id.expect("checked proof ID"),
    };
    binding.validate()?;
    Ok(Some(binding))
}

fn parse_tier_line(content: &str, prefix: &str) -> TraceTier {
    let value = content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("incomplete");
    TraceTier::from_str(value)
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let content = match read_text(path)? {
        Some(content) => content,
        None => {
            replace_text(path, header)?;
            header.to_string()
        }
    };
    let separator = if content.is_empty() || content.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    append_text(path, &format!("{separator}{item}\n"))
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

impl TraceOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

impl TraceTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Detailed => "detailed",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "minimal" => Self::Minimal,
            "standard" => Self::Standard,
            "detailed" => Self::Detailed,
            _ => Self::Incomplete,
        }
    }
}
