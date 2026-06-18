use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde_json::json;
use sha2::{Digest, Sha256};

const SKILL_MIN_LINES: usize = 80;
const AGENT_MIN_LINES: usize = 45;

const REQUIRED_SKILL_TERMS: &[&str] = &[
    "baron contract",
    "use when",
    "output contract",
    "verification",
    "superpowers",
    "proof",
    "trace",
    "unknown",
    "evidence",
];

const REQUIRED_AGENT_TERMS: &[&str] = &[
    "core contract",
    "scope",
    "anti-hallucination",
    "output contract",
    "evidence",
    "proof",
    "trace",
    "do not invoke other subagents",
];

const MANAGED_SKILLS: &[&str] = &[
    "superpowers",
    "frontend-design",
    "vibe-security-scan",
    "api-and-interface-design",
    "observability-and-instrumentation",
    "performance-optimization",
    "deprecation-and-migration",
];

const MANAGED_AGENTS: &[&str] = &[
    "code-reviewer",
    "security-auditor",
    "test-engineer",
    "web-performance-auditor",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Skill,
    Agent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetAuditItem {
    pub name: String,
    pub kind: AssetKind,
    pub path: PathBuf,
    pub external_runtime_link: bool,
    pub thin: bool,
    pub missing_terms: Vec<String>,
    pub workflow_conflict: bool,
    pub recursive_orchestration: bool,
    pub managed: bool,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetAuditReport {
    pub passed: bool,
    pub items: Vec<AssetAuditItem>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReport {
    pub quarantined: Vec<PathBuf>,
    pub skipped_managed: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedSkillUpdate {
    pub proposal_path: PathBuf,
    pub diff_path: PathBuf,
    pub metadata_path: PathBuf,
}

pub fn audit_runtime_assets(repo_root: impl AsRef<Path>) -> Result<AssetAuditReport> {
    let repo_root = repo_root.as_ref();
    let mut items = Vec::new();
    for (root, is_managed_root) in [
        (repo_root.join(".codex/skills"), false),
        (repo_root.join(".claude/skills"), false),
        (repo_root.join(".baron/core/skills"), false),
    ] {
        collect_skill_items(&root, is_managed_root, &mut items)?;
    }
    for root in [
        repo_root.join(".codex/agents"),
        repo_root.join(".claude/agents"),
        repo_root.join(".baron/core/agents"),
    ] {
        collect_agent_items(&root, &mut items)?;
    }
    let diagnostics = items
        .iter()
        .filter(|item| !item.passed)
        .map(|item| {
            format!(
                "{} {:?} failed asset audit: external_link={}, thin={}, missing={}",
                item.name,
                item.kind,
                item.external_runtime_link,
                item.thin,
                if item.missing_terms.is_empty() {
                    "none".to_string()
                } else {
                    item.missing_terms.join(", ")
                }
            )
        })
        .collect::<Vec<_>>();
    Ok(AssetAuditReport {
        passed: diagnostics.is_empty(),
        items,
        diagnostics,
    })
}

pub fn quarantine_failing_assets(repo_root: impl AsRef<Path>) -> Result<QuarantineReport> {
    let repo_root = repo_root.as_ref();
    let report = audit_runtime_assets(repo_root)?;
    let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut quarantined = Vec::new();
    let mut skipped_managed = Vec::new();
    for item in report.items.into_iter().filter(|item| !item.passed) {
        if item.managed {
            skipped_managed.push(item.path);
            continue;
        }
        let quarantine_root = repo_root
            .join(".baron/quarantine/asset-lifecycle")
            .join(&stamp);
        let relative = item.path.strip_prefix(repo_root).unwrap_or(&item.path);
        let destination = quarantine_root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        if item.kind == AssetKind::Skill {
            let skill_root = item.path.parent().context("Skill file must have parent")?;
            let relative_skill = skill_root.strip_prefix(repo_root).unwrap_or(skill_root);
            let destination_root = quarantine_root.join(relative_skill);
            if let Some(parent) = destination_root.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(skill_root, &destination_root).with_context(|| {
                format!(
                    "Could not quarantine {} to {}",
                    skill_root.display(),
                    destination_root.display()
                )
            })?;
            quarantined.push(destination_root);
        } else {
            fs::rename(&item.path, &destination).with_context(|| {
                format!(
                    "Could not quarantine {} to {}",
                    item.path.display(),
                    destination.display()
                )
            })?;
            quarantined.push(destination);
        }
    }
    Ok(QuarantineReport {
        quarantined,
        skipped_managed,
    })
}

pub fn stage_skill_update(
    repo_root: impl AsRef<Path>,
    skill_name: &str,
    reason: &str,
    proposed_skill_body: &str,
) -> Result<StagedSkillUpdate> {
    let repo_root = repo_root.as_ref();
    let skill_name = slugify(skill_name);
    if skill_name.is_empty() {
        bail!("Skill name must contain letters or numbers");
    }
    let stamp = Local::now().to_rfc3339_opts(SecondsFormat::Secs, false);
    let folder_stamp = stamp
        .replace(':', "")
        .replace('+', "-")
        .replace('-', "")
        .replace('T', "-");
    let root = repo_root
        .join(".baron/skill-lifecycle/pending")
        .join(format!("{folder_stamp}-{skill_name}"));
    fs::create_dir_all(&root)?;

    let runtime_path = find_skill_path(repo_root, &skill_name);
    let current = runtime_path
        .as_ref()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_else(|| "<missing runtime skill>".to_string());
    let proposal_path = root.join("SKILL.md");
    let diff_path = root.join("DIFF.md");
    let metadata_path = root.join("metadata.json");
    fs::write(&proposal_path, proposed_skill_body)?;
    fs::write(
        &diff_path,
        format!(
            "# Skill Update Proposal: {skill_name}\n\n## Reason\n\n{reason}\n\n## Current Runtime Body\n\n```md\n{current}\n```\n\n## Proposed Runtime Body\n\n```md\n{proposed_skill_body}\n```\n"
        ),
    )?;
    fs::write(
        &metadata_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": 1,
                "skill": skill_name,
                "reason": reason,
                "status": "proposed",
                "approvalRequired": true,
                "createdAt": stamp,
                "runtimePath": runtime_path.map(|path| path.to_string_lossy().replace('\\', "/")),
                "proposalHash": hash(proposed_skill_body)
            }))?
        ),
    )?;
    Ok(StagedSkillUpdate {
        proposal_path,
        diff_path,
        metadata_path,
    })
}

fn collect_skill_items(
    root: &Path,
    _is_managed_root: bool,
    items: &mut Vec<AssetAuditItem>,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().join("SKILL.md");
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        items.push(audit_skill(name, path, &content));
    }
    Ok(())
}

fn collect_agent_items(root: &Path, items: &mut Vec<AssetAuditItem>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let extension = path.extension().and_then(|value| value.to_str());
        if !matches!(extension, Some("toml" | "md")) {
            continue;
        }
        if path.file_name().and_then(|value| value.to_str()) == Some("INDEX.md") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("agent")
            .to_string();
        let content = fs::read_to_string(&path)?;
        items.push(audit_agent(name, path, &content));
    }
    Ok(())
}

fn audit_skill(name: String, path: PathBuf, content: &str) -> AssetAuditItem {
    let lower = content.to_lowercase();
    let managed = MANAGED_SKILLS.contains(&name.as_str());
    let missing_terms = if name == "superpowers" {
        Vec::new()
    } else {
        missing_terms(REQUIRED_SKILL_TERMS, &lower)
    };
    let line_count = content.lines().count();
    let external_runtime_link = has_external_runtime_link(content);
    let workflow_conflict = name != "superpowers"
        && lower.contains("workflow core")
        && !lower.contains("superpowers remains the workflow core");
    let recursive_orchestration =
        lower.contains("invoke other subagents") || lower.contains("dispatch subagents");
    let thin = name != "superpowers" && (line_count < SKILL_MIN_LINES || !missing_terms.is_empty());
    let passed = !external_runtime_link && !thin && !workflow_conflict && !recursive_orchestration;
    AssetAuditItem {
        name,
        kind: AssetKind::Skill,
        path,
        external_runtime_link,
        thin,
        missing_terms,
        workflow_conflict,
        recursive_orchestration,
        managed,
        passed,
    }
}

fn audit_agent(name: String, path: PathBuf, content: &str) -> AssetAuditItem {
    let lower = content.to_lowercase();
    let managed = MANAGED_AGENTS.contains(&name.as_str());
    let missing_terms = missing_terms(REQUIRED_AGENT_TERMS, &lower);
    let line_count = content.lines().count();
    let external_runtime_link = has_external_runtime_link(content);
    let workflow_conflict =
        lower.contains("workflow core") && !lower.contains("superpowers remains the workflow core");
    let recursive_orchestration = lower.contains("invoke other subagents")
        && !lower.contains("do not invoke other subagents");
    let thin = line_count < AGENT_MIN_LINES || !missing_terms.is_empty();
    let passed = !external_runtime_link && !thin && !workflow_conflict && !recursive_orchestration;
    AssetAuditItem {
        name,
        kind: AssetKind::Agent,
        path,
        external_runtime_link,
        thin,
        missing_terms,
        workflow_conflict,
        recursive_orchestration,
        managed,
        passed,
    }
}

fn missing_terms(required: &[&str], lower: &str) -> Vec<String> {
    required
        .iter()
        .filter(|term| !lower.contains(**term))
        .map(|term| term.to_string())
        .collect()
}

fn has_external_runtime_link(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("http://") || lower.contains("https://") || lower.contains("github.com")
}

fn find_skill_path(repo_root: &Path, skill_name: &str) -> Option<PathBuf> {
    [
        repo_root
            .join(".codex/skills")
            .join(skill_name)
            .join("SKILL.md"),
        repo_root
            .join(".claude/skills")
            .join(skill_name)
            .join("SKILL.md"),
        repo_root
            .join(".baron/core/skills")
            .join(skill_name)
            .join("SKILL.md"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            output.push(character);
            dash = false;
        } else if !dash && !output.is_empty() {
            output.push('-');
            dash = true;
        }
    }
    while output.ends_with('-') {
        output.pop();
    }
    output
}

fn hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}
