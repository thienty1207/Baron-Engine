use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn review_finding_requires_evidence_to_close() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--agent",
            "--frontend",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();

    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "review",
            "finding",
            "Footer overlaps mobile navigation",
            repo.to_str().unwrap(),
            "--severity",
            "important",
            "--evidence",
            "390px screenshot shows overlap",
            "--affected-file",
            "src/HomePage.tsx",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("- Finding ID: `"))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "review",
            "close",
            id,
            repo.to_str().unwrap(),
            "--fix-evidence",
            "Adjusted layout constraints",
            "--verification",
            "responsive tests passed",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .args(["review", "status", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Open findings: 0"));
}
