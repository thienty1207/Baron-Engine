use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};
use serde_json::Value;

use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopilotReview {
    pub candidate_count: usize,
    pub candidate_ids: Vec<String>,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub approval_required: bool,
    pub observed_automation: Vec<String>,
    pub resume_sources: Vec<String>,
}

pub fn review_after_task(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<AutopilotReview> {
    let repo_root = repo_root.as_ref();
    let summary = single_line(summary);
    let summary = if summary.is_empty() {
        "post-task review captured with no summary".to_string()
    } else {
        summary
    };
    let observed_automation = observed_automation(vault);
    let resume_sources = resume_sources(repo_root);
    let candidate_ids = build_candidate_ids(&summary);
    let approval_required = true;
    let repo_path = repo_root.join("docs/baron/autopilot/CANDIDATES.md");
    let vault_path = vault.project_root.join("Autopilot/CANDIDATES.md");
    let entry = render_candidate_entry(
        &candidate_ids,
        &summary,
        approval_required,
        &observed_automation,
        &resume_sources,
    );
    append(
        &repo_path,
        "# Baron Autopilot Learning Candidates\n\nCandidates are not trusted facts. They require approval before becoming durable policy, memory, or runtime asset changes.\n\n",
        &entry,
    )?;
    append(
        &vault_path,
        "# Baron Autopilot Learning Candidates\n\nCandidates are not trusted facts. They require approval before becoming durable policy, memory, or runtime asset changes.\n\n",
        &entry,
    )?;
    Ok(AutopilotReview {
        candidate_count: candidate_ids.len(),
        candidate_ids,
        repo_path,
        vault_path,
        approval_required,
        observed_automation,
        resume_sources,
    })
}

pub fn approve_candidate(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    candidate_id: &str,
) -> Result<()> {
    update_candidate_status(
        &repo_root
            .as_ref()
            .join("docs/baron/autopilot/CANDIDATES.md"),
        candidate_id,
        "approved",
    )?;
    update_candidate_status(
        &vault.project_root.join("Autopilot/CANDIDATES.md"),
        candidate_id,
        "approved",
    )?;
    let approved = format!(
        "## {}\n\n- Approved: {}\n- Source candidate: `{}`\n- Trusted fact: `not automatically`; use the approved item only as evidence-backed guidance.\n\n",
        candidate_id,
        now(),
        candidate_id
    );
    append(
        &repo_root.as_ref().join("docs/baron/autopilot/APPROVED.md"),
        "# Baron Approved Autopilot Learning\n\nApproved candidates stay separated from facts unless a human or explicit command promotes them into project memory.\n\n",
        &approved,
    )?;
    append(
        &vault.project_root.join("Autopilot/APPROVED.md"),
        "# Baron Approved Autopilot Learning\n\nApproved candidates stay separated from facts unless a human or explicit command promotes them into project memory.\n\n",
        &approved,
    )
}

pub fn reject_candidate(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    candidate_id: &str,
) -> Result<()> {
    update_candidate_status(
        &repo_root
            .as_ref()
            .join("docs/baron/autopilot/CANDIDATES.md"),
        candidate_id,
        "rejected",
    )?;
    update_candidate_status(
        &vault.project_root.join("Autopilot/CANDIDATES.md"),
        candidate_id,
        "rejected",
    )
}

pub fn autopilot_status(repo_root: impl AsRef<Path>, vault: &VaultContext) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let candidates = fs::read_to_string(repo_root.join("docs/baron/autopilot/CANDIDATES.md"))
        .unwrap_or_default();
    let candidate_count = candidates
        .lines()
        .filter(|line| line.starts_with("## "))
        .count();
    let open_count = candidates
        .lines()
        .filter(|line| line.contains("Status: `candidate`"))
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
    output.push_str("- Trusted fact policy: `candidates are not facts until approved`\n");
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
    output.push_str("## Resume Point\n\n");
    output.push_str(&truncate(&continuity, 1_400));
    output.push_str("\n\n## Rules\n\n");
    output.push_str(
        "- Do not infer completion from silence, shutdown, quota exhaustion, or network loss.\n",
    );
    output.push_str("- Treat candidates as candidates until approved.\n");
    output.push_str(
        "- Runtime-affecting changes require explicit approval metadata before activation.\n",
    );
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
        Err(error) => format!("## Autopilot Learning And Resume\n\n- unavailable: {error}\n\n"),
    }
}

fn build_candidate_ids(summary: &str) -> Vec<String> {
    let base = if contains_any(summary, &["skill", "agent", "routing", "instruction"]) {
        "skill-routing"
    } else if contains_any(summary, &["proof", "trace", "verification", "test"]) {
        "proof-continuity"
    } else {
        "memory"
    };
    vec![format!("candidate-{}-{base}", nanos())]
}

fn render_candidate_entry(
    ids: &[String],
    summary: &str,
    approval_required: bool,
    observed_automation: &[String],
    resume_sources: &[String],
) -> String {
    let mut output = String::new();
    for id in ids {
        output.push_str(&format!(
            "## {id}\n\n\
- Status: `candidate`\n\
- Trusted fact: `no`\n\
- Approval required: `{}`\n\
- Created: {}\n\
- Summary: {}\n\
- Observed automation: {}\n\
- Resume sources: {}\n\
- Safe action: keep as candidate until approved; do not rewrite skills, agents, memory facts, or runtime policy from this item alone.\n\n",
            if approval_required { "yes" } else { "no" },
            now(),
            summary,
            values_or_none(observed_automation),
            values_or_none(resume_sources)
        ));
    }
    output
}

fn update_candidate_status(path: &Path, candidate_id: &str, status: &str) -> Result<()> {
    let mut content =
        fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
    let heading = format!("## {}", candidate_id.trim());
    if !content.contains(&heading) {
        anyhow::bail!("Candidate not found: {}", candidate_id);
    }
    let mut in_target = false;
    let mut updated = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        if line.starts_with("## ") {
            in_target = line.trim() == heading;
        }
        if in_target && line.starts_with("- Status: `") {
            lines.push(format!("- Status: `{status}`"));
            updated = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if updated {
        content = lines.join("\n");
        content.push('\n');
        write(path, &content)?;
    }
    Ok(())
}

fn observed_automation(vault: &VaultContext) -> Vec<String> {
    let path = vault
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    let mut events = fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|value| value.get("event").and_then(Value::as_str).map(pretty_event))
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
        "docs/baron/plans/CURRENT.md",
        "docs/baron/harness/CURRENT.md",
        "docs/baron/proofs/INDEX.md",
        "docs/baron/traces/INDEX.md",
    ]
    .iter()
    .filter(|relative| repo_root.join(relative).exists())
    .map(|relative| (*relative).to_string())
    .collect()
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    if !content.ends_with('\n') {
        content.push('\n');
    }
    write(path, &content)
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    let lower = value.to_lowercase();
    needles.iter().any(|needle| lower.contains(needle))
}

fn values_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn truncate(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
