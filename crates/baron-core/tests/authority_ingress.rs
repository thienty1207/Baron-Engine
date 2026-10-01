use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::{
    active_plan_authority_for_binding, indexed_active_plan_authority_for_binding,
    start_or_resume_plan, start_or_resume_plan_for_identity, PlanOperationBinding,
};
use baron_core::proof::{record_proof, record_proof_for_operation};
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
    let identity = LifecycleIdentity::resolve(
        &vault.project_id,
        "fix README alpha typo",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, "fix README alpha typo", &identity).unwrap();
    let binding = PlanOperationBinding::from_identity(&identity);
    assert!(indexed_active_plan_authority_for_binding(&repo, &binding)
        .unwrap()
        .is_some());

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
