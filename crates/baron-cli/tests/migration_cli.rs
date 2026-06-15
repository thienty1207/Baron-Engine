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

fn file_count(root: &Path) -> usize {
    fn visit(path: &Path, count: &mut usize) {
        if !path.exists() {
            return;
        }
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(&entry.path(), count);
            } else {
                *count += 1;
            }
        }
    }
    let mut count = 0;
    visit(root, &mut count);
    count
}

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-app");
    let vault = temp.path().join("Vault");
    let legacy_project = vault.join("Projects/legacy-app-old");
    fs::create_dir_all(&repo).unwrap();
    write(
        &repo.join("vault.config.json"),
        &serde_json::to_string_pretty(&serde_json::json!({
            "vault_root": vault,
            "project_slug": "legacy-app-old",
            "project_root": legacy_project
        }))
        .unwrap(),
    );
    write(
        &repo.join(".agent-bootstrap-manifest.json"),
        "{\"version\":1,\"entries\":{}}\n",
    );
    write(
        &repo.join("AGENTS.md"),
        "# User Rules\n\nPreserve this.\n\n<!-- agent-bootstrap:start -->\nlegacy\n<!-- agent-bootstrap:end -->\n",
    );
    write(
        &repo.join("scripts/agent-memory.js"),
        "// agent-bootstrap runtime\n",
    );
    write(
        &legacy_project.join("Facts.md"),
        "# Facts\n\n- Login uses email.\n",
    );
    write(
        &legacy_project.join("Research/backend.md"),
        "# Backend Research\n\n- Axum tower middleware handles request tracing.\n",
    );
    write(
        &repo.join(".codex/skills/rust-api/SKILL.md"),
        "---\nname: rust-api\ndescription: Use when implementing Rust APIs.\n---\n\nRequire test evidence.\n",
    );
    (temp, repo, vault)
}

#[test]
fn dry_run_prints_inventory_and_writes_nothing() {
    let (_temp, repo, vault) = fixture();
    let repo_files = file_count(&repo);
    let vault_files = file_count(&vault);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "migrate",
            "agent-bootstrap",
            repo.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Migration Dry Run"))
        .stdout(predicate::str::contains("No files were written"))
        .stdout(predicate::str::contains("scripts/agent-memory.js"));

    assert_eq!(file_count(&repo), repo_files);
    assert_eq!(file_count(&vault), vault_files);
}

#[test]
fn apply_installs_baron_imports_memory_and_retires_legacy_runtime() {
    let (_temp, repo, vault) = fixture();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["migrate", "agent-bootstrap", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `completed`"))
        .stdout(predicate::str::contains("Backup:"));

    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join(".baron/local.toml").exists());
    assert!(repo.join(".codex/skills/superpowers/SKILL.md").exists());
    assert!(repo.join(".codex/skills/rust-api/SKILL.md").exists());
    assert!(!repo.join("scripts/agent-memory.js").exists());
    assert!(!repo.join("vault.config.json").exists());
    assert!(vault.join("Projects/legacy-app/Facts.md").exists());
    assert!(
        fs::read_to_string(vault.join("Projects/legacy-app/Facts.md"))
            .unwrap()
            .contains("Login uses email")
    );
    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("Preserve this."));
    assert!(agents.contains("BARON:MANAGED:START"));
    assert!(!agents.contains("agent-bootstrap:start"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["migrate", "status", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `completed`"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["recall", "Axum tower middleware", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Axum tower middleware"));
}

#[test]
fn rollback_command_restores_legacy_runtime() {
    let (_temp, repo, vault) = fixture();
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["migrate", "agent-bootstrap", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let id = stdout
        .lines()
        .find_map(|line| line.strip_prefix("- Migration ID: `"))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "migrate",
            "rollback",
            "--id",
            id,
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `rolled_back`"));

    assert!(repo.join("vault.config.json").exists());
    assert!(repo.join("scripts/agent-memory.js").exists());
    assert!(!repo.join(".baron/project.toml").exists());
}
