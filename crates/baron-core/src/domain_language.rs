use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::vault::VaultContext;

const DOMAIN_LANGUAGE_RELATIVE_PATH: &str = "docs/baron/harness/DOMAIN_LANGUAGE.md";
const VAULT_DOMAIN_LANGUAGE_RELATIVE_PATH: &str = "ProductHarness/DOMAIN_LANGUAGE.md";
const TEMPLATE: &str = "# Product Domain Language\n\n\
## Rules\n\n\
- Add terms only from user, repository, product, or verified runtime evidence.\n\
- Mark disputed or unclear meanings as `ambiguous`.\n\
- Do not promote a term to verified without an evidence path.\n\n\
## Terms\n\n\
| Term | Meaning | Status | Evidence |\n\
| --- | --- | --- | --- |\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainLanguageStatus {
    pub path: PathBuf,
    pub term_count: usize,
    pub ambiguous_count: usize,
    pub mirror_in_sync: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DomainTerm {
    term: String,
    meaning: String,
    status: String,
    evidence: String,
}

pub fn ensure_domain_language(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> Result<DomainLanguageStatus> {
    let repo_path = repo_root.as_ref().join(DOMAIN_LANGUAGE_RELATIVE_PATH);
    let vault_path = vault.project_root.join(VAULT_DOMAIN_LANGUAGE_RELATIVE_PATH);

    match (repo_path.exists(), vault_path.exists()) {
        (false, false) => {
            atomic_write(&repo_path, TEMPLATE)?;
            atomic_write(&vault_path, TEMPLATE)?;
        }
        (true, false) => copy_existing(&repo_path, &vault_path)?,
        (false, true) => copy_existing(&vault_path, &repo_path)?,
        (true, true) => {
            // Both copies may be user-authored. Never merge or overwrite either
            // copy without an explicit reconciliation command.
        }
    }

    let content = fs::read_to_string(&repo_path)
        .with_context(|| format!("Could not read {}", repo_path.display()))?;
    let vault_content = fs::read_to_string(&vault_path)
        .with_context(|| format!("Could not read {}", vault_path.display()))?;
    let terms = parse_terms(&content);
    Ok(DomainLanguageStatus {
        path: repo_path,
        term_count: terms.len(),
        ambiguous_count: terms
            .iter()
            .filter(|term| term.status.eq_ignore_ascii_case("ambiguous"))
            .count(),
        mirror_in_sync: content == vault_content,
    })
}

pub fn render_domain_language_context(
    repo_root: impl AsRef<Path>,
    max_chars: usize,
) -> Result<String> {
    if max_chars == 0 {
        return Ok(String::new());
    }
    let path = repo_root.as_ref().join(DOMAIN_LANGUAGE_RELATIVE_PATH);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("Could not read {}", path.display()))
        }
    };
    let terms = parse_terms(&content);
    if terms.is_empty() {
        return Ok(String::new());
    }

    let mut output = String::from("## Product Domain Language\n\n");
    for term in terms {
        let row = format!(
            "- `{}`: {} Status: `{}`. Evidence: `{}`.\n",
            term.term, term.meaning, term.status, term.evidence
        );
        if char_count(&output) + char_count(&row) > max_chars {
            break;
        }
        output.push_str(&row);
    }

    if output == "## Product Domain Language\n\n" {
        output.push_str("- Domain terms exist but exceed the compact context budget.\n");
    }
    Ok(truncate_chars(&output, max_chars))
}

fn parse_terms(content: &str) -> Vec<DomainTerm> {
    let mut found_header = false;
    let mut terms = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "| Term | Meaning | Status | Evidence |" {
            found_header = true;
            continue;
        }
        if !found_header || trimmed.is_empty() || trimmed.starts_with("| ---") {
            continue;
        }
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            if !terms.is_empty() {
                break;
            }
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() != 4 || cells.iter().any(|cell| cell.is_empty()) {
            continue;
        }
        terms.push(DomainTerm {
            term: cells[0].to_string(),
            meaning: cells[1].to_string(),
            status: cells[2].to_string(),
            evidence: cells[3].to_string(),
        });
    }
    terms
}

fn copy_existing(source: &Path, destination: &Path) -> Result<()> {
    let content = fs::read_to_string(source)
        .with_context(|| format!("Could not read {}", source.display()))?;
    atomic_write(destination, &content)
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("baron-tmp");
    fs::write(&temporary, content)
        .with_context(|| format!("Could not write {}", temporary.display()))?;
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("Could not replace {}", path.display()))?;
    }
    fs::rename(&temporary, path).with_context(|| format!("Could not write {}", path.display()))
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if char_count(value) <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }
    let mut output = value.chars().take(max_chars - 3).collect::<String>();
    output.push_str("...");
    output
}
