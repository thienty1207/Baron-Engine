use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::safe_io::{ensure_directory_chain, read_text_required, replace_text};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentRecord {
    pub id: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

pub fn start_experiment(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    baseline: &str,
    hypothesis: &str,
    intervention: &str,
    approved: bool,
) -> Result<ExperimentRecord> {
    if !approved {
        bail!("Harness experiments require explicit human approval before intervention");
    }
    let baseline = one_line(baseline);
    let hypothesis = one_line(hypothesis);
    let intervention = one_line(intervention);
    if baseline.is_empty() || hypothesis.is_empty() || intervention.is_empty() {
        bail!("Harness experiment requires baseline, hypothesis, and intervention");
    }
    let id = format!("experiment-{}", Local::now().format("%Y%m%d%H%M%S%3f"));
    let repo_path = repo_root
        .as_ref()
        .join("docs/baron/harness/experiments")
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("ProductHarness/Experiments")
        .join(format!("{id}.md"));
    let content = format!(
        "# Baron Harness Experiment\n\n- ID: `{id}`\n- Status: `awaiting_fresh_rerun`\n- Human approval: `approved`\n- Created: {}\n\n## Baseline\n\n{baseline}\n\n## Hypothesis\n\n{hypothesis}\n\n## Intervention\n\n{intervention}\n\n## Fresh Agent Rerun\n\n- Available: `unknown`\n- Retrieved: `unknown`\n- Invoked: `unknown`\n- Relevant: `unknown`\n- Outcome: `pending`\n\n## Decision\n\n- Keep/revise/remove: `pending`\n",
        now()
    );
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
    Ok(ExperimentRecord {
        id,
        repo_path,
        vault_path,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn record_fresh_rerun(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    id: &str,
    available: bool,
    retrieved: bool,
    invoked: bool,
    relevant: bool,
    outcome: &str,
) -> Result<()> {
    let id = safe_component(id, "harness experiment ID")?;
    let outcome = one_line(outcome);
    if outcome.is_empty() {
        bail!("Fresh experiment rerun requires an observed outcome");
    }
    let repo_path = repo_root
        .as_ref()
        .join("docs/baron/harness/experiments")
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("ProductHarness/Experiments")
        .join(format!("{id}.md"));
    let mut content = read_text_required(&repo_path)
        .with_context(|| format!("Harness experiment not found: {id}"))?;
    content = replace_line(
        &content,
        "- Status: `awaiting_fresh_rerun`",
        "- Status: `rerun_recorded`",
    );
    content = replace_line(
        &content,
        "- Available: `unknown`",
        &format!("- Available: `{}`", yes_no(available)),
    );
    content = replace_line(
        &content,
        "- Retrieved: `unknown`",
        &format!("- Retrieved: `{}`", yes_no(retrieved)),
    );
    content = replace_line(
        &content,
        "- Invoked: `unknown`",
        &format!("- Invoked: `{}`", yes_no(invoked)),
    );
    content = replace_line(
        &content,
        "- Relevant: `unknown`",
        &format!("- Relevant: `{}`", yes_no(relevant)),
    );
    content = replace_line(
        &content,
        "- Outcome: `pending`",
        &format!("- Outcome: `{outcome}`"),
    );
    content.push_str(&format!("\n- Rerun recorded: {}\n", now()));
    write(&repo_path, &content)?;
    write(&vault_path, &content)
}

pub fn finalize_experiment(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    id: &str,
    decision: &str,
) -> Result<()> {
    let id = safe_component(id, "harness experiment ID")?;
    let decision = one_line(decision).to_lowercase();
    if !["keep", "revise", "remove", "pending"].contains(&decision.as_str()) {
        bail!("Experiment decision must be keep, revise, remove, or pending");
    }
    let repo_path = repo_root
        .as_ref()
        .join("docs/baron/harness/experiments")
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("ProductHarness/Experiments")
        .join(format!("{id}.md"));
    let mut content = read_text_required(&repo_path)
        .with_context(|| format!("Harness experiment not found: {id}"))?;
    if !content.contains("- Status: `rerun_recorded`") && decision != "pending" {
        bail!("Experiment cannot be finalized before a fresh rerun is recorded");
    }
    content = content.replace(
        "- Keep/revise/remove: `pending`",
        &format!("- Keep/revise/remove: `{decision}`"),
    );
    content = content.replace(
        "- Status: `rerun_recorded`",
        &format!("- Status: `completed_{decision}`"),
    );
    write(&repo_path, &content)?;
    write(&vault_path, &content)
}

fn replace_line(content: &str, old: &str, new: &str) -> String {
    content.replace(old, new)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
fn one_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}
fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory_chain(parent)?;
    }
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn safe_component<'a>(value: &'a str, label: &str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty()
        || Path::new(value).is_absolute()
        || Path::new(value)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || value.contains(['/', '\\'])
    {
        bail!("{label} must be one safe path component");
    }
    Ok(value)
}
