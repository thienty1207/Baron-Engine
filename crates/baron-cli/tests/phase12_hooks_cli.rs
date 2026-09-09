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
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "automation",
            "hook",
            event,
            repo.to_str().unwrap(),
            "--adapter",
            "codex",
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
