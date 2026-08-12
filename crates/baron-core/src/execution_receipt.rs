use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::identity::project_id_for_path;

const RECEIPT_PATH: &str = ".baron/cache/execution-receipts.jsonl";
const MAX_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_ARG_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionResult {
    Passed,
    Failed,
    TimedOut,
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
            digest_hex(format!("{project_id}:{started_at}"), 12)
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
        integrity_digest: String::new(),
    };
    receipt.integrity_digest = receipt_integrity(&receipt)?;
    append_receipt(&repo_root, &receipt)?;
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
        digest.update(
            metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|value| value.as_nanos().to_le_bytes().to_vec())
                .unwrap_or_default(),
        );
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
