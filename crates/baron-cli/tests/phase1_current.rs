use std::fs;

use assert_cmd::Command;
use baron_core::vault::vault_context_without_create;
use tempfile::tempdir;

// Portability: CLI fixtures use assert_cmd and temp directories on all hosts.

fn init(repo: &std::path::Path, vault: &std::path::Path, adapter: &str) {
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
fn current_cli_multi_adapter_initialization_preserves_shared_project_and_vault_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("cli-multi-adapter");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    init(&repo, &vault, "--codex");
    init(&repo, &vault, "--claude");

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("codex"));
    assert!(config.contains("claude"));
    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join("CLAUDE.md").is_file());
    let context = vault_context_without_create(&vault, &repo).unwrap();
    assert!(context.project_root.join("Facts.md").is_file());
    assert!(context.vault_root.ends_with("Vault"));
}

#[test]
fn current_cli_installation_order_permutations_keep_both_adapter_surfaces() {
    for order in [["--codex", "--claude"], ["--claude", "--codex"]] {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("cli-order");
        let vault = temp.path().join("Vault");
        fs::create_dir_all(&repo).unwrap();
        for adapter in order {
            init(&repo, &vault, adapter);
        }
        assert!(repo.join("AGENTS.md").is_file());
        assert!(repo.join("CLAUDE.md").is_file());
        assert!(repo.join(".baron/project.toml").is_file());
    }
}
