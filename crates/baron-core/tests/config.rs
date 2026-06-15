use std::fs;
use std::path::Path;
use std::sync::Mutex;

use baron_core::config::{
    find_project_root, initialize_project, load_project_config, resolve_vault_path_for_repo,
    AdapterKind,
};
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn initialize_creates_shared_and_local_config() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("TomoTy");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let config = initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

    assert_eq!(config.project_slug, "tomoty");
    assert_eq!(config.schema_version, 2);
    assert!(!config.project_id.is_empty());
    assert_eq!(config.adapters, vec![AdapterKind::Codex]);
    assert!(config.automation.context);
    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join(".baron/local.toml").exists());
    assert!(repo.join(".baron/.gitignore").exists());
    let ignore = fs::read_to_string(repo.join(".baron/.gitignore")).unwrap();
    assert!(ignore.contains("local.toml"));
}

#[test]
fn repositories_with_the_same_name_receive_different_project_ids() {
    let temp = tempdir().unwrap();
    let first = temp.path().join("one").join("same-app");
    let second = temp.path().join("two").join("same-app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();

    let first_config = initialize_project(&first, AdapterKind::Codex, &vault).unwrap();
    let second_config = initialize_project(&second, AdapterKind::Codex, &vault).unwrap();

    assert_eq!(first_config.project_slug, second_config.project_slug);
    assert_ne!(first_config.project_id, second_config.project_id);
}

#[test]
fn moving_a_configured_repository_preserves_project_identity() {
    let temp = tempdir().unwrap();
    let original = temp.path().join("original").join("demo");
    let moved = temp.path().join("moved").join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&original).unwrap();

    let before = initialize_project(&original, AdapterKind::Codex, &vault).unwrap();
    fs::create_dir_all(moved.parent().unwrap()).unwrap();
    fs::rename(&original, &moved).unwrap();
    let after = load_project_config(&moved).unwrap();

    assert_eq!(before.project_id, after.project_id);
    assert_eq!(before.project_slug, after.project_slug);
}

#[test]
fn repeated_initialize_registers_multiple_adapters_without_duplicates() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    initialize_project(&repo, AdapterKind::Claude, &vault).unwrap();
    let config = initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

    assert_eq!(
        config.adapters,
        vec![AdapterKind::Codex, AdapterKind::Claude]
    );
}

#[test]
fn nested_paths_discover_project_root_and_local_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src/features/auth");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();
    initialize_project(&repo, AdapterKind::Generic, &vault).unwrap();

    assert_eq!(
        find_project_root(&nested).unwrap(),
        repo.canonicalize().unwrap()
    );
    assert_eq!(
        resolve_vault_path_for_repo(None, &nested).unwrap(),
        vault.canonicalize().unwrap_or(vault)
    );
}

#[test]
fn explicit_vault_wins_over_environment_and_local_config() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let local_vault = temp.path().join("LocalVault");
    let env_vault = temp.path().join("EnvVault");
    let explicit_vault = temp.path().join("ExplicitVault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &local_vault).unwrap();

    std::env::set_var("BARON_VAULT", &env_vault);
    let resolved = resolve_vault_path_for_repo(Some(explicit_vault.clone()), &repo).unwrap();
    std::env::remove_var("BARON_VAULT");

    assert_eq!(resolved, explicit_vault);
}

#[test]
fn environment_vault_wins_over_local_config() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let local_vault = temp.path().join("LocalVault");
    let env_vault = temp.path().join("EnvVault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &local_vault).unwrap();

    std::env::set_var("BARON_VAULT", &env_vault);
    let resolved = resolve_vault_path_for_repo(None, &repo).unwrap();
    std::env::remove_var("BARON_VAULT");

    assert_eq!(resolved, env_vault);
}

#[test]
fn malformed_project_config_fails_without_rewriting_user_file() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let config_path = repo.join(".baron/project.toml");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, "this = [is not valid").unwrap();
    let before = fs::read_to_string(&config_path).unwrap();

    let error = load_project_config(&repo).unwrap_err();

    assert!(error.to_string().contains("Could not parse"));
    assert_eq!(fs::read_to_string(&config_path).unwrap(), before);
}

#[test]
fn config_files_do_not_store_memory_content() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

    for file in [".baron/project.toml", ".baron/local.toml"] {
        let content = fs::read_to_string(repo.join(file)).unwrap();
        assert!(!content.contains("Facts.md"));
        assert!(!content.contains("memory record"));
    }
}

#[allow(dead_code)]
fn _assert_path(_: &Path) {}
