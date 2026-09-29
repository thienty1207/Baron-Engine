use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use baron_core::config::load_project_config;
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::proof::record_proof_for_operation;
use baron_core::trace::{
    record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::ensure_vault;
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

fn active_plan_paths(repo: &Path) -> Vec<PathBuf> {
    fs::read_to_string(repo.join("docs/baron/plans/ACTIVE.md"))
        .unwrap()
        .lines()
        .filter_map(|line| {
            line.strip_prefix("<!-- BARON:ACTIVE-PLAN ")
                .and_then(|value| value.strip_suffix(" -->"))
                .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
                .and_then(|entry| entry["plan_path"].as_str().map(|path| repo.join(path)))
        })
        .collect()
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

#[test]
fn identified_cli_mutations_target_exact_active_plan_and_legacy_is_ambiguous() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("plan-identity-mutations");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault);

    let first_title = "fix README alpha typo";
    let second_title = "fix README beta typo";
    let start = |title: &str, adapter: &str, session: &str, request: &str| {
        Command::cargo_bin("baron")
            .unwrap()
            .args([
                "plan",
                "start",
                title,
                "--adapter",
                adapter,
                "--session-id",
                session,
                "--request-id",
                request,
            ])
            .current_dir(&repo)
            .assert()
            .success();
    };
    start(first_title, "codex", "first-session", "first-request");
    start(second_title, "claude", "second-session", "second-request");

    let paths = active_plan_paths(&repo);
    assert_eq!(paths.len(), 2);
    let first_path = paths
        .iter()
        .find(|path| fs::read_to_string(path).unwrap().contains(first_title))
        .unwrap()
        .clone();
    let second_path = paths
        .iter()
        .find(|path| fs::read_to_string(path).unwrap().contains(second_title))
        .unwrap()
        .clone();
    let second_before = fs::read(&second_path).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "update",
            "alpha-only update",
            "--task",
            first_title,
            "--adapter",
            "codex",
            "--session-id",
            "first-session",
            "--request-id",
            "first-request",
        ])
        .current_dir(&repo)
        .assert()
        .success();
    assert!(fs::read_to_string(&first_path)
        .unwrap()
        .contains("alpha-only update"));
    assert_eq!(fs::read(&second_path).unwrap(), second_before);

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "interrupt",
            "alpha paused",
            "--task",
            first_title,
            "--adapter",
            "codex",
            "--session-id",
            "first-session",
            "--request-id",
            "first-request",
        ])
        .current_dir(&repo)
        .assert()
        .success();
    assert!(fs::read_to_string(&first_path)
        .unwrap()
        .contains("status: interrupted"));
    assert!(fs::read_to_string(&second_path)
        .unwrap()
        .contains("status: in_progress"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "update",
            "beta-only update",
            "--task",
            second_title,
            "--adapter",
            "claude",
            "--session-id",
            "second-session",
            "--request-id",
            "second-request",
        ])
        .current_dir(&repo)
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["plan", "update", "must be rejected as ambiguous"])
        .current_dir(&repo)
        .assert()
        .failure()
        .stderr(predicates::str::contains("ambiguous"));

    let config = load_project_config(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let second_identity = LifecycleIdentity::resolve(
        &config.project_id,
        second_title,
        SupportedAdapter::Claude,
        Some("second-session"),
        Some("second-request"),
    )
    .unwrap();
    let second_operation = OperationContext::from_identity(&second_identity);
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &second_operation,
        "README verification passed",
    )
    .unwrap();
    let binding = TraceOperationBinding::from_operation(&second_operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "README beta typo corrected",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "plan",
            "complete",
            "beta verification passed",
            "--task",
            second_title,
            "--adapter",
            "claude",
            "--session-id",
            "second-session",
            "--request-id",
            "second-request",
        ])
        .current_dir(&repo)
        .assert()
        .success();
    assert!(fs::read_to_string(&second_path)
        .unwrap()
        .contains("status: completed"));
    assert!(fs::read_to_string(&first_path)
        .unwrap()
        .contains("status: interrupted"));
}
