use std::collections::BTreeSet;
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
fn context_codex_outputs_bounded_bundle_without_writing_repo_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("tomoty");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    write(&repo.join("AGENTS.md"), "# Agent Guide\n");
    write(
        &repo.join("package.json"),
        r#"{"scripts":{"build":"vite build","test":"vitest"}}"#,
    );

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
    let context = vault_context_without_create(&vault, &repo).unwrap();
    write(
        &context.project_root.join("Facts.md"),
        "# Facts\n\n- TomoTy context proof is verified.\n",
    );
    let before = list_files(&repo);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Context Bundle - Codex"))
        .stdout(predicate::str::contains("TomoTy context proof"))
        .stdout(predicate::str::contains("Skipped Context"));

    assert_eq!(before, list_files(&repo));
}

#[test]
fn context_supports_claude_generic_and_why_modes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--claude",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Context Bundle - Claude"))
        .stdout(predicate::str::contains("For Claude"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--agent",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "# Baron Context Bundle - Generic Agent",
        ))
        .stdout(predicate::str::contains("For generic agents"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--why",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Context Selection Why"))
        .stdout(predicate::str::contains("Skipped: full Vault scan"));
}

#[test]
fn context_requires_vault_path_or_environment() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["context", repo.to_str().unwrap(), "--codex"])
        .env_remove("BARON_VAULT")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Provide --vault <path> or set BARON_VAULT",
        ));
}

#[test]
fn context_task_flag_changes_risk_guidance() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--codex",
            "--task",
            "implement auth login",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Risk lane: `high`"))
        .stdout(predicate::str::contains("security evidence"));
}

#[test]
fn context_requires_exactly_one_target_unless_why_is_used() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Choose one context target: --codex, --claude, or --agent.",
        ));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--codex",
            "--claude",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Choose only one context target: --codex, --claude, or --agent.",
        ));
}
