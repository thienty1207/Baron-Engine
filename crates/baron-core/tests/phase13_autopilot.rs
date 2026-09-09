//! Phase 13 Autopilot fixtures.
//!
//! Cross-platform fixtures exercise the candidate trust boundary, deterministic
//! lifecycle, conversational approval, project scope, and bounded projection.
//! Native permission/junction mechanics remain in the Phase 2 suites.

use std::fs;

use baron_core::autopilot::{
    autopilot_status, candidate_records, housekeep_candidates, render_autopilot_context_summary,
    respond_to_pending_approval, respond_to_pending_approval_for_operation, review_after_task,
    review_after_task_for_operation,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::firewall::trusted_recall_current;
use baron_core::intent::{record_intent, IntentBriefInput};
use baron_core::memory::build_memory_index;
use baron_core::operation::{OperationContext, SupportedAdapter};
use baron_core::prepare::{prepare, PrepareRequestV1};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn project(
    adapter: AdapterKind,
) -> (
    tempfile::TempDir,
    std::path::PathBuf,
    baron_core::vault::VaultContext,
) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, adapter, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    (temp, repo, context)
}

#[test]
fn pending_learning_is_deterministic_untrusted_and_provenanced() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let first = review_after_task(&repo, &vault, "Prefer deterministic API tests").unwrap();
    let second = review_after_task(&repo, &vault, "prefer  deterministic api tests").unwrap();

    assert_eq!(first.candidate_ids, second.candidate_ids);
    assert_eq!(first.candidate_count, 1);
    let candidates = candidate_records(&repo, &vault).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].evidence_count, 2);
    assert!(candidates[0].provenance.len() >= 2);
    assert!(matches!(
        candidates[0].status.as_str(),
        "candidate" | "ready"
    ));
    assert!(!candidates[0].trusted);

    build_memory_index(&vault).unwrap();
    let current = trusted_recall_current(&vault, "deterministic API tests", 20).unwrap();
    assert!(current
        .results
        .iter()
        .all(|hit| !hit.record.path.contains("Autopilot/CANDIDATES")));
    assert!(autopilot_status(&repo, &vault)
        .unwrap()
        .contains("Trusted fact policy"));
}

#[test]
fn cross_adapter_observations_share_scope_without_raising_trust() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let codex = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("codex-session")
        .with_request_id("codex-request");
    let claude = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("claude-session")
        .with_request_id("claude-request");
    let first =
        review_after_task_for_operation(&repo, &vault, "Keep API errors structured", &codex)
            .unwrap();
    let second =
        review_after_task_for_operation(&repo, &vault, "Keep API errors structured", &claude)
            .unwrap();
    assert_eq!(first.candidate_ids, second.candidate_ids);
    let candidates = candidate_records(&repo, &vault).unwrap();
    assert_eq!(candidates[0].evidence_count, 2);
    assert!(!candidates[0].trusted);
    let status = autopilot_status(&repo, &vault).unwrap();
    assert!(status.contains("codex") && status.contains("claude"));
}

#[test]
fn conversational_approval_promotes_only_through_project_decision_authority_and_is_idempotent() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let review = review_after_task(&repo, &vault, "Use Rust for the project API").unwrap();
    let first =
        respond_to_pending_approval(&repo, &vault, "yes, make that the project rule").unwrap();
    assert_eq!(first.candidate_id, review.candidate_ids[0]);
    assert_eq!(first.status, "approved");
    assert!(first.changed);
    assert!(first.promotion.contains("project decision"));

    let second =
        respond_to_pending_approval(&repo, &vault, "yes, make that the project rule").unwrap();
    assert_eq!(second.candidate_id, first.candidate_id);
    assert!(!second.changed);
    let decisions = fs::read_to_string(repo.join("docs/baron/harness/DECISIONS.md")).unwrap();
    assert_eq!(decisions.matches("Use Rust for the project API").count(), 1);
    assert!(
        !repo.join("assets/core").exists()
            || !repo.join("assets/core").join("AUTOPILOT.md").exists()
    );
}

#[test]
fn rejection_and_defer_suppress_reprompts_but_correction_creates_a_scoped_replacement() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let deferred = review_after_task(&repo, &vault, "Adopt a shared API error envelope").unwrap();
    let defer = respond_to_pending_approval(&repo, &vault, "not now").unwrap();
    assert_eq!(defer.status, "deferred");
    let repeated = review_after_task(&repo, &vault, "Adopt a shared API error envelope").unwrap();
    assert_eq!(repeated.candidate_ids, deferred.candidate_ids);
    assert!(!repeated.approval_required);

    let correction = respond_to_pending_approval(
        &repo,
        &vault,
        "change it to only apply to backend API modules",
    )
    .unwrap();
    assert_eq!(correction.status, "corrected");
    assert_ne!(correction.candidate_id, deferred.candidate_ids[0]);
    let correction_retry = respond_to_pending_approval(
        &repo,
        &vault,
        "change it to only apply to backend API modules",
    )
    .unwrap();
    assert_eq!(correction_retry.candidate_id, correction.candidate_id);
    assert!(!correction_retry.changed);
    let candidates = candidate_records(&repo, &vault).unwrap();
    assert!(candidates
        .iter()
        .any(|candidate| candidate.status == "superseded"));
    assert!(candidates
        .iter()
        .any(|candidate| candidate.status == "candidate"));

    let (_reject_temp, reject_repo, reject_vault) = project(AdapterKind::Codex);
    let rejected = review_after_task(
        &reject_repo,
        &reject_vault,
        "Never add an API error envelope",
    )
    .unwrap();
    let reject =
        respond_to_pending_approval(&reject_repo, &reject_vault, "no, reject this").unwrap();
    assert_eq!(reject.status, "rejected");
    let no_repeat = review_after_task(
        &reject_repo,
        &reject_vault,
        "Never add an API error envelope",
    )
    .unwrap();
    assert_eq!(no_repeat.candidate_ids, rejected.candidate_ids);
    assert!(!no_repeat.approval_required);
}

#[test]
fn conflicts_are_surfaceable_and_cannot_be_approved_implicitly() {
    let (_temp, repo, vault) = project(AdapterKind::Claude);
    review_after_task(&repo, &vault, "Always use API version v1").unwrap();
    review_after_task(&repo, &vault, "Never use API version v1").unwrap();
    let candidates = candidate_records(&repo, &vault).unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.status == "conflict"));
    let result = respond_to_pending_approval(&repo, &vault, "yes");
    assert!(
        result.is_err(),
        "a conflicted proposal must require clarification"
    );
}

#[test]
fn pending_approval_is_bounded_in_context_and_prepare_warnings_without_packet_change() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    review_after_task(&repo, &vault, "Keep task recovery evidence explicit").unwrap();
    let context = render_autopilot_context_summary(&repo, &vault);
    assert!(context.len() <= 2_400);
    assert!(context.contains("approval") && context.contains("untrusted"));

    let packet = prepare(
        PrepareRequestV1 {
            schema_version: 1,
            task: "inspect task recovery evidence".to_string(),
            session_id: Some("phase13-session".to_string()),
            request_id: Some("phase13-request".to_string()),
        },
        "codex",
        &repo,
        Some(vault.vault_root.clone()),
    )
    .unwrap();
    assert_eq!(packet.schema_version, 1);
    assert!(packet
        .warnings
        .iter()
        .any(|warning| warning.code == "autopilot_approval_pending"));
}

#[test]
fn approval_correlation_can_cross_from_codex_to_claude_without_child_promotion() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let codex = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("shared-session")
        .with_request_id("shared-request");
    let claude = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("shared-session")
        .with_request_id("shared-request");
    let review = review_after_task_for_operation(
        &repo,
        &vault,
        "Preserve interrupted task recovery",
        &codex,
    )
    .unwrap();
    let outcome = respond_to_pending_approval_for_operation(&repo, &vault, "yes", &claude).unwrap();
    assert_eq!(outcome.candidate_id, review.candidate_ids[0]);
    assert_eq!(outcome.status, "approved");
    assert!(!outcome.child_promoted);
}

#[test]
fn mismatched_operation_correlation_cannot_approve_another_pending_task() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    let codex = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("session-a")
        .with_request_id("request-a");
    let wrong = OperationContext::new(SupportedAdapter::Claude)
        .with_session_id("session-b")
        .with_request_id("request-b");
    review_after_task_for_operation(&repo, &vault, "Preserve task A recovery", &codex).unwrap();
    assert!(respond_to_pending_approval_for_operation(&repo, &vault, "yes", &wrong).is_err());
    assert_eq!(
        candidate_records(&repo, &vault).unwrap()[0].status,
        "candidate"
    );
}

#[test]
fn multiple_unrelated_pending_candidates_require_clarification() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    review_after_task(&repo, &vault, "Prefer API version v1").unwrap();
    review_after_task(&repo, &vault, "Keep database migrations reversible").unwrap();

    let result = respond_to_pending_approval(&repo, &vault, "yes");
    assert!(result.is_err(), "ambiguous approval must fail closed");
    let candidates = candidate_records(&repo, &vault).unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates
        .iter()
        .all(|candidate| matches!(candidate.status.as_str(), "candidate" | "ready")));
}

#[test]
fn archived_resolution_keeps_response_idempotent() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    review_after_task(&repo, &vault, "Use a stable API error envelope").unwrap();
    let first = respond_to_pending_approval(&repo, &vault, "yes").unwrap();
    assert_eq!(first.status, "approved");

    let housekeeping = housekeep_candidates(&repo, &vault).unwrap();
    assert_eq!(housekeeping.pending, 0);
    let state: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("docs/baron/autopilot/STATE.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(state["candidates"].as_array().unwrap().len(), 0);
    assert_eq!(state["archived"].as_array().unwrap().len(), 1);

    let retry = respond_to_pending_approval(&repo, &vault, "yes").unwrap();
    assert_eq!(retry.candidate_id, first.candidate_id);
    assert!(!retry.changed);
}

#[test]
fn housekeeping_expires_weak_stale_candidates_without_promoting_them() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    review_after_task(&repo, &vault, "Remember the old API migration note").unwrap();
    let state_path = repo.join("docs/baron/autopilot/STATE.json");
    let mut state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    state["candidates"][0]["updated_at"] =
        serde_json::Value::String("2020-01-01T00:00:00+00:00".to_string());
    state["candidates"][0]["evidence_count"] = serde_json::Value::Number(1.into());
    state["candidates"][0]["status"] = serde_json::Value::String("candidate".to_string());
    let encoded = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&state_path, &encoded).unwrap();
    fs::write(vault.project_root.join("Autopilot/STATE.json"), &encoded).unwrap();

    let report = housekeep_candidates(&repo, &vault).unwrap();
    assert_eq!(report.expired, 1);
    let candidates = candidate_records(&repo, &vault).unwrap();
    assert!(candidates.is_empty());
    let state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
    assert_eq!(state["archived"][0]["status"], "expired");
    assert_eq!(state["archived"][0]["trusted"], false);
}

#[test]
fn project_scope_isolation_blocks_cross_project_approval() {
    let temp = tempdir().unwrap();
    let vault_root = temp.path().join("vault");
    let repo_a = temp.path().join("repo-a");
    let repo_b = temp.path().join("repo-b");
    fs::create_dir_all(&repo_a).unwrap();
    fs::create_dir_all(&repo_b).unwrap();
    initialize_project(&repo_a, AdapterKind::Codex, &vault_root).unwrap();
    initialize_project(&repo_b, AdapterKind::Claude, &vault_root).unwrap();
    let vault_a = ensure_vault(&vault_root, &repo_a).unwrap();
    let vault_b = ensure_vault(&vault_root, &repo_b).unwrap();
    review_after_task(&repo_a, &vault_a, "Keep project A API errors structured").unwrap();
    assert!(candidate_records(&repo_b, &vault_b).unwrap().is_empty());
    assert!(respond_to_pending_approval(&repo_b, &vault_b, "yes").is_err());
}

#[test]
fn recovery_provenance_survives_restart_and_explicit_intent_wins() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    fs::create_dir_all(repo.join("docs/baron/continuity")).unwrap();
    fs::write(
        repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        "# Recovery\n\n- Outcome: `interrupted`\n\n## Safe Next Action\n\nresume from evidence\n",
    )
    .unwrap();
    let review = review_after_task(&repo, &vault, "Always use API version v1").unwrap();
    let resumed = ensure_vault(&vault.vault_root, &repo).unwrap();
    let candidates = candidate_records(&repo, &resumed).unwrap();
    assert_eq!(candidates[0].id, review.candidate_ids[0]);
    assert!(candidates[0]
        .provenance
        .iter()
        .any(|source| source.contains("CURRENT_RECOVERY")));

    record_intent(
        &repo,
        &resumed,
        IntentBriefInput {
            title: "API compatibility decision".to_string(),
            current_behavior: "The project has competing API version proposals".to_string(),
            target_behavior: "Never use API version v1".to_string(),
            scope: "Project API modules".to_string(),
            non_goals: vec!["No unrelated refactor".to_string()],
            constraints: vec!["Preserve existing clients".to_string()],
            decisions: vec![],
            required_proof: "Run API compatibility tests".to_string(),
            unknowns: vec![],
            confirmed: true,
        },
    )
    .unwrap();
    let superseded = review_after_task(&repo, &resumed, "Always use API version v1").unwrap();
    assert_eq!(superseded.status, "superseded");
    assert!(respond_to_pending_approval(&repo, &resumed, "yes").is_err());
}

#[test]
fn approved_candidate_remains_outside_autopilot_current_memory_path() {
    let (_temp, repo, vault) = project(AdapterKind::Codex);
    review_after_task(&repo, &vault, "Remember the API error envelope").unwrap();
    let outcome = respond_to_pending_approval(&repo, &vault, "yes").unwrap();
    assert_eq!(outcome.status, "approved");
    build_memory_index(&vault).unwrap();
    let current = trusted_recall_current(&vault, "API error envelope", 20).unwrap();
    assert!(current
        .results
        .iter()
        .all(|hit| !hit.record.path.contains("Autopilot/CANDIDATES")));
}
