use std::fs;

use baron_core::automation::{handle_hook, AutomationEvent, HookAdapter};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::continuity::{
    continuity_status, record_continuity_checkpoint, record_recovery, RecoveryInput,
    RecoveryOutcome,
};
use baron_core::plan::start_or_resume_plan;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

#[test]
fn continuity_checkpoint_writes_repo_and_vault_resume_packet() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    start_or_resume_plan(&repo, &context, "backend login auth").unwrap();

    let packet =
        record_continuity_checkpoint(&repo, &context, "before editing auth handler", "codex")
            .unwrap();

    assert!(packet.repo_path.exists());
    assert!(packet.vault_path.exists());
    let repo_packet = fs::read_to_string(&packet.repo_path).unwrap();
    let vault_packet = fs::read_to_string(&packet.vault_path).unwrap();
    for content in [&repo_packet, &vault_packet] {
        assert!(content.contains("# Baron Continuity Resume"));
        assert!(content.contains("backend login auth"));
        assert!(content.contains("before editing auth handler"));
        assert!(content.contains("Proof status"));
        assert!(content.contains("Trace status"));
        assert!(content.contains("Next action"));
    }
}

#[test]
fn hooks_update_continuity_resume_without_user_commands() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::SessionStart,
        r#"{"session_id":"resume-session","cwd":"demo"}"#,
    )
    .unwrap();

    let status = continuity_status(&repo, &context).unwrap();
    assert!(status.contains("# Baron Continuity Status"));
    assert!(status.contains("SessionStart"));
    assert!(repo.join("docs/baron/continuity/CURRENT.md").exists());
    assert!(context.project_root.join("Continuity/CURRENT.md").exists());
}

fn recovery_input(cause: &str) -> RecoveryInput {
    RecoveryInput {
        outcome: RecoveryOutcome::Interrupted,
        root_cause: cause.to_string(),
        last_successful_step: "Intent confirmed and intake created.".to_string(),
        evidence: vec!["Focused auth test did not finish.".to_string()],
        affected_files: vec!["backend/auth.rs".to_string()],
        next_action: "Resume the auth test, then inspect the failing assertion.".to_string(),
        retry_conditions: vec!["Network and test runner are available.".to_string()],
    }
}

#[test]
fn recovery_packet_preserves_actionable_evidence_and_mirrors_to_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    fs::write(
        repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: mobile login auth\n- Status: `interrupted`\n",
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/harness/CURRENT.md"),
        "# Current Harness\n\n- Title: mobile login auth\n- Risk: `high`\n",
    )
    .unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let packet = record_recovery(
        &repo,
        &context,
        recovery_input("The session ended before verification completed."),
    )
    .unwrap();

    assert!(!packet.resumed);
    assert!(packet.repo_path.exists());
    assert!(packet.vault_path.exists());
    let repo_content = fs::read_to_string(&packet.repo_path).unwrap();
    let vault_content = fs::read_to_string(&packet.vault_path).unwrap();
    for content in [repo_content, vault_content] {
        assert!(content.contains("# Baron Actionable Recovery"));
        assert!(content.contains("Outcome: `interrupted`"));
        assert!(content.contains("session ended before verification"));
        assert!(content.contains("Intent confirmed and intake created"));
        assert!(content.contains("Focused auth test did not finish"));
        assert!(content.contains("backend/auth.rs"));
        assert!(content.contains("Resume the auth test"));
        assert!(content.contains("Network and test runner are available"));
        assert!(content.contains("mobile login auth"));
    }
    assert!(repo
        .join("docs/baron/continuity/CURRENT_RECOVERY.md")
        .exists());
    assert!(context
        .project_root
        .join("Continuity/CURRENT_RECOVERY.md")
        .exists());
}

#[test]
fn repeated_recovery_is_deduplicated_and_new_attempt_preserves_history() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    let first = record_recovery(&repo, &context, recovery_input("Network lost.")).unwrap();
    let second = record_recovery(&repo, &context, recovery_input("Network lost.")).unwrap();
    let third = record_recovery(
        &repo,
        &context,
        recovery_input("Test assertion failed after reconnect."),
    )
    .unwrap();

    assert!(!first.resumed);
    assert!(second.resumed);
    assert_eq!(first.repo_path, second.repo_path);
    assert_ne!(first.repo_path, third.repo_path);
    assert!(first.repo_path.exists());
    assert!(third.repo_path.exists());
    let history_count = fs::read_dir(repo.join("docs/baron/continuity/recovery"))
        .unwrap()
        .flat_map(|entry| fs::read_dir(entry.unwrap().path()).unwrap())
        .count();
    assert_eq!(history_count, 2);
}

#[test]
fn continuity_status_includes_bounded_latest_recovery() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();
    record_recovery(&repo, &context, recovery_input("Network lost.")).unwrap();

    let status = continuity_status(&repo, &context).unwrap();

    assert!(status.contains("## Current Recovery"));
    assert!(status.contains("Network lost"));
    assert!(status.contains("Resume the auth test"));
}

#[test]
fn recovery_packets_preserve_every_supported_outcome() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    let context = ensure_vault(&vault, &repo).unwrap();

    for (outcome, label) in [
        (RecoveryOutcome::Failed, "failed"),
        (RecoveryOutcome::Blocked, "blocked"),
        (RecoveryOutcome::Interrupted, "interrupted"),
    ] {
        let mut input = recovery_input(&format!("{label} attempt"));
        input.outcome = outcome;
        let packet = record_recovery(&repo, &context, input).unwrap();
        assert_eq!(packet.outcome, outcome);
        assert!(fs::read_to_string(packet.repo_path)
            .unwrap()
            .contains(&format!("Outcome: `{label}`")));
    }
}
