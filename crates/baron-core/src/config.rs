use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::identity::project_id_for_path;
use crate::vault::ensure_vault_root;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectPlatform {
    Frontend,
    Backend,
    Fullstack,
    Mobile,
    Desktop,
    Tool,
    Library,
    Data,
    Cloud,
    Unknown,
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
    #[serde(default)]
    pub project_id: String,
    pub project_slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<ProjectPlatform>,
    pub adapters: Vec<AdapterKind>,
    pub automation: AutomationConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalConfig {
    pub vault_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineConfig {
    pub default_vault_path: PathBuf,
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
    initialize_project_with_options(repo_path, Some(adapter), vault_path, None)
}

pub fn initialize_project_with_options(
    repo_path: impl AsRef<Path>,
    adapter: Option<AdapterKind>,
    vault_path: impl AsRef<Path>,
    platform: Option<ProjectPlatform>,
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
            schema_version: 3,
            project_id: project_id_for_path(&repo_root)?,
            project_slug: project_slug(&repo_root),
            platform: None,
            adapters: Vec::new(),
            automation: AutomationConfig::default(),
        }
    };
    if config.project_id.is_empty() {
        config.project_id = project_id_for_path(&repo_root)?;
    }
    config.schema_version = 3;
    if let Some(platform) = platform {
        config.platform = Some(platform);
    }
    if let Some(adapter) = adapter {
        if !config.adapters.contains(&adapter) {
            config.adapters.push(adapter);
        }
    }
    atomic_write(&project_path, &toml::to_string_pretty(&config)?)?;

    let local = LocalConfig {
        vault_path: vault_path.as_ref().to_path_buf(),
    };
    atomic_write(
        &repo_root.join(LOCAL_CONFIG_PATH),
        &toml::to_string_pretty(&local)?,
    )?;
    write_if_missing(&baron_root.join(".gitignore"), "local.toml\ncache/\ntmp/\n")?;
    Ok(config)
}

pub fn set_project_platform(
    repo_path: impl AsRef<Path>,
    platform: ProjectPlatform,
) -> Result<ProjectConfig> {
    let repo_root = find_project_root(repo_path)?;
    let mut config = load_project_config(&repo_root)?;
    config.schema_version = 3;
    config.platform = Some(platform);
    atomic_write(
        &repo_root.join(PROJECT_CONFIG_PATH),
        &toml::to_string_pretty(&config)?,
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
    if let Ok(repo_root) = find_project_root(start_path.as_ref()) {
        let local_path = repo_root.join(LOCAL_CONFIG_PATH);
        if local_path.is_file() {
            let local = load_local_config(&repo_root).with_context(|| {
                format!(
                    "No machine-local Vault configuration found. Provide --vault <path>, set BARON_VAULT, or restore {}.",
                    local_path.display()
                )
            })?;
            return Ok(local.vault_path);
        }
    }
    load_machine_config()
        .map(|config| config.default_vault_path)
        .with_context(|| {
            "No default Baron Vault found. Run `baron setup --vault` inside your Vault folder, pass --vault <path>, or set BARON_VAULT."
        })
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

pub fn setup_machine_vault(vault_path: impl AsRef<Path>) -> Result<PathBuf> {
    let vault_root = ensure_vault_root(vault_path.as_ref())?;
    let config_path = machine_config_path()?;
    let config = MachineConfig {
        default_vault_path: vault_root.clone(),
    };
    atomic_write(&config_path, &toml::to_string_pretty(&config)?)?;
    Ok(vault_root)
}

pub fn load_machine_config() -> Result<MachineConfig> {
    let path = machine_config_path()?;
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Could not parse {}", path.display()))
}

pub fn machine_config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("BARON_HOME") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path).join("config.toml"));
        }
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .context("Could not resolve home directory for Baron machine config")?;
    Ok(home.join(".baron").join("config.toml"))
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
