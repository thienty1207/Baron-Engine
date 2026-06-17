use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Local, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::context::{compile_context, ContextTarget};
use crate::continuity::record_continuity_checkpoint;
use crate::proof::latest_proof;
use crate::trace::latest_trace_score;
use crate::vault::VaultContext;

const CHECKPOINT_INTERVAL_SECONDS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationEvent {
    SessionStart,
    Prompt,
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
    Agent,
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
    adapter: HookAdapter,
    session_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AutomationState {
    last_checkpoint: Option<String>,
}

pub fn handle_hook(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    adapter: HookAdapter,
    event: AutomationEvent,
    payload: &str,
) -> Result<String> {
    let repo_root = repo_root.as_ref();
    let payload: Value = if payload.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(payload).context("Could not parse native hook payload")?
    };
    let session_id = payload
        .get("session_id")
        .or_else(|| payload.get("sessionId"))
        .and_then(Value::as_str)
        .map(str::to_string);

    if event != AutomationEvent::Checkpoint || checkpoint_due(repo_root)? {
        append_journal(
            vault,
            JournalEntry {
                timestamp: now(),
                event,
                adapter,
                session_id,
            },
        )?;
        record_continuity_checkpoint(
            repo_root,
            vault,
            &format!("{} hook observed.", event_name(event)),
            adapter_name(adapter),
        )?;
    }

    match event {
        AutomationEvent::SessionStart => {
            let context = compile_context(
                repo_root,
                &vault.vault_root,
                match adapter {
                    HookAdapter::Codex => ContextTarget::Codex,
                    HookAdapter::Claude => ContextTarget::Claude,
                    HookAdapter::Agent => ContextTarget::Generic,
                },
            )?;
            Ok(serde_json::to_string(&json!({
                "continue": true,
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": context
                }
            }))?)
        }
        AutomationEvent::Stop => {
            let stop_hook_active = payload
                .get("stop_hook_active")
                .or_else(|| payload.get("stopHookActive"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let report = reconcile(repo_root)?;
            if !report.passed && !stop_hook_active {
                Ok(serde_json::to_string(&json!({
                    "decision": "block",
                    "reason": format!(
                        "Baron completion gate is not satisfied: {}. Record the missing evidence or interrupt the active plan before ending.",
                        report.gaps.join("; ")
                    )
                }))?)
            } else {
                Ok(serde_json::to_string(&json!({
                    "continue": true,
                    "systemMessage": if report.passed {
                        "Baron reconciliation passed."
                    } else {
                        "Baron reconciliation already requested once; avoid a hook loop and preserve the active state."
                    }
                }))?)
            }
        }
        _ => Ok(serde_json::to_string(&json!({"continue": true}))?),
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
            adapter,
            session_id: None,
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
            .map(|entry| event_name(entry.event))
            .unwrap_or("none"),
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
    let path = journal_path(vault);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    content.push_str(&serde_json::to_string(&entry)?);
    content.push('\n');
    fs::write(&path, content)?;
    Ok(())
}

fn journal_path(vault: &VaultContext) -> std::path::PathBuf {
    vault
        .project_root
        .join("Artifacts/automation-journal.jsonl")
}

fn checkpoint_due(repo_root: &Path) -> Result<bool> {
    let path = repo_root.join(".baron/cache/automation-state.json");
    let state = fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<AutomationState>(&content).ok())
        .unwrap_or_default();
    let due = state
        .last_checkpoint
        .as_deref()
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| {
            Utc::now()
                .signed_duration_since(value.with_timezone(&Utc))
                .num_seconds()
        })
        .map(|seconds| seconds >= CHECKPOINT_INTERVAL_SECONDS)
        .unwrap_or(true);
    if due {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            path,
            serde_json::to_string_pretty(&AutomationState {
                last_checkpoint: Some(now()),
            })?,
        )?;
    }
    Ok(due)
}

fn event_name(event: AutomationEvent) -> &'static str {
    match event {
        AutomationEvent::SessionStart => "session_start",
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

fn adapter_name(adapter: HookAdapter) -> &'static str {
    match adapter {
        HookAdapter::Codex => "codex",
        HookAdapter::Claude => "claude",
        HookAdapter::Agent => "agent",
    }
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
