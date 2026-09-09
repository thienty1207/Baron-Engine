//! Phase 10 CLI target evidence.
//!
//! The fail-closed assertions intentionally fail against the pre-Phase-10 CLI,
//! which silently resolves omitted identity through project `active_adapter`.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use tempfile::tempdir;

fn init(repo: &std::path::Path, vault: &std::path::Path, adapter: &str) {
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            adapter,
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn reciprocal_prepare_is_explicit_while_project_active_adapter_is_codex() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["init", "--claude"])
        .assert()
        .success();

    let output = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["control-plane", "prepare", "--adapter", "claude", "--json"])
        .write_stdin(
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "task": "resume the shared task",
                "session_id": "claude-session",
                "request_id": "claude-request"
            }))
            .unwrap(),
        )
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let packet: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(packet["adapter"], "claude");
    assert_eq!(packet["context"]["target"], "claude");
    assert_eq!(packet["session_id"], "claude-session");
    assert_eq!(packet["request_id"], "claude-request");
}

#[test]
fn capability_check_without_identity_fails_closed_instead_of_using_active_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("capability-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check", "--json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("explicit adapter identity"));
}

#[test]
fn explicit_capability_identity_wins_over_the_serialized_active_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("capability-explicit-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["init", "--claude"])
        .assert()
        .success();

    let output = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check", "--adapter", "codex", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let state: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(state["adapter"], "codex");
}

#[test]
fn runtime_check_without_identity_fails_closed_instead_of_using_active_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("runtime-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["runtime", "check", "--json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("explicit adapter identity"));
}

#[test]
fn context_without_identity_fails_closed_instead_of_using_active_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("context-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["context", "--vault", vault.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Choose one context target"));
}

#[test]
fn continuity_checkpoint_requires_explicit_adapter_and_preserves_claude_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("continuity-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["continuity", "checkpoint", "missing identity must fail"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("explicit adapter identity"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "continuity",
            "checkpoint",
            "Claude resumed the shared task",
            "--adapter",
            "claude",
        ])
        .assert()
        .success();
    let content = fs::read_to_string(repo.join("docs/baron/continuity/CURRENT.md")).unwrap();
    assert!(content.contains("- Adapter: `claude`"));
}

#[test]
fn capability_bearing_proof_requires_explicit_adapter_and_does_not_follow_active_value() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("proof-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "proof",
            "record",
            "Claude explicit evidence",
            "--adapter",
            "claude",
            "--capability-evidence",
            "source-control|git-cli|explicit Claude evidence",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adapter: `claude`"));
}

#[test]
fn adapter_switch_remains_compatibility_ui_and_declares_non_authority() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("switch-cli");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["adapter", "switch", "--to", "claude", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("non-authoritative"));
}
