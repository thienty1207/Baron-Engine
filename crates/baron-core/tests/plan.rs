use std::fs;
use std::process::Command;

use baron_core::harness::start_or_resume_intake;
use baron_core::plan::{
    complete_plan, interrupt_plan, plan_status, start_or_resume_plan, update_plan,
};
use baron_core::proof::record_proof;
use baron_core::trace::{record_trace, score_trace, TraceOutcome};
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
fn start_creates_dated_plan_current_index_and_vault_mirror() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let plan = start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();

    assert!(plan.repo_path.exists());
    assert!(plan.vault_path.exists());
    assert!(!plan.resumed);
    assert!(fs::read_to_string(repo.join("docs/baron/plans/CURRENT.md"))
        .unwrap()
        .contains("frontend HomePage"));
    assert!(repo.join("docs/baron/plans/INDEX.md").exists());
}

#[test]
fn repeated_start_resumes_matching_plan() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first = start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();
    let second = start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();

    assert_eq!(first.repo_path, second.repo_path);
    assert!(second.resumed);
}

#[test]
fn update_and_interrupt_preserve_last_known_state() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "frontend HomePage").unwrap();

    update_plan(&repo, &context, "Implemented hero; responsive remains").unwrap();
    interrupt_plan(
        &repo,
        &context,
        "Stopped before mobile verification; next run responsive tests",
    )
    .unwrap();

    let status = plan_status(&repo).unwrap();
    assert!(status.contains("Status: `interrupted`"));
    assert!(status.contains("mobile verification"));
    let plan = fs::read_to_string(
        fs::read_dir(repo.join("docs/baron/plans"))
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.path().is_dir())
            .unwrap()
            .path()
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap();
    assert!(plan.contains("Implemented hero"));
    let repo_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    let vault_index = fs::read_to_string(context.project_root.join("Plans/INDEX.md")).unwrap();
    for index in [repo_index, vault_index] {
        assert!(index.contains("frontend HomePage"));
        assert!(index.contains("status: `interrupted`"));
        assert!(!index.contains("frontend HomePage") || !index.contains("status: `in_progress`"));
    }
}

#[test]
fn completion_is_blocked_without_proof_and_passing_trace() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "backend login security").unwrap();

    let error = complete_plan(&repo, &context, "all done").unwrap_err();

    assert!(error.to_string().contains("proof"));
    assert!(plan_status(&repo).unwrap().contains("in_progress"));
}

#[test]
fn high_risk_plan_completes_after_valid_proof_and_detailed_trace() {
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
    start_or_resume_plan(&repo, &context, "backend login security").unwrap();
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
        "Implemented backend login security",
        TraceOutcome::Completed,
    )
    .unwrap();
    assert!(
        score_trace(&repo, &context, Some(&trace.id))
            .unwrap()
            .passed
    );

    complete_plan(
        &repo,
        &context,
        "cargo test auth passed with authorization review",
    )
    .unwrap();

    let status = plan_status(&repo).unwrap();
    assert!(status.contains("Status: `completed`"));
    assert!(status.contains("authorization review"));
    let repo_index = fs::read_to_string(repo.join("docs/baron/plans/INDEX.md")).unwrap();
    let vault_index = fs::read_to_string(context.project_root.join("Plans/INDEX.md")).unwrap();
    for index in [repo_index, vault_index] {
        assert!(index.contains("backend login security"));
        assert!(index.contains("status: `completed`"));
    }
}
