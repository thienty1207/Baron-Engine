use std::fs;

use baron_core::automation::{handle_hook, AutomationEvent, HookAdapter};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::continuity::{continuity_status, record_continuity_checkpoint};
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
