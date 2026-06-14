use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::vault::project_slug;

const PROJECT_CONFIG_PATH: &str = ".baron/project.toml";
const LOCAL_CONFIG_PATH: &str = ".baron/local.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterKind {
    Codex,
    Claude,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub context: bool,
    pub plan: bool,
    pub harness: bool,
    pub proof: bool,
    pub trace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub schema_version: u32,
    pub project_slug: String,
    pub adapters: Vec<AdapterKind>,
    pub automation: AutomationConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalConfig {
    pub vault_path: PathBuf,
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            context: true,
            plan: true,
            harness: true,
            proof: true,
            trace: true,
        }
    }
}

pub fn initialize_project(
    repo_path: impl AsRef<Path>,
    adapter: AdapterKind,
    vault_path: impl AsRef<Path>,
) -> Result<ProjectConfig> {
    let repo_root = canonical_directory(repo_path.as_ref())?;
    let baron_root = repo_root.join(".baron");
    fs::create_dir_all(&baron_root)
        .with_context(|| format!("Could not create {}", baron_root.display()))?;

    let project_path = repo_root.join(PROJECT_CONFIG_PATH);
    let mut config = if project_path.exists() {
        load_project_config(&repo_root)?
    } else {
        ProjectConfig {
            schema_version: 1,
            project_slug: project_slug(&repo_root),
            adapters: Vec::new(),
            automation: AutomationConfig::default(),
        }
    };
    if !config.adapters.contains(&adapter) {
        config.adapters.push(adapter);
    }
    atomic_write(&project_path, &toml::to_string_pretty(&config)?)?;

    let local = LocalConfig {
        vault_path: vault_path.as_ref().to_path_buf(),
    };
    atomic_write(
        &repo_root.join(LOCAL_CONFIG_PATH),
        &toml::to_string_pretty(&local)?,
    )?;
    write_if_missing(
        &baron_root.join(".gitignore"),
        "local.toml\ncache/\ntmp/\n",
    )?;
    Ok(config)
}

pub fn load_project_config(repo_root: impl AsRef<Path>) -> Result<ProjectConfig> {
    let path = repo_root.as_ref().join(PROJECT_CONFIG_PATH);
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Could not parse {}", path.display()))
}

pub fn load_local_config(repo_root: impl AsRef<Path>) -> Result<LocalConfig> {
    let path = repo_root.as_ref().join(LOCAL_CONFIG_PATH);
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Could not parse {}", path.display()))
}

pub fn find_project_root(start_path: impl AsRef<Path>) -> Result<PathBuf> {
    let start = start_path.as_ref();
    let canonical = start
        .canonicalize()
        .with_context(|| format!("Could not resolve path: {}", start.display()))?;
    let mut current = if canonical.is_file() {
        canonical.parent().map(Path::to_path_buf)
    } else {
        Some(canonical)
    };
    while let Some(directory) = current {
        if directory.join(PROJECT_CONFIG_PATH).is_file() {
            return Ok(directory);
        }
        current = directory.parent().map(Path::to_path_buf);
    }
    bail!(
        "Baron project config not found. Run `baron init <repo-path> --codex|--claude|--agent --vault <vault-path>` first."
    )
}

pub fn resolve_vault_path_for_repo(
    cli_vault: Option<PathBuf>,
    start_path: impl AsRef<Path>,
) -> Result<PathBuf> {
    if let Some(path) = cli_vault {
        return Ok(path);
    }
    if let Ok(path) = std::env::var("BARON_VAULT") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    let repo_root = find_project_root(start_path)?;
    let local = load_local_config(&repo_root).with_context(|| {
        format!(
            "No machine-local Vault configuration found. Provide --vault <path>, set BARON_VAULT, or restore {}.",
            repo_root.join(LOCAL_CONFIG_PATH).display()
        )
    })?;
    Ok(local.vault_path)
}

pub fn load_project_from(start_path: impl AsRef<Path>) -> Result<(PathBuf, ProjectConfig)> {
    let root = find_project_root(start_path)?;
    let config = load_project_config(&root)?;
    Ok((root, config))
}

fn canonical_directory(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Could not resolve repo path: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!("Repo path is not a directory: {}", canonical.display());
    }
    Ok(canonical)
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    atomic_write(path, content)
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("baron-tmp");
    fs::write(&temp, content).with_context(|| format!("Could not write {}", temp.display()))?;
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("Could not replace {}", path.display()))?;
    }
    fs::rename(&temp, path).with_context(|| format!("Could not write {}", path.display()))
}
