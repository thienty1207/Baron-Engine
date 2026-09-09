use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

// Portability: profile CLI tests are cross-platform and use deterministic
// temporary repositories without OS-specific hooks or permission behavior.

fn init(repo: &std::path::Path, vault: &std::path::Path, adapter: &str) {
    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            adapter,
            "--database",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("database"));
}

#[test]
fn codex_database_profile_is_available_and_materialized() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("codex-database");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("platform = \"database\""));
    assert!(repo
        .join(".baron/core/skills/database-engineering/SKILL.md")
        .is_file());
}

#[test]
fn claude_database_profile_is_available_and_materialized() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("claude-database");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("platform = \"database\""));
    assert!(repo
        .join(".baron/core/skills/database-engineering/SKILL.md")
        .is_file());
}
