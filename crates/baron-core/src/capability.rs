use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};

use crate::config::{load_project_config, AdapterKind};

const REGISTRY_PATH: &str = ".baron/capabilities.toml";
const STATE_PATH: &str = ".baron/cache/capability-state.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Cli,
    Binary,
    Mcp,
    Skill,
    Http,
    AgentAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Requirement {
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    Present,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProvider {
    pub name: String,
    pub capability: String,
    pub kind: ProviderKind,
    pub requirement: Requirement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_target: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adapters: Vec<AdapterKind>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    pub schema_version: u32,
    #[serde(default)]
    pub providers: Vec<CapabilityProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderObservation {
    pub provider: String,
    pub capability: String,
    pub kind: ProviderKind,
    pub requirement: Requirement,
    pub presence: Presence,
    pub compatible: bool,
    pub checked_at: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityState {
    pub schema_version: u32,
    pub adapter: AdapterKind,
    pub checked_at: String,
    pub observations: Vec<ProviderObservation>,
    pub required_gaps: Vec<String>,
    pub optional_gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckOptions {
    pub adapter: AdapterKind,
    pub capability: Option<String>,
    pub allow_network: bool,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self {
            schema_version: 1,
            providers: Vec::new(),
        }
    }
}

pub fn normalize_identifier(value: &str) -> Option<String> {
    let mut normalized = String::new();
    let mut pending_separator = false;
    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            if pending_separator && !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push(character.to_ascii_lowercase());
            pending_separator = false;
        } else {
            pending_separator = true;
        }
    }
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn load_registry(repo_root: impl AsRef<Path>) -> Result<CapabilityRegistry> {
    let path = repo_root.as_ref().join(REGISTRY_PATH);
    if !path.exists() {
        return Ok(CapabilityRegistry::default());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("Could not parse {}", path.display()))
}

pub fn register_provider(
    repo_root: impl AsRef<Path>,
    mut provider: CapabilityProvider,
) -> Result<CapabilityRegistry> {
    let repo_root = repo_root.as_ref();
    provider.name = normalize_identifier(&provider.name)
        .context("Provider name must contain letters or numbers")?;
    provider.capability = normalize_identifier(&provider.capability)
        .context("Capability id must contain letters or numbers")?;
    provider.description = provider.description.trim().to_string();
    validate_provider(&provider)?;

    let mut registry = load_registry(repo_root)?;
    if registry
        .providers
        .iter()
        .any(|existing| existing.name == provider.name)
    {
        bail!(
            "Capability provider name already registered: {}",
            provider.name
        );
    }
    registry.providers.push(provider);
    registry
        .providers
        .sort_by(|left, right| left.name.cmp(&right.name));
    save_registry(repo_root, &registry)?;
    Ok(registry)
}

pub fn remove_provider(
    repo_root: impl AsRef<Path>,
    capability: &str,
    provider_name: &str,
) -> Result<bool> {
    let repo_root = repo_root.as_ref();
    let capability = normalize_identifier(capability)
        .context("Capability id must contain letters or numbers")?;
    let provider_name = normalize_identifier(provider_name)
        .context("Provider name must contain letters or numbers")?;
    let mut registry = load_registry(repo_root)?;
    let before = registry.providers.len();
    registry
        .providers
        .retain(|provider| !(provider.capability == capability && provider.name == provider_name));
    let removed = before != registry.providers.len();
    if removed {
        save_registry(repo_root, &registry)?;
    }
    Ok(removed)
}

pub fn check_capabilities(
    repo_root: impl AsRef<Path>,
    options: CheckOptions,
) -> Result<CapabilityState> {
    let repo_root = repo_root.as_ref();
    let registry = load_registry(repo_root)?;
    let capability_filter = options.capability.as_deref().and_then(normalize_identifier);
    let checked_at = now();
    let mut observations = Vec::new();
    for provider in registry.providers.iter().filter(|provider| {
        capability_filter
            .as_ref()
            .map(|capability| capability == &provider.capability)
            .unwrap_or(true)
    }) {
        let compatible = provider_compatible(provider, options.adapter);
        let (presence, evidence) = if compatible {
            probe_provider(repo_root, provider, options.adapter, options.allow_network)
        } else {
            (
                Presence::Unknown,
                format!(
                    "provider is not compatible with the {} adapter",
                    adapter_name(options.adapter)
                ),
            )
        };
        observations.push(ProviderObservation {
            provider: provider.name.clone(),
            capability: provider.capability.clone(),
            kind: provider.kind,
            requirement: provider.requirement,
            presence,
            compatible,
            checked_at: checked_at.clone(),
            evidence,
        });
    }
    observations.sort_by(|left, right| {
        left.capability
            .cmp(&right.capability)
            .then_with(|| left.provider.cmp(&right.provider))
    });
    let (required_gaps, optional_gaps) = capability_gaps(&observations);
    let state = CapabilityState {
        schema_version: 1,
        adapter: options.adapter,
        checked_at,
        observations,
        required_gaps,
        optional_gaps,
    };
    save_state(repo_root, &state)?;
    Ok(state)
}

pub fn load_capability_state(repo_root: impl AsRef<Path>) -> Result<Option<CapabilityState>> {
    let path = repo_root.as_ref().join(STATE_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    let state = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse {}", path.display()))?;
    Ok(Some(state))
}

fn validate_provider(provider: &CapabilityProvider) -> Result<()> {
    if !(10..=200).contains(&provider.description.chars().count()) {
        bail!("Provider description must be between 10 and 200 characters");
    }
    match provider.kind {
        ProviderKind::Cli | ProviderKind::Binary => {
            if provider
                .command
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                bail!("CLI and binary providers require --command");
            }
        }
        ProviderKind::Mcp | ProviderKind::Skill => {
            if provider
                .scan_target
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                bail!("MCP and skill providers require --scan");
            }
            if provider.adapters.is_empty() {
                bail!("MCP and skill providers require at least one compatible adapter");
            }
        }
        ProviderKind::Http => {
            if provider
                .scan_target
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                bail!("HTTP providers require --scan");
            }
        }
        ProviderKind::AgentAdapter => {
            if provider.adapters.is_empty() {
                bail!("Agent adapter providers require at least one compatible adapter");
            }
        }
    }
    Ok(())
}

fn save_registry(repo_root: &Path, registry: &CapabilityRegistry) -> Result<()> {
    let content = toml::to_string_pretty(registry)?;
    atomic_write(&repo_root.join(REGISTRY_PATH), &content)
}

fn save_state(repo_root: &Path, state: &CapabilityState) -> Result<()> {
    let content = serde_json::to_string_pretty(state)?;
    atomic_write(&repo_root.join(STATE_PATH), &content)
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

fn provider_compatible(provider: &CapabilityProvider, adapter: AdapterKind) -> bool {
    if !provider.adapters.is_empty() {
        return provider.adapters.contains(&adapter);
    }
    matches!(
        provider.kind,
        ProviderKind::Cli | ProviderKind::Binary | ProviderKind::Http
    )
}

fn probe_provider(
    repo_root: &Path,
    provider: &CapabilityProvider,
    adapter: AdapterKind,
    allow_network: bool,
) -> (Presence, String) {
    match provider.kind {
        ProviderKind::Cli | ProviderKind::Binary => {
            probe_command(repo_root, provider.command.as_deref().unwrap_or_default())
        }
        ProviderKind::Mcp | ProviderKind::Skill => probe_path(
            repo_root,
            provider.scan_target.as_deref().unwrap_or_default(),
        ),
        ProviderKind::Http => {
            if allow_network {
                probe_http(provider.scan_target.as_deref().unwrap_or_default())
            } else {
                (
                    Presence::Unknown,
                    "network probe disabled for bounded automatic checks".to_string(),
                )
            }
        }
        ProviderKind::AgentAdapter => match load_project_config(repo_root) {
            Ok(config) if config.adapters.contains(&adapter) => (
                Presence::Present,
                format!(
                    "{} adapter is registered in .baron/project.toml",
                    adapter_name(adapter)
                ),
            ),
            Ok(_) => (
                Presence::Missing,
                format!(
                    "{} adapter is not registered in .baron/project.toml",
                    adapter_name(adapter)
                ),
            ),
            Err(_) => (
                Presence::Unknown,
                "Baron project configuration could not be read".to_string(),
            ),
        },
    }
}

fn probe_command(repo_root: &Path, command: &str) -> (Presence, String) {
    let executable = command
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches('"');
    if executable.is_empty() {
        return (Presence::Unknown, "provider command is empty".to_string());
    }
    let direct = PathBuf::from(executable);
    if direct.components().count() > 1 || direct.is_absolute() {
        let path = if direct.is_absolute() {
            direct
        } else {
            repo_root.join(direct)
        };
        return if path.is_file() {
            (
                Presence::Present,
                format!("resolved executable path {}", path.display()),
            )
        } else {
            (
                Presence::Missing,
                format!("executable path not found: {}", path.display()),
            )
        };
    }
    match resolve_on_path(executable) {
        Some(path) => (
            Presence::Present,
            format!("resolved on PATH: {}", path.display()),
        ),
        None => (
            Presence::Missing,
            format!("command not found on PATH: {executable}"),
        ),
    }
}

fn resolve_on_path(executable: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    let extensions = executable_extensions();
    for directory in env::split_paths(&path) {
        for extension in &extensions {
            let candidate = directory.join(format!("{executable}{extension}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn executable_extensions() -> Vec<String> {
    if cfg!(windows) {
        let mut extensions = vec![String::new()];
        extensions.extend(
            env::var("PATHEXT")
                .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
                .split(';')
                .filter(|value| !value.trim().is_empty())
                .map(|value| value.to_ascii_lowercase()),
        );
        extensions.extend(
            extensions
                .clone()
                .into_iter()
                .filter(|value| !value.is_empty())
                .map(|value| value.to_ascii_uppercase()),
        );
        extensions
    } else {
        vec![String::new()]
    }
}

fn probe_path(repo_root: &Path, target: &str) -> (Presence, String) {
    if target.trim().is_empty() {
        return (Presence::Unknown, "scan target is empty".to_string());
    }
    let path = expand_path(repo_root, target);
    if path.exists() {
        (
            Presence::Present,
            format!("scan target exists: {}", path.display()),
        )
    } else {
        (
            Presence::Missing,
            format!("scan target not found: {}", path.display()),
        )
    }
}

fn expand_path(repo_root: &Path, target: &str) -> PathBuf {
    if let Some(relative) = target
        .strip_prefix("~/")
        .or_else(|| target.strip_prefix("~\\"))
    {
        if let Some(home) = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME")) {
            return PathBuf::from(home).join(relative);
        }
    }
    let path = PathBuf::from(target);
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

fn probe_http(target: &str) -> (Presence, String) {
    let Some(address) = http_socket_address(target) else {
        return (
            Presence::Unknown,
            "HTTP scan target must include a host and optional port".to_string(),
        );
    };
    match TcpStream::connect_timeout(&address, Duration::from_secs(2)) {
        Ok(_) => (
            Presence::Present,
            format!("TCP endpoint reachable: {address}"),
        ),
        Err(error) => (
            Presence::Missing,
            format!("TCP endpoint unreachable: {address} ({error})"),
        ),
    }
}

fn http_socket_address(target: &str) -> Option<SocketAddr> {
    let without_scheme = target
        .strip_prefix("http://")
        .map(|value| (value, 80))
        .or_else(|| target.strip_prefix("https://").map(|value| (value, 443)))?;
    let authority = without_scheme.0.split('/').next()?.trim();
    if authority.is_empty() {
        return None;
    }
    let host_port = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{}:{}", authority, without_scheme.1)
    };
    host_port.to_socket_addrs().ok()?.next()
}

fn capability_gaps(observations: &[ProviderObservation]) -> (Vec<String>, Vec<String>) {
    let capabilities = observations
        .iter()
        .map(|observation| observation.capability.clone())
        .collect::<BTreeSet<_>>();
    let mut required = Vec::new();
    let mut optional = Vec::new();
    for capability in capabilities {
        let providers = observations
            .iter()
            .filter(|observation| observation.capability == capability)
            .collect::<Vec<_>>();
        let available = providers
            .iter()
            .any(|observation| observation.compatible && observation.presence == Presence::Present);
        if available {
            continue;
        }
        if providers
            .iter()
            .any(|observation| observation.requirement == Requirement::Required)
        {
            required.push(capability);
        } else {
            optional.push(capability);
        }
    }
    (required, optional)
}

fn adapter_name(adapter: AdapterKind) -> &'static str {
    match adapter {
        AdapterKind::Codex => "codex",
        AdapterKind::Claude => "claude",
        AdapterKind::Generic => "agent",
    }
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
