use std::fs;

use baron_core::automation::{
    automation_status, handle_hook, reconcile, AutomationEvent, HookAdapter,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::plan::start_or_resume_plan;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

#[test]
fn session_start_injects_context_and_records_an_observable_event() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let response = handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::SessionStart,
        r#"{"session_id":"session-1","cwd":"demo"}"#,
    )
    .unwrap();
    let status = automation_status(&repo, &context).unwrap();

    assert!(response.contains("Baron Context"));
    assert!(response.contains("additionalContext"));
    assert!(status.contains("session_start"));
    assert!(context
        .project_root
        .join("Artifacts/automation-journal.jsonl")
        .exists());
}

#[test]
fn repeated_checkpoint_events_are_throttled() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    for _ in 0..2 {
        handle_hook(
            &repo,
            &context,
            HookAdapter::Codex,
            AutomationEvent::Checkpoint,
            r#"{"session_id":"session-1"}"#,
        )
        .unwrap();
    }

    let journal = fs::read_to_string(
        context
            .project_root
            .join("Artifacts/automation-journal.jsonl"),
    )
    .unwrap();
    assert_eq!(journal.matches("\"event\":\"checkpoint\"").count(), 1);
}

#[test]
fn stop_reconciliation_blocks_once_when_active_work_lacks_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "backend login auth").unwrap();

    let report = reconcile(&repo).unwrap();
    let first = handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        r#"{"session_id":"session-1","stop_hook_active":false}"#,
    )
    .unwrap();
    let second = handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        r#"{"session_id":"session-1","stop_hook_active":true}"#,
    )
    .unwrap();

    assert!(!report.passed);
    assert!(report.gaps.iter().any(|gap| gap.contains("proof")));
    assert!(report.gaps.iter().any(|gap| gap.contains("trace")));
    assert!(first.contains(r#""decision":"block""#));
    assert!(!second.contains(r#""decision":"block""#));
}
