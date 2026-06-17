use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};

use crate::proof::latest_proof;
use crate::trace::latest_trace_score;
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuityPacket {
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
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
    Ok(format!(
        "# Baron Continuity Status\n\n- Repo packet: `{}`\n- Vault packet: `{}`\n\n{}",
        current.display(),
        vault.project_root.join("Continuity/CURRENT.md").display(),
        body
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
        list_or_none(&changed_files),
        next_action
    ))
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
