use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultContext {
    pub vault_root: PathBuf,
    pub repo_root: PathBuf,
    pub project_slug: String,
    pub project_root: PathBuf,
    pub baron_artifacts_root: PathBuf,
    pub index_path: PathBuf,
    pub state_path: PathBuf,
    pub approved_global_path: PathBuf,
    pub global_candidates_path: PathBuf,
}

pub fn resolve_vault_path(cli_vault: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_vault {
        return Ok(path);
    }
    if let Ok(path) = std::env::var("BARON_VAULT") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    bail!("Provide --vault <path> or set BARON_VAULT");
}

pub fn project_slug(repo_path: &Path) -> String {
    let name = repo_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("project");
    slugify(name)
}

pub fn ensure_vault(
    vault_path: impl AsRef<Path>,
    repo_path: impl AsRef<Path>,
) -> Result<VaultContext> {
    let vault_root = vault_path.as_ref().to_path_buf();
    let repo_root = repo_path.as_ref().canonicalize().with_context(|| {
        format!(
            "Could not resolve repo path: {}",
            repo_path.as_ref().display()
        )
    })?;
    let project_slug = project_slug(&repo_root);
    let project_root = vault_root.join("Projects").join(&project_slug);
    let baron_artifacts_root = vault_root.join("Artifacts").join("Baron");
    let context = VaultContext {
        vault_root: vault_root.clone(),
        repo_root,
        project_slug,
        project_root: project_root.clone(),
        index_path: baron_artifacts_root.join("memory-index.sqlite"),
        state_path: baron_artifacts_root.join("memory-engine-state.json"),
        approved_global_path: baron_artifacts_root.join("APPROVED_GLOBAL.md"),
        global_candidates_path: baron_artifacts_root.join("GLOBAL_CANDIDATES.md"),
        baron_artifacts_root: baron_artifacts_root.clone(),
    };

    fs::create_dir_all(&context.vault_root).with_context(|| {
        format!(
            "Could not create vault root: {}",
            context.vault_root.display()
        )
    })?;
    fs::create_dir_all(&context.project_root).with_context(|| {
        format!(
            "Could not create project capsule: {}",
            context.project_root.display()
        )
    })?;
    fs::create_dir_all(&context.baron_artifacts_root).with_context(|| {
        format!(
            "Could not create Baron artifacts folder: {}",
            context.baron_artifacts_root.display()
        )
    })?;

    write_if_missing(
        &context.vault_root.join("AGENTS.md"),
        "# Vault Agent Guide\n\nBaron Vault Markdown is the source of truth. SQLite files are rebuildable indexes.\n",
    )?;
    write_if_missing(
        &context.vault_root.join("Init.md"),
        "# Baron Vault\n\nUse this vault as durable memory for Baron-managed projects.\n",
    )?;
    write_if_missing(
        &context.project_root.join("README.md"),
        &format!(
            "# Project Capsule: {}\n\nThis folder stores durable memory for one project.\n",
            context.project_slug
        ),
    )?;
    write_if_missing(&context.project_root.join("Facts.md"), "# Facts\n\n")?;
    write_if_missing(
        &context.project_root.join("Decisions.md"),
        "# Decisions\n\n",
    )?;
    write_if_missing(&context.project_root.join("Tasks.md"), "# Tasks\n\n")?;

    for directory in ["Plans", "ProductHarness", "Sessions", "Artifacts"] {
        fs::create_dir_all(context.project_root.join(directory))?;
    }

    write_if_missing(
        &context.approved_global_path,
        "# Approved Global Memory\n\nOnly durable lessons that are safe across projects belong here.\n",
    )?;
    write_if_missing(
        &context.global_candidates_path,
        "# Global Memory Candidates\n\nCandidates are not loaded as facts until promoted.\n",
    )?;
    write_if_missing(
        &context.state_path,
        "{\n  \"engine\": \"baron-memory-firewall\",\n  \"schemaVersion\": 1\n}\n",
    )?;

    Ok(context)
}

pub fn vault_context_without_create(
    vault_path: impl AsRef<Path>,
    repo_path: impl AsRef<Path>,
) -> Result<VaultContext> {
    let vault_root = vault_path.as_ref().to_path_buf();
    let repo_root = repo_path.as_ref().canonicalize().with_context(|| {
        format!(
            "Could not resolve repo path: {}",
            repo_path.as_ref().display()
        )
    })?;
    let project_slug = project_slug(&repo_root);
    let project_root = vault_root.join("Projects").join(&project_slug);
    let baron_artifacts_root = vault_root.join("Artifacts").join("Baron");
    Ok(VaultContext {
        vault_root,
        repo_root,
        project_slug,
        project_root,
        index_path: baron_artifacts_root.join("memory-index.sqlite"),
        state_path: baron_artifacts_root.join("memory-engine-state.json"),
        approved_global_path: baron_artifacts_root.join("APPROVED_GLOBAL.md"),
        global_candidates_path: baron_artifacts_root.join("GLOBAL_CANDIDATES.md"),
        baron_artifacts_root,
    })
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for character in value.chars().flat_map(|character| character.to_lowercase()) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}
