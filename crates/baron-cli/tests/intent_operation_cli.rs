use std::fs;
use std::path::Path;

use assert_cmd::Command;
use baron_core::operation::{LifecycleIdentity, SupportedAdapter};
use baron_core::vault::ensure_vault;
use serde_json::json;
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

fn run_prepare(repo: &Path, task: &str, session_id: &str, request_id: &str) -> serde_json::Value {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "prepare", "--adapter", "codex", "--json"])
        .current_dir(repo)
        .write_stdin(
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "task": task,
                "session_id": session_id,
                "request_id": request_id
            }))
            .unwrap(),
        )
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

#[test]
fn harness_intent_cli_binds_confirmation_to_prepare_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault);
    let vault_context = ensure_vault(&vault, &repo).unwrap();
    let task = "implement the authorization policy after deciding which policy we should use";
    let session_id = "authorization-session";
    let request_id = "authorization-request";

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "harness",
            "intent",
            "Authorization policy choice",
            "--current",
            "No policy has been selected.",
            "--target",
            "Implement the explicitly selected policy.",
            "--scope",
            "Authorization policy only.",
            "--proof",
            "Authorization tests pass.",
            "--confirmed",
            "--task",
            task,
            "--adapter",
            "codex",
            "--session-id",
            session_id,
            "--request-id",
            request_id,
        ])
        .assert()
        .success();

    let identity = LifecycleIdentity::resolve(
        &vault_context.project_id,
        task,
        SupportedAdapter::Codex,
        Some(session_id),
        Some(request_id),
    )
    .unwrap();
    let operation_path = repo
        .join("docs/baron/harness/intent-operations")
        .join(format!("{}.md", identity.operation_id()));
    assert!(fs::read_to_string(operation_path)
        .unwrap()
        .contains("- Confirmation: `confirmed`"));

    let packet = run_prepare(&repo, task, session_id, request_id);
    assert_eq!(packet["intent"]["title"], "Authorization policy choice");
    assert_eq!(packet["intent"]["confirmed"], true);
    assert!(!packet["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|blocker| blocker["code"] == "intent_confirmation_required"));
}

#[test]
fn harness_intent_partial_identity_fails_before_current_or_operation_writes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault);

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "harness",
            "intent",
            "Authorization policy choice",
            "--current",
            "No policy has been selected.",
            "--target",
            "Implement the explicitly selected policy.",
            "--scope",
            "Authorization policy only.",
            "--proof",
            "Authorization tests pass.",
            "--confirmed",
            "--task",
            "implement the authorization policy",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "operation-scoped selection requires",
        ));

    assert!(!repo.join("docs/baron/harness/CURRENT_INTENT.md").exists());
    assert!(!repo.join("docs/baron/harness/intent-operations").exists());
}
