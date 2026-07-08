use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewFindingInput {
    pub severity: String,
    pub summary: String,
    pub evidence: Vec<String>,
    pub affected_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFinding {
    pub id: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

pub fn record_finding(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    mut input: ReviewFindingInput,
) -> Result<ReviewFinding> {
    normalize(&mut input);
    if input.severity.is_empty() || input.summary.is_empty() || input.evidence.is_empty() {
        bail!("Review finding requires severity, summary, and concrete evidence.");
    }
    let id = finding_id(&input)?;
    let repo_path = repo_root
        .as_ref()
        .join("docs/baron/reviews/findings")
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("Reviews/Findings")
        .join(format!("{id}.md"));
    let content = format!(
        "# Baron Review Finding\n\n- ID: `{id}`\n- Status: `open`\n- Severity: `{}`\n- Recorded: {}\n\n## Summary\n\n{}\n\n## Evidence\n\n{}\n\n## Affected Files\n\n{}\n\n## Closure Rule\n\nA finding closes only after both fix evidence and verification are recorded.\n",
        input.severity,
        now(),
        input.summary,
        list(&input.evidence),
        list(&input.affected_files)
    );
    write_if_missing(&repo_path, &content)?;
    write_if_missing(&vault_path, &content)?;
    append_index(
        &repo_root.as_ref().join("docs/baron/reviews/INDEX.md"),
        &id,
        &input.summary,
    )?;
    append_index(
        &vault.project_root.join("Reviews/INDEX.md"),
        &id,
        &input.summary,
    )?;
    Ok(ReviewFinding {
        id,
        repo_path,
        vault_path,
    })
}

pub fn close_finding(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    id: &str,
    fix_evidence: &str,
    verification: &str,
) -> Result<()> {
    let fix_evidence = one_line(fix_evidence);
    let verification = one_line(verification);
    if fix_evidence.is_empty() {
        bail!("Review finding closure requires fix evidence.");
    }
    if verification.is_empty() {
        bail!("Review finding closure requires verification evidence.");
    }
    let repo_path = repo_root
        .as_ref()
        .join("docs/baron/reviews/findings")
        .join(format!("{}.md", id.trim()));
    let vault_path = vault
        .project_root
        .join("Reviews/Findings")
        .join(format!("{}.md", id.trim()));
    let mut content = fs::read_to_string(&repo_path)
        .with_context(|| format!("Review finding not found: {}", id.trim()))?;
    if content.contains("- Status: `closed`") {
        return Ok(());
    }
    content = content.replacen("- Status: `open`", "- Status: `closed`", 1);
    content.push_str(&format!(
        "\n## Closure Evidence\n\n- Closed: {}\n- Fix evidence: {}\n- Verification: {}\n",
        now(),
        fix_evidence,
        verification
    ));
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
    Ok(())
}

pub fn review_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let root = repo_root.as_ref().join("docs/baron/reviews/findings");
    let mut total = 0;
    let mut open = 0;
    if root.is_dir() {
        for entry in fs::read_dir(&root)?.filter_map(Result::ok) {
            if !entry.path().is_file() {
                continue;
            }
            total += 1;
            if fs::read_to_string(entry.path())
                .unwrap_or_default()
                .contains("- Status: `open`")
            {
                open += 1;
            }
        }
    }
    Ok(format!(
        "# Baron Review Gate Status\n\n- Total findings: {total}\n- Open findings: {open}\n- Closure rule: fix evidence plus verification required\n"
    ))
}

fn normalize(input: &mut ReviewFindingInput) {
    input.severity = one_line(&input.severity).to_lowercase();
    input.summary = one_line(&input.summary);
    input.evidence = input
        .evidence
        .iter()
        .map(|v| one_line(v))
        .filter(|v| !v.is_empty())
        .collect();
    input.affected_files = input
        .affected_files
        .iter()
        .map(|v| one_line(v))
        .filter(|v| !v.is_empty())
        .collect();
}
fn finding_id(input: &ReviewFindingInput) -> Result<String> {
    let digest = Sha256::digest(serde_json::to_vec(input)?);
    Ok(format!(
        "finding-{}",
        digest
            .iter()
            .take(8)
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}
fn list(values: &[String]) -> String {
    if values.is_empty() {
        "- none recorded".to_string()
    } else {
        values
            .iter()
            .map(|v| format!("- {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
fn one_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}
fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        write(path, content)
    }
}
fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}
fn append_index(path: &Path, id: &str, summary: &str) -> Result<()> {
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Review Findings\n\n".to_string());
    if !content.contains(&format!("`{id}`")) {
        content.push_str(&format!("- `{id}` - {summary}\n"));
        write(path, &content)?;
    }
    Ok(())
}
