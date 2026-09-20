use std::fs;
use std::path::Path;

use assert_cmd::Command;
use tempfile::tempdir;

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

fn vault_plan_contents(vault: &Path) -> String {
    fs::read_dir(vault.join("Projects"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .flat_map(|project| {
            fs::read_dir(project.path().join("Plans"))
                .into_iter()
                .flatten()
        })
        .filter_map(Result::ok)
        .flat_map(|date| fs::read_dir(date.path()).into_iter().flatten())
        .filter_map(Result::ok)
        .find_map(|entry| fs::read_to_string(entry.path()).ok())
        .unwrap_or_default()
}

#[test]
fn plan_start_requires_atomic_complete_identity_before_any_write() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("plan-identity");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault);

    let current = repo.join("docs/baron/plans/CURRENT.md");
    Command::cargo_bin("baron")
        .unwrap()
        .args(["plan", "start", "frontend dashboard"])
        .current_dir(&repo)
        .assert()
        .failure();
    assert!(!current.exists());
    assert!(vault_plan_contents(&vault).is_empty());

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "start",
            "frontend dashboard",
            "--adapter",
            "codex",
            "--session-id",
            "cli-session",
        ])
        .current_dir(&repo)
        .assert()
        .failure();
    assert!(!current.exists());
}

#[test]
fn plan_start_rejects_empty_identity_values_before_any_write() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("plan-identity-empty");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "start",
            "frontend dashboard",
            "--adapter",
            "codex",
            "--session-id",
            "",
            "--request-id",
            "request-a",
        ])
        .current_dir(&repo)
        .assert()
        .failure();

    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
    assert!(vault_plan_contents(&vault).is_empty());
}

#[test]
fn plan_start_persists_the_supplied_complete_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("plan-identity");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "start",
            "frontend dashboard",
            "--adapter",
            "codex",
            "--session-id",
            "cli-session",
            "--request-id",
            "cli-request",
        ])
        .current_dir(&repo)
        .assert()
        .success();

    let current = fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md")).unwrap();
    let vault_plan = vault_plan_contents(&vault);
    assert!(vault_plan.contains("operation_id:"));
    assert!(vault_plan.contains("adapter: codex"));
    assert!(vault_plan.contains("session_id: cli-session"));
    assert!(vault_plan.contains("request_id: cli-request"));
    assert!(current.contains("Operation ID:"));
    assert!(current.contains("Adapter: `codex`"));
    assert!(current.contains("Session ID: `cli-session`"));
    assert!(current.contains("Request ID: `cli-request`"));
}
