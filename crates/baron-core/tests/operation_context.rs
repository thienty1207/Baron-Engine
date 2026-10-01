use std::fs;
use std::path::{Path, PathBuf};

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::context::{
    compile_context_for_lifecycle_identity, compile_context_for_operation,
    compile_context_for_task, ContextTarget,
};
use baron_core::continuity::{
    record_continuity_checkpoint_for_event, record_continuity_checkpoint_for_operation,
    record_recovery, RecoveryInput, RecoveryOutcome,
};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::start_or_resume_plan_for_identity;
use baron_core::prepare::{prepare, PrepareRequestV1};
use baron_core::proof::record_proof_for_operation;
use baron_core::task_state::{compile_task_state, compile_task_state_for_operation};
use baron_core::trace::{
    record_trace_for_operation, score_trace, TraceOperationBinding, TraceOutcome,
};
use baron_core::vault::{ensure_vault, VaultContext};
use tempfile::{tempdir, TempDir};

const A_TASK: &str = "fix README alpha typo";
const B_TASK: &str = "fix README beta typo";

struct Fixture {
    _temp: TempDir,
    repo: PathBuf,
    vault_root: PathBuf,
    vault: VaultContext,
    a: LifecycleIdentity,
    b: LifecycleIdentity,
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn fixture(b_task: &str) -> Fixture {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("operation-context");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let a = LifecycleIdentity::resolve(
        &vault.project_id,
        A_TASK,
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let b = LifecycleIdentity::resolve(
        &vault.project_id,
        b_task,
        SupportedAdapter::Claude,
        Some("session-b"),
        Some("request-b"),
    )
    .unwrap();
    Fixture {
        _temp: temp,
        repo,
        vault_root,
        vault,
        a,
        b,
    }
}

fn plans(f: &Fixture, b_task: &str) {
    start_or_resume_plan_for_identity(&f.repo, &f.vault, A_TASK, &f.a).unwrap();
    start_or_resume_plan_for_identity(&f.repo, &f.vault, b_task, &f.b).unwrap();
}

fn foreign_shared_state(f: &Fixture) {
    write(&f.repo.join("docs/baron/harness/CURRENT_INTENT.md"),
        "# Baron Intent Brief\n\n- Title: B-only intent\n- Confirmation: `confirmed`\n\n## Target Behavior\n\nB-only target\n\n## Constraints\n\n- B-only constraint\n");
    write(
        &f.repo.join("docs/baron/harness/CURRENT.md"),
        "# Product Harness\n\n- Title: B-only story\n- Risk: `high`\n",
    );
    record_recovery(
        &f.repo,
        &f.vault,
        RecoveryInput {
            outcome: RecoveryOutcome::Interrupted,
            root_cause: "B-only blocker".into(),
            last_successful_step: "B-only successful step".into(),
            evidence: vec!["B-only evidence".into()],
            affected_files: vec!["B-only.rs".into()],
            next_action: "B-only recovery action".into(),
            retry_conditions: vec![],
        },
    )
    .unwrap();
    record_continuity_checkpoint_for_operation(
        &f.repo,
        &f.vault,
        "B-only checkpoint",
        &OperationContext::from_identity(&f.b),
    )
    .unwrap();
}

fn evidence(f: &Fixture, identity: &LifecycleIdentity, summary: &str) -> String {
    let operation = OperationContext::from_identity(identity);
    let proof = record_proof_for_operation(&f.repo, &f.vault, &operation, summary).unwrap();
    let binding = TraceOperationBinding::from_operation(&operation, &proof.id).unwrap();
    let trace = record_trace_for_operation(
        &f.repo,
        &f.vault,
        summary,
        TraceOutcome::Completed,
        &binding,
    )
    .unwrap();
    score_trace(&f.repo, &f.vault, Some(&trace.id)).unwrap();
    proof.id
}

#[test]
fn identified_checkpoint_uses_a_plan_and_evidence_while_current_and_latest_are_b() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    let a_proof = evidence(&f, &f.a, "A-only README verification passed");
    let b_proof = evidence(&f, &f.b, "B-only README verification passed");
    foreign_shared_state(&f);
    let packet = record_continuity_checkpoint_for_event(
        &f.repo,
        &f.vault,
        "A-only checkpoint",
        &OperationContext::from_identity(&f.a),
        "event-a",
    )
    .unwrap();
    let content = fs::read_to_string(&packet.repo_path).unwrap();
    assert!(
        content.contains(&format!("- Current task: `{A_TASK}`")),
        "{content}"
    );
    assert!(content.contains(&format!("- Task ID: `{}`", f.a.task_id())));
    assert!(content.contains(&format!("- Operation ID: `{}`", f.a.operation_id())));
    assert!(content.contains(&format!("- Project ID: `{}`", f.a.project_id())));
    assert!(content.contains(&a_proof));
    assert!(!content.contains(&b_proof));
    assert!(!content.contains("B-only"), "{content}");
    assert_eq!(content, fs::read_to_string(packet.vault_path).unwrap());
}

#[test]
fn identified_task_state_selects_a_plan_and_proof_instead_of_latest_b() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    let a_proof = evidence(&f, &f.a, "A-only README verification passed");
    evidence(&f, &f.b, "B-only README verification passed");
    let state = compile_task_state_for_operation(&f.repo, &f.vault, &f.a, Some(A_TASK)).unwrap();
    assert!(state.current_plan.as_deref().unwrap().contains(A_TASK));
    assert!(state.proof_state.as_deref().unwrap().contains(&a_proof));
    assert!(!state.proof_state.as_deref().unwrap().contains("B-only"));
    assert!(state.trace_state.is_some());
}

#[test]
fn unscoped_context_and_task_state_fail_closed_with_concurrent_identified_work() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    let task_state = compile_task_state(&f.repo, &f.vault, Some(A_TASK)).unwrap_err();
    assert!(
        task_state.to_string().contains("ambiguous"),
        "{task_state:#}"
    );
    let context =
        compile_context_for_task(&f.repo, &f.vault_root, ContextTarget::Codex, Some(A_TASK))
            .unwrap_err();
    assert!(context.to_string().contains("ambiguous"), "{context:#}");
}

#[test]
fn same_task_different_operation_does_not_import_shared_intent_checkpoint_or_recovery() {
    let f = fixture(A_TASK);
    // Same-title anti-hijack policy intentionally forbids starting a second
    // plan under B. A foreign checkpoint still carries B's exact operation
    // tuple even when its plan state is unknown.
    start_or_resume_plan_for_identity(&f.repo, &f.vault, A_TASK, &f.a).unwrap();
    foreign_shared_state(&f);
    let state = compile_task_state_for_operation(&f.repo, &f.vault, &f.a, Some(A_TASK)).unwrap();
    assert!(state.continuity.is_none(), "{state:?}");
    assert!(state.recovery.is_none());
    assert!(state.last_successful_step.is_none());
    assert!(state.affected_files.is_empty());
    assert!(state.intent.is_none());
    assert!(state.target_behavior.is_none());
    assert!(state.constraints.is_empty());
    assert_eq!(state.original_intent.as_deref(), Some(A_TASK));
    assert!(!state.next_action.contains("B-only"));
    assert!(state.unknowns.iter().any(|item| item.contains("recovery")));
}

#[test]
fn prepare_without_a_authority_does_not_resume_b_or_confirm_b_intent() {
    let f = fixture(B_TASK);
    start_or_resume_plan_for_identity(&f.repo, &f.vault, B_TASK, &f.b).unwrap();
    foreign_shared_state(&f);
    let packet = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: A_TASK.into(),
            session_id: Some("session-a".into()),
            request_id: Some("request-a".into()),
        },
        "codex",
        &f.repo,
        None,
    )
    .unwrap();
    assert!(!packet.task.resumed, "{:#?}", packet.continuity);
    assert!(!packet.continuity.available);
    assert!(!packet.intent.available);
    assert!(!packet.intent.confirmed);
    assert!(packet.continuity.recovery_outcome.is_none());
    assert!(packet.continuity.safe_next_action.is_none());
    assert!(!packet.context.text.contains("B-only recovery action"));
}

#[test]
fn lifecycle_context_omits_shared_current_resume_and_latest_indexes() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    foreign_shared_state(&f);
    write(
        &f.repo.join("docs/baron/proofs/INDEX.md"),
        "# Proof Index\n\nB-only proof index\n",
    );
    write(
        &f.repo.join("docs/baron/traces/INDEX.md"),
        "# Trace Index\n\nB-only trace index\n",
    );
    let output =
        compile_context_for_lifecycle_identity(&f.repo, &f.vault_root, &f.a, Some(A_TASK)).unwrap();
    for foreign in [
        "B-only recovery action",
        "B-only intent",
        "B-only story",
        "B-only proof index",
        "B-only trace index",
    ] {
        assert!(!output.contains(foreign), "imported {foreign}: {output}");
    }
    assert!(output.contains(A_TASK));
    assert!(output.chars().count() <= 20_000);
}

#[test]
fn operation_context_entrypoint_keeps_full_binding() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    let output = compile_context_for_operation(
        &f.repo,
        &f.vault_root,
        &OperationContext::from_identity(&f.a),
        Some(A_TASK),
    )
    .unwrap();
    assert!(output.contains(&format!("title={A_TASK}")), "{output}");
    assert!(!output.contains(&format!("title={B_TASK}")));
}

#[test]
fn malformed_exact_plan_authority_refuses_checkpoint_publication() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    foreign_shared_state(&f);
    let path = f.repo.join("docs/baron/continuity/CURRENT.md");
    let before = fs::read(&path).unwrap();
    write(
        &f.repo.join("docs/baron/plans/ACTIVE.md"),
        "<!-- BARON:ACTIVE-PLAN malformed -->\n",
    );
    assert!(record_continuity_checkpoint_for_operation(
        &f.repo,
        &f.vault,
        "A checkpoint",
        &OperationContext::from_identity(&f.a)
    )
    .is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn partial_operation_identity_cannot_compile_as_unscoped_context() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    for operation in [
        OperationContext::new(SupportedAdapter::Codex).with_task_id(f.a.task_id()),
        OperationContext::new(SupportedAdapter::Codex).with_session_id("session-a"),
        OperationContext::new(SupportedAdapter::Codex).with_request_id("request-a"),
    ] {
        assert!(
            compile_context_for_operation(&f.repo, &f.vault_root, &operation, Some(A_TASK))
                .is_err()
        );
    }
}

#[test]
fn partial_checkpoint_identity_cannot_fall_back_to_current() {
    let f = fixture(B_TASK);
    plans(&f, B_TASK);
    foreign_shared_state(&f);
    let path = f.repo.join("docs/baron/continuity/CURRENT.md");
    let before = fs::read(&path).unwrap();
    let partial = OperationContext::new(SupportedAdapter::Codex).with_session_id("session-a");
    assert!(record_continuity_checkpoint_for_operation(
        &f.repo,
        &f.vault,
        "partial identity must not use CURRENT",
        &partial,
    )
    .is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn identity_only_checkpoint_without_plan_is_unknown_not_resumed_work() {
    let f = fixture(B_TASK);
    start_or_resume_plan_for_identity(&f.repo, &f.vault, B_TASK, &f.b).unwrap();
    record_continuity_checkpoint_for_operation(
        &f.repo,
        &f.vault,
        "A observed lifecycle",
        &OperationContext::from_identity(&f.a),
    )
    .unwrap();
    let state = compile_task_state_for_operation(&f.repo, &f.vault, &f.a, Some(A_TASK)).unwrap();
    assert!(!state.resumed, "{state:?}");
    assert!(state.current_plan.is_none());
    assert!(state.continuity.is_none());
    assert!(state.next_action.starts_with("unknown"));
}
