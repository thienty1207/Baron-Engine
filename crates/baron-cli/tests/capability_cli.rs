use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn init_project() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src/nested")).unwrap();
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
fn register_check_list_and_remove_work_from_nested_paths() {
    let (_temp, repo, _vault) = init_project();
    let nested = repo.join("src/nested");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args([
            "capability",
            "register",
            "Source Control",
            "--name",
            "git-cli",
            "--kind",
            "cli",
            "--required",
            "--command",
            "git",
            "--description",
            "Provides repository state and change evidence.",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Capability: `source-control`"))
        .stdout(predicate::str::contains("Provider: `git-cli`"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["capability", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Presence: `present`"))
        .stdout(predicate::str::contains("Compatible: `yes`"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["capability", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("source-control"))
        .stdout(predicate::str::contains("required"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args([
            "capability",
            "remove",
            "source-control",
            "--name",
            "git-cli",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed: `yes`"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["capability", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No providers registered"));
}

#[test]
fn capability_check_and_list_have_machine_readable_json() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "security scan",
            "--name",
            "security-skill",
            "--kind",
            "skill",
            "--scan",
            ".codex/skills/vibe-security-scan",
            "--adapter",
            "codex",
            "--description",
            "Provides defensive repository security review guidance.",
        ])
        .assert()
        .success();

    let check = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check", "security-scan", "--json"])
        .output()
        .unwrap();
    assert!(check.status.success());
    let state: serde_json::Value = serde_json::from_slice(&check.stdout).unwrap();
    assert_eq!(state["adapter"], "codex");
    assert_eq!(state["observations"][0]["provider"], "security-skill");

    let list = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "list", "--json"])
        .output()
        .unwrap();
    assert!(list.status.success());
    let registry: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(registry["providers"][0]["capability"], "security-scan");
}

#[test]
fn malformed_provider_contract_fails_clearly() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "impact analysis",
            "--name",
            "graph",
            "--kind",
            "mcp",
            "--description",
            "Provides repository impact analysis evidence.",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "MCP and skill providers require --scan",
        ));
}

#[test]
fn missing_required_capability_is_reported_as_diagnostic_not_command_crash() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "deploy verification",
            "--name",
            "deploy-check",
            "--kind",
            "binary",
            "--required",
            "--command",
            "baron-definitely-missing",
            "--description",
            "Checks deployment health before release completion.",
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Required gaps: deploy-verification",
        ));
}

#[test]
fn list_does_not_reuse_presence_cache_from_another_adapter() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "security scan",
            "--name",
            "security-skill",
            "--kind",
            "skill",
            "--scan",
            ".codex/skills/vibe-security-scan",
            "--adapter",
            "codex",
            "--description",
            "Provides defensive repository security review guidance.",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "check", "--adapter", "codex"])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["capability", "list", "--adapter", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| security-scan | security-skill | skill | optional | unknown | unknown |",
        ));
}
