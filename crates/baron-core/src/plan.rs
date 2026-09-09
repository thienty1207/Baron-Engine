use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::control_plane::gate_evidence_status_strict_for_operation;
use crate::operation::OperationContext;
use crate::proof::{
    latest_proof, proof_has_current_receipt, proof_receipt_context, proof_satisfies_risk,
};
use crate::risk::{classify_risk, RiskLane};
use crate::safe_io::{read_text, read_text_required, replace_text};
use crate::trace::{latest_trace_score, TraceTier};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRecord {
    pub title: String,
    pub risk: RiskLane,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
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
    fn from_operation(operation: &OperationContext) -> Result<Self> {
        let (Some(task_id), Some(operation_id), Some(session_id), Some(request_id)) = (
            operation.task_id.clone(),
            operation.operation_id.clone(),
            operation.session_id.clone(),
            operation.request_id.clone(),
        ) else {
            bail!(
                "plan operation binding requires task, operation, session, and request identities"
            );
        };
        if task_id.trim().is_empty()
            || operation_id.trim().is_empty()
            || session_id.trim().is_empty()
            || request_id.trim().is_empty()
        {
            bail!("plan operation binding requires non-empty identities");
        }
        Ok(Self {
            task_id,
            operation_id,
            adapter: operation.adapter.as_str().to_string(),
            session_id,
            request_id,
        })
    }
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
    let binding = PlanOperationBinding::from_operation(operation)?;
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
    let content = plan_content(title, risk, binding);
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
    let proof = latest_proof(repo_root)?.context(
        "Plan completion blocked: proof is missing. Run `baron proof record \"<verification>\"`.",
    )?;
    if !proof_satisfies_risk(&proof.summary, active.risk) {
        bail!(
            "Plan completion blocked: proof does not satisfy `{}` risk requirements.",
            active.risk.as_str()
        );
    }
    if active.risk != RiskLane::Low && !proof_has_current_receipt(repo_root, &proof)? {
        bail!(
            "Plan completion blocked: medium/high-risk proof must reference a current trusted execution receipt."
        );
    }
    if active.risk != RiskLane::Low {
        let required_agents = [
            "code-reviewer".to_string(),
            "security-auditor".to_string(),
            "test-engineer".to_string(),
        ];
        let (_, proof_binding) = proof_receipt_context(&proof)?
            .context("Plan completion blocked: proof receipt binding is missing.")?;
        let expected_binding = active.binding.as_ref().context(
            "Plan completion blocked: active plan has no complete operation binding; restart it through an identified Baron operation.",
        )?;
        if proof_binding.task_id != expected_binding.task_id
            || proof_binding.operation_id != expected_binding.operation_id
            || proof_binding.adapter != expected_binding.adapter
            || proof_binding.session_id != expected_binding.session_id
            || proof_binding.request_id != expected_binding.request_id
        {
            bail!(
                "Plan completion blocked: proof receipt identity does not match the active plan operation.",
            );
        }
        let gate_status = gate_evidence_status_strict_for_operation(
            repo_root,
            &required_agents,
            &proof_binding.task_id,
            &proof_binding.operation_id,
            &proof_binding.adapter,
            Some(&proof_binding.session_id),
            Some(&proof_binding.request_id),
        )?;
        if !gate_status.passed {
            bail!(
                "Plan completion blocked: trusted quality-gate receipts are missing for {}.",
                gate_status.missing_agents.join(", ")
            );
        }
    }
    let trace = latest_trace_score(repo_root)?.context(
        "Plan completion blocked: scored trace is missing. Run `baron trace record` and `baron trace score`.",
    )?;
    let required = required_tier(active.risk);
    if !trace.passed || trace.achieved < required {
        bail!(
            "Plan completion blocked: trace quality must pass `{}`.",
            required.as_str()
        );
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
        },
    )
}

pub fn plan_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let path = repo_root.join("docs/baron/plans/CURRENT.md");
    let Some(current) = read_text(&path)? else {
        return Ok("# Baron Plan Status\n\n- Active plan: none\n".to_string());
    };
    let mut output = format!("# Baron Plan Status\n\n{current}");
    if current.contains("- Status: `completed`") {
        let issues = completion_integrity_issues(repo_root, &current)?;
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
            if !body.lines().any(|line| line == "status: completed") {
                issues.push("plan file is not marked completed".to_string());
            }
            let plan_verification = body
                .lines()
                .find_map(|line| line.strip_prefix("verification: "))
                .unwrap_or_default();
            if plan_verification.trim().is_empty() || plan_verification.trim() == "not_run" {
                issues.push("plan verification evidence is missing".to_string());
            }
            match latest_proof(repo_root)? {
                Some(proof) => {
                    let trusted =
                        plan.risk == RiskLane::Low || proof_has_current_receipt(repo_root, &proof)?;
                    if !proof_satisfies_risk(&proof.summary, plan.risk) || !trusted {
                        issues.push("proof does not satisfy the plan risk".to_string());
                    }
                    if plan.risk != RiskLane::Low {
                        match (plan.binding.as_ref(), proof_receipt_context(&proof)?) {
                            (Some(expected), Some((_, binding)))
                                if binding.task_id == expected.task_id
                                    && binding.operation_id == expected.operation_id
                                    && binding.adapter == expected.adapter
                                    && binding.session_id == expected.session_id
                                    && binding.request_id == expected.request_id => {}
                            _ => issues.push(
                                "proof operation identity does not match the active plan"
                                    .to_string(),
                            ),
                        }
                    }
                }
                None => issues.push("proof is missing".to_string()),
            }
            if plan.risk != RiskLane::Low {
                let required_agents = [
                    "code-reviewer".to_string(),
                    "security-auditor".to_string(),
                    "test-engineer".to_string(),
                ];
                let gate_scope = latest_proof(repo_root)?
                    .as_ref()
                    .map(proof_receipt_context)
                    .transpose()?
                    .flatten();
                let gates_passed = gate_scope
                    .as_ref()
                    .map(|(_, binding)| {
                        gate_evidence_status_strict_for_operation(
                            repo_root,
                            &required_agents,
                            &binding.task_id,
                            &binding.operation_id,
                            &binding.adapter,
                            Some(&binding.session_id),
                            Some(&binding.request_id),
                        )
                        .map(|status| status.passed)
                    })
                    .transpose()?
                    .unwrap_or(false);
                if !gates_passed {
                    issues.push("trusted quality-gate evidence is missing".to_string());
                }
            }
            let required = required_tier(plan.risk);
            match latest_trace_score(repo_root)? {
                Some(trace) if trace.passed && trace.achieved >= required => {}
                _ => issues.push("passing trace is missing".to_string()),
            }
        }
        _ => issues.push("linked plan file is missing".to_string()),
    }
    Ok(issues)
}

fn write_current(repo_root: &Path, vault: &VaultContext, view: CurrentPlanView<'_>) -> Result<()> {
    let task_id = view
        .binding
        .map(|binding| binding.task_id.clone())
        .unwrap_or_else(|| format!("task-{}", slugify(view.title)));
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
    let title = field(&content, "- Title: ").unwrap_or_default();
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
    let status = field(&content, "- Status: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string))
        .unwrap_or_else(|| "in_progress".to_string());
    let risk = if content.contains("- Risk: `high`") {
        RiskLane::High
    } else if content.contains("- Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    };
    let task_id = field(&content, "- Task ID: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string));
    let operation_id = field(&content, "- Operation ID: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string));
    let adapter = field(&content, "- Adapter: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string));
    let session_id = field(&content, "- Session ID: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string));
    let request_id = field(&content, "- Request ID: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string));
    let binding = match (task_id, operation_id, adapter, session_id, request_id) {
        (Some(task_id), Some(operation_id), Some(adapter), Some(session_id), Some(request_id)) => {
            Some(PlanOperationBinding {
                task_id,
                operation_id,
                adapter,
                session_id,
                request_id,
            })
        }
        _ => None,
    };
    Ok(path.map(|path| ActivePlan {
        title,
        path,
        status,
        risk,
        binding,
    }))
}

fn require_active_plan(repo_root: &Path) -> Result<ActivePlan> {
    active_plan(repo_root)?.context("No active Baron plan. Run `baron plan start \"<title>\"`.")
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

fn plan_content(title: &str, risk: RiskLane, binding: Option<&PlanOperationBinding>) -> String {
    let task_id = binding
        .map(|binding| binding.task_id.clone())
        .unwrap_or_else(|| format!("task-{}", slugify(title)));
    let operation_identity = binding
        .map(|binding| {
            format!(
                "operation_id: {}\nadapter: {}\nsession_id: {}\nrequest_id: {}\n",
                binding.operation_id, binding.adapter, binding.session_id, binding.request_id
            )
        })
        .unwrap_or_default();
    format!(
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
    )
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
    binding: Option<PlanOperationBinding>,
}

struct CurrentPlanView<'a> {
    title: &'a str,
    risk: RiskLane,
    status: &'a str,
    plan_path: &'a Path,
    next_action: &'a str,
    verification: &'a str,
    binding: Option<&'a PlanOperationBinding>,
}
