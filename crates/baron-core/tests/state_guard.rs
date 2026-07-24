use std::fs;

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::state_guard::require_coherent_execution_state;
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn initialized() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    ensure_vault(&vault, &repo).unwrap();
    (temp, repo, vault)
}

#[test]
fn coherent_initialized_state_is_accepted_without_writes() {
    let (_temp, repo, vault) = initialized();
    let metadata = baron_core::vault::vault_context_without_create(&vault, &repo)
        .unwrap()
        .project_root
        .join(".baron-project.json");
    let before = fs::read(&metadata).unwrap();

    let context = require_coherent_execution_state(&repo, &vault).unwrap();

    assert_eq!(context.repo_root, repo.canonicalize().unwrap());
    assert_eq!(before, fs::read(metadata).unwrap());
}

#[test]
fn identity_mismatch_is_rejected_without_repairing_or_overwriting_metadata() {
    let (_temp, repo, vault) = initialized();
    let context = baron_core::vault::vault_context_without_create(&vault, &repo).unwrap();
    let metadata = context.project_root.join(".baron-project.json");
    let tampered = "{\n  \"schemaVersion\": 2,\n  \"projectId\": \"wrong-project\",\n  \"projectSlug\": \"demo\"\n}\n";
    fs::write(&metadata, tampered).unwrap();

    let error = require_coherent_execution_state(&repo, &vault)
        .unwrap_err()
        .to_string();

    assert!(error.contains("identity mismatch"));
    assert!(error.contains("baron automation reconcile"));
    assert_eq!(fs::read_to_string(metadata).unwrap(), tampered);
}

#[test]
fn missing_capsule_is_rejected_and_not_silently_recreated() {
    let (_temp, repo, vault) = initialized();
    let context = baron_core::vault::vault_context_without_create(&vault, &repo).unwrap();
    fs::remove_dir_all(&context.project_root).unwrap();

    let error = require_coherent_execution_state(&repo, &vault)
        .unwrap_err()
        .to_string();

    assert!(error.contains("capsule is missing"));
    assert!(error.contains("baron automation reconcile"));
    assert!(!context.project_root.exists());
}

#[test]
fn unsupported_project_schema_is_rejected_without_rewriting_config() {
    let (_temp, repo, vault) = initialized();
    let config_path = repo.join(".baron/project.toml");
    let tampered = fs::read_to_string(&config_path)
        .unwrap()
        .replace("schema_version = 4", "schema_version = 999");
    fs::write(&config_path, &tampered).unwrap();

    let error = require_coherent_execution_state(&repo, &vault)
        .unwrap_err()
        .to_string();

    assert!(error.contains("schema"));
    assert!(error.contains("baron automation reconcile"));
    assert_eq!(fs::read_to_string(config_path).unwrap(), tampered);
}
