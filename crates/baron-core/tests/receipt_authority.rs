use std::fs;
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
    execute_command_with_context, receipt_is_current_authority, ExecutionRequest, ReceiptContext,
    ReceiptProvenance,
};
use baron_core::operation::{OperationContext, SupportedAdapter};
use baron_core::proof::{record_proof_from_receipt, record_proof_from_receipt_bound};
use baron_core::vault::ensure_vault;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

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
    let receipt = execute_command_with_context(command(&repo), binding.clone()).unwrap();
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
    let format_receipt =
        execute_command_with_context(command(&repo), format_binding.clone()).unwrap();
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
    let receipt = execute_command_with_context(command(&repo), binding.clone()).unwrap();
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
    let wrong_receipt = execute_command_with_context(command(&repo), wrong_gate.clone()).unwrap();
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
    let receipt = execute_command_with_context(command(&repo), receipt_context.clone()).unwrap();
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
        .with_task_id("task-a")
        .with_operation_id("operation-a")
        .with_session_id("session-a")
        .with_request_id("request-a");
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
        ("task-b", "operation-a", "session-a", "request-a"),
        ("task-a", "operation-b", "session-a", "request-a"),
        ("task-a", "operation-a", "session-b", "request-a"),
        ("task-a", "operation-a", "session-a", "request-b"),
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
    let receipt = execute_command_with_context(command(&repo), binding.clone()).unwrap();
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
            "task-a",
            "codex",
            Some("session-a"),
            Some("request-a"),
        )
        .unwrap()
        .passed
    );
    for (task, session, request) in [
        ("task-b", "session-a", "request-a"),
        ("task-a", "session-b", "request-a"),
        ("task-a", "session-a", "request-b"),
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
            "task-a",
            "codex",
            None,
            Some("request-a"),
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
    let receipt = execute_command_with_context(command(&repo), binding.clone()).unwrap();
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
            "task-a",
            "operation-a",
            "codex",
            Some("session-a"),
            Some("request-a"),
        )
        .unwrap()
        .passed
    );
    assert!(
        !gate_evidence_status_strict_for_operation(
            &repo,
            &required,
            "task-a",
            "operation-b",
            "codex",
            Some("session-a"),
            Some("request-a"),
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
    let receipt = execute_command_with_context(command(&repo), binding.clone()).unwrap();
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
    let receipt = execute_command_with_context(command(&repo), binding).unwrap();
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
    let receipt = execute_command_with_context(command(&repo), task_a.clone()).unwrap();
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
