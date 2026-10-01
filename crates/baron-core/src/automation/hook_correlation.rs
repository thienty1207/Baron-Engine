//! Durable native turn correlation, separate from response deduplication.
//! Official schema snapshot (checked 2026-10-01):
//! https://learn.chatgpt.com/docs/hooks (unversioned Codex reference)
//! https://code.claude.com/docs/en/hooks (Claude Code >=2.1.196).
//! Host   SessionStart   UserPromptSubmit       PreCompact/PostToolUse  Stop
//! Codex  session only   session+turn_id+prompt session+turn_id         session+turn_id
//! Claude session only* session+prompt_id+prompt session+prompt_id*     session+prompt_id*
//! * prompt_id is common, absent before first input and in older hosts.
//!
//! Stop has last_assistant_message, never the original prompt. Transcripts
//! can lag asynchronously and are deliberately not a correlation source.
//!
//! request_id is Baron compatibility transport; Claude turn_id is not an alias.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{adapter_name, now, AutomationEvent, HookAdapter};
use crate::operation::{
    canonical_task_text, LifecycleIdentity, SupportedAdapter, MAX_IDENTIFIER_CHARS,
};
use crate::plan::{active_plan_authority_for_binding, PlanOperationBinding};
use crate::prepare::PREPARE_MAX_INPUT_BYTES;
use crate::safe_io::{read_text, replace_text};
use crate::vault::{canonical_project_id, VaultContext};

const MAP_PATH: &str = ".baron/state/hook-correlations.json";
const MAX_ENTRIES: usize = 256;
const MAX_STATE_BYTES: usize = 4 * 1024 * 1024;
const COMPLETED_RETENTION_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Debug)]
pub(super) struct Ingress {
    pub session: Option<String>,
    pub turn: Option<String>,
    pub task: Option<String>,
    pub native_turn: bool,
}

impl Ingress {
    pub fn parse(payload: &Value, adapter: HookAdapter) -> Result<Self> {
        if !payload.is_object() {
            bail!("hook input must be a JSON object");
        }
        if payload
            .get("request")
            .is_some_and(|request| !request.is_object())
        {
            bail!("hook request container must be a JSON object");
        }
        let session = aliases(payload, &["session_id", "sessionId"], false)?;
        let (turn, native_turn) = match adapter {
            HookAdapter::Codex => (
                aliases(
                    payload,
                    &["request_id", "requestId", "turn_id", "turnId"],
                    false,
                )?,
                aliases(payload, &["turn_id", "turnId"], false)?.is_some(),
            ),
            HookAdapter::Claude => {
                // Claude prompt_id has its own host semantics. Do not treat
                // Codex turn_id or tool_use_id as a Claude prompt identifier.
                let prompt = aliases(payload, &["prompt_id", "promptId"], false)?;
                let compatibility = aliases(payload, &["request_id", "requestId"], false)?;
                if let (Some(prompt), Some(request)) = (&prompt, &compatibility) {
                    if prompt != request {
                        bail!("conflicting Claude prompt_id and Baron request_id");
                    }
                }
                let native = prompt.is_some();
                (prompt.or(compatibility), native)
            }
            HookAdapter::Neutral => (
                aliases(payload, &["request_id", "requestId"], false)?,
                false,
            ),
        };
        let task = aliases(
            payload,
            &[
                "task",
                "prompt",
                "user_prompt",
                "userPrompt",
                "text",
                "message",
            ],
            true,
        )?;
        Ok(Self {
            session,
            turn,
            task,
            native_turn,
        })
    }
}

fn aliases(payload: &Value, keys: &[&str], task: bool) -> Result<Option<String>> {
    let mut selected = None;
    for object in std::iter::once(payload).chain(payload.get("request")) {
        for key in keys {
            let Some(value) = object.get(*key) else {
                continue;
            };
            let value = value
                .as_str()
                .with_context(|| format!("hook field {key} must be a string"))?;
            let normalized = if task {
                if value.len() > PREPARE_MAX_INPUT_BYTES {
                    bail!("hook task exceeds input bound");
                }
                canonical_task_text(value)?
            } else {
                let value = value.trim();
                if value.is_empty()
                    || value.chars().count() > MAX_IDENTIFIER_CHARS
                    || value.chars().any(char::is_control)
                {
                    bail!("invalid hook identifier {key}");
                }
                value.to_string()
            };
            if selected
                .as_ref()
                .is_some_and(|previous| previous != &normalized)
            {
                bail!("conflicting hook aliases for {}", keys[0]);
            }
            selected = Some(normalized);
        }
    }
    Ok(selected)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema_version: u32,
    project_id: String,
    entries: Vec<Entry>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    host_session_id: String,
    host_turn_id: Option<String>,
    canonical_task: String,
    identity: StoredIdentity,
    created_at: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredIdentity {
    project_id: String,
    task_id: String,
    operation_id: String,
    adapter: SupportedAdapter,
    session_id: String,
    request_id: String,
}

impl StoredIdentity {
    fn from_identity(identity: &LifecycleIdentity) -> Self {
        Self {
            project_id: identity.project_id().into(),
            task_id: identity.task_id().into(),
            operation_id: identity.operation_id().into(),
            adapter: identity.adapter(),
            session_id: identity.session_id().into(),
            request_id: identity.request_id().into(),
        }
    }
    fn checked(&self, task: &str) -> Result<LifecycleIdentity> {
        let identity = LifecycleIdentity::from_parts_checked(
            &self.project_id,
            &self.task_id,
            &self.operation_id,
            self.adapter,
            &self.session_id,
            &self.request_id,
        )?;
        identity.validate_task(task)?;
        Ok(identity)
    }
}

fn load(repo: &Path, vault: &VaultContext) -> Result<State> {
    if canonical_project_id(repo)? != vault.project_id
        || repo.canonicalize()? != vault.repo_root.canonicalize()?
    {
        bail!("hook correlation project/repository does not match Vault");
    }
    let path = repo.join(MAP_PATH);
    // Check the size before reading, then again after reading. safe_io rejects
    // symlinks/reparse points and non-regular files throughout the parent chain.
    if path.try_exists()? && std::fs::symlink_metadata(&path)?.len() > MAX_STATE_BYTES as u64 {
        bail!("hook correlation state exceeds byte bound");
    }
    let Some(content) = read_text(&path)? else {
        return Ok(State {
            schema_version: 1,
            project_id: vault.project_id.clone(),
            entries: vec![],
        });
    };
    if content.len() > MAX_STATE_BYTES {
        bail!("hook correlation state exceeds byte bound");
    }
    let state: State =
        serde_json::from_str(&content).context("malformed hook correlation state")?;
    if state.schema_version != 1
        || state.project_id != vault.project_id
        || state.entries.len() > MAX_ENTRIES
    {
        bail!("hook correlation schema, project, or entry bound mismatch");
    }
    let mut keys = HashSet::new();
    let mut operations = HashSet::new();
    for entry in &state.entries {
        let identity = entry.identity.checked(&entry.canonical_task)?;
        if entry.canonical_task.len() > PREPARE_MAX_INPUT_BYTES
            || canonical_task_text(&entry.canonical_task)? != entry.canonical_task
            || StoredIdentity::from_identity(&identity) != entry.identity
            || identity.project_id() != vault.project_id
            || identity.session_id() != entry.host_session_id
            || entry
                .host_turn_id
                .as_ref()
                .is_some_and(|turn| turn != identity.request_id())
        {
            bail!("hook correlation host tuple or canonical task mismatch");
        }
        let created = DateTime::parse_from_rfc3339(&entry.created_at)
            .context("invalid hook correlation timestamp")?;
        if created.timestamp() > Local::now().timestamp() + 60 {
            bail!("future hook correlation timestamp");
        }
        let key = (
            identity.adapter().as_str(),
            &entry.host_session_id,
            &entry.host_turn_id,
        );
        if !keys.insert(key) && entry.host_turn_id.is_some()
            || !operations.insert(identity.operation_id().to_string())
        {
            bail!("duplicate hook correlation key or operation");
        }
    }
    Ok(state)
}

fn binding(identity: &LifecycleIdentity) -> PlanOperationBinding {
    PlanOperationBinding {
        task_id: identity.task_id().into(),
        operation_id: identity.operation_id().into(),
        adapter: identity.adapter().as_str().into(),
        session_id: identity.session_id().into(),
        request_id: identity.request_id().into(),
    }
}

/// All callers hold the project lock. The plan API validates every ACTIVE
/// row and canonical frontmatter/path/status. We additionally require a row:
/// its legacy frontmatter discovery fallback cannot stand in for ACTIVE.
fn indexed_status(repo: &Path, identity: &LifecycleIdentity) -> Result<Option<String>> {
    let expected = binding(identity);
    let authority = active_plan_authority_for_binding(repo, &expected)?;
    let Some(content) = read_text(repo.join("docs/baron/plans/ACTIVE.md"))? else {
        return Ok(None);
    };
    let mut found = None;
    for line in content.lines() {
        let Some(row) = line
            .strip_prefix("<!-- BARON:ACTIVE-PLAN ")
            .and_then(|s| s.strip_suffix(" -->"))
        else {
            continue;
        };
        let row: Value = serde_json::from_str(row).context("malformed ACTIVE correlation row")?;
        if row["operation_id"] != identity.operation_id() {
            continue;
        }
        if found.is_some()
            || row["task_id"] != identity.task_id()
            || row["adapter"] != identity.adapter().as_str()
            || row["session_id"] != identity.session_id()
            || row["request_id"] != identity.request_id()
        {
            bail!("hook correlation identity disagrees with ACTIVE");
        }
        let status = row["status"].as_str().context("malformed ACTIVE status")?;
        if status != "completed"
            && authority.as_ref().and_then(|a| a.binding.as_ref()) != Some(&expected)
        {
            bail!("hook correlation lacks matching active plan authority");
        }
        found = Some(status.to_string());
    }
    Ok(found)
}

pub(super) fn validate_active(repo: &Path, identity: &LifecycleIdentity) -> Result<()> {
    match indexed_status(repo, identity)?.as_deref() {
        Some("in_progress" | "interrupted" | "needs_correction" | "blocked") => Ok(()),
        Some("completed") => bail!("hook correlation selects a completed plan"),
        _ => bail!("hook correlation has no exact active ACTIVE entry"),
    }
}

/// Resolve and, for Prompt only, establish a mapping under the existing
/// project lock. A map publication precedes slow prepare and is immutable
/// for that host tuple. Crash before/after response publication can retry the
/// same identity; the existing journal response and fenced dedup lease govern
/// delivery recovery. Pending mappings are retained even without a plan yet.
pub(super) fn resolve_locked(
    repo: &Path,
    vault: &VaultContext,
    adapter: SupportedAdapter,
    event: AutomationEvent,
    ingress: &Ingress,
) -> Result<(String, LifecycleIdentity)> {
    let mut state = load(repo, vault)?;
    let prompt = matches!(
        event,
        AutomationEvent::UserPromptSubmit | AutomationEvent::Prompt
    );
    let stop = event == AutomationEvent::Stop;
    if prompt && ingress.native_turn && ingress.task.is_none() {
        bail!("native Prompt correlation requires original prompt text");
    }
    let Some(session) = ingress.session.as_deref() else {
        if stop || ingress.native_turn {
            bail!("hook correlation requires a host session");
        }
        let task = ingress
            .task
            .clone()
            .unwrap_or_else(|| "current repository state".into());
        let identity = LifecycleIdentity::resolve(
            &vault.project_id,
            &task,
            adapter,
            None,
            ingress.turn.as_deref(),
        )?;
        return Ok((task, identity));
    };
    let mut candidates = Vec::new();
    for entry in &state.entries {
        if entry.identity.adapter != adapter || entry.host_session_id != session {
            continue;
        }
        let matches = if let Some(turn) = ingress.turn.as_deref() {
            entry.host_turn_id.as_deref() == Some(turn)
        } else if prompt {
            // Session+task cannot distinguish a new identical prompt from a
            // retry. Every id-less Prompt gets a fresh synthesized request.
            false
        } else {
            true
        };
        if matches {
            let identity = entry.identity.checked(&entry.canonical_task)?;
            // Include completed mappings in session-only ambiguity. A delayed
            // Stop has no field proving it belongs to a newer active prompt.
            candidates.push((entry, identity));
        }
    }
    if candidates.len() > 1 {
        bail!("hook correlation session selection is ambiguous");
    }
    if let Some((entry, identity)) = candidates.pop() {
        if ingress
            .task
            .as_ref()
            .is_some_and(|task| task != &entry.canonical_task)
        {
            bail!("hook correlation task conflicts with established host turn");
        }
        if stop {
            validate_active(repo, &identity)?;
        }
        return Ok((entry.canonical_task.clone(), identity));
    }
    if !prompt {
        // Historical Baron callers can still supply the original task with a
        // complete compatibility tuple. Never use this for native task-less
        // Stop or infer from session/CURRENT/adapter alone.
        if stop && !ingress.native_turn && ingress.turn.is_some() && ingress.task.is_some() {
            let task = ingress.task.clone().unwrap();
            let identity = LifecycleIdentity::resolve(
                &vault.project_id,
                &task,
                adapter,
                Some(session),
                ingress.turn.as_deref(),
            )?;
            validate_active(repo, &identity)?;
            return Ok((task, identity));
        }
        if stop || ingress.native_turn || ingress.turn.is_none() && !state.entries.is_empty() {
            bail!("unknown hook correlation for host session/turn");
        }
        let task = ingress
            .task
            .clone()
            .unwrap_or_else(|| "current repository state".into());
        let identity = LifecycleIdentity::resolve(
            &vault.project_id,
            &task,
            adapter,
            Some(session),
            ingress.turn.as_deref(),
        )?;
        return Ok((task, identity));
    }
    let task = ingress
        .task
        .clone()
        .unwrap_or_else(|| "current repository state".into());
    let identity = LifecycleIdentity::resolve(
        &vault.project_id,
        &task,
        adapter,
        Some(session),
        ingress.turn.as_deref(),
    )?;
    // Only proven completed rows can be pruned. Pending, missing, interrupted,
    // blocked, and live rows stay. If no room remains, fail before any write.
    let mut retained = Vec::new();
    for entry in state.entries {
        let old = entry.identity.checked(&entry.canonical_task)?;
        let completed = indexed_status(repo, &old)?.as_deref() == Some("completed");
        let age =
            Local::now().timestamp() - DateTime::parse_from_rfc3339(&entry.created_at)?.timestamp();
        // Id-less session mappings must remain as ambiguity tombstones; pruning
        // one would let a delayed Stop select a subsequent prompt in that
        // session. The bounded map fails closed when such tombstones fill it.
        if completed && entry.host_turn_id.is_some() && age > COMPLETED_RETENTION_SECONDS {
            continue;
        }
        retained.push(entry);
    }
    // Reclaim completed rows anywhere in the map when capacity is reached.
    if retained.len() >= MAX_ENTRIES {
        let mut removable = Vec::new();
        for (index, entry) in retained.iter().enumerate() {
            if entry.host_turn_id.is_some()
                && indexed_status(repo, &entry.identity.checked(&entry.canonical_task)?)?.as_deref()
                    == Some("completed")
            {
                removable.push(index);
            }
        }
        if let Some(index) = removable.first() {
            retained.remove(*index);
        }
    }
    if retained.len() >= MAX_ENTRIES {
        bail!("hook correlation capacity reached; live and pending mappings cannot be pruned");
    }
    retained.push(Entry {
        host_session_id: session.into(),
        host_turn_id: ingress.turn.clone(),
        canonical_task: task.clone(),
        identity: StoredIdentity::from_identity(&identity),
        created_at: now(),
    });
    state.entries = retained;
    let content = serde_json::to_string_pretty(&state)?;
    if content.len() + 1 > MAX_STATE_BYTES {
        bail!("hook correlation publication exceeds byte bound");
    }
    replace_text(repo.join(MAP_PATH), &(content + "\n"))?;
    Ok((task, identity))
}

pub(super) fn verify_stop_locked(
    repo: &Path,
    vault: &VaultContext,
    adapter: SupportedAdapter,
    ingress: &Ingress,
    expected: &LifecycleIdentity,
) -> Result<()> {
    let (_, identity) = resolve_locked(repo, vault, adapter, AutomationEvent::Stop, ingress)?;
    if identity != *expected {
        bail!("hook correlation identity changed during delivery");
    }
    Ok(())
}

pub(super) fn failure(error: &anyhow::Error) -> String {
    format!("unsafe hook correlation: {error:#}")
}

pub(super) fn blocked(
    error: &anyhow::Error,
    adapter: HookAdapter,
    loop_observation: bool,
) -> Result<String> {
    // Historical session-only Stop retries without any correlation are purely
    // advisory loop observations. They authorize no completion or mutation.
    let mut response = serde_json::json!({"completed":false,"baron":{"adapter":adapter_name(adapter),"hook_failure":"hard","message":failure(error),"reconciliation_passed":false}});
    if loop_observation {
        response["continue"] = Value::Bool(true);
    } else {
        response["decision"] = Value::String("block".into());
        response["reason"] = Value::String(failure(error));
    }
    Ok(serde_json::to_string(&response)?)
}
