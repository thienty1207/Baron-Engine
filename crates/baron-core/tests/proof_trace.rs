use std::fs;
use std::process::Command;

use baron_core::harness::start_or_resume_intake;
use baron_core::proof::{proof_status, record_proof};
use baron_core::trace::{record_trace, score_trace, TraceOutcome, TraceTier};
use baron_core::vault::ensure_vault;
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

#[test]
fn proof_record_is_written_to_repo_and_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let proof = record_proof(&repo, &context, "cargo test passed: 42 tests").unwrap();

    assert!(proof.repo_path.exists());
    assert!(proof.vault_path.exists());
    assert!(proof_status(&repo).unwrap().contains("42 tests"));
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
}
