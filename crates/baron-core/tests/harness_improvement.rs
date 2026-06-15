use std::fs;
use std::path::Path;

use baron_core::automation::{record_lifecycle_event, AutomationEvent, HookAdapter};
use baron_core::harness::record_friction;
use baron_core::harness_improvement::{
    audit_harness, propose_improvements, record_improvement_outcome, record_intervention,
    verify_open_stories,
};
use baron_core::plan::start_or_resume_plan;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn audit_scores_context_reads_and_reports_harness_gaps() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "auth login").unwrap();
    record_friction(&repo, &context, "proof command was unclear").unwrap();

    let audit = audit_harness(&repo, &context).unwrap();

    assert!(audit.context_read_score < 100);
    assert!(audit
        .diagnostics
        .iter()
        .any(|item| item.contains("context was not observed")));
    assert!(audit
        .diagnostics
        .iter()
        .any(|item| item.contains("proof is missing")));
    assert_eq!(audit.open_friction_count, 1);

    record_lifecycle_event(
        &context,
        HookAdapter::Codex,
        AutomationEvent::ContextCompiled,
    )
    .unwrap();
    let improved = audit_harness(&repo, &context).unwrap();
    assert!(improved.context_read_score > audit.context_read_score);
}

#[test]
fn intervention_records_are_mirrored_to_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let record =
        record_intervention(&repo, &context, "reviewer corrected missing security proof").unwrap();

    assert!(record.repo_path.exists());
    assert!(record.vault_path.exists());
    assert!(fs::read_to_string(record.repo_path)
        .unwrap()
        .contains("missing security proof"));
    assert!(fs::read_to_string(record.vault_path)
        .unwrap()
        .contains("missing security proof"));
}

#[test]
fn drift_audit_reports_contradictory_status_files_without_rewriting_them() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    write(
        &repo.join("docs/BARON_STATUS.md"),
        "Phase 11 - completed\nPhase 12 - planned\n",
    );
    write(
        &repo.join("docs/BARON_STATUS.json"),
        r#"{"currentPhaseStatus":"in_progress"}"#,
    );

    let audit = audit_harness(&repo, &context).unwrap();

    assert!(audit
        .diagnostics
        .iter()
        .any(|item| item.contains("documentation drift")));
    assert!(fs::read_to_string(repo.join("docs/BARON_STATUS.md"))
        .unwrap()
        .contains("Phase 11 - completed"));
}

#[test]
fn verify_open_stories_reports_pending_and_insufficient_proof_gaps() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    write(
        &repo.join("docs/baron/harness/TEST_MATRIX.md"),
        "# Baron Validation Matrix\n\n\
| Story | Risk | Status | Evidence |\n\
| --- | --- | --- | --- |\n\
| auth login | high | pending | pending |\n\
| docs copy | low | verified | cargo test passed |\n\
| billing webhook | high | insufficient | missing replay smoke |\n",
    );

    let report = verify_open_stories(&repo, 10).unwrap();

    assert_eq!(report.checked_count, 3);
    assert_eq!(report.proof_gaps.len(), 2);
    assert!(report
        .proof_gaps
        .iter()
        .any(|gap| gap.contains("auth login")));
    assert!(report
        .proof_gaps
        .iter()
        .any(|gap| gap.contains("billing webhook")));
}

#[test]
fn repeated_friction_creates_human_approval_proposal_and_tracks_outcome() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    record_friction(&repo, &context, "proof command was unclear").unwrap();
    record_friction(&repo, &context, "proof evidence was unclear").unwrap();
    record_friction(&repo, &context, "trace proof was unclear").unwrap();

    let proposal = propose_improvements(&repo, &context).unwrap();

    assert!(proposal.proposal_count >= 1);
    let content = fs::read_to_string(&proposal.repo_path).unwrap();
    assert!(content.contains("human approval required"));
    assert!(content.contains("proof"));
    assert!(!fs::read_to_string(repo.join("AGENTS.md"))
        .unwrap_or_default()
        .contains("proof command was unclear"));

    record_improvement_outcome(
        &repo,
        &context,
        &proposal.proposal_ids[0],
        "After adding clearer proof guidance, repeated proof friction dropped.",
    )
    .unwrap();
    let updated = fs::read_to_string(&proposal.repo_path).unwrap();
    assert!(updated.contains("Actual outcome"));
    assert!(updated.contains("friction dropped"));
    let vault_updated = fs::read_to_string(&proposal.vault_path).unwrap();
    assert!(vault_updated.contains("friction dropped"));
}
