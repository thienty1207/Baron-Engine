use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::proof::{latest_proof, proof_satisfies_risk};
use crate::risk::{classify_risk, RiskLane};
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

pub fn start_or_resume_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
) -> Result<PlanRecord> {
    let repo_root = repo_root.as_ref();
    let title = title.trim();
    if let Some(active) = active_plan(repo_root)? {
        if active.title.eq_ignore_ascii_case(title) && active.status != "completed" {
            set_plan_status(&active.path, "in_progress")?;
            append_progress(&active.path, "Plan resumed.")?;
            mirror_plan(repo_root, vault, &active.path)?;
            write_current(
                repo_root,
                vault,
                title,
                active.risk,
                "in_progress",
                &active.path,
                "continue from last known state",
                "not_run",
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
    let content = plan_content(title, risk);
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
        title,
        risk,
        "in_progress",
        &repo_path,
        "continue from current task scope",
        "not_run",
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
        &active.title,
        active.risk,
        &active.status,
        &active.path,
        note.trim(),
        "not_run",
    )
}

pub fn interrupt_plan(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    state: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let active = require_active_plan(repo_root)?;
    set_plan_status(&active.path, "interrupted")?;
    append_progress(&active.path, &format!("Interrupted: {}", state.trim()))?;
    mirror_plan(repo_root, vault, &active.path)?;
    write_current(
        repo_root,
        vault,
        &active.title,
        active.risk,
        "interrupted",
        &active.path,
        state.trim(),
        "not_run",
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
    set_plan_status(&active.path, "completed")?;
    append_progress(
        &active.path,
        &format!(
            "Completed with verification: {}",
            verification_summary.trim()
        ),
    )?;
    mirror_plan(repo_root, vault, &active.path)?;
    write_current(
        repo_root,
        vault,
        &active.title,
        active.risk,
        "completed",
        &active.path,
        "start the next explicit task",
        verification_summary.trim(),
    )
}

pub fn plan_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let path = repo_root.as_ref().join("docs/baron/plans/CURRENT.md");
    if !path.exists() {
        return Ok("# Baron Plan Status\n\n- Active plan: none\n".to_string());
    }
    Ok(format!(
        "# Baron Plan Status\n\n{}",
        fs::read_to_string(path)?
    ))
}

fn write_current(
    repo_root: &Path,
    vault: &VaultContext,
    title: &str,
    risk: RiskLane,
    status: &str,
    plan_path: &Path,
    next_action: &str,
    verification: &str,
) -> Result<()> {
    let content = format!(
        "# Current Baron Plan\n\n\
- Title: {title}\n\
- Plan: `{}`\n\
- Status: `{status}`\n\
- Risk: `{}`\n\
- Verification: {verification}\n\
- Next action: {next_action}\n\
- Updated: {}\n\n\
## Rules\n\n\
- Silence or shutdown never means completed.\n\
- Completion requires risk-appropriate proof and a passing trace score.\n",
        normalize(plan_path, repo_root),
        risk.as_str(),
        now()
    );
    write(&repo_root.join("docs/baron/plans/CURRENT.md"), &content)?;
    write(&vault.project_root.join("Plans/CURRENT.md"), &content)
}

fn active_plan(repo_root: &Path) -> Result<Option<ActivePlan>> {
    let current_path = repo_root.join("docs/baron/plans/CURRENT.md");
    if !current_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&current_path)?;
    let title = field(&content, "- Title: ").unwrap_or_default();
    let path = field(&content, "- Plan: `")
        .and_then(|value| value.strip_suffix('`').map(str::to_string))
        .map(|value| repo_root.join(value));
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
    Ok(path.map(|path| ActivePlan {
        title,
        path,
        status,
        risk,
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

fn set_plan_status(path: &Path, status: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let updated = content
        .lines()
        .map(|line| {
            if line.starts_with("status: ") {
                format!("status: {status}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    write(path, &(updated + "\n"))
}

fn append_progress(path: &Path, note: &str) -> Result<()> {
    let mut content = fs::read_to_string(path)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("- {} - {}\n", now(), note));
    write(path, &content)
}

fn mirror_plan(repo_root: &Path, vault: &VaultContext, plan_path: &Path) -> Result<()> {
    let content = fs::read_to_string(plan_path)?;
    write(&vault_plan_path(repo_root, vault, plan_path), &content)
}

fn vault_plan_path(repo_root: &Path, vault: &VaultContext, repo_path: &Path) -> PathBuf {
    let relative = repo_path
        .strip_prefix(repo_root.join("docs/baron/plans"))
        .unwrap_or(repo_path);
    vault.project_root.join("Plans").join(relative)
}

fn plan_content(title: &str, risk: RiskLane) -> String {
    format!(
        "---\n\
type: baron-plan\n\
title: {title}\n\
status: in_progress\n\
risk: {}\n\
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
}
