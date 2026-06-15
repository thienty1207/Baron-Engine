use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn init_project() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src/features")).unwrap();
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
fn plan_commands_work_from_nested_directory() {
    let (_temp, repo, _vault) = init_project();
    let nested = repo.join("src/features");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "start", "frontend dashboard"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Risk: `medium`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "update", "Implemented layout; tests remain"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "interrupt", "Stopped before responsive smoke"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `interrupted`"))
        .stdout(predicate::str::contains("responsive smoke"));
}

#[test]
fn harness_commands_record_intent_decisions_and_friction() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intake", "backend login with Gin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Risk: `high`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "decision", "Use Rust Axum for API boundaries"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "friction", "Security proof command was unclear"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backend login with Gin"))
        .stdout(predicate::str::contains("Open friction: 1"));
}

#[test]
fn proof_and_trace_commands_support_a_complete_low_risk_flow() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "start", "fix README typo"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["proof", "record", "README text verified"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "trace",
            "record",
            "Corrected README typo",
            "--outcome",
            "completed",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["trace", "score"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Passed: `yes`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "complete", "README text verified"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `completed`"));
}

#[test]
fn high_risk_completion_is_rejected_without_evidence() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "start", "backend login security"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "complete", "done"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("proof is missing"));
}

#[test]
fn proof_status_and_trace_score_report_missing_state_clearly() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["proof", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Latest proof: none"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["trace", "score"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No Baron trace found"));
}

#[test]
fn trace_score_returns_failure_when_quality_gate_does_not_pass() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intake", "frontend dashboard flow"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "trace",
            "record",
            "Implemented dashboard state",
            "--outcome",
            "completed",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["trace", "score"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Passed: `no`"))
        .stderr(predicate::str::contains("Trace quality gate failed"));
}

#[test]
fn proof_cli_accepts_structured_capability_execution_evidence() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "source control",
            "--name",
            "git-cli",
            "--kind",
            "cli",
            "--required",
            "--command",
            "git",
            "--description",
            "Provides repository state and change evidence.",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intake", "fix README typo"])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "proof",
            "record",
            "README text verified",
            "--capability-evidence",
            "source-control|git-cli|git status completed and repository state inspected",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Capability gate: `passed`"));
}
