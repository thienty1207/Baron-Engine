use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};

use crate::risk::{classify_risk, RiskLane};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessStory {
    pub title: String,
    pub risk: RiskLane,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
}

pub fn start_or_resume_intake(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
) -> Result<HarnessStory> {
    let repo_root = repo_root.as_ref();
    let title = title.trim();
    let risk = classify_risk(title);
    let date = today();
    let slug = slugify(title);
    let repo_path = repo_root
        .join("docs/baron/harness/stories")
        .join(&date)
        .join(format!("{date}-{slug}.md"));
    let vault_path = vault
        .project_root
        .join("ProductHarness/Stories")
        .join(&date)
        .join(format!("{date}-{slug}.md"));
    let resumed = repo_path.exists();
    if !resumed {
        let content = story_content(title, risk);
        write(&repo_path, &content)?;
        write(&vault_path, &content)?;
        append_unique(
            &repo_root.join("docs/baron/harness/STORIES.md"),
            "# Product Harness Stories\n\n",
            &format!(
                "- [{}]({}) - risk: `{}`",
                title,
                normalize(&repo_path, repo_root),
                risk.as_str()
            ),
        )?;
        append_unique(
            &vault.project_root.join("ProductHarness/STORIES.md"),
            "# Product Harness Stories\n\n",
            &format!(
                "- [{}]({}) - risk: `{}`",
                title,
                normalize(&vault_path, &vault.project_root),
                risk.as_str()
            ),
        )?;
    }
    let current = format!(
        "# Current Product Harness\n\n- Title: {title}\n- Risk: `{}`\n- Story: `{}`\n- Updated: {}\n",
        risk.as_str(),
        normalize(&repo_path, repo_root),
        now()
    );
    write(&repo_root.join("docs/baron/harness/CURRENT.md"), &current)?;
    write(
        &vault.project_root.join("ProductHarness/CURRENT.md"),
        &current,
    )?;
    Ok(HarnessStory {
        title: title.to_string(),
        risk,
        repo_path,
        vault_path,
        resumed,
    })
}

pub fn record_decision(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<()> {
    append(
        &repo_root.as_ref().join("docs/baron/harness/DECISIONS.md"),
        "# Product Decisions\n\n",
        &format!("- {} - {}", now(), summary.trim()),
    )?;
    append(
        &vault.project_root.join("ProductHarness/DECISIONS.md"),
        "# Product Decisions\n\n",
        &format!("- {} - {}", now(), summary.trim()),
    )
}

pub fn record_friction(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<()> {
    let item = format!("- [ ] {} - {}", now(), summary.trim());
    append(
        &repo_root.as_ref().join("docs/baron/harness/FRICTION.md"),
        "# Harness Friction\n\n",
        &item,
    )?;
    append(
        &vault.project_root.join("ProductHarness/FRICTION.md"),
        "# Harness Friction\n\n",
        &item,
    )
}

pub fn harness_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let root = repo_root.as_ref().join("docs/baron/harness");
    let current_path = root.join("CURRENT.md");
    let current = if current_path.exists() {
        fs::read_to_string(&current_path)?
    } else {
        "# Current Product Harness\n\n- none\n".to_string()
    };
    let friction = fs::read_to_string(root.join("FRICTION.md")).unwrap_or_default();
    let open = friction
        .lines()
        .filter(|line| line.starts_with("- [ ]"))
        .count();
    Ok(format!(
        "# Baron Harness Status\n\n{}\n- Open friction: {}\n",
        current.trim(),
        open
    ))
}

pub fn current_harness_risk(repo_root: impl AsRef<Path>) -> RiskLane {
    let content = fs::read_to_string(repo_root.as_ref().join("docs/baron/harness/CURRENT.md"))
        .unwrap_or_default();
    if content.contains("Risk: `high`") {
        RiskLane::High
    } else if content.contains("Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

pub fn current_harness_title(repo_root: impl AsRef<Path>) -> Option<String> {
    let content =
        fs::read_to_string(repo_root.as_ref().join("docs/baron/harness/CURRENT.md")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("- Title: "))
        .map(str::to_string)
}

fn story_content(title: &str, risk: RiskLane) -> String {
    let proof = match risk {
        RiskLane::Low => "concrete verification result",
        RiskLane::Medium => "focused test/build/smoke proof",
        RiskLane::High => "focused verification plus security/data-impact proof",
    };
    format!(
        "# Product Story - {title}\n\n\
- Status: `in_progress`\n\
- Risk: `{}`\n\
- Created: {}\n\n\
## Goal\n\n{title}\n\n\
## Scope\n\n- Work tied to this story only.\n\n\
## Out Of Scope\n\n- Unrelated cleanup or speculative features.\n\n\
## Required Proof\n\n- [ ] {proof}\n\
- [ ] Trace tier `{}` or stronger\n\n\
## Progress\n\n- {} - Intake created.\n",
        risk.as_str(),
        now(),
        risk.required_trace_tier(),
        now()
    )
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

fn append_unique(path: &Path, header: &str, item: &str) -> Result<()> {
    let content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if content.contains(item) {
        return Ok(());
    }
    append(path, header, item)
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
        "story".to_string()
    } else {
        slug
    }
}
