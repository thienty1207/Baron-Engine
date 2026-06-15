use std::fs;
use std::path::Path;

use baron_core::automation::{record_lifecycle_event, AutomationEvent, HookAdapter};
use baron_core::harness::record_friction;
use baron_core::harness_improvement::{audit_harness, record_intervention};
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
