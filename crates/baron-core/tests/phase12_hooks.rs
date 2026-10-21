use std::fs;
use std::process::Command;
use std::sync::{Arc, Barrier};
use std::thread;

use baron_core::automation::{
    automation_status, handle_hook, record_lifecycle_event_for_operation, AutomationEvent,
    HookAdapter,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::start_or_resume_plan_for_identity;
use baron_core::prepare::{prepare, PrepareRequestV1};
use baron_core::vault::{ensure_vault, vault_context_without_create};
use tempfile::tempdir;

const REPLAY_REPO_ENV: &str = "BARON_TEST_HOOK_REPLAY_REPO";
const REPLAY_VAULT_ENV: &str = "BARON_TEST_HOOK_REPLAY_VAULT";
const REPLAY_PAYLOAD_ENV: &str = "BARON_TEST_HOOK_REPLAY_PAYLOAD";
const REPLAY_EVENT_KEY_ENV: &str = "BARON_TEST_HOOK_REPLAY_EVENT_KEY";
const REPLAY_EVENT_ENV: &str = "BARON_TEST_HOOK_REPLAY_EVENT";

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
fn anonymous_hook_deliveries_are_intentionally_non_idempotent_without_delivery_id() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let payload = r#"{"task":"same anonymous lifecycle task"}"#;

    let first = json(
        &handle_hook(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            payload,
        )
        .unwrap(),
    );
    let second = json(
        &handle_hook(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            payload,
        )
        .unwrap(),
    );

    assert_eq!(first["baron"]["task_id"], second["baron"]["task_id"]);
    assert!(first["baron"]["operation_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(first["baron"]["session_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(first["baron"]["request_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_ne!(
        first["baron"]["operation_id"],
        second["baron"]["operation_id"]
    );
    assert_ne!(first["baron"]["session_id"], second["baron"]["session_id"]);
    assert_ne!(first["baron"]["request_id"], second["baron"]["request_id"]);
    assert_eq!(journal(&vault).lines().count(), 2);

    let packet = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: "same anonymous lifecycle task".into(),
            session_id: first["baron"]["session_id"].as_str().map(str::to_string),
            request_id: first["baron"]["request_id"].as_str().map(str::to_string),
        },
        "codex",
        &repo,
        Some(vault.vault_root.clone()),
    )
    .unwrap();
    assert_eq!(packet.task.id, first["baron"]["task_id"]);
    assert_eq!(
        packet.operation_id.as_deref(),
        first["baron"]["operation_id"].as_str()
    );
}

#[test]
fn identified_hook_retry_is_idempotent_but_new_request_is_distinct() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let first_payload = serde_json::json!({
        "task": "same identified lifecycle task",
        "session_id": "identified-session",
        "request_id": "identified-request-a"
    })
    .to_string();
    let first = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        &first_payload,
    )
    .unwrap();
    let retry = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        &first_payload,
    )
    .unwrap();
    assert_eq!(first, retry);
    assert_eq!(journal(&vault).lines().count(), 1);

    let second_payload = serde_json::json!({
        "task": "same identified lifecycle task",
        "session_id": "identified-session",
        "request_id": "identified-request-b"
    })
    .to_string();
    let second = json(
        &handle_hook(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            &second_payload,
        )
        .unwrap(),
    );
    assert_eq!(second["baron"]["task_id"], json(&first)["baron"]["task_id"]);
    assert_ne!(
        second["baron"]["operation_id"],
        json(&first)["baron"]["operation_id"]
    );
    assert_ne!(
        second["baron"]["event_key"],
        json(&first)["baron"]["event_key"]
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
    let task = "backend login security";
    handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        r#"{"session_id":"stop-session","turn_id":"stop-1","prompt":"backend login security"}"#,
    )
    .unwrap();
    let identity = LifecycleIdentity::resolve(
        &vault.project_id,
        task,
        SupportedAdapter::Codex,
        Some("stop-session"),
        Some("stop-1"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, task, &identity).unwrap();
    let payload = r#"{"session_id":"stop-session","turn_id":"stop-1","stop_hook_active":false}"#;
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

    let retry = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        r#"{"session_id":"stop-session","turn_id":"stop-1","stop_hook_active":true}"#,
    )
    .unwrap();
    // The host's retry flag cannot waive failed operation reconciliation.
    let retry = json(&retry);
    assert_eq!(retry["decision"], "block");
    assert_eq!(retry["completed"], false);
    assert_eq!(retry["baron"]["reconciliation_passed"], false);
    assert_eq!(retry["baron"]["operation_id"], identity.operation_id());
    assert_ne!(retry["continue"], true);
    assert_eq!(journal(&vault).lines().count(), 3);
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
    assert_eq!(value["baron"]["session_id"], "child-session");
    assert_eq!(value["baron"]["request_id"], "child-request");
    assert!(value["baron"]["operation_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
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
    assert_eq!(value["baron"]["session_id"], "recursive");
    assert_eq!(value["baron"]["request_id"], "recursive-1");
    assert!(value["baron"]["operation_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
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

    let first_payload = r#"{"session_id":"retention","request_id":"0"}"#;
    let first_response = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        first_payload,
    )
    .unwrap();
    let first_value = json(&first_response);
    let first_event_key = first_value["baron"]["event_key"]
        .as_str()
        .unwrap()
        .to_string();
    let first_operation = first_value["baron"]["operation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let first_checkpoint_dir = repo
        .join("docs/baron/continuity/operations")
        .join(&first_operation)
        .join("checkpoints");
    let first_checkpoint = fs::read_dir(&first_checkpoint_dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let first_checkpoint_before = fs::read(&first_checkpoint).unwrap();

    for index in 1..280 {
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
    assert!(!dedup["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["key"] == first_event_key));

    let journal_before_retry = journal(&vault);
    assert_eq!(
        journal_event_count(&journal_before_retry, &first_event_key),
        1
    );
    let replay = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        first_payload,
    )
    .unwrap();
    assert_eq!(replay, first_response);
    assert_eq!(journal(&vault), journal_before_retry);
    assert_eq!(
        fs::read(&first_checkpoint).unwrap(),
        first_checkpoint_before
    );
}

#[test]
fn malformed_journal_tail_blocks_hook_before_publication() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let journal_path = vault
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    let malformed = b"{not-valid-json}\n";
    fs::write(&journal_path, malformed).unwrap();
    let dedup_path = repo.join(".baron/cache/automation-dedup.json");
    let correlation_path = repo.join(".baron/state/hook-correlations.json");
    let continuity_path = repo.join("docs/baron/continuity/CURRENT.md");
    let operation_checkpoints = repo.join("docs/baron/continuity/operations");

    let result = handle_hook(
        &repo,
        &vault,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        r#"{"session_id":"malformed-tail-session","request_id":"malformed-tail-request","task":"fix README typo"}"#,
    );

    assert!(
        result.is_err(),
        "malformed replay evidence must fail closed"
    );
    let error = result.unwrap_err();
    assert!(
        format!("{error:#}").contains("Malformed automation journal JSONL row"),
        "unexpected malformed journal error: {error:#}"
    );
    assert_eq!(fs::read(&journal_path).unwrap(), malformed);
    assert!(
        !dedup_path.exists(),
        "failed lookup must not claim or cache an event"
    );
    assert!(
        !correlation_path.exists(),
        "failed lookup must precede host turn correlation publication"
    );
    assert!(
        !continuity_path.exists(),
        "failed lookup must not publish continuity"
    );
    assert!(
        !operation_checkpoints.exists(),
        "failed lookup must not create an operation checkpoint"
    );
}

#[test]
fn hook_replay_after_both_finite_evidence_windows_expires_across_restart() {
    exercise_hook_replay_after_both_finite_evidence_windows_expire(false);
}

#[test]
fn stop_replay_after_both_finite_evidence_windows_rechecks_authority() {
    exercise_hook_replay_after_both_finite_evidence_windows_expire(true);
}

fn exercise_hook_replay_after_both_finite_evidence_windows_expire(stop: bool) {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let (event, target_payload) = if stop {
        let task = "backend login security";
        let prompt_payload = serde_json::json!({
            "session_id": "retention-stop-session",
            "turn_id": "retention-stop-turn",
            "prompt": task
        })
        .to_string();
        handle_hook(
            &repo,
            &vault,
            HookAdapter::Codex,
            AutomationEvent::UserPromptSubmit,
            &prompt_payload,
        )
        .unwrap();
        let identity = LifecycleIdentity::resolve(
            &vault.project_id,
            task,
            SupportedAdapter::Codex,
            Some("retention-stop-session"),
            Some("retention-stop-turn"),
        )
        .unwrap();
        start_or_resume_plan_for_identity(&repo, &vault, task, &identity).unwrap();
        (
            AutomationEvent::Stop,
            serde_json::json!({
                "session_id": "retention-stop-session",
                "turn_id": "retention-stop-turn",
                "stop_hook_active": false
            })
            .to_string(),
        )
    } else {
        (
            AutomationEvent::PreCompact,
            serde_json::json!({
                "session_id": "retention-target-session",
                "request_id": "retention-target-request",
                "task": "current repository state"
            })
            .to_string(),
        )
    };
    let first_response =
        handle_hook(&repo, &vault, HookAdapter::Codex, event, &target_payload).unwrap();
    let first_value = json(&first_response);
    if stop {
        assert_eq!(first_value["decision"], "block");
        assert_ne!(first_value["completed"], true);
    }
    let event_key = first_value["baron"]["event_key"]
        .as_str()
        .unwrap()
        .to_string();
    let operation_id = first_value["baron"]["operation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let checkpoint_dir = repo
        .join("docs/baron/continuity/operations")
        .join(&operation_id)
        .join("checkpoints");
    let checkpoint_paths = fs::read_dir(&checkpoint_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect::<Vec<_>>();
    assert!(!checkpoint_paths.is_empty());
    let checkpoint_snapshots = checkpoint_paths
        .iter()
        .map(|path| (path.clone(), fs::read(path).unwrap()))
        .collect::<Vec<_>>();
    let vault_checkpoint_snapshots = checkpoint_paths
        .iter()
        .map(|path| {
            let vault_path = vault
                .project_root
                .join("Continuity/Operations")
                .join(&operation_id)
                .join("Checkpoints")
                .join(path.file_name().unwrap());
            (vault_path.clone(), fs::read(vault_path).unwrap())
        })
        .collect::<Vec<_>>();

    seed_response_cache(&repo, "expired-response-window", 256);
    append_journal_events(&vault, "expired-journal-window", 513);
    let dedup_path = repo.join(".baron/cache/automation-dedup.json");
    let dedup: serde_json::Value = serde_json::from_slice(&fs::read(&dedup_path).unwrap()).unwrap();
    assert!(
        !dedup["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["key"] == event_key),
        "the target response must be outside the seeded 256-entry cache window"
    );

    let vault_reopened = vault_context_without_create(&vault.vault_root, &repo).unwrap();
    assert_eq!(vault_reopened.project_id, vault.project_id);
    let journal_before_out_of_window_retry = journal(&vault_reopened);
    assert_eq!(
        journal_event_count(&journal_before_out_of_window_retry, &event_key),
        1
    );
    let tail: Vec<_> = journal_before_out_of_window_retry
        .lines()
        .rev()
        .take(512)
        .collect();
    assert_eq!(tail.len(), 512);
    assert!(!tail.iter().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .is_some_and(|entry| entry["event_key"] == event_key)
    }));

    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "hook_replay_after_both_finite_evidence_windows_expire_child_process",
            "--nocapture",
        ])
        .env(REPLAY_REPO_ENV, &repo)
        .env(REPLAY_VAULT_ENV, &vault.vault_root)
        .env(REPLAY_PAYLOAD_ENV, &target_payload)
        .env(REPLAY_EVENT_KEY_ENV, &event_key)
        .env(REPLAY_EVENT_ENV, if stop { "stop" } else { "pre_compact" })
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child replay failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let journal_after_out_of_window_retry = journal(&vault_reopened);
    assert_eq!(
        journal_event_count(&journal_after_out_of_window_retry, &event_key),
        2,
        "outside both finite evidence windows the hook may publish another journal row"
    );
    assert_eq!(
        journal_after_out_of_window_retry.lines().count(),
        journal_before_out_of_window_retry.lines().count() + 1
    );
    for (path, bytes) in &checkpoint_snapshots {
        assert_eq!(fs::read(path).unwrap(), *bytes);
    }
    for (path, bytes) in &vault_checkpoint_snapshots {
        assert_eq!(fs::read(path).unwrap(), *bytes);
    }
    assert_eq!(
        fs::read_dir(&checkpoint_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
            .count(),
        checkpoint_snapshots.len(),
        "continuity checkpoint set remains immutable after replay"
    );
}

#[test]
fn hook_replay_after_both_finite_evidence_windows_expire_child_process() {
    let (Ok(repo), Ok(vault_root), Ok(payload), Ok(expected_event_key)) = (
        std::env::var(REPLAY_REPO_ENV),
        std::env::var(REPLAY_VAULT_ENV),
        std::env::var(REPLAY_PAYLOAD_ENV),
        std::env::var(REPLAY_EVENT_KEY_ENV),
    ) else {
        return;
    };
    let repo = std::path::PathBuf::from(repo);
    let vault = vault_context_without_create(vault_root, &repo).unwrap();
    let event = if std::env::var(REPLAY_EVENT_ENV).as_deref() == Ok("stop") {
        AutomationEvent::Stop
    } else {
        AutomationEvent::PreCompact
    };
    let response = handle_hook(&repo, &vault, HookAdapter::Codex, event, &payload).unwrap();
    let value = json(&response);
    assert_eq!(value["baron"]["event_key"], expected_event_key);
    if event == AutomationEvent::Stop {
        assert_eq!(value["decision"], "block");
        assert_ne!(value["completed"], true);
    }
}

#[test]
fn concurrent_same_event_from_distinct_checkouts_sharing_a_vault_is_published_once() {
    let (_temp, repo_a, vault_a) = project(AdapterKind::Codex);
    let repo_b = repo_a.parent().unwrap().join("second-checkout");
    fs::create_dir_all(repo_b.join(".baron")).unwrap();
    fs::copy(
        repo_a.join(".baron/project.toml"),
        repo_b.join(".baron/project.toml"),
    )
    .unwrap();
    let vault_b = vault_context_without_create(&vault_a.vault_root, &repo_b).unwrap();
    assert_eq!(vault_a.project_id, vault_b.project_id);
    assert_eq!(vault_a.project_root, vault_b.project_root);

    let payload = r#"{"session_id":"shared-replay-session","request_id":"shared-replay-request","task":"current repository state"}"#;
    let barrier = Arc::new(Barrier::new(3));
    let repo_a_worker = repo_a.clone();
    let vault_a_worker = vault_a.clone();
    let barrier_a = Arc::clone(&barrier);
    let worker_a = thread::spawn(move || {
        barrier_a.wait();
        handle_hook(
            &repo_a_worker,
            &vault_a_worker,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            payload,
        )
        .unwrap()
    });
    let repo_b_worker = repo_b.clone();
    let vault_b_worker = vault_b.clone();
    let barrier_b = Arc::clone(&barrier);
    let worker_b = thread::spawn(move || {
        barrier_b.wait();
        handle_hook(
            &repo_b_worker,
            &vault_b_worker,
            HookAdapter::Codex,
            AutomationEvent::PreCompact,
            payload,
        )
        .unwrap()
    });
    barrier.wait();
    let response_a = worker_a.join().unwrap();
    let response_b = worker_b.join().unwrap();
    let response_value_a = json(&response_a);
    let response_value_b = json(&response_b);
    let event_key = response_value_a["baron"]["event_key"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(response_value_b["baron"]["event_key"], event_key);
    assert_eq!(response_value_a["continue"], true);
    assert_eq!(response_value_b["continue"], true);
    let journal_before_retry = journal(&vault_a);
    assert_eq!(journal_event_count(&journal_before_retry, &event_key), 1);

    let retry = handle_hook(
        &repo_b,
        &vault_b,
        HookAdapter::Codex,
        AutomationEvent::PreCompact,
        payload,
    )
    .unwrap();
    assert_eq!(json(&retry)["baron"]["event_key"], event_key);
    let dedup: serde_json::Value = serde_json::from_slice(
        &fs::read(repo_b.join(".baron/cache/automation-dedup.json")).unwrap(),
    )
    .unwrap();
    assert!(dedup["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["key"] == event_key && entry["response"].is_string()));
    assert_eq!(journal(&vault_a), journal_before_retry);
}

fn journal_event_count(content: &str, event_key: &str) -> usize {
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|entry| entry["event_key"].as_str() == Some(event_key))
        .count()
}

fn seed_response_cache(repo: &std::path::Path, prefix: &str, count: usize) {
    let entries = (0..count)
        .map(|index| {
            serde_json::json!({
                "key": format!("seeded-{prefix}-{index}"),
                "response": "{\"continue\":true}",
                "claimed_at": null,
                "claim_token": null
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        repo.join(".baron/cache/automation-dedup.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "entries": entries
        }))
        .unwrap(),
    )
    .unwrap();
}

fn append_journal_events(vault: &baron_core::vault::VaultContext, prefix: &str, count: usize) {
    use std::io::Write;

    let path = vault
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    let mut lines = String::new();
    for index in 0..count {
        let event_key = format!("seeded-{prefix}-event-{index}");
        lines.push_str(
            &serde_json::json!({
                "timestamp": "2026-10-18T00:00:00+00:00",
                "event": "pre_compact",
                "adapter": "codex",
                "session_id": format!("seeded-{prefix}-session-{index}"),
                "request_id": format!("seeded-{prefix}-request-{index}"),
                "event_key": event_key,
                "event_kind": "pre_compact",
                "task_id": "seeded-task",
                "child": false,
                "response": "{\"continue\":true}"
            })
            .to_string(),
        );
        lines.push('\n');
    }
    fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();
}
