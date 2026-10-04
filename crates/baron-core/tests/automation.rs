use std::fs;

use baron_core::automation::{
    automation_status, handle_hook, reconcile, reconcile_for_operation, AutomationEvent,
    HookAdapter,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::{
    complete_plan_for_identity, start_or_resume_plan, start_or_resume_plan_for_identity,
};
use baron_core::proof::record_proof_for_operation;
use baron_core::trace::{
    record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::ensure_vault;
use serde_json::{json, Value};
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
            r#"{"session_id":"session-1","request_id":"checkpoint-1"}"#,
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
fn historical_journal_adapter_values_remain_readable_as_opaque_data() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let historical_value: String = ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect();
    fs::create_dir_all(context.project_root.join("Artifacts")).unwrap();
    fs::write(
        context
            .project_root
            .join("Artifacts/automation-journal.jsonl"),
        format!(
            "{{\"timestamp\":\"2026-09-08T00:00:00Z\",\"event\":\"prompt\",\"adapter\":\"{historical_value}\",\"session_id\":null}}\n"
        ),
    )
    .unwrap();

    let status = automation_status(&repo, &context).unwrap();
    assert!(status.contains("prompt"));
    assert!(context
        .project_root
        .join("Artifacts/automation-journal.jsonl")
        .starts_with(&context.project_root));
}

#[test]
fn stop_retry_without_operation_correlation_fails_closed() {
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
    assert!(second.contains(r#""decision":"block""#), "{second}");
    assert!(!second.contains(r#""continue":true""#));
    assert!(!repo.join(".baron/state/hook-correlations.json").exists());
}

#[test]
fn unknown_stop_retry_without_an_active_plan_fails_closed_for_both_hosts() {
    for adapter in [HookAdapter::Codex, HookAdapter::Claude] {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("demo");
        let vault = temp.path().join("Vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();

        for payload in [
            json!({"session_id":"unknown-session","stop_hook_active":true}),
            json!({"session_id":"unknown-session","request_id":"unknown-turn","stop_hook_active":true}),
        ] {
            let response: Value = serde_json::from_str(
                &handle_hook(
                    &repo,
                    &context,
                    adapter,
                    AutomationEvent::Stop,
                    &payload.to_string(),
                )
                .unwrap(),
            )
            .unwrap();
            assert_eq!(response["decision"], "block", "{response}");
            assert_eq!(response["completed"], false);
            assert_eq!(response["baron"]["reconciliation_passed"], false);
            assert_ne!(response["continue"], true);
            assert!(!repo.join(".baron/state/hook-correlations.json").exists());
            assert!(!repo.join(".baron/cache/automation-dedup.json").exists());
            assert!(!context
                .project_root
                .join("Artifacts/automation-journal.jsonl")
                .exists());
        }
    }
}

#[test]
fn identified_stop_retry_blocks_failed_a_when_current_b_has_passing_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let a_title = "fix README alpha typo";
    let b_title = "fix README beta typo";
    let a = LifecycleIdentity::resolve(
        &context.project_id,
        a_title,
        SupportedAdapter::Codex,
        Some("retry-alpha-session"),
        Some("retry-alpha-turn"),
    )
    .unwrap();
    handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &json!({"session_id":a.session_id(),"turn_id":a.request_id(),"prompt":a_title}).to_string(),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, a_title, &a).unwrap();
    let b = LifecycleIdentity::resolve(
        &context.project_id,
        b_title,
        SupportedAdapter::Claude,
        Some("retry-beta-session"),
        Some("retry-beta-prompt"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, b_title, &b).unwrap();
    let operation_b = OperationContext::from_identity(&b);
    let proof =
        record_proof_for_operation(&repo, &context, &operation_b, "Beta verification passed")
            .unwrap();
    let binding = TraceOperationBinding::from_operation(&operation_b, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Beta README task verified",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );
    assert!(reconcile_for_operation(&repo, &context, &b).unwrap().passed);
    assert!(!reconcile_for_operation(&repo, &context, &a).unwrap().passed);
    let current_path = repo.join("docs/baron/plans/CURRENT.md");
    let current_b = fs::read(&current_path).unwrap();

    for stop_hook_active in [false, true] {
        let stop: Value = serde_json::from_str(
            &handle_hook(
                &repo,
                &context,
                HookAdapter::Codex,
                AutomationEvent::Stop,
                &json!({"session_id":a.session_id(),"turn_id":a.request_id(),"stop_hook_active":stop_hook_active})
                    .to_string(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(stop["baron"]["operation_id"], a.operation_id());
        assert_eq!(stop["baron"]["task_id"], a.task_id());
        assert_eq!(stop["baron"]["session_id"], a.session_id());
        assert_eq!(stop["baron"]["request_id"], a.request_id());
        assert_eq!(stop["decision"], "block", "{stop}");
        assert_eq!(stop["baron"]["reconciliation_passed"], false);
        assert_ne!(stop["continue"], true);
        assert_eq!(current_b, fs::read(&current_path).unwrap());
    }
}

#[test]
fn reconcile_and_stop_do_not_use_newer_unrelated_operation_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let active = LifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("active-session"),
        Some("active-request"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "fix README typo", &active).unwrap();

    let unrelated = LifecycleIdentity::resolve(
        &context.project_id,
        "unrelated docs task",
        SupportedAdapter::Claude,
        Some("unrelated-session"),
        Some("unrelated-request"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, "unrelated docs task", &unrelated).unwrap();
    let unrelated_operation = OperationContext::from_identity(&unrelated);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &unrelated_operation,
        "Unrelated verification passed",
    )
    .unwrap();
    let binding = TraceOperationBinding::from_operation(&unrelated_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Unrelated docs task completed",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let report = reconcile_for_operation(&repo, &context, &active).unwrap();
    assert!(!report.passed);
    assert!(report
        .gaps
        .iter()
        .any(|gap| gap.contains("proof") || gap.contains("binding")));

    let stop = handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        r#"{"task":"fix README typo","session_id":"active-session","request_id":"active-request","stop_hook_active":false}"#,
    )
    .unwrap();
    assert!(stop.contains(r#""decision":"block""#));
}

#[test]
fn identified_stop_reconciles_the_stop_operation_not_current() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first_title = "fix README alpha typo";
    let second_title = "backend login security";
    let first = LifecycleIdentity::resolve(
        &context.project_id,
        first_title,
        SupportedAdapter::Codex,
        Some("stop-alpha-session"),
        Some("stop-alpha-request"),
    )
    .unwrap();
    let second = LifecycleIdentity::resolve(
        &context.project_id,
        second_title,
        SupportedAdapter::Claude,
        Some("stop-beta-session"),
        Some("stop-beta-request"),
    )
    .unwrap();
    handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        &json!({"session_id":first.session_id(),"turn_id":first.request_id(),"prompt":first_title})
            .to_string(),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &context, first_title, &first).unwrap();
    start_or_resume_plan_for_identity(&repo, &context, second_title, &second).unwrap();

    let first_operation = OperationContext::from_identity(&first);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &first_operation,
        "Alpha README verification passed",
    )
    .unwrap();
    let binding = TraceOperationBinding::from_operation(&first_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "Alpha README task completed",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let stop_a = handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::Stop,
        &format!(
            r#"{{"task":"{first_title}","session_id":"{}","request_id":"{}","stop_hook_active":false}}"#,
            first.session_id(),
            first.request_id()
        ),
    )
    .unwrap();
    assert!(stop_a.contains(r#""reconciliation_passed":true"#));
    assert!(!stop_a.contains(r#""decision":"block""#));

    // Move the presentation pointer to A. B must still be evaluated from its
    // own ACTIVE entry, so A's passing evidence cannot satisfy Stop(B).
    start_or_resume_plan_for_identity(&repo, &context, first_title, &first).unwrap();
    let stop_b = handle_hook(
        &repo,
        &context,
        HookAdapter::Claude,
        AutomationEvent::Stop,
        &format!(
            r#"{{"task":"{second_title}","session_id":"{}","request_id":"{}","stop_hook_active":false}}"#,
            second.session_id(),
            second.request_id()
        ),
    )
    .unwrap();
    assert!(stop_b.contains(r#""decision":"block""#));

    complete_plan_for_identity(&repo, &context, "Alpha README verified", &first).unwrap();
    let completed_retry: Value = serde_json::from_str(
        &handle_hook(
            &repo,
            &context,
            HookAdapter::Codex,
            AutomationEvent::Stop,
            &json!({"session_id":first.session_id(),"request_id":first.request_id(),"stop_hook_active":true})
                .to_string(),
        )
        .unwrap(),
    )
    .unwrap();
    assert_ne!(completed_retry["decision"], "block", "{completed_retry}");
    assert_eq!(completed_retry["continue"], true);
    assert_eq!(
        completed_retry["baron"]["operation_already_completed"],
        true
    );
    assert_eq!(
        completed_retry["baron"]["operation_id"],
        first.operation_id()
    );
}
