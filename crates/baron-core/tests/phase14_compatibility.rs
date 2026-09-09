//! Phase 14 persisted-state compatibility fixtures.
//!
//! The tests in this file are cross-platform unless explicitly marked
//! otherwise. Target architecture tests use `ignore` so the normal regression
//! suite remains green while the red assertion is still executable evidence.

use std::fs;
use std::path::Path;

use baron_core::automation::{automation_status, handle_hook, AutomationEvent, HookAdapter};
use baron_core::autopilot::autopilot_status;
use baron_core::config::{
    initialize_project, load_project_config, set_active_adapter, set_project_platform, AdapterKind,
    ProjectPlatform,
};
use baron_core::vault::{ensure_vault, vault_context_without_create};
use tempfile::tempdir;

fn initialized() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    ensure_vault(&vault, &repo).unwrap();
    (temp, repo, vault)
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

/// Target: every project writer must refuse a future schema before touching
/// bytes. Today the state guard has this property, but direct project writers
/// still normalize and rewrite the future document.
#[test]
fn future_project_schema_refuses_every_direct_writer_without_rewriting() {
    let (_temp, repo, vault) = initialized();
    let config_path = repo.join(".baron/project.toml");
    let future = fs::read_to_string(&config_path)
        .unwrap()
        .replace("schema_version = 4", "schema_version = 999");
    fs::write(&config_path, &future).unwrap();
    let before = fs::read(&config_path).unwrap();

    let switch = set_active_adapter(&repo, AdapterKind::Claude);
    assert!(
        switch.is_err(),
        "future config must block adapter switching"
    );
    assert_eq!(fs::read(&config_path).unwrap(), before);

    let platform = set_project_platform(&repo, ProjectPlatform::Backend);
    assert!(
        platform.is_err(),
        "future config must block platform writes"
    );
    assert_eq!(fs::read(&config_path).unwrap(), before);

    let initialize = initialize_project(&repo, AdapterKind::Codex, &vault);
    assert!(
        initialize.is_err(),
        "future config must block reinitialization"
    );
    assert_eq!(fs::read(&config_path).unwrap(), before);
}

#[test]
fn older_known_project_schema_remains_readable_and_serializes_deterministically() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("old-project");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    write(
        &repo.join(".baron/project.toml"),
        "schema_version = 3\nproject_id = \"old-project-id\"\nproject_slug = \"old-project\"\nadapters = [\"codex\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
    );

    let config = load_project_config(&repo).unwrap();
    assert_eq!(config.schema_version, 3);
    assert_eq!(config.adapters, vec![AdapterKind::Codex]);
    let first = toml::to_string_pretty(&config).unwrap();
    let second = toml::to_string_pretty(&config).unwrap();
    assert_eq!(first, second);
}

#[test]
fn future_autopilot_state_is_read_only_and_actionable() {
    let (_temp, repo, vault_root) = initialized();
    let vault = vault_context_without_create(&vault_root, &repo).unwrap();
    let path = repo.join("docs/baron/autopilot/STATE.json");
    let future = "{\n  \"schema_version\": 999,\n  \"project_id\": \"repo\",\n  \"candidates\": [],\n  \"responses\": [],\n  \"archived\": []\n}\n";
    write(&path, future);
    let before = fs::read(&path).unwrap();

    let error = autopilot_status(&repo, &vault).unwrap_err().to_string();

    assert!(error.contains("newer than supported"));
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn future_dedup_state_blocks_hook_before_any_lifecycle_write() {
    let (_temp, repo, vault_root) = initialized();
    let context = vault_context_without_create(&vault_root, &repo).unwrap();
    let dedup_path = repo.join(".baron/cache/automation-dedup.json");
    let future = "{\n  \"schema_version\": 999,\n  \"entries\": []\n}\n";
    write(&dedup_path, future);
    let before = fs::read(&dedup_path).unwrap();
    let journal = context
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    let journal_before = fs::read(&journal).unwrap_or_default();

    let error = handle_hook(
        &repo,
        &context,
        HookAdapter::Codex,
        AutomationEvent::UserPromptSubmit,
        r#"{"task":"future dedup fixture","session_id":"phase14","request_id":"one"}"#,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("Unsupported hook dedup schema"));
    assert_eq!(fs::read(&dedup_path).unwrap(), before);
    assert_eq!(fs::read(&journal).unwrap_or_default(), journal_before);
}

#[test]
fn additive_journal_fields_are_readable_without_rewriting_history() {
    let (_temp, repo, vault_root) = initialized();
    let context = vault_context_without_create(&vault_root, &repo).unwrap();
    let journal = context
        .project_root
        .join("Artifacts/automation-journal.jsonl");
    let historical = r#"{"timestamp":"2026-09-08T00:00:00+00:00","event":"checkpoint","adapter":"codex","session_id":null,"future_field":{"preserve":true}}
"#;
    fs::write(&journal, historical).unwrap();

    let before = fs::read(&journal).unwrap();
    let status = automation_status(&repo, &context).unwrap();

    assert!(status.contains("Events recorded: 1"));
    assert!(status.contains("Latest event: `checkpoint`"));
    assert_eq!(fs::read(&journal).unwrap(), before);
}
