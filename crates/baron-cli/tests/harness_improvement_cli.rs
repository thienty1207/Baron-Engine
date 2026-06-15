use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn harness_audit_and_verify_all_are_available_to_agents() {
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
    write(
        &repo.join("docs/baron/harness/TEST_MATRIX.md"),
        "# Baron Validation Matrix\n\n\
| Story | Risk | Status | Evidence |\n\
| --- | --- | --- | --- |\n\
| auth login | high | pending | pending |\n",
    );

    Command::cargo_bin("baron")
        .unwrap()
        .args(["harness", "audit", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Harness Audit"))
        .stdout(predicate::str::contains("Context-read score"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["harness", "verify-all", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "# Baron Harness Story Verification",
        ))
        .stdout(predicate::str::contains("auth login"));
}

#[test]
fn harness_intervention_propose_and_outcome_form_a_safe_loop() {
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
    for item in [
        "proof command was unclear",
        "proof evidence was unclear",
        "trace proof was unclear",
    ] {
        Command::cargo_bin("baron")
            .unwrap()
            .args(["harness", "friction", item, repo.to_str().unwrap()])
            .assert()
            .success();
    }

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "harness",
            "intervention",
            "reviewer corrected missing proof",
            repo.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Intervention recorded"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["harness", "propose", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("human approval required"))
        .stdout(predicate::str::contains("proposal-proof-guidance"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "harness",
            "outcome",
            "proposal-proof-guidance",
            "proof ambiguity reduced",
            repo.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Outcome recorded"));
}
