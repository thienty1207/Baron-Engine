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
use baron_core::plan::start_or_resume_plan_for_operation;
use baron_core::proof::{
    latest_proof, proof_status, record_proof, record_proof_for_operation,
    record_proof_from_receipt_bound, record_proof_with_capabilities_for_operation,
};
use baron_core::trace::{
    latest_trace_score_for_operation, record_trace, record_trace_for_operation, score_trace,
    TraceOperationBinding, TraceOutcome, TraceTier,
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
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::write(
        repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: backend login security\n- Status: `in_progress`\n",
    )
    .unwrap();
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

    assert_eq!(score.achieved, TraceTier::Detailed);
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
        "task-proof",
        SupportedAdapter::Codex,
        Some("session-proof"),
        Some("request-proof"),
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
