use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::autopilot::housekeep_candidates;
use crate::context::compile_context_for_operation;
use crate::continuity::{
    record_continuity_checkpoint, record_continuity_checkpoint_for_event,
    record_continuity_checkpoint_for_operation,
};
use crate::operation::{OperationContext, SupportedAdapter};
use crate::prepare::{
    prepare, task_id_for_request, PreparePacketV1, PrepareRequestV1, PREPARE_MAX_INPUT_BYTES,
};
use crate::proof::latest_proof;
use crate::safe_io::{acquire_project_lock, append_text, read_bytes, read_text, replace_text};
use crate::trace::latest_trace_score;
use crate::vault::VaultContext;

const HOOK_MAX_CONTEXT_CHARS: usize = 6_000;
const MAX_CHILD_EVIDENCE_CHARS: usize = 1_600;
const MAX_DEDUP_ENTRIES: usize = 256;
const MAX_JOURNAL_SCAN_LINES: usize = 512;
const DEDUP_SCHEMA_VERSION: u32 = 1;
const DEDUP_PATH: &str = ".baron/cache/automation-dedup.json";

static ACTIVE_HOOK_KEYS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationEvent {
    SessionStart,
    UserPromptSubmit,
    PreCompact,
    /// Compatibility spelling retained for older callers. New hook delivery
    /// uses [`AutomationEvent::UserPromptSubmit`].
    Prompt,
    /// Compatibility spelling retained for older callers. New hook delivery
    /// uses [`AutomationEvent::PreCompact`].
    Checkpoint,
    ContextCompiled,
    PlanStarted,
    HarnessStarted,
    ProofRecorded,
    TraceScored,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookAdapter {
    Codex,
    Claude,
    Neutral,
}

/// Stable identity for one logical lifecycle delivery. The serialized digest
/// is used only for bounded deduplication; the fields remain available in the
/// journal for diagnostics and provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleEventKey {
    pub project_id: String,
    pub adapter: String,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub event_kind: String,
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub stop_hook_active: bool,
}

impl LifecycleEventKey {
    pub fn stable_id(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("lifecycle event key is serializable");
        let digest = Sha256::digest(bytes);
        let suffix = digest
            .iter()
            .take(16)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        format!("lifecycle-{suffix}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JournalAdapter {
    Supported(HookAdapter),
    UnsupportedLegacy(String),
}

impl From<HookAdapter> for JournalAdapter {
    fn from(adapter: HookAdapter) -> Self {
        Self::Supported(adapter)
    }
}

impl Serialize for JournalAdapter {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Supported(HookAdapter::Codex) => "codex",
            Self::Supported(HookAdapter::Claude) => "claude",
            Self::Supported(HookAdapter::Neutral) => "neutral",
            Self::UnsupportedLegacy(value) => value,
        })
    }
}

impl<'de> Deserialize<'de> for JournalAdapter {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "codex" => Self::Supported(HookAdapter::Codex),
            "claude" => Self::Supported(HookAdapter::Claude),
            "neutral" => Self::Supported(HookAdapter::Neutral),
            _ => Self::UnsupportedLegacy(value),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub passed: bool,
    pub active_plan: bool,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JournalEntry {
    timestamp: String,
    event: AutomationEvent,
    adapter: JournalAdapter,
    session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    event_key: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    event_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    child_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_session_id: Option<String>,
    #[serde(default)]
    child: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DedupState {
    #[serde(default = "dedup_schema_version")]
    schema_version: u32,
    #[serde(default)]
    entries: Vec<DedupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DedupEntry {
    key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response: Option<String>,
}

impl Default for DedupState {
    fn default() -> Self {
        Self {
            schema_version: DEDUP_SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }
}

fn dedup_schema_version() -> u32 {
    DEDUP_SCHEMA_VERSION
}

struct ActiveHookGuard {
    key: String,
}

impl Drop for ActiveHookGuard {
    fn drop(&mut self) {
        if let Some(active) = ACTIVE_HOOK_KEYS.get() {
            active
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .remove(&self.key);
        }
    }
}

/// Handle one native lifecycle delivery. The hook path consumes structured
/// stdin, performs one Core operation, and returns a bounded host projection.
/// The project lock spans journal, continuity, and dedup writes so retries
/// cannot create competing state.
pub fn handle_hook(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    adapter: HookAdapter,
    event: AutomationEvent,
    payload_text: &str,
) -> Result<String> {
    let repo_root = repo_root.as_ref();
    if payload_text.len() > PREPARE_MAX_INPUT_BYTES {
        bail!(
            "native hook payload exceeds the {} byte limit",
            PREPARE_MAX_INPUT_BYTES
        );
    }
    let payload: Value = if payload_text.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(payload_text).context("Could not parse native hook payload")?
    };
    let session_id = payload_identifier(&payload, &["session_id", "sessionId"]);
    let request_id = payload_identifier(&payload, &["request_id", "requestId"]);
    let task = payload_task(&payload);
    let child_id = payload_identifier(&payload, &["child_id", "childId", "agent_id", "agentId"]);
    let parent_task_id = payload_identifier(&payload, &["parent_task_id", "parentTaskId"]);
    let parent_session_id = payload_identifier(&payload, &["parent_session_id", "parentSessionId"]);
    let is_child = payload_bool(&payload, &["is_child", "isChild"])
        || child_id.is_some()
        || parent_task_id.is_some()
        || parent_session_id.is_some();
    let child_evidence = payload_string(&payload, &["evidence", "result", "output"])
        .map(|value| bounded(&value, MAX_CHILD_EVIDENCE_CHARS).0);
    let stop_hook_active = if event == AutomationEvent::Stop {
        payload_bool(&payload, &["stop_hook_active", "stopHookActive"])
    } else {
        false
    };
    let operation = supported_operation(adapter, session_id.clone(), request_id.clone());
    let request = PrepareRequestV1 {
        schema_version: payload
            .get("schema_version")
            .or_else(|| payload.get("schemaVersion"))
            .and_then(Value::as_u64)
            .unwrap_or(1) as u32,
        task: task.clone(),
        session_id: session_id.clone(),
        request_id: request_id.clone(),
    };
    let task_id = task_id_for_request(&vault.project_id, &request);
    let event_kind = normalized_event_kind(event, stop_hook_active);
    let key = LifecycleEventKey {
        project_id: vault.project_id.clone(),
        adapter: adapter_name(adapter).to_string(),
        session_id: session_id.clone(),
        request_id: request_id.clone(),
        event_kind: event_kind.to_string(),
        task_id: task_id.clone(),
        child_id: child_id.clone(),
        stop_hook_active,
    };
    let event_key = key.stable_id();

    if recursion_depth(&payload) > 0 {
        return Ok(serde_json::to_string(&json!({
            "continue": true,
            "baron": {
                "project_id": vault.project_id,
                "adapter": adapter_name(adapter),
                "event": event_kind,
                "event_key": event_key,
                "task_id": task_id,
                "recursion_guard": true,
                "child": is_child
            }
        }))?);
    }

    let _lock = acquire_project_lock(repo_root)?;
    let active = ACTIVE_HOOK_KEYS.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut active = active.lock().unwrap_or_else(|poison| poison.into_inner());
        if !active.insert(event_key.clone()) {
            return Ok(serde_json::to_string(&json!({
                "continue": true,
                "baron": {
                    "project_id": vault.project_id,
                    "adapter": adapter_name(adapter),
                    "event": event_kind,
                    "event_key": event_key,
                    "task_id": task_id,
                    "recursion_guard": true,
                    "child": is_child
                }
            }))?);
        }
    }
    let _active_guard = ActiveHookGuard {
        key: event_key.clone(),
    };

    let mut dedup = load_dedup_state(vault)?;
    if let Some(response) = dedup_response(&dedup, &event_key) {
        return Ok(response);
    }

    let entry = JournalEntry {
        timestamp: now(),
        event,
        adapter: adapter.into(),
        session_id: session_id.clone(),
        request_id: request_id.clone(),
        event_key: Some(event_key.clone()),
        event_kind: event_kind.to_string(),
        task_id: Some(task_id.clone()),
        child_id: child_id.clone(),
        parent_task_id: parent_task_id.clone(),
        parent_session_id,
        child: is_child,
        evidence: child_evidence,
    };

    let response_value = if is_child {
        append_journal_locked(vault, &entry)?;
        json!({
            "continue": true,
            "baron": {
                "project_id": vault.project_id,
                "adapter": adapter_name(adapter),
                "event": event_kind,
                "event_key": event_key,
                "task_id": task_id,
                "child": true,
                "evidence_recorded": entry.evidence.is_some()
            }
        })
    } else {
        match event {
            AutomationEvent::SessionStart => {
                let operation = operation
                    .as_ref()
                    .context("adapter-neutral hooks cannot compile host-specific context")?;
                let context =
                    compile_context_for_operation(repo_root, &vault.vault_root, operation, None)?;
                let context = bounded_bytes(&context, HOOK_MAX_CONTEXT_CHARS);
                record_continuity_checkpoint_for_event(
                    repo_root,
                    vault,
                    "SessionStart hook observed.",
                    operation,
                    &event_key,
                )?;
                json!({
                    "continue": true,
                    "hookSpecificOutput": {
                        "hookEventName": "SessionStart",
                        "additionalContext": context
                    },
                    "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, false)
                })
            }
            AutomationEvent::UserPromptSubmit | AutomationEvent::Prompt => {
                let packet = prepare(
                    request,
                    adapter_name(adapter),
                    repo_root,
                    Some(vault.vault_root.clone()),
                )
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                let context = render_prepare_projection(&packet);
                record_continuity_checkpoint_for_event(
                    repo_root,
                    vault,
                    "UserPromptSubmit hook prepared the task.",
                    operation
                        .as_ref()
                        .context("adapter-neutral prompt hooks require an adapter")?,
                    &event_key,
                )?;
                json!({
                    "continue": true,
                    "hookSpecificOutput": {
                        "hookEventName": "UserPromptSubmit",
                        "additionalContext": context
                    },
                    "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, false)
                })
            }
            AutomationEvent::PreCompact | AutomationEvent::Checkpoint => {
                let operation = operation
                    .as_ref()
                    .context("adapter-neutral pre-compact hooks require an adapter")?;
                let note = format!("{} hook observed.", event_kind.replace('_', " "));
                record_continuity_checkpoint_for_event(
                    repo_root, vault, &note, operation, &event_key,
                )?;
                json!({
                    "continue": true,
                    "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, false)
                })
            }
            AutomationEvent::Stop => {
                let operation = operation
                    .as_ref()
                    .context("adapter-neutral stop hooks require an adapter")?;
                // Housekeeping is bounded metadata maintenance only. It may
                // expire or compact existing candidates, but Stop never
                // creates a proposal or promotes one.
                let _ = housekeep_candidates(repo_root, vault);
                let note = if stop_hook_active {
                    "Stop hook retry observed; preserve the active task state."
                } else {
                    "Stop hook observed; reconcile before allowing completion."
                };
                record_continuity_checkpoint_for_event(
                    repo_root, vault, note, operation, &event_key,
                )?;
                let report = reconcile(repo_root)?;
                let metadata =
                    hook_metadata(vault, adapter, event_kind, &event_key, &task_id, false);
                if !report.passed && !stop_hook_active {
                    json!({
                        "decision": "block",
                        "completed": false,
                        "reason": format!(
                            "Baron completion gate is not satisfied: {}. Record the missing evidence or interrupt the active plan before ending.",
                            report.gaps.join("; ")
                        ),
                        "baron": {
                            "project_id": metadata["project_id"],
                            "adapter": metadata["adapter"],
                            "event": metadata["event"],
                            "event_key": metadata["event_key"],
                            "task_id": metadata["task_id"],
                            "stop_is_completion": false,
                            "reconciliation_passed": false
                        }
                    })
                } else {
                    json!({
                        "continue": true,
                        "completed": false,
                        "systemMessage": if report.passed {
                            "Baron reconciliation passed; Stop does not mark completion."
                        } else {
                            "Baron reconciliation already requested once; avoid a hook loop and preserve the active state."
                        },
                        "baron": {
                            "project_id": metadata["project_id"],
                            "adapter": metadata["adapter"],
                            "event": metadata["event"],
                            "event_key": metadata["event_key"],
                            "task_id": metadata["task_id"],
                            "stop_is_completion": false,
                            "reconciliation_passed": report.passed
                        }
                    })
                }
            }
            _ => {
                if let Some(operation) = operation.as_ref() {
                    record_continuity_checkpoint_for_operation(
                        repo_root,
                        vault,
                        &format!("{} hook observed.", event_kind.replace('_', " ")),
                        operation,
                    )?;
                } else {
                    record_continuity_checkpoint(
                        repo_root,
                        vault,
                        &format!("{} hook observed.", event_kind.replace('_', " ")),
                        adapter_name(adapter),
                    )?;
                }
                json!({
                    "continue": true,
                    "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, false)
                })
            }
        }
    };
    append_journal_locked(vault, &entry)?;
    let response = serde_json::to_string(&response_value)?;
    dedup_store_response(&mut dedup, &event_key, &response);
    save_dedup_state(vault, &dedup)?;
    Ok(response)
}

pub fn record_lifecycle_event(
    vault: &VaultContext,
    adapter: HookAdapter,
    event: AutomationEvent,
) -> Result<()> {
    append_journal(
        vault,
        JournalEntry {
            timestamp: now(),
            event,
            adapter: adapter.into(),
            session_id: None,
            request_id: None,
            event_kind: normalized_event_kind(event, false).to_string(),
            event_key: None,
            task_id: None,
            child_id: None,
            parent_task_id: None,
            parent_session_id: None,
            child: false,
            evidence: None,
        },
    )
}

/// Record an adapter-neutral Baron Core event without selecting a project
/// adapter. The event is diagnostic evidence only and cannot authorize an
/// adapter-specific operation.
pub fn record_lifecycle_event_neutral(vault: &VaultContext, event: AutomationEvent) -> Result<()> {
    record_lifecycle_event(vault, HookAdapter::Neutral, event)
}

/// Record an adapter-attributed lifecycle event from an explicit operation.
/// This does not perform hook deduplication; that remains a later phase.
pub fn record_lifecycle_event_for_operation(
    vault: &VaultContext,
    operation: &OperationContext,
    event: AutomationEvent,
) -> Result<()> {
    append_journal(
        vault,
        JournalEntry {
            timestamp: now(),
            event,
            adapter: JournalAdapter::from(hook_adapter(operation.adapter)),
            session_id: operation.session_id.clone(),
            request_id: operation.request_id.clone(),
            event_kind: normalized_event_kind(event, false).to_string(),
            event_key: None,
            task_id: None,
            child_id: None,
            parent_task_id: None,
            parent_session_id: None,
            child: false,
            evidence: None,
        },
    )
}

pub fn reconcile(repo_root: impl AsRef<Path>) -> Result<ReconciliationReport> {
    let repo_root = repo_root.as_ref();
    let current_path = repo_root.join("docs/baron/plans/CURRENT.md");
    if !current_path.exists() {
        return Ok(ReconciliationReport {
            passed: true,
            active_plan: false,
            gaps: Vec::new(),
        });
    }
    let current = fs::read_to_string(&current_path)?;
    let active_plan = current.contains("- Status: `in_progress`")
        || current.contains("- Status: `interrupted`")
        || current.contains("- Status: `needs_correction`")
        || current.contains("- Status: `blocked`");
    if !active_plan {
        return Ok(ReconciliationReport {
            passed: true,
            active_plan: false,
            gaps: Vec::new(),
        });
    }

    let mut gaps = Vec::new();
    if latest_proof(repo_root)?.is_none() {
        gaps.push("verification proof is missing".to_string());
    }
    match latest_trace_score(repo_root)? {
        Some(score) if score.passed => {}
        Some(score) => gaps.push(format!(
            "trace quality failed ({}/{})",
            score.achieved.as_str(),
            score.required.as_str()
        )),
        None => gaps.push("a passing scored trace is missing".to_string()),
    }
    Ok(ReconciliationReport {
        passed: gaps.is_empty(),
        active_plan: true,
        gaps,
    })
}

pub fn automation_status(repo_root: impl AsRef<Path>, vault: &VaultContext) -> Result<String> {
    let journal_path = journal_path(vault);
    let journal = fs::read_to_string(&journal_path).unwrap_or_default();
    let event_count = journal
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let latest = journal
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<JournalEntry>(line).ok());
    let reconciliation = reconcile(repo_root)?;
    Ok(format!(
        "# Baron Automation Status\n\n\
- Journal: `{}`\n\
- Events recorded: {}\n\
- Latest event: `{}`\n\
- Reconciliation: `{}`\n\
- Gaps: {}\n",
        journal_path.display(),
        event_count,
        latest
            .map(|entry| {
                if entry.event_kind.is_empty() {
                    event_name(entry.event).to_string()
                } else {
                    entry.event_kind
                }
            })
            .unwrap_or_else(|| "none".to_string()),
        if reconciliation.passed {
            "passed"
        } else {
            "attention_required"
        },
        if reconciliation.gaps.is_empty() {
            "none".to_string()
        } else {
            reconciliation.gaps.join("; ")
        }
    ))
}

fn append_journal(vault: &VaultContext, entry: JournalEntry) -> Result<()> {
    let _lock = acquire_project_lock(&vault.repo_root)?;
    append_journal_locked(vault, &entry)
}

fn append_journal_locked(vault: &VaultContext, entry: &JournalEntry) -> Result<()> {
    let path = journal_path(vault);
    if let Some(event_key) = entry.event_key.as_deref() {
        if journal_contains_key(&path, event_key)? {
            return Ok(());
        }
    }
    let line = serde_json::to_string(entry)?;
    let separator = match read_bytes(&path)? {
        None => "",
        Some(bytes) if bytes.is_empty() || bytes.ends_with(b"\n") => "",
        Some(_) => "\n",
    };
    append_text(&path, &format!("{separator}{line}\n"))?;
    Ok(())
}

fn journal_path(vault: &VaultContext) -> std::path::PathBuf {
    vault
        .project_root
        .join("Artifacts/automation-journal.jsonl")
}

fn event_name(event: AutomationEvent) -> &'static str {
    match event {
        AutomationEvent::SessionStart => "session_start",
        AutomationEvent::UserPromptSubmit => "user_prompt_submit",
        AutomationEvent::PreCompact => "pre_compact",
        AutomationEvent::Prompt => "prompt",
        AutomationEvent::Checkpoint => "checkpoint",
        AutomationEvent::ContextCompiled => "context_compiled",
        AutomationEvent::PlanStarted => "plan_started",
        AutomationEvent::HarnessStarted => "harness_started",
        AutomationEvent::ProofRecorded => "proof_recorded",
        AutomationEvent::TraceScored => "trace_scored",
        AutomationEvent::Stop => "stop",
    }
}

fn normalized_event_kind(event: AutomationEvent, _stop_hook_active: bool) -> &'static str {
    match event {
        AutomationEvent::SessionStart => "session_start",
        AutomationEvent::UserPromptSubmit | AutomationEvent::Prompt => "user_prompt_submit",
        AutomationEvent::PreCompact | AutomationEvent::Checkpoint => "pre_compact",
        AutomationEvent::ContextCompiled => "context_compiled",
        AutomationEvent::PlanStarted => "plan_started",
        AutomationEvent::HarnessStarted => "harness_started",
        AutomationEvent::ProofRecorded => "proof_recorded",
        AutomationEvent::TraceScored => "trace_scored",
        AutomationEvent::Stop => "stop",
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn hook_metadata(
    vault: &VaultContext,
    adapter: HookAdapter,
    event_kind: &str,
    event_key: &str,
    task_id: &str,
    child: bool,
) -> Value {
    json!({
        "project_id": vault.project_id,
        "adapter": adapter_name(adapter),
        "event": event_kind,
        "event_key": event_key,
        "task_id": task_id,
        "child": child,
    })
}

fn recursion_depth(payload: &Value) -> u64 {
    payload
        .get("baron_recursion_depth")
        .or_else(|| payload.get("recursion_depth"))
        .or_else(|| payload.get("baronRecursionDepth"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn payload_task(payload: &Value) -> String {
    payload_string(
        payload,
        &[
            "task",
            "prompt",
            "user_prompt",
            "userPrompt",
            "text",
            "message",
        ],
    )
    .filter(|value| !value.trim().is_empty())
    .unwrap_or_else(|| "current repository state".to_string())
}

fn payload_identifier(payload: &Value, keys: &[&str]) -> Option<String> {
    payload_string(payload, keys).and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

fn payload_string(payload: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = payload.get(*key).and_then(Value::as_str) {
            return Some(value.to_string());
        }
    }
    payload
        .get("request")
        .filter(|value| value.is_object())
        .and_then(|request| payload_string(request, keys))
}

fn payload_bool(payload: &Value, keys: &[&str]) -> bool {
    keys.iter()
        .any(|key| payload.get(*key).and_then(Value::as_bool).unwrap_or(false))
        || payload
            .get("request")
            .filter(|value| value.is_object())
            .map(|request| payload_bool(request, keys))
            .unwrap_or(false)
}

fn load_dedup_state(vault: &VaultContext) -> Result<DedupState> {
    let path = vault.repo_root.join(DEDUP_PATH);
    let Some(content) = read_text(&path)? else {
        return Ok(DedupState::default());
    };
    let state: DedupState = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse hook dedup state: {}", path.display()))?;
    if state.schema_version != DEDUP_SCHEMA_VERSION {
        bail!(
            "Unsupported hook dedup schema {}; expected {}",
            state.schema_version,
            DEDUP_SCHEMA_VERSION
        );
    }
    Ok(state)
}

fn save_dedup_state(vault: &VaultContext, state: &DedupState) -> Result<()> {
    let path = vault.repo_root.join(DEDUP_PATH);
    let content = format!("{}\n", serde_json::to_string_pretty(state)?);
    replace_text(path, &content)
}

fn dedup_response(state: &DedupState, key: &str) -> Option<String> {
    state
        .entries
        .iter()
        .rev()
        .find(|entry| entry.key == key)
        .and_then(|entry| entry.response.clone())
}

fn dedup_store_response(state: &mut DedupState, key: &str, response: &str) {
    state.entries.retain(|entry| entry.key != key);
    state.entries.push(DedupEntry {
        key: key.to_string(),
        response: Some(response.to_string()),
    });
    let excess = state.entries.len().saturating_sub(MAX_DEDUP_ENTRIES);
    if excess > 0 {
        state.entries.drain(0..excess);
    }
}

fn journal_contains_key(path: &Path, key: &str) -> Result<bool> {
    let Some(content) = read_text(path)? else {
        return Ok(false);
    };
    Ok(content
        .lines()
        .rev()
        .take(MAX_JOURNAL_SCAN_LINES)
        .filter_map(|line| serde_json::from_str::<JournalEntry>(line).ok())
        .any(|entry| entry.event_key.as_deref() == Some(key)))
}

fn render_prepare_projection(packet: &PreparePacketV1) -> String {
    let mut output = String::new();
    output.push_str("# Baron Prepare Context\n\n");
    output.push_str(&format!("- Project: `{}`\n", packet.project_id));
    output.push_str(&format!("- Adapter: `{}`\n", packet.adapter));
    output.push_str(&format!("- Task ID: `{}`\n", packet.task.id));
    output.push_str(&format!("- Task: {}\n", single_line(&packet.task.summary)));
    output.push_str(&format!(
        "- Work shape: `{}`; risk: `{}`; lifecycle: `{}`\n",
        packet.work_shape.work_shape,
        packet.risk.as_str(),
        packet.work_shape.lifecycle
    ));
    output.push_str(&format!(
        "- Route: {}\n",
        single_line(&packet.route.explanation)
    ));
    output.push_str("- Selected skills: ");
    output.push_str(
        &packet
            .route
            .selected_skills
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    output.push('\n');
    output.push_str("- Mandatory agents: ");
    output.push_str(
        &packet
            .route
            .mandatory_agents
            .iter()
            .map(|item| item.path.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    output.push('\n');
    if !packet.intent.constraints.is_empty() {
        output.push_str(&format!(
            "- Constraints: {}\n",
            packet.intent.constraints.join("; ")
        ));
    }
    output.push_str(&format!(
        "- Verification: proof_required=`{}`; gate_evidence_passed=`{}`\n",
        packet.verification.proof_required, packet.verification.gate_evidence_passed
    ));
    if !packet.blockers.is_empty() {
        output.push_str("- Blockers: ");
        output.push_str(
            &packet
                .blockers
                .iter()
                .map(|issue| issue.message.as_str())
                .collect::<Vec<_>>()
                .join("; "),
        );
        output.push('\n');
    }
    output.push_str(&format!("- Next action: {}\n", packet.next_action.action));
    output.push_str("\n## Bounded Core Context\n\n");
    output.push_str(&bounded(&packet.context.text, 2_700).0);
    bounded_bytes(&output, HOOK_MAX_CONTEXT_CHARS)
}

fn bounded(value: &str, limit: usize) -> (String, bool) {
    let output = value.chars().take(limit).collect::<String>();
    (output, value.chars().count() > limit)
}

fn bounded_bytes(value: &str, limit: usize) -> String {
    let mut end = 0;
    for (index, character) in value.char_indices() {
        let next = index + character.len_utf8();
        if next > limit {
            break;
        }
        end = next;
    }
    value[..end].to_string()
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn adapter_name(adapter: HookAdapter) -> &'static str {
    match adapter {
        HookAdapter::Codex => "codex",
        HookAdapter::Claude => "claude",
        HookAdapter::Neutral => "neutral",
    }
}

fn hook_adapter(adapter: SupportedAdapter) -> HookAdapter {
    match adapter {
        SupportedAdapter::Codex => HookAdapter::Codex,
        SupportedAdapter::Claude => HookAdapter::Claude,
    }
}

fn supported_operation(
    adapter: HookAdapter,
    session_id: Option<String>,
    request_id: Option<String>,
) -> Option<OperationContext> {
    let adapter = match adapter {
        HookAdapter::Codex => SupportedAdapter::Codex,
        HookAdapter::Claude => SupportedAdapter::Claude,
        HookAdapter::Neutral => return None,
    };
    Some(OperationContext {
        adapter,
        session_id,
        request_id,
    })
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
