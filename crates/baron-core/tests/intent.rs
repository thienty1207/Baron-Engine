use std::fs;

use baron_core::harness::start_or_resume_intake;
use baron_core::intent::{
    intent_status, operation_intent_for_identity, record_intent, record_intent_for_operation,
    IntentBriefInput, MAX_OPERATION_INTENT_CHARS,
};
use baron_core::operation::{LifecycleIdentity, SupportedAdapter};
use baron_core::risk::RiskLane;
use baron_core::task_state::compile_task_state_for_operation;
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

#[test]
fn operation_intents_are_durable_and_isolated_from_current_projection() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let task = "implement authorization policy after deciding access rules";
    let first = LifecycleIdentity::resolve(
        &context.project_id,
        task,
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let second = LifecycleIdentity::resolve(
        &context.project_id,
        task,
        SupportedAdapter::Claude,
        Some("session-b"),
        Some("request-b"),
    )
    .unwrap();

    record_intent_for_operation(
        &repo,
        &context,
        task,
        &first,
        input("access policy A", true),
    )
    .unwrap();
    record_intent_for_operation(
        &repo,
        &context,
        task,
        &second,
        input("access policy B", false),
    )
    .unwrap();

    let first_content = operation_intent_for_identity(&repo, &first, 8_000)
        .unwrap()
        .unwrap();
    let second_content = operation_intent_for_identity(&repo, &second, 8_000)
        .unwrap()
        .unwrap();
    assert!(first_content.contains(&format!("- Operation ID: `{}`", first.operation_id())));
    assert!(first_content.contains("- Confirmation: `confirmed`"));
    assert!(first_content.contains("- Title: access policy A"));
    assert!(second_content.contains(&format!("- Operation ID: `{}`", second.operation_id())));
    assert!(second_content.contains("- Confirmation: `needs_confirmation`"));
    assert!(second_content.contains("- Title: access policy B"));

    let first_state =
        compile_task_state_for_operation(&repo, &context, &first, Some(task)).unwrap();
    let second_state =
        compile_task_state_for_operation(&repo, &context, &second, Some(task)).unwrap();
    assert_eq!(first_state.intent.as_deref(), Some("access policy A"));
    assert_eq!(second_state.intent.as_deref(), Some("access policy B"));

    let current = fs::read_to_string(repo.join("docs/baron/harness/CURRENT_INTENT.md")).unwrap();
    assert!(current.contains(&format!("- Operation ID: `{}`", second.operation_id())));
    assert!(!current.contains(&format!("- Operation ID: `{}`", first.operation_id())));
}

#[test]
fn another_operations_confirmed_current_intent_cannot_authorize_unscoped_intake() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let task = "implement authorization policy after deciding access rules";
    let first = LifecycleIdentity::resolve(
        &context.project_id,
        task,
        SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let second = LifecycleIdentity::resolve(
        &context.project_id,
        task,
        SupportedAdapter::Claude,
        Some("session-b"),
        Some("request-b"),
    )
    .unwrap();

    record_intent_for_operation(&repo, &context, task, &first, input(task, false)).unwrap();
    record_intent_for_operation(&repo, &context, task, &second, input(task, true)).unwrap();
    let current_path = repo.join("docs/baron/harness/CURRENT_INTENT.md");
    let current_before = fs::read_to_string(&current_path).unwrap();
    assert!(current_before.contains(&format!("- Operation ID: `{}`", second.operation_id())));
    assert!(current_before.contains("- Confirmation: `confirmed`"));

    let error = start_or_resume_intake(&repo, &context, task)
        .expect_err("identity-less intake must not borrow another operation's confirmation");

    assert!(
        error.to_string().contains("operation-scoped"),
        "unexpected intake error: {error}"
    );
    assert_eq!(fs::read_to_string(current_path).unwrap(), current_before);
    let harness_current = repo.join("docs/baron/harness/CURRENT.md");
    assert!(!fs::read_to_string(harness_current)
        .unwrap_or_default()
        .contains(task));
}

#[test]
fn corrupt_operation_intent_fails_closed_instead_of_falling_back_to_current() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let task = "implement the authorization policy";
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        task,
        baron_core::operation::SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    record_intent_for_operation(
        &repo,
        &context,
        task,
        &identity,
        input("authorization policy", true),
    )
    .unwrap();

    let operation_path = repo
        .join("docs/baron/harness/intent-operations")
        .join(format!("{}.md", identity.operation_id()));
    let content = fs::read_to_string(&operation_path).unwrap();
    fs::write(
        &operation_path,
        content.replace("- Session ID: `session-a`", "- Session ID: `session-b`"),
    )
    .unwrap();

    assert!(operation_intent_for_identity(&repo, &identity, 8_000).is_err());
}

#[test]
fn oversized_operation_intent_fails_before_publishing_any_projection() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    let task = "implement the authorization policy";
    let identity = LifecycleIdentity::resolve(
        &context.project_id,
        task,
        baron_core::operation::SupportedAdapter::Codex,
        Some("session-a"),
        Some("request-a"),
    )
    .unwrap();
    let mut oversized = input("authorization policy", true);
    oversized.current_behavior = "x".repeat(MAX_OPERATION_INTENT_CHARS);

    assert!(record_intent_for_operation(&repo, &context, task, &identity, oversized).is_err());
    assert!(!repo.join("docs/baron/harness/CURRENT_INTENT.md").exists());
    assert!(!repo.join("docs/baron/harness/intent-operations").exists());
}
