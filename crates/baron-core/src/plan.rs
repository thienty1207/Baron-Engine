use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::control_plane::gate_evidence_status_strict_for_operation;
use crate::execution_receipt::ReceiptContext;
use crate::operation::{task_id_for_task, LifecycleIdentity, OperationContext, SupportedAdapter};
use crate::proof::{
    proof_for_operation, proof_has_current_receipt, proof_operation_binding, proof_satisfies_risk,
};
use crate::risk::{classify_risk, RiskLane};
use crate::safe_io::{read_text, read_text_required, replace_text};
use crate::trace::{latest_trace_score_for_operation, TraceOperationBinding, TraceTier};
use crate::vault::{canonical_project_id, VaultContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRecord {
    pub title: String,
    pub risk: RiskLane,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEvidenceStatus {
    pub passed: bool,
    pub issues: Vec<String>,
}

/// Identity captured when a plan is started from a concrete Baron operation.
/// Legacy title-only plans remain readable, but medium/high-risk completion
/// cannot authorize them without this complete binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanOperationBinding {
    pub task_id: String,
    pub operation_id: String,
    pub adapter: String,
    pub session_id: String,
    pub request_id: String,
}

impl PlanOperationBinding {
    fn from_identity(identity: &LifecycleIdentity) -> Self {
        Self {
            task_id: identity.task_id().to_string(),
            operation_id: identity.operation_id().to_string(),
            adapter: identity.adapter().as_str().to_string(),
            session_id: identity.session_id().to_string(),
            request_id: identity.request_id().to_string(),
        }
    }

    pub fn to_operation_context(&self) -> Result<OperationContext> {
        let adapter =
            self.adapter
                .parse()
                .map_err(|error: crate::operation::OperationIdentityError| {
                    anyhow::anyhow!(error.to_string())
                })?;
        Ok(OperationContext::new(adapter)
            .with_task_id(self.task_id.clone())
            .with_operation_id(self.operation_id.clone())
            .with_session_id(self.session_id.clone())
            .with_request_id(self.request_id.clone()))
    }

    pub fn proof_binding(&self) -> ReceiptContext {
        ReceiptContext::new(
            self.task_id.clone(),
            self.operation_id.clone(),
            self.adapter.clone(),
            self.session_id.clone(),
            self.request_id.clone(),
            "proof",
        )
    }

    fn trace_binding(&self, proof_id: &str) -> TraceOperationBinding {
        TraceOperationBinding {
            task_id: self.task_id.clone(),
            operation_id: self.operation_id.clone(),
            adapter: self.adapter.clone(),
            session_id: self.session_id.clone(),
            request_id: self.request_id.clone(),
            proof_id: proof_id.to_string(),
        }
    }
}

/// Canonical active-plan metadata after CURRENT.md has been validated against
/// its linked plan file. Callers may use this for correctness-sensitive
/// evidence creation without parsing CURRENT.md independently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePlanAuthority {
    pub title: String,
    pub risk: RiskLane,
    pub binding: Option<PlanOperationBinding>,
}

pub fn active_plan_authority(repo_root: impl AsRef<Path>) -> Result<Option<ActivePlanAuthority>> {
    let Some(active) = active_plan(repo_root.as_ref())? else {
        return Ok(None);
    };
    active.ensure_authority()?;
    Ok(Some(ActivePlanAuthority {
        title: active.title,
        risk: active.risk,
        binding: active.binding,
    }))
}

/// Return the active plan's persisted operation binding for ingress adapters.
/// Missing bindings remain explicit instead of being inferred from the latest
/// repository artifact.
pub fn active_plan_operation_binding(
    repo_root: impl AsRef<Path>,
) -> Result<Option<PlanOperationBinding>> {
    Ok(active_plan_authority(repo_root)?.and_then(|authority| authority.binding))
}

/// Evaluate the current active plan using the same scoped completion evidence
/// consumed by plan completion and completion-integrity diagnostics. A
/// completed plan is not an active reconciliation target; a malformed active
/// pointer remains a failing reconciliation target.
pub fn active_plan_completion_evidence_status(
    repo_root: impl AsRef<Path>,
) -> Result<Option<CompletionEvidenceStatus>> {
    let repo_root = repo_root.as_ref();
    let current_path = repo_root.join("docs/baron/plans/CURRENT.md");
    let Some(_) = read_text(&current_path)? else {
        return Ok(None);
    };
    let Some(active) = active_plan(repo_root)? else {
        return Ok(Some(CompletionEvidenceStatus {
            passed: false,
            issues: vec!["active plan is missing".to_string()],
        }));
    };
    if !active.authority_issues.is_empty() {
        return Ok(Some(CompletionEvidenceStatus {
            passed: false,
            issues: active.authority_issues.clone(),
        }));
    }
    if !is_active_plan_status(&active.status) {
        return Ok(None);
    }
    Ok(Some(completion_evidence_status(repo_root, &active)?))
}

pub fn start_or_resume_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
) -> Result<PlanRecord> {
    start_or_resume_plan_internal(repo_root.as_ref(), vault, title, None)
}

/// Start or resume a plan while persisting the exact operation identity that
/// will be required by medium/high-risk completion. This is the operation-
/// aware entry point used by trusted Baron integrations.
pub fn start_or_resume_plan_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
    operation: &OperationContext,
) -> Result<PlanRecord> {
    let identity = operation
        .lifecycle_identity_for_task(&vault.project_id, title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    start_or_resume_plan_for_identity(repo_root, vault, title, &identity)
}

/// Start or resume a plan from one complete canonical lifecycle identity.
/// Legacy title-only plan files remain readable, but this entry point never
/// upgrades an unbound plan by inference.
pub fn start_or_resume_plan_for_identity(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
    identity: &LifecycleIdentity,
) -> Result<PlanRecord> {
    if identity.project_id() != vault.project_id {
        bail!(
            "plan lifecycle identity project `{}` does not match Vault project `{}`",
            identity.project_id(),
            vault.project_id
        );
    }
    identity
        .validate_task(title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let binding = PlanOperationBinding::from_identity(identity);
    start_or_resume_plan_internal(repo_root.as_ref(), vault, title, Some(&binding))
}

fn start_or_resume_plan_internal(
    repo_root: &Path,
    vault: &VaultContext,
    title: &str,
    binding: Option<&PlanOperationBinding>,
) -> Result<PlanRecord> {
    let title = title.trim();
    if let Some(active) = active_plan(repo_root)? {
        active.ensure_authority()?;
        if active.title.eq_ignore_ascii_case(title) && active.status != "completed" {
            if let Some(requested) = binding {
                match active.binding.as_ref() {
                    Some(existing) if existing != requested => {
                        bail!("Cannot resume plan `{title}` under a different operation identity");
                    }
                    None => {
                        bail!(
                            "Cannot authorize legacy unbound plan `{title}`; start a new identified operation"
                        );
                    }
                    Some(_) => {}
                }
            }
            set_plan_state(&active.path, "in_progress", None)?;
            append_progress(&active.path, "Plan resumed.")?;
            mirror_plan(repo_root, vault, &active.path)?;
            update_plan_indexes(
                repo_root,
                vault,
                &active.title,
                &active.path,
                active.risk,
                "in_progress",
            )?;
            write_current(
                repo_root,
                vault,
                CurrentPlanView {
                    title,
                    risk: active.risk,
                    status: "in_progress",
                    plan_path: &active.path,
                    next_action: "continue from last known state",
                    verification: "not_run",
                    binding: active.binding.as_ref(),
                    task_id: Some(&active.task_id),
                },
            )?;
            return Ok(PlanRecord {
                title: title.to_string(),
                risk: active.risk,
                repo_path: active.path.clone(),
                vault_path: vault_plan_path(repo_root, vault, &active.path),
                resumed: true,
            });
        }
    }
    let risk = classify_risk(title);
    let date = today();
    let repo_path = repo_root
        .join("docs/baron/plans")
        .join(&date)
        .join(format!("{date}-{}.md", slugify(title)));
    let vault_path = vault_plan_path(repo_root, vault, &repo_path);
    let content = plan_content(&vault.project_id, title, risk, binding)?;
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
    append_unique(
        &repo_root.join("docs/baron/plans/INDEX.md"),
        "# Baron Plan Index\n\n",
        &format!(
            "- [{}]({}) - status: `in_progress` - risk: `{}`",
            title,
            normalize(&repo_path, repo_root),
            risk.as_str()
        ),
    )?;
    append_unique(
        &vault.project_root.join("Plans/INDEX.md"),
        "# Baron Plan Index\n\n",
        &format!(
            "- [{}]({}) - status: `in_progress` - risk: `{}`",
            title,
            normalize(&vault_path, &vault.project_root),
            risk.as_str()
        ),
    )?;
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title,
            risk,
            status: "in_progress",
            plan_path: &repo_path,
            next_action: "continue from current task scope",
            verification: "not_run",
            binding,
            task_id: None,
        },
    )?;
    Ok(PlanRecord {
        title: title.to_string(),
        risk,
        repo_path,
        vault_path,
        resumed: false,
    })
}

pub fn update_plan(repo_root: impl AsRef<Path>, vault: &VaultContext, note: &str) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let active = require_active_plan(repo_root)?;
    append_progress(&active.path, note.trim())?;
    mirror_plan(repo_root, vault, &active.path)?;
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title: &active.title,
            risk: active.risk,
            status: &active.status,
            plan_path: &active.path,
            next_action: note.trim(),
            verification: "not_run",
            binding: active.binding.as_ref(),
            task_id: Some(&active.task_id),
        },
    )
}

pub fn interrupt_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    state: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let active = require_active_plan(repo_root)?;
    set_plan_state(&active.path, "interrupted", None)?;
    append_progress(&active.path, &format!("Interrupted: {}", state.trim()))?;
    mirror_plan(repo_root, vault, &active.path)?;
    update_plan_indexes(
        repo_root,
        vault,
        &active.title,
        &active.path,
        active.risk,
        "interrupted",
    )?;
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title: &active.title,
            risk: active.risk,
            status: "interrupted",
            plan_path: &active.path,
            next_action: state.trim(),
            verification: "not_run",
            binding: active.binding.as_ref(),
            task_id: Some(&active.task_id),
        },
    )
}

pub fn complete_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    verification_summary: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let active = require_active_plan(repo_root)?;
    if let Some(issue) = completion_evidence_status(repo_root, &active)?
        .issues
        .into_iter()
        .next()
    {
        bail!("Plan completion blocked: {issue}.");
    }
    if verification_summary.trim().is_empty() {
        bail!("Plan completion requires a non-empty verification summary.");
    }
    set_plan_state(&active.path, "completed", Some(verification_summary.trim()))?;
    append_progress(
        &active.path,
        &format!(
            "Completed with verification: {}",
            verification_summary.trim()
        ),
    )?;
    mirror_plan(repo_root, vault, &active.path)?;
    update_plan_indexes(
        repo_root,
        vault,
        &active.title,
        &active.path,
        active.risk,
        "completed",
    )?;
    write_current(
        repo_root,
        vault,
        CurrentPlanView {
            title: &active.title,
            risk: active.risk,
            status: "completed",
            plan_path: &active.path,
            next_action: "start the next explicit task",
            verification: verification_summary.trim(),
            binding: active.binding.as_ref(),
            task_id: Some(&active.task_id),
        },
    )
}

/// Evaluate every completion-sensitive artifact from one active plan scope.
/// Both completion and post-completion integrity diagnostics call this helper
/// so they cannot disagree about which proof, trace, or gates are authoritative.
fn completion_evidence_status(
    repo_root: &Path,
    active: &ActivePlan,
) -> Result<CompletionEvidenceStatus> {
    if !active.authority_issues.is_empty() {
        return Ok(CompletionEvidenceStatus {
            passed: false,
            issues: active.authority_issues.clone(),
        });
    }
    let issues = completion_evidence_issues(repo_root, active)?;
    Ok(CompletionEvidenceStatus {
        passed: issues.is_empty(),
        issues,
    })
}

fn completion_evidence_issues(repo_root: &Path, active: &ActivePlan) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    let Some(expected_binding) = active.binding.as_ref() else {
        issues.push("proof is missing".to_string());
        issues.push("passing trace is missing".to_string());
        issues.push("active plan operation binding is missing".to_string());
        return Ok(issues);
    };

    let proof_binding = expected_binding.proof_binding();
    let proof = proof_for_operation(repo_root, &proof_binding)?;
    if let Some(proof) = proof.as_ref() {
        if !proof_satisfies_risk(&proof.summary, active.risk) {
            issues.push(format!(
                "proof does not satisfy `{}` risk requirements",
                active.risk.as_str()
            ));
        }
        let Some(actual_binding) = proof_operation_binding(proof) else {
            issues.push("proof operation identity does not match the active plan".to_string());
            return Ok(issues);
        };
        if !same_operation_binding(&actual_binding, &proof_binding)
            || actual_binding.gate_kind.trim() != "proof"
        {
            issues.push("proof operation identity does not match the active plan".to_string());
        }
        if active.risk != RiskLane::Low && !proof_has_current_receipt(repo_root, proof)? {
            issues.push(
                "medium/high-risk proof must reference a current trusted execution receipt"
                    .to_string(),
            );
        }
    } else {
        issues.push("proof is missing".to_string());
    }

    if active.risk != RiskLane::Low {
        let required_agents = [
            "code-reviewer".to_string(),
            "security-auditor".to_string(),
            "test-engineer".to_string(),
        ];
        let gate_status = gate_evidence_status_strict_for_operation(
            repo_root,
            &required_agents,
            &expected_binding.task_id,
            &expected_binding.operation_id,
            &expected_binding.adapter,
            Some(&expected_binding.session_id),
            Some(&expected_binding.request_id),
        )?;
        if !gate_status.passed {
            issues.push(format!(
                "trusted quality-gate receipts are missing for {}",
                gate_status.missing_agents.join(", ")
            ));
        }
    }

    if let Some(proof) = proof {
        let trace_binding = expected_binding.trace_binding(&proof.id);
        match latest_trace_score_for_operation(repo_root, &trace_binding)? {
            Some(trace) if trace.passed && trace.achieved >= required_tier(active.risk) => {}
            _ => issues.push("passing trace is missing".to_string()),
        }
    } else {
        issues.push("passing trace is missing".to_string());
    }
    Ok(issues)
}

fn same_operation_binding(left: &ReceiptContext, right: &ReceiptContext) -> bool {
    left.task_id == right.task_id
        && left.operation_id == right.operation_id
        && left.adapter == right.adapter
        && left.session_id == right.session_id
        && left.request_id == right.request_id
}

pub fn plan_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let path = repo_root.join("docs/baron/plans/CURRENT.md");
    let Some(current) = read_text(&path)? else {
        return Ok("# Baron Plan Status\n\n- Active plan: none\n".to_string());
    };
    let mut output = format!("# Baron Plan Status\n\n{current}");
    let active = active_plan(repo_root)?;
    let authority_issues = active
        .as_ref()
        .map(|plan| plan.authority_issues.clone())
        .unwrap_or_default();
    let linked_completed = active
        .as_ref()
        .is_some_and(|plan| plan.status == "completed");
    let should_report_integrity = !authority_issues.is_empty()
        || linked_completed
        || (active.is_none() && !current.trim().is_empty());
    if should_report_integrity {
        let issues = if authority_issues.is_empty() {
            completion_integrity_issues(repo_root, &current)?
        } else {
            authority_issues
        };
        if issues.is_empty() {
            output.push_str("\n## Completion Integrity\n\n- Completion integrity: `passed`\n");
        } else {
            output.push_str("\n## Completion Integrity\n\n- Completion integrity: `failed`\n");
            for issue in issues {
                output.push_str(&format!("- {issue}\n"));
            }
        }
    } else {
        output.push_str("\n## Completion Integrity\n\n- Completion integrity: `not_applicable`\n");
    }
    Ok(output)
}

fn completion_integrity_issues(repo_root: &Path, current: &str) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    let verification = field(current, "- Verification: ").unwrap_or_default();
    if verification.trim().is_empty() || verification.trim() == "not_run" {
        issues.push("verification evidence is missing".to_string());
    }
    let active = active_plan(repo_root)?;
    match active {
        Some(plan) if plan.path.is_file() => {
            let body = read_text_required(&plan.path)?;
            if plan.linked_status.as_deref() != Some("completed")
                || !body.lines().any(|line| line == "status: completed")
            {
                issues.push("plan file is not marked completed".to_string());
            }
            let plan_verification = body
                .lines()
                .find_map(|line| line.strip_prefix("verification: "))
                .unwrap_or_default();
            if plan_verification.trim().is_empty() || plan_verification.trim() == "not_run" {
                issues.push("plan verification evidence is missing".to_string());
            }
            issues.extend(completion_evidence_status(repo_root, &plan)?.issues);
        }
        _ => issues.push("linked plan file is missing".to_string()),
    }
    Ok(issues)
}

fn write_current(repo_root: &Path, vault: &VaultContext, view: CurrentPlanView<'_>) -> Result<()> {
    let task_id = view
        .task_id
        .map(str::to_owned)
        .or_else(|| view.binding.map(|binding| binding.task_id.clone()))
        .map_or_else(
            || {
                task_id_for_task(&vault.project_id, view.title)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))
            },
            Ok,
        )?;
    let operation_identity = view
        .binding
        .map(|binding| {
            format!(
                "- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n",
                binding.operation_id, binding.adapter, binding.session_id, binding.request_id
            )
        })
        .unwrap_or_default();
    let content = format!(
        "# Current Baron Plan\n\n\
- Title: {}\n\
- Plan: `{}`\n\
- Status: `{}`\n\
- Risk: `{}`\n\
- Task ID: `{}`\n\
{}\
- Verification: {}\n\
- Next action: {}\n\
- Updated: {}\n\n\
## Rules\n\n\
- Silence or shutdown never means completed.\n\
- Completion requires risk-appropriate proof and a passing trace score.\n",
        view.title,
        normalize(view.plan_path, repo_root),
        view.status,
        view.risk.as_str(),
        task_id,
        operation_identity,
        view.verification,
        view.next_action,
        now()
    );
    write(&repo_root.join("docs/baron/plans/CURRENT.md"), &content)?;
    write(&vault.project_root.join("Plans/CURRENT.md"), &content)
}

fn active_plan(repo_root: &Path) -> Result<Option<ActivePlan>> {
    let current_path = repo_root.join("docs/baron/plans/CURRENT.md");
    let Some(content) = read_text(&current_path)? else {
        return Ok(None);
    };
    let mut authority_issues = Vec::new();
    let current_title = field(&content, "- Title: ")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let path = field(&content, "- Plan: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string))
        .map(|value| {
            if !is_safe_plan_path(&value) {
                return Err(anyhow::anyhow!(
                    "Active Baron plan path escapes the project: {value}"
                ));
            }
            Ok(repo_root.join(value))
        })
        .transpose()?;
    let current_status = match current_backtick_field(&content, "- Status: `") {
        Some(value) => match parse_plan_status(&value) {
            Ok(status) => status,
            Err(error) => {
                authority_issues.push(format!("CURRENT status is invalid: {error}"));
                value
            }
        },
        None => {
            authority_issues.push("CURRENT status is missing".to_string());
            "unknown".to_string()
        }
    };
    let mut status = current_status.clone();
    let risk = match current_backtick_field(&content, "- Risk: `") {
        Some(value) => match parse_risk_lane(&value) {
            Ok(risk) => risk,
            Err(error) => {
                authority_issues.push(format!("CURRENT risk is invalid: {error}"));
                RiskLane::Medium
            }
        },
        None => {
            authority_issues.push("CURRENT risk is missing".to_string());
            RiskLane::Medium
        }
    };
    let task_id = current_backtick_field(&content, "- Task ID: `");
    let operation_id = current_backtick_field(&content, "- Operation ID: `");
    let adapter = current_backtick_field(&content, "- Adapter: `");
    let session_id = current_backtick_field(&content, "- Session ID: `");
    let request_id = current_backtick_field(&content, "- Request ID: `");
    let (current_binding, binding_issues) = current_operation_binding(
        task_id.clone(),
        operation_id,
        adapter,
        session_id,
        request_id,
    );
    authority_issues.extend(binding_issues);

    Ok(path.map(|path| {
        let mut title = current_title;
        let mut canonical_risk = risk;
        let mut linked_task_id = task_id.clone();
        let mut binding = current_binding;
        let mut linked_status = None;
        match load_plan_file_metadata(&path) {
            Ok(metadata) => {
                linked_status = Some(metadata.status.clone());
                let linked_binding = match metadata.operation_binding() {
                    Ok(linked_binding) => linked_binding,
                    Err(error) => {
                        authority_issues.push(error.to_string());
                        None
                    }
                };
                compare_current_with_linked_plan(
                    CurrentPlanProjection {
                        title: &title,
                        risk: canonical_risk,
                        status: &status,
                        task_id: task_id.as_deref(),
                        binding: binding.as_ref(),
                    },
                    &metadata,
                    linked_binding.as_ref(),
                    &mut authority_issues,
                );
                if let Err(error) = validate_linked_plan_authority(repo_root, &metadata) {
                    authority_issues.push(format!("linked plan authority is invalid: {error}"));
                }
                title = metadata.title.clone();
                status = metadata.status.clone();
                canonical_risk = metadata.risk;
                linked_task_id = Some(metadata.task_id.clone());
                binding = linked_binding;
            }
            Err(error) => {
                authority_issues.push(format!("linked plan metadata is unavailable: {error}"));
            }
        }
        ActivePlan {
            title,
            path,
            status,
            risk: canonical_risk,
            task_id: linked_task_id.unwrap_or_default(),
            binding,
            linked_status,
            authority_issues,
        }
    }))
}

fn require_active_plan(repo_root: &Path) -> Result<ActivePlan> {
    let active = active_plan(repo_root)?
        .context("No active Baron plan. Run `baron plan start \"<title>\"`.")?;
    active.ensure_authority()?;
    Ok(active)
}

fn current_backtick_field(content: &str, prefix: &str) -> Option<String> {
    field(content, prefix).and_then(|value| {
        value
            .strip_suffix('`')
            .map(|value| value.trim().to_string())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanFileMetadata {
    title: String,
    status: String,
    risk: RiskLane,
    task_id: String,
    operation_id: Option<String>,
    adapter: Option<String>,
    session_id: Option<String>,
    request_id: Option<String>,
}

impl PlanFileMetadata {
    fn operation_binding(&self) -> Result<Option<PlanOperationBinding>> {
        match (
            self.operation_id.as_ref(),
            self.adapter.as_ref(),
            self.session_id.as_ref(),
            self.request_id.as_ref(),
        ) {
            (None, None, None, None) => Ok(None),
            (Some(operation_id), Some(adapter), Some(session_id), Some(request_id))
                if !operation_id.trim().is_empty()
                    && !adapter.trim().is_empty()
                    && !session_id.trim().is_empty()
                    && !request_id.trim().is_empty() =>
            {
                adapter
                    .parse::<SupportedAdapter>()
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                Ok(Some(PlanOperationBinding {
                    task_id: self.task_id.clone(),
                    operation_id: operation_id.clone(),
                    adapter: adapter.clone(),
                    session_id: session_id.clone(),
                    request_id: request_id.clone(),
                }))
            }
            _ => bail!("linked plan operation metadata is incomplete"),
        }
    }
}

fn load_plan_file_metadata(path: &Path) -> Result<PlanFileMetadata> {
    let content = read_text_required(path)?;
    let title = required_plan_field(&content, "title:")?;
    let status = parse_plan_status(&required_plan_field(&content, "status:")?)?;
    let risk = parse_risk_lane(&required_plan_field(&content, "risk:")?)?;
    let task_id = required_plan_field(&content, "task_id:")?;
    if task_id.is_empty() {
        bail!("linked plan task_id is empty");
    }
    Ok(PlanFileMetadata {
        title,
        status,
        risk,
        task_id,
        operation_id: optional_plan_field(&content, "operation_id:"),
        adapter: optional_plan_field(&content, "adapter:"),
        session_id: optional_plan_field(&content, "session_id:"),
        request_id: optional_plan_field(&content, "request_id:"),
    })
}

fn validate_linked_plan_authority(
    repo_root: &Path,
    metadata: &PlanFileMetadata,
) -> Result<Option<PlanOperationBinding>> {
    let expected_risk = classify_risk(&metadata.title);
    if metadata.risk != expected_risk {
        bail!("linked plan risk does not match canonical classifier");
    }

    let project_id = canonical_project_id(repo_root)?;
    let expected_task_id = task_id_for_task(&project_id, &metadata.title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let binding = metadata.operation_binding()?;
    let Some(binding) = binding else {
        if metadata.task_id != expected_task_id
            && metadata.task_id != format!("task-{}", slugify(&metadata.title))
        {
            bail!("linked plan task_id does not match canonical task or legacy format");
        }
        return Ok(None);
    };

    if metadata.task_id != expected_task_id {
        bail!("linked plan task_id does not match canonical task");
    }

    let adapter = SupportedAdapter::parse(&binding.adapter)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let identity = LifecycleIdentity::from_parts_checked(
        project_id,
        binding.task_id.clone(),
        binding.operation_id.clone(),
        adapter,
        binding.session_id.clone(),
        binding.request_id.clone(),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    identity
        .validate_task(&metadata.title)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    Ok(Some(binding))
}

fn required_plan_field(content: &str, prefix: &str) -> Result<String> {
    optional_plan_field(content, prefix)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("linked plan metadata is missing `{prefix}`"))
}

fn optional_plan_field(content: &str, prefix: &str) -> Option<String> {
    content.lines().find_map(|line| {
        line.strip_prefix(prefix)
            .map(|value| value.trim().to_string())
    })
}

fn parse_risk_lane(value: &str) -> Result<RiskLane> {
    match value.trim() {
        "low" => Ok(RiskLane::Low),
        "medium" => Ok(RiskLane::Medium),
        "high" => Ok(RiskLane::High),
        other => bail!("unsupported risk lane `{other}`"),
    }
}

fn parse_plan_status(value: &str) -> Result<String> {
    match value.trim() {
        "in_progress" | "interrupted" | "needs_correction" | "blocked" | "completed" => {
            Ok(value.trim().to_string())
        }
        other => bail!("unsupported plan status `{other}`"),
    }
}

fn is_active_plan_status(value: &str) -> bool {
    matches!(
        value,
        "in_progress" | "interrupted" | "needs_correction" | "blocked"
    )
}

fn current_operation_binding(
    task_id: Option<String>,
    operation_id: Option<String>,
    adapter: Option<String>,
    session_id: Option<String>,
    request_id: Option<String>,
) -> (Option<PlanOperationBinding>, Vec<String>) {
    let fields = [
        operation_id.as_ref(),
        adapter.as_ref(),
        session_id.as_ref(),
        request_id.as_ref(),
    ];
    if fields.iter().all(Option::is_none) {
        return (None, Vec::new());
    }
    if task_id.is_none()
        || fields.iter().any(Option::is_none)
        || [
            operation_id.as_deref(),
            adapter.as_deref(),
            session_id.as_deref(),
            request_id.as_deref(),
        ]
        .iter()
        .any(|value| value.is_some_and(str::is_empty))
    {
        return (
            None,
            vec!["CURRENT operation metadata is incomplete".to_string()],
        );
    }
    let adapter = adapter.expect("checked above");
    if let Err(error) = adapter.parse::<SupportedAdapter>() {
        return (None, vec![format!("CURRENT adapter is invalid: {error}")]);
    }
    (
        Some(PlanOperationBinding {
            task_id: task_id.expect("checked above"),
            operation_id: operation_id.expect("checked above"),
            adapter,
            session_id: session_id.expect("checked above"),
            request_id: request_id.expect("checked above"),
        }),
        Vec::new(),
    )
}

struct CurrentPlanProjection<'a> {
    title: &'a str,
    status: &'a str,
    risk: RiskLane,
    task_id: Option<&'a str>,
    binding: Option<&'a PlanOperationBinding>,
}

fn compare_current_with_linked_plan(
    current: CurrentPlanProjection<'_>,
    linked: &PlanFileMetadata,
    linked_binding: Option<&PlanOperationBinding>,
    issues: &mut Vec<String>,
) {
    if current.title != linked.title {
        issues.push("CURRENT title does not match linked plan title".to_string());
    }
    if current.risk != linked.risk {
        issues.push("CURRENT risk does not match linked plan risk".to_string());
    }
    if current.status != linked.status {
        issues.push("CURRENT status does not match linked plan status".to_string());
    }
    match current.task_id {
        Some(current_task_id) if current_task_id == linked.task_id => {}
        Some(_) => issues.push("CURRENT task_id does not match linked plan task_id".to_string()),
        None => issues.push("CURRENT task_id is missing from linked plan authority".to_string()),
    }
    match (current.binding, linked_binding) {
        (None, None) => {}
        (Some(current), Some(linked)) => {
            if current.operation_id != linked.operation_id {
                issues.push("CURRENT operation_id does not match linked plan".to_string());
            }
            if current.adapter != linked.adapter {
                issues.push("CURRENT adapter does not match linked plan".to_string());
            }
            if current.session_id != linked.session_id {
                issues.push("CURRENT session_id does not match linked plan".to_string());
            }
            if current.request_id != linked.request_id {
                issues.push("CURRENT request_id does not match linked plan".to_string());
            }
        }
        _ => issues.push("CURRENT and linked plan operation binding state differ".to_string()),
    }
}

fn field(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::to_string)
}

fn set_plan_state(path: &Path, status: &str, verification: Option<&str>) -> Result<()> {
    let content = read_text_required(path)?;
    let updated = now();
    let verification = verification.map(single_line);
    let updated = content
        .lines()
        .map(|line| {
            if line.starts_with("status: ") {
                format!("status: {status}")
            } else if line.starts_with("updated: ") {
                format!("updated: {updated}")
            } else if line.starts_with("verification: ") {
                verification
                    .as_ref()
                    .map(|value| format!("verification: {value}"))
                    .unwrap_or_else(|| line.to_string())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    write(path, &(updated + "\n"))
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn append_progress(path: &Path, note: &str) -> Result<()> {
    let mut content = read_text_required(path)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("- {} - {}\n", now(), note));
    write(path, &content)
}

fn mirror_plan(repo_root: &Path, vault: &VaultContext, plan_path: &Path) -> Result<()> {
    let content = read_text_required(plan_path)?;
    write(&vault_plan_path(repo_root, vault, plan_path), &content)
}

fn vault_plan_path(repo_root: &Path, vault: &VaultContext, repo_path: &Path) -> PathBuf {
    let relative = repo_path
        .strip_prefix(repo_root.join("docs/baron/plans"))
        .unwrap_or(repo_path);
    vault.project_root.join("Plans").join(relative)
}

fn plan_content(
    project_id: &str,
    title: &str,
    risk: RiskLane,
    binding: Option<&PlanOperationBinding>,
) -> Result<String> {
    let task_id = binding.map(|binding| binding.task_id.clone()).map_or_else(
        || task_id_for_task(project_id, title).map_err(|error| anyhow::anyhow!(error.to_string())),
        Ok,
    )?;
    let operation_identity = binding
        .map(|binding| {
            format!(
                "operation_id: {}\nadapter: {}\nsession_id: {}\nrequest_id: {}\n",
                binding.operation_id, binding.adapter, binding.session_id, binding.request_id
            )
        })
        .unwrap_or_default();
    Ok(format!(
        "---\n\
type: baron-plan\n\
title: {title}\n\
status: in_progress\n\
risk: {}\n\
task_id: {task_id}\n\
{operation_identity}\
created: {}\n\
updated: {}\n\
verification: not_run\n\
---\n\n\
# {title}\n\n\
## Goal\n\n{title}\n\n\
## Scope\n\n- Work tied to this task only.\n\n\
## Checklist\n\n\
- [ ] Define the implementation path.\n\
- [ ] Implement the requested change.\n\
- [ ] Record risk-appropriate proof.\n\
- [ ] Record and score the execution trace.\n\n\
## Progress Log\n\n\
- {} - Plan started.\n",
        risk.as_str(),
        today(),
        now(),
        now()
    ))
}

fn required_tier(risk: RiskLane) -> TraceTier {
    match risk {
        RiskLane::Low => TraceTier::Minimal,
        RiskLane::Medium => TraceTier::Standard,
        RiskLane::High => TraceTier::Detailed,
    }
}

fn append_unique(path: &Path, header: &str, item: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if content.contains(item) {
        return Ok(());
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    content.push('\n');
    write(path, &content)
}

fn update_plan_indexes(
    repo_root: &Path,
    vault: &VaultContext,
    title: &str,
    repo_path: &Path,
    risk: RiskLane,
    status: &str,
) -> Result<()> {
    let vault_path = vault_plan_path(repo_root, vault, repo_path);
    replace_plan_index_row(
        &repo_root.join("docs/baron/plans/INDEX.md"),
        title,
        &normalize(repo_path, repo_root),
        risk,
        status,
    )?;
    replace_plan_index_row(
        &vault.project_root.join("Plans/INDEX.md"),
        title,
        &normalize(&vault_path, &vault.project_root),
        risk,
        status,
    )
}

fn replace_plan_index_row(
    path: &Path,
    title: &str,
    relative_path: &str,
    risk: RiskLane,
    status: &str,
) -> Result<()> {
    let row = format!(
        "- [{title}]({relative_path}) - status: `{status}` - risk: `{}`",
        risk.as_str()
    );
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Plan Index\n\n".to_string());
    let prefix = format!("- [{title}](");
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

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn is_safe_plan_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.trim().is_empty()
        && !value.contains('\\')
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn normalize(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "plan".to_string()
    } else {
        slug
    }
}

struct ActivePlan {
    title: String,
    path: PathBuf,
    status: String,
    risk: RiskLane,
    task_id: String,
    binding: Option<PlanOperationBinding>,
    linked_status: Option<String>,
    authority_issues: Vec<String>,
}

impl ActivePlan {
    fn ensure_authority(&self) -> Result<()> {
        if let Some(issue) = self.authority_issues.first() {
            bail!("Active Baron plan authority mismatch: {issue}");
        }
        Ok(())
    }
}

struct CurrentPlanView<'a> {
    title: &'a str,
    risk: RiskLane,
    status: &'a str,
    plan_path: &'a Path,
    next_action: &'a str,
    verification: &'a str,
    binding: Option<&'a PlanOperationBinding>,
    task_id: Option<&'a str>,
}
