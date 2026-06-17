use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn automation_hook_reads_native_payload_and_reports_status() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
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

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["automation", "hook", "session-start", "--adapter", "codex"])
        .write_stdin(r#"{"session_id":"cli-session","cwd":"demo"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("additionalContext"))
        .stdout(predicate::str::contains("Baron Context"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["automation", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("session_start"));
}

#[test]
fn continuity_checkpoint_and_status_are_available_for_ai_resume() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
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

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["continuity", "checkpoint", "before editing auth handler"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Baron Continuity Checkpoint"))
        .stdout(predicate::str::contains("before editing auth handler"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["continuity", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Baron Continuity Status"))
        .stdout(predicate::str::contains("before editing auth handler"));
}
