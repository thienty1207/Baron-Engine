use std::fs;

use baron_core::harness::{
    harness_status, record_decision, record_friction, start_or_resume_intake,
};
use baron_core::risk::{classify_risk, RiskLane};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

#[test]
fn risk_classifier_enforces_hard_high_risk_terms() {
    for title in [
        "backend login with Gin",
        "tenant RLS permissions",
        "payment provider migration",
        "security review for file upload",
        "destructive data cleanup",
    ] {
        assert_eq!(classify_risk(title), RiskLane::High, "{title}");
    }
    assert_eq!(classify_risk("frontend dashboard flow"), RiskLane::Medium);
    assert_eq!(classify_risk("fix README typo"), RiskLane::Low);
}

#[test]
fn high_risk_intake_creates_repo_and_vault_story() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let story = start_or_resume_intake(&repo, &context, "backend login with Gin").unwrap();

    assert_eq!(story.risk, RiskLane::High);
    assert!(!story.resumed);
    assert!(story.repo_path.exists());
    assert!(story.vault_path.exists());
    let content = fs::read_to_string(&story.repo_path).unwrap();
    assert!(content.contains("Risk: `high`"));
    assert!(content.contains("security/data-impact proof"));
    let repo_matrix = fs::read_to_string(repo.join("docs/baron/harness/TEST_MATRIX.md")).unwrap();
    let vault_matrix =
        fs::read_to_string(context.project_root.join("ProductHarness/TEST_MATRIX.md")).unwrap();
    for matrix in [repo_matrix, vault_matrix] {
        assert!(matrix.contains("| backend login with Gin | high | pending | pending |"));
    }
}

#[test]
fn duplicate_intake_resumes_without_duplicate_story() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first = start_or_resume_intake(&repo, &context, "frontend dashboard").unwrap();
    let second = start_or_resume_intake(&repo, &context, "frontend dashboard").unwrap();

    assert_eq!(first.repo_path, second.repo_path);
    assert!(second.resumed);
}

#[test]
fn decisions_and_friction_are_mirrored() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    record_decision(&repo, &context, "Use Rust Axum for backend APIs").unwrap();
    record_friction(&repo, &context, "Auth proof command was unclear").unwrap();

    let repo_decisions = fs::read_to_string(repo.join("docs/baron/harness/DECISIONS.md")).unwrap();
    let vault_decisions =
        fs::read_to_string(context.project_root.join("ProductHarness/DECISIONS.md")).unwrap();
    assert!(repo_decisions.contains("Rust Axum"));
    assert!(vault_decisions.contains("Rust Axum"));
    assert!(
        fs::read_to_string(repo.join("docs/baron/harness/FRICTION.md"))
            .unwrap()
            .contains("proof command was unclear")
    );
}

#[test]
fn harness_status_reports_current_story_and_open_friction() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_intake(&repo, &context, "frontend dashboard").unwrap();
    record_friction(&repo, &context, "Missing browser verification").unwrap();

    let status = harness_status(&repo).unwrap();

    assert!(status.contains("frontend dashboard"));
    assert!(status.contains("Risk: `medium`"));
    assert!(status.contains("Open friction: 1"));
}
