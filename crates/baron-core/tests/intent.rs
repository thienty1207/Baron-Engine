use std::fs;

use baron_core::harness::start_or_resume_intake;
use baron_core::intent::{intent_status, record_intent, IntentBriefInput};
use baron_core::risk::RiskLane;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn input(title: &str, confirmed: bool) -> IntentBriefInput {
    IntentBriefInput {
        title: title.to_string(),
        current_behavior: "Users cannot sign in on mobile.".to_string(),
        target_behavior: "Users can sign in through the existing backend API.".to_string(),
        scope: "Mobile login UI and API integration only.".to_string(),
        non_goals: vec!["Do not redesign the web login.".to_string()],
        constraints: vec!["Reuse the existing authentication contract.".to_string()],
        decisions: vec!["Keep the backend as the auth source of truth.".to_string()],
        required_proof: "Mobile login integration test passes.".to_string(),
        unknowns: vec!["Biometric login remains unknown.".to_string()],
        confirmed,
    }
}

#[test]
fn confirmed_intent_is_mirrored_and_deduplicated() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first = record_intent(&repo, &context, input("mobile login auth", true)).unwrap();
    let second = record_intent(&repo, &context, input("mobile login auth", true)).unwrap();

    assert_eq!(first.risk, RiskLane::High);
    assert!(first.confirmed);
    assert!(!first.resumed);
    assert!(second.resumed);
    assert_eq!(first.repo_path, second.repo_path);
    assert_eq!(first.vault_path, second.vault_path);
    let repo_content = fs::read_to_string(&first.repo_path).unwrap();
    let vault_content = fs::read_to_string(&first.vault_path).unwrap();
    for content in [repo_content, vault_content] {
        assert!(content.contains("# Baron Intent Brief"));
        assert!(content.contains("Confirmation: `confirmed`"));
        assert!(content.contains("Users cannot sign in on mobile"));
        assert!(content.contains("Biometric login remains unknown"));
    }
    assert!(repo.join("docs/baron/harness/CURRENT_INTENT.md").exists());
    assert!(context
        .project_root
        .join("ProductHarness/CURRENT_INTENT.md")
        .exists());
    let history_count = fs::read_dir(repo.join("docs/baron/harness/intents"))
        .unwrap()
        .flat_map(|entry| fs::read_dir(entry.unwrap().path()).unwrap())
        .count();
    assert_eq!(history_count, 1);
}

#[test]
fn medium_and_high_risk_intake_requires_matching_confirmed_intent() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let missing = start_or_resume_intake(&repo, &context, "mobile login auth").unwrap_err();
    assert!(missing.to_string().contains("confirmed intent"));

    record_intent(&repo, &context, input("mobile login auth", false)).unwrap();
    let unconfirmed = start_or_resume_intake(&repo, &context, "mobile login auth").unwrap_err();
    assert!(unconfirmed.to_string().contains("not confirmed"));

    record_intent(&repo, &context, input("mobile login auth", true)).unwrap();
    let story = start_or_resume_intake(&repo, &context, "mobile login auth").unwrap();
    assert_eq!(story.risk, RiskLane::High);

    let different = start_or_resume_intake(&repo, &context, "frontend dashboard flow").unwrap_err();
    assert!(different.to_string().contains("does not match"));
}

#[test]
fn low_risk_intake_remains_lightweight_without_formal_intent() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let story = start_or_resume_intake(&repo, &context, "fix README typo").unwrap();

    assert_eq!(story.risk, RiskLane::Low);
    assert!(story.repo_path.exists());
    assert!(!repo.join("docs/baron/harness/CURRENT_INTENT.md").exists());
}

#[test]
fn intent_status_is_clear_when_missing_or_confirmed() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    assert!(intent_status(&repo)
        .unwrap()
        .contains("no intent brief recorded"));

    record_intent(&repo, &context, input("mobile login auth", true)).unwrap();
    let status = intent_status(&repo).unwrap();
    assert!(status.contains("mobile login auth"));
    assert!(status.contains("Confirmation: `confirmed`"));
}
