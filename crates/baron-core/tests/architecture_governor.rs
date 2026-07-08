use std::fs;

use baron_core::architecture::ensure_architecture_governor;
use baron_core::config::{initialize_project_with_options, AdapterKind, ProjectPlatform};
use tempfile::tempdir;

#[test]
fn fullstack_can_expand_to_mobile_without_replacing_primary_platform() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Fullstack),
    )
    .unwrap();
    let config =
        initialize_project_with_options(&repo, None, &vault, Some(ProjectPlatform::Mobile))
            .unwrap();

    assert_eq!(config.platform, Some(ProjectPlatform::Fullstack));
    assert_eq!(config.platform_extensions, vec![ProjectPlatform::Mobile]);
}

#[test]
fn architecture_contract_uses_evidence_and_never_moves_existing_code() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-app");
    let vault = temp.path().join("Vault");
    let legacy = repo.join("weird-old-folder/server.ts");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, "export const server = true;").unwrap();
    let config = initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Fullstack),
    )
    .unwrap();

    let report = ensure_architecture_governor(&repo, &config).unwrap();

    assert!(legacy.exists());
    for path in [
        report.current_architecture,
        report.project_structure,
        report.boundaries,
        report.dependency_rules,
        report.expansion_rules,
    ] {
        assert!(path.exists(), "{}", path.display());
    }
    let structure =
        fs::read_to_string(repo.join("docs/baron/architecture/PROJECT_STRUCTURE.md")).unwrap();
    assert!(structure.contains("weird-old-folder"));
    assert!(structure.contains("must not move existing paths automatically"));
}

#[test]
fn repeated_extension_is_idempotent() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("app");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project_with_options(
        &repo,
        Some(AdapterKind::Generic),
        &vault,
        Some(ProjectPlatform::Backend),
    )
    .unwrap();
    initialize_project_with_options(&repo, None, &vault, Some(ProjectPlatform::Cloud)).unwrap();
    let config =
        initialize_project_with_options(&repo, None, &vault, Some(ProjectPlatform::Cloud)).unwrap();

    assert_eq!(config.platform, Some(ProjectPlatform::Backend));
    assert_eq!(config.platform_extensions, vec![ProjectPlatform::Cloud]);
}
