use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use baron_core::capability::{
    check_capabilities, evaluate_execution_evidence, evaluate_execution_evidence_for_operation,
    register_provider, CapabilityExecutionEvidence, CapabilityProvider, CheckOptions, ProviderKind,
    Requirement,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::control_plane::{
    gate_evidence_status_strict, gate_evidence_status_strict_for_context,
    gate_evidence_status_strict_for_operation, gate_evidence_status_strict_for_request,
    gate_evidence_status_strict_for_scope, record_gate_evidence,
    record_gate_evidence_with_receipt_bound, GateReceiptBinding,
};
use baron_core::execution_receipt::{
    execute_command, execute_command_for_identity, execute_command_with_context, load_receipts,
    load_valid_receipts_for_diagnostics, load_verified_receipt, load_verified_receipts_strict,
    receipt_is_current_authority, ExecutionReceipt, ExecutionRequest, ReceiptContext,
    ReceiptProvenance,
};
use baron_core::identity::project_id_for_path;
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::proof::{record_proof_from_receipt, record_proof_from_receipt_bound};
use baron_core::vault::ensure_vault;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

fn identity(repo: &Path, task: &str, session: &str, request: &str) -> LifecycleIdentity {
    let project_id = project_id_for_path(repo).unwrap();
    LifecycleIdentity::resolve(
        &project_id,
        task,
        SupportedAdapter::Codex,
        Some(session),
        Some(request),
    )
    .unwrap()
}

fn sentinel_command(repo: &Path, sentinel: &Path) -> ExecutionRequest {
    #[cfg(windows)]
    let (executable, arguments) = (
        "cmd",
        vec![
            "/C".to_string(),
            format!("echo touched>\"{}\"", sentinel.display()),
        ],
    );
    #[cfg(not(windows))]
    let (executable, arguments) = (
        "sh",
        vec![
            "-c".to_string(),
            format!("printf touched > '{}'", sentinel.display()),
        ],
    );
    ExecutionRequest {
        capability: "security-review".to_string(),
        provider: "trusted-runner".to_string(),
        executable: executable.to_string(),
        arguments,
        working_directory: repo.to_path_buf(),
        timeout: Duration::from_secs(5),
    }
}

fn execute_authoritative(
    repo: &Path,
    request: ExecutionRequest,
    requested_context: ReceiptContext,
) -> (ExecutionReceipt, ReceiptContext) {
    let project_id = project_id_for_path(repo).unwrap();
    let adapter = SupportedAdapter::parse(&requested_context.adapter).unwrap();
    let identity = LifecycleIdentity::resolve(
        &project_id,
        &requested_context.task_id,
        adapter,
        Some(&requested_context.session_id),
        Some(&requested_context.request_id),
    )
    .unwrap();
    let context = ReceiptContext::for_identity(&identity, requested_context.gate_kind).unwrap();
    let receipt = execute_command_for_identity(request, &identity, &context.gate_kind).unwrap();
    (receipt, context)
}

fn authority_worker_output(
    mode: &str,
    repo: &Path,
    vault: &Path,
    machine_home: &Path,
) -> std::process::Output {
    Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", mode)
        .env("BARON_TEST_REPO", repo)
        .env("BARON_TEST_VAULT", vault)
        .env("BARON_HOME", machine_home)
        .output()
        .unwrap()
}

#[test]
fn receipt_authority_scope_worker() {
    let Ok(mode) = env::var("BARON_AUTHORITY_TEST_MODE") else {
        return;
    };
    let repo = std::path::PathBuf::from(env::var_os("BARON_TEST_REPO").unwrap());
    let vault = std::path::PathBuf::from(env::var_os("BARON_TEST_VAULT").unwrap());
    match mode.as_str() {
        "containment" => {
            let sentinel = std::path::PathBuf::from(env::var_os("BARON_TEST_SENTINEL").unwrap());
            let identity = identity(
                &repo,
                &env::var("BARON_TEST_TASK").unwrap(),
                "session-scope-worker",
                "request-scope-worker",
            );
            let result = execute_command_for_identity(
                sentinel_command(&repo, &sentinel),
                &identity,
                "proof",
            );
            assert!(result.is_err());
            assert!(!sentinel.exists());
        }
        "external" => {
            let identity = identity(
                &repo,
                "external authority root",
                "session-external",
                "request-external",
            );
            let receipt = execute_command_for_identity(command(&repo), &identity, "proof").unwrap();
            assert_eq!(receipt.schema_version, 2);
        }
        "normal" => {
            let normal_home =
                std::path::PathBuf::from(env::var_os("BARON_TEST_NORMAL_HOME").unwrap());
            env::remove_var("BARON_HOME");
            env::set_var("HOME", &normal_home);
            env::set_var("USERPROFILE", &normal_home);
            let identity = identity(
                &repo,
                "normal home authority root",
                "session-normal",
                "request-normal",
            );
            let receipt = execute_command_for_identity(command(&repo), &identity, "proof").unwrap();
            assert_eq!(receipt.schema_version, 2);
        }
        "malformed" => {
            let identity = identity(&repo, "malformed seed", "session-seed", "request-seed");
            assert!(execute_command_for_identity(command(&repo), &identity, "proof").is_err());
        }
        "strict" => {
            let identity = identity(&repo, "strict loader", "session-strict", "request-strict");
            let valid = execute_command_for_identity(command(&repo), &identity, "proof").unwrap();
            let mut invalid = valid.clone();
            invalid.receipt_id = "receipt-invalid-schema-v2".to_string();
            invalid.stdout_excerpt = "tampered but digest recomputed".to_string();
            invalid.integrity_digest.clear();
            invalid.integrity_digest = format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&invalid).unwrap())
            );
            fs::write(
                repo.join(".baron/cache/execution-receipts.jsonl"),
                format!(
                    "{}\n{}\n",
                    serde_json::to_string(&valid).unwrap(),
                    serde_json::to_string(&invalid).unwrap()
                ),
            )
            .unwrap();
            assert!(load_verified_receipts_strict(&repo).is_err());
            assert_eq!(load_valid_receipts_for_diagnostics(&repo).unwrap().len(), 1);
        }
        other => panic!(
            "unknown authority worker mode: {other}; vault={}",
            vault.display()
        ),
    }
}

fn command(repo: &std::path::Path) -> ExecutionRequest {
    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    ExecutionRequest {
        capability: "security-review".to_string(),
        provider: "trusted-runner".to_string(),
        executable: executable.to_string(),
        arguments,
        working_directory: repo.to_path_buf(),
        timeout: Duration::from_secs(5),
    }
}

fn provider() -> CapabilityProvider {
    #[cfg(windows)]
    let command = "cmd";
    #[cfg(not(windows))]
    let command = "true";
    CapabilityProvider {
        name: "trusted-runner".to_string(),
        capability: "security-review".to_string(),
        kind: ProviderKind::Cli,
        requirement: Requirement::Required,
        command: Some(command.to_string()),
        scan_target: None,
        adapters: vec![AdapterKind::Codex],
        description: "Provides trusted security verification evidence.".to_string(),
    }
}

#[test]
fn free_form_gate_evidence_is_diagnostic_only() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    record_gate_evidence(&repo, &context, "security-auditor", "security looks good").unwrap();
    let status = gate_evidence_status_strict(&repo, &["security-auditor".to_string()]).unwrap();
    assert!(!status.passed);
    assert_eq!(status.missing_agents, ["security-auditor"]);
}

#[test]
fn typed_receipt_is_bound_to_gate_task_and_operation() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let binding = GateReceiptBinding::new(
        "task-a",
        "operation-a",
        "codex",
        "session-a",
        "request-a",
        "quality:security-auditor",
    );
    let (receipt, binding) = execute_authoritative(&repo, command(&repo), binding);
    record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "security-auditor",
        "security review executed by Baron",
        &receipt.receipt_id,
        &binding,
    )
    .unwrap();

    let passed = gate_evidence_status_strict_for_context(
        &repo,
        &["security-auditor".to_string()],
        Some(&binding),
    )
    .unwrap();
    assert!(passed.passed);
    assert!(
        !gate_evidence_status_strict_for_scope(
            &repo,
            &["security-auditor".to_string()],
            "task-b",
            "codex",
        )
        .unwrap()
        .passed
    );
    assert!(record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "security-auditor",
        "replayed receipt",
        &receipt.receipt_id,
        &binding,
    )
    .is_err());

    let other_task = GateReceiptBinding::new(
        "task-b",
        "operation-a",
        "codex",
        "session-a",
        "request-a",
        "quality:security-auditor",
    );
    let status = gate_evidence_status_strict_for_context(
        &repo,
        &["security-auditor".to_string()],
        Some(&other_task),
    )
    .unwrap();
    assert!(!status.passed);
    assert!(record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "test-engineer",
        "re-attribution attempt",
        &receipt.receipt_id,
        &other_task,
    )
    .is_err());

    let format_binding = GateReceiptBinding::new(
        "task-a",
        "operation-format",
        "codex",
        "session-format",
        "request-format",
        "quality:code-reviewer",
    );
    let (format_receipt, format_binding) =
        execute_authoritative(&repo, command(&repo), format_binding);
    assert!(record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "security-auditor",
        "format receipt re-attribution",
        &format_receipt.receipt_id,
        &format_binding,
    )
    .is_err());
}

#[test]
fn capability_summary_cannot_satisfy_a_required_gate_without_receipt() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    register_provider(&repo, provider()).unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    let fake = CapabilityExecutionEvidence {
        capability: "security-review".to_string(),
        provider: "trusted-runner".to_string(),
        summary: "security looks good".to_string(),
        ..Default::default()
    };
    let gate = evaluate_execution_evidence(&repo, AdapterKind::Codex, &[fake]).unwrap();
    assert!(!gate.passed);
    assert!(gate.gaps.iter().any(|gap| gap.contains("lacks execution")));
}

#[test]
fn capability_gate_requires_the_matching_current_receipt_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    register_provider(&repo, provider()).unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();
    let binding = ReceiptContext::new(
        "task-a",
        "operation-a",
        "codex",
        "session-a",
        "request-a",
        "capability_execution",
    );
    let (receipt, binding) = execute_authoritative(&repo, command(&repo), binding);
    let evidence = CapabilityExecutionEvidence {
        capability: "security-review".to_string(),
        provider: "trusted-runner".to_string(),
        summary: "security review command passed".to_string(),
        receipt_id: Some(receipt.receipt_id.clone()),
        task_id: Some(binding.task_id.clone()),
        operation_id: Some(binding.operation_id.clone()),
        gate_kind: Some(binding.gate_kind.clone()),
        session_id: Some(binding.session_id.clone()),
        request_id: Some(binding.request_id.clone()),
    };
    let operation = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id(binding.session_id.clone())
        .with_request_id(binding.request_id.clone())
        .with_task_id(binding.task_id.clone())
        .with_operation_id(binding.operation_id.clone());
    assert!(
        evaluate_execution_evidence_for_operation(&repo, &operation, &[evidence])
            .unwrap()
            .passed
    );

    let wrong_gate = ReceiptContext::new(
        "task-a",
        "operation-wrong-gate",
        "codex",
        "session-a",
        "request-a",
        "quality:security-auditor",
    );
    let (wrong_receipt, wrong_gate) = execute_authoritative(&repo, command(&repo), wrong_gate);
    let wrong_evidence = CapabilityExecutionEvidence {
        capability: "security-review".to_string(),
        provider: "trusted-runner".to_string(),
        summary: "wrong gate kind".to_string(),
        receipt_id: Some(wrong_receipt.receipt_id),
        task_id: Some(wrong_gate.task_id),
        operation_id: Some(wrong_gate.operation_id),
        gate_kind: Some(wrong_gate.gate_kind),
        session_id: Some(wrong_gate.session_id),
        request_id: Some(wrong_gate.request_id),
    };
    assert!(
        !evaluate_execution_evidence_for_operation(&repo, &operation, &[wrong_evidence])
            .unwrap()
            .passed
    );

    let mut persisted = receipt;
    persisted.provenance = ReceiptProvenance::PersistedDiagnostic;
    assert!(!receipt_is_current_authority(&repo, &persisted).unwrap());
}

#[test]
fn capability_receipt_cannot_cross_operation_session_or_request() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    register_provider(&repo, provider()).unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();
    let receipt_context = ReceiptContext::new(
        "task-a",
        "operation-a",
        "codex",
        "session-a",
        "request-a",
        "capability_execution",
    );
    let (receipt, receipt_context) = execute_authoritative(&repo, command(&repo), receipt_context);
    let evidence = CapabilityExecutionEvidence {
        capability: "security-review".to_string(),
        provider: "trusted-runner".to_string(),
        summary: "security review command passed".to_string(),
        receipt_id: Some(receipt.receipt_id),
        task_id: Some(receipt_context.task_id.clone()),
        operation_id: Some(receipt_context.operation_id.clone()),
        gate_kind: Some(receipt_context.gate_kind.clone()),
        session_id: Some(receipt_context.session_id.clone()),
        request_id: Some(receipt_context.request_id.clone()),
    };
    let operation_a = OperationContext::new(SupportedAdapter::Codex)
        .with_task_id(receipt_context.task_id.clone())
        .with_operation_id(receipt_context.operation_id.clone())
        .with_session_id(receipt_context.session_id.clone())
        .with_request_id(receipt_context.request_id.clone());
    assert!(
        evaluate_execution_evidence_for_operation(
            &repo,
            &operation_a,
            std::slice::from_ref(&evidence)
        )
        .unwrap()
        .passed
    );
    for (task, operation, session, request) in [
        (
            "task-b",
            receipt_context.operation_id.as_str(),
            receipt_context.session_id.as_str(),
            receipt_context.request_id.as_str(),
        ),
        (
            receipt_context.task_id.as_str(),
            "operation-b",
            receipt_context.session_id.as_str(),
            receipt_context.request_id.as_str(),
        ),
        (
            receipt_context.task_id.as_str(),
            receipt_context.operation_id.as_str(),
            "session-b",
            receipt_context.request_id.as_str(),
        ),
        (
            receipt_context.task_id.as_str(),
            receipt_context.operation_id.as_str(),
            receipt_context.session_id.as_str(),
            "request-b",
        ),
    ] {
        let other = OperationContext::new(SupportedAdapter::Codex)
            .with_task_id(task)
            .with_operation_id(operation)
            .with_session_id(session)
            .with_request_id(request);
        assert!(
            !evaluate_execution_evidence_for_operation(
                &repo,
                &other,
                std::slice::from_ref(&evidence),
            )
            .unwrap()
            .passed,
            "receipt should not satisfy {task}/{operation}/{session}/{request}"
        );
    }
}

#[test]
fn prepare_gate_status_cannot_reuse_another_task_or_request_receipt() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let binding = GateReceiptBinding::new(
        "task-a",
        "operation-a",
        "codex",
        "session-a",
        "request-a",
        "quality:test-engineer",
    );
    let (receipt, binding) = execute_authoritative(&repo, command(&repo), binding);
    record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "test-engineer",
        "tests passed for task a",
        &receipt.receipt_id,
        &binding,
    )
    .unwrap();

    let required = ["test-engineer".to_string()];
    assert!(
        gate_evidence_status_strict_for_request(
            &repo,
            &required,
            binding.task_id.as_str(),
            binding.adapter.as_str(),
            Some(binding.session_id.as_str()),
            Some(binding.request_id.as_str()),
        )
        .unwrap()
        .passed
    );
    for (task, session, request) in [
        (
            "task-b",
            binding.session_id.as_str(),
            binding.request_id.as_str(),
        ),
        (
            binding.task_id.as_str(),
            "session-b",
            binding.request_id.as_str(),
        ),
        (
            binding.task_id.as_str(),
            binding.session_id.as_str(),
            "request-b",
        ),
    ] {
        assert!(
            !gate_evidence_status_strict_for_request(
                &repo,
                &required,
                task,
                "codex",
                Some(session),
                Some(request),
            )
            .unwrap()
            .passed
        );
    }
    assert!(
        !gate_evidence_status_strict_for_request(
            &repo,
            &required,
            binding.task_id.as_str(),
            binding.adapter.as_str(),
            None,
            Some(binding.request_id.as_str()),
        )
        .unwrap()
        .passed
    );
}

#[test]
fn gate_status_cannot_reuse_a_receipt_from_another_operation() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let binding = GateReceiptBinding::new(
        "task-a",
        "operation-a",
        "codex",
        "session-a",
        "request-a",
        "quality:security-auditor",
    );
    let (receipt, binding) = execute_authoritative(&repo, command(&repo), binding);
    record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "security-auditor",
        "security review passed for operation a",
        &receipt.receipt_id,
        &binding,
    )
    .unwrap();

    let required = ["security-auditor".to_string()];
    assert!(
        gate_evidence_status_strict_for_operation(
            &repo,
            &required,
            binding.task_id.as_str(),
            binding.operation_id.as_str(),
            binding.adapter.as_str(),
            Some(binding.session_id.as_str()),
            Some(binding.request_id.as_str()),
        )
        .unwrap()
        .passed
    );
    assert!(
        !gate_evidence_status_strict_for_operation(
            &repo,
            &required,
            binding.task_id.as_str(),
            "operation-b",
            binding.adapter.as_str(),
            Some(binding.session_id.as_str()),
            Some(binding.request_id.as_str()),
        )
        .unwrap()
        .passed
    );
}

#[test]
fn source_change_invalidates_a_current_gate_receipt_before_recording() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    fs::write(repo.join("README.md"), "before\n").unwrap();
    let binding = GateReceiptBinding::new(
        "task-source",
        "operation-source",
        "codex",
        "session-source",
        "request-source",
        "quality:test-engineer",
    );
    let (receipt, binding) = execute_authoritative(&repo, command(&repo), binding);
    fs::write(repo.join("README.md"), "after\n").unwrap();
    assert!(record_gate_evidence_with_receipt_bound(
        &repo,
        &context,
        "test-engineer",
        "stale source receipt",
        &receipt.receipt_id,
        &binding,
    )
    .is_err());
}

#[test]
fn recomputed_persisted_receipt_cannot_upgrade_to_current_authority() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let binding = ReceiptContext::new(
        "task-rewrite",
        "operation-rewrite",
        "codex",
        "session-rewrite",
        "request-rewrite",
        "quality:security-auditor",
    );
    let (receipt, _binding) = execute_authoritative(&repo, command(&repo), binding);
    let path = repo.join(".baron/cache/execution-receipts.jsonl");
    let mut rewritten = receipt.clone();
    rewritten.stdout_excerpt = "attacker replaced the observed output".to_string();
    rewritten.integrity_digest.clear();
    rewritten.integrity_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&rewritten).unwrap())
    );
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string(&rewritten).unwrap()),
    )
    .unwrap();

    let loaded = baron_core::execution_receipt::load_receipts(&repo)
        .unwrap()
        .pop()
        .unwrap();
    assert!(!receipt_is_current_authority(&repo, &loaded).unwrap());
}

#[test]
fn proof_receipt_cannot_cross_task_or_use_the_unbound_legacy_api() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let task_a = ReceiptContext::new(
        "task-a",
        "operation-proof",
        "codex",
        "session-proof",
        "request-proof",
        "proof",
    );
    let (receipt, task_a) = execute_authoritative(&repo, command(&repo), task_a);
    assert!(record_proof_from_receipt(&repo, &context, &receipt.receipt_id).is_err());

    let task_b = ReceiptContext::new(
        "task-b",
        "operation-proof",
        "codex",
        "session-proof",
        "request-proof",
        "proof",
    );
    assert!(
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &task_b).is_err()
    );
    assert!(record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &task_a).is_ok());
}

#[test]
fn persisted_receipt_authority_survives_the_creator_process_boundary() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let binding = ReceiptContext::new(
        "task-cross-process",
        "operation-cross-process",
        "codex",
        "session-cross-process",
        "request-cross-process",
        "proof",
    );
    let (receipt, _binding) = execute_authoritative(&repo, command(&repo), binding);
    let verified = load_verified_receipt(&repo, &receipt.receipt_id).unwrap();
    assert_eq!(verified.receipt_id, receipt.receipt_id);
}

#[cfg(unix)]
#[test]
fn receipt_append_rejects_a_symlink_target_before_writing_outside_the_project() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside.jsonl");
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::write(&outside, "outside-before\n").unwrap();
    symlink(&outside, repo.join(".baron/cache/execution-receipts.jsonl")).unwrap();
    let binding = ReceiptContext::new(
        "task-symlink",
        "operation-symlink",
        "codex",
        "session-symlink",
        "request-symlink",
        "proof",
    );

    assert!(execute_command_with_context(command(&repo), binding).is_err());
    assert_eq!(fs::read_to_string(outside).unwrap(), "outside-before\n");
}

#[test]
fn duplicate_receipt_ids_fail_closed_during_raw_load() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    let binding = ReceiptContext::new(
        "task-duplicate",
        "operation-duplicate",
        "codex",
        "session-duplicate",
        "request-duplicate",
        "proof",
    );
    let receipt = execute_command_with_context(command(&repo), binding).unwrap();
    let line = serde_json::to_string(&receipt).unwrap();
    fs::write(
        repo.join(".baron/cache/execution-receipts.jsonl"),
        format!("{line}\n{line}\n"),
    )
    .unwrap();

    assert!(baron_core::execution_receipt::load_receipts(&repo).is_err());
}

#[test]
fn schema_v1_receipts_remain_diagnostic_only() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let receipt = execute_command(command(&repo)).unwrap();
    assert_eq!(receipt.schema_version, 1);
    assert!(load_valid_receipts_for_diagnostics(&repo)
        .unwrap()
        .is_empty());
    assert!(load_verified_receipt(&repo, &receipt.receipt_id).is_err());
}

#[test]
fn historical_schema_v1_without_authority_fields_remains_readable() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let receipt = execute_command(command(&repo)).unwrap();
    let mut historical = receipt.clone();
    historical.authority_key_id = None;
    historical.authority_signature = None;
    historical.integrity_digest.clear();
    historical.integrity_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&historical).unwrap())
    );
    fs::write(
        repo.join(".baron/cache/execution-receipts.jsonl"),
        format!("{}\n", serde_json::to_string(&historical).unwrap()),
    )
    .unwrap();

    let loaded = load_receipts(&repo).unwrap();
    assert_eq!(loaded, vec![historical]);
    assert!(load_valid_receipts_for_diagnostics(&repo)
        .unwrap()
        .is_empty());
}

#[test]
fn receipt_append_rejects_a_non_regular_target() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(repo.join(".baron/cache/execution-receipts.jsonl")).unwrap();
    let binding = ReceiptContext::new(
        "task-directory-target",
        "operation-directory-target",
        "codex",
        "session-directory-target",
        "request-directory-target",
        "proof",
    );

    assert!(execute_command_with_context(command(&repo), binding).is_err());
}

#[test]
fn failed_timed_out_and_foreign_project_receipts_never_verify_as_authority() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let foreign_repo = temp.path().join("foreign-repo");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&foreign_repo).unwrap();

    let failed_binding = ReceiptContext::new(
        "task-failed",
        "operation-failed",
        "codex",
        "session-failed",
        "request-failed",
        "proof",
    );
    let mut failed_request = command(&repo);
    #[cfg(windows)]
    {
        failed_request.arguments = vec!["/C".to_string(), "exit 7".to_string()];
    }
    #[cfg(not(windows))]
    {
        failed_request.arguments = vec!["-c".to_string(), "exit 7".to_string()];
    }
    let failed = execute_command_with_context(failed_request, failed_binding).unwrap();
    assert_eq!(
        failed.result,
        baron_core::execution_receipt::ExecutionResult::Failed
    );
    assert!(load_verified_receipt(&repo, &failed.receipt_id).is_err());

    let timed_out_binding = ReceiptContext::new(
        "task-timeout",
        "operation-timeout",
        "codex",
        "session-timeout",
        "request-timeout",
        "proof",
    );
    let mut timeout_request = command(&repo);
    timeout_request.timeout = Duration::from_millis(1);
    #[cfg(windows)]
    {
        timeout_request.arguments = vec!["/C".to_string(), "ping -n 6 127.0.0.1 >NUL".to_string()];
    }
    #[cfg(not(windows))]
    {
        timeout_request.arguments = vec!["-c".to_string(), "sleep 1".to_string()];
    }
    let timed_out = execute_command_with_context(timeout_request, timed_out_binding).unwrap();
    assert_eq!(
        timed_out.result,
        baron_core::execution_receipt::ExecutionResult::TimedOut
    );
    assert!(load_verified_receipt(&repo, &timed_out.receipt_id).is_err());

    let source = repo.join(".baron/cache/execution-receipts.jsonl");
    let destination = foreign_repo.join(".baron/cache/execution-receipts.jsonl");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::copy(source, &destination).unwrap();
    assert!(load_verified_receipt(&foreign_repo, &failed.receipt_id).is_err());
}

#[cfg(windows)]
#[test]
fn receipt_append_rejects_a_windows_reparse_target() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    let target = repo.join(".baron/cache/execution-receipts.jsonl");
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let linked = std::os::windows::fs::symlink_dir(&outside, &target).is_ok();
    let linked = if linked {
        true
    } else {
        let script = format!(
            "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
            target.display().to_string().replace('\'', "''"),
            outside.display().to_string().replace('\'', "''")
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .map(|status| status.success() && target.is_dir())
            .unwrap_or(false)
    };
    if !linked {
        return;
    }

    let binding = ReceiptContext::new(
        "task-reparse-target",
        "operation-reparse-target",
        "codex",
        "session-reparse-target",
        "request-reparse-target",
        "proof",
    );
    assert!(execute_command_with_context(command(&repo), binding).is_err());
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
}

#[test]
fn arbitrary_receipt_context_is_diagnostic_only() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let binding = ReceiptContext::new(
        "task-worker-0",
        "operation-worker-0",
        "codex",
        "session-worker-0",
        "request-worker-0",
        "proof",
    );
    let receipt = execute_command_with_context(command(&repo), binding).unwrap();
    assert_eq!(receipt.schema_version, 1);
    assert!(load_verified_receipt(&repo, &receipt.receipt_id).is_err());
}

#[test]
fn authority_root_containment_fails_before_child_execution() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let known_machine_home = temp.path().join("known-machine-home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    fs::create_dir_all(&known_machine_home).unwrap();

    let cases = vec![
        ("repo root", repo.clone()),
        ("repo descendant", repo.join("nested-machine-home")),
        ("repo baron", repo.join(".baron")),
        (
            "repo canonical alias",
            repo.join("..")
                .join(repo.file_name().unwrap())
                .join("alias-home"),
        ),
        ("vault root", vault.clone()),
        ("vault descendant", vault.join("nested-machine-home")),
    ];
    for (label, machine_home) in cases {
        let sentinel = temp.path().join(format!("{label}-sentinel"));
        let task = format!("reject unsafe authority root {label}");
        let output = Command::new(env::current_exe().unwrap())
            .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
            .env("BARON_AUTHORITY_TEST_MODE", "containment")
            .env("BARON_TEST_REPO", &repo)
            .env("BARON_TEST_VAULT", &vault)
            .env("BARON_TEST_SENTINEL", &sentinel)
            .env("BARON_TEST_TASK", &task)
            .env("BARON_HOME", &machine_home)
            .env("BARON_VAULT", &vault)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{label} worker failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!sentinel.exists(), "{label} ran the child command");
        assert!(
            !machine_home
                .join("authority/execution-receipt-ed25519.seed")
                .exists(),
            "{label} created a trust root"
        );
    }

    assert!(!known_machine_home.join("authority").exists());

    let current = env::current_dir().unwrap();
    let relative_temp = tempfile::tempdir_in(&current).unwrap();
    let relative_repo = relative_temp.path().join("repo");
    fs::create_dir_all(&relative_repo).unwrap();
    let relative_home = relative_repo.join("relative-machine-home");
    let relative_home = relative_home.strip_prefix(&current).unwrap().to_path_buf();
    let relative_sentinel = relative_repo.join("relative-sentinel");
    let relative_output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", "containment")
        .env("BARON_TEST_REPO", &relative_repo)
        .env("BARON_TEST_VAULT", &vault)
        .env("BARON_TEST_SENTINEL", &relative_sentinel)
        .env("BARON_TEST_TASK", "reject relative authority root")
        .env("BARON_HOME", &relative_home)
        .env("BARON_VAULT", &vault)
        .output()
        .unwrap();
    assert!(
        relative_output.status.success(),
        "relative worker failed: {}",
        String::from_utf8_lossy(&relative_output.stderr)
    );
    assert!(!relative_sentinel.exists());
}

#[cfg(unix)]
#[test]
fn authority_root_rejects_a_vault_symlink_alias_before_child_execution() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let vault_alias = temp.path().join("vault-alias");
    let machine_home = vault.join("nested-machine-home");
    let sentinel = temp.path().join("vault-alias-sentinel");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    symlink(&vault, &vault_alias).unwrap();

    let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", "containment")
        .env("BARON_TEST_REPO", &repo)
        .env("BARON_TEST_VAULT", &vault_alias)
        .env("BARON_TEST_SENTINEL", &sentinel)
        .env("BARON_TEST_TASK", "reject Vault symlink alias")
        .env("BARON_HOME", &machine_home)
        .env("BARON_VAULT", &vault_alias)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Vault-alias worker failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!sentinel.exists());
    assert!(!machine_home
        .join("authority/execution-receipt-ed25519.seed")
        .exists());
}

#[cfg(windows)]
#[test]
fn authority_root_rejects_a_vault_reparse_alias_before_child_execution() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let vault_alias = temp.path().join("vault-alias");
    let machine_home = vault.join("nested-machine-home");
    let sentinel = temp.path().join("vault-reparse-alias-sentinel");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();

    let linked = std::os::windows::fs::symlink_dir(&vault, &vault_alias).is_ok();
    let linked = if linked {
        true
    } else {
        let script = format!(
            "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
            vault_alias.display(),
            vault.display()
        );
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .unwrap()
            .success()
    };
    if !linked {
        return;
    }

    let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", "containment")
        .env("BARON_TEST_REPO", &repo)
        .env("BARON_TEST_VAULT", &vault_alias)
        .env("BARON_TEST_SENTINEL", &sentinel)
        .env("BARON_TEST_TASK", "reject Vault reparse alias")
        .env("BARON_HOME", &machine_home)
        .env("BARON_VAULT", &vault_alias)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Vault-reparse-alias worker failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!sentinel.exists());
    assert!(!machine_home
        .join("authority/execution-receipt-ed25519.seed")
        .exists());
}

#[test]
fn authority_root_external_and_normal_home_paths_succeed() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let machine_home = temp.path().join("machine-home");
    let normal_home = temp.path().join("normal-home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    fs::create_dir_all(&machine_home).unwrap();
    fs::create_dir_all(&normal_home).unwrap();

    let external = authority_worker_output("external", &repo, &vault, &machine_home);
    assert!(
        external.status.success(),
        "external worker failed: {}",
        String::from_utf8_lossy(&external.stderr)
    );
    assert!(machine_home
        .join("authority/execution-receipt-ed25519.seed")
        .is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode =
            fs::symlink_metadata(machine_home.join("authority/execution-receipt-ed25519.seed"))
                .unwrap()
                .permissions()
                .mode();
        assert_eq!(mode & 0o077, 0);
    }
    assert!(!repo.join("authority").exists());
    assert!(!vault.join("authority").exists());

    let normal = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", "normal")
        .env("BARON_TEST_REPO", &repo)
        .env("BARON_TEST_VAULT", &vault)
        .env("BARON_TEST_NORMAL_HOME", &normal_home)
        .env_remove("BARON_HOME")
        .env("BARON_VAULT", &vault)
        .output()
        .unwrap();
    assert!(
        normal.status.success(),
        "normal-home worker failed: {}",
        String::from_utf8_lossy(&normal.stderr)
    );
    assert!(normal_home
        .join(".baron/authority/execution-receipt-ed25519.seed")
        .is_file());
}

#[test]
fn authority_root_rejects_malformed_final_seed_and_failed_staging_without_repair() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let machine_home = temp.path().join("machine-home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    let authority_dir = machine_home.join("authority");
    let seed = authority_dir.join("execution-receipt-ed25519.seed");

    for bytes in [vec![], vec![7_u8], vec![9_u8; 33]] {
        fs::create_dir_all(&authority_dir).unwrap();
        fs::write(&seed, &bytes).unwrap();
        let result = authority_worker_output("malformed", &repo, &vault, &machine_home);
        assert!(
            result.status.success(),
            "malformed-seed worker failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(fs::read(&seed).unwrap(), bytes);
        fs::remove_file(&seed).unwrap();
    }

    let staging_blocker = seed.with_extension("baron-tmp");
    fs::create_dir_all(&authority_dir).unwrap();
    fs::create_dir_all(&staging_blocker).unwrap();
    let result = authority_worker_output("malformed", &repo, &vault, &machine_home);
    assert!(
        result.status.success(),
        "staging-blocker worker failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!seed.exists());
}

#[cfg(unix)]
#[test]
fn authority_root_rejects_a_symlinked_parent_before_child_execution() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let machine_home = temp.path().join("machine-home");
    let outside = temp.path().join("outside-authority");
    let sentinel = temp.path().join("authority-link-sentinel");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    fs::create_dir_all(&machine_home).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, machine_home.join("authority")).unwrap();

    let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", "containment")
        .env("BARON_TEST_REPO", &repo)
        .env("BARON_TEST_VAULT", &vault)
        .env("BARON_TEST_SENTINEL", &sentinel)
        .env("BARON_TEST_TASK", "reject linked authority parent")
        .env("BARON_HOME", &machine_home)
        .env("BARON_VAULT", &vault)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "linked-parent worker failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!sentinel.exists());
    assert!(!outside.join("execution-receipt-ed25519.seed").exists());
}

#[cfg(windows)]
#[test]
fn authority_root_rejects_a_reparse_parent_before_child_execution() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let machine_home = temp.path().join("machine-home");
    let outside = temp.path().join("outside-authority");
    let authority = machine_home.join("authority");
    let sentinel = temp.path().join("authority-reparse-sentinel");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    fs::create_dir_all(&machine_home).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let linked = std::os::windows::fs::symlink_dir(&outside, &authority).is_ok();
    let linked = if linked {
        true
    } else {
        let script = format!(
            "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
            authority.display().to_string().replace('\'', "''"),
            outside.display().to_string().replace('\'', "''")
        );
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .map(|status| status.success() && authority.is_dir())
            .unwrap_or(false)
    };
    if !linked {
        return;
    }

    let output = Command::new(env::current_exe().unwrap())
        .args(["--exact", "receipt_authority_scope_worker", "--nocapture"])
        .env("BARON_AUTHORITY_TEST_MODE", "containment")
        .env("BARON_TEST_REPO", &repo)
        .env("BARON_TEST_VAULT", &vault)
        .env("BARON_TEST_SENTINEL", &sentinel)
        .env("BARON_TEST_TASK", "reject reparse authority parent")
        .env("BARON_HOME", &machine_home)
        .env("BARON_VAULT", &vault)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "reparse-parent worker failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!sentinel.exists());
    assert!(!outside.join("execution-receipt-ed25519.seed").exists());
}

#[test]
fn strict_verified_loader_does_not_hide_invalid_schema_v2_records() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let machine_home = temp.path().join("machine-home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    let output = authority_worker_output("strict", &repo, &vault, &machine_home);
    assert!(
        output.status.success(),
        "strict-loader worker failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
