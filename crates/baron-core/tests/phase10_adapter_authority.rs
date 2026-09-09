//! Phase 10 target evidence.
//!
//! These tests are intentionally written against the explicit-operation API.
//! Before Phase 10 implementation they are expected to fail to compile or
//! fail behaviorally because the project-global `active_adapter` still leaks
//! into correctness paths. They must not be weakened to make the pre-Phase-10
//! implementation green.

use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use baron_core::automation::{
    handle_hook, record_lifecycle_event_for_operation, AutomationEvent, HookAdapter,
};
use baron_core::capability::CapabilityExecutionEvidence;
use baron_core::config::{
    initialize_project, load_project_config, set_active_adapter, AdapterKind,
};
use baron_core::context::compile_context_for_operation;
use baron_core::continuity::{
    record_continuity_checkpoint_for_operation, record_recovery, RecoveryInput, RecoveryOutcome,
};
use baron_core::control_plane::route_task_for_operation;
use baron_core::operation::{OperationContext, SupportedAdapter};
use baron_core::plan::{interrupt_plan, start_or_resume_plan};
use baron_core::prepare::{prepare, PrepareRequestV1};
use baron_core::proof::{record_proof_with_capabilities_for_operation, ProofRecord};
use baron_core::risk::RiskLane;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn project() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("phase10-project");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Claude, &vault).unwrap();
    (temp, repo, vault)
}

fn journal_path(vault: &baron_core::vault::VaultContext) -> std::path::PathBuf {
    vault
        .project_root
        .join("Artifacts/automation-journal.jsonl")
}

#[test]
fn runtime_adapter_type_accepts_only_supported_adapters() {
    assert_eq!(
        SupportedAdapter::parse("codex").unwrap(),
        SupportedAdapter::Codex
    );
    assert_eq!(
        SupportedAdapter::parse("CLAUDE").unwrap(),
        SupportedAdapter::Claude
    );
    assert!(SupportedAdapter::parse("generic").is_err());
}

#[test]
fn explicit_route_and_context_ignore_serialized_active_adapter() {
    let (_temp, repo, vault) = project();
    let operation = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("codex-session")
        .with_request_id("codex-request");

    let route_with_active_claude = route_task_for_operation(
        &repo,
        "review the API contract",
        RiskLane::Medium,
        &operation,
    )
    .unwrap();
    let context_with_active_claude =
        compile_context_for_operation(&repo, &vault, &operation, Some("review the API contract"))
            .unwrap();

    set_active_adapter(&repo, AdapterKind::Codex).unwrap();
    let route_with_active_codex = route_task_for_operation(
        &repo,
        "review the API contract",
        RiskLane::Medium,
        &operation,
    )
    .unwrap();
    let context_with_active_codex =
        compile_context_for_operation(&repo, &vault, &operation, Some("review the API contract"))
            .unwrap();

    assert_eq!(route_with_active_claude, route_with_active_codex);
    assert_eq!(context_with_active_claude, context_with_active_codex);
    assert!(context_with_active_claude.contains("- Adapter target: `codex`"));
}

#[test]
fn route_capability_state_matches_operation_identity_not_project_active_value() {
    let (_temp, repo, _vault) = project();
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::write(
        repo.join(".baron/cache/capability-state.json"),
        r#"{"schema_version":1,"adapter":"claude","checked_at":"2026-09-08T00:00:00Z","observations":[],"required_gaps":[],"optional_gaps":[]}"#,
    )
    .unwrap();
    let codex = OperationContext::new(SupportedAdapter::Codex);
    let claude = OperationContext::new(SupportedAdapter::Claude);
    let codex_route =
        route_task_for_operation(&repo, "inspect shared task state", RiskLane::Low, &codex)
            .unwrap();
    let claude_route =
        route_task_for_operation(&repo, "inspect shared task state", RiskLane::Low, &claude)
            .unwrap();
    assert!(codex_route.explanation.contains("capabilities=unknown"));
    assert!(claude_route.explanation.contains("capabilities=available"));
}

#[test]
fn reciprocal_prepare_uses_claude_when_project_active_adapter_is_codex() {
    let (_temp, repo, _vault) = project();
    set_active_adapter(&repo, AdapterKind::Codex).unwrap();
    let packet = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: "resume the shared API task".to_string(),
            session_id: Some("claude-session".to_string()),
            request_id: Some("claude-request".to_string()),
        },
        "claude",
        &repo,
        None,
    )
    .unwrap();

    assert_eq!(packet.adapter, "claude");
    assert_eq!(packet.session_id.as_deref(), Some("claude-session"));
    assert_eq!(packet.request_id.as_deref(), Some("claude-request"));
    assert_eq!(packet.context.target, "claude");
}

#[test]
fn explicit_operation_does_not_rewrite_legacy_active_adapter_or_schema() {
    let (_temp, repo, _vault) = project();
    set_active_adapter(&repo, AdapterKind::Codex).unwrap();
    let before = load_project_config(&repo).unwrap();
    let operation = OperationContext::new(SupportedAdapter::Claude);
    let _ = route_task_for_operation(&repo, "inspect the shared task", RiskLane::Low, &operation)
        .unwrap();
    let after = load_project_config(&repo).unwrap();
    assert_eq!(after.schema_version, before.schema_version);
    assert_eq!(after.active_adapter, Some(AdapterKind::Codex));
}

#[test]
fn codex_and_claude_prepare_share_semantic_route_and_task_identity() {
    let (_temp, repo, _vault) = project();
    let task = "review the shared API contract";
    let request = |session_id: &str, request_id: &str| PrepareRequestV1 {
        schema_version: 1,
        task: task.to_string(),
        session_id: Some(session_id.to_string()),
        request_id: Some(request_id.to_string()),
    };
    let codex = prepare(
        request("same-session", "same-request"),
        "codex",
        &repo,
        None,
    )
    .unwrap();
    let claude = prepare(
        request("same-session", "same-request"),
        "claude",
        &repo,
        None,
    )
    .unwrap();
    assert_eq!(codex.task.id, claude.task.id);
    assert_eq!(codex.risk, claude.risk);
    assert_eq!(codex.profile, claude.profile);
    assert_eq!(codex.route, claude.route);
    assert_eq!(
        codex.verification.mandatory_gates,
        claude.verification.mandatory_gates
    );
    assert_eq!(codex.adapter, "codex");
    assert_eq!(claude.adapter, "claude");
}

#[test]
fn interrupted_codex_task_resumes_in_claude_and_returns_to_codex_without_global_switching() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    set_active_adapter(&repo, AdapterKind::Codex).unwrap();
    start_or_resume_plan(&repo, &vault, "resume the shared API task").unwrap();
    interrupt_plan(&repo, &vault, "Codex session ended before verification").unwrap();
    record_recovery(
        &repo,
        &vault,
        RecoveryInput {
            outcome: RecoveryOutcome::Interrupted,
            root_cause: "Codex session ended before verification".to_string(),
            last_successful_step: "API contract review completed".to_string(),
            evidence: vec!["Codex checkpoint recorded in shared continuity".to_string()],
            affected_files: vec!["src/api.rs".to_string()],
            next_action: "resume API verification from the shared checkpoint".to_string(),
            retry_conditions: vec!["reconcile the current repo before editing".to_string()],
        },
    )
    .unwrap();
    let codex_operation = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("codex-session-1")
        .with_request_id("codex-request-1");
    record_continuity_checkpoint_for_operation(
        &repo,
        &vault,
        "Codex completed the shared API review",
        &codex_operation,
    )
    .unwrap();

    let claude = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: "resume the shared API task".to_string(),
            session_id: Some("claude-session-2".to_string()),
            request_id: Some("claude-request-2".to_string()),
        },
        "claude",
        &repo,
        None,
    )
    .unwrap();
    assert_eq!(claude.adapter, "claude");
    assert!(claude.task.resumed);
    assert!(claude.continuity.resumed);
    assert_eq!(
        claude.continuity.safe_next_action.as_deref(),
        Some("resume API verification from the shared checkpoint")
    );
    assert!(claude.continuity.summary.contains("Adapter: `codex`"));
    assert!(claude.context.text.contains("- Adapter target: `claude`"));

    let claude_operation = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("claude-session-2")
        .with_request_id("claude-request-2");
    record_continuity_checkpoint_for_operation(
        &repo,
        &vault,
        "Claude continued the shared API verification",
        &claude_operation,
    )
    .unwrap();
    set_active_adapter(&repo, AdapterKind::Claude).unwrap();
    let codex = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: "resume the shared API task".to_string(),
            session_id: Some("codex-session-3".to_string()),
            request_id: Some("codex-request-3".to_string()),
        },
        "codex",
        &repo,
        None,
    )
    .unwrap();
    assert_eq!(codex.adapter, "codex");
    assert!(codex.task.resumed);
    assert!(codex.continuity.resumed);
    assert!(codex.continuity.summary.contains("Adapter: `claude`"));
    assert!(codex.context.text.contains("- Adapter target: `codex`"));
}

#[test]
fn capability_bearing_proof_requires_and_records_explicit_operation_identity() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    let operation = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("session-proof")
        .with_request_id("request-proof");
    let proof = record_proof_with_capabilities_for_operation(
        &repo,
        &vault,
        &operation,
        "Claude proof evidence",
        &[CapabilityExecutionEvidence {
            capability: "source-control".to_string(),
            provider: "git-cli".to_string(),
            summary: "explicit operation evidence".to_string(),
            ..Default::default()
        }],
    )
    .unwrap();

    assert_operation_identity_in_proof(&proof, &operation);
}

#[test]
fn codex_and_claude_proof_attribution_comes_from_the_operation() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    let codex = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("proof-codex-session")
        .with_request_id("proof-codex-request");
    let claude = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("proof-claude-session")
        .with_request_id("proof-claude-request");
    let codex_proof =
        record_proof_with_capabilities_for_operation(&repo, &vault, &codex, "Codex proof", &[])
            .unwrap();
    let codex_content = fs::read_to_string(&codex_proof.repo_path).unwrap();
    let claude_proof =
        record_proof_with_capabilities_for_operation(&repo, &vault, &claude, "Claude proof", &[])
            .unwrap();
    let claude_content = fs::read_to_string(claude_proof.repo_path).unwrap();
    assert!(codex_content.contains("- Adapter: `codex`"));
    assert!(codex_content.contains("- Session ID: `proof-codex-session`"));
    assert!(claude_content.contains("- Adapter: `claude`"));
    assert!(claude_content.contains("- Session ID: `proof-claude-session`"));
}

#[test]
fn capability_bearing_proof_without_identity_fails_closed() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    let error = baron_core::proof::record_proof_with_capabilities(
        &repo,
        &vault,
        "missing operation identity",
        &[CapabilityExecutionEvidence {
            capability: "source-control".to_string(),
            provider: "git-cli".to_string(),
            summary: "unattributed evidence".to_string(),
            ..Default::default()
        }],
    )
    .expect_err("capability evidence must not guess an adapter");
    assert!(error.to_string().contains("explicit adapter identity"));
}

#[test]
fn capability_bearing_proof_does_not_promote_persisted_state_to_operation_identity() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    fs::create_dir_all(repo.join(".baron/cache")).unwrap();
    fs::write(
        repo.join(".baron/cache/capability-state.json"),
        r#"{"schema_version":1,"adapter":"codex","checked_at":"2026-09-08T00:00:00Z","observations":[],"required_gaps":[],"optional_gaps":[]}"#,
    )
    .unwrap();
    let error = baron_core::proof::record_proof_with_capabilities(
        &repo,
        &vault,
        "persisted state is not operation identity",
        &[CapabilityExecutionEvidence {
            capability: "source-control".to_string(),
            provider: "git-cli".to_string(),
            summary: "unattributed evidence".to_string(),
            ..Default::default()
        }],
    )
    .expect_err("persisted capability state must not authorize evidence attribution");
    assert!(error.to_string().contains("explicit adapter identity"));
}

fn assert_operation_identity_in_proof(proof: &ProofRecord, operation: &OperationContext) {
    let content = fs::read_to_string(&proof.repo_path).unwrap();
    assert!(content.contains("- Adapter: `claude`"));
    assert!(content.contains("- Session ID: `session-proof`"));
    assert!(content.contains("- Request ID: `request-proof`"));
    assert_eq!(proof.summary, "Claude proof evidence");
    assert_eq!(operation.adapter, SupportedAdapter::Claude);
}

#[test]
fn continuity_and_journal_are_shared_but_keep_explicit_provenance() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    let codex = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("session-codex")
        .with_request_id("request-codex");
    let claude = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("session-claude")
        .with_request_id("request-claude");

    let first = record_continuity_checkpoint_for_operation(
        &repo,
        &vault,
        "Codex completed the shared step",
        &codex,
    )
    .unwrap();
    let second = record_continuity_checkpoint_for_operation(
        &repo,
        &vault,
        "Claude resumed the shared step",
        &claude,
    )
    .unwrap();
    assert_eq!(first.repo_path, second.repo_path);
    assert_eq!(first.vault_path, second.vault_path);
    let continuity = fs::read_to_string(first.repo_path).unwrap();
    assert!(continuity.contains("- Adapter: `claude`"));
    assert!(continuity.contains("- Session ID: `session-claude`"));
    assert!(continuity.contains("- Request ID: `request-claude`"));

    record_lifecycle_event_for_operation(&vault, &codex, AutomationEvent::Prompt).unwrap();
    record_lifecycle_event_for_operation(&vault, &claude, AutomationEvent::Prompt).unwrap();
    let lines = fs::read_to_string(journal_path(&vault))
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\"adapter\":\"codex\""));
    assert!(lines[0].contains("\"session_id\":\"session-codex\""));
    assert!(lines[0].contains("\"request_id\":\"request-codex\""));
    assert!(lines[1].contains("\"adapter\":\"claude\""));
    assert!(lines[1].contains("\"session_id\":\"session-claude\""));
    assert!(lines[1].contains("\"request_id\":\"request-claude\""));
}

#[test]
fn native_hook_keeps_session_and_request_identity_without_global_adapter_lookup() {
    let (_temp, repo, vault_path) = project();
    let vault = ensure_vault(&vault_path, &repo).unwrap();
    let response = handle_hook(
        &repo,
        &vault,
        HookAdapter::Claude,
        AutomationEvent::Prompt,
        r#"{"session_id":"hook-session","request_id":"hook-request"}"#,
    )
    .unwrap();
    assert!(response.contains("continue"));
    let journal = fs::read_to_string(journal_path(&vault)).unwrap();
    assert!(journal.contains("\"adapter\":\"claude\""));
    assert!(journal.contains("\"session_id\":\"hook-session\""));
    assert!(journal.contains("\"request_id\":\"hook-request\""));
    let continuity = fs::read_to_string(repo.join("docs/baron/continuity/CURRENT.md")).unwrap();
    assert!(continuity.contains("- Adapter: `claude`"));
    assert!(continuity.contains("- Session ID: `hook-session`"));
    assert!(continuity.contains("- Request ID: `hook-request`"));
}

#[test]
fn deterministic_parallel_operations_keep_their_own_identity() {
    let (_temp, repo, _vault) = project();
    let barrier = Arc::new(Barrier::new(2));
    let codex_repo = repo.clone();
    let claude_repo = repo.clone();
    let codex_barrier = Arc::clone(&barrier);
    let claude_barrier = Arc::clone(&barrier);
    let codex = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("parallel-codex")
        .with_request_id("parallel-request-codex");
    let claude = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("parallel-claude")
        .with_request_id("parallel-request-claude");

    let codex_thread = thread::spawn(move || {
        codex_barrier.wait();
        let route = route_task_for_operation(
            &codex_repo,
            "inspect shared task state",
            RiskLane::Low,
            &codex,
        )
        .unwrap();
        (codex.adapter, codex.session_id, codex.request_id, route)
    });
    let claude_thread = thread::spawn(move || {
        claude_barrier.wait();
        let route = route_task_for_operation(
            &claude_repo,
            "inspect shared task state",
            RiskLane::Low,
            &claude,
        )
        .unwrap();
        (claude.adapter, claude.session_id, claude.request_id, route)
    });

    let (codex_adapter, codex_session, codex_request, codex_route) = codex_thread.join().unwrap();
    let (claude_adapter, claude_session, claude_request, claude_route) =
        claude_thread.join().unwrap();
    assert_eq!(codex_adapter, SupportedAdapter::Codex);
    assert_eq!(claude_adapter, SupportedAdapter::Claude);
    assert_eq!(codex_session.as_deref(), Some("parallel-codex"));
    assert_eq!(claude_session.as_deref(), Some("parallel-claude"));
    assert_eq!(codex_request.as_deref(), Some("parallel-request-codex"));
    assert_eq!(claude_request.as_deref(), Some("parallel-request-claude"));
    assert_eq!(codex_route, claude_route);
}
