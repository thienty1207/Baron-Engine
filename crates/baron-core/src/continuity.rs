use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::proof::latest_proof;
use crate::trace::latest_trace_score;
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
    mut input: RecoveryInput,
) -> Result<RecoveryPacket> {
    let repo_root = repo_root.as_ref();
    normalize_recovery_input(&mut input);
    validate_recovery_input(&input)?;
    let id = recovery_id(&input)?;
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
    let resumed = repo_path.is_file();
    let content = render_recovery(repo_root, &id, &input)?;
    if !resumed {
        write(&repo_path, &content)?;
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
    if !vault_path.is_file() {
        write(&vault_path, &content)?;
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
    let repo_root = repo_root.as_ref();
    let content = render_resume_packet(repo_root, vault, note, adapter)?;
    let repo_path = repo_root.join("docs/baron/continuity/CURRENT.md");
    let vault_path = vault.project_root.join("Continuity/CURRENT.md");
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

fn render_resume_packet(
    repo_root: &Path,
    vault: &VaultContext,
    note: &str,
    adapter: &str,
) -> Result<String> {
    let plan = read_optional(&repo_root.join("docs/baron/plans/CURRENT.md"));
    let harness = read_optional(&repo_root.join("docs/baron/harness/CURRENT.md"));
    let proof = latest_proof(repo_root)?;
    let trace = latest_trace_score(repo_root)?;
    let latest_event = latest_automation_event(vault);
    let changed_files = changed_files(repo_root);
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
        adapter.trim(),
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
        list_or_none(&changed_files),
        next_action
    ))
}

fn render_recovery(repo_root: &Path, id: &str, input: &RecoveryInput) -> Result<String> {
    let plan = read_optional(&repo_root.join("docs/baron/plans/CURRENT.md"));
    let harness = read_optional(&repo_root.join("docs/baron/harness/CURRENT.md"));
    let proof = latest_proof(repo_root)?;
    let trace = latest_trace_score(repo_root)?;
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

fn recovery_id(input: &RecoveryInput) -> Result<String> {
    let digest = Sha256::digest(serde_json::to_vec(input)?);
    let suffix = digest
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("recovery-{suffix}"))
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
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Recovery Index\n\n".to_string());
    if content
        .lines()
        .any(|line| line.contains(&format!("[{id}]")))
    {
        return Ok(());
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&item);
    content.push('\n');
    write(path, &content)
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
        "CURRENT",
        normalize(current, root),
        single_line(note)
    );
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Continuity Index\n\n".to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&row);
    content.push('\n');
    write(path, &content)
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
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
