use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::identity::{new_identity_binding, project_id_for_path};
use crate::safe_io::{
    acquire_project_lock, ensure_directory_chain, read_bytes, read_text_required, replace_text,
};
use crate::vault::ensure_vault_root;
use crate::vault::project_slug;

const PROJECT_CONFIG_PATH: &str = ".baron/project.toml";
const LOCAL_CONFIG_PATH: &str = ".baron/local.toml";
pub const PROJECT_SCHEMA_VERSION: u32 = 4;

/// A supported Baron integration. Persisted compatibility values are decoded
/// at the project-config boundary into `ConfiguredAdapter` and never enter
/// this active runtime enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterKind {
    Codex,
    Claude,
}

impl AdapterKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            _ => None,
        }
    }
}

/// Persisted adapter configuration is intentionally more tolerant than the
/// active runtime. Unknown historical values are retained as opaque strings so
/// old projects can be inspected and later initialized explicitly without
/// guessing a replacement integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfiguredAdapter {
    Supported(AdapterKind),
    UnsupportedLegacy(String),
}

impl Serialize for ConfiguredAdapter {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ConfiguredAdapter {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::parse(value))
    }
}

impl ConfiguredAdapter {
    pub fn parse(value: impl Into<String>) -> Self {
        let value = value.into();
        AdapterKind::parse(&value)
            .map(Self::Supported)
            .unwrap_or(Self::UnsupportedLegacy(value))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Supported(adapter) => adapter.as_str(),
            Self::UnsupportedLegacy(value) => value,
        }
    }

    pub fn supported(&self) -> Option<AdapterKind> {
        match self {
            Self::Supported(adapter) => Some(*adapter),
            Self::UnsupportedLegacy(_) => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
// `Database` is an additive schema-4 value. Older binaries reject an unknown
// value while loading instead of silently downgrading the configured profile.
pub enum ProjectPlatform {
    Frontend,
    Backend,
    Fullstack,
    Mobile,
    Desktop,
    Tool,
    Library,
    Data,
    Database,
    Cloud,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub context: bool,
    pub plan: bool,
    pub harness: bool,
    pub proof: bool,
    pub trace: bool,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub schema_version: u32,
    pub project_id: String,
    pub identity_binding: String,
    pub project_slug: String,
    pub platform: Option<ProjectPlatform>,
    pub platform_extensions: Vec<ProjectPlatform>,
    pub adapters: Vec<AdapterKind>,
    pub active_adapter: Option<AdapterKind>,
    pub automation: AutomationConfig,
    /// Opaque values retained from older project files. These fields are not
    /// consulted by runtime operations or adapter selection.
    pub legacy_adapters: Vec<String>,
    pub legacy_active_adapter: Option<String>,
    /// Unknown project-level TOML values are retained for forward/backward
    /// compatibility and are never consulted by runtime routing.
    pub unknown_fields: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ProjectConfigWire {
    schema_version: u32,
    #[serde(default)]
    project_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    identity_binding: String,
    project_slug: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    platform: Option<ProjectPlatform>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    platform_extensions: Vec<ProjectPlatform>,
    #[serde(default)]
    adapters: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    active_adapter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    legacy_active_adapter: Option<String>,
    automation: AutomationConfig,
    #[serde(flatten)]
    unknown_fields: BTreeMap<String, toml::Value>,
}

impl Serialize for ProjectConfig {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut adapters = self
            .adapters
            .iter()
            .map(|adapter| adapter.as_str().to_string())
            .collect::<Vec<_>>();
        adapters.extend(self.legacy_adapters.iter().cloned());
        ProjectConfigWire {
            schema_version: self.schema_version,
            project_id: self.project_id.clone(),
            identity_binding: self.identity_binding.clone(),
            project_slug: self.project_slug.clone(),
            platform: self.platform,
            platform_extensions: self.platform_extensions.clone(),
            adapters,
            active_adapter: self
                .active_adapter
                .map(AdapterKind::as_str)
                .map(str::to_string)
                .or_else(|| self.legacy_active_adapter.clone()),
            legacy_active_adapter: if self.active_adapter.is_some() {
                self.legacy_active_adapter.clone()
            } else {
                None
            },
            automation: self.automation.clone(),
            unknown_fields: self.unknown_fields.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProjectConfig {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProjectConfigWire::deserialize(deserializer)?;
        let mut adapters = Vec::new();
        let mut legacy_adapters = Vec::new();
        for value in wire.adapters {
            match ConfiguredAdapter::parse(value) {
                ConfiguredAdapter::Supported(adapter) => {
                    if !adapters.contains(&adapter) {
                        adapters.push(adapter);
                    }
                }
                ConfiguredAdapter::UnsupportedLegacy(value) => legacy_adapters.push(value),
            }
        }
        let (active_adapter, legacy_active_adapter) = match wire.active_adapter {
            Some(value) => match ConfiguredAdapter::parse(value) {
                ConfiguredAdapter::Supported(adapter) => {
                    (Some(adapter), wire.legacy_active_adapter)
                }
                ConfiguredAdapter::UnsupportedLegacy(value) => {
                    (None, Some(value).or(wire.legacy_active_adapter))
                }
            },
            None => (None, wire.legacy_active_adapter),
        };
        Ok(Self {
            schema_version: wire.schema_version,
            project_id: wire.project_id,
            identity_binding: wire.identity_binding,
            project_slug: wire.project_slug,
            platform: wire.platform,
            platform_extensions: wire.platform_extensions,
            adapters,
            active_adapter,
            automation: wire.automation,
            legacy_adapters,
            legacy_active_adapter,
            unknown_fields: wire.unknown_fields,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub vault_path: PathBuf,
    #[serde(flatten)]
    pub unknown_fields: BTreeMap<String, toml::Value>,
}

// `toml::Value` deliberately does not implement `Eq` because TOML floats
// include NaN. These public config types historically implemented `Eq`, so
// retain that source contract while comparing float payloads by bits. This
// also makes equality deterministic for opaque forward-compatible fields.
fn toml_value_eq(left: &toml::Value, right: &toml::Value) -> bool {
    match (left, right) {
        (toml::Value::String(left), toml::Value::String(right)) => left == right,
        (toml::Value::Integer(left), toml::Value::Integer(right)) => left == right,
        (toml::Value::Float(left), toml::Value::Float(right)) => left.to_bits() == right.to_bits(),
        (toml::Value::Boolean(left), toml::Value::Boolean(right)) => left == right,
        (toml::Value::Datetime(left), toml::Value::Datetime(right)) => left == right,
        (toml::Value::Array(left), toml::Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| toml_value_eq(left, right))
        }
        (toml::Value::Table(left), toml::Value::Table(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, value)| {
                    right
                        .get(key)
                        .is_some_and(|other| toml_value_eq(value, other))
                })
        }
        _ => false,
    }
}

fn toml_map_eq(
    left: &BTreeMap<String, toml::Value>,
    right: &BTreeMap<String, toml::Value>,
) -> bool {
    left.len() == right.len()
        && left.iter().all(|(key, value)| {
            right
                .get(key)
                .is_some_and(|other| toml_value_eq(value, other))
        })
}

impl PartialEq for AutomationConfig {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context
            && self.plan == other.plan
            && self.harness == other.harness
            && self.proof == other.proof
            && self.trace == other.trace
            && toml_map_eq(&self.unknown_fields, &other.unknown_fields)
    }
}

impl Eq for AutomationConfig {}

impl PartialEq for ProjectConfig {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.project_id == other.project_id
            && self.identity_binding == other.identity_binding
            && self.project_slug == other.project_slug
            && self.platform == other.platform
            && self.platform_extensions == other.platform_extensions
            && self.adapters == other.adapters
            && self.active_adapter == other.active_adapter
            && self.automation == other.automation
            && self.legacy_adapters == other.legacy_adapters
            && self.legacy_active_adapter == other.legacy_active_adapter
            && toml_map_eq(&self.unknown_fields, &other.unknown_fields)
    }
}

impl Eq for ProjectConfig {}

impl PartialEq for LocalConfig {
    fn eq(&self, other: &Self) -> bool {
        self.vault_path == other.vault_path
            && toml_map_eq(&self.unknown_fields, &other.unknown_fields)
    }
}

impl Eq for LocalConfig {}

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
            unknown_fields: BTreeMap::new(),
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
    // Establish only the directory scaffold before locking. This is the
    // lock-path prerequisite required for first initialization; no managed
    // file is read or written until the project lock is held below.
    ensure_directory_chain(&baron_root).with_context(|| {
        format!(
            "Could not create safe Baron state directory: {}",
            baron_root.display()
        )
    })?;
    let _lock = acquire_project_lock(&repo_root)?;

    let project_path = repo_root.join(PROJECT_CONFIG_PATH);
    let mut config = if project_path.exists() {
        load_project_config(&repo_root)?
    } else {
        ProjectConfig {
            schema_version: PROJECT_SCHEMA_VERSION,
            project_id: project_id_for_path(&repo_root)?,
            identity_binding: new_identity_binding()?,
            project_slug: project_slug(&repo_root),
            platform: None,
            platform_extensions: Vec::new(),
            adapters: Vec::new(),
            active_adapter: None,
            automation: AutomationConfig::default(),
            legacy_adapters: Vec::new(),
            legacy_active_adapter: None,
            unknown_fields: BTreeMap::new(),
        }
    };
    if config.project_id.is_empty() {
        config.project_id = project_id_for_path(&repo_root)?;
    }
    if config.identity_binding.is_empty() {
        config.identity_binding = new_identity_binding()?;
    }
    config.schema_version = PROJECT_SCHEMA_VERSION;
    if let Some(platform) = platform {
        reconcile_platform(&mut config, platform);
    }
    if let Some(adapter) = adapter {
        if !config.adapters.contains(&adapter) {
            config.adapters.push(adapter);
        }
        config.active_adapter = Some(adapter);
    }
    atomic_write(&project_path, &toml::to_string_pretty(&config)?)?;

    let local_path = repo_root.join(LOCAL_CONFIG_PATH);
    let mut local = if read_bytes(&local_path)?.is_some() {
        load_local_config(&repo_root)?
    } else {
        LocalConfig {
            vault_path: PathBuf::new(),
            unknown_fields: BTreeMap::new(),
        }
    };
    local.vault_path = vault_path.as_ref().to_path_buf();
    atomic_write(&local_path, &toml::to_string_pretty(&local)?)?;
    write_if_missing(&baron_root.join(".gitignore"), "local.toml\ncache/\ntmp/\n")?;
    Ok(config)
}

pub fn set_project_platform(
    repo_path: impl AsRef<Path>,
    platform: ProjectPlatform,
) -> Result<ProjectConfig> {
    let repo_root = find_project_root(repo_path)?;
    let _lock = acquire_project_lock(&repo_root)?;
    let mut config = load_project_config(&repo_root)?;
    config.schema_version = PROJECT_SCHEMA_VERSION;
    reconcile_platform(&mut config, platform);
    atomic_write(
        &repo_root.join(PROJECT_CONFIG_PATH),
        &toml::to_string_pretty(&config)?,
    )?;
    Ok(config)
}

pub fn active_adapter(config: &ProjectConfig) -> Option<AdapterKind> {
    config
        .active_adapter
        .or_else(|| config.adapters.first().copied())
}

pub fn set_active_adapter(
    repo_path: impl AsRef<Path>,
    adapter: AdapterKind,
) -> Result<ProjectConfig> {
    let repo_root = find_project_root(repo_path)?;
    let _lock = acquire_project_lock(&repo_root)?;
    let mut config = load_project_config(&repo_root)?;
    if !config.adapters.contains(&adapter) {
        config.adapters.push(adapter);
    }
    config.active_adapter = Some(adapter);
    config.schema_version = PROJECT_SCHEMA_VERSION;
    atomic_write(
        &repo_root.join(PROJECT_CONFIG_PATH),
        &toml::to_string_pretty(&config)?,
    )?;
    Ok(config)
}

fn reconcile_platform(config: &mut ProjectConfig, platform: ProjectPlatform) {
    match config.platform {
        None => {
            config.platform = Some(platform);
            config
                .platform_extensions
                .retain(|value| *value != platform);
        }
        Some(ProjectPlatform::Unknown) if platform != ProjectPlatform::Unknown => {
            config.platform = Some(platform);
            config
                .platform_extensions
                .retain(|value| *value != platform);
        }
        Some(primary) if primary == platform => {}
        _ if !config.platform_extensions.contains(&platform) => {
            config.platform_extensions.push(platform);
        }
        _ => {}
    }
}

pub fn load_project_config(repo_root: impl AsRef<Path>) -> Result<ProjectConfig> {
    let path = repo_root.as_ref().join(PROJECT_CONFIG_PATH);
    let content =
        read_text_required(&path).with_context(|| format!("Could not read {}", path.display()))?;
    let config: ProjectConfig =
        toml::from_str(&content).with_context(|| format!("Could not parse {}", path.display()))?;
    if config.schema_version > PROJECT_SCHEMA_VERSION {
        bail!(
            "Unsupported Baron project schema {}; this runtime supports schema {} or older",
            config.schema_version,
            PROJECT_SCHEMA_VERSION
        );
    }
    Ok(config)
}

pub fn load_local_config(repo_root: impl AsRef<Path>) -> Result<LocalConfig> {
    let path = repo_root.as_ref().join(LOCAL_CONFIG_PATH);
    let content =
        read_text_required(&path).with_context(|| format!("Could not read {}", path.display()))?;
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
        if read_bytes(directory.join(PROJECT_CONFIG_PATH))?.is_some() {
            return Ok(directory);
        }
        current = directory.parent().map(Path::to_path_buf);
    }
    bail!(
        "Baron project config not found. Run `baron init <repo-path> --codex|--claude --vault <vault-path>` first."
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
        if read_bytes(&local_path)?.is_some() {
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

/// Returns a configured Vault path when one is already available without
/// creating or inventing project state. Receipt authority uses this optional
/// value only to enforce the machine-root boundary.
pub fn configured_vault_path_if_available(repo_root: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    if let Ok(path) = std::env::var("BARON_VAULT") {
        if !path.trim().is_empty() {
            return Ok(Some(PathBuf::from(path)));
        }
    }

    let repo_root = repo_root.as_ref();
    let local_path = repo_root.join(LOCAL_CONFIG_PATH);
    if read_bytes(&local_path)?.is_some() {
        let vault_path = load_local_config(repo_root)?.vault_path;
        return Ok(Some(if vault_path.is_absolute() {
            vault_path
        } else {
            repo_root.join(vault_path)
        }));
    }

    let machine_path = machine_config_path()?;
    if read_bytes(&machine_path)?.is_some() {
        return Ok(Some(load_machine_config()?.default_vault_path));
    }
    Ok(None)
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
        read_text_required(&path).with_context(|| format!("Could not read {}", path.display()))?;
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
    if read_bytes(path)?.is_some() {
        return Ok(());
    }
    atomic_write(path, content)
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content)
}
