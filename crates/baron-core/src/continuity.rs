use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::execution_receipt::ReceiptContext;
use crate::operation::{LifecycleIdentity, OperationContext};
use crate::plan::{
    indexed_active_plan_authority_for_binding, managed_active_plan_operation_binding,
    plan_status_for_identity, PlanOperationBinding,
};
use crate::proof::{latest_proof, proof_for_operation};
use crate::safe_io::{
    acquire_project_lock, append_text, artifact_instance_id, create_new_text, read_text,
    replace_text,
};
use crate::task_state::{
    canonical_plan_next, compile_task_state_for_operation, operation_scoped_source,
};
use crate::trace::{latest_trace_score, latest_trace_score_for_operation, TraceOperationBinding};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuityPacket {
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryOutcome {
    Failed,
    Blocked,
    Interrupted,
}

impl RecoveryOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Interrupted => "interrupted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoveryInput {
    pub outcome: RecoveryOutcome,
    pub root_cause: String,
    pub last_successful_step: String,
    pub evidence: Vec<String>,
    pub affected_files: Vec<String>,
    pub next_action: String,
    pub retry_conditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPacket {
    pub id: String,
    pub outcome: RecoveryOutcome,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
}

pub fn record_recovery(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    input: RecoveryInput,
) -> Result<RecoveryPacket> {
    record_recovery_internal(repo_root.as_ref(), vault, input, None)
}

/// Persist actionable recovery for one exact operation. Public input/packet
/// shapes remain compatible; selectors are an additive API, not new fields.
pub fn record_recovery_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    input: RecoveryInput,
    identity: &LifecycleIdentity,
) -> Result<RecoveryPacket> {
    record_recovery_internal(repo_root.as_ref(), vault, input, Some(identity))
}

fn record_recovery_internal(
    repo_root: &Path,
    vault: &VaultContext,
    mut input: RecoveryInput,
    explicit_identity: Option<&LifecycleIdentity>,
) -> Result<RecoveryPacket> {
    normalize_recovery_input(&mut input);
    validate_recovery_input(&input)?;
    let _lock = acquire_project_lock(repo_root)?;
    let selected = if explicit_identity.is_none() {
        managed_active_plan_operation_binding(repo_root)?
            .map(|binding| {
                binding
                    .to_operation_context()?
                    .lifecycle_identity(&vault.project_id)
                    .map_err(anyhow::Error::from)
            })
            .transpose()?
    } else {
        None
    };
    let identity = explicit_identity.or(selected.as_ref());
    if let Some(identity) = identity {
        anyhow::ensure!(
            identity.project_id() == vault.project_id,
            "recovery identity does not match Vault project"
        );
        indexed_active_plan_authority_for_binding(
            repo_root,
            &PlanOperationBinding::from_identity(identity),
        )?
        .context("no validated active plan matches recovery identity")?;
    }
    // Shared Vault projections/indexes are serialized after the checkout lock.
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let id = recovery_id(&input, identity)?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    let filename = format!("{id}.md");
    let repo_path = repo_root
        .join("docs/baron/continuity/recovery")
        .join(&date)
        .join(&filename);
    let vault_path = vault
        .project_root
        .join("Continuity/Recovery")
        .join(&date)
        .join(&filename);
    let existing = existing_mirrored_packet(&repo_path, &vault_path, identity)?;
    let resumed = existing.is_some();
    let content = match existing {
        Some(content) => content,
        None => render_recovery(repo_root, &id, &input, identity)?,
    };
    if read_text(&repo_path)?.is_none() {
        create_new_text(&repo_path, &content)?;
    }
    if !resumed {
        append_recovery_index(
            &repo_root.join("docs/baron/continuity/RECOVERY_INDEX.md"),
            &id,
            input.outcome,
            &repo_path,
            repo_root,
        )?;
        append_recovery_index(
            &vault.project_root.join("Continuity/RECOVERY_INDEX.md"),
            &id,
            input.outcome,
            &vault_path,
            &vault.project_root,
        )?;
    }
    if read_text(&vault_path)?.is_none() {
        create_new_text(&vault_path, &content)?;
    }
    if let Some(identity) = identity {
        let operation_path = operation_recovery_path(repo_root, identity);
        let capsule_path = vault
            .project_root
            .join("Continuity/Operations")
            .join(identity.operation_id())
            .join("RECOVERY.md");
        // An old delivery must not replace a newer recovery for this operation.
        if !resumed || read_text(&operation_path)?.is_none() {
            write(&operation_path, &content)?;
        }
        if !resumed || read_text(&capsule_path)?.is_none() {
            write(&capsule_path, &content)?;
        }
    }
    write(
        &repo_root.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        &content,
    )?;
    write(
        &vault.project_root.join("Continuity/CURRENT_RECOVERY.md"),
        &content,
    )?;
    Ok(RecoveryPacket {
        id,
        outcome: input.outcome,
        repo_path,
        vault_path,
        resumed,
    })
}

pub fn record_continuity_checkpoint(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    note: &str,
    adapter: &str,
) -> Result<ContinuityPacket> {
    record_continuity_checkpoint_internal(
        repo_root,
        vault,
        note,
        ResumePacketMetadata {
            adapter,
            session_id: None,
            request_id: None,
            event_key: None,
            identity: None,
            changed_files: &[],
        },
    )
}

/// Persist a separate immutable checkpoint for the exact operation. Shared
/// CURRENT is updated only as a human-facing latest projection.
pub fn record_continuity_checkpoint_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    note: &str,
    operation: &OperationContext,
) -> Result<ContinuityPacket> {
    let identity = checkpoint_identity(vault, operation)?;
    record_continuity_checkpoint_internal(
        repo_root,
        vault,
        note,
        ResumePacketMetadata {
            adapter: operation.adapter.as_str(),
            session_id: operation.session_id.as_deref(),
            request_id: operation.request_id.as_deref(),
            event_key: None,
            identity: identity.as_ref(),
            changed_files: &[],
        },
    )
}

/// Record a checkpoint tied to one normalized lifecycle event. Retrying the
/// same event is a byte-stable no-op once the event key is already present in
/// that operation's immutable event packet, even after another checkpoint.
pub fn record_continuity_checkpoint_for_event(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    note: &str,
    operation: &OperationContext,
    event_key: &str,
) -> Result<ContinuityPacket> {
    let identity = checkpoint_identity(vault, operation)?;
    record_continuity_checkpoint_internal(
        repo_root,
        vault,
        note,
        ResumePacketMetadata {
            adapter: operation.adapter.as_str(),
            session_id: operation.session_id.as_deref(),
            request_id: operation.request_id.as_deref(),
            event_key: Some(event_key),
            identity: identity.as_ref(),
            changed_files: &[],
        },
    )
}

fn checkpoint_identity(
    vault: &VaultContext,
    operation: &OperationContext,
) -> Result<Option<LifecycleIdentity>> {
    if operation.task_id.is_some()
        || operation.operation_id.is_some()
        || operation.session_id.is_some()
        || operation.request_id.is_some()
    {
        Ok(Some(operation.lifecycle_identity(&vault.project_id)?))
    } else {
        // Preserve adapter/session-only diagnostic callers without inferring
        // task or operation identity from the latest shared projection.
        Ok(None)
    }
}

fn record_continuity_checkpoint_internal(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    note: &str,
    metadata: ResumePacketMetadata<'_>,
) -> Result<ContinuityPacket> {
    let repo_root = repo_root.as_ref();
    let repo_path = repo_root.join("docs/baron/continuity/CURRENT.md");
    let vault_path = vault.project_root.join("Continuity/CURRENT.md");
    // Git status is diagnostic context and may spawn a child process. Collect
    // it before entering the shared continuity mutation critical section.
    let changed_files = changed_files(repo_root);
    let _lock = acquire_project_lock(repo_root)?;
    if let Some(identity) = metadata.identity {
        // Validate even retries: an event-key match cannot hide corrupted
        // ACTIVE/frontmatter authority or authorize a stale checkpoint.
        plan_status_for_identity(repo_root, identity)?;
    }
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    if let Some(identity) = metadata.identity {
        let id = match metadata.event_key {
            Some(key) => format!("event-{:x}", Sha256::digest(key.as_bytes())),
            None => artifact_instance_id(&Local::now().format("%Y%m%d").to_string())?,
        };
        let repo_path = repo_root
            .join("docs/baron/continuity/operations")
            .join(identity.operation_id())
            .join("checkpoints")
            .join(format!("{id}.md"));
        let vault_path = vault
            .project_root
            .join("Continuity/Operations")
            .join(identity.operation_id())
            .join("Checkpoints")
            .join(format!("{id}.md"));
        if let Some(content) = existing_mirrored_packet(&repo_path, &vault_path, Some(identity))? {
            if read_text(&repo_path)?.is_none() {
                create_new_text(&repo_path, &content)?;
            }
            if read_text(&vault_path)?.is_none() {
                create_new_text(&vault_path, &content)?;
            }
            // Repair an interrupted first publication without regressing a
            // newer per-operation pointer or another operation's CURRENT.
            let operation_path = operation_checkpoint_path(repo_root, identity);
            let capsule_path = vault
                .project_root
                .join("Continuity/Operations")
                .join(identity.operation_id())
                .join("CHECKPOINT.md");
            if read_text(&operation_path)?.is_none() {
                write(&operation_path, &content)?;
            }
            if read_text(&capsule_path)?.is_none() {
                write(&capsule_path, &content)?;
            }
            return Ok(ContinuityPacket {
                repo_path,
                vault_path,
            });
        }
        let content = render_resume_packet(
            repo_root,
            vault,
            note,
            ResumePacketMetadata {
                changed_files: &changed_files,
                ..metadata
            },
        )?;
        create_new_text(&repo_path, &content)?;
        if read_text(&vault_path)?.is_none() {
            create_new_text(&vault_path, &content)?;
        }
        write(&operation_checkpoint_path(repo_root, identity), &content)?;
        write(
            &vault
                .project_root
                .join("Continuity/Operations")
                .join(identity.operation_id())
                .join("CHECKPOINT.md"),
            &content,
        )?;
        write(
            &repo_root.join("docs/baron/continuity/CURRENT.md"),
            &content,
        )?;
        write(&vault.project_root.join("Continuity/CURRENT.md"), &content)?;
        append_index(
            &repo_root.join("docs/baron/continuity/INDEX.md"),
            note.trim(),
            &repo_path,
            repo_root,
        )?;
        append_index(
            &vault.project_root.join("Continuity/INDEX.md"),
            note.trim(),
            &vault_path,
            &vault.project_root,
        )?;
        return Ok(ContinuityPacket {
            repo_path,
            vault_path,
        });
    }
    if let Some(event_key) = metadata.event_key {
        if checkpoint_has_event_key(&repo_path, event_key)?
            && metadata.identity.is_none_or(|identity| {
                !operation_scoped_source(&repo_path, identity, 8_000).is_empty()
            })
        {
            return Ok(ContinuityPacket {
                repo_path,
                vault_path,
            });
        }
    }
    let metadata = ResumePacketMetadata {
        changed_files: &changed_files,
        ..metadata
    };
    let content = render_resume_packet(repo_root, vault, note, metadata)?;
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
    append_index(
        &repo_root.join("docs/baron/continuity/INDEX.md"),
        note.trim(),
        &repo_path,
        repo_root,
    )?;
    append_index(
        &vault.project_root.join("Continuity/INDEX.md"),
        note.trim(),
        &vault_path,
        &vault.project_root,
    )?;
    Ok(ContinuityPacket {
        repo_path,
        vault_path,
    })
}

pub fn continuity_status(repo_root: impl AsRef<Path>, vault: &VaultContext) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let current = repo_root.join("docs/baron/continuity/CURRENT.md");
    let body = fs::read_to_string(&current).unwrap_or_else(|_| {
        "# Baron Continuity Resume\n\n- Status: no checkpoint recorded\n- Next action: inspect context, plan, harness, proof, and trace before editing\n".to_string()
    });
    let recovery = bounded_read(
        &repo_root.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        2_400,
        "- Status: no recovery packet recorded",
    );
    Ok(format!(
        "# Baron Continuity Status\n\n- Repo packet: `{}`\n- Vault packet: `{}`\n\n{}\n\n## Current Recovery\n\n{}\n",
        current.display(),
        vault.project_root.join("Continuity/CURRENT.md").display(),
        body.trim(),
        recovery.trim()
    ))
}

struct ResumePacketMetadata<'a> {
    adapter: &'a str,
    session_id: Option<&'a str>,
    request_id: Option<&'a str>,
    event_key: Option<&'a str>,
    identity: Option<&'a LifecycleIdentity>,
    changed_files: &'a [String],
}

fn render_resume_packet(
    repo_root: &Path,
    vault: &VaultContext,
    note: &str,
    metadata: ResumePacketMetadata<'_>,
) -> Result<String> {
    if let Some(identity) = metadata.identity {
        return render_operation_resume_packet(repo_root, vault, note, identity, &metadata);
    }
    let plan = read_optional(&repo_root.join("docs/baron/plans/CURRENT.md"));
    let harness = read_optional(&repo_root.join("docs/baron/harness/CURRENT.md"));
    let proof = latest_proof(repo_root)?;
    let trace = latest_trace_score(repo_root)?;
    let latest_event = latest_automation_event(vault);
    let recovery = read_optional(&repo_root.join("docs/baron/continuity/CURRENT_RECOVERY.md"));

    let plan_title = field(&plan, "- Title: ").unwrap_or("unknown");
    let plan_status = field(&plan, "- Status: `")
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("unknown");
    let plan_next = field(&plan, "- Next action: ").unwrap_or("inspect current plan");
    let harness_title = field(&harness, "- Title: ").unwrap_or("unknown");
    let harness_risk = field(&harness, "- Risk: `")
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("unknown");
    let proof_status = proof
        .as_ref()
        .map(|value| format!("recorded `{}` - {}", value.id, single_line(&value.summary)))
        .unwrap_or_else(|| "missing".to_string());
    let trace_status = trace
        .as_ref()
        .map(|value| {
            format!(
                "scored `{}/{}` passed `{}`",
                value.achieved.as_str(),
                value.required.as_str(),
                if value.passed { "yes" } else { "no" }
            )
        })
        .unwrap_or_else(|| "missing".to_string());
    let next_action = if plan_next == "inspect current plan" {
        "read this resume, inspect plan/harness, then continue only with evidence"
    } else {
        plan_next
    };
    let recovery_outcome = field(&recovery, "- Outcome: `")
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("none");
    let recovery_next =
        section_first_line(&recovery, "## Safe Next Action").unwrap_or("none recorded");

    Ok(format!(
        "# Baron Continuity Resume\n\n\
- Last updated: {}\n\
- Adapter: `{}`\n\
- Session ID: `{}`\n\
- Request ID: `{}`\n\
- Lifecycle event key: `{}`\n\
- Latest checkpoint: {}\n\
- Latest automation event: `{}`\n\
- Current task: `{}`\n\
- Plan status: `{}`\n\
- Harness story: `{}`\n\
- Harness risk: `{}`\n\
- Proof status: {}\n\
- Trace status: {}\n\
- Recovery outcome: `{}`\n\
- Recovery next action: {}\n\
- Changed files: {}\n\
- Next action: {}\n\n\
## Resume Rules\n\n\
- Do not infer completion from silence, shutdown, network loss, or quota exhaustion.\n\
- Before editing, reconcile this packet with repo files and bounded context.\n\
- If proof or trace is missing for meaningful work, continue or interrupt; do not claim completion.\n\
- If the task scope changed, start a new explicit plan and write a new checkpoint.\n",
        now(),
        metadata.adapter.trim(),
        metadata.session_id.unwrap_or("none"),
        metadata.request_id.unwrap_or("none"),
        metadata.event_key.unwrap_or("none"),
        single_line(note),
        latest_event.unwrap_or_else(|| "none".to_string()),
        plan_title,
        plan_status,
        harness_title,
        harness_risk,
        proof_status,
        trace_status,
        recovery_outcome,
        recovery_next,
        list_or_none(metadata.changed_files),
        next_action
    ))
}

fn render_operation_resume_packet(
    repo_root: &Path,
    vault: &VaultContext,
    note: &str,
    identity: &LifecycleIdentity,
    metadata: &ResumePacketMetadata<'_>,
) -> Result<String> {
    let plan = plan_status_for_identity(repo_root, identity)?;
    let title = field(&plan, "- Title: ");
    let state = title
        .map(|task| compile_task_state_for_operation(repo_root, vault, identity, Some(task)))
        .transpose()?;
    let proof = proof_for_operation(repo_root, &ReceiptContext::for_identity(identity, "proof")?)?;
    let trace = proof
        .as_ref()
        .map(|proof| {
            let binding = TraceOperationBinding::from_operation(
                &OperationContext::from_identity(identity),
                &proof.id,
            )?;
            latest_trace_score_for_operation(repo_root, &binding)
        })
        .transpose()?
        .flatten();
    let recovery = operation_scoped_source(
        &operation_recovery_path(repo_root, identity),
        identity,
        3_600,
    );
    let proof_status = proof
        .map(|proof| format!("recorded `{}` - {}", proof.id, single_line(&proof.summary)))
        .unwrap_or_else(|| "unknown".to_string());
    let trace_status = trace
        .map(|trace| {
            format!(
                "scored `{}/{}` passed `{}`",
                trace.achieved.as_str(),
                trace.required.as_str(),
                if trace.passed { "yes" } else { "no" }
            )
        })
        .unwrap_or_else(|| "unknown".to_string());
    let next = section_first_line(&recovery, "## Safe Next Action")
        .map(str::to_string)
        .or_else(|| canonical_plan_next(&plan))
        .unwrap_or_else(|| {
            "unknown; inspect exact operation authorities before acting".to_string()
        });
    Ok(format!(
        "# Baron Continuity Resume\n\n- Last updated: {}\n- Project ID: `{}`\n- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n- Lifecycle event key: `{}`\n- Latest checkpoint: {}\n- Latest automation event: `unknown`\n- Current task: `{}`\n- Plan status: {}\n- Harness story: `unknown`\n- Harness risk: `unknown`\n- Proof status: {}\n- Trace status: {}\n- Recovery outcome: {}\n- Recovery next action: {}\n- Operation gate state: {}\n- Changed files: {}\n- Next action: {}\n\n## Resume Rules\n\n- Resume only when the full operation binding matches.\n- Unknown evidence cannot authorize completion.\n",
        now(), identity.project_id(), identity.task_id(), identity.operation_id(), identity.adapter().as_str(), identity.session_id(), identity.request_id(),
        metadata.event_key.unwrap_or("none"), single_line(note), title.unwrap_or("unknown"), field(&plan, "- Status: ").unwrap_or("`unknown`"), proof_status, trace_status,
        field(&recovery, "- Outcome: ").unwrap_or("`unknown`"), section_first_line(&recovery, "## Safe Next Action").unwrap_or("unknown"),
        state.as_ref().map(|state| list_or_none(&state.unknowns)).unwrap_or_else(|| "unknown".to_string()),
        // Git status is project-wide diagnostic data, never an operation's
        // affected-file authority. The scoped Task State carries exact files.
        state.as_ref().map(|state| list_or_none(&state.affected_files)).unwrap_or_else(|| "none".to_string()), next,
    ))
}

fn render_recovery(
    repo_root: &Path,
    id: &str,
    input: &RecoveryInput,
    identity: Option<&LifecycleIdentity>,
) -> Result<String> {
    let plan = match identity {
        Some(identity) => plan_status_for_identity(repo_root, identity)?,
        None => read_optional(&repo_root.join("docs/baron/plans/CURRENT.md")),
    };
    let harness = if identity.is_none() {
        read_optional(&repo_root.join("docs/baron/harness/CURRENT.md"))
    } else {
        String::new()
    };
    let proof = match identity {
        Some(identity) => {
            proof_for_operation(repo_root, &ReceiptContext::for_identity(identity, "proof")?)?
        }
        None => latest_proof(repo_root)?.filter(|proof| proof.binding.is_none()),
    };
    let trace = match identity {
        Some(identity) => proof
            .as_ref()
            .map(|proof| {
                let binding = TraceOperationBinding::from_operation(
                    &OperationContext::from_identity(identity),
                    &proof.id,
                )?;
                latest_trace_score_for_operation(repo_root, &binding)
            })
            .transpose()?
            .flatten(),
        None => latest_trace_score(repo_root)?.filter(|trace| trace.binding.is_none()),
    };
    let provenance = identity.map(|identity| format!("- Project ID: `{}`\n- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n", identity.project_id(), identity.task_id(), identity.operation_id(), identity.adapter().as_str(), identity.session_id(), identity.request_id())).unwrap_or_default();
    let plan_title = field(&plan, "- Title: ").unwrap_or("unknown");
    let harness_title = field(&harness, "- Title: ").unwrap_or("unknown");
    let harness_risk = field(&harness, "- Risk: `")
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("unknown");
    let proof_state = proof
        .map(|value| format!("{} - {}", value.id, single_line(&value.summary)))
        .unwrap_or_else(|| "missing".to_string());
    let trace_state = trace
        .map(|value| {
            format!(
                "{}/{} passed {}",
                value.achieved.as_str(),
                value.required.as_str(),
                if value.passed { "yes" } else { "no" }
            )
        })
        .unwrap_or_else(|| "missing".to_string());
    Ok(format!(
        "# Baron Actionable Recovery\n\n\
- Recovery ID: `{id}`\n\
- Binding: {}\n\
{}\
- Outcome: `{}`\n\
- Recorded: {}\n\n\
## Root Cause\n\n{}\n\n\
## Last Successful Step\n\n{}\n\n\
## Evidence\n\n{}\n\n\
## Affected Files\n\n{}\n\n\
## Safe Next Action\n\n{}\n\n\
## Retry Conditions\n\n{}\n\n\
## Linked State\n\n\
- Plan: `{}`\n\
- Harness story: `{}`\n\
- Harness risk: `{}`\n\
- Proof: {}\n\
- Trace: {}\n\n\
## Recovery Rules\n\n\
- Preserve this failed attempt even after a later retry succeeds.\n\
- Reconcile repo state before retrying.\n\
- Do not claim completion until required proof and trace pass.\n",
        if identity.is_some() {
            "exact operation"
        } else {
            "legacy unbound"
        },
        provenance,
        input.outcome.as_str(),
        now(),
        input.root_cause,
        input.last_successful_step,
        markdown_list(&input.evidence),
        markdown_list(&input.affected_files),
        input.next_action,
        markdown_list(&input.retry_conditions),
        plan_title,
        harness_title,
        harness_risk,
        proof_state,
        trace_state
    ))
}

fn validate_recovery_input(input: &RecoveryInput) -> Result<()> {
    for (name, value) in [
        ("root cause", input.root_cause.as_str()),
        ("last successful step", input.last_successful_step.as_str()),
        ("safe next action", input.next_action.as_str()),
    ] {
        if value.is_empty() {
            anyhow::bail!("Recovery {name} must not be empty.");
        }
    }
    Ok(())
}

fn normalize_recovery_input(input: &mut RecoveryInput) {
    input.root_cause = single_line(&input.root_cause);
    input.last_successful_step = single_line(&input.last_successful_step);
    input.next_action = single_line(&input.next_action);
    for values in [
        &mut input.evidence,
        &mut input.affected_files,
        &mut input.retry_conditions,
    ] {
        *values = values
            .iter()
            .map(|value| single_line(value))
            .filter(|value| !value.is_empty())
            .collect();
    }
}

fn recovery_id(input: &RecoveryInput, identity: Option<&LifecycleIdentity>) -> Result<String> {
    let bytes = match identity {
        Some(identity) => serde_json::to_vec(&(identity.operation_id(), input))?,
        None => serde_json::to_vec(input)?,
    };
    let digest = Sha256::digest(bytes);
    let suffix = digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("recovery-{suffix}"))
}

pub(crate) fn operation_checkpoint_path(repo_root: &Path, identity: &LifecycleIdentity) -> PathBuf {
    repo_root
        .join("docs/baron/continuity/operations")
        .join(identity.operation_id())
        .join("CHECKPOINT.md")
}

pub(crate) fn operation_recovery_path(repo_root: &Path, identity: &LifecycleIdentity) -> PathBuf {
    repo_root
        .join("docs/baron/continuity/operations")
        .join(identity.operation_id())
        .join("RECOVERY.md")
}

fn existing_mirrored_packet(
    repo_path: &Path,
    vault_path: &Path,
    identity: Option<&LifecycleIdentity>,
) -> Result<Option<String>> {
    let repo = read_text(repo_path)?;
    let vault = read_text(vault_path)?;
    if let (Some(repo), Some(vault)) = (&repo, &vault) {
        anyhow::ensure!(
            repo == vault,
            "conflicting repo/Vault continuity artifact; preserve both and recover explicitly"
        );
    }
    if let Some(identity) = identity {
        for (path, content) in [(repo_path, &repo), (vault_path, &vault)] {
            if content.is_some() {
                anyhow::ensure!(
                    !operation_scoped_source(path, identity, 8_000).is_empty(),
                    "continuity artifact binding mismatch"
                );
            }
        }
    }
    Ok(repo.or(vault))
}

fn append_recovery_index(
    path: &Path,
    id: &str,
    outcome: RecoveryOutcome,
    packet: &Path,
    root: &Path,
) -> Result<()> {
    let item = format!(
        "- {} - [{}]({}) - outcome: `{}`",
        now(),
        id,
        normalize(packet, root),
        outcome.as_str()
    );
    let header = "# Baron Recovery Index\n\n";
    let content = match read_text(path)? {
        Some(content) => content,
        None => {
            replace_text(path, header)?;
            header.to_string()
        }
    };
    if content
        .lines()
        .any(|line| line.contains(&format!("[{id}]")))
    {
        return Ok(());
    }
    let separator = if content.is_empty() || content.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    append_text(path, &format!("{separator}{item}\n"))
}

fn markdown_list(values: &[String]) -> String {
    if values.is_empty() {
        "- none recorded".to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn section_first_line<'a>(content: &'a str, heading: &str) -> Option<&'a str> {
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if line == heading {
            return lines.find(|value| !value.trim().is_empty()).map(str::trim);
        }
    }
    None
}

fn bounded_read(path: &Path, limit: usize, missing: &str) -> String {
    let content = fs::read_to_string(path).unwrap_or_else(|_| missing.to_string());
    if content.chars().count() <= limit {
        content
    } else {
        format!(
            "{}\n- recovery body truncated for bounded status\n",
            content.chars().take(limit).collect::<String>()
        )
    }
}

fn read_optional(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn field<'a>(content: &'a str, prefix: &str) -> Option<&'a str> {
    content.lines().find_map(|line| line.strip_prefix(prefix))
}

fn latest_automation_event(vault: &VaultContext) -> Option<String> {
    let path = vault
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    fs::read_to_string(path)
        .ok()?
        .lines()
        .rev()
        .find_map(|line| {
            let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
            value
                .get("event")
                .and_then(|event| event.as_str())
                .map(pretty_event)
        })
}

fn pretty_event(event: &str) -> String {
    match event {
        "session_start" => "SessionStart".to_string(),
        "checkpoint" => "Checkpoint".to_string(),
        "prompt" => "Prompt".to_string(),
        "context_compiled" => "ContextCompiled".to_string(),
        "plan_started" => "PlanStarted".to_string(),
        "harness_started" => "HarnessStarted".to_string(),
        "proof_recorded" => "ProofRecorded".to_string(),
        "trace_scored" => "TraceScored".to_string(),
        "stop" => "Stop".to_string(),
        other => other.to_string(),
    }
}

fn changed_files(repo_root: &Path) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(repo_root)
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(12)
        .map(str::to_string)
        .collect()
}

fn append_index(path: &Path, note: &str, current: &Path, root: &Path) -> Result<()> {
    let row = format!(
        "- {} - [{}]({}) - {}",
        now(),
        current
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("checkpoint"),
        normalize(current, root),
        single_line(note)
    );
    let header = "# Baron Continuity Index\n\n";
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
    append_text(path, &format!("{separator}{row}\n"))
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn checkpoint_has_event_key(path: &Path, event_key: &str) -> Result<bool> {
    let expected = format!("- Lifecycle event key: `{event_key}`");
    match fs::read_to_string(path) {
        Ok(content) => Ok(content.lines().any(|line| line == expected)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| {
            format!(
                "Could not inspect continuity checkpoint: {}",
                path.display()
            )
        }),
    }
}

fn normalize(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
