use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

fn init_project() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src/features")).unwrap();
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
fn plan_commands_work_from_nested_directory() {
    let (_temp, repo, _vault) = init_project();
    let nested = repo.join("src/features");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args([
            "plan",
            "start",
            "frontend dashboard",
            "--adapter",
            "codex",
            "--session-id",
            "plan-session",
            "--request-id",
            "plan-request",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Risk: `medium`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "update", "Implemented layout; tests remain"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "interrupt", "Stopped before responsive smoke"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["plan", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `interrupted`"))
        .stdout(predicate::str::contains("responsive smoke"));
}

#[test]
fn harness_commands_record_intent_decisions_and_friction() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "harness",
            "intent",
            "backend login with Gin",
            "--current",
            "Login is not implemented.",
            "--target",
            "Users can sign in through the Gin API.",
            "--scope",
            "Backend login endpoint and tests.",
            "--non-goal",
            "Do not redesign the frontend.",
            "--constraint",
            "Preserve the existing user schema.",
            "--decision",
            "Use the current token contract.",
            "--proof",
            "Auth integration tests pass.",
            "--unknown",
            "Social login remains unknown.",
            "--confirmed",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Confirmation: `confirmed`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intent-status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backend login with Gin"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intake", "backend login with Gin"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Risk: `high`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "decision", "Use Rust Axum for API boundaries"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "friction", "Security proof command was unclear"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("backend login with Gin"))
        .stdout(predicate::str::contains("Open friction: 1"));
}

#[test]
fn proof_and_trace_commands_support_a_complete_low_risk_flow() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "plan",
            "start",
            "fix README typo",
            "--adapter",
            "codex",
            "--session-id",
            "proof-session",
            "--request-id",
            "proof-request",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["proof", "record", "README text verified"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "trace",
            "record",
            "Corrected README typo",
            "--outcome",
            "completed",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["trace", "score"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Passed: `yes`"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "complete", "README text verified"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status: `completed`"))
        .stdout(predicate::str::contains("Completion integrity: `passed`"));
}

#[test]
fn execution_command_rejects_identity_mismatch_without_repairing_vault_state() {
    let (_temp, repo, vault) = init_project();
    let project_root = fs::read_dir(vault.join("Projects"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let metadata_path = project_root.join(".baron-project.json");
    let mut metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
    metadata["projectId"] = serde_json::Value::String("wrong-project".to_string());
    let tampered = format!("{}\n", serde_json::to_string_pretty(&metadata).unwrap());
    fs::write(&metadata_path, &tampered).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "plan",
            "start",
            "must not be created",
            "--adapter",
            "codex",
            "--session-id",
            "mismatch-session",
            "--request-id",
            "mismatch-request",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("identity mismatch"))
        .stderr(predicate::str::contains("baron automation reconcile"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["automation", "reconcile"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Automation evidence recorded: no (state is not coherent)",
        ))
        .stdout(predicate::str::contains(
            "Runtime replacement: not attempted",
        ));

    assert_eq!(fs::read_to_string(metadata_path).unwrap(), tampered);
    let plan_index = repo.join("docs/baron/plans/INDEX.md");
    let plan_index = if plan_index.exists() {
        fs::read_to_string(plan_index).unwrap()
    } else {
        String::new()
    };
    assert!(!plan_index.contains("must not be created"));
}

#[test]
fn high_risk_completion_is_rejected_without_evidence() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "plan",
            "start",
            "backend login security",
            "--adapter",
            "codex",
            "--session-id",
            "high-risk-session",
            "--request-id",
            "high-risk-request",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["plan", "complete", "done"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("proof is missing"));
}

#[test]
fn proof_status_and_trace_score_report_missing_state_clearly() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["proof", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Latest proof: none"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["trace", "score"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No Baron trace found"));
}

#[test]
fn trace_score_returns_failure_when_quality_gate_does_not_pass() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "harness",
            "intent",
            "frontend dashboard flow",
            "--current",
            "Dashboard state is incomplete.",
            "--target",
            "Dashboard state is implemented.",
            "--scope",
            "Dashboard flow only.",
            "--proof",
            "Focused dashboard tests pass.",
            "--confirmed",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intake", "frontend dashboard flow"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "trace",
            "record",
            "Implemented dashboard state",
            "--outcome",
            "completed",
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["trace", "score"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Passed: `no`"))
        .stderr(predicate::str::contains("Trace quality gate failed"));
}

#[test]
fn risky_harness_intake_rejects_unconfirmed_cli_intent() {
    let (_temp, repo, _vault) = init_project();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "harness",
            "intent",
            "payment provider migration",
            "--current",
            "Payments use the legacy provider.",
            "--target",
            "Payments use the new provider.",
            "--scope",
            "Provider integration only.",
            "--proof",
            "Payment integration tests pass.",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Confirmation: `needs_confirmation`",
        ));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["harness", "intake", "payment provider migration"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not confirmed"));
}

#[test]
fn proof_cli_keeps_summary_only_capability_evidence_diagnostic() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "capability",
            "register",
            "source control",
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
        .args(["harness", "intake", "fix README typo"])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "proof",
            "record",
            "README text verified",
            "--adapter",
            "codex",
            "--capability-evidence",
            "source-control|git-cli|git status completed and repository state inspected",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Capability gate: `failed`"))
        .stdout(predicate::str::contains(
            "source-control lacks execution evidence",
        ));
}

#[test]
fn proof_execute_requires_and_persists_complete_lifecycle_identity() {
    let (_temp, repo, _vault) = init_project();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args([
            "proof",
            "execute",
            "--capability",
            "test",
            "--provider",
            "cmd",
            "--task",
            "proof execute identity",
            "--adapter",
            "codex",
            "--session-id",
            "proof-execute-session",
            "--request-id",
            "proof-execute-request",
            "cmd",
            "--",
            "/C",
            "exit 0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task ID:"))
        .stdout(predicate::str::contains("Operation ID:"))
        .stdout(predicate::str::contains("Gate kind: `proof`"));
}

#[test]
fn proof_execute_blank_identity_fails_before_child_side_effect() {
    let (_temp, repo, _vault) = init_project();
    let sentinel = repo.join("proof-execute-sentinel.txt");
    #[cfg(windows)]
    let (command, arguments) = (
        "cmd",
        vec!["/C".to_string(), format!("echo ran>{}", sentinel.display())],
    );
    #[cfg(not(windows))]
    let (command, arguments) = (
        "sh",
        vec![
            "-c".to_string(),
            format!("printf ran > '{}'", sentinel.display()),
        ],
    );
    let mut args = vec![
        "proof",
        "execute",
        "--capability",
        "test",
        "--provider",
        "trusted-runner",
        "--task",
        "proof execute blank identity",
        "--adapter",
        "codex",
        "--session-id",
        "",
        "--request-id",
        "proof-execute-request",
        command,
        "--",
    ];
    args.extend(arguments.iter().map(String::as_str));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(args)
        .assert()
        .failure()
        .stderr(predicate::str::contains("must not be blank"));
    assert!(!sentinel.exists());
}

#[test]
fn proof_execute_rejects_repository_local_authority_before_child_side_effect() {
    let (_temp, repo, _vault) = init_project();
    let sentinel = repo.join("repo-local-authority-sentinel.txt");
    let repo_local_home = repo.join(".baron/machine-home");
    #[cfg(windows)]
    let (command, arguments) = (
        "cmd",
        vec!["/C".to_string(), format!("echo ran>{}", sentinel.display())],
    );
    #[cfg(not(windows))]
    let (command, arguments) = (
        "sh",
        vec![
            "-c".to_string(),
            format!("printf ran > '{}'", sentinel.display()),
        ],
    );
    let mut args = vec![
        "proof",
        "execute",
        "--capability",
        "test",
        "--provider",
        "trusted-runner",
        "--task",
        "repository local authority boundary",
        "--adapter",
        "codex",
        "--session-id",
        "repo-local-session",
        "--request-id",
        "repo-local-request",
        command,
        "--",
    ];
    args.extend(arguments.iter().map(String::as_str));

    let output = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .env("BARON_HOME", &repo_local_home)
        .args(args)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "unexpected success\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("outside the repository boundary"),
        "unexpected stderr: {stderr}"
    );
    assert!(!sentinel.exists());
    assert!(!repo_local_home
        .join("authority/execution-receipt-ed25519.seed")
        .exists());
}

#[test]
fn proof_execute_and_record_cross_process_with_exact_binding() {
    let (temp, repo, vault) = init_project();
    let machine_home = temp.path().join("machine-home");
    let mut execute = Command::cargo_bin("baron").unwrap();
    let output = execute
        .current_dir(&repo)
        .env("BARON_HOME", &machine_home)
        .args([
            "proof",
            "execute",
            "--capability",
            "test",
            "--provider",
            "trusted-runner",
            "--task",
            "cross process proof",
            "--adapter",
            "codex",
            "--session-id",
            "cross-process-session",
            "--request-id",
            "cross-process-request",
            "cmd",
            "--",
            "/C",
            "exit 0",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let receipt_id = output_field(&stdout, "Receipt");
    let task_id = output_field(&stdout, "Task ID");
    let operation_id = output_field(&stdout, "Operation ID");

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .env("BARON_HOME", &machine_home)
        .args([
            "proof",
            "record",
            "cross process receipt recorded",
            "--receipt",
            &receipt_id,
            "--task-id",
            &task_id,
            "--operation-id",
            &operation_id,
            "--adapter",
            "codex",
            "--session-id",
            "cross-process-session",
            "--request-id",
            "cross-process-request",
            "--gate-kind",
            "proof",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Proof ID:"));
    assert!(fs::read_dir(vault.join("Projects"))
        .unwrap()
        .next()
        .is_some());
}

#[test]
fn proof_record_rejects_cross_process_binding_mismatches_and_foreign_key() {
    let (temp, repo, _vault) = init_project();
    let machine_home = temp.path().join("machine-home");
    let foreign_home = temp.path().join("foreign-machine-home");
    let (receipt_id, task_id, operation_id) = execute_proof_for_test(
        &repo,
        &machine_home,
        "rejection matrix proof",
        "rejection-session",
        "rejection-request",
    );

    for (label, task, operation, adapter, session, request, gate) in [
        (
            "wrong task",
            "task-not-the-receipt",
            operation_id.as_str(),
            "codex",
            "rejection-session",
            "rejection-request",
            "proof",
        ),
        (
            "wrong operation",
            task_id.as_str(),
            "operation-not-the-receipt",
            "codex",
            "rejection-session",
            "rejection-request",
            "proof",
        ),
        (
            "wrong adapter",
            task_id.as_str(),
            operation_id.as_str(),
            "claude",
            "rejection-session",
            "rejection-request",
            "proof",
        ),
        (
            "wrong session",
            task_id.as_str(),
            operation_id.as_str(),
            "codex",
            "other-session",
            "rejection-request",
            "proof",
        ),
        (
            "wrong request",
            task_id.as_str(),
            operation_id.as_str(),
            "codex",
            "rejection-session",
            "other-request",
            "proof",
        ),
        (
            "wrong gate",
            task_id.as_str(),
            operation_id.as_str(),
            "codex",
            "rejection-session",
            "rejection-request",
            "quality:test-engineer",
        ),
    ] {
        assert_proof_record_rejected(
            &repo,
            &machine_home,
            &receipt_id,
            task,
            operation,
            adapter,
            session,
            request,
            gate,
            label,
        );
    }

    let _ = execute_proof_for_test(
        &repo,
        &foreign_home,
        "foreign authority bootstrap",
        "foreign-session",
        "foreign-request",
    );
    assert_proof_record_rejected(
        &repo,
        &foreign_home,
        &receipt_id,
        &task_id,
        &operation_id,
        "codex",
        "rejection-session",
        "rejection-request",
        "proof",
        "foreign authority key",
    );

    fs::write(repo.join("stale-source.txt"), "changed after execute\n").unwrap();
    assert_proof_record_rejected(
        &repo,
        &machine_home,
        &receipt_id,
        &task_id,
        &operation_id,
        "codex",
        "rejection-session",
        "rejection-request",
        "proof",
        "stale source",
    );
}

#[test]
fn proof_record_rejects_tampered_receipt_even_after_recomputing_unkeyed_digest() {
    let (temp, repo, _vault) = init_project();
    let machine_home = temp.path().join("machine-home");
    let (receipt_id, task_id, operation_id) = execute_proof_for_test(
        &repo,
        &machine_home,
        "tampered proof",
        "tampered-session",
        "tampered-request",
    );
    let path = repo.join(".baron/cache/execution-receipts.jsonl");
    let line = fs::read_to_string(&path).unwrap();
    let mut receipt: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
    receipt["stdout_excerpt"] = serde_json::Value::String("attacker changed output".to_string());
    receipt["integrity_digest"] = serde_json::Value::String(String::new());
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&receipt).unwrap())
    );
    receipt["integrity_digest"] = serde_json::Value::String(digest);
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string(&receipt).unwrap()),
    )
    .unwrap();

    assert_proof_record_rejected(
        &repo,
        &machine_home,
        &receipt_id,
        &task_id,
        &operation_id,
        "codex",
        "tampered-session",
        "tampered-request",
        "proof",
        "tampered receipt",
    );
}

fn execute_proof_for_test(
    repo: &std::path::Path,
    machine_home: &std::path::Path,
    task: &str,
    session: &str,
    request: &str,
) -> (String, String, String) {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(repo)
        .env("BARON_HOME", machine_home)
        .args([
            "proof",
            "execute",
            "--capability",
            "test",
            "--provider",
            "trusted-runner",
            "--task",
            task,
            "--adapter",
            "codex",
            "--session-id",
            session,
            "--request-id",
            request,
            "cmd",
            "--",
            "/C",
            "exit 0",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    (
        output_field(&stdout, "Receipt"),
        output_field(&stdout, "Task ID"),
        output_field(&stdout, "Operation ID"),
    )
}

#[allow(clippy::too_many_arguments)]
fn assert_proof_record_rejected(
    repo: &std::path::Path,
    machine_home: &std::path::Path,
    receipt_id: &str,
    task_id: &str,
    operation_id: &str,
    adapter: &str,
    session_id: &str,
    request_id: &str,
    gate_kind: &str,
    label: &str,
) {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(repo)
        .env("BARON_HOME", machine_home)
        .args([
            "proof",
            "record",
            "rejected proof attempt",
            "--receipt",
            receipt_id,
            "--task-id",
            task_id,
            "--operation-id",
            operation_id,
            "--adapter",
            adapter,
            "--session-id",
            session_id,
            "--request-id",
            request_id,
            "--gate-kind",
            gate_kind,
        ])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "{label} unexpectedly succeeded:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn output_field(output: &str, label: &str) -> String {
    output
        .lines()
        .find_map(|line| line.strip_prefix(&format!("- {label}: `")))
        .and_then(|value| value.strip_suffix('`'))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("missing {label} in output:\n{output}"))
}
