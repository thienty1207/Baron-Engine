//! Optional, local-only Graphify adapter.
//!
//! This module deliberately owns a very small command contract. It never
//! installs Graphify, configures hooks, touches a Vault, or uses a network/API
//! backend. A missing or incompatible provider is a diagnostic, not a failure
//! of Baron itself.

use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use wait_timeout::ChildExt;

use crate::code_graph::{
    code_graph_cache_root, compute_code_source_fingerprint, ensure_code_graph_cache_root,
    graph_state_freshness, load_code_graph_state, normalize_code_graph_hits,
    validate_code_graph_artifact, validate_code_graph_cache_path, write_code_graph_state,
    CodeGraphHit, CodeGraphProvider, CodeGraphState, GraphConfidence, GraphFreshness,
    ProviderProbe, QueryLimits, MAX_GRAPH_BYTES,
};

pub const SUPPORTED_GRAPHIFY_VERSION: &str = "0.9.25";
pub const AUDITED_GRAPHIFY_REVISION: &str = "2fa6cd3d5548577f8c5f591b713f0bf80c1af183";
pub const GRAPHIFY_PROBE_TIMEOUT: Duration = Duration::from_secs(3);
pub const GRAPHIFY_REFRESH_TIMEOUT: Duration = Duration::from_secs(120);
pub const GRAPHIFY_QUERY_TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_PROVIDER_STDOUT_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_PROVIDER_STDERR_BYTES: u64 = 256 * 1024;

const PROVIDER_NAME: &str = "graphify-local";
const GRAPHIFY_DIR: &str = "graphify";
const STAGING_PREFIX: &str = ".staging";
const QUERY_PREFIX: &str = ".query";
const BACKUP_PREFIX: &str = ".backup";
const GRAPH_OUTPUT_DIRECTORY: &str = "graphify-out";
const GRAPH_FILE_NAME: &str = "graph.json";

static UNIQUE_PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct GraphifyLimits {
    pub probe_timeout: Duration,
    pub refresh_timeout: Duration,
    pub query_timeout: Duration,
    pub max_graph_bytes: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
}

impl Default for GraphifyLimits {
    fn default() -> Self {
        Self {
            probe_timeout: GRAPHIFY_PROBE_TIMEOUT,
            refresh_timeout: GRAPHIFY_REFRESH_TIMEOUT,
            query_timeout: GRAPHIFY_QUERY_TIMEOUT,
            max_graph_bytes: MAX_GRAPH_BYTES,
            max_stdout_bytes: MAX_PROVIDER_STDOUT_BYTES,
            max_stderr_bytes: MAX_PROVIDER_STDERR_BYTES,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphifyProvider {
    program: PathBuf,
    prefix_args: Vec<OsString>,
    limits: GraphifyLimits,
}

#[derive(Debug)]
struct ProcessCapture {
    status: ExitStatus,
    stdout: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct RawGraphHit {
    node_id: String,
    label: String,
    #[serde(default)]
    source_file: Option<String>,
    #[serde(default)]
    relation: Option<String>,
    #[serde(default)]
    confidence: Option<String>,
    explanation: String,
    #[serde(default)]
    score: Option<f64>,
}

#[derive(Debug)]
struct ScoredHit {
    score: f64,
    hit: CodeGraphHit,
}

impl GraphifyProvider {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            prefix_args: Vec::new(),
            limits: GraphifyLimits::default(),
        }
    }

    #[cfg(windows)]
    pub fn powershell_script(script: impl AsRef<Path>) -> Self {
        Self {
            program: PathBuf::from("powershell.exe"),
            prefix_args: vec![
                OsString::from("-NoProfile"),
                OsString::from("-NonInteractive"),
                OsString::from("-ExecutionPolicy"),
                OsString::from("Bypass"),
                OsString::from("-File"),
                script.as_ref().as_os_str().to_os_string(),
            ],
            limits: GraphifyLimits::default(),
        }
    }

    pub fn with_limits(mut self, limits: GraphifyLimits) -> Self {
        self.limits = limits;
        self
    }

    fn expected_probe(&self, repo_root: &Path) -> Result<ProviderProbe> {
        // A version probe must not create a project cache. This keeps a missing
        // optional binary entirely non-mutating for new repositories.
        let run_dir = create_system_run_directory(QUERY_PREFIX)?;
        let result = self.run_command(
            repo_root,
            &[OsString::from("--version")],
            self.limits.probe_timeout,
            &run_dir,
        );
        let cleanup = remove_system_directory(&run_dir);
        let capture = match result {
            Ok(capture) => capture,
            Err(error) => {
                let _ = cleanup;
                return Ok(ProviderProbe {
                    provider: PROVIDER_NAME.to_string(),
                    present: false,
                    version: None,
                    diagnostics: vec![controlled_provider_diagnostic(&error)],
                });
            }
        };
        cleanup?;
        if !capture.status.success() {
            return Ok(ProviderProbe {
                provider: PROVIDER_NAME.to_string(),
                present: true,
                version: None,
                diagnostics: vec![
                    "Graphify version check did not complete successfully".to_string()
                ],
            });
        }
        let version = parse_version(&capture.stdout);
        let mut diagnostics = Vec::new();
        if version.as_deref() != Some(SUPPORTED_GRAPHIFY_VERSION) {
            diagnostics.push(format!(
                "Graphify is optional and requires exactly version {SUPPORTED_GRAPHIFY_VERSION}"
            ));
        }
        Ok(ProviderProbe {
            provider: PROVIDER_NAME.to_string(),
            present: true,
            version,
            diagnostics,
        })
    }

    fn require_supported(&self, repo_root: &Path) -> Result<ProviderProbe> {
        let probe = self.expected_probe(repo_root)?;
        if !probe.present {
            bail!("Graphify is unavailable locally; Survey fallback remains active");
        }
        if probe.version.as_deref() != Some(SUPPORTED_GRAPHIFY_VERSION) {
            bail!(
                "Graphify is optional and must be exactly version {SUPPORTED_GRAPHIFY_VERSION}; Survey fallback remains active"
            );
        }
        Ok(probe)
    }

    fn run_command(
        &self,
        repo_root: &Path,
        args: &[OsString],
        timeout: Duration,
        run_dir: &Path,
    ) -> Result<ProcessCapture> {
        let stdout_path = run_dir.join("stdout.log");
        let stderr_path = run_dir.join("stderr.log");
        let stdout = create_output_file(&stdout_path)?;
        let stderr = create_output_file(&stderr_path)?;
        let mut command = Command::new(&self.program);
        command
            .args(&self.prefix_args)
            .args(args)
            .current_dir(repo_root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .env("GRAPHIFY_QUERY_LOG_DISABLE", "1")
            .env_remove("GRAPHIFY_API_KEY")
            .env_remove("GRAPHIFY_TOKEN")
            .env_remove("OPENAI_API_KEY")
            .env_remove("ANTHROPIC_API_KEY");
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                bail!("Graphify command is not available locally")
            }
            Err(_) => bail!("Graphify command could not be started locally"),
        };
        let status = match child.wait_timeout(timeout)? {
            Some(status) => status,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                bail!("Graphify command timed out; Survey fallback remains active");
            }
        };
        let stdout = read_bounded_file(
            &stdout_path,
            self.limits.max_stdout_bytes,
            "Graphify stdout",
        )?;
        let _stderr = read_bounded_file(
            &stderr_path,
            self.limits.max_stderr_bytes,
            "Graphify stderr",
        )?;
        Ok(ProcessCapture { status, stdout })
    }

    fn promote_refresh(
        &self,
        repo_root: &Path,
        staging: &Path,
        source_fingerprint: &str,
    ) -> Result<CodeGraphState> {
        let cache_root = ensure_code_graph_cache_root(repo_root)?;
        let graph_path = staging.join(GRAPH_OUTPUT_DIRECTORY).join(GRAPH_FILE_NAME);
        validate_code_graph_cache_path(repo_root, &graph_path)?;
        validate_graph_json(&graph_path, self.limits.max_graph_bytes)?;

        let provider_root =
            ensure_relative_directory(repo_root, &cache_root, Path::new(GRAPHIFY_DIR))?;
        let target = provider_root.join(source_fingerprint);
        validate_code_graph_cache_path(repo_root, &target)?;
        let backup = cache_root.join(unique_directory_name(BACKUP_PREFIX));
        validate_code_graph_cache_path(repo_root, &backup)?;
        let had_target = target.exists();
        if had_target {
            fs::rename(&target, &backup).with_context(|| {
                format!("Could not stage prior Graphify cache: {}", target.display())
            })?;
        }
        if let Err(error) = fs::rename(staging, &target) {
            if had_target {
                let _ = fs::rename(&backup, &target);
            }
            return Err(error).with_context(|| "Could not promote validated Graphify cache");
        }
        let promoted_graph = target.join(GRAPH_OUTPUT_DIRECTORY).join(GRAPH_FILE_NAME);
        let state_result = write_code_graph_state(
            repo_root,
            PROVIDER_NAME,
            SUPPORTED_GRAPHIFY_VERSION,
            source_fingerprint,
            &promoted_graph,
        );
        match state_result {
            Ok(state) => {
                if had_target {
                    let _ = remove_owned_directory(repo_root, &backup);
                }
                Ok(state)
            }
            Err(error) => {
                let _ = remove_owned_directory(repo_root, &target);
                if had_target {
                    let _ = fs::rename(&backup, &target);
                }
                Err(error)
            }
        }
    }
}

impl CodeGraphProvider for GraphifyProvider {
    fn probe(&self, repo_root: &Path) -> Result<ProviderProbe> {
        self.expected_probe(repo_root)
    }

    fn refresh(&self, repo_root: &Path, cache_root: &Path) -> Result<CodeGraphState> {
        let repo_root = repo_root.canonicalize().with_context(|| {
            format!("Could not resolve repository path: {}", repo_root.display())
        })?;
        let expected_cache = code_graph_cache_root(&repo_root)?;
        if cache_root != expected_cache {
            bail!("Graphify may only write to the current project's managed cache");
        }
        self.require_supported(&repo_root)?;
        let _cache_root = ensure_code_graph_cache_root(&repo_root)?;
        let source_fingerprint = compute_code_source_fingerprint(&repo_root)?;
        let staging = create_owned_directory(&repo_root, STAGING_PREFIX)?;
        let graph_output = staging.join(GRAPH_OUTPUT_DIRECTORY);
        let args = vec![
            OsString::from("extract"),
            provider_path(&repo_root),
            OsString::from("--code-only"),
            OsString::from("--out"),
            provider_path(&graph_output),
            OsString::from("--no-cluster"),
        ];
        let result = self.run_command(&repo_root, &args, self.limits.refresh_timeout, &staging);
        let capture = match result {
            Ok(capture) => capture,
            Err(error) => {
                let _ = remove_owned_directory(&repo_root, &staging);
                return Err(error);
            }
        };
        if !capture.status.success() {
            let _ = remove_owned_directory(&repo_root, &staging);
            bail!(
                "Graphify extraction did not complete successfully; Survey fallback remains active"
            );
        }
        let promoted = self.promote_refresh(&repo_root, &staging, &source_fingerprint);
        if promoted.is_err() && staging.exists() {
            let _ = remove_owned_directory(&repo_root, &staging);
        }
        promoted
    }

    fn query(
        &self,
        repo_root: &Path,
        cache_root: &Path,
        question: &str,
        limits: QueryLimits,
    ) -> Result<Vec<CodeGraphHit>> {
        let repo_root = repo_root.canonicalize().with_context(|| {
            format!("Could not resolve repository path: {}", repo_root.display())
        })?;
        let expected_cache = code_graph_cache_root(&repo_root)?;
        if cache_root != expected_cache {
            bail!("Graphify may only read from the current project's managed cache");
        }
        let question = question.trim();
        if question.is_empty() {
            bail!("Graphify query must not be empty");
        }
        if question.chars().count() > crate::code_graph::MAX_QUERY_CHARS {
            bail!("Graphify query exceeds the bounded input limit");
        }
        self.require_supported(&repo_root)?;
        let state = load_code_graph_state(&repo_root)?
            .context("No local Graphify cache is available; Survey fallback remains active")?;
        if state.provider != PROVIDER_NAME || state.provider_version != SUPPORTED_GRAPHIFY_VERSION {
            bail!("Local code map is incompatible; Survey fallback remains active");
        }
        if graph_state_freshness(&repo_root, &state)? != GraphFreshness::Fresh {
            bail!("Local code map is stale; Survey fallback remains active");
        }
        let graph_path = validate_code_graph_artifact(&repo_root, &state)?;
        let run_dir = create_owned_directory(&repo_root, QUERY_PREFIX)?;
        let limits = limits.bounded();
        let args = vec![
            OsString::from("query"),
            OsString::from(question),
            OsString::from("--graph"),
            provider_path(&graph_path),
            OsString::from("--json"),
            OsString::from("--budget"),
            OsString::from(limits.max_hits.to_string()),
        ];
        let result = self.run_command(&repo_root, &args, self.limits.query_timeout, &run_dir);
        let cleanup = remove_owned_directory(&repo_root, &run_dir);
        let capture = result?;
        cleanup?;
        if !capture.status.success() {
            bail!("Graphify query did not complete successfully; Survey fallback remains active");
        }
        parse_query_hits(&repo_root, &capture.stdout, limits)
    }
}

fn parse_version(stdout: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(stdout).ok()?;
    text.lines().find_map(|line| {
        line.trim()
            .strip_prefix("graphify ")
            .map(str::trim)
            .filter(|version| !version.is_empty())
            .map(str::to_string)
    })
}

fn parse_query_hits(
    repo_root: &Path,
    stdout: &[u8],
    limits: QueryLimits,
) -> Result<Vec<CodeGraphHit>> {
    let value: Value = serde_json::from_slice(stdout)
        .context("Graphify query returned malformed JSON; Survey fallback remains active")?;
    let raw_hits: Vec<RawGraphHit> = match value {
        Value::Array(_) => serde_json::from_value(value)?,
        Value::Object(mut object) => {
            let results = object
                .remove("results")
                .or_else(|| object.remove("hits"))
                .context("Graphify query JSON has no results array")?;
            serde_json::from_value(results)?
        }
        _ => bail!("Graphify query JSON must be an array or contain a results array"),
    };
    let mut scored = Vec::new();
    for raw in raw_hits {
        let candidate = CodeGraphHit {
            node_id: raw.node_id,
            label: raw.label,
            source_file: raw.source_file,
            relation: raw.relation,
            confidence: parse_confidence(raw.confidence.as_deref()),
            explanation: raw.explanation,
        };
        let mut normalized = normalize_code_graph_hits(
            repo_root,
            vec![candidate],
            QueryLimits {
                max_hits: 1,
                max_chars: limits.max_chars,
            },
        )?;
        if let Some(hit) = normalized.pop() {
            scored.push(ScoredHit {
                score: raw.score.filter(|score| score.is_finite()).unwrap_or(0.0),
                hit,
            });
        }
    }
    scored.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                confidence_order(left.hit.confidence).cmp(&confidence_order(right.hit.confidence))
            })
            .then_with(|| left.hit.source_file.cmp(&right.hit.source_file))
            .then_with(|| left.hit.label.cmp(&right.hit.label))
            .then_with(|| left.hit.node_id.cmp(&right.hit.node_id))
    });
    let mut seen = std::collections::BTreeSet::new();
    let mut hits = Vec::new();
    for scored_hit in scored {
        let key = format!(
            "{}\0{}\0{}\0{:?}",
            scored_hit.hit.node_id,
            scored_hit.hit.source_file.as_deref().unwrap_or_default(),
            scored_hit.hit.relation.as_deref().unwrap_or_default(),
            scored_hit.hit.confidence
        );
        if seen.insert(key) {
            hits.push(scored_hit.hit);
        }
        if hits.len() >= limits.max_hits {
            break;
        }
    }
    Ok(hits)
}

fn parse_confidence(value: Option<&str>) -> GraphConfidence {
    match value
        .map(str::trim)
        .map(|value| value.eq_ignore_ascii_case("extracted"))
    {
        Some(true) => GraphConfidence::Extracted,
        _ => GraphConfidence::Inferred,
    }
}

fn confidence_order(confidence: GraphConfidence) -> u8 {
    match confidence {
        GraphConfidence::Extracted => 0,
        GraphConfidence::Inferred => 1,
    }
}

fn validate_graph_json(path: &Path, max_graph_bytes: u64) -> Result<()> {
    let content = read_bounded_file(path, max_graph_bytes, "Graphify graph")?;
    let value: Value =
        serde_json::from_slice(&content).context("Graphify graph is not valid JSON")?;
    let object = value
        .as_object()
        .context("Graphify graph JSON must be an object")?;
    for key in ["nodes", "edges"] {
        if object.contains_key(key) && !object[key].is_array() {
            bail!("Graphify graph field `{key}` must be an array when present");
        }
    }
    Ok(())
}

fn create_output_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| {
            format!(
                "Could not create Baron-owned provider output: {}",
                path.display()
            )
        })
}

fn read_bounded_file(path: &Path, maximum: u64, label: &str) -> Result<Vec<u8>> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("Could not inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular Baron-owned file");
    }
    if metadata.len() > maximum {
        bail!(
            "{label} exceeds its bounded output limit ({} bytes)",
            metadata.len()
        );
    }
    fs::read(path).with_context(|| format!("Could not read {label}"))
}

fn create_owned_directory(repo_root: &Path, prefix: &str) -> Result<PathBuf> {
    let cache_root = ensure_code_graph_cache_root(repo_root)?;
    for _ in 0..32 {
        let candidate = cache_root.join(unique_directory_name(prefix));
        validate_code_graph_cache_path(repo_root, &candidate)?;
        match fs::create_dir(&candidate) {
            Ok(()) => {
                validate_code_graph_cache_path(repo_root, &candidate)?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not create Baron-owned provider directory: {}",
                        candidate.display()
                    )
                })
            }
        }
    }
    bail!("Could not allocate a Baron-owned provider directory")
}

fn create_system_run_directory(prefix: &str) -> Result<PathBuf> {
    let temp_root = std::env::temp_dir();
    for _ in 0..32 {
        let candidate = temp_root.join(format!("baron-{prefix}-{}", unique_directory_name("run")));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| "Could not create Baron-owned temporary provider directory")
            }
        }
    }
    bail!("Could not allocate a Baron-owned temporary provider directory")
}

fn ensure_relative_directory(repo_root: &Path, root: &Path, relative: &Path) -> Result<PathBuf> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(part) = component else {
            bail!("Managed Graphify cache path must be relative and safe");
        };
        current.push(part);
        validate_code_graph_cache_path(repo_root, &current)?;
        if current.exists() {
            if !current.is_dir() {
                bail!("Managed Graphify cache component is not a directory");
            }
            continue;
        }
        match fs::create_dir(&current) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| "Could not create managed Graphify cache directory")
            }
        }
        validate_code_graph_cache_path(repo_root, &current)?;
    }
    Ok(current)
}

fn remove_owned_directory(repo_root: &Path, path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    validate_code_graph_cache_path(repo_root, path)?;
    remove_owned_tree(path)
}

fn remove_system_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let temp_root = std::env::temp_dir();
    if !path.starts_with(&temp_root) {
        bail!("Refusing to remove a temporary provider directory outside the system temp root");
    }
    remove_owned_tree(path)
}

fn remove_owned_tree(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Could not inspect Baron-owned path: {}", path.display()))?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!("Refusing to remove a linked or junctioned provider path")
    }
    if metadata.is_file() {
        fs::remove_file(path)?;
        return Ok(());
    }
    if !metadata.is_dir() {
        bail!("Refusing to remove a non-directory provider path")
    }
    for entry in fs::read_dir(path)? {
        remove_owned_tree(&entry?.path())?;
    }
    fs::remove_dir(path)?;
    Ok(())
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x0400 != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

fn unique_directory_name(prefix: &str) -> String {
    let counter = UNIQUE_PATH_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}-{}-{nanos}-{counter}", std::process::id())
}

fn controlled_provider_diagnostic(error: &anyhow::Error) -> String {
    let text = error.to_string();
    if text.contains("timed out") {
        "Graphify timed out locally; Survey fallback remains active".to_string()
    } else if text.contains("not available") {
        "Graphify is not installed locally; Survey fallback remains active".to_string()
    } else {
        "Graphify could not be checked locally; Survey fallback remains active".to_string()
    }
}

fn provider_path(path: &Path) -> OsString {
    #[cfg(windows)]
    {
        let path = path.as_os_str().to_string_lossy();
        if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
            return OsString::from(format!(r"\\{unc}"));
        }
        if let Some(path) = path.strip_prefix(r"\\?\") {
            return OsString::from(path);
        }
        OsString::from(path.as_ref())
    }
    #[cfg(not(windows))]
    path.as_os_str().to_os_string()
}
