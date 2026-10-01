use std::fs;
use std::process::Command;
use std::time::Duration;

use baron_core::capability::{
    check_capabilities, register_provider, CapabilityExecutionEvidence, CapabilityProvider,
    CheckOptions, ProviderKind, Requirement,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::execution_receipt::{
    execute_command_for_identity, ExecutionRequest, ReceiptContext,
};
use baron_core::harness::start_or_resume_intake;
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::operation::{AuthoritativeLifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::{start_or_resume_plan, start_or_resume_plan_for_operation};
use baron_core::proof::{
    latest_proof, proof_for_operation, proof_status, record_proof, record_proof_for_operation,
    record_proof_from_receipt_bound, record_proof_with_capabilities_for_operation,
};
use baron_core::trace::{
    latest_trace_score_for_operation, record_trace, record_trace_for_operation, score_trace,
    trace_for_operation, TraceOperationBinding, TraceOutcome, TraceTier,
};
use baron_core::vault::ensure_vault;
use chrono::{Duration as ChronoDuration, Local};
use tempfile::tempdir;

fn setup_git(repo: &std::path::Path) {
    Command::new("git").arg("init").arg(repo).output().unwrap();
    Command::new("git")
        .args(["config", "user.email", "baron@example.test"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Baron Test"])
        .current_dir(repo)
        .output()
        .unwrap();
}

fn confirm_intent(repo: &std::path::Path, vault: &baron_core::vault::VaultContext, title: &str) {
    record_intent(
        repo,
        vault,
        IntentBriefInput {
            title: title.to_string(),
            current_behavior: "Current behavior recorded from project evidence.".to_string(),
            target_behavior: "Target behavior confirmed for this story.".to_string(),
            scope: "Only work required by this story.".to_string(),
            non_goals: vec!["No unrelated cleanup.".to_string()],
            constraints: vec!["Preserve existing contracts.".to_string()],
            decisions: vec!["Use the current architecture.".to_string()],
            required_proof: "Focused verification passes.".to_string(),
            unknowns: Vec::new(),
            confirmed: true,
        },
    )
    .unwrap();
}

fn harness_bytes(repo: &std::path::Path, vault: &baron_core::vault::VaultContext) -> Vec<Vec<u8>> {
    [
        repo.join("docs/baron/harness/TEST_MATRIX.md"),
        vault.project_root.join("ProductHarness/TEST_MATRIX.md"),
        repo.join("docs/baron/harness/CURRENT.md"),
        vault.project_root.join("ProductHarness/CURRENT.md"),
    ]
    .into_iter()
    .map(|path| fs::read(path).unwrap())
    .collect()
}

fn assert_scoped_proof_preserves_other_harness(proof_kind: &str) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README alpha",
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let second = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README beta",
        SupportedAdapter::Claude,
        Some("session-b"),
        Some("request-b"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&first);
    start_or_resume_plan_for_operation(&repo, &context, "fix README alpha", &operation).unwrap();
    start_or_resume_plan_for_operation(
        &repo,
        &context,
        "fix README beta",
        &OperationContext::from_identity(&second),
    )
    .unwrap();
    let story = start_or_resume_intake(&repo, &context, "fix README beta").unwrap();
    let before = harness_bytes(&repo, &context);
    let story_before = fs::read(&story.repo_path).unwrap();
    let vault_story_before = fs::read(&story.vault_path).unwrap();
    let proof = match proof_kind {
        "receipt" => {
            let (receipt, binding) = passing_proof_receipt(&repo, &first);
            record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &binding).unwrap()
        }
        "capability" => record_proof_with_capabilities_for_operation(
            &repo,
            &context,
            &operation,
            "README alpha verification passed",
            &[],
        )
        .unwrap(),
        _ => record_proof_for_operation(
            &repo,
            &context,
            &operation,
            "README alpha verification passed",
        )
        .unwrap(),
    };
    assert!(
        harness_bytes(&repo, &context) == before,
        "{proof_kind} proof for A must preserve B harness bytes"
    );
    assert_eq!(fs::read(&story.repo_path).unwrap(), story_before);
    assert_eq!(fs::read(&story.vault_path).unwrap(), vault_story_before);
    let binding = proof.binding.unwrap();
    assert_eq!(binding.operation_id, first.operation_id());
    assert_ne!(binding.operation_id, second.operation_id());
    assert!(proof.repo_path.is_file());
    assert!(proof.vault_path.is_file());
}

#[test]
fn scoped_proof_does_not_promote_another_operations_harness() {
    assert_scoped_proof_preserves_other_harness("free-form");
}

#[test]
fn scoped_receipt_proof_does_not_promote_another_operations_harness() {
    assert_scoped_proof_preserves_other_harness("receipt");
}

#[test]
fn scoped_capability_proof_does_not_promote_another_operations_harness() {
    assert_scoped_proof_preserves_other_harness("capability");
}

#[test]
fn scoped_proof_does_not_promote_a_story_shared_by_two_operations_of_the_same_task() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identities = ["request-a", "request-b"].map(|request| {
        AuthoritativeLifecycleIdentity::resolve(
            &context.project_id,
            "fix README shared",
            SupportedAdapter::Codex,
            Some("shared-session"),
            Some(request),
        )
        .unwrap()
    });
    let operation = OperationContext::from_identity(&identities[0]);
    let plan = start_or_resume_plan_for_operation(&repo, &context, "fix README shared", &operation)
        .unwrap();
    // Model valid persisted concurrent ownership directly: the intake API has
    // no binding, and a same-title plan start may refuse a second identity.
    let second_path = repo.join("docs/baron/plans/shared-second.md");
    let second_content = fs::read_to_string(&plan.repo_path)
        .unwrap()
        .replace(identities[0].operation_id(), identities[1].operation_id())
        .replace(identities[0].request_id(), identities[1].request_id());
    fs::write(&second_path, second_content).unwrap();
    let index_path = repo.join("docs/baron/plans/ACTIVE.md");
    let mut index = fs::read_to_string(&index_path).unwrap();
    index.push_str(&format!(
        "<!-- BARON:ACTIVE-PLAN {} -->\n",
        serde_json::json!({
            "task_id": identities[1].task_id(),
            "operation_id": identities[1].operation_id(),
            "adapter": "codex",
            "session_id": "shared-session",
            "request_id": "request-b",
            "plan_path": "docs/baron/plans/shared-second.md",
            "status": "in_progress",
        })
    ));
    fs::write(index_path, index).unwrap();
    start_or_resume_intake(&repo, &context, "fix README shared").unwrap();
    let before = harness_bytes(&repo, &context);
    record_proof_for_operation(&repo, &context, &operation, "README verification passed").unwrap();
    assert!(
        harness_bytes(&repo, &context) == before,
        "ambiguous task story changed"
    );
}

#[test]
fn scoped_proof_without_exact_active_ownership_does_not_promote_a_matching_story() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README unowned",
        SupportedAdapter::Codex,
        Some("unowned-session"),
        Some("unowned-request"),
    )
    .unwrap();
    start_or_resume_intake(&repo, &context, "fix README unowned").unwrap();
    let before = harness_bytes(&repo, &context);
    record_proof_for_operation(
        &repo,
        &context,
        &OperationContext::from_identity(&identity),
        "README verification passed",
    )
    .unwrap();
    assert!(
        harness_bytes(&repo, &context) == before,
        "unowned story changed"
    );
}

#[test]
fn partial_operation_proof_does_not_promote_the_current_harness() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_intake(&repo, &context, "fix README beta").unwrap();
    let before = harness_bytes(&repo, &context);
    record_proof_with_capabilities_for_operation(
        &repo,
        &context,
        &OperationContext::new(SupportedAdapter::Codex),
        "README alpha verification passed",
        &[],
    )
    .unwrap();
    assert!(
        harness_bytes(&repo, &context) == before,
        "partial operation changed story"
    );
}

#[test]
fn scoped_proof_uses_the_owned_storys_canonical_risk() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "backend login security",
        SupportedAdapter::Codex,
        Some("risk-session"),
        Some("risk-request"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    start_or_resume_plan_for_operation(&repo, &context, "backend login security", &operation)
        .unwrap();
    confirm_intent(&repo, &context, "backend login security");
    start_or_resume_intake(&repo, &context, "backend login security").unwrap();
    let other = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README beta",
        SupportedAdapter::Claude,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
    start_or_resume_plan_for_operation(
        &repo,
        &context,
        "fix README beta",
        &OperationContext::from_identity(&other),
    )
    .unwrap();
    let current = repo.join("docs/baron/harness/CURRENT.md");
    let content = fs::read_to_string(&current)
        .unwrap()
        .replace("Risk: `high`", "Risk: `low`");
    fs::write(current, content).unwrap();
    record_proof_for_operation(&repo, &context, &operation, "cargo test passed").unwrap();
    for path in [
        repo.join("docs/baron/harness/TEST_MATRIX.md"),
        context.project_root.join("ProductHarness/TEST_MATRIX.md"),
    ] {
        let matrix = fs::read_to_string(path).unwrap();
        assert!(
            matrix.contains("| backend login security | high | insufficient | cargo test passed |")
        );
    }
}

#[test]
fn scoped_proof_rejects_malformed_owned_story_before_publication() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README alpha",
        SupportedAdapter::Codex,
        Some("story-session"),
        Some("story-request"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    start_or_resume_plan_for_operation(&repo, &context, "fix README alpha", &operation).unwrap();
    let story = start_or_resume_intake(&repo, &context, "fix README alpha").unwrap();
    let content = fs::read_to_string(&story.repo_path)
        .unwrap()
        .replace("Risk: `low`", "Risk: `high`");
    fs::write(story.repo_path, content).unwrap();
    let before = harness_bytes(&repo, &context);
    let result = record_proof_for_operation(&repo, &context, &operation, "README passed");
    assert!(
        result.is_err(),
        "malformed owned story must fail before publishing proof"
    );
    assert!(!repo.join("docs/baron/proofs").exists());
    assert!(!context.project_root.join("Proofs/INDEX.md").exists());
    assert!(harness_bytes(&repo, &context) == before);
}

#[test]
fn proof_record_is_written_to_repo_and_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    confirm_intent(&repo, &context, "frontend dashboard flow");
    start_or_resume_intake(&repo, &context, "frontend dashboard flow").unwrap();
    let proof = record_proof(&repo, &context, "cargo test passed: 42 tests").unwrap();

    assert!(proof.repo_path.exists());
    assert!(proof.vault_path.exists());
    assert!(proof_status(&repo).unwrap().contains("42 tests"));
    let repo_matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    let vault_matrix =
        fs::read_to_string(context.project_root.join("ProductHarness/TEST_MATRIX.md")).unwrap();
    for matrix in [repo_matrix, vault_matrix] {
        assert!(matrix.contains(
            "| frontend dashboard flow | medium | verified | cargo test passed: 42 tests |"
        ));
    }
}

#[test]
fn latest_selection_orders_new_artifacts_after_legacy_timestamp_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let proof = record_proof(&repo, &context, "new proof is current").unwrap();
    let date = Local::now().format("%Y-%m-%d").to_string();
    let legacy_stamp = (Local::now() - ChronoDuration::seconds(1))
        .format("%Y%m%d%H%M%S%3f")
        .to_string();
    let legacy_proof = repo
        .join("docs/baron/proofs")
        .join(&date)
        .join(format!("{legacy_stamp}.md"));
    fs::copy(&proof.repo_path, &legacy_proof).unwrap();

    let selected = latest_proof(&repo).unwrap().unwrap();
    assert_eq!(selected.repo_path, proof.repo_path);

    let trace = record_trace(
        &repo,
        &context,
        "new trace is current",
        TraceOutcome::Completed,
    )
    .unwrap();
    let legacy_trace = repo
        .join("docs/baron/traces")
        .join(&date)
        .join(format!("{legacy_stamp}.md"));
    fs::copy(&trace.repo_path, &legacy_trace).unwrap();
    let score = score_trace(&repo, &context, None).unwrap();
    assert_eq!(score.trace_id, trace.id);
    assert!(fs::read_to_string(&trace.repo_path)
        .unwrap()
        .contains("BARON:TRACE-SCORE:START"));
    assert!(!fs::read_to_string(&legacy_trace)
        .unwrap()
        .contains("BARON:TRACE-SCORE:START"));
}

#[test]
fn operation_bound_selection_prefers_new_ids_over_older_legacy_aliases() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "legacy selector README note",
        SupportedAdapter::Codex,
        Some("legacy-selector-session"),
        Some("legacy-selector-request"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    start_or_resume_plan_for_operation(&repo, &context, "legacy selector README note", &operation)
        .unwrap();
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &operation,
        "legacy selector proof is current",
    )
    .unwrap();
    let expected = proof.binding.clone().unwrap();
    let date = Local::now().format("%Y-%m-%d").to_string();
    let legacy_stamp = (Local::now() - ChronoDuration::seconds(1))
        .format("%Y%m%d%H%M%S%3f")
        .to_string();
    let legacy_proof = repo
        .join("docs/baron/proofs")
        .join(&date)
        .join(format!("{legacy_stamp}.md"));
    fs::copy(&proof.repo_path, &legacy_proof).unwrap();
    let selected_proof = proof_for_operation(&repo, &expected).unwrap().unwrap();
    assert_eq!(selected_proof.repo_path, proof.repo_path);

    let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "legacy selector trace is current",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    let legacy_trace = repo
        .join("docs/baron/traces")
        .join(&date)
        .join(format!("{legacy_stamp}.md"));
    fs::copy(&trace.repo_path, &legacy_trace).unwrap();
    let selected_trace = trace_for_operation(&repo, &binding).unwrap().unwrap();
    assert_eq!(selected_trace.repo_path, trace.repo_path);
}

#[test]
fn foreign_markdown_in_proof_and_trace_trees_is_ignored_by_selectors() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "docs foreign artifact selector task",
        SupportedAdapter::Codex,
        Some("foreign-artifact-session"),
        Some("foreign-artifact-request"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    start_or_resume_plan_for_operation(
        &repo,
        &context,
        "docs foreign artifact selector task",
        &operation,
    )
    .unwrap();
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &operation,
        "foreign artifact selector proof passed",
    )
    .unwrap();
    let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "foreign artifact selector trace passed",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();

    std::thread::sleep(Duration::from_millis(10));
    let proof_notes = proof.repo_path.parent().unwrap().join("notes.md");
    fs::write(
        &proof_notes,
        "# Notes\n\n- Operation ID: `forged-proof-operation`\n",
    )
    .unwrap();
    let trace_notes = trace.repo_path.parent().unwrap().join("debug.md");
    fs::write(
        &trace_notes,
        "# Debug\n\n- Operation ID: `forged-trace-operation`\n",
    )
    .unwrap();
    fs::write(
        proof.repo_path.parent().unwrap().join("binary.md"),
        [0xff, 0xfe],
    )
    .unwrap();
    fs::write(
        trace.repo_path.parent().unwrap().join("binary.md"),
        [0xff, 0xfe],
    )
    .unwrap();

    assert_eq!(
        latest_proof(&repo).unwrap().unwrap().repo_path,
        proof.repo_path
    );
    assert_eq!(
        proof_for_operation(&repo, &proof.binding.clone().unwrap())
            .unwrap()
            .unwrap()
            .repo_path,
        proof.repo_path
    );
    assert_eq!(
        score_trace(&repo, &context, None).unwrap().trace_id,
        trace.id
    );
    assert_eq!(
        trace_for_operation(&repo, &binding)
            .unwrap()
            .unwrap()
            .repo_path,
        trace.repo_path
    );
}

#[test]
fn non_timestamp_legacy_proof_and_trace_files_remain_selectable() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("legacy-named-session"),
        Some("legacy-named-request"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    start_or_resume_plan_for_operation(&repo, &context, "fix README typo", &operation).unwrap();
    let proof = record_proof_for_operation(
        &repo,
        &context,
        &operation,
        "legacy named proof remains readable",
    )
    .unwrap();
    let expected = proof.binding.clone().unwrap();
    let legacy_proof = proof.repo_path.parent().unwrap().join("legacy-proof.md");
    let legacy_vault_proof = proof.vault_path.parent().unwrap().join("legacy-proof.md");
    fs::copy(&proof.repo_path, &legacy_proof).unwrap();
    fs::copy(&proof.vault_path, &legacy_vault_proof).unwrap();

    let selected_proof = proof_for_operation(&repo, &expected).unwrap().unwrap();
    assert_eq!(selected_proof.repo_path, legacy_proof);

    let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "legacy named trace remains readable",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    let legacy_trace = trace.repo_path.parent().unwrap().join("legacy-trace.md");
    let legacy_vault_trace = trace.vault_path.parent().unwrap().join("legacy-trace.md");
    fs::copy(&trace.repo_path, &legacy_trace).unwrap();
    fs::copy(&trace.vault_path, &legacy_vault_trace).unwrap();

    let selected_trace = trace_for_operation(&repo, &binding).unwrap().unwrap();
    assert_eq!(selected_trace.repo_path, legacy_trace);

    let score = score_trace(&repo, &context, Some("legacy-trace")).unwrap();
    assert_eq!(score.trace_id, trace.id);
    assert!(fs::read_to_string(&legacy_trace)
        .unwrap()
        .contains("BARON:TRACE-SCORE:START"));
}

#[test]
fn weak_high_risk_proof_remains_insufficient_in_validation_matrix() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    confirm_intent(&repo, &context, "backend login security");
    start_or_resume_intake(&repo, &context, "backend login security").unwrap();

    record_proof(&repo, &context, "manual check completed").unwrap();

    let matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    assert!(matrix
        .contains("| backend login security | high | insufficient | manual check completed |"));
    assert!(!matrix.contains("| backend login security | high | verified |"));
}

#[test]
fn low_risk_trace_with_summary_and_outcome_passes_minimal() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_intake(&repo, &context, "fix README typo").unwrap();

    let trace = record_trace(
        &repo,
        &context,
        "Corrected README typo",
        TraceOutcome::Completed,
    )
    .unwrap();
    let score = score_trace(&repo, &context, Some(&trace.id)).unwrap();

    assert_eq!(score.achieved, TraceTier::Minimal);
    assert_eq!(score.required, TraceTier::Minimal);
    assert!(score.passed);
}

#[test]
fn medium_risk_trace_without_plan_and_proof_fails_standard() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    confirm_intent(&repo, &context, "frontend dashboard flow");
    start_or_resume_intake(&repo, &context, "frontend dashboard flow").unwrap();

    let trace = record_trace(
        &repo,
        &context,
        "Implemented dashboard state",
        TraceOutcome::Completed,
    )
    .unwrap();
    let score = score_trace(&repo, &context, Some(&trace.id)).unwrap();

    assert_eq!(score.required, TraceTier::Standard);
    assert!(!score.passed);
    assert!(score.missing_fields.contains(&"current plan".to_string()));
    assert!(score.missing_fields.contains(&"proof".to_string()));
}

#[test]
fn high_risk_trace_with_plan_story_proof_and_files_passes_detailed() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("src")).unwrap();
    setup_git(&repo);
    fs::write(repo.join("README.md"), "# Demo\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&repo)
        .output()
        .unwrap();
    fs::write(repo.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    confirm_intent(&repo, &context, "backend login security");
    start_or_resume_intake(&repo, &context, "backend login security").unwrap();
    start_or_resume_plan(&repo, &context, "backend login security").unwrap();
    record_proof(
        &repo,
        &context,
        "cargo test auth passed; security authorization and tenant impact verified",
    )
    .unwrap();

    let trace = record_trace(
        &repo,
        &context,
        "Implemented backend login with verified authorization",
        TraceOutcome::Completed,
    )
    .unwrap();
    let score = score_trace(&repo, &context, Some(&trace.id)).unwrap();

    assert_eq!(
        score.achieved,
        TraceTier::Detailed,
        "{score:?}\n{}",
        fs::read_to_string(&trace.repo_path).unwrap()
    );
    assert_eq!(score.required, TraceTier::Detailed);
    assert!(score.passed);
    assert!(score.missing_fields.is_empty());
    assert!(fs::read_to_string(&trace.repo_path)
        .unwrap()
        .contains("src/auth.rs"));
}

#[test]
fn high_risk_trace_does_not_count_baron_state_as_product_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    setup_git(&repo);
    fs::write(repo.join("README.md"), "# Demo\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&repo)
        .output()
        .unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    baron_core::plan::start_or_resume_plan(&repo, &context, "backend login security").unwrap();
    confirm_intent(&repo, &context, "backend login security");
    start_or_resume_intake(&repo, &context, "backend login security").unwrap();
    record_proof(
        &repo,
        &context,
        "cargo test auth passed; security authorization and tenant impact verified",
    )
    .unwrap();

    let trace = record_trace(
        &repo,
        &context,
        "Claimed backend login implementation without product changes",
        TraceOutcome::Completed,
    )
    .unwrap();
    let score = score_trace(&repo, &context, Some(&trace.id)).unwrap();

    assert!(!score.passed);
    assert!(score.missing_fields.contains(&"files changed".to_string()));
    let trace_content = fs::read_to_string(trace.repo_path).unwrap();
    assert!(trace_content.contains("## Files Changed\n\n- none detected"));
}

#[test]
fn scoring_updates_repo_and_vault_trace() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_intake(&repo, &context, "fix docs copy").unwrap();
    let trace = record_trace(
        &repo,
        &context,
        "Updated documentation copy",
        TraceOutcome::Completed,
    )
    .unwrap();

    score_trace(&repo, &context, Some(&trace.id)).unwrap();

    assert!(fs::read_to_string(&trace.repo_path)
        .unwrap()
        .contains("Trace Quality Score"));
    assert!(fs::read_to_string(&trace.vault_path)
        .unwrap()
        .contains("Trace Quality Score"));
    let repo_index = fs::read_to_string(repo.join("docs/baron/traces/INDEX.md")).unwrap();
    let vault_index = fs::read_to_string(context.project_root.join("Traces/INDEX.md")).unwrap();
    for index in [repo_index, vault_index] {
        assert!(index.contains(&format!(
            "`{}` - completed - score: `minimal/minimal` - passed: `yes`",
            trace.id
        )));
    }
}

fn register_required_git(repo: &std::path::Path, vault: &std::path::Path) {
    initialize_project(repo, AdapterKind::Codex, vault).unwrap();
    register_provider(
        repo,
        CapabilityProvider {
            name: "git-cli".to_string(),
            capability: "source-control".to_string(),
            kind: ProviderKind::Cli,
            requirement: Requirement::Required,
            command: Some("git".to_string()),
            scan_target: None,
            adapters: Vec::new(),
            description: "Provides repository state and change evidence.".to_string(),
        },
    )
    .unwrap();
    check_capabilities(
        repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();
}

#[test]
fn provider_presence_alone_does_not_satisfy_required_execution_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    register_required_git(&repo, &vault);
    start_or_resume_intake(&repo, &context, "fix README typo").unwrap();

    let proof = record_proof(&repo, &context, "README text verified").unwrap();
    let content = fs::read_to_string(proof.repo_path).unwrap();

    assert!(content.contains("- Capability gate: `failed`"));
    assert!(content.contains("source-control lacks execution evidence"));
    let matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    assert!(matrix.contains("| fix README typo | low | insufficient |"));
}

#[test]
fn structured_execution_evidence_satisfies_present_required_capability() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    register_required_git(&repo, &vault);
    start_or_resume_intake(&repo, &context, "fix README typo").unwrap();

    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("session-proof"),
        Some("request-proof"),
    )
    .unwrap();
    start_or_resume_plan_for_operation(
        &repo,
        &context,
        "fix README typo",
        &OperationContext::from_identity(&identity),
    )
    .unwrap();
    let binding = ReceiptContext::for_identity(&identity, "capability_execution").unwrap();
    let receipt = execute_command_for_identity(
        ExecutionRequest {
            capability: "source-control".to_string(),
            provider: "git-cli".to_string(),
            executable: executable.to_string(),
            arguments,
            working_directory: repo.clone(),
            timeout: Duration::from_secs(5),
        },
        &identity,
        &binding.gate_kind,
    )
    .unwrap();
    let proof = record_proof_with_capabilities_for_operation(
        &repo,
        &context,
        &OperationContext::new(SupportedAdapter::Codex)
            .with_task_id(binding.task_id.clone())
            .with_operation_id(binding.operation_id.clone())
            .with_session_id(binding.session_id.clone())
            .with_request_id(binding.request_id.clone()),
        "README text verified",
        &[CapabilityExecutionEvidence {
            capability: "source-control".to_string(),
            provider: "git-cli".to_string(),
            summary: "git status completed and repository state was inspected".to_string(),
            receipt_id: Some(receipt.receipt_id),
            task_id: Some(binding.task_id.clone()),
            operation_id: Some(binding.operation_id.clone()),
            gate_kind: Some(binding.gate_kind.clone()),
            session_id: Some(binding.session_id.clone()),
            request_id: Some(binding.request_id.clone()),
        }],
    )
    .unwrap();
    let content = fs::read_to_string(proof.repo_path).unwrap();

    assert!(content.contains("- Capability gate: `passed`"));
    assert!(content.contains("source-control"));
    assert!(content.contains("git status completed"));
    let matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    assert!(matrix.contains("| fix README typo | low | verified |"));
}

#[test]
fn trace_score_inherits_failed_required_capability_gate() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    register_required_git(&repo, &vault);
    start_or_resume_intake(&repo, &context, "fix README typo").unwrap();
    record_proof(&repo, &context, "README text verified").unwrap();

    let trace = record_trace(
        &repo,
        &context,
        "Corrected README typo",
        TraceOutcome::Completed,
    )
    .unwrap();
    let score = score_trace(&repo, &context, Some(&trace.id)).unwrap();

    assert!(!score.passed);
    assert!(score
        .missing_fields
        .contains(&"required capability execution evidence".to_string()));
}

#[test]
fn missing_optional_capability_does_not_block_proof_or_trace() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    register_provider(
        &repo,
        CapabilityProvider {
            name: "optional-linter".to_string(),
            capability: "lint".to_string(),
            kind: ProviderKind::Binary,
            requirement: Requirement::Optional,
            command: Some("baron-definitely-missing".to_string()),
            scan_target: None,
            adapters: Vec::new(),
            description: "Provides optional project lint diagnostics.".to_string(),
        },
    )
    .unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();
    start_or_resume_intake(&repo, &context, "fix README typo").unwrap();
    let proof = record_proof(&repo, &context, "README text verified").unwrap();
    assert!(fs::read_to_string(proof.repo_path)
        .unwrap()
        .contains("- Capability gate: `passed`"));

    let trace = record_trace(
        &repo,
        &context,
        "Corrected README typo",
        TraceOutcome::Completed,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );
}

fn passing_proof_receipt(
    repo: &std::path::Path,
    identity: &AuthoritativeLifecycleIdentity,
) -> (
    baron_core::execution_receipt::ExecutionReceipt,
    ReceiptContext,
) {
    let binding = ReceiptContext::for_identity(identity, "proof").unwrap();
    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    let receipt = execute_command_for_identity(
        ExecutionRequest {
            capability: "proof".to_string(),
            provider: "test-runner".to_string(),
            executable: executable.to_string(),
            arguments,
            working_directory: repo.to_path_buf(),
            timeout: Duration::from_secs(5),
        },
        identity,
        &binding.gate_kind,
    )
    .unwrap();
    (receipt, binding)
}

#[test]
fn receipt_bound_proof_is_complete_on_first_publication() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "receipt-bound proof publication",
        SupportedAdapter::Codex,
        Some("proof-session"),
        Some("proof-request"),
    )
    .unwrap();
    let (receipt, binding) = passing_proof_receipt(&repo, &identity);

    let proof =
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &binding).unwrap();
    let content = fs::read_to_string(&proof.repo_path).unwrap();

    for expected in [
        format!("- Proof ID: `{}`", proof.id),
        format!("- Receipt ID: `{}`", receipt.receipt_id),
        format!("- Task ID: `{}`", identity.task_id()),
        format!("- Operation ID: `{}`", identity.operation_id()),
        "- Adapter: `codex`".to_string(),
        "- Session ID: `proof-session`".to_string(),
        "- Request ID: `proof-request`".to_string(),
        "- Gate kind: `proof`".to_string(),
        format!("- Source fingerprint: `{}`", receipt.source_fingerprint),
    ] {
        assert!(content.contains(&expected), "missing {expected}");
    }
    assert_eq!(content.matches("## Trusted Execution Receipt").count(), 1);
    assert_eq!(
        proof.receipt_id.as_deref(),
        Some(receipt.receipt_id.as_str())
    );
}

#[test]
fn receipt_bound_proof_rejects_source_changes_before_publication() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "receipt-bound README note",
        SupportedAdapter::Codex,
        Some("stale-proof-session"),
        Some("stale-proof-request"),
    )
    .unwrap();
    let (receipt, binding) = passing_proof_receipt(&repo, &identity);
    fs::write(repo.join("README.md"), "changed after receipt\n").unwrap();

    assert!(
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &binding).is_err()
    );
    let proof_files = fs::read_dir(context.project_root.join("Proofs"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .count();
    assert_eq!(proof_files, 0);
}

#[test]
fn bound_proof_write_failure_does_not_promote_repo_or_validation_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    confirm_intent(&repo, &context, "fix README typo");
    start_or_resume_intake(&repo, &context, "fix README typo").unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("failure-session"),
        Some("failure-request"),
    )
    .unwrap();
    let (receipt, binding) = passing_proof_receipt(&repo, &identity);

    fs::create_dir_all(repo.join("docs/baron")).unwrap();
    fs::write(repo.join("docs/baron/proofs"), "injected write failure").unwrap();
    assert!(
        record_proof_from_receipt_bound(&repo, &context, &receipt.receipt_id, &binding,).is_err()
    );

    assert!(repo.join("docs/baron/proofs").is_file());
    let proof_files = fs::read_dir(context.project_root.join("Proofs"))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().is_file() && entry.file_name() != "INDEX.md")
                .count()
        })
        .unwrap_or_default();
    assert_eq!(proof_files, 0);
    let matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    assert!(!matrix.contains("| fix README typo | low | verified |"));
}

#[test]
fn operation_bound_trace_rejects_a_cross_operation_proof_reference() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "trace operation one",
        SupportedAdapter::Codex,
        Some("trace-session-one"),
        Some("trace-request-one"),
    )
    .unwrap();
    let second = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "trace operation two",
        SupportedAdapter::Codex,
        Some("trace-session-two"),
        Some("trace-request-two"),
    )
    .unwrap();
    let first_operation = OperationContext::from_identity(&first);
    let second_operation = OperationContext::from_identity(&second);
    let first_proof = record_proof_for_operation(
        &repo,
        &context,
        &first_operation,
        "README verification passed",
    )
    .unwrap();
    let second_binding =
        TraceOperationBinding::from_operation(&second_operation, &first_proof.id).unwrap();

    let error = record_trace_for_operation(
        &repo,
        &context,
        "Cross-operation trace must fail",
        TraceOutcome::Completed,
        &second_binding,
    )
    .unwrap_err();
    assert!(error.to_string().contains("proof"));
}

#[test]
fn operation_trace_score_recomputes_after_persisted_score_tampering() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let identity = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "fix README typo",
        SupportedAdapter::Codex,
        Some("score-session"),
        Some("score-request"),
    )
    .unwrap();
    let operation = OperationContext::from_identity(&identity);
    start_or_resume_plan_for_operation(&repo, &context, "fix README typo", &operation).unwrap();
    let proof =
        record_proof_for_operation(&repo, &context, &operation, "README verification passed")
            .unwrap();
    let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &repo,
        &context,
        "README typo corrected",
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    let content = fs::read_to_string(&trace.repo_path).unwrap();
    let tampered = content
        .replace("## Task Summary\n\n", "## Task Summary Removed\n\n")
        .replace("- Achieved: `minimal`", "- Achieved: `detailed`");
    fs::write(&trace.repo_path, tampered).unwrap();

    let fresh = latest_trace_score_for_operation(&repo, &binding)
        .unwrap()
        .expect("fresh evaluation should find the exact trace");
    assert_eq!(fresh.achieved, TraceTier::Incomplete);
    assert!(!fresh.passed);

    let without_score = fs::read_to_string(&trace.repo_path)
        .unwrap()
        .split("<!-- BARON:TRACE-SCORE:START -->")
        .next()
        .unwrap()
        .trim_end()
        .to_string();
    fs::write(&trace.repo_path, format!("{without_score}\n")).unwrap();
    let fresh_without_cache = latest_trace_score_for_operation(&repo, &binding)
        .unwrap()
        .expect("fresh evaluation must not require a cached score block");
    assert_eq!(fresh_without_cache.achieved, TraceTier::Incomplete);
    assert!(!fresh_without_cache.passed);
}

#[test]
fn operation_trace_fresh_evaluation_rejects_rewritten_header_with_stale_score() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let first = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "active README task",
        SupportedAdapter::Codex,
        Some("active-session"),
        Some("active-request"),
    )
    .unwrap();
    let first_operation = OperationContext::from_identity(&first);
    start_or_resume_plan_for_operation(&repo, &context, "active README task", &first_operation)
        .unwrap();
    let first_proof = record_proof_for_operation(
        &repo,
        &context,
        &first_operation,
        "Active README verification passed",
    )
    .unwrap();
    let second = AuthoritativeLifecycleIdentity::resolve(
        &context.project_id,
        "unrelated README task",
        SupportedAdapter::Claude,
        Some("other-session"),
        Some("other-request"),
    )
    .unwrap();
    let second_operation = OperationContext::from_identity(&second);
    start_or_resume_plan_for_operation(&repo, &context, "unrelated README task", &second_operation)
        .unwrap();
    let second_proof = record_proof_for_operation(
        &repo,
        &context,
        &second_operation,
        "Unrelated README verification passed",
    )
    .unwrap();
    let second_binding =
        TraceOperationBinding::from_operation(&second_operation, &second_proof.id).unwrap();
    let second_trace = record_trace_for_operation(
        &repo,
        &context,
        "Unrelated README task completed",
        TraceOutcome::Completed,
        &second_binding,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&second_trace.id))
            .unwrap()
            .passed
    );

    let content = fs::read_to_string(&second_trace.repo_path).unwrap();
    let rewritten = content
        .replace(
            &format!("- Task ID: `{}`", second.task_id()),
            &format!("- Task ID: `{}`", first.task_id()),
        )
        .replace(
            &format!("- Operation ID: `{}`", second.operation_id()),
            &format!("- Operation ID: `{}`", first.operation_id()),
        )
        .replace(
            &format!("- Adapter: `{}`", second.adapter().as_str()),
            &format!("- Adapter: `{}`", first.adapter().as_str()),
        )
        .replace(
            &format!("- Session ID: `{}`", second.session_id()),
            &format!("- Session ID: `{}`", first.session_id()),
        )
        .replace(
            &format!("- Request ID: `{}`", second.request_id()),
            &format!("- Request ID: `{}`", first.request_id()),
        )
        .replace(
            &format!("- Proof ID: `{}`", second_proof.id),
            &format!("- Proof ID: `{}`", first_proof.id),
        )
        .replace("## Task Summary\n\n", "## Task Summary Removed\n\n");
    fs::write(&second_trace.repo_path, rewritten).unwrap();

    let first_binding =
        TraceOperationBinding::from_operation(&first_operation, &first_proof.id).unwrap();
    let fresh = latest_trace_score_for_operation(&repo, &first_binding)
        .unwrap()
        .expect("rewritten trace should be found by its forged header");
    assert_eq!(fresh.achieved, TraceTier::Incomplete);
    assert!(!fresh.passed);
}
