use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{atomic::AtomicU64, atomic::Ordering, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::identity::project_id_for_path;

const RECEIPT_PATH: &str = ".baron/cache/execution-receipts.jsonl";
const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_ARG_BYTES: usize = 16 * 1024;

/// The process-local authority registry retains the exact integrity value that
/// the trusted runner emitted.  Keeping only receipt IDs would let a caller
/// rewrite the JSONL record and recompute its unkeyed digest during the same
/// process, turning an otherwise diagnostic record into false authority.
static CURRENT_RECEIPTS: OnceLock<Mutex<std::collections::BTreeMap<String, String>>> =
    OnceLock::new();
static RECEIPT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionResult {
    Passed,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptProvenance {
    TrustedCurrentOperation,
    #[default]
    PersistedDiagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptContext {
    pub task_id: String,
    pub operation_id: String,
    pub adapter: String,
    pub session_id: String,
    pub request_id: String,
    pub gate_kind: String,
}

impl ReceiptContext {
    pub fn new(
        task_id: impl Into<String>,
        operation_id: impl Into<String>,
        adapter: impl Into<String>,
        session_id: impl Into<String>,
        request_id: impl Into<String>,
        gate_kind: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            operation_id: operation_id.into(),
            adapter: adapter.into(),
            session_id: session_id.into(),
            request_id: request_id.into(),
            gate_kind: gate_kind.into(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        validate_context(self)
    }
}

impl ExecutionResult {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::TimedOut => "timed_out",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub schema_version: u32,
    pub receipt_id: String,
    pub project_id: String,
    pub source_fingerprint: String,
    pub capability: String,
    pub provider: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: String,
    pub started_at: String,
    pub finished_at: String,
    pub exit_code: Option<i32>,
    pub result: ExecutionResult,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub stdout_excerpt: String,
    pub stderr_excerpt: String,
    pub artifact_digests: Vec<String>,
    /// The operation binding is optional only for legacy generic executions.
    /// A gate-authoritative receipt must carry every field and be produced by
    /// `execute_command_with_context` in the current process.
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub adapter: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub gate_kind: Option<String>,
    #[serde(default)]
    pub provenance: ReceiptProvenance,
    pub integrity_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRequest {
    pub capability: String,
    pub provider: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
    pub timeout: Duration,
}

pub fn execute_command(request: ExecutionRequest) -> Result<ExecutionReceipt> {
    execute_command_internal(request, None)
}

pub fn execute_command_with_context(
    request: ExecutionRequest,
    context: ReceiptContext,
) -> Result<ExecutionReceipt> {
    context.validate()?;
    execute_command_internal(request, Some(context))
}

fn execute_command_internal(
    request: ExecutionRequest,
    context: Option<ReceiptContext>,
) -> Result<ExecutionReceipt> {
    validate_request(&request)?;
    let repo_root = request.working_directory.canonicalize().with_context(|| {
        format!(
            "Could not resolve execution working directory: {}",
            request.working_directory.display()
        )
    })?;
    if !repo_root.is_dir() {
        bail!("Execution working directory is not a directory");
    }
    let project_id = project_id_for_path(&repo_root)?;
    let source_fingerprint = source_fingerprint(&repo_root)?;
    let started_at = now();
    let started = Instant::now();
    let mut command = Command::new(&request.executable);
    command
        .args(&request.arguments)
        .current_dir(&repo_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().with_context(|| {
        format!(
            "Trusted execution runner could not start `{}`",
            request.executable
        )
    })?;
    let stdout_pipe = child
        .stdout
        .take()
        .context("Trusted execution runner did not expose stdout")?;
    let stderr_pipe = child
        .stderr
        .take()
        .context("Trusted execution runner did not expose stderr")?;
    let stdout_reader = std::thread::spawn(move || read_bounded(stdout_pipe, MAX_CAPTURE_BYTES));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr_pipe, MAX_CAPTURE_BYTES));
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if started.elapsed() >= request.timeout {
            child.kill().ok();
            child.wait().ok();
            break None;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| anyhow::anyhow!("Trusted stdout reader panicked"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| anyhow::anyhow!("Trusted stderr reader panicked"))??;
    let result = match status {
        Some(status) if status.success() => ExecutionResult::Passed,
        Some(_) => ExecutionResult::Failed,
        None => ExecutionResult::TimedOut,
    };
    let exit_code = status.and_then(|value| value.code());
    let mut receipt = ExecutionReceipt {
        schema_version: 1,
        receipt_id: format!(
            "receipt-{}",
            digest_hex(
                format!(
                    "{project_id}:{started_at}:{}",
                    RECEIPT_SEQUENCE.fetch_add(1, Ordering::Relaxed)
                ),
                16,
            )
        ),
        project_id,
        source_fingerprint,
        capability: normalize_label(&request.capability),
        provider: normalize_label(&request.provider),
        executable: request.executable.clone(),
        arguments: request.arguments.clone(),
        working_directory: repo_root.to_string_lossy().replace('\\', "/"),
        started_at,
        finished_at: now(),
        exit_code,
        result,
        stdout_digest: digest_bytes(&stdout),
        stderr_digest: digest_bytes(&stderr),
        stdout_excerpt: redact(&String::from_utf8_lossy(&stdout)),
        stderr_excerpt: redact(&String::from_utf8_lossy(&stderr)),
        artifact_digests: Vec::new(),
        task_id: context.as_ref().map(|value| value.task_id.clone()),
        operation_id: context.as_ref().map(|value| value.operation_id.clone()),
        adapter: context.as_ref().map(|value| value.adapter.clone()),
        session_id: context.as_ref().map(|value| value.session_id.clone()),
        request_id: context.as_ref().map(|value| value.request_id.clone()),
        gate_kind: context.as_ref().map(|value| value.gate_kind.clone()),
        provenance: ReceiptProvenance::TrustedCurrentOperation,
        integrity_digest: String::new(),
    };
    receipt.integrity_digest = receipt_integrity(&receipt)?;
    append_receipt(&repo_root, &receipt)?;
    register_current_receipt(&receipt.receipt_id, &receipt.integrity_digest)?;
    Ok(receipt)
}

pub fn load_receipts(repo_root: impl AsRef<Path>) -> Result<Vec<ExecutionReceipt>> {
    let path = repo_root.as_ref().join(RECEIPT_PATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut receipts = Vec::new();
    for (line_number, line) in fs::read_to_string(&path)?.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let receipt: ExecutionReceipt = serde_json::from_str(line)
            .with_context(|| format!("Malformed execution receipt at line {}", line_number + 1))?;
        if receipt_integrity(&receipt)? != receipt.integrity_digest {
            bail!(
                "Execution receipt integrity check failed for {}",
                receipt.receipt_id
            );
        }
        receipts.push(receipt);
    }
    Ok(receipts)
}

pub fn receipt_is_current(repo_root: impl AsRef<Path>, receipt: &ExecutionReceipt) -> Result<bool> {
    let repo_root = repo_root.as_ref().canonicalize()?;
    Ok(receipt.project_id == project_id_for_path(&repo_root)?
        && receipt.working_directory == repo_root.to_string_lossy().replace('\\', "/")
        && receipt.source_fingerprint == source_fingerprint(&repo_root)?
        && receipt.result == ExecutionResult::Passed
        && receipt_integrity(receipt)? == receipt.integrity_digest)
}

/// A receipt is gate-authoritative only while the trusted runner that created
/// it is still the current Baron operation. Persisted JSONL records are
/// intentionally diagnostic after process restart; their unkeyed integrity
/// digest cannot establish authenticity.
pub fn receipt_is_current_authority(
    repo_root: impl AsRef<Path>,
    receipt: &ExecutionReceipt,
) -> Result<bool> {
    if receipt.provenance != ReceiptProvenance::TrustedCurrentOperation
        || receipt
            .task_id
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
        || receipt
            .operation_id
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
        || receipt
            .adapter
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
        || receipt
            .session_id
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
        || receipt
            .request_id
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
        || receipt
            .gate_kind
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
    {
        return Ok(false);
    }
    let Some(registry) = CURRENT_RECEIPTS.get() else {
        return Ok(false);
    };
    let registered = registry
        .lock()
        .map(|receipts| {
            receipts
                .get(&receipt.receipt_id)
                .is_some_and(|integrity| integrity == &receipt.integrity_digest)
        })
        .unwrap_or(false);
    Ok(registered && receipt_is_current(repo_root, receipt)?)
}

pub fn receipt_matches_context(
    repo_root: impl AsRef<Path>,
    receipt: &ExecutionReceipt,
    context: &ReceiptContext,
) -> Result<bool> {
    context.validate()?;
    Ok(receipt_is_current_authority(repo_root, receipt)?
        && receipt.task_id.as_deref() == Some(context.task_id.trim())
        && receipt.operation_id.as_deref() == Some(context.operation_id.trim())
        && receipt.adapter.as_deref() == Some(context.adapter.trim())
        && receipt.session_id.as_deref() == Some(context.session_id.trim())
        && receipt.request_id.as_deref() == Some(context.request_id.trim())
        && receipt.gate_kind.as_deref() == Some(context.gate_kind.trim()))
}

fn register_current_receipt(receipt_id: &str, integrity_digest: &str) -> Result<()> {
    let registry = CURRENT_RECEIPTS.get_or_init(|| Mutex::new(std::collections::BTreeMap::new()));
    registry
        .lock()
        .map_err(|_| anyhow::anyhow!("trusted receipt registry was poisoned"))?
        .insert(receipt_id.to_string(), integrity_digest.to_string());
    Ok(())
}

fn validate_context(context: &ReceiptContext) -> Result<()> {
    for (name, value) in [
        ("task_id", context.task_id.as_str()),
        ("operation_id", context.operation_id.as_str()),
        ("adapter", context.adapter.as_str()),
        ("session_id", context.session_id.as_str()),
        ("request_id", context.request_id.as_str()),
        ("gate_kind", context.gate_kind.as_str()),
    ] {
        if value.trim().is_empty()
            || value.chars().count() > 240
            || value
                .chars()
                .any(|character| matches!(character, '`' | '\n' | '\r'))
        {
            bail!("trusted receipt {name} is missing or exceeds the bounded size");
        }
    }
    if !matches!(
        context.adapter.trim().to_ascii_lowercase().as_str(),
        "codex" | "claude"
    ) {
        bail!("trusted receipt adapter must be codex or claude");
    }
    Ok(())
}

fn validate_request(request: &ExecutionRequest) -> Result<()> {
    if request.capability.trim().is_empty() || request.provider.trim().is_empty() {
        bail!("Trusted execution requires capability and provider labels");
    }
    if request.executable.trim().is_empty() {
        bail!("Trusted execution requires an executable");
    }
    if request
        .arguments
        .iter()
        .any(|arg| arg.len() > MAX_ARG_BYTES)
    {
        bail!("Trusted execution argument exceeds the bounded size");
    }
    if request.timeout.is_zero() || request.timeout > Duration::from_secs(300) {
        bail!("Trusted execution timeout must be between one millisecond and five minutes");
    }
    Ok(())
}

fn append_receipt(repo_root: &Path, receipt: &ExecutionReceipt) -> Result<()> {
    let path = repo_root.join(RECEIPT_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let line = format!("{}\n", serde_json::to_string(receipt)?);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Could not open {}", path.display()))?
        .write_all(line.as_bytes())
        .with_context(|| format!("Could not append {}", path.display()))
}

fn read_bounded(mut reader: impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if bytes.len() < limit {
            let remaining = limit - bytes.len();
            let copied = read.min(remaining);
            bytes.extend_from_slice(&buffer[..copied]);
            truncated |= copied < read;
        } else {
            truncated = true;
        }
    }
    if truncated {
        bytes.extend_from_slice(b"\n[baron output truncated]");
    }
    Ok(bytes)
}

fn source_fingerprint(repo_root: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_source_files(repo_root, repo_root, &mut files)?;
    files.sort();
    let mut digest = Sha256::new();
    for relative in files {
        let path = repo_root.join(&relative);
        let metadata = fs::metadata(&path)?;
        let contents = fs::read(&path).with_context(|| {
            format!(
                "Could not read source file for fingerprint: {}",
                path.display()
            )
        })?;
        digest.update(relative.as_bytes());
        digest.update(metadata.len().to_le_bytes());
        digest.update(digest_bytes(&contents).as_bytes());
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn collect_source_files(root: &Path, current: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let normalized = relative.to_string_lossy().replace('\\', "/");
        let first = relative
            .components()
            .next()
            .and_then(|value| value.as_os_str().to_str());
        if matches!(
            first,
            Some(
                ".git"
                    | ".baron"
                    | ".codex"
                    | ".claude"
                    | "target"
                    | "node_modules"
                    | "dist"
                    | "build"
            )
        ) || normalized == "docs/baron"
            || normalized.starts_with("docs/baron/")
        {
            continue;
        }
        if entry.file_type()?.is_dir() {
            collect_source_files(root, &path, files)?;
        } else if entry.file_type()?.is_file() {
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn receipt_integrity(receipt: &ExecutionReceipt) -> Result<String> {
    // This digest detects accidental or malformed persisted state only. It is
    // deliberately not treated as signer authenticity; authority comes from
    // the in-process trusted runner registry and exact identity binding.
    let mut value = receipt.clone();
    value.integrity_digest.clear();
    Ok(digest_bytes(&serde_json::to_vec(&value)?))
}

fn digest_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn digest_hex(value: String, take: usize) -> String {
    digest_bytes(value.as_bytes()).chars().take(take).collect()
}

fn normalize_label(value: &str) -> String {
    value.trim().chars().take(120).collect()
}

fn redact(value: &str) -> String {
    let redacted = value
        .lines()
        .map(|line| {
            let lower = line.to_lowercase();
            if [
                "token=",
                "password=",
                "secret=",
                "api_key=",
                "authorization:",
            ]
            .iter()
            .any(|term| lower.contains(term))
            {
                "[baron secret-bearing output redacted]"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut output = redacted.chars().take(8_000).collect::<String>();
    if redacted.chars().count() > 8_000 {
        output.push_str("\n[baron output truncated]");
    }
    output
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}

use std::io::Write;
