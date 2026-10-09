use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::control_plane::{
    gate_evidence_status_strict_for_operation, record_gate_evidence_with_receipt_bound,
};
use baron_core::execution_receipt::{
    execute_command_for_identity, ExecutionRequest, ReceiptContext,
};
use baron_core::operation::{
    AuthoritativeLifecycleIdentity, LifecycleIdentity, OperationContext, SupportedAdapter,
};
use baron_core::plan::{
    active_plan_authority_for_binding, active_plan_completion_evidence_status_for_identity,
    indexed_active_plan_authority_for_binding, start_or_resume_plan,
    start_or_resume_plan_for_identity, PlanOperationBinding,
};
use baron_core::proof::{proof_for_operation, record_proof, record_proof_for_operation};
use baron_core::trace::{
    record_trace, record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, result);
            } else if path.file_name().unwrap() != ".baron-mutation.lock" {
                result.insert(path.clone(), fs::read(path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, &mut result);
    result
}

#[test]
fn core_unscoped_trace_record_and_score_reject_multiple_operations_without_writes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let a = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("a"),
        Some("a"),
    )
    .unwrap();
    let b = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README beta typo",
        SupportedAdapter::Claude,
        Some("b"),
        Some("b"),
    )
    .unwrap();
    for (identity, title) in [(&a, "fix README alpha typo"), (&b, "fix README beta typo")] {
        start_or_resume_plan_for_identity(&repo, &vault, title, identity).unwrap();
        let op = OperationContext::from_identity(identity);
        let proof =
            record_proof_for_operation(&repo, &vault, &op, "README verification passed").unwrap();
        record_trace_for_operation(
            &repo,
            &vault,
            "README corrected",
            TraceOutcome::Completed,
            &TraceOperationBinding::from_operation(&op, &proof.id).unwrap(),
        )
        .unwrap();
    }
    let before = snapshot(temp.path());
    let error = score_trace(&repo, &vault, None).unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
    assert_eq!(snapshot(temp.path()), before);
    let error =
        record_trace(&repo, &vault, "must not publish", TraceOutcome::Completed).unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
    assert_eq!(snapshot(temp.path()), before);
}

#[test]
fn identified_resume_does_not_read_current_and_legacy_start_cannot_disambiguate_with_it() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let a = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("a"),
        Some("a"),
    )
    .unwrap();
    let b = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README beta typo",
        SupportedAdapter::Claude,
        Some("b"),
        Some("b"),
    )
    .unwrap();
    let first =
        start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", &a).unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, "fix README beta typo", &b).unwrap();
    fs::write(repo.join("docs/baron/plans/CURRENT.md"), [0xff]).unwrap();
    let resumed =
        start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", &a).unwrap();
    assert_eq!(resumed.repo_path, first.repo_path);
    let before = snapshot(temp.path());
    assert!(start_or_resume_plan(&repo, &vault, "fix README alpha typo")
        .unwrap_err()
        .to_string()
        .contains("ambiguous"));
    assert_eq!(snapshot(temp.path()), before);
}

#[test]
fn evidence_ingress_can_require_indexed_authority_without_removing_legacy_plan_reads() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let authoritative_identity = AuthoritativeLifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let identity = authoritative_identity.as_lifecycle_identity();
    let plan = start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", identity)
        .unwrap();
    let binding = PlanOperationBinding::from_identity(identity);
    assert!(indexed_active_plan_authority_for_binding(&repo, &binding)
        .unwrap()
        .is_some());

    let operation = OperationContext::from_identity(identity);
    let proof = record_proof_for_operation(
        &repo,
        &vault,
        &operation,
        "README verification passed before index loss",
    )
    .unwrap();
    let read_generation = |content: &str| {
        content
            .lines()
            .find_map(|line| line.strip_prefix("authority_generation: "))
            .map(str::to_string)
    };
    let original_generation = read_generation(&fs::read_to_string(&plan.repo_path).unwrap());
    let trace_binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let gate_binding = ReceiptContext::for_identity(identity, "quality:code-reviewer").unwrap();
    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    let gate_receipt = execute_command_for_identity(
        ExecutionRequest {
            capability: "review-authority-check".to_string(),
            provider: "test-runner".to_string(),
            executable: executable.to_string(),
            arguments,
            working_directory: repo.clone(),
            timeout: Duration::from_secs(5),
        },
        &authoritative_identity,
        &gate_binding.gate_kind,
    )
    .unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &vault,
        "README verification before index loss",
        TraceOutcome::Completed,
        &trace_binding,
    )
    .unwrap();
    record_gate_evidence_with_receipt_bound(
        &repo,
        &vault,
        "code-reviewer",
        "review passed before index loss",
        &gate_receipt.receipt_id,
        &gate_binding,
    )
    .unwrap();

    fs::write(
        repo.join("docs/baron/plans/ACTIVE.md"),
        "# Baron Active Plan Index\n",
    )
    .unwrap();
    assert!(active_plan_authority_for_binding(&repo, &binding)
        .unwrap()
        .is_some());
    assert!(indexed_active_plan_authority_for_binding(&repo, &binding)
        .unwrap()
        .is_none());
    fs::remove_file(repo.join("docs/baron/plans/ACTIVE.md")).unwrap();
    let before = snapshot(temp.path());

    let unselected_proof = record_proof(&repo, &vault, "must not publish without ACTIVE");
    assert!(
        unselected_proof.is_err(),
        "frontmatter discovery must not authorize no-selector proof publication"
    );
    assert_eq!(
        snapshot(temp.path()),
        before,
        "failed proof ingress wrote data"
    );

    let selected_proof =
        record_proof_for_operation(&repo, &vault, &operation, "must not publish without ACTIVE");
    assert!(
        selected_proof.is_err(),
        "operation identity alone must not authorize proof publication"
    );
    assert_eq!(
        snapshot(temp.path()),
        before,
        "failed bound proof ingress wrote data"
    );

    let trace_attempt = record_trace_for_operation(
        &repo,
        &vault,
        "must not publish without ACTIVE",
        TraceOutcome::Completed,
        &trace_binding,
    );
    assert!(
        trace_attempt.is_err(),
        "frontmatter discovery must not authorize operation-bound trace publication"
    );
    assert_eq!(
        snapshot(temp.path()),
        before,
        "failed trace ingress wrote data"
    );

    let gate = record_gate_evidence_with_receipt_bound(
        &repo,
        &vault,
        "code-reviewer",
        "review must not publish without ACTIVE",
        &gate_receipt.receipt_id,
        &gate_binding,
    );
    assert!(
        gate.is_err(),
        "a valid receipt must not authorize gate publication without indexed ACTIVE"
    );
    assert_eq!(
        snapshot(temp.path()),
        before,
        "failed gate ingress wrote data"
    );

    start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", identity).unwrap();
    indexed_active_plan_authority_for_binding(&repo, &binding)
        .unwrap()
        .unwrap();
    let restored_generation = read_generation(&fs::read_to_string(&plan.repo_path).unwrap());
    assert_ne!(
        restored_generation, original_generation,
        "same-identity recovery must rotate the evidence authority generation"
    );
    assert!(
        proof_for_operation(
            &repo,
            &ReceiptContext::for_identity(identity, "proof").unwrap()
        )
        .unwrap()
        .is_none(),
        "old-generation proof must not be presented as current after ACTIVE recovery"
    );
    let restored_gate_status = gate_evidence_status_strict_for_operation(
        &repo,
        &["code-reviewer".to_string()],
        identity.task_id(),
        identity.operation_id(),
        identity.adapter().as_str(),
        Some(identity.session_id()),
        Some(identity.request_id()),
    )
    .unwrap();
    assert!(
        !restored_gate_status.passed,
        "old-generation gate receipt must not be presented as current after ACTIVE recovery"
    );
    let restored_trace_score = score_trace(&repo, &vault, Some(&trace.id)).unwrap();
    assert!(
        !restored_trace_score.passed,
        "old-generation trace must not pass scoring after ACTIVE recovery"
    );
    let completion = active_plan_completion_evidence_status_for_identity(&repo, &vault, identity)
        .unwrap()
        .unwrap();
    assert!(
        completion
            .issues
            .iter()
            .any(|issue| issue == "proof is missing"),
        "pre-recovery proof must not become current after ACTIVE is restored"
    );
    assert!(
        completion
            .issues
            .iter()
            .any(|issue| issue == "passing trace is missing"),
        "pre-recovery trace must not become current after ACTIVE is restored"
    );
}

#[test]
fn single_active_core_trace_ingress_does_not_use_a_completed_operations_latest_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let a = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("a"),
        Some("a"),
    )
    .unwrap();
    let b = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README beta typo",
        SupportedAdapter::Claude,
        Some("b"),
        Some("b"),
    )
    .unwrap();
    let mut traces = Vec::new();
    for (identity, title) in [(&a, "fix README alpha typo"), (&b, "fix README beta typo")] {
        start_or_resume_plan_for_identity(&repo, &vault, title, identity).unwrap();
        let op = OperationContext::from_identity(identity);
        let proof =
            record_proof_for_operation(&repo, &vault, &op, "README verification passed").unwrap();
        let trace = record_trace_for_operation(
            &repo,
            &vault,
            "README corrected",
            TraceOutcome::Completed,
            &TraceOperationBinding::from_operation(&op, &proof.id).unwrap(),
        )
        .unwrap();
        score_trace(&repo, &vault, Some(&trace.id)).unwrap();
        traces.push(trace);
    }
    baron_core::plan::complete_plan_for_identity(&repo, &vault, "README verification passed", &b)
        .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", &a).unwrap();
    assert_eq!(
        score_trace(&repo, &vault, None).unwrap().trace_id,
        traces[0].id
    );
    let trace = record_trace(&repo, &vault, "alpha corrected", TraceOutcome::Completed).unwrap();
    assert_eq!(trace.binding.unwrap().operation_id, a.operation_id());
    assert_eq!(trace.proof_id, traces[0].proof_id);
}

#[test]
fn identified_trace_ingress_never_falls_back_to_unbound_legacy_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();

    // These unbound artifacts predate the operation and cannot be attributed
    // to it, even when they are the latest entries in the global indexes.
    record_proof(
        &repo,
        &vault,
        "previous unrelated README verification passed",
    )
    .unwrap();
    record_trace(
        &repo,
        &vault,
        "previous unrelated task",
        TraceOutcome::Completed,
    )
    .unwrap();
    let a = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", &a).unwrap();

    let before = snapshot(temp.path());
    let score_error = score_trace(&repo, &vault, None).unwrap_err();
    assert!(score_error.to_string().contains("requires proof"));
    assert_eq!(snapshot(temp.path()), before);

    let record_error = record_trace(
        &repo,
        &vault,
        "must not borrow a previous proof",
        TraceOutcome::Completed,
    )
    .unwrap_err();
    assert!(record_error.to_string().contains("requires proof"));
    assert_eq!(snapshot(temp.path()), before);
}

#[test]
fn bound_trace_does_not_borrow_another_operations_current_story() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let a = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("a"),
        Some("a"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", &a).unwrap();
    let op = OperationContext::from_identity(&a);
    let proof =
        record_proof_for_operation(&repo, &vault, &op, "README verification passed").unwrap();
    baron_core::harness::start_or_resume_intake(&repo, &vault, "fix README beta typo").unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &vault,
        "alpha corrected",
        TraceOutcome::Completed,
        &TraceOperationBinding::from_operation(&op, &proof.id).unwrap(),
    )
    .unwrap();
    let text = fs::read_to_string(trace.repo_path).unwrap();
    assert!(text.contains("- Current story: `missing`"));
    assert!(!text.contains("fix README beta typo"));
}
