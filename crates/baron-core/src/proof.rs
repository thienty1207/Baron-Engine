use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};

use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofRecord {
    pub id: String,
    pub summary: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

pub fn record_proof(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<ProofRecord> {
    let repo_root = repo_root.as_ref();
    let id = Local::now().format("%Y%m%d%H%M%S%3f").to_string();
    let date = Local::now().format("%Y-%m-%d").to_string();
    let repo_path = repo_root
        .join("docs/baron/proofs")
        .join(&date)
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("Proofs")
        .join(&date)
        .join(format!("{id}.md"));
    let content = format!(
        "# Baron Proof\n\n- Proof ID: `{id}`\n- Recorded: {}\n\n## Evidence\n\n{}\n",
        now(),
        summary.trim()
    );
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
    append(
        &repo_root.join("docs/baron/proofs/INDEX.md"),
        "# Baron Proof Index\n\n",
        &format!("- `{id}` - {}", summary.trim()),
    )?;
    append(
        &vault.project_root.join("Proofs/INDEX.md"),
        "# Baron Proof Index\n\n",
        &format!("- `{id}` - {}", summary.trim()),
    )?;
    Ok(ProofRecord {
        id,
        summary: summary.trim().to_string(),
        repo_path,
        vault_path,
    })
}

pub fn proof_status(repo_root: impl AsRef<Path>) -> Result<String> {
    match latest_proof(repo_root.as_ref())? {
        Some(proof) => Ok(format!(
            "# Baron Proof Status\n\n- Latest proof: `{}`\n- Evidence: {}\n",
            proof.id, proof.summary
        )),
        None => Ok("# Baron Proof Status\n\n- Latest proof: none\n".to_string()),
    }
}

pub fn latest_proof(repo_root: &Path) -> Result<Option<ProofRecord>> {
    let root = repo_root.join("docs/baron/proofs");
    let Some(path) = latest_markdown(&root)? else {
        return Ok(None);
    };
    if path.file_name().and_then(|value| value.to_str()) == Some("INDEX.md") {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    let id = content
        .lines()
        .find_map(|line| line.strip_prefix("- Proof ID: `"))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("unknown")
        .to_string();
    let summary = content
        .split("## Evidence")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(Some(ProofRecord {
        id,
        summary,
        repo_path: path,
        vault_path: PathBuf::new(),
    }))
}

fn latest_markdown(root: &Path) -> Result<Option<PathBuf>> {
    if !root.exists() {
        return Ok(None);
    }
    let mut files = Vec::new();
    collect_markdown(root, &mut files)?;
    files.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("INDEX.md"));
    files.sort();
    Ok(files.pop())
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
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
