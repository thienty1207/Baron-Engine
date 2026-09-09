//! Phase 14 CLI update and persisted-state preservation fixtures.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use baron_core::vault::vault_context_without_create;
use predicates::prelude::*;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn snapshot_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        if !current.exists() {
            return;
        }
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else if path.is_file() {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn init(repo: &Path, vault: &Path, adapter: &str) {
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
fn dry_run_preserves_project_vault_task_hook_and_autopilot_state_bytes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("update-project");
    let nested = repo.join("src/features");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&nested).unwrap();
    init(&repo, &vault, "--codex");
    init(&repo, &vault, "--claude");

    write(
        &repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Status: `in_progress`\n- Next action: preserve this task\n",
    );
    write(
        &repo.join("docs/baron/continuity/CURRENT.md"),
        "# Continuity\n\n- Status: interrupted\n- Next action: resume task\n",
    );
    write(
        &repo.join("docs/baron/autopilot/STATE.json"),
        "{\n  \"schema_version\": 1,\n  \"project_id\": \"update-project\",\n  \"candidates\": [],\n  \"responses\": [],\n  \"archived\": []\n}\n",
    );
    let context = vault_context_without_create(&vault, &repo).unwrap();
    write(
        &repo.join(".codex/hooks.json"),
        "{\"hooks\":{\"SessionStart\":[{\"command\":\"user-hook\"}]}}\n",
    );
    write(
        &context
            .project_root
            .join("Artifacts/automation-journal.jsonl"),
        "{\"event\":\"checkpoint\",\"adapter\":\"codex\",\"future\":true}\n",
    );

    let repo_before = snapshot_files(&repo);
    let vault_before = snapshot_files(&vault);

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["update", "--dry-run", "--installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Safe Update Preview"))
        .stdout(predicate::str::contains("No project files were written."));

    assert_eq!(snapshot_files(&repo), repo_before);
    assert_eq!(snapshot_files(&vault), vault_before);
    assert!(!repo.join(".baron/update").exists());
}

#[test]
fn update_with_a_future_project_schema_fails_closed_before_transaction_staging() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("future-project");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    let config = repo.join(".baron/project.toml");
    let future = fs::read_to_string(&config)
        .unwrap()
        .replace("schema_version = 4", "schema_version = 999");
    fs::write(&config, future).unwrap();
    let before = fs::read(&config).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["update", repo.to_str().unwrap(), "--dry-run", "--installed"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported Baron project schema"));

    assert_eq!(fs::read(&config).unwrap(), before);
    assert!(!repo.join(".baron/update/transactions").exists());
}
