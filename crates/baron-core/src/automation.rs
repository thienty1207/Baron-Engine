use std::collections::{BTreeSet, HashSet};
use std::fmt;
#[cfg(test)]
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Take};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, SecondsFormat};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::autopilot::housekeep_candidates;
use crate::context::compile_context_for_lifecycle_identity;
use crate::continuity::{
    record_continuity_checkpoint, record_continuity_checkpoint_for_event,
    record_continuity_checkpoint_for_operation,
};
use crate::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use crate::plan::{
    active_plan_completion_evidence_status, active_plan_completion_evidence_status_for_identity,
};
use crate::prepare::{prepare, PreparePacketV1, PrepareRequestV1, PREPARE_MAX_INPUT_BYTES};
use crate::safe_io::{
    acquire_project_lock, append_text, artifact_instance_id, open_regular_file_for_read, read_text,
    replace_text,
};
use crate::vault::VaultContext;

mod hook_correlation;

const HOOK_MAX_CONTEXT_CHARS: usize = 6_000;
const MAX_CHILD_EVIDENCE_CHARS: usize = 1_600;
const MAX_DEDUP_ENTRIES: usize = 256;
const MAX_JOURNAL_SCAN_LINES: usize = 512;
const MAX_JOURNAL_SCAN_BYTES: usize = 4 * 1024 * 1024;
const JOURNAL_READ_CHUNK_BYTES: usize = 8 * 1024;
const MAX_DIAGNOSTIC_JOURNAL_ROW_BYTES: usize = 1024 * 1024;
const MAX_DIAGNOSTIC_EVENT_KIND_BYTES: usize = 256;
const MAX_DIAGNOSTIC_EVENT_KINDS: usize = 64;
const DEDUP_SCHEMA_VERSION: u32 = 1;
const DEDUP_CLAIM_TTL_SECONDS: i64 = 300;
const DEDUP_PATH: &str = ".baron/cache/automation-dedup.json";

static ACTIVE_HOOK_KEYS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[cfg(test)]
type StopPublicationHook = Box<dyn FnOnce() + Send>;

#[cfg(test)]
type StopPublicationPause = (std::path::PathBuf, StopPublicationHook);

#[cfg(test)]
static STOP_BEFORE_PUBLICATION: OnceLock<Mutex<Option<StopPublicationPause>>> = OnceLock::new();

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

/// Explicit disposition for an error crossing the native hook boundary.
/// Untyped errors are treated as fail-closed by adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookFailureKind {
    Advisory,
    FailClosed,
}

impl fmt::Display for HookFailureKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Advisory => "advisory native hook failure",
            Self::FailClosed => "fail-closed native hook failure",
        })
    }
}

impl std::error::Error for HookFailureKind {}

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
    /// Persist the bounded host response with the journal event so a retry
    /// can recover it if the process dies before dedup-state replacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JournalPresence {
    Missing,
    Empty,
    Present,
}

impl JournalPresence {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Empty => "empty",
            Self::Present => "present",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JournalDiagnostics {
    pub(crate) presence: JournalPresence,
    pub(crate) event_count: u64,
    pub(crate) latest_event: Option<String>,
    pub(crate) observed_event_kinds: Vec<String>,
}

struct JournalSnapshot {
    reader: BufReader<Take<File>>,
    path: PathBuf,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claimed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claim_token: Option<String>,
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
/// cannot create competing state when the host supplies the same session and
/// request identity. Native Codex turn_id and Claude prompt_id are correlated
/// durably to the original canonical task before response deduplication.
/// Prompt submissions without those stable host IDs or an explicit Baron
/// request_id fail closed before any durable operation or delivery state is
/// created; session and prompt text alone cannot distinguish retries from a
/// new turn.
pub fn handle_hook(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    adapter: HookAdapter,
    event: AutomationEvent,
    payload_text: &str,
) -> Result<String> {
    handle_hook_inner(repo_root, vault, adapter, event, payload_text).map_err(|error| {
        if error.downcast_ref::<HookFailureKind>().is_some() {
            error
        } else {
            error.context(HookFailureKind::FailClosed)
        }
    })
}

fn handle_hook_inner(
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
        serde_json::from_str(payload_text)
            .context("Could not parse native hook payload")
            .map_err(|error| {
                if matches!(
                    event,
                    AutomationEvent::UserPromptSubmit | AutomationEvent::Prompt
                ) {
                    error.context(HookFailureKind::Advisory)
                } else {
                    error
                }
            })?
    };
    let ingress = match hook_correlation::Ingress::parse(&payload, adapter) {
        Ok(ingress) => ingress,
        Err(error) if event == AutomationEvent::Stop => {
            return hook_correlation::blocked(&error, adapter, false);
        }
        Err(error) => bail!("{}", hook_correlation::failure(&error)),
    };
    // Validate durable replay evidence before correlation, continuity, or
    // delivery-state publication can mutate the checkout or Vault.
    {
        let _lock = acquire_project_lock(repo_root)?;
        let _vault_lock = acquire_project_lock(&vault.project_root)?;
        let _ = journal_tail_entries(&journal_path(vault))?;
    }
    let supplied_session_id = ingress.session.clone();
    let supplied_request_id = ingress.turn.clone();
    let mut task = ingress
        .task
        .clone()
        .unwrap_or_else(|| "current repository state".into());
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
    let supported_adapter = match adapter {
        HookAdapter::Codex => Some(SupportedAdapter::Codex),
        HookAdapter::Claude => Some(SupportedAdapter::Claude),
        HookAdapter::Neutral => None,
    };
    let correlated = !is_child
        && recursion_depth(&payload) == 0
        && matches!(
            event,
            AutomationEvent::UserPromptSubmit
                | AutomationEvent::Prompt
                | AutomationEvent::PreCompact
                | AutomationEvent::Checkpoint
                | AutomationEvent::Stop
        );
    let mut stop_completed = false;
    let identity = if let Some(supported) = supported_adapter.filter(|_| correlated) {
        let resolved = (|| -> Result<_> {
            let _lock = acquire_project_lock(repo_root)?;
            // A corrupt response cache must not permit a new mapping write.
            let _ = load_dedup_state(vault)?;
            hook_correlation::resolve_locked(repo_root, vault, supported, event, &ingress)
        })();
        match resolved {
            Ok((canonical_task, identity, completed)) => {
                task = canonical_task;
                stop_completed = event == AutomationEvent::Stop && completed;
                Some(identity)
            }
            Err(error) if event == AutomationEvent::Stop => {
                return hook_correlation::blocked(&error, adapter, false);
            }
            Err(error) => bail!("{}", hook_correlation::failure(&error)),
        }
    } else {
        supported_adapter
            .map(|adapter| {
                LifecycleIdentity::resolve(
                    &vault.project_id,
                    &task,
                    adapter,
                    supplied_session_id.as_deref(),
                    supplied_request_id.as_deref(),
                )
            })
            .transpose()?
    };
    let (session_id, request_id, task_id, operation) = if let Some(identity) = identity.as_ref() {
        (
            Some(identity.session_id().to_string()),
            Some(identity.request_id().to_string()),
            identity.task_id().to_string(),
            Some(OperationContext::from_identity(identity)),
        )
    } else {
        (
            supplied_session_id,
            supplied_request_id,
            crate::operation::task_id_for_task(&vault.project_id, &task)?,
            None,
        )
    };
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

    if stop_completed {
        return completed_stop_response(
            vault,
            adapter,
            event_kind,
            &event_key,
            &task_id,
            identity
                .as_ref()
                .context("completed Stop requires identity")?,
            is_child,
        );
    }

    if recursion_depth(&payload) > 0 {
        let mut metadata = hook_metadata(
            vault,
            adapter,
            event_kind,
            &event_key,
            &task_id,
            identity.as_ref(),
            is_child,
        );
        metadata["recursion_guard"] = json!(true);
        return Ok(serde_json::to_string(&json!({
            "continue": true,
            "baron": metadata
        }))?);
    }

    let active = ACTIVE_HOOK_KEYS.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut active = active.lock().unwrap_or_else(|poison| poison.into_inner());
        if !active.insert(event_key.clone()) {
            let mut metadata = hook_metadata(
                vault,
                adapter,
                event_kind,
                &event_key,
                &task_id,
                identity.as_ref(),
                is_child,
            );
            metadata["recursion_guard"] = json!(true);
            if event == AutomationEvent::Stop {
                metadata["reconciliation_pending"] = json!(true);
                return Ok(serde_json::to_string(&json!({
                    "decision": "block",
                    "completed": false,
                    "reason": "Baron is already processing this Stop delivery; retry after the active delivery publishes its reconciliation result.",
                    "baron": metadata
                }))?);
            }
            return Ok(serde_json::to_string(&json!({
                "continue": true,
                "baron": metadata
            }))?);
        }
    }
    let _active_guard = ActiveHookGuard {
        key: event_key.clone(),
    };
    // An earlier Stop response only describes evidence as of that delivery.
    // Retries for the same host event must reconcile again because operation
    // evidence can change while the operation remains active.
    let refresh_stop_response = event == AutomationEvent::Stop && !is_child;

    let claim_token = {
        let _lock = acquire_project_lock(repo_root)?;
        if event == AutomationEvent::Stop && !is_child {
            let completed = hook_correlation::verify_stop_locked(
                repo_root,
                vault,
                supported_adapter.context("Stop requires a supported adapter")?,
                &ingress,
                identity.as_ref().context("Stop requires identity")?,
            )
            .map_err(|error| anyhow::anyhow!(hook_correlation::failure(&error)))?;
            if completed {
                return completed_stop_response(
                    vault,
                    adapter,
                    event_kind,
                    &event_key,
                    &task_id,
                    identity
                        .as_ref()
                        .context("completed Stop requires identity")?,
                    is_child,
                );
            }
        }
        let mut dedup = load_dedup_state(vault)?;
        if !refresh_stop_response {
            if let Some(response) = dedup_response(&dedup, &event_key) {
                return Ok(response);
            }
            // The response journal is shared by all checkouts attached to
            // this project capsule. Observe it only under checkout -> Vault
            // lock order; publication repeats the lookup under the same lock.
            let journal_response = {
                let _vault_lock = acquire_project_lock(&vault.project_root)?;
                journal_response(&journal_path(vault), &event_key)?
            };
            if let Some(response) = journal_response {
                dedup_store_response(&mut dedup, &event_key, &response)?;
                save_dedup_state(vault, &dedup)?;
                return Ok(response);
            }
        }
        if dedup_claim_is_live(&dedup, &event_key) {
            let mut metadata = hook_metadata(
                vault,
                adapter,
                event_kind,
                &event_key,
                &task_id,
                identity.as_ref(),
                is_child,
            );
            metadata["deduplicated"] = json!(true);
            if event == AutomationEvent::Stop {
                metadata["reconciliation_pending"] = json!(true);
                return Ok(serde_json::to_string(&json!({
                    "decision": "block",
                    "completed": false,
                    "reason": "Baron is still processing this Stop delivery; retry after the active delivery publishes its reconciliation result.",
                    "baron": metadata
                }))?);
            }
            return Ok(serde_json::to_string(&json!({
                "continue": true,
                "baron": metadata
            }))?);
        }
        let claim_token = dedup_claim(&mut dedup, &event_key)?;
        save_dedup_state(vault, &dedup)?;
        claim_token
    };

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
        response: None,
    };

    let response_value = match (|| -> Result<Value> {
        Ok(if is_child {
            let mut metadata = hook_metadata(
                vault,
                adapter,
                event_kind,
                &event_key,
                &task_id,
                identity.as_ref(),
                true,
            );
            metadata["evidence_recorded"] = json!(entry.evidence.is_some());
            json!({
                "continue": true,
                "baron": metadata
            })
        } else {
            match event {
                AutomationEvent::SessionStart => {
                    let operation = operation
                        .as_ref()
                        .context("adapter-neutral hooks cannot compile host-specific context")?;
                    let context_identity = identity
                        .as_ref()
                        .context("host-specific hooks require a complete lifecycle identity")?;
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
                    let context = compile_context_for_lifecycle_identity(
                        repo_root,
                        &vault.vault_root,
                        context_identity,
                        Some(&task),
                    )?;
                    let context = bounded_bytes(&context, HOOK_MAX_CONTEXT_CHARS);
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
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
                        "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, identity.as_ref(), false)
                    })
                }
                AutomationEvent::UserPromptSubmit | AutomationEvent::Prompt => {
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
                    let packet = prepare(
                        request,
                        adapter_name(adapter),
                        repo_root,
                        Some(vault.vault_root.clone()),
                    )
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                    let context = render_prepare_projection(&packet);
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
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
                        "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, identity.as_ref(), false)
                    })
                }
                AutomationEvent::PreCompact | AutomationEvent::Checkpoint => {
                    let operation = operation
                        .as_ref()
                        .context("adapter-neutral pre-compact hooks require an adapter")?;
                    let note = format!("{} hook observed.", event_kind.replace('_', " "));
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
                    record_continuity_checkpoint_for_event(
                        repo_root, vault, &note, operation, &event_key,
                    )?;
                    json!({
                        "continue": true,
                        "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, identity.as_ref(), false)
                    })
                }
                AutomationEvent::Stop => {
                    let operation = operation
                        .as_ref()
                        .context("adapter-neutral stop hooks require an adapter")?;
                    // Housekeeping is bounded metadata maintenance only. It may
                    // expire or compact existing candidates, but Stop never
                    // creates a proposal or promotes one.
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
                    let _ = housekeep_candidates(repo_root, vault);
                    let note = if stop_hook_active {
                        "Stop hook retry observed; preserve the active task state."
                    } else {
                        "Stop hook observed; reconcile before allowing completion."
                    };
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
                    record_continuity_checkpoint_for_event(
                        repo_root, vault, note, operation, &event_key,
                    )?;
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
                    let lifecycle_identity = identity
                        .as_ref()
                        .context("identified Stop hooks require a complete lifecycle identity")?;
                    let report = reconcile_for_operation(repo_root, vault, lifecycle_identity)?;
                    let response_context = StopResponseContext {
                        vault,
                        adapter,
                        event_kind,
                        event_key: &event_key,
                        task_id: &task_id,
                        identity: lifecycle_identity,
                        child: false,
                    };
                    stop_reconciliation_response(&response_context, &report)
                }
                _ => {
                    renew_dedup_claim(repo_root, vault, &event_key, &claim_token)?;
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
                        "baron": hook_metadata(vault, adapter, event_kind, &event_key, &task_id, identity.as_ref(), false)
                    })
                }
            }
        })
    })() {
        Ok(response) => response,
        Err(error) => {
            release_dedup_claim(repo_root, vault, &event_key, &claim_token);
            return Err(error);
        }
    };
    let response = match serde_json::to_string(&response_value) {
        Ok(response) => response,
        Err(error) => {
            release_dedup_claim(repo_root, vault, &event_key, &claim_token);
            return Err(error.into());
        }
    };
    #[cfg(test)]
    if event == AutomationEvent::Stop {
        pause_stop_before_publication(repo_root);
    }
    // Revalidate correlation and ACTIVE under the same lock as publication.
    // Prepare/context/probes and the first reconciliation stay outside; the
    // final exact-operation evidence check runs with authority publication.
    let publication = (|| -> Result<String> {
        let _lock = acquire_project_lock(repo_root)?;
        let response = if event == AutomationEvent::Stop && !is_child {
            let response_context = StopResponseContext {
                vault,
                adapter,
                event_kind,
                event_key: &event_key,
                task_id: &task_id,
                identity: identity.as_ref().context("Stop requires identity")?,
                child: is_child,
            };
            finalize_stop_response_locked(
                repo_root,
                supported_adapter.context("Stop requires a supported adapter")?,
                &ingress,
                &response_context,
            )?
        } else {
            response.clone()
        };
        publish_hook_response(
            repo_root,
            vault,
            &entry,
            &event_key,
            &claim_token,
            &response,
            refresh_stop_response,
        )
    })();
    match publication {
        Ok(response) => Ok(response),
        Err(error) => {
            release_dedup_claim(repo_root, vault, &event_key, &claim_token);
            Err(error)
        }
    }
}

#[cfg(test)]
fn pause_stop_before_publication(repo_root: &Path) {
    let slot = STOP_BEFORE_PUBLICATION.get_or_init(|| Mutex::new(None));
    let callback = {
        let mut slot = slot.lock().unwrap_or_else(|poison| poison.into_inner());
        if slot
            .as_ref()
            .is_some_and(|(expected_repo, _)| expected_repo == repo_root)
        {
            slot.take().map(|(_, callback)| callback)
        } else {
            None
        }
    };
    if let Some(callback) = callback {
        callback();
    }
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
            response: None,
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
            response: None,
        },
    )
}

pub fn reconcile(repo_root: impl AsRef<Path>) -> Result<ReconciliationReport> {
    let Some(status) = active_plan_completion_evidence_status(repo_root)? else {
        return Ok(ReconciliationReport {
            passed: true,
            active_plan: false,
            gaps: Vec::new(),
        });
    };
    Ok(ReconciliationReport {
        passed: status.passed,
        active_plan: true,
        gaps: status.issues,
    })
}

/// Reconcile exactly the operation that delivered an identified Stop event.
/// This path never falls back to the latest CURRENT projection.
pub fn reconcile_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    identity: &LifecycleIdentity,
) -> Result<ReconciliationReport> {
    let Some(status) =
        active_plan_completion_evidence_status_for_identity(repo_root, vault, identity)?
    else {
        return Ok(ReconciliationReport {
            passed: false,
            active_plan: true,
            gaps: vec!["active plan for the Stop operation is missing".to_string()],
        });
    };
    Ok(ReconciliationReport {
        passed: status.passed,
        active_plan: true,
        gaps: status.issues,
    })
}

struct StopResponseContext<'a> {
    vault: &'a VaultContext,
    adapter: HookAdapter,
    event_kind: &'a str,
    event_key: &'a str,
    task_id: &'a str,
    identity: &'a LifecycleIdentity,
    child: bool,
}

fn stop_reconciliation_response(
    context: &StopResponseContext<'_>,
    report: &ReconciliationReport,
) -> Value {
    let metadata = hook_metadata(
        context.vault,
        context.adapter,
        context.event_kind,
        context.event_key,
        context.task_id,
        Some(context.identity),
        context.child,
    );
    // A host retry flag is not identity, reconciliation, or completion evidence.
    if !report.passed {
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
                "operation_id": metadata["operation_id"],
                "session_id": metadata["session_id"],
                "request_id": metadata["request_id"],
                "stop_is_completion": false,
                "reconciliation_passed": false
            }
        })
    } else {
        json!({
            "continue": true,
            "completed": false,
            "systemMessage": "Baron reconciliation passed; Stop does not mark completion.",
            "baron": {
                "project_id": metadata["project_id"],
                "adapter": metadata["adapter"],
                "event": metadata["event"],
                "event_key": metadata["event_key"],
                "task_id": metadata["task_id"],
                "operation_id": metadata["operation_id"],
                "session_id": metadata["session_id"],
                "request_id": metadata["request_id"],
                "stop_is_completion": false,
                "reconciliation_passed": report.passed
            }
        })
    }
}

/// Revalidate identity and the exact operation evidence while holding the
/// publication lock. Evidence can change after the initial, unlocked
/// reconciliation; a stale passing response must never be published.
fn finalize_stop_response_locked(
    repo_root: &Path,
    supported_adapter: SupportedAdapter,
    ingress: &hook_correlation::Ingress,
    context: &StopResponseContext<'_>,
) -> Result<String> {
    let completed = hook_correlation::verify_stop_locked(
        repo_root,
        context.vault,
        supported_adapter,
        ingress,
        context.identity,
    )
    .map_err(|error| anyhow::anyhow!(hook_correlation::failure(&error)))?;
    if completed {
        return completed_stop_response(
            context.vault,
            context.adapter,
            context.event_kind,
            context.event_key,
            context.task_id,
            context.identity,
            context.child,
        );
    }

    let current_report = reconcile_for_operation(repo_root, context.vault, context.identity)?;
    serde_json::to_string(&stop_reconciliation_response(context, &current_report))
        .map_err(Into::into)
}

pub fn automation_status(repo_root: impl AsRef<Path>, vault: &VaultContext) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let journal_path = journal_path(vault);
    let journal = scan_automation_journal(repo_root, vault)?;
    let reconciliation = reconcile(repo_root)?;
    Ok(format!(
        "# Baron Automation Status\n\n\
- Journal: `{}`\n\
- Journal state: `{}`\n\
- Events recorded: {}\n\
- Latest event: `{}`\n\
- Reconciliation: `{}`\n\
- Gaps: {}\n",
        journal_path.display(),
        journal.presence.as_str(),
        journal.event_count,
        journal.latest_event.unwrap_or_else(|| "none".to_string()),
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

pub(crate) fn scan_automation_journal(
    repo_root: &Path,
    vault: &VaultContext,
) -> Result<JournalDiagnostics> {
    match open_automation_journal_snapshot(repo_root, vault)? {
        Some(snapshot) => scan_journal_snapshot(snapshot),
        None => Ok(JournalDiagnostics {
            presence: JournalPresence::Missing,
            event_count: 0,
            latest_event: None,
            observed_event_kinds: Vec::new(),
        }),
    }
}

fn open_automation_journal_snapshot(
    repo_root: &Path,
    vault: &VaultContext,
) -> Result<Option<JournalSnapshot>> {
    let _repo_lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let path = journal_path(vault);
    let Some(file) = open_regular_file_for_read(&path)? else {
        return Ok(None);
    };
    let length = file
        .metadata()
        .with_context(|| format!("Could not inspect automation journal: {}", path.display()))?
        .len();
    Ok(Some(JournalSnapshot {
        reader: BufReader::with_capacity(JOURNAL_READ_CHUNK_BYTES, file.take(length)),
        path,
    }))
}

fn scan_journal_snapshot(snapshot: JournalSnapshot) -> Result<JournalDiagnostics> {
    scan_journal_reader(snapshot.reader, &snapshot.path)
}

fn scan_journal_reader<R: BufRead>(mut reader: R, path: &Path) -> Result<JournalDiagnostics> {
    let mut offset = 0u64;
    let mut event_count = 0u64;
    let mut latest_event = None;
    let mut observed_event_kinds = BTreeSet::new();
    loop {
        let row_offset = offset;
        let Some((mut row, consumed)) = read_bounded_journal_row(&mut reader, path, offset)? else {
            break;
        };
        offset = offset.saturating_add(consumed as u64);
        if row.last() == Some(&b'\r') {
            row.pop();
        }
        if row.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let line = std::str::from_utf8(&row).with_context(|| {
            format!(
                "Automation journal row at byte {row_offset} is not valid UTF-8: {}",
                path.display()
            )
        })?;
        let entry = serde_json::from_str::<JournalEntry>(line).with_context(|| {
            format!(
                "Malformed automation journal JSONL row at byte {row_offset}: {}",
                path.display()
            )
        })?;
        let event_kind = if entry.event_kind.is_empty() {
            event_name(entry.event).to_string()
        } else {
            entry.event_kind
        };
        if event_kind.len() > MAX_DIAGNOSTIC_EVENT_KIND_BYTES {
            bail!(
                "Automation journal event kind exceeds the {MAX_DIAGNOSTIC_EVENT_KIND_BYTES} byte diagnostic limit at byte {row_offset}: {}",
                path.display()
            );
        }
        latest_event = Some(event_kind.clone());
        observed_event_kinds.insert(event_kind);
        if observed_event_kinds.len() > MAX_DIAGNOSTIC_EVENT_KINDS {
            bail!(
                "Automation journal exceeds the {MAX_DIAGNOSTIC_EVENT_KINDS} unique event-kind diagnostic limit: {}",
                path.display()
            );
        }
        event_count = event_count
            .checked_add(1)
            .context("Automation journal event count overflowed")?;
    }
    Ok(JournalDiagnostics {
        presence: if event_count == 0 {
            JournalPresence::Empty
        } else {
            JournalPresence::Present
        },
        event_count,
        latest_event,
        observed_event_kinds: observed_event_kinds.into_iter().collect(),
    })
}

fn read_bounded_journal_row<R: BufRead>(
    reader: &mut R,
    path: &Path,
    offset: u64,
) -> Result<Option<(Vec<u8>, usize)>> {
    let mut row = Vec::new();
    let mut consumed = 0usize;
    loop {
        let available = reader
            .fill_buf()
            .with_context(|| format!("Could not read automation journal: {}", path.display()))?;
        if available.is_empty() {
            return if row.is_empty() {
                Ok(None)
            } else {
                Ok(Some((row, consumed)))
            };
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let row_bytes = newline.unwrap_or(available.len());
        if row.len().saturating_add(row_bytes) > MAX_DIAGNOSTIC_JOURNAL_ROW_BYTES {
            bail!(
                "Automation journal row exceeds the {MAX_DIAGNOSTIC_JOURNAL_ROW_BYTES} byte diagnostic limit at byte {offset}: {}",
                path.display()
            );
        }
        row.extend_from_slice(&available[..row_bytes]);
        let used = row_bytes + usize::from(newline.is_some());
        reader.consume(used);
        consumed = consumed.saturating_add(used);
        if newline.is_some() {
            return Ok(Some((row, consumed)));
        }
    }
}

fn append_journal(vault: &VaultContext, entry: JournalEntry) -> Result<()> {
    let _lock = acquire_project_lock(&vault.repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
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
    let separator = if journal_needs_separator(&path)? {
        "\n"
    } else {
        ""
    };
    append_text(&path, &format!("{separator}{line}\n"))?;
    Ok(())
}

fn publish_hook_response(
    repo_root: &Path,
    vault: &VaultContext,
    entry: &JournalEntry,
    event_key: &str,
    claim_token: &str,
    response: &str,
    refresh_stop_response: bool,
) -> Result<String> {
    let _lock = acquire_project_lock(repo_root)?;
    // Different worktrees have different checkout locks but may share this
    // durable capsule journal. Hold its lock across dedup read, append, and
    // crash-recovery publication, preserving checkout -> Vault ordering.
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let mut dedup = load_dedup_state(vault)?;
    if !refresh_stop_response {
        if let Some(existing) = dedup_response(&dedup, event_key) {
            return Ok(existing);
        }
        if let Some(existing) = journal_response(&journal_path(vault), event_key)? {
            dedup_store_response(&mut dedup, event_key, &existing)?;
            save_dedup_state(vault, &dedup)?;
            return Ok(existing);
        }
    }
    if !dedup_claim_matches(&dedup, event_key, claim_token) {
        bail!(
            "Hook claim is no longer owned by this delivery; refusing stale publication: {}",
            event_key
        );
    }
    renew_dedup_claim_from_state(&mut dedup, event_key, claim_token)?;
    let mut published_entry = entry.clone();
    published_entry.response = Some(response.to_string());
    // The journal's first Stop response is historical only. Stop retries do
    // not recover it; the fenced dedup cache is refreshed after reconciliation.
    append_journal_locked(vault, &published_entry)?;
    dedup_store_response(&mut dedup, event_key, response)?;
    save_dedup_state(vault, &dedup)?;
    Ok(response.to_string())
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
    identity: Option<&LifecycleIdentity>,
    child: bool,
) -> Value {
    let mut metadata = json!({
        "project_id": vault.project_id,
        "adapter": adapter_name(adapter),
        "event": event_kind,
        "event_key": event_key,
        "task_id": task_id,
        "child": child,
    });
    if let Some(identity) = identity {
        metadata["operation_id"] = json!(identity.operation_id());
        metadata["session_id"] = json!(identity.session_id());
        metadata["request_id"] = json!(identity.request_id());
    }
    metadata
}

fn completed_stop_response(
    vault: &VaultContext,
    adapter: HookAdapter,
    event_kind: &str,
    event_key: &str,
    task_id: &str,
    identity: &LifecycleIdentity,
    child: bool,
) -> Result<String> {
    let mut metadata = hook_metadata(
        vault,
        adapter,
        event_kind,
        event_key,
        task_id,
        Some(identity),
        child,
    );
    metadata["operation_already_completed"] = json!(true);
    metadata["stop_is_completion"] = json!(false);
    Ok(serde_json::to_string(&json!({
        "continue": true,
        "systemMessage": "The exact Baron operation is already complete; allowing host Stop without repeating reconciliation.",
        "baron": metadata
    }))?)
}

fn recursion_depth(payload: &Value) -> u64 {
    payload
        .get("baron_recursion_depth")
        .or_else(|| payload.get("recursion_depth"))
        .or_else(|| payload.get("baronRecursionDepth"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
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

fn dedup_claim_is_live(state: &DedupState, key: &str) -> bool {
    state
        .entries
        .iter()
        .rev()
        .find(|entry| entry.key == key && entry.response.is_none())
        .is_some_and(dedup_entry_claim_is_live)
}

fn dedup_entry_claim_is_live(entry: &DedupEntry) -> bool {
    if entry.response.is_some() {
        return false;
    }
    let Some(claimed_at) = entry.claimed_at.as_deref() else {
        return true;
    };
    let Ok(claimed_at) = DateTime::parse_from_rfc3339(claimed_at) else {
        return true;
    };
    let age = Local::now().timestamp() - claimed_at.timestamp();
    age <= DEDUP_CLAIM_TTL_SECONDS
}

fn dedup_claim(state: &mut DedupState, key: &str) -> Result<String> {
    let claim_token = artifact_instance_id("00000000")?;
    let mut updated = state.clone();
    updated.entries.retain(|entry| entry.key != key);
    updated.entries.push(DedupEntry {
        key: key.to_string(),
        response: None,
        claimed_at: Some(now()),
        claim_token: Some(claim_token.clone()),
    });
    trim_dedup_entries(&mut updated)?;
    *state = updated;
    Ok(claim_token)
}

fn dedup_claim_matches(state: &DedupState, key: &str, claim_token: &str) -> bool {
    state
        .entries
        .iter()
        .rev()
        .find(|entry| entry.key == key && entry.response.is_none())
        .and_then(|entry| entry.claim_token.as_deref())
        == Some(claim_token)
}

fn renew_dedup_claim(
    repo_root: &Path,
    vault: &VaultContext,
    key: &str,
    claim_token: &str,
) -> Result<()> {
    let _lock = acquire_project_lock(repo_root)?;
    let mut state = load_dedup_state(vault)?;
    renew_dedup_claim_from_state(&mut state, key, claim_token)?;
    save_dedup_state(vault, &state)
}

fn renew_dedup_claim_from_state(
    state: &mut DedupState,
    key: &str,
    claim_token: &str,
) -> Result<()> {
    let Some(entry) = state
        .entries
        .iter_mut()
        .rev()
        .find(|entry| entry.key == key && entry.response.is_none())
    else {
        bail!(
            "Hook claim is no longer owned by this delivery; refusing stale mutation: {}",
            key
        );
    };
    if entry.claim_token.as_deref() != Some(claim_token) {
        bail!(
            "Hook claim is no longer owned by this delivery; refusing stale mutation: {}",
            key
        );
    }
    entry.claimed_at = Some(now());
    Ok(())
}

fn release_dedup_claim_from_state(state: &mut DedupState, key: &str, claim_token: &str) {
    state.entries.retain(|entry| {
        entry.key != key
            || entry.response.is_some()
            || entry.claim_token.as_deref() != Some(claim_token)
    });
}

fn release_dedup_claim(repo_root: &Path, vault: &VaultContext, key: &str, claim_token: &str) {
    let Ok(_lock) = acquire_project_lock(repo_root) else {
        return;
    };
    let Ok(mut state) = load_dedup_state(vault) else {
        return;
    };
    let before = state.entries.len();
    release_dedup_claim_from_state(&mut state, key, claim_token);
    if state.entries.len() != before {
        let _ = save_dedup_state(vault, &state);
    }
}

fn dedup_store_response(state: &mut DedupState, key: &str, response: &str) -> Result<()> {
    let mut updated = state.clone();
    updated.entries.retain(|entry| entry.key != key);
    updated.entries.push(DedupEntry {
        key: key.to_string(),
        response: Some(response.to_string()),
        claimed_at: None,
        claim_token: None,
    });
    trim_dedup_entries(&mut updated)?;
    *state = updated;
    Ok(())
}

fn trim_dedup_entries(state: &mut DedupState) -> Result<()> {
    while state.entries.len() > MAX_DEDUP_ENTRIES {
        let index = state
            .entries
            .iter()
            .position(|entry| entry.response.is_some())
            .or_else(|| {
                state
                    .entries
                    .iter()
                    .position(|entry| !dedup_entry_claim_is_live(entry))
            })
            .context("Hook dedup cache is full of live claims; refusing new delivery")?;
        state.entries.remove(index);
    }
    Ok(())
}

fn journal_needs_separator(path: &Path) -> Result<bool> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error).with_context(|| {
                format!("Could not inspect automation journal: {}", path.display())
            })
        }
    };
    journal_needs_separator_from_reader(file)
        .with_context(|| format!("Could not inspect automation journal: {}", path.display()))
}

fn journal_needs_separator_from_reader<R>(mut reader: R) -> std::io::Result<bool>
where
    R: Read + Seek,
{
    let length = reader.seek(SeekFrom::End(0))?;
    if length == 0 {
        return Ok(false);
    }
    reader.seek(SeekFrom::End(-1))?;
    let mut last_byte = [0u8; 1];
    reader.read_exact(&mut last_byte)?;
    Ok(last_byte[0] != b'\n')
}

fn journal_tail_entries(path: &Path) -> Result<Vec<JournalEntry>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Could not read automation journal: {}", path.display()))
        }
    };
    journal_tail_entries_from_reader(file, path)
}

fn journal_tail_entries_from_reader<R>(mut reader: R, path: &Path) -> Result<Vec<JournalEntry>>
where
    R: Read + Seek,
{
    let length = reader
        .seek(SeekFrom::End(0))
        .with_context(|| format!("Could not read automation journal: {}", path.display()))?;
    if length == 0 {
        return Ok(Vec::new());
    }

    reader
        .seek(SeekFrom::End(-1))
        .with_context(|| format!("Could not read automation journal: {}", path.display()))?;
    let mut last_byte = [0u8; 1];
    reader
        .read_exact(&mut last_byte)
        .with_context(|| format!("Could not read automation journal: {}", path.display()))?;
    let trailing_newline = last_byte[0] == b'\n';

    let mut chunks = Vec::new();
    let mut cursor = length;
    let mut bytes_scanned = 0usize;
    let mut newline_count = 0usize;
    let mut skip_trailing_newline = trailing_newline;
    while cursor > 0 && bytes_scanned < MAX_JOURNAL_SCAN_BYTES {
        let remaining_limit = MAX_JOURNAL_SCAN_BYTES - bytes_scanned;
        let chunk_len = cursor
            .min(remaining_limit as u64)
            .min(JOURNAL_READ_CHUNK_BYTES as u64) as usize;
        cursor -= chunk_len as u64;
        reader
            .seek(SeekFrom::Start(cursor))
            .with_context(|| format!("Could not read automation journal: {}", path.display()))?;
        let mut chunk = vec![0u8; chunk_len];
        reader
            .read_exact(&mut chunk)
            .with_context(|| format!("Could not read automation journal: {}", path.display()))?;
        for byte in chunk.iter().rev() {
            if skip_trailing_newline {
                skip_trailing_newline = false;
                continue;
            }
            if *byte == b'\n' {
                newline_count += 1;
            }
        }
        bytes_scanned += chunk_len;
        chunks.push(chunk);
        if newline_count >= MAX_JOURNAL_SCAN_LINES {
            break;
        }
    }

    if newline_count < MAX_JOURNAL_SCAN_LINES && cursor > 0 {
        bail!(
            "Automation journal tail exceeds the {} byte scan limit before reaching {} lines: {}",
            MAX_JOURNAL_SCAN_BYTES,
            MAX_JOURNAL_SCAN_LINES,
            path.display()
        );
    }

    let mut content = Vec::with_capacity(bytes_scanned);
    for chunk in chunks.iter().rev() {
        content.extend_from_slice(chunk);
    }
    if newline_count >= MAX_JOURNAL_SCAN_LINES {
        let mut skip_trailing_newline = trailing_newline;
        let mut seen = 0usize;
        let mut start = None;
        for index in (0..content.len()).rev() {
            if skip_trailing_newline {
                skip_trailing_newline = false;
                continue;
            }
            if content[index] == b'\n' {
                seen += 1;
                if seen == MAX_JOURNAL_SCAN_LINES {
                    start = Some(index);
                    break;
                }
            }
        }
        let start = start.context("Could not locate the bounded automation journal tail")?;
        content.drain(..start);
    }

    let content = std::str::from_utf8(&content).with_context(|| {
        format!(
            "Automation journal tail is not valid UTF-8 within the latest {MAX_JOURNAL_SCAN_LINES} lines: {}",
            path.display()
        )
    })?;
    let mut entries = Vec::new();
    for (offset_from_end, line) in content
        .lines()
        .rev()
        .take(MAX_JOURNAL_SCAN_LINES)
        .enumerate()
    {
        if line.trim().is_empty() {
            continue;
        }
        let entry = serde_json::from_str::<JournalEntry>(line).with_context(|| {
            format!(
                "Malformed automation journal JSONL row within the latest {MAX_JOURNAL_SCAN_LINES} lines ({} lines from end): {}",
                offset_from_end + 1,
                path.display()
            )
        })?;
        entries.push(entry);
    }
    Ok(entries)
}

fn journal_contains_key(path: &Path, key: &str) -> Result<bool> {
    Ok(journal_tail_entries(path)?
        .iter()
        .any(|entry| entry.event_key.as_deref() == Some(key)))
}

fn journal_response(path: &Path, key: &str) -> Result<Option<String>> {
    Ok(journal_tail_entries(path)?.into_iter().find_map(|entry| {
        (entry.event_key.as_deref() == Some(key))
            .then_some(entry.response)
            .flatten()
    }))
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

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{initialize_project, AdapterKind};
    use crate::vault::ensure_vault;
    use tempfile::tempdir;

    fn diagnostic_test_entry(event: AutomationEvent, event_kind: &str) -> JournalEntry {
        JournalEntry {
            timestamp: now(),
            event,
            adapter: JournalAdapter::Supported(HookAdapter::Codex),
            session_id: Some("diagnostic-session".to_string()),
            request_id: None,
            event_key: None,
            event_kind: event_kind.to_string(),
            task_id: None,
            child_id: None,
            parent_task_id: None,
            parent_session_id: None,
            child: false,
            evidence: None,
            response: None,
        }
    }

    #[test]
    fn journal_diagnostic_snapshot_excludes_appends_after_capture() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault_root = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
        let vault = ensure_vault(&vault_root, &repo).unwrap();
        let journal = journal_path(&vault);
        fs::create_dir_all(journal.parent().unwrap()).unwrap();
        let first = serde_json::to_vec(&diagnostic_test_entry(
            AutomationEvent::ContextCompiled,
            "context_compiled",
        ))
        .unwrap();
        fs::write(&journal, [first.as_slice(), b"\n"].concat()).unwrap();

        let snapshot = open_automation_journal_snapshot(&repo, &vault)
            .unwrap()
            .unwrap();
        record_lifecycle_event(&vault, HookAdapter::Codex, AutomationEvent::PlanStarted).unwrap();

        let diagnostics = scan_journal_snapshot(snapshot).unwrap();
        assert_eq!(diagnostics.presence, JournalPresence::Present);
        assert_eq!(diagnostics.event_count, 1);
        assert_eq!(
            diagnostics.latest_event.as_deref(),
            Some("context_compiled")
        );
    }

    #[test]
    fn journal_diagnostic_snapshot_scans_while_mutation_locks_are_held() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault_root = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
        let vault = ensure_vault(&vault_root, &repo).unwrap();
        let journal = journal_path(&vault);
        fs::create_dir_all(journal.parent().unwrap()).unwrap();
        let row = serde_json::to_vec(&diagnostic_test_entry(
            AutomationEvent::ContextCompiled,
            "context_compiled",
        ))
        .unwrap();
        fs::write(&journal, [row.as_slice(), b"\n"].concat()).unwrap();
        let snapshot = open_automation_journal_snapshot(&repo, &vault)
            .unwrap()
            .unwrap();

        let repo_lock = acquire_project_lock(&repo).unwrap();
        let vault_lock = acquire_project_lock(&vault.project_root).unwrap();
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            result_tx.send(scan_journal_snapshot(snapshot)).unwrap();
        });
        let result_while_locked = result_rx.recv_timeout(std::time::Duration::from_secs(2));
        drop(vault_lock);
        drop(repo_lock);

        let diagnostics = result_while_locked
            .expect("snapshot scan should not wait for mutation locks")
            .unwrap();
        worker.join().unwrap();
        assert_eq!(diagnostics.event_count, 1);
    }

    #[test]
    fn journal_diagnostic_scanner_stops_at_the_bounded_row_limit() {
        let content = vec![b'x'; MAX_DIAGNOSTIC_JOURNAL_ROW_BYTES + JOURNAL_READ_CHUNK_BYTES * 8];
        let file_len = content.len();
        let reader = CountingReader {
            inner: std::io::Cursor::new(content),
            bytes_read: 0,
        };
        let mut reader = BufReader::with_capacity(JOURNAL_READ_CHUNK_BYTES, reader);

        let error =
            scan_journal_reader(&mut reader, Path::new("automation-journal.jsonl")).unwrap_err();

        assert!(error.to_string().contains("row exceeds"));
        assert!(
            reader.get_ref().bytes_read
                <= MAX_DIAGNOSTIC_JOURNAL_ROW_BYTES + JOURNAL_READ_CHUNK_BYTES
        );
        assert!(reader.get_ref().bytes_read < file_len);
    }

    #[test]
    fn journal_diagnostic_scanner_caps_retained_event_kinds() {
        let mut content = Vec::new();
        for index in 0..=MAX_DIAGNOSTIC_EVENT_KINDS {
            let row = serde_json::to_vec(&diagnostic_test_entry(
                AutomationEvent::ContextCompiled,
                &format!("event-kind-{index}"),
            ))
            .unwrap();
            content.extend_from_slice(&row);
            content.push(b'\n');
        }
        let reader =
            BufReader::with_capacity(JOURNAL_READ_CHUNK_BYTES, std::io::Cursor::new(content));

        let error = scan_journal_reader(reader, Path::new("automation-journal.jsonl")).unwrap_err();

        assert!(error
            .to_string()
            .contains("unique event-kind diagnostic limit"));
    }

    fn seed_stop_correlation(
        repo: &Path,
        vault: &VaultContext,
        identity: &LifecycleIdentity,
        task: &str,
    ) {
        let payload = json!({"session_id":identity.session_id(),"request_id":identity.request_id(),"task":task});
        let ingress = hook_correlation::Ingress::parse(&payload, HookAdapter::Codex).unwrap();
        let _lock = acquire_project_lock(repo).unwrap();
        let (_, established, completed) = hook_correlation::resolve_locked(
            repo,
            vault,
            SupportedAdapter::Codex,
            AutomationEvent::UserPromptSubmit,
            &ingress,
        )
        .unwrap();
        assert!(!completed);
        assert_eq!(&established, identity);
        crate::plan::start_or_resume_plan_for_identity(repo, vault, task, identity).unwrap();
    }

    #[test]
    fn stale_hook_claim_cannot_release_or_publish_a_replacement_claim() {
        let mut state = DedupState::default();
        let old_claim = dedup_claim(&mut state, "event-key").unwrap();
        let new_claim = dedup_claim(&mut state, "event-key").unwrap();

        assert_ne!(old_claim, new_claim);
        assert!(!dedup_claim_matches(&state, "event-key", &old_claim));
        assert!(dedup_claim_matches(&state, "event-key", &new_claim));

        release_dedup_claim_from_state(&mut state, "event-key", &old_claim);
        assert!(dedup_claim_matches(&state, "event-key", &new_claim));

        release_dedup_claim_from_state(&mut state, "event-key", &new_claim);
        assert!(!dedup_claim_matches(&state, "event-key", &new_claim));
    }

    #[test]
    fn malformed_or_future_hook_claims_fail_closed() {
        let mut state = DedupState::default();
        state.entries.push(DedupEntry {
            key: "malformed".to_string(),
            response: None,
            claimed_at: Some("not-a-timestamp".to_string()),
            claim_token: Some("owner".to_string()),
        });
        state.entries.push(DedupEntry {
            key: "future".to_string(),
            response: None,
            claimed_at: Some((Local::now() + chrono::Duration::minutes(5)).to_rfc3339()),
            claim_token: Some("owner".to_string()),
        });

        assert!(dedup_claim_is_live(&state, "malformed"));
        assert!(dedup_claim_is_live(&state, "future"));
    }

    #[test]
    fn live_hook_claim_survives_response_cache_trimming() {
        let mut state = DedupState::default();
        state.entries.push(DedupEntry {
            key: "in-flight".to_string(),
            response: None,
            claimed_at: Some(now()),
            claim_token: Some("live-owner".to_string()),
        });
        for index in 0..MAX_DEDUP_ENTRIES - 1 {
            state.entries.push(DedupEntry {
                key: format!("response-{index}"),
                response: Some("cached response".to_string()),
                claimed_at: None,
                claim_token: None,
            });
        }

        dedup_store_response(&mut state, "new-response", "new cached response").unwrap();

        assert_eq!(state.entries.len(), MAX_DEDUP_ENTRIES);
        assert!(dedup_claim_is_live(&state, "in-flight"));
        assert!(dedup_claim_matches(&state, "in-flight", "live-owner"));
        assert!(dedup_response(&state, "response-0").is_none());
    }

    #[test]
    fn dedup_capacity_of_live_claims_fails_closed() {
        let mut state = DedupState::default();
        for index in 0..MAX_DEDUP_ENTRIES {
            state.entries.push(DedupEntry {
                key: format!("live-{index}"),
                response: None,
                claimed_at: Some(now()),
                claim_token: Some(format!("owner-{index}")),
            });
        }
        let before = serde_json::to_value(&state).unwrap();

        let result = dedup_claim(&mut state, "new-live-claim");

        assert!(
            result.is_err(),
            "a full cache of in-flight claims must reject a new delivery"
        );
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }

    #[test]
    fn dedup_trimming_preserves_malformed_and_future_claims() {
        let mut state = DedupState::default();
        state.entries.push(DedupEntry {
            key: "expired".to_string(),
            response: None,
            claimed_at: Some((Local::now() - chrono::Duration::minutes(10)).to_rfc3339()),
            claim_token: Some("expired-owner".to_string()),
        });
        state.entries.push(DedupEntry {
            key: "malformed".to_string(),
            response: None,
            claimed_at: Some("not-a-timestamp".to_string()),
            claim_token: Some("malformed-owner".to_string()),
        });
        state.entries.push(DedupEntry {
            key: "future".to_string(),
            response: None,
            claimed_at: Some((Local::now() + chrono::Duration::minutes(5)).to_rfc3339()),
            claim_token: Some("future-owner".to_string()),
        });
        while state.entries.len() < MAX_DEDUP_ENTRIES {
            let index = state.entries.len();
            state.entries.push(DedupEntry {
                key: format!("live-{index}"),
                response: None,
                claimed_at: Some(now()),
                claim_token: Some(format!("owner-{index}")),
            });
        }

        let replacement = dedup_claim(&mut state, "replacement").unwrap();

        assert_eq!(state.entries.len(), MAX_DEDUP_ENTRIES);
        assert!(!state.entries.iter().any(|entry| entry.key == "expired"));
        assert!(dedup_claim_is_live(&state, "malformed"));
        assert!(dedup_claim_is_live(&state, "future"));
        assert!(dedup_claim_matches(&state, "replacement", &replacement));
    }

    #[test]
    fn journal_response_rehydrates_dedup_after_publication_crash() {
        let temp = tempdir().unwrap();
        let journal = temp.path().join("automation-journal.jsonl");
        let event_key = "event-after-journal-append";
        let response = r#"{"continue":true,"baron":{"event_key":"event-after-journal-append"}}"#;
        let entry = JournalEntry {
            timestamp: now(),
            event: AutomationEvent::UserPromptSubmit,
            adapter: JournalAdapter::Supported(HookAdapter::Codex),
            session_id: Some("session".to_string()),
            request_id: Some("request".to_string()),
            event_key: Some(event_key.to_string()),
            event_kind: "user_prompt_submit".to_string(),
            task_id: Some("task".to_string()),
            child_id: None,
            parent_task_id: None,
            parent_session_id: None,
            child: false,
            evidence: None,
            response: Some(response.to_string()),
        };
        fs::write(
            &journal,
            format!("{}\n", serde_json::to_string(&entry).unwrap()),
        )
        .unwrap();

        let mut state = DedupState::default();
        state.entries.push(DedupEntry {
            key: event_key.to_string(),
            response: None,
            claimed_at: Some(now()),
            claim_token: Some("crashed-owner".to_string()),
        });
        let recovered = journal_response(&journal, event_key).unwrap().unwrap();
        dedup_store_response(&mut state, event_key, &recovered).unwrap();

        assert_eq!(recovered, response);
        assert_eq!(dedup_response(&state, event_key).as_deref(), Some(response));
    }

    #[test]
    fn journal_tail_scan_ignores_a_large_prefix_outside_the_replay_window() {
        let temp = tempdir().unwrap();
        let journal = temp.path().join("automation-journal.jsonl");
        fs::write(&journal, journal_with_large_invalid_prefix()).unwrap();

        let entries = journal_tail_entries(&journal).unwrap();

        assert_eq!(entries.len(), MAX_JOURNAL_SCAN_LINES);
        assert_eq!(
            entries.first().unwrap().event_key.as_deref(),
            Some("tail-512")
        );
        assert_eq!(entries.last().unwrap().event_key.as_deref(), Some("tail-1"));
        assert!(!entries
            .iter()
            .any(|entry| entry.event_key.as_deref() == Some("tail-0")));
    }

    #[test]
    fn journal_tail_reader_does_not_read_the_full_history() {
        let content = journal_with_large_invalid_prefix();
        let file_len = content.len();
        let mut reader = CountingReader {
            inner: std::io::Cursor::new(content),
            bytes_read: 0,
        };

        let entries =
            journal_tail_entries_from_reader(&mut reader, Path::new("automation-journal.jsonl"))
                .unwrap();

        assert_eq!(entries.len(), MAX_JOURNAL_SCAN_LINES);
        assert!(reader.bytes_read <= MAX_JOURNAL_SCAN_BYTES + 1);
        assert!(reader.bytes_read < file_len / 2);
    }

    #[test]
    fn journal_separator_check_reads_only_the_final_byte() {
        let content = vec![b'x'; 8 * 1024 * 1024];
        let mut reader = CountingReader {
            inner: std::io::Cursor::new(content),
            bytes_read: 0,
        };

        assert!(journal_needs_separator_from_reader(&mut reader).unwrap());
        assert_eq!(reader.bytes_read, 1);
    }

    fn journal_with_large_invalid_prefix() -> Vec<u8> {
        let mut content = vec![0xff; 8 * 1024 * 1024];
        content.push(b'\n');

        for index in 0..=MAX_JOURNAL_SCAN_LINES {
            let entry = JournalEntry {
                timestamp: now(),
                event: AutomationEvent::UserPromptSubmit,
                adapter: JournalAdapter::Supported(HookAdapter::Codex),
                session_id: Some("session".to_string()),
                request_id: Some(format!("request-{index}")),
                event_key: Some(format!("tail-{index}")),
                event_kind: "user_prompt_submit".to_string(),
                task_id: Some("task".to_string()),
                child_id: None,
                parent_task_id: None,
                parent_session_id: None,
                child: false,
                evidence: None,
                response: Some("{}".to_string()),
            };
            content.extend(serde_json::to_vec(&entry).unwrap());
            content.push(b'\n');
        }
        content
    }

    struct CountingReader {
        inner: std::io::Cursor<Vec<u8>>,
        bytes_read: usize,
    }

    impl Read for CountingReader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let read = self.inner.read(buffer)?;
            self.bytes_read += read;
            Ok(read)
        }
    }

    impl Seek for CountingReader {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(position)
        }
    }

    #[test]
    fn hook_response_publication_serializes_shared_vault_journal_across_checkouts() {
        let temp = tempdir().unwrap();
        let repo_a = temp.path().join("checkout-a");
        let repo_b = temp.path().join("checkout-b");
        let vault_root = temp.path().join("shared-vault");
        std::fs::create_dir_all(&repo_a).unwrap();
        initialize_project(&repo_a, AdapterKind::Codex, &vault_root).unwrap();
        let vault_a = ensure_vault(&vault_root, &repo_a).unwrap();

        std::fs::create_dir_all(repo_b.join(".baron")).unwrap();
        std::fs::copy(
            repo_a.join(".baron/project.toml"),
            repo_b.join(".baron/project.toml"),
        )
        .unwrap();
        let vault_b = crate::vault::vault_context_without_create(&vault_root, &repo_b).unwrap();
        assert_eq!(vault_a.project_root, vault_b.project_root);

        let event_key = "shared-checkout-hook-event";
        let mut state_a = DedupState::default();
        let claim_a = dedup_claim(&mut state_a, event_key).unwrap();
        save_dedup_state(&vault_a, &state_a).unwrap();
        let entry = JournalEntry {
            timestamp: now(),
            event: AutomationEvent::ContextCompiled,
            adapter: HookAdapter::Codex.into(),
            session_id: Some("shared-session".to_string()),
            request_id: Some("shared-request".to_string()),
            event_kind: "context_compiled".to_string(),
            event_key: Some(event_key.to_string()),
            task_id: Some("shared-task".to_string()),
            child_id: None,
            parent_task_id: None,
            parent_session_id: None,
            child: false,
            evidence: None,
            response: None,
        };
        let response_a = r#"{"source":"checkout-a"}"#;
        assert_eq!(
            publish_hook_response(
                &repo_a, &vault_a, &entry, event_key, &claim_a, response_a, false,
            )
            .unwrap(),
            response_a
        );

        let mut state_b = DedupState::default();
        let claim_b = dedup_claim(&mut state_b, event_key).unwrap();
        save_dedup_state(&vault_b, &state_b).unwrap();
        let shared_vault_lock = acquire_project_lock(&vault_a.project_root).unwrap();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            publish_hook_response(
                &repo_b,
                &vault_b,
                &entry,
                event_key,
                &claim_b,
                r#"{"source":"checkout-b"}"#,
                false,
            )
        });
        started_rx.recv().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let completed_while_locked = worker.is_finished();
        drop(shared_vault_lock);
        let response_b = worker.join().unwrap().unwrap();

        assert!(
            !completed_while_locked,
            "a checkout published or deduplicated a hook response without the shared Vault lock"
        );
        assert_eq!(response_b, response_a);
        let journal = std::fs::read_to_string(journal_path(&vault_a)).unwrap();
        assert_eq!(
            journal
                .lines()
                .filter(|line| line.contains(event_key))
                .count(),
            1,
            "the same lifecycle event must have one shared Vault journal row"
        );
    }

    #[test]
    fn stop_publication_rejects_reconciliation_staled_by_new_operation_trace() {
        for stop_hook_active in [false, true] {
            assert_stop_publication_rechecks_failed_evidence(stop_hook_active);
        }
    }

    fn assert_stop_publication_rechecks_failed_evidence(stop_hook_active: bool) {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault_root = temp.path().join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
        let vault = ensure_vault(&vault_root, &repo).unwrap();
        let identity = LifecycleIdentity::resolve(
            &vault.project_id,
            "fix README typo",
            SupportedAdapter::Codex,
            Some("publication-race-session"),
            Some("publication-race-turn"),
        )
        .unwrap();
        let prompt = json!({
            "session_id": "publication-race-session",
            "request_id": "publication-race-turn",
            "task": "fix README typo"
        });
        let ingress = hook_correlation::Ingress::parse(&prompt, HookAdapter::Codex).unwrap();
        {
            let _lock = acquire_project_lock(&repo).unwrap();
            let (_, established, completed) = hook_correlation::resolve_locked(
                &repo,
                &vault,
                SupportedAdapter::Codex,
                AutomationEvent::UserPromptSubmit,
                &ingress,
            )
            .unwrap();
            assert_eq!(established, identity);
            assert!(!completed);
        }
        crate::plan::start_or_resume_plan_for_identity(&repo, &vault, "fix README typo", &identity)
            .unwrap();

        let operation = OperationContext::from_identity(&identity);
        let proof = crate::proof::record_proof_for_operation(
            &repo,
            &vault,
            &operation,
            "README verification passed",
        )
        .unwrap();
        let binding =
            crate::trace::TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
        let passing_trace = crate::trace::record_trace_for_operation(
            &repo,
            &vault,
            "README typo fixed and verified",
            crate::trace::TraceOutcome::Completed,
            &binding,
        )
        .unwrap();
        assert!(
            crate::trace::score_trace(&repo, &vault, Some(&passing_trace.id))
                .unwrap()
                .passed
        );
        assert!(
            reconcile_for_operation(&repo, &vault, &identity)
                .unwrap()
                .passed
        );

        let publication_gate = std::sync::Arc::new((
            std::sync::Mutex::new((false, false)),
            std::sync::Condvar::new(),
        ));
        let hook_gate = publication_gate.clone();
        *STOP_BEFORE_PUBLICATION
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some((
            repo.clone(),
            Box::new(move || {
                let (state, changed) = &*hook_gate;
                let mut state = state.lock().unwrap();
                state.0 = true;
                changed.notify_all();
                while !state.1 {
                    state = changed.wait(state).unwrap();
                }
            }),
        ));

        let stop_repo = repo.clone();
        let stop_vault = vault.clone();
        let stop = std::thread::spawn(move || {
            handle_hook(
                &stop_repo,
                &stop_vault,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                &json!({
                    "session_id": "publication-race-session",
                    "request_id": "publication-race-turn",
                    "stop_hook_active": stop_hook_active
                })
                .to_string(),
            )
        });
        {
            let (state, changed) = &*publication_gate;
            let mut state = state.lock().unwrap();
            while !state.0 {
                let (next, timeout) = changed
                    .wait_timeout(state, std::time::Duration::from_secs(15))
                    .unwrap();
                state = next;
                assert!(!timeout.timed_out(), "Stop did not reach final publication");
            }
        }

        // Another process can publish newer evidence for this exact operation
        // after the first reconciliation and before Stop response publication.
        std::thread::sleep(std::time::Duration::from_millis(5));
        let failing_trace = crate::trace::record_trace_for_operation(
            &repo,
            &vault,
            "",
            crate::trace::TraceOutcome::Failed,
            &binding,
        )
        .unwrap();
        assert!(
            !crate::trace::score_trace(&repo, &vault, Some(&failing_trace.id))
                .unwrap()
                .passed
        );

        {
            let (state, changed) = &*publication_gate;
            let mut state = state.lock().unwrap();
            state.1 = true;
            changed.notify_all();
        }
        let response: Value = serde_json::from_str(&stop.join().unwrap().unwrap()).unwrap();
        assert_eq!(
            response["decision"], "block",
            "Stop must not publish a stale passing reconciliation: {response}"
        );
        assert_eq!(response["baron"]["reconciliation_passed"], false);
    }

    #[test]
    fn stop_replay_reconciles_latest_operation_evidence_after_cached_pass() {
        for stop_hook_active in [false, true] {
            assert_stop_replay_rechecks_failed_evidence(stop_hook_active);
        }
    }

    fn assert_stop_replay_rechecks_failed_evidence(stop_hook_active: bool) {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault_root = temp.path().join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
        let vault = ensure_vault(&vault_root, &repo).unwrap();
        let identity = LifecycleIdentity::resolve(
            &vault.project_id,
            "fix README typo",
            SupportedAdapter::Codex,
            Some("stop-replay-session"),
            Some("stop-replay-turn"),
        )
        .unwrap();
        seed_stop_correlation(&repo, &vault, &identity, "fix README typo");

        let operation = OperationContext::from_identity(&identity);
        let proof = crate::proof::record_proof_for_operation(
            &repo,
            &vault,
            &operation,
            "README verification passed",
        )
        .unwrap();
        let binding =
            crate::trace::TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
        let passing_trace = crate::trace::record_trace_for_operation(
            &repo,
            &vault,
            "README typo fixed and verified",
            crate::trace::TraceOutcome::Completed,
            &binding,
        )
        .unwrap();
        assert!(
            crate::trace::score_trace(&repo, &vault, Some(&passing_trace.id))
                .unwrap()
                .passed
        );
        assert!(
            reconcile_for_operation(&repo, &vault, &identity)
                .unwrap()
                .passed
        );

        let stop_payload = json!({
            "session_id": "stop-replay-session",
            "request_id": "stop-replay-turn",
            "stop_hook_active": stop_hook_active
        })
        .to_string();
        let first: Value = serde_json::from_str(
            &handle_hook(
                &repo,
                &vault,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                &stop_payload,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(first["continue"], true);
        assert_eq!(first["baron"]["reconciliation_passed"], true);

        let failing_trace = crate::trace::record_trace_for_operation(
            &repo,
            &vault,
            "",
            crate::trace::TraceOutcome::Failed,
            &binding,
        )
        .unwrap();
        assert!(
            !crate::trace::score_trace(&repo, &vault, Some(&failing_trace.id))
                .unwrap()
                .passed
        );

        // Re-open the Vault handle to exercise durable dedup/journal recovery
        // rather than relying on process-local hook state.
        let restarted_vault = ensure_vault(&vault_root, &repo).unwrap();
        let replay: Value = serde_json::from_str(
            &handle_hook(
                &repo,
                &restarted_vault,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                &stop_payload,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            replay["decision"], "block",
            "identical Stop replay must reconcile current evidence: {replay}"
        );
        assert_eq!(replay["baron"]["reconciliation_passed"], false);

        // Simulate losing the bounded cache after restart: the journal still
        // contains the original passing response and must not authorize Stop.
        std::fs::remove_file(repo.join(DEDUP_PATH)).unwrap();
        let replay_from_journal: Value = serde_json::from_str(
            &handle_hook(
                &repo,
                &restarted_vault,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                &stop_payload,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            replay_from_journal["decision"], "block",
            "journal recovery must not reuse a historical Stop pass: {replay_from_journal}"
        );
        assert_eq!(replay_from_journal["baron"]["reconciliation_passed"], false);
    }

    #[test]
    fn live_duplicate_stop_claim_fails_closed_while_reconciliation_is_pending() {
        for stop_hook_active in [false, true] {
            assert_live_duplicate_stop_claim_blocks(stop_hook_active);
        }
    }

    fn assert_live_duplicate_stop_claim_blocks(stop_hook_active: bool) {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let identity = LifecycleIdentity::resolve(
            &context.project_id,
            "current repository state",
            SupportedAdapter::Codex,
            Some("stop-session"),
            Some("stop-request"),
        )
        .unwrap();
        seed_stop_correlation(&repo, &context, &identity, "current repository state");
        let event_key = LifecycleEventKey {
            project_id: context.project_id.clone(),
            adapter: "codex".to_string(),
            session_id: Some(identity.session_id().to_string()),
            request_id: Some(identity.request_id().to_string()),
            event_kind: "stop".to_string(),
            task_id: identity.task_id().to_string(),
            child_id: None,
            stop_hook_active,
        }
        .stable_id();
        let dedup_path = repo.join(DEDUP_PATH);
        std::fs::create_dir_all(dedup_path.parent().unwrap()).unwrap();
        std::fs::write(
            &dedup_path,
            serde_json::to_string(&DedupState {
                schema_version: DEDUP_SCHEMA_VERSION,
                entries: vec![DedupEntry {
                    key: event_key,
                    response: None,
                    claimed_at: Some(now()),
                    claim_token: Some("active-owner".to_string()),
                }],
            })
            .unwrap(),
        )
        .unwrap();

        let pending_claim = fs::read(&dedup_path).unwrap();
        let response = handle_hook(
            &repo,
            &context,
            HookAdapter::Codex,
            AutomationEvent::Stop,
            &json!({
                "session_id": "stop-session",
                "request_id": "stop-request",
                "stop_hook_active": stop_hook_active
            })
            .to_string(),
        )
        .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["decision"], "block");
        assert_eq!(response["completed"], false);
        assert_eq!(response["baron"]["reconciliation_pending"], true);
        assert_eq!(
            pending_claim,
            fs::read(&dedup_path).unwrap(),
            "a duplicate must preserve the pending owner's claim"
        );
    }

    #[test]
    fn same_process_duplicate_stop_fails_closed_while_reconciliation_is_pending() {
        for stop_hook_active in [false, true] {
            assert_same_process_duplicate_stop_blocks(stop_hook_active);
        }
    }

    fn assert_same_process_duplicate_stop_blocks(stop_hook_active: bool) {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let identity = LifecycleIdentity::resolve(
            &context.project_id,
            "current repository state",
            SupportedAdapter::Codex,
            Some("same-process-stop-session"),
            Some("same-process-stop-request"),
        )
        .unwrap();
        seed_stop_correlation(&repo, &context, &identity, "current repository state");
        let event_key = LifecycleEventKey {
            project_id: context.project_id.clone(),
            adapter: "codex".to_string(),
            session_id: Some(identity.session_id().to_string()),
            request_id: Some(identity.request_id().to_string()),
            event_kind: "stop".to_string(),
            task_id: identity.task_id().to_string(),
            child_id: None,
            stop_hook_active,
        }
        .stable_id();
        let active = ACTIVE_HOOK_KEYS.get_or_init(|| Mutex::new(HashSet::new()));
        active
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(event_key.clone());

        let response = handle_hook(
            &repo,
            &context,
            HookAdapter::Codex,
            AutomationEvent::Stop,
            &json!({
                "session_id": "same-process-stop-session",
                "request_id": "same-process-stop-request",
                "stop_hook_active": stop_hook_active
            })
            .to_string(),
        )
        .unwrap();

        active
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .remove(&event_key);
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["decision"], "block");
        assert_eq!(response["completed"], false);
        assert_eq!(response["baron"]["reconciliation_pending"], true);
    }
}
