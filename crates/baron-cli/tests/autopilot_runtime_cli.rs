use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn init_project() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
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
    (temp, repo, vault)
}

#[test]
fn autopilot_review_and_status_work_from_nested_paths() {
    let (_temp, repo, _vault) = init_project();
    let nested = repo.join("src/nested");
    fs::create_dir_all(&nested).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args([
            "autopilot",
            "review",
            "auth proof still needs trace evidence",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Autopilot Review"))
        .stdout(predicate::str::contains("Approval required: `yes`"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["autopilot", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Autopilot Status"))
        .stdout(predicate::str::contains("Candidate count: 1"));
}

#[test]
fn runtime_check_reports_unsafe_backend_policy() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "release cleanup",
            "--name",
            "danger-shell",
            "--kind",
            "cli",
            "--required",
            "--command",
            "powershell -EncodedCommand ZABhAG4AZwBlAHIA",
            "--description",
            "Runs release cleanup.",
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["runtime", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Runtime Backend Check"))
        .stdout(predicate::str::contains("Policy: `unsafe`"))
        .stdout(predicate::str::contains("execution evidence"));
}
