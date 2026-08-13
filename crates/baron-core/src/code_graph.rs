use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::capability::{
    load_registry, register_provider, CapabilityProvider, CapabilityRegistry, ProviderKind,
    Requirement,
};
use crate::config::{load_local_config, load_project_config};
use crate::identity::project_id_for_path;

pub const CODE_GRAPH_CACHE_DIR: &str = ".baron/cache/code-graph";
pub const CODE_GRAPH_STATE_FILE: &str = "state.json";
pub const CODE_GRAPH_QUERY_CACHE_FILE: &str = "last-query.json";
pub const MAX_GRAPH_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_QUERY_HITS: usize = 8;
pub const MAX_QUERY_CHARS: usize = 2_400;
const MAX_SOURCE_VERIFICATION_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphConfidence {
    Extracted,
    Inferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphFreshness {
    Fresh,
    Stale,
    Missing,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGraphHit {
    pub node_id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    pub confidence: GraphConfidence,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGraphState {
    pub schema_version: u32,
    pub provider: String,
    pub provider_version: String,
    pub project_id: String,
    pub repo_root: String,
    pub source_fingerprint: String,
    pub graph_sha256: String,
    pub graph_size_bytes: u64,
    pub built_at: String,
    pub freshness: GraphFreshness,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGraphQueryCache {
    pub schema_version: u32,
    pub project_id: String,
    pub repo_root: String,
    pub source_fingerprint: String,
    pub graph_sha256: String,
    pub question: String,
    pub queried_at: String,
    pub hits: Vec<CodeGraphHit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceVerificationStatus {
    Verified,
    Advisory,
    MissingSource,
    MissingEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceVerification {
    pub status: SourceVerificationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderProbe {
    pub provider: String,
    pub present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryLimits {
    pub max_hits: usize,
    pub max_chars: usize,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_hits: MAX_QUERY_HITS,
            max_chars: MAX_QUERY_CHARS,
        }
    }
}

impl QueryLimits {
    pub fn bounded(self) -> Self {
        Self {
            max_hits: self.max_hits.min(MAX_QUERY_HITS),
            max_chars: self.max_chars.min(MAX_QUERY_CHARS),
        }
    }
}

pub trait CodeGraphProvider {
    fn probe(&self, repo_root: &Path) -> Result<ProviderProbe>;
    fn refresh(&self, repo_root: &Path, cache_root: &Path) -> Result<CodeGraphState>;
    fn query(
        &self,
        repo_root: &Path,
        cache_root: &Path,
        question: &str,
        limits: QueryLimits,
    ) -> Result<Vec<CodeGraphHit>>;
}

pub fn ensure_code_map_capability(repo_root: impl AsRef<Path>) -> Result<CapabilityRegistry> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let registry = load_registry(&repo_root)?;
    if registry
        .providers
        .iter()
        .any(|provider| provider.capability == "code-map")
    {
        return Ok(registry);
    }
    register_provider(
        &repo_root,
        CapabilityProvider {
            name: "graphify-local".to_string(),
            capability: "code-map".to_string(),
            kind: ProviderKind::Cli,
            requirement: Requirement::Optional,
            command: Some("graphify".to_string()),
            scan_target: None,
            adapters: Vec::new(),
            description: "Optional local project-scoped code map".to_string(),
        },
    )
}

pub fn code_graph_cache_root(repo_root: impl AsRef<Path>) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let cache_root = repo_root.join(CODE_GRAPH_CACHE_DIR);
    validate_code_graph_cache_path(&repo_root, &cache_root)
}

/// Creates Baron-owned cache directories one component at a time after checking
/// every existing component. This avoids following a project-controlled link or
/// junction while keeping the cache strictly project-local.
pub fn ensure_code_graph_cache_root(repo_root: impl AsRef<Path>) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let cache_root = code_graph_cache_root(&repo_root)?;
    let relative = cache_root
        .strip_prefix(&repo_root)
        .context("Code graph cache root is outside the current repository")?;
    let mut current = repo_root.clone();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            continue;
        };
        current.push(part);
        if current.exists() {
            validate_existing_ancestors_within_repo(&repo_root, &current)?;
            if !current.is_dir() {
                bail!(
                    "Code graph cache component is not a directory: {}",
                    current.display()
                );
            }
            continue;
        }
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not create Baron-owned cache directory: {}",
                        current.display()
                    )
                })
            }
        }
        validate_existing_ancestors_within_repo(&repo_root, &current)?;
        if !current.is_dir() {
            bail!(
                "Code graph cache component is not a directory: {}",
                current.display()
            );
        }
    }
    validate_code_graph_cache_path(&repo_root, &cache_root)
}

pub fn code_graph_state_path(repo_root: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(code_graph_cache_root(repo_root)?.join(CODE_GRAPH_STATE_FILE))
}

pub fn code_graph_query_cache_path(repo_root: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(code_graph_cache_root(repo_root)?.join(CODE_GRAPH_QUERY_CACHE_FILE))
}

pub fn graphify_graph_path(
    repo_root: impl AsRef<Path>,
    source_fingerprint: &str,
) -> Result<PathBuf> {
    validate_fingerprint(source_fingerprint)?;
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let cache_root = code_graph_cache_root(&repo_root)?;
    let graph_path = cache_root
        .join("graphify")
        .join(source_fingerprint)
        .join("graphify-out")
        .join("graph.json");
    validate_code_graph_cache_path(&repo_root, &graph_path)
}

pub fn validate_code_graph_cache_path(
    repo_root: impl AsRef<Path>,
    candidate: impl AsRef<Path>,
) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let cache_root = repo_root.join(CODE_GRAPH_CACHE_DIR);
    let candidate = candidate.as_ref();
    if !candidate.is_absolute() {
        bail!("Code graph cache path must be absolute");
    }
    if !candidate.starts_with(&cache_root) {
        bail!(
            "Code graph cache path escapes the current repository: {}",
            candidate.display()
        );
    }
    let relative = candidate
        .strip_prefix(&cache_root)
        .context("Code graph cache path is outside the managed cache root")?;
    if !relative.as_os_str().is_empty() {
        validate_safe_relative_path(relative, "Code graph cache path")?;
    }
    validate_existing_ancestors_within_repo(&repo_root, candidate)?;
    Ok(candidate.to_path_buf())
}

pub fn compute_code_source_fingerprint(repo_root: impl AsRef<Path>) -> Result<String> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let nested_vault = load_local_config(&repo_root)
        .ok()
        .and_then(|config| config.vault_path.canonicalize().ok())
        .filter(|vault| vault.starts_with(&repo_root));
    let vault_for_filter = nested_vault.clone();
    let mut builder = WalkBuilder::new(&repo_root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true);

    let mut rows = Vec::new();
    for entry in builder
        .filter_entry(move |entry| {
            !is_skipped_source_path(entry.path())
                && !vault_for_filter
                    .as_ref()
                    .is_some_and(|vault| entry.path().starts_with(vault))
        })
        .build()
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if path == repo_root || !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        let relative = match path.strip_prefix(&repo_root) {
            Ok(relative) => relative,
            Err(_) => continue,
        };
        if validate_safe_relative_path(relative, "Source path").is_err() {
            continue;
        }
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let modified = metadata
            .modified()
            .ok()
            .and_then(system_time_parts)
            .unwrap_or((0, 0));
        rows.push((
            normalize_relative_path(relative),
            metadata.len(),
            modified.0,
            modified.1,
        ));
    }
    rows.sort();

    let mut digest = Sha256::new();
    digest.update(b"baron-code-graph-source-fingerprint-v1\0");
    for (path, size, seconds, nanos) in rows {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(size.to_le_bytes());
        digest.update(seconds.to_le_bytes());
        digest.update(nanos.to_le_bytes());
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub fn write_code_graph_state(
    repo_root: impl AsRef<Path>,
    provider: &str,
    provider_version: &str,
    source_fingerprint: &str,
    graph_path: impl AsRef<Path>,
) -> Result<CodeGraphState> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    validate_fingerprint(source_fingerprint)?;
    let graph_path = validate_code_graph_cache_path(&repo_root, graph_path)?;
    let metadata = fs::metadata(&graph_path)
        .with_context(|| format!("Could not read code graph: {}", graph_path.display()))?;
    if !metadata.is_file() {
        bail!("Code graph must be a file: {}", graph_path.display());
    }
    if metadata.len() > MAX_GRAPH_BYTES {
        bail!("Code graph exceeds the {} byte limit", MAX_GRAPH_BYTES);
    }
    let state = CodeGraphState {
        schema_version: 1,
        provider: required_label(provider, "Code graph provider")?,
        provider_version: required_label(provider_version, "Code graph provider version")?,
        project_id: current_project_id(&repo_root)?,
        repo_root: normalize_absolute_path(&repo_root),
        source_fingerprint: source_fingerprint.to_string(),
        graph_sha256: sha256_file(&graph_path)?,
        graph_size_bytes: metadata.len(),
        built_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        freshness: GraphFreshness::Fresh,
        diagnostics: Vec::new(),
    };
    validate_code_graph_state(&repo_root, &state)?;
    let state_path = ensure_code_graph_cache_root(&repo_root)?.join(CODE_GRAPH_STATE_FILE);
    atomic_write_json(&state_path, &state)?;
    Ok(state)
}

/// Confirms that the state still points to the exact project-local graph that
/// was recorded at refresh time. The graph is cache data, never Vault memory.
pub fn validate_code_graph_artifact(
    repo_root: impl AsRef<Path>,
    state: &CodeGraphState,
) -> Result<PathBuf> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    validate_code_graph_state(&repo_root, state)?;
    let graph_path = graphify_graph_path(&repo_root, &state.source_fingerprint)?;
    let metadata = fs::metadata(&graph_path)
        .with_context(|| format!("Could not read code graph: {}", graph_path.display()))?;
    if !metadata.is_file() {
        bail!("Code graph must be a file: {}", graph_path.display());
    }
    if metadata.len() > MAX_GRAPH_BYTES || metadata.len() != state.graph_size_bytes {
        bail!("Code graph artifact does not match its recorded size");
    }
    if sha256_file(&graph_path)? != state.graph_sha256 {
        bail!("Code graph artifact does not match its recorded checksum");
    }
    Ok(graph_path)
}

pub fn load_code_graph_state(repo_root: impl AsRef<Path>) -> Result<Option<CodeGraphState>> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let state_path = code_graph_state_path(&repo_root)?;
    if !state_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&state_path)
        .with_context(|| format!("Could not read {}", state_path.display()))?;
    let state: CodeGraphState = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse {}", state_path.display()))?;
    validate_code_graph_state(&repo_root, &state)?;
    Ok(Some(state))
}

pub fn write_code_graph_query_cache(
    repo_root: impl AsRef<Path>,
    state: &CodeGraphState,
    question: &str,
    hits: Vec<CodeGraphHit>,
) -> Result<CodeGraphQueryCache> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    validate_code_graph_state(&repo_root, state)?;
    let question = normalize_query(question)?;
    let hits = normalize_code_graph_hits(&repo_root, hits, QueryLimits::default())?;
    let cache = CodeGraphQueryCache {
        schema_version: 1,
        project_id: current_project_id(&repo_root)?,
        repo_root: normalize_absolute_path(&repo_root),
        source_fingerprint: state.source_fingerprint.clone(),
        graph_sha256: state.graph_sha256.clone(),
        question,
        queried_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        hits,
    };
    validate_code_graph_query_cache(&repo_root, &cache)?;
    let path = ensure_code_graph_cache_root(&repo_root)?.join(CODE_GRAPH_QUERY_CACHE_FILE);
    atomic_write_json(&path, &cache)?;
    Ok(cache)
}

pub fn load_code_graph_query_cache(
    repo_root: impl AsRef<Path>,
) -> Result<Option<CodeGraphQueryCache>> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let path = code_graph_query_cache_path(&repo_root)?;
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Could not read {}", path.display()))?;
    let cache: CodeGraphQueryCache = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse {}", path.display()))?;
    validate_code_graph_query_cache(&repo_root, &cache)?;
    Ok(Some(cache))
}

pub fn cached_code_graph_hits_for_task(
    repo_root: impl AsRef<Path>,
    state: &CodeGraphState,
    task: &str,
) -> Result<Option<Vec<CodeGraphHit>>> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    validate_code_graph_state(&repo_root, state)?;
    let Some(cache) = load_code_graph_query_cache(&repo_root)? else {
        return Ok(None);
    };
    if cache.source_fingerprint != state.source_fingerprint
        || cache.graph_sha256 != state.graph_sha256
        || normalize_task_key(&cache.question) != normalize_task_key(task)
    {
        return Ok(None);
    }
    Ok(Some(cache.hits))
}

pub fn validate_code_graph_state(
    repo_root: impl AsRef<Path>,
    state: &CodeGraphState,
) -> Result<()> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    if state.schema_version != 1 {
        bail!(
            "Unsupported code graph state schema: {}",
            state.schema_version
        );
    }
    if state.project_id != current_project_id(&repo_root)? {
        bail!("Code graph state belongs to another project identity");
    }
    if state.repo_root != normalize_absolute_path(&repo_root) {
        bail!("Code graph state belongs to another repository root");
    }
    required_label(&state.provider, "Code graph provider")?;
    required_label(&state.provider_version, "Code graph provider version")?;
    validate_fingerprint(&state.source_fingerprint)?;
    if state.graph_sha256.len() != 64
        || !state
            .graph_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        bail!("Code graph state has an invalid graph checksum");
    }
    if state.graph_size_bytes > MAX_GRAPH_BYTES {
        bail!("Code graph state exceeds the graph size limit");
    }
    Ok(())
}

fn validate_code_graph_query_cache(repo_root: &Path, cache: &CodeGraphQueryCache) -> Result<()> {
    if cache.schema_version != 1 {
        bail!(
            "Unsupported code graph query cache schema: {}",
            cache.schema_version
        );
    }
    if cache.project_id != current_project_id(repo_root)? {
        bail!("Code graph query cache belongs to another project identity");
    }
    if cache.repo_root != normalize_absolute_path(repo_root) {
        bail!("Code graph query cache belongs to another repository root");
    }
    validate_fingerprint(&cache.source_fingerprint)?;
    if cache.graph_sha256.len() != 64
        || !cache
            .graph_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        bail!("Code graph query cache has an invalid graph checksum");
    }
    normalize_query(&cache.question)?;
    normalize_code_graph_hits(repo_root, cache.hits.clone(), QueryLimits::default())?;
    Ok(())
}

pub fn verify_graph_hit_source(
    repo_root: impl AsRef<Path>,
    hit: &CodeGraphHit,
) -> Result<SourceVerification> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let Some(source_file) = hit.source_file.as_deref() else {
        return Ok(SourceVerification {
            status: SourceVerificationStatus::MissingSource,
            source_file: None,
            evidence: None,
            message: "graph result has no source file to verify".to_string(),
        });
    };
    let relative = Path::new(source_file.trim());
    validate_safe_relative_path(relative, "Code graph source path")?;
    let source_path = repo_root.join(relative);
    validate_existing_ancestors_within_repo(&repo_root, &source_path)?;
    let metadata = match fs::symlink_metadata(&source_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(SourceVerification {
                status: SourceVerificationStatus::MissingSource,
                source_file: Some(normalize_relative_path(relative)),
                evidence: None,
                message: "graph source file no longer exists in this repository".to_string(),
            })
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!("Could not inspect source file: {}", source_path.display())
            })
        }
    };
    if metadata.file_type().is_symlink() || is_link_or_reparse_point(&source_path)? {
        bail!("Code graph source file must not be a symlink or junction")
    }
    if !metadata.is_file() {
        return Ok(SourceVerification {
            status: SourceVerificationStatus::MissingSource,
            source_file: Some(normalize_relative_path(relative)),
            evidence: None,
            message: "graph source path is not a regular repository file".to_string(),
        });
    }
    if metadata.len() > MAX_SOURCE_VERIFICATION_BYTES {
        return Ok(SourceVerification {
            status: SourceVerificationStatus::MissingEvidence,
            source_file: Some(normalize_relative_path(relative)),
            evidence: None,
            message: "graph source file exceeds the bounded verification read limit".to_string(),
        });
    }
    let content = match fs::read_to_string(&source_path) {
        Ok(content) => content,
        Err(_) => {
            return Ok(SourceVerification {
                status: SourceVerificationStatus::MissingEvidence,
                source_file: Some(normalize_relative_path(relative)),
                evidence: None,
                message: "graph source file is not readable as current text evidence".to_string(),
            })
        }
    };
    let matched = [hit.label.trim(), hit.node_id.trim()]
        .into_iter()
        .filter(|term| !term.is_empty())
        .find(|term| content.contains(term));
    let source_file = normalize_relative_path(relative);
    if hit.confidence == GraphConfidence::Inferred {
        return Ok(SourceVerification {
            status: SourceVerificationStatus::Advisory,
            source_file: Some(source_file),
            evidence: matched.map(|term| format!("current source contains `{term}`")),
            message: "inferred graph guidance remains advisory and cannot become proof".to_string(),
        });
    }
    match matched {
        Some(term) => Ok(SourceVerification {
            status: SourceVerificationStatus::Verified,
            source_file: Some(source_file),
            evidence: Some(format!("current source contains extracted symbol `{term}`")),
            message: "extracted graph guidance is verified against current source".to_string(),
        }),
        None => Ok(SourceVerification {
            status: SourceVerificationStatus::MissingEvidence,
            source_file: Some(source_file),
            evidence: None,
            message: "current source does not contain the extracted graph symbol".to_string(),
        }),
    }
}

pub fn task_needs_code_map(task: Option<&str>) -> bool {
    let Some(task) = task.map(str::trim).filter(|task| !task.is_empty()) else {
        return false;
    };
    let normalized = task.to_ascii_lowercase();
    let lightweight = ["readme", "docs", "copy", "typo", "wording", "config text"];
    if lightweight.iter().any(|term| normalized.contains(term)) {
        return false;
    }
    [
        "architecture",
        "dependency",
        "dependencies",
        "impact",
        "ownership",
        "owner",
        "entrypoint",
        "entry point",
        "call flow",
        "callflow",
        "cross-module",
        "cross module",
        "module",
        "refactor",
        "affected",
        "trace",
    ]
    .iter()
    .any(|term| normalized.contains(term))
}

pub fn render_optional_code_map_context(
    repo_root: impl AsRef<Path>,
    task: Option<&str>,
) -> Result<String> {
    if !task_needs_code_map(task) {
        return Ok(String::new());
    }
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let task = task.expect("checked above").trim();
    let mut output = String::from("## Optional Code Map\n\n");
    let state = match load_code_graph_state(&repo_root) {
        Ok(Some(state)) => state,
        Ok(None) => {
            output.push_str("- No local code map is ready for this code-navigation task.\n");
            output.push_str("- AI action: silently run `baron automation code-map refresh`, then `baron automation code-map query \"<task>\"`. Survey remains active while the map is unavailable.\n\n");
            return Ok(output);
        }
        Err(_) => {
            output
                .push_str("- Local code-map state is invalid; Survey fallback remains active.\n\n");
            return Ok(output);
        }
    };
    if state.provider != "graphify-local" || state.provider_version != "0.9.25" {
        output.push_str("- Local code map is incompatible; Survey fallback remains active.\n\n");
        return Ok(output);
    }
    match graph_state_freshness(&repo_root, &state) {
        Ok(GraphFreshness::Fresh) => {}
        Ok(_) => {
            output.push_str("- Local code map is stale for the current repository.\n");
            output.push_str("- AI action: silently refresh it before querying; Survey remains active until then.\n\n");
            return Ok(output);
        }
        Err(_) => {
            output.push_str(
                "- Local code map cannot be validated; Survey fallback remains active.\n\n",
            );
            return Ok(output);
        }
    }
    match cached_code_graph_hits_for_task(&repo_root, &state, task) {
        Ok(Some(hits)) if !hits.is_empty() => {
            output.push_str("- Fresh task-matched local graph hints; they are navigation guidance, not proof.\n\n");
            output.push_str(&render_code_graph_hits(&hits, QueryLimits::default()));
            if hits
                .iter()
                .any(|hit| hit.confidence == GraphConfidence::Inferred)
            {
                output.push_str("- Inferred results: verify against source before proof, trace, or durable memory.\n");
            }
            output.push('\n');
        }
        Ok(_) => {
            output.push_str("- A fresh local code map is available but has no cached answer for this exact task.\n");
            output.push_str("- AI action: silently run `baron automation code-map query \"<task>\"`; verify selected source before edits or proof.\n\n");
        }
        Err(_) => {
            output.push_str(
                "- Local code-map query cache is invalid; Survey fallback remains active.\n\n",
            );
        }
    }
    Ok(output)
}

pub fn graph_state_freshness(
    repo_root: impl AsRef<Path>,
    state: &CodeGraphState,
) -> Result<GraphFreshness> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    validate_code_graph_state(&repo_root, state)?;
    let fingerprint = compute_code_source_fingerprint(&repo_root)?;
    Ok(if fingerprint == state.source_fingerprint {
        GraphFreshness::Fresh
    } else {
        GraphFreshness::Stale
    })
}

pub fn normalize_code_graph_hits(
    repo_root: impl AsRef<Path>,
    hits: Vec<CodeGraphHit>,
    limits: QueryLimits,
) -> Result<Vec<CodeGraphHit>> {
    let repo_root = canonical_repo_root(repo_root.as_ref())?;
    let limits = limits.bounded();
    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    for mut hit in hits {
        hit.node_id = required_label(&hit.node_id, "Code graph node id")?;
        hit.label = required_label(&hit.label, "Code graph label")?;
        hit.explanation = required_label(&hit.explanation, "Code graph explanation")?;
        hit.relation = normalize_optional_label(hit.relation);
        if let Some(source_file) = hit.source_file.take() {
            let relative = Path::new(source_file.trim());
            validate_safe_relative_path(relative, "Code graph source path")?;
            let candidate = repo_root.join(relative);
            if !candidate.starts_with(&repo_root) {
                bail!("Code graph source path escapes the current repository");
            }
            hit.source_file = Some(normalize_relative_path(relative));
        }
        let key = format!(
            "{}\0{}\0{}\0{}",
            hit.node_id,
            hit.source_file.as_deref().unwrap_or_default(),
            hit.relation.as_deref().unwrap_or_default(),
            confidence_name(hit.confidence)
        );
        if seen.insert(key) {
            normalized.push(hit);
        }
    }
    normalized.sort_by(|left, right| {
        confidence_rank(left.confidence)
            .cmp(&confidence_rank(right.confidence))
            .then_with(|| left.source_file.cmp(&right.source_file))
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    normalized.truncate(limits.max_hits);
    Ok(normalized)
}

pub fn render_code_graph_hits(hits: &[CodeGraphHit], limits: QueryLimits) -> String {
    let limits = limits.bounded();
    if limits.max_hits == 0 || limits.max_chars == 0 {
        return String::new();
    }
    let mut output = String::new();
    for hit in hits.iter().take(limits.max_hits) {
        let source = hit.source_file.as_deref().unwrap_or("source unknown");
        let relation = hit
            .relation
            .as_deref()
            .map(|relation| format!(" ({relation})"))
            .unwrap_or_default();
        let line = format!(
            "- [{}] {}{} - {} - {}\n",
            confidence_name(hit.confidence),
            hit.label,
            relation,
            source,
            hit.explanation
        );
        let remaining = limits.max_chars.saturating_sub(output.chars().count());
        if remaining == 0 {
            break;
        }
        if line.chars().count() <= remaining {
            output.push_str(&line);
        } else {
            output.extend(line.chars().take(remaining));
            break;
        }
    }
    output
}

fn canonical_repo_root(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Could not resolve repository path: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!(
            "Repository path is not a directory: {}",
            canonical.display()
        );
    }
    Ok(canonical)
}

fn current_project_id(repo_root: &Path) -> Result<String> {
    match load_project_config(repo_root) {
        Ok(config) if !config.project_id.trim().is_empty() => Ok(config.project_id),
        _ => project_id_for_path(repo_root),
    }
}

fn validate_safe_relative_path(path: &Path, label: &str) -> Result<()> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        bail!("{label} must be a non-empty relative path");
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("{label} contains an unsafe path component")
            }
        }
    }
    Ok(())
}

fn validate_existing_ancestors_within_repo(repo_root: &Path, candidate: &Path) -> Result<()> {
    let relative = candidate
        .strip_prefix(repo_root)
        .context("Code graph cache path is outside the current repository")?;
    let mut current = repo_root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            continue;
        };
        current.push(part);
        if !current.exists() {
            continue;
        }
        if is_link_or_reparse_point(&current)? {
            bail!(
                "Code graph cache cannot traverse a symlink or junction: {}",
                current.display()
            );
        }
        let resolved = current
            .canonicalize()
            .with_context(|| format!("Could not resolve cache path: {}", current.display()))?;
        if !resolved.starts_with(repo_root) {
            bail!("Code graph cache path resolves outside the current repository");
        }
    }
    Ok(())
}

fn is_link_or_reparse_point(path: &Path) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(true);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        Ok(metadata.file_attributes() & 0x0400 != 0)
    }
    #[cfg(not(windows))]
    Ok(false)
}

fn is_skipped_source_path(path: &Path) -> bool {
    let names = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().to_ascii_lowercase()),
            _ => None,
        })
        .collect::<Vec<_>>();
    names.iter().any(|name| {
        matches!(
            name.as_str(),
            ".git"
                | ".baron"
                | ".codex"
                | ".claude"
                | "node_modules"
                | "target"
                | "dist"
                | "build"
                | ".next"
                | ".cache"
                | ".tmp"
                | "vendor"
                | "graphify-out"
                | "__pycache__"
        )
    }) || names.last().is_some_and(|name| {
        matches!(
            name.as_str(),
            "readme.md" | "release.md" | ".tmp-profile.txt"
        )
    }) || names
        .windows(2)
        .any(|pair| pair[0] == "docs" && pair[1] == "assessment")
        || names.windows(2).any(|pair| {
            (pair[0] == "docs"
                && matches!(
                    pair[1].as_str(),
                    "baron"
                        | "assessment"
                        | "superpowers"
                        | "baron_status.md"
                        | "baron_status.json"
                ))
                || (pair[0] == "notes" && pair[1] == "build-log")
        })
}

fn validate_fingerprint(value: &str) -> Result<()> {
    if value.len() != 64 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        bail!("Code graph source fingerprint must be a SHA-256 hex value");
    }
    Ok(())
}

fn required_label(value: &str, label: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{label} must not be empty");
    }
    Ok(value.to_string())
}

fn normalize_optional_label(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn normalize_query(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("Code graph query must not be empty");
    }
    if value.chars().count() > MAX_QUERY_CHARS {
        bail!("Code graph query exceeds the bounded input limit");
    }
    Ok(value.to_string())
}

fn normalize_task_key(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_relative_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_absolute_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn system_time_parts(time: SystemTime) -> Option<(u64, u32)> {
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    Some((duration.as_secs(), duration.subsec_nanos()))
}

fn sha256_file(path: &Path) -> Result<String> {
    let content = fs::read(path).with_context(|| format!("Could not read {}", path.display()))?;
    let mut digest = Sha256::new();
    digest.update(content);
    Ok(format!("{:x}", digest.finalize()))
}

fn atomic_write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = path.parent().context("Code graph state has no parent")?;
    if !parent.is_dir() {
        bail!(
            "Code graph state parent does not exist: {}",
            parent.display()
        );
    }
    let temp = parent.join(format!(
        ".{}-{}-tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("state"),
        std::process::id()
    ));
    let content = serde_json::to_vec_pretty(value)?;
    fs::write(&temp, content).with_context(|| format!("Could not write {}", temp.display()))?;
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("Could not replace {}", path.display()))?;
    }
    fs::rename(&temp, path).with_context(|| format!("Could not write {}", path.display()))
}

fn confidence_rank(confidence: GraphConfidence) -> u8 {
    match confidence {
        GraphConfidence::Extracted => 0,
        GraphConfidence::Inferred => 1,
    }
}

fn confidence_name(confidence: GraphConfidence) -> &'static str {
    match confidence {
        GraphConfidence::Extracted => "extracted",
        GraphConfidence::Inferred => "inferred",
    }
}
