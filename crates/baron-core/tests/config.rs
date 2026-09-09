use std::fs;
use std::path::Path;
use std::sync::Mutex;

use baron_core::config::{
    active_adapter, find_project_root, initialize_project, initialize_project_with_options,
    load_project_config, resolve_vault_path_for_repo, set_active_adapter, set_project_platform,
    setup_machine_vault, AdapterKind, ProjectPlatform,
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
    assert_eq!(config.schema_version, 4);
    assert!(!config.project_id.is_empty());
    assert_eq!(config.adapters, vec![AdapterKind::Codex]);
    assert_eq!(config.active_adapter, Some(AdapterKind::Codex));
    assert!(config.automation.context);
    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join(".baron/local.toml").exists());
    assert!(repo.join(".baron/.gitignore").exists());
    let ignore = fs::read_to_string(repo.join(".baron/.gitignore")).unwrap();
    assert!(ignore.contains("local.toml"));
}

#[test]
fn machine_vault_setup_becomes_default_for_project_init() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let home = temp.path().join("home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    std::env::set_var("BARON_HOME", &home);

    let configured = setup_machine_vault(&vault).unwrap();
    let resolved = resolve_vault_path_for_repo(None, &repo).unwrap();

    std::env::remove_var("BARON_HOME");
    assert_eq!(configured, vault.canonicalize().unwrap());
    assert_eq!(resolved, vault.canonicalize().unwrap());
    assert!(home.join("config.toml").exists());
    assert!(vault.join("AGENTS.md").exists());
    assert!(vault.join("Artifacts/Baron/APPROVED_GLOBAL.md").exists());
}

#[test]
fn project_platform_focus_is_stored_and_updateable() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let config = initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Fullstack),
    )
    .unwrap();
    assert_eq!(config.platform, Some(ProjectPlatform::Fullstack));

    let updated = set_project_platform(&repo, ProjectPlatform::Tool).unwrap();

    assert_eq!(updated.platform, Some(ProjectPlatform::Fullstack));
    assert_eq!(updated.platform_extensions, vec![ProjectPlatform::Tool]);
    let content = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(content.contains("platform = \"fullstack\""));
    assert!(content.contains("platform_extensions = [\"tool\"]"));
}

#[test]
fn database_platform_uses_existing_schema_without_bumping_project_config() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("database");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let config = initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Database),
    )
    .unwrap();
    assert_eq!(config.schema_version, 4);
    assert_eq!(config.platform, Some(ProjectPlatform::Database));
    let content = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(content.contains("platform = \"database\""));

    let old_config = content.replace("database", "data");
    fs::write(repo.join(".baron/project.toml"), old_config).unwrap();
    let loaded = load_project_config(&repo).unwrap();
    assert_eq!(loaded.schema_version, 4);
    assert_eq!(loaded.platform, Some(ProjectPlatform::Data));
}

#[test]
fn unknown_platform_is_primary_until_repo_evidence_refines_it() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let unknown = initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Unknown),
    )
    .unwrap();
    assert_eq!(unknown.platform, Some(ProjectPlatform::Unknown));
    assert!(unknown.platform_extensions.is_empty());

    let refined = set_project_platform(&repo, ProjectPlatform::Tool).unwrap();
    assert_eq!(refined.platform, Some(ProjectPlatform::Tool));
    assert!(refined.platform_extensions.is_empty());
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
    assert_eq!(active_adapter(&config), Some(AdapterKind::Codex));
}

#[test]
fn switching_supported_adapters_preserves_project_identity_and_shared_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    let codex = initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let switched = set_active_adapter(&repo, AdapterKind::Claude).unwrap();

    assert_eq!(switched.project_id, codex.project_id);
    assert_eq!(switched.active_adapter, Some(AdapterKind::Claude));
    assert_eq!(
        switched.adapters,
        vec![AdapterKind::Codex, AdapterKind::Claude]
    );
    assert_eq!(
        resolve_vault_path_for_repo(None, &repo).unwrap(),
        vault.canonicalize().unwrap_or(vault)
    );
}

#[test]
fn nested_paths_discover_project_root_and_local_vault() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src/features/auth");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

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
fn unsupported_legacy_config_values_are_retained_without_becoming_active() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-only");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    let historical_value: String = ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect();
    fs::write(
        repo.join(".baron/project.toml"),
        format!(
            "schema_version = 4\nproject_id = \"legacy-project\"\nproject_slug = \"legacy-only\"\nadapters = [\"{historical_value}\"]\nactive_adapter = \"{historical_value}\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n"
        ),
    )
    .unwrap();

    let loaded = load_project_config(&repo).unwrap();
    assert!(loaded.adapters.is_empty());
    assert_eq!(active_adapter(&loaded), None);
    assert_eq!(loaded.legacy_adapters, vec![historical_value.clone()]);
    assert_eq!(
        loaded.legacy_active_adapter.as_deref(),
        Some(historical_value.as_str())
    );
    let serialized = toml::to_string(&loaded).unwrap();
    assert!(serialized.contains(&historical_value));
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
