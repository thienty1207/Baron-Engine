use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationsRunbook {
    pub path: PathBuf,
    pub scope: String,
    pub start_command: String,
    pub readiness: String,
    pub interface: String,
    pub runtime_evidence: String,
    pub cleanup: String,
}

pub fn load_runbook(repo_root: impl AsRef<Path>) -> Result<Option<OperationsRunbook>> {
    let path = repo_root.as_ref().join("docs/baron/operations/RUNBOOK.md");
    if !path.is_file() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    Ok(Some(OperationsRunbook {
        path,
        scope: field(&content, "Scope"),
        start_command: field(&content, "Start command"),
        readiness: field(&content, "Readiness"),
        interface: field(&content, "Real interface"),
        runtime_evidence: field(&content, "Runtime evidence"),
        cleanup: field(&content, "Owned cleanup"),
    }))
}

pub fn relevant_to_task(task: &str) -> bool {
    let lower = task.to_lowercase();
    [
        "run",
        "operate",
        "runtime",
        "end-to-end",
        "e2e",
        "deploy",
        "readiness",
        "reproduce",
        "khởi động",
        "chạy ứng dụng",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

pub fn render_bounded_context(runbook: &OperationsRunbook) -> String {
    format!(
        "## Application Runbook\n\n- Scope: {}\n- Start command: {}\n- Readiness: {}\n- Real interface: {}\n- Runtime evidence: {}\n- Owned cleanup: {}\n- Unknowns remain unknown until the current run observes them.\n",
        safe(&runbook.scope), safe(&runbook.start_command), safe(&runbook.readiness), safe(&runbook.interface), safe(&runbook.runtime_evidence), safe(&runbook.cleanup)
    )
}

fn field(content: &str, heading: &str) -> String {
    let marker = format!("## {heading}");
    content
        .split(&marker)
        .nth(1)
        .and_then(|value| value.split("\n## ").next())
        .unwrap_or("unknown")
        .trim()
        .to_string()
}
fn safe(value: &str) -> String {
    value.lines().take(4).collect::<Vec<_>>().join(" ")
}
