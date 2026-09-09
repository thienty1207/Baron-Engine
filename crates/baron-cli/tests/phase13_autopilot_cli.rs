//! Cross-platform CLI fixtures for the Phase 13 conversational approval path.

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
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

#[test]
fn conversational_response_needs_no_internal_candidate_id() {
    let (_temp, repo, _vault) = init_repo();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "autopilot",
            "review",
            "Keep API errors structured",
            repo.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Approval required: `yes`"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "autopilot",
            "respond",
            "yes, make that the project rule",
            repo.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `approved`"))
        .stdout(predicate::str::contains("project decision authority"));
}

#[test]
fn ambiguous_response_fails_safe_without_changing_candidate() {
    let (_temp, repo, _vault) = init_repo();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "autopilot",
            "review",
            "Keep API errors structured",
            repo.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["autopilot", "respond", "maybe", repo.to_str().unwrap()])
        .assert()
        .failure();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["autopilot", "status", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Open candidates: 1"));
}
