use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};

use crate::proof::latest_proof;
use crate::trace::latest_trace_score;
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessAudit {
    pub context_read_score: u8,
    pub diagnostics: Vec<String>,
    pub open_friction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterventionRecord {
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

pub fn audit_harness(repo_root: impl AsRef<Path>, vault: &VaultContext) -> Result<HarnessAudit> {
    let repo_root = repo_root.as_ref();
    let journal = fs::read_to_string(
        vault
            .project_root
            .join("Artifacts/automation-journal.jsonl"),
    )
    .unwrap_or_default();
    let context_observed = journal.contains("context_compiled");
    let plan_observed =
        journal.contains("plan_started") || repo_root.join("docs/baron/plans/CURRENT.md").exists();
    let harness_observed = journal.contains("harness_started")
        || repo_root.join("docs/baron/harness/CURRENT.md").exists();

    let mut score = 0;
    if context_observed {
        score += 50;
    }
    if plan_observed {
        score += 25;
    }
    if harness_observed {
        score += 25;
    }

    let mut diagnostics = Vec::new();
    if !context_observed {
        diagnostics.push("context was not observed in the automation journal".to_string());
    }
    if repo_root.join("docs/baron/plans/CURRENT.md").exists() && latest_proof(repo_root)?.is_none()
    {
        diagnostics.push("active work proof is missing".to_string());
    }
    if repo_root.join("docs/baron/plans/CURRENT.md").exists()
        && latest_trace_score(repo_root)?.is_none()
    {
        diagnostics.push("passing trace score is missing".to_string());
    }

    let friction =
        fs::read_to_string(repo_root.join("docs/baron/harness/FRICTION.md")).unwrap_or_default();
    let open_friction_count = friction
        .lines()
        .filter(|line| line.starts_with("- [ ]"))
        .count();
    if open_friction_count > 0 {
        diagnostics.push(format!("open friction items: {open_friction_count}"));
    }
    if documentation_drift(repo_root) {
        diagnostics.push("documentation drift detected between status files".to_string());
    }

    Ok(HarnessAudit {
        context_read_score: score,
        diagnostics,
        open_friction_count,
    })
}

pub fn record_intervention(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<InterventionRecord> {
    let repo_path = repo_root
        .as_ref()
        .join("docs/baron/harness/INTERVENTIONS.md");
    let vault_path = vault.project_root.join("ProductHarness/INTERVENTIONS.md");
    let item = format!("- {} - {}", now(), summary.trim());
    append(
        &repo_path,
        "# Baron Harness Interventions\n\nHuman, reviewer, CI, and agent corrections are recorded here for later improvement analysis.\n\n",
        &item,
    )?;
    append(
        &vault_path,
        "# Baron Harness Interventions\n\nHuman, reviewer, CI, and agent corrections are recorded here for later improvement analysis.\n\n",
        &item,
    )?;
    Ok(InterventionRecord {
        repo_path,
        vault_path,
    })
}

fn documentation_drift(repo_root: &Path) -> bool {
    let markdown = fs::read_to_string(repo_root.join("docs/BARON_STATUS.md")).unwrap_or_default();
    let json = fs::read_to_string(repo_root.join("docs/BARON_STATUS.json")).unwrap_or_default();
    (markdown.contains("completed") && json.contains("in_progress"))
        || (markdown.contains("in progress") && json.contains("\"completed\""))
        || (markdown.contains("planned") && json.contains("\"completed\""))
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    content.push('\n');
    write(path, &content)
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
