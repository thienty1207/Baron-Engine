use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use baron_core::automation::{
    automation_status, handle_hook, record_lifecycle_event_for_operation, AutomationEvent,
    HookAdapter,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::operation::{OperationContext, SupportedAdapter};
use baron_core::prepare::{prepare, PrepareRequestV1};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn project(
    adapter: AdapterKind,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    baron_core::vault::VaultContext,
) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, adapter, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    (temp, repo, context)
}

fn journal(context: &baron_core::vault::VaultContext) -> String {
    fs::read_to_string(
        context
            .project_root
            .join("Artifacts/automation-journal.jsonl"),
    )
    .unwrap_or_default()
}

fn json(response: &str) -> serde_json::Value {
    serde_json::from_str(response).unwrap()
}

#[test]
fn session_start_is_bounded_and_idempotent() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let payload = r#"{"session_id":"session-start","request_id":"start-1"}"#;

    let first = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::SessionStart,
        payload,
    )
    .unwrap();
    let first_journal = journal(&vault);
    let first_checkpoint = fs::read(repo.join("docs/baron/continuity/CURRENT.md")).unwrap();

    let second = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::SessionStart,
        payload,
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first_journal, journal(&vault));
    assert_eq!(
        first_checkpoint,
        fs::read(repo.join("docs/baron/continuity/CURRENT.md")).unwrap()
    );
    let response = json(&first);
    let context = response["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        context.len() <= 6_000,
        "SessionStart context was not bounded"
    );
    assert!(context.contains("Protected Task State") || context.contains("Baron Context"));
    assert_eq!(first_journal.lines().count(), 1);
}

#[test]
fn user_prompt_submit_uses_structured_prepare_and_deduplicates() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let task = "quotes \"unicode tiếng Việt\"\n$(do-not-run) && JSON {\"x\":1}";
    let payload = serde_json::json!({
        "session_id": "prompt-session",
        "request_id": "prompt-request",
        "task": task,
        "cwd": repo,
    })
    .to_string();

    let first = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &payload,
    )
    .unwrap();
    let first_response = json(&first);
    let first_task_id = first_response["baron"]["task_id"].as_str().unwrap();
    let first_journal = journal(&vault);

    let second = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &payload,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first_journal, journal(&vault));
    assert_eq!(first_journal.lines().count(), 1);
    assert!(
        first_response["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .len()
            <= 6_000
    );

    let fallback = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: task.to_string(),
            session_id: Some("prompt-session".to_string()),
            request_id: Some("prompt-request".to_string()),
        },
        "codex",
        &repo,
        Some(vault.vault_root.clone()),
    )
    .unwrap();
    assert_eq!(fallback.task.id, first_task_id);
    assert_eq!(fallback.project_id, vault.project_id);
    assert_eq!(fallback.adapter, "codex");

    let different_task = serde_json::json!({
        "session_id": "prompt-session",
        "request_id": "prompt-request-2",
        "task": "a different prompt in the same session"
    })
    .to_string();
    let different = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &different_task,
    )
    .unwrap();
    assert_ne!(
        first_task_id,
        json(&different)["baron"]["task_id"].as_str().unwrap()
    );
    assert_eq!(journal(&vault).lines().count(), 2);
}

#[test]
fn legacy_prompt_and_checkpoint_names_normalize_to_canonical_events() {
    let (_temp, repo, vault) = project(AdapterKind::Claude);
    handle_hook(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Prompt,
        r#"{"session_id":"legacy","request_id":"prompt"}"#,
    )
    .unwrap();
    handle_hook(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::UserPromptSubmit,
        r#"{"session_id":"legacy","request_id":"prompt","task":"current repository state"}"#,
    )
    .unwrap();
    handle_hook(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Checkpoint,
        r#"{"session_id":"legacy","request_id":"compact"}"#,
    )
    .unwrap();

    let content = journal(&vault);
    assert!(content.contains("\"event_kind\":\"user_prompt_submit\""));
    assert!(content.contains("\"event_kind\":\"pre_compact\""));
}

#[test]
fn precompact_is_cheap_durable_and_byte_stable_on_retry() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let payload = r#"{"session_id":"compact-session","request_id":"compact-1"}"#;
    let first = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        payload,
    )
    .unwrap();
    let checkpoint = repo.join("docs/baron/continuity/CURRENT.md");
    let first_checkpoint = fs::read(&checkpoint).unwrap();
    let first_journal = journal(&vault);
    assert!(
        !vault.index_path.exists(),
        "PreCompact performed a full memory index"
    );

    let second = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        payload,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(first_checkpoint, fs::read(&checkpoint).unwrap());
    assert_eq!(first_journal, journal(&vault));
}

#[test]
fn stop_is_not_completion_and_retries_are_idempotent() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::write(
        repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: active\n- Status: `in_progress`\n- Next action: verify\n",
    )
    .unwrap();
    let payload = r#"{"session_id":"stop-session","request_id":"stop-1","stop_hook_active":false}"#;
    let first = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        payload,
    )
    .unwrap();
    let response = json(&first);
    assert_eq!(response["decision"], "block");
    assert_ne!(response["completed"], true);
    let before_retry = journal(&vault);
    let second = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        payload,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(before_retry, journal(&vault));

    let loop_break = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        r#"{"session_id":"stop-session","request_id":"stop-1","stop_hook_active":true}"#,
    )
    .unwrap();
    assert!(loop_break.contains("continue"));
    assert_eq!(journal(&vault).lines().count(), 2);
}

#[test]
fn child_events_record_bounded_evidence_without_parent_lifecycle_mutation() {
    let (_temp, repo, vault) = project(AdapterKind::Claude);
    let continuity = repo.join("docs/baron/continuity/CURRENT.md");
    let payload = serde_json::json!({
        "is_child": true,
        "child_id": "child-1",
        "parent_session_id": "parent-session",
        "parent_task_id": "task-parent",
        "session_id": "child-session",
        "request_id": "child-request",
        "task": "child review",
        "evidence": "bounded child finding"
    })
    .to_string();

    let response = handle_hook(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::UserPromptSubmit,
        &payload,
    )
    .unwrap();
    let value = json(&response);
    assert_eq!(value["baron"]["child"], true);
    assert!(value["hookSpecificOutput"].is_null());
    assert!(!continuity.exists());
    let content = journal(&vault);
    assert!(content.contains("\"child\":true"));
    assert!(content.contains("bounded child finding"));

    let retry = handle_hook(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::UserPromptSubmit,
        &payload,
    )
    .unwrap();
    assert_eq!(response, retry);
    assert_eq!(journal(&vault).lines().count(), 1);
}

#[test]
fn recursion_guard_does_not_enter_core_lifecycle() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let payload = r#"{"session_id":"recursive","request_id":"recursive-1","task":"should not prepare","baron_recursion_depth":1}"#;
    let response = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        payload,
    )
    .unwrap();
    let value = json(&response);
    assert_eq!(value["baron"]["recursion_guard"], true);
    assert_eq!(journal(&vault), "");
    assert!(!repo.join("docs/baron/continuity/CURRENT.md").exists());
}

#[test]
fn concurrent_cross_adapter_events_and_restart_like_retry_are_scoped() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let barrier = Arc::new(Barrier::new(2));
    let codex_repo = repo.clone();
    let codex_vault = vault.clone();
    let codex_barrier = Arc::clone(&barrier);
    let claude_repo = repo.clone();
    let claude_vault = vault.clone();
    let claude_barrier = Arc::clone(&barrier);
    let codex = thread::spawn(move || {
        codex_barrier.wait();
        handle_hook(
            &codex_repo,
            &codex_vault,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            r#"{"session_id":"codex-a","request_id":"same-request"}"#,
        )
        .unwrap()
    });
    let claude = thread::spawn(move || {
        claude_barrier.wait();
        handle_hook(
            &claude_repo,
            &claude_vault,
            HookAdapter::Claude,
            AutomationEvent::PreCompact,
            r#"{"session_id":"claude-b","request_id":"same-request"}"#,
        )
        .unwrap()
    });
    codex.join().unwrap();
    claude.join().unwrap();
    let content = journal(&vault);
    assert_eq!(content.lines().count(), 2);
    assert!(content.contains("\"adapter\":\"codex\""));
    assert!(content.contains("\"adapter\":\"claude\""));
    assert!(content.contains("codex-a"));
    assert!(content.contains("claude-b"));

    let before = content;
    handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        r#"{"session_id":"codex-a","request_id":"same-request"}"#,
    )
    .unwrap();
    assert_eq!(before, journal(&vault));
}

#[test]
fn concurrent_internal_event_append_is_lossless_and_dedup_retention_is_bounded() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let barrier = Arc::new(Barrier::new(12));
    let mut workers = Vec::new();
    for index in 0..12 {
        let barrier = Arc::clone(&barrier);
        let vault = vault.clone();
        workers.push(thread::spawn(move || {
            barrier.wait();
            let adapter = if index % 2 == 0 {
                SupportedAdapter::Codex
            } else {
                SupportedAdapter::Claude
            };
            let operation = OperationContext::new(adapter)
                .with_session_id(format!("append-session-{index}"))
                .with_request_id(format!("append-request-{index}"));
            record_lifecycle_event_for_operation(&vault, &operation, AutomationEvent::Stop)
                .unwrap();
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(journal(&vault).lines().count(), 12);
    assert!(automation_status(&repo, &vault)
        .unwrap()
        .contains("Events recorded: 12"));

    for index in 0..280 {
        handle_hook(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            &format!("{{\"session_id\":\"retention\",\"request_id\":\"{index}\"}}"),
        )
        .unwrap();
    }
    let dedup = fs::read_to_string(repo.join(".baron/cache/automation-dedup.json")).unwrap();
    let dedup: serde_json::Value = serde_json::from_str(&dedup).unwrap();
    assert!(dedup["entries"].as_array().unwrap().len() <= 256);
}
