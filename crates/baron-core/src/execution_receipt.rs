use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::identity::project_id_for_path;
use crate::operation::LifecycleIdentity;
use crate::receipt_authority::{hex_encode, ReceiptAuthority};
use crate::safe_io::{acquire_project_lock, append_text, read_text};

const RECEIPT_PATH: &str = ".baron/cache/execution-receipts.jsonl";
const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_ARG_BYTES: usize = 16 * 1024;
const RECEIPT_SCHEMA_V1: u32 = 1;
const RECEIPT_SCHEMA_V2: u32 = 2;
const RECEIPT_SIGNATURE_DOMAIN: &[u8] = b"baron-execution-receipt-v2\0";

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

    pub fn for_identity(
        identity: &LifecycleIdentity,
        gate_kind: impl Into<String>,
    ) -> Result<Self> {
        let context = Self::new(
            identity.task_id(),
            identity.operation_id(),
            identity.adapter().as_str(),
            identity.session_id(),
            identity.request_id(),
            gate_kind,
        );
        context.validate()?;
        Ok(context)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_signature: Option<String>,
    pub integrity_digest: String,
}

/// A receipt that has passed machine-key, signature, freshness, result, and
/// binding verification. Raw `ExecutionReceipt` values remain diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedExecutionReceipt(ExecutionReceipt);

impl VerifiedExecutionReceipt {
    pub fn as_receipt(&self) -> &ExecutionReceipt {
        &self.0
    }
}

impl std::ops::Deref for VerifiedExecutionReceipt {
    type Target = ExecutionReceipt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
    let authority = context
        .as_ref()
        .map(|_| ReceiptAuthority::load_or_create())
        .transpose()?;
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
        schema_version: if context.is_some() {
            RECEIPT_SCHEMA_V2
        } else {
            RECEIPT_SCHEMA_V1
        },
        receipt_id: new_receipt_id()?,
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
        provenance: if context.is_some() {
            ReceiptProvenance::TrustedCurrentOperation
        } else {
            ReceiptProvenance::PersistedDiagnostic
        },
        authority_key_id: authority.as_ref().map(|value| value.key_id().to_string()),
        authority_signature: None,
        integrity_digest: String::new(),
    };
    if let Some(authority) = authority.as_ref() {
        receipt.authority_signature = Some(authority.sign(&canonical_receipt_bytes(&receipt)?));
    }
    receipt.integrity_digest = receipt_integrity(&receipt)?;
    append_receipt(&repo_root, &receipt)?;
    Ok(receipt)
}

pub fn load_receipts(repo_root: impl AsRef<Path>) -> Result<Vec<ExecutionReceipt>> {
    let path = repo_root.as_ref().join(RECEIPT_PATH);
    let Some(content) = read_text(&path)? else {
        return Ok(Vec::new());
    };
    let mut receipts = Vec::new();
    let mut receipt_ids = std::collections::BTreeSet::new();
    for (line_number, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let receipt: ExecutionReceipt = serde_json::from_str(line)
            .with_context(|| format!("Malformed execution receipt at line {}", line_number + 1))?;
        if !receipt_ids.insert(receipt.receipt_id.clone()) {
            bail!(
                "Duplicate execution receipt ID is not authoritative: {}",
                receipt.receipt_id
            );
        }
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

pub fn load_verified_receipt(
    repo_root: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<VerifiedExecutionReceipt> {
    let receipt = load_receipts(repo_root.as_ref())?
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id.trim())
        .with_context(|| format!("Trusted execution receipt not found: {}", receipt_id.trim()))?;
    verify_receipt_authority(repo_root, &receipt)
}

/// Loads only receipts that can become current authority. Historical or
/// invalid records remain available through [`load_receipts`] as diagnostics
/// and are skipped here rather than promoted.
pub fn load_verified_receipts(
    repo_root: impl AsRef<Path>,
) -> Result<Vec<VerifiedExecutionReceipt>> {
    let repo_root = repo_root.as_ref();
    let verified = load_receipts(repo_root)?
        .into_iter()
        .filter_map(|receipt| verify_receipt_authority(repo_root, &receipt).ok())
        .collect::<Vec<_>>();
    Ok(verified)
}

pub fn verify_receipt_authority(
    repo_root: impl AsRef<Path>,
    receipt: &ExecutionReceipt,
) -> Result<VerifiedExecutionReceipt> {
    if receipt.schema_version != RECEIPT_SCHEMA_V2 {
        bail!(
            "execution receipt schema {} is diagnostic-only; schema v2 is required for authority",
            receipt.schema_version
        );
    }
    let key_id = receipt
        .authority_key_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .context("authoritative receipt key ID is missing")?;
    let signature = receipt
        .authority_signature
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .context("authoritative receipt signature is missing")?;
    ReceiptAuthority::verify(key_id, signature, &canonical_receipt_bytes(receipt)?)?;
    if receipt_integrity(receipt)? != receipt.integrity_digest {
        bail!(
            "Execution receipt integrity check failed for {}",
            receipt.receipt_id
        );
    }
    let context = authoritative_context(receipt)?;
    context.validate()?;
    if !receipt_is_current(repo_root, receipt)? {
        bail!(
            "Execution receipt `{}` is stale, failed, or bound to another project",
            receipt.receipt_id
        );
    }
    Ok(VerifiedExecutionReceipt(receipt.clone()))
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
    Ok(verify_receipt_authority(repo_root, receipt).is_ok())
}

pub fn receipt_matches_context(
    repo_root: impl AsRef<Path>,
    receipt: &ExecutionReceipt,
    context: &ReceiptContext,
) -> Result<bool> {
    let verified = match verify_receipt_authority(repo_root, receipt) {
        Ok(verified) => verified,
        Err(_) => return Ok(false),
    };
    receipt_matches_verified_context(&verified, context)
}

pub fn receipt_matches_verified_context(
    receipt: &VerifiedExecutionReceipt,
    context: &ReceiptContext,
) -> Result<bool> {
    context.validate()?;
    Ok(receipt.task_id.as_deref() == Some(context.task_id.trim())
        && receipt.operation_id.as_deref() == Some(context.operation_id.trim())
        && receipt.adapter.as_deref() == Some(context.adapter.trim())
        && receipt.session_id.as_deref() == Some(context.session_id.trim())
        && receipt.request_id.as_deref() == Some(context.request_id.trim())
        && receipt.gate_kind.as_deref() == Some(context.gate_kind.trim()))
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
    let line = format!("{}\n", serde_json::to_string(receipt)?);
    let _lock = acquire_project_lock(repo_root)?;
    append_text(&path, &line)
        .with_context(|| format!("Could not append execution receipt: {}", path.display()))
}

fn authoritative_context(receipt: &ExecutionReceipt) -> Result<ReceiptContext> {
    Ok(ReceiptContext::new(
        receipt
            .task_id
            .clone()
            .context("authoritative receipt task ID is missing")?,
        receipt
            .operation_id
            .clone()
            .context("authoritative receipt operation ID is missing")?,
        receipt
            .adapter
            .clone()
            .context("authoritative receipt adapter is missing")?,
        receipt
            .session_id
            .clone()
            .context("authoritative receipt session ID is missing")?,
        receipt
            .request_id
            .clone()
            .context("authoritative receipt request ID is missing")?,
        receipt
            .gate_kind
            .clone()
            .context("authoritative receipt gate kind is missing")?,
    ))
}

#[derive(Serialize)]
struct ReceiptSigningPayload<'a> {
    schema_version: u32,
    receipt_id: &'a str,
    project_id: &'a str,
    source_fingerprint: &'a str,
    capability: &'a str,
    provider: &'a str,
    executable: &'a str,
    arguments: &'a [String],
    working_directory: &'a str,
    started_at: &'a str,
    finished_at: &'a str,
    exit_code: Option<i32>,
    result: ExecutionResult,
    stdout_digest: &'a str,
    stderr_digest: &'a str,
    stdout_excerpt: &'a str,
    stderr_excerpt: &'a str,
    artifact_digests: &'a [String],
    task_id: Option<&'a str>,
    operation_id: Option<&'a str>,
    adapter: Option<&'a str>,
    session_id: Option<&'a str>,
    request_id: Option<&'a str>,
    gate_kind: Option<&'a str>,
    provenance: ReceiptProvenance,
    authority_key_id: Option<&'a str>,
}

fn canonical_receipt_bytes(receipt: &ExecutionReceipt) -> Result<Vec<u8>> {
    let payload = ReceiptSigningPayload {
        schema_version: receipt.schema_version,
        receipt_id: &receipt.receipt_id,
        project_id: &receipt.project_id,
        source_fingerprint: &receipt.source_fingerprint,
        capability: &receipt.capability,
        provider: &receipt.provider,
        executable: &receipt.executable,
        arguments: &receipt.arguments,
        working_directory: &receipt.working_directory,
        started_at: &receipt.started_at,
        finished_at: &receipt.finished_at,
        exit_code: receipt.exit_code,
        result: receipt.result,
        stdout_digest: &receipt.stdout_digest,
        stderr_digest: &receipt.stderr_digest,
        stdout_excerpt: &receipt.stdout_excerpt,
        stderr_excerpt: &receipt.stderr_excerpt,
        artifact_digests: &receipt.artifact_digests,
        task_id: receipt.task_id.as_deref(),
        operation_id: receipt.operation_id.as_deref(),
        adapter: receipt.adapter.as_deref(),
        session_id: receipt.session_id.as_deref(),
        request_id: receipt.request_id.as_deref(),
        gate_kind: receipt.gate_kind.as_deref(),
        provenance: receipt.provenance,
        authority_key_id: receipt.authority_key_id.as_deref(),
    };
    let mut bytes = RECEIPT_SIGNATURE_DOMAIN.to_vec();
    bytes.extend(serde_json::to_vec(&payload)?);
    Ok(bytes)
}

fn new_receipt_id() -> Result<String> {
    let mut random = [0_u8; 16];
    getrandom::getrandom(&mut random)
        .map_err(|error| anyhow::anyhow!("Could not generate receipt ID: {error}"))?;
    Ok(format!("receipt-{}", hex_encode(random)))
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
    // the machine-key signature and exact identity binding.
    let mut value = receipt.clone();
    value.integrity_digest.clear();
    Ok(digest_bytes(&serde_json::to_vec(&value)?))
}

fn digest_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
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
