use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::to_string_pretty;

use crate::config::load_project_config;
use crate::identity::{capsule_key, project_id_for_path, CapsuleMetadata, ProjectIdentity};
use crate::safe_io::{ensure_directory_chain, read_bytes, replace_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultContext {
    pub vault_root: PathBuf,
    pub repo_root: PathBuf,
    pub project_id: String,
    pub identity_binding: String,
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

pub fn ensure_vault_root(vault_path: impl AsRef<Path>) -> Result<PathBuf> {
    let vault_root = vault_path.as_ref().to_path_buf();
    ensure_directory_chain(&vault_root)
        .with_context(|| format!("Could not create vault root: {}", vault_root.display()))?;
    let vault_root = vault_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve vault root: {}",
            vault_path.as_ref().display()
        )
    })?;
    let baron_artifacts_root = vault_root.join("Artifacts").join("Baron");
    ensure_directory_chain(&baron_artifacts_root).with_context(|| {
        format!(
            "Could not create Baron artifacts folder: {}",
            baron_artifacts_root.display()
        )
    })?;
    write_if_missing(
        &vault_root.join("AGENTS.md"),
        "# Vault Agent Guide\n\nBaron Vault Markdown is the source of truth. SQLite files are rebuildable indexes.\n",
    )?;
    write_if_missing(
        &vault_root.join("Init.md"),
        "# Baron Vault\n\nUse this vault as durable memory for Baron-managed projects.\n",
    )?;
    write_if_missing(
        &baron_artifacts_root.join("APPROVED_GLOBAL.md"),
        "# Approved Global Memory\n\nOnly durable lessons that are safe across projects belong here.\n",
    )?;
    write_if_missing(
        &baron_artifacts_root.join("GLOBAL_CANDIDATES.md"),
        "# Global Memory Candidates\n\nCandidates are not loaded as facts until promoted.\n",
    )?;
    write_if_missing(
        &baron_artifacts_root.join("memory-engine-state.json"),
        "{\n  \"engine\": \"baron-memory-firewall\",\n  \"schemaVersion\": 1\n}\n",
    )?;
    Ok(vault_root)
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
    let identity = resolve_project_identity(&repo_root)?;
    let project_slug = identity.project_slug.clone();
    let projects_root = vault_root.join("Projects");
    ensure_directory_chain(&projects_root)?;
    let project_root = projects_root.join(&identity.capsule_key);
    migrate_legacy_capsule(&projects_root, &project_slug, &project_root)?;
    let baron_artifacts_root = vault_root.join("Artifacts").join("Baron");
    let context = VaultContext {
        vault_root: vault_root.clone(),
        repo_root,
        project_id: identity.project_id.clone(),
        identity_binding: identity.identity_binding.clone(),
        project_slug,
        project_root: project_root.clone(),
        index_path: baron_artifacts_root.join("memory-index.sqlite"),
        state_path: baron_artifacts_root.join("memory-engine-state.json"),
        approved_global_path: baron_artifacts_root.join("APPROVED_GLOBAL.md"),
        global_candidates_path: baron_artifacts_root.join("GLOBAL_CANDIDATES.md"),
        baron_artifacts_root: baron_artifacts_root.clone(),
    };

    ensure_vault_root(&context.vault_root)?;
    ensure_directory_chain(&context.project_root).with_context(|| {
        format!(
            "Could not create project capsule: {}",
            context.project_root.display()
        )
    })?;
    write_capsule_metadata(&context)?;
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

    for directory in [
        "Plans",
        "ProductHarness",
        "Proofs",
        "Traces",
        "Sessions",
        "Artifacts",
    ] {
        ensure_directory_chain(context.project_root.join(directory))?;
    }

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
    let identity = resolve_project_identity(&repo_root)?;
    let project_slug = identity.project_slug.clone();
    let project_root = vault_root
        .join("Projects")
        .join(capsule_key(&project_slug, &identity.project_id));
    let baron_artifacts_root = vault_root.join("Artifacts").join("Baron");
    Ok(VaultContext {
        vault_root,
        repo_root,
        project_id: identity.project_id,
        identity_binding: identity.identity_binding,
        project_slug,
        project_root,
        index_path: baron_artifacts_root.join("memory-index.sqlite"),
        state_path: baron_artifacts_root.join("memory-engine-state.json"),
        approved_global_path: baron_artifacts_root.join("APPROVED_GLOBAL.md"),
        global_candidates_path: baron_artifacts_root.join("GLOBAL_CANDIDATES.md"),
        baron_artifacts_root,
    })
}

pub fn load_capsule_metadata(project_root: &Path) -> Result<Option<CapsuleMetadata>> {
    let path = project_root.join(".baron-project.json");
    if !path.exists() {
        return Ok(None);
    }
    let content = crate::safe_io::read_text_required(&path)
        .with_context(|| format!("Could not read {}", path.display()))?;
    let metadata = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse {}", path.display()))?;
    Ok(Some(metadata))
}

fn resolve_project_identity(repo_root: &Path) -> Result<ProjectIdentity> {
    let project_slug = project_slug(repo_root);
    let config_path = repo_root.join(".baron/project.toml");
    let (project_id, identity_binding) = if read_bytes(&config_path)?.is_some() {
        let config = load_project_config(repo_root)?;
        let project_id = if config.project_id.is_empty() {
            project_id_for_path(repo_root)?
        } else {
            config.project_id
        };
        (project_id, config.identity_binding)
    } else {
        (project_id_for_path(repo_root)?, String::new())
    };
    Ok(crate::identity::identity(
        project_slug,
        project_id,
        identity_binding,
    ))
}

fn migrate_legacy_capsule(
    projects_root: &Path,
    project_slug: &str,
    project_root: &Path,
) -> Result<()> {
    let legacy_root = projects_root.join(project_slug);
    if !legacy_root.exists() || project_root.exists() {
        return Ok(());
    }
    ensure_directory_chain(projects_root)?;
    fs::rename(&legacy_root, project_root).with_context(|| {
        format!(
            "Could not migrate legacy capsule {} to {}",
            legacy_root.display(),
            project_root.display()
        )
    })
}

fn write_capsule_metadata(context: &VaultContext) -> Result<()> {
    let metadata = CapsuleMetadata {
        schema_version: 2,
        project_id: context.project_id.clone(),
        project_slug: context.project_slug.clone(),
        identity_binding: context.identity_binding.clone(),
    };
    let content = format!("{}\n", to_string_pretty(&metadata)?);
    replace_text(context.project_root.join(".baron-project.json"), &content)?;
    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if read_bytes(path)?.is_some() {
        return Ok(());
    }
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
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
