use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn list_files(root: &Path) -> BTreeSet<PathBuf> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeSet<PathBuf>) {
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(path.strip_prefix(root).unwrap().to_path_buf());
            }
        }
    }

    let mut files = BTreeSet::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn memory_status_reports_missing_vault_without_creating_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let before = list_files(&repo);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "status",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Memory Status"))
        .stdout(predicate::str::contains("Vault exists: no"))
        .stdout(predicate::str::contains("No files were written"));

    assert_eq!(before, list_files(&repo));
    assert!(!vault.exists());
}

#[test]
fn memory_index_creates_vault_and_project_capsule() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("tomoty");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "index",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Memory Index"))
        .stdout(predicate::str::contains("Project slug: `tomoty`"))
        .stdout(predicate::str::contains("memory-index.sqlite"));

    assert!(vault.join("Projects/tomoty/Facts.md").exists());
    assert!(vault.join("Artifacts/Baron/memory-index.sqlite").exists());
}

#[test]
fn memory_compact_prints_bounded_firewall_brief() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("tomoty");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "index",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    write(
        &vault.join("Projects/tomoty/Facts.md"),
        "# Facts\n\n- Survey engine proof is verified.\n",
    );

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "compact",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Memory Firewall Brief"))
        .stdout(predicate::str::contains("Survey engine proof"));
}

#[test]
fn recall_returns_current_project_before_other_project() {
    let temp = tempdir().unwrap();
    let tomoty = temp.path().join("tomoty");
    let legacy = temp.path().join("legacy-crm");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&tomoty).unwrap();
    fs::create_dir_all(&legacy).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "index",
            tomoty.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "index",
            legacy.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    write(
        &vault.join("Projects/tomoty/Facts.md"),
        "# Facts\n\n- Auth login uses Rust Axum.\n",
    );
    write(
        &vault.join("Projects/legacy-crm/Facts.md"),
        "# Facts\n\n- Auth login uses legacy PHP.\n",
    );

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "memory",
            "index",
            tomoty.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "recall",
            "auth login",
            tomoty.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Recall"))
        .stdout(predicate::str::contains("Auth login uses Rust Axum"))
        .stdout(predicate::str::contains("Blocked cross-project"));
}

#[test]
fn memory_commands_require_vault_path_or_environment() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["memory", "index", repo.to_str().unwrap()])
        .env_remove("BARON_VAULT")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Provide --vault <path> or set BARON_VAULT",
        ));
}
