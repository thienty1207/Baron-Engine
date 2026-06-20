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

fn init(repo: &Path, vault: &Path) {
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
}

#[test]
fn certify_run_writes_report_and_status_reads_latest() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("certified-app");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("package.json"),
        r#"{"scripts":{"build":"vite build","test":"vitest"}}"#,
    );
    write(
        &repo.join("src/auth/login.ts"),
        "export const login = true;\n",
    );
    init(&repo, &vault);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "certify",
            "run",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
            "--profile",
            "smoke",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Certification"))
        .stdout(predicate::str::contains("Target release: `3.1.2`"))
        .stdout(predicate::str::contains("Passed: `yes`"));

    assert!(repo.join("docs/baron/certification/latest.json").is_file());
    assert!(fs::read_dir(repo.join("docs/baron/certification"))
        .unwrap()
        .any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with("-certification.md")));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["certify", "status", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Certification Status"))
        .stdout(predicate::str::contains("latest certification passed"))
        .stdout(predicate::str::contains("3.1.2"));
}

#[test]
fn certify_help_exposes_run_and_status() {
    Command::cargo_bin("baron")
        .unwrap()
        .args(["certify", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("status"));
}
