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
fn non_shadow_init_installs_codex_and_configuration() {
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
        .success()
        .stdout(predicate::str::contains("Adapter initialized: `codex`"));

    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join(".baron/local.toml").exists());
    assert!(repo.join("AGENTS.md").exists());
    assert!(repo.join(".codex/skills/superpowers/SKILL.md").exists());
    assert!(vault.join("Projects/demo/Facts.md").exists());
}

#[test]
fn repeated_init_registers_codex_and_claude() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    for flag in ["--codex", "--claude"] {
        Command::cargo_bin("baron")
            .unwrap()
            .args([
                "init",
                repo.to_str().unwrap(),
                flag,
                "--vault",
                vault.to_str().unwrap(),
            ])
            .assert()
            .success();
    }

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("codex"));
    assert!(config.contains("claude"));
    assert!(repo.join("AGENTS.md").exists());
    assert!(repo.join("CLAUDE.md").exists());
}

#[test]
fn update_from_nested_path_refreshes_registered_adapters() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src/features");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();

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
        &repo.join("AGENTS.md"),
        "# User Header\n\n<!-- BARON:MANAGED:START -->\nstale\n<!-- BARON:MANAGED:END -->\n",
    );

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated adapters: `codex`"));

    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("# User Header"));
    assert!(agents.contains("Baron Automatic Agent Contract"));
    assert!(!agents.contains("\nstale\n"));
}

#[test]
fn context_uses_registered_adapter_and_local_vault_automatically() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();

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

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Context Bundle - Codex"))
        .stdout(predicate::str::contains("Project: `demo`"));
}

#[test]
fn shadow_init_remains_read_only_and_does_not_require_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", repo.to_str().unwrap(), "--agent", "--shadow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No files were written"));

    assert!(!repo.join(".baron/project.toml").exists());
}
