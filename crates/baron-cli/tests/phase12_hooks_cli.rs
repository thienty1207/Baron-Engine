use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn init_repo() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    (temp, repo, vault)
}

fn run_hook(repo: &std::path::Path, event: &str, payload: &str) -> Value {
    run_hook_adapter(repo, event, payload, "codex")
}

fn run_hook_adapter(repo: &std::path::Path, event: &str, payload: &str, adapter: &str) -> Value {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "automation",
            "hook",
            event,
            repo.to_str().unwrap(),
            "--adapter",
            adapter,
        ])
        .write_stdin(payload)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "hook failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn cli_native_taskless_stop_recovers_prompt_identity_across_processes_and_current_b() {
    use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
    use baron_core::plan::start_or_resume_plan_for_identity;
    use baron_core::proof::record_proof_for_operation;
    use baron_core::trace::{
        record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
    };
    use baron_core::vault::ensure_vault;

    for (adapter, supported, turn_field) in [
        ("codex", SupportedAdapter::Codex, "turn_id"),
        ("claude", SupportedAdapter::Claude, "prompt_id"),
    ] {
        let (_temp, repo, vault) = init_repo();
        let context = ensure_vault(&vault, &repo).unwrap();
        let turn = "550e8400-e29b-41d4-a716-446655440000";
        let mut prompt = serde_json::json!({"session_id":"native-session","hook_event_name":"UserPromptSubmit","transcript_path":"/workspace/transcript.jsonl","cwd":repo,"prompt":"fix README alpha typo"});
        prompt[turn_field] = serde_json::json!(turn);
        let started = run_hook_adapter(&repo, "user-prompt-submit", &prompt.to_string(), adapter);
        assert_eq!(started["baron"]["request_id"], turn);
        let a = LifecycleIdentity::resolve(
            &context.project_id,
            "fix README alpha typo",
            supported,
            Some("native-session"),
            Some(turn),
        )
        .unwrap();
        assert_eq!(started["baron"]["operation_id"], a.operation_id());
        start_or_resume_plan_for_identity(&repo, &context, "fix README alpha typo", &a).unwrap();
        let operation = OperationContext::from_identity(&a);
        let proof =
            record_proof_for_operation(&repo, &context, &operation, "README verification passed")
                .unwrap();
        let trace = record_trace_for_operation(
            &repo,
            &context,
            "README task completed",
            TraceOutcome::Completed,
            &TraceOperationBinding::from_operation(&operation, &proof.id).unwrap(),
        )
        .unwrap();
        assert!(
            score_trace(&repo, &context, Some(&trace.id))
                .unwrap()
                .passed
        );
        let b = LifecycleIdentity::resolve(
            &context.project_id,
            "backend login security",
            if supported == SupportedAdapter::Codex {
                SupportedAdapter::Claude
            } else {
                SupportedAdapter::Codex
            },
            Some("other"),
            Some("other"),
        )
        .unwrap();
        start_or_resume_plan_for_identity(&repo, &context, "backend login security", &b).unwrap();
        let mut stop = serde_json::json!({"session_id":"native-session","hook_event_name":"Stop","transcript_path":"/workspace/transcript.jsonl","cwd":repo,"stop_hook_active":false,"last_assistant_message":"Ready to verify"});
        stop[turn_field] = serde_json::json!(turn);
        let first = run_hook_adapter(&repo, "stop", &stop.to_string(), adapter);
        assert_eq!(first["baron"]["operation_id"], a.operation_id());
        assert_eq!(first["baron"]["reconciliation_passed"], true);
        assert_eq!(first["completed"], false);
        assert_ne!(first["decision"], "block");
        assert_eq!(
            first,
            run_hook_adapter(&repo, "stop", &stop.to_string(), adapter)
        );
        fs::write(
            repo.join(".baron/state/hook-correlations.json"),
            "{malformed",
        )
        .unwrap();
        let corrupted = run_hook_adapter(&repo, "stop", &stop.to_string(), adapter);
        assert_eq!(corrupted["decision"], "block");
        assert_eq!(corrupted["baron"]["hook_failure"], "hard");
    }
}

#[test]
fn cli_unknown_turn_fails_closed_without_publishing_lifecycle_state() {
    let (_temp, repo, _vault) = init_repo();
    let stop = run_hook(
        &repo,
        "stop",
        r#"{"session_id":"unknown-session","turn_id":"unknown-turn","stop_hook_active":false}"#,
    );
    assert_eq!(stop["decision"], "block");
    assert_eq!(stop["baron"]["hook_failure"], "hard");
    assert!(!repo.join(".baron/cache/automation-dedup.json").exists());
    assert!(!repo.join(".baron/state/hook-correlations.json").exists());
}

#[test]
fn cli_user_prompt_submit_uses_structured_stdin_and_bounded_projection() {
    let (_temp, repo, vault) = init_repo();
    let payload = serde_json::json!({
        "schema_version": 1,
        "task": "Unicode tiếng Việt\nquotes \"x\" && $(do-not-run)",
        "session_id": "cli-session",
        "request_id": "cli-request"
    })
    .to_string();
    let first = run_hook(&repo, "user-prompt-submit", &payload);
    let second = run_hook(&repo, "user-prompt-submit", &payload);
    assert_eq!(first, second);
    let context = first["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.len() <= 6_000);
    assert!(context.contains("Unicode") || context.contains("Baron"));
    assert_eq!(first["baron"]["adapter"], "codex");
    assert!(!first["baron"]["project_id"].as_str().unwrap().is_empty());
    let journal = fs::read_to_string(
        vault
            .join("Projects")
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("Artifacts/automation-journal.jsonl"),
    )
    .unwrap();
    assert_eq!(journal.lines().count(), 1);
}

#[test]
fn cli_hook_failure_is_soft_and_fallback_command_remains_available() {
    let (_temp, repo, _vault) = init_repo();
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "automation",
            "hook",
            "user-prompt-submit",
            repo.to_str().unwrap(),
            "--adapter",
            "codex",
        ])
        .write_stdin("{malformed")
        .output()
        .unwrap();
    assert!(output.status.success());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["continue"], true);
    assert_eq!(response["baron"]["hook_failure"], "soft");

    let fallback = serde_json::json!({
        "schema_version": 1,
        "task": "fallback still works",
        "session_id": "fallback-session",
        "request_id": "fallback-request"
    })
    .to_string();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "control-plane",
            "prepare",
            repo.to_str().unwrap(),
            "--adapter",
            "codex",
            "--vault",
            _vault.to_str().unwrap(),
            "--json",
        ])
        .write_stdin(fallback)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":1"));
}

#[test]
fn cli_corrupt_dedup_state_is_a_hard_hook_failure_with_blocking_response() {
    let (_temp, repo, _vault) = init_repo();
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::write(repo.join(".baron/cache/automation-dedup.json"), "{not-json").unwrap();
    let response = run_hook(
        &repo,
        "user-prompt-submit",
        r#"{"session_id":"hard","request_id":"hard","task":"inspect"}"#,
    );
    assert_eq!(response["decision"], "block");
    assert_eq!(response["baron"]["hook_failure"], "hard");
    assert_ne!(response["completed"], true);
}

#[test]
fn cli_stop_does_not_claim_completion_and_child_hook_does_not_prepare() {
    let (_temp, repo, _vault) = init_repo();
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::write(
        repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: active\n- Status: `in_progress`\n- Next action: verify\n",
    )
    .unwrap();
    let stop = run_hook(
        &repo,
        "stop",
        r#"{"session_id":"stop","request_id":"stop","stop_hook_active":false}"#,
    );
    assert_eq!(stop["decision"], "block");
    assert_ne!(stop["completed"], true);
    let child = run_hook(
        &repo,
        "user-prompt-submit",
        r#"{"is_child":true,"child_id":"child","session_id":"child","request_id":"child","task":"child task"}"#,
    );
    assert_eq!(child["baron"]["child"], true);
    assert!(child["hookSpecificOutput"].is_null());
}
