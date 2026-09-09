//! Phase 3 managed ownership and legacy migration contracts.
//!
//! These tests intentionally exercise only ownership metadata, legacy skill
//! classification, and migration safety. Canonical Core installation and
//! adapter bridge projection remain Phase 4 work.

use std::fs;
use std::path::Path;

use baron_adapters::{install_adapter, load_managed_baseline, AgentAdapter};
use tempfile::tempdir;

fn write(path: &Path, content: impl AsRef<[u8]>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn materialize_v1_fixture(repo: &Path) {
    let state = repo.join(".baron/managed-state");
    write(
        &state.join("manifest.json"),
        include_str!("fixtures/phase3/managed-state-v1/manifest.json"),
    );
    write(
        &state.join("base/agent/.baron/core/skills/legacy/SKILL.md"),
        include_str!(
            "fixtures/phase3/managed-state-v1/base/agent/.baron/core/skills/legacy/SKILL.md"
        ),
    );
    write(
        &state.join("base/codex/.codex/skills/legacy-owned/SKILL.md"),
        include_str!(
            "fixtures/phase3/managed-state-v1/base/codex/.codex/skills/legacy-owned/SKILL.md"
        ),
    );
}

#[test]
fn v1_generic_core_fixture_upgrades_without_rewriting_the_live_bytes() {
    let temp = tempdir().unwrap();
    materialize_v1_fixture(temp.path());
    let live = temp.path().join(".baron/core/skills/legacy/SKILL.md");
    let original = include_bytes!(
        "fixtures/phase3/managed-state-v1/base/agent/.baron/core/skills/legacy/SKILL.md"
    );
    write(&live, original);

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert_eq!(fs::read(&live).unwrap(), original);
    let manifest =
        fs::read_to_string(temp.path().join(".baron/managed-state/manifest.json")).unwrap();
    assert!(manifest.contains("\"schema_version\": 2"));
    assert!(manifest.contains("\"owner\": \"core\""));
}

#[test]
fn ownership_migration_rolls_back_and_retries_after_a_publication_collision() {
    let temp = tempdir().unwrap();
    let legacy = temp.path().join(".codex/skills/legacy-owned/SKILL.md");
    let content = include_bytes!(
        "fixtures/phase3/managed-state-v1/base/codex/.codex/skills/legacy-owned/SKILL.md"
    );
    let state = temp.path().join(".baron/managed-state");
    let manifest = r#"{
  "schema_version": 1,
  "installed_version": "4.2.2",
  "records": [{"adapter":"codex","relative_path":".codex/skills/legacy-owned/SKILL.md","base_sha256":"c38610dfee14c115a11ca1b77a78f4c0591301b387ba499d0930c2464ed1ebee","installed_version":"4.2.2","merge_kind":"full_text"}]
}
"#;
    write(&state.join("manifest.json"), manifest);
    write(
        &state.join("base/codex/.codex/skills/legacy-owned/SKILL.md"),
        content,
    );
    write(&legacy, content);

    let collision = state.join("base/core/.baron/core/skills/legacy-owned/SKILL.baron-tmp");
    write(&collision, "migration sentinel\n");

    let error = install_adapter(temp.path(), AgentAdapter::Codex)
        .unwrap_err()
        .to_string();
    assert!(error.contains("rolled back") || error.contains("collision"));
    assert_eq!(
        fs::read_to_string(state.join("manifest.json")).unwrap(),
        manifest
    );
    assert_eq!(fs::read(&legacy).unwrap(), content);
    assert!(!temp
        .path()
        .join(".baron/core/skills/legacy-owned/SKILL.md")
        .exists());
    assert_eq!(
        fs::read_to_string(&collision).unwrap(),
        "migration sentinel\n"
    );

    fs::remove_file(collision).unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    assert!(!legacy.exists());
    assert!(temp
        .path()
        .join(".baron/core/skills/legacy-owned/SKILL.md")
        .is_file());
}

#[test]
fn interrupted_legacy_move_resumes_from_a_verified_staged_core_copy() {
    let temp = tempdir().unwrap();
    materialize_v1_fixture(temp.path());
    let content = include_bytes!(
        "fixtures/phase3/managed-state-v1/base/codex/.codex/skills/legacy-owned/SKILL.md"
    );
    let legacy = temp.path().join(".codex/skills/legacy-owned/SKILL.md");
    let canonical = temp.path().join(".baron/core/skills/legacy-owned/SKILL.md");
    write(&legacy, content);
    write(&canonical, content);
    fs::remove_file(&legacy).unwrap();

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert!(!legacy.exists());
    assert_eq!(fs::read(&canonical).unwrap(), content);
    let manifest =
        fs::read_to_string(temp.path().join(".baron/managed-state/manifest.json")).unwrap();
    assert!(manifest.contains("\"schema_version\": 2"));
    assert!(manifest.contains("\"owner\": \"core\""));
}

#[test]
fn duplicate_effective_live_paths_are_rejected_before_manifest_publication() {
    let temp = tempdir().unwrap();
    let state = temp.path().join(".baron/managed-state");
    let duplicate = r#"{
  "schema_version": 2,
  "installed_version": "4.2.2",
  "minimum_writer_schema": 2,
  "records": [
    {"owner":"core","relative_path":".baron/core/skills/shared/SKILL.md","base_sha256":"0000000000000000000000000000000000000000000000000000000000000000","installed_version":"4.2.2","merge_kind":"full_text"},
    {"owner":"codex","relative_path":".baron/core/skills/shared/SKILL.md","base_sha256":"0000000000000000000000000000000000000000000000000000000000000000","installed_version":"4.2.2","merge_kind":"full_text"}
  ]
}
"#;
    write(&state.join("manifest.json"), duplicate);

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();
    assert!(error.contains("live") || error.contains("owner"));
    assert_eq!(
        fs::read_to_string(state.join("manifest.json")).unwrap(),
        duplicate
    );
}

#[test]
fn newer_minimum_writer_schema_is_refused_without_rewriting_state() {
    let temp = tempdir().unwrap();
    let state = temp.path().join(".baron/managed-state");
    let manifest = r#"{
  "schema_version": 2,
  "installed_version": "4.2.2",
  "minimum_writer_schema": 3,
  "records": []
}
"#;
    write(&state.join("manifest.json"), manifest);

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();
    assert!(error.contains("minimum writer schema"));
    assert_eq!(
        fs::read_to_string(state.join("manifest.json")).unwrap(),
        manifest
    );
}

#[test]
fn historical_unsupported_adapter_value_is_kept_as_generic_legacy_owner() {
    let temp = tempdir().unwrap();
    let legacy_adapter = String::from_utf8(vec![114, 101, 97, 115, 111, 110, 105, 120]).unwrap();
    let legacy_path = format!(".{legacy_adapter}/skills/legacy/SKILL.md");
    let content = b"# Historical adapter asset\n";
    let hash = sha256(content);
    let state = temp.path().join(".baron/managed-state");
    write(
        &state.join("manifest.json"),
        format!(
            "{{\n  \"schema_version\": 1,\n  \"installed_version\": \"4.2.2\",\n  \"records\": [{{\"adapter\":\"{legacy_adapter}\",\"relative_path\":\"{legacy_path}\",\"base_sha256\":\"{hash}\",\"installed_version\":\"4.2.2\",\"merge_kind\":\"full_text\"}}]\n}}\n"
        ),
    );
    write(
        &state.join(format!("base/{legacy_adapter}/{legacy_path}")),
        content,
    );
    write(&temp.path().join(&legacy_path), content);

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    let baseline = load_managed_baseline(temp.path()).unwrap();
    let record = baseline
        .records
        .iter()
        .find(|record| record.relative_path == Path::new(&legacy_path))
        .unwrap();
    assert!(matches!(
        &record.owner,
        baron_adapters::ManagedOwner::UnsupportedLegacy(value) if value == &legacy_adapter
    ));
    assert_eq!(record.adapter, legacy_adapter);
    assert_eq!(baseline.schema_version, 2);
}

#[test]
fn unchanged_legacy_skill_is_migrated_without_changing_its_bytes() {
    let temp = tempdir().unwrap();
    let legacy = temp.path().join(".codex/skills/legacy-owned/SKILL.md");
    let content = include_bytes!(
        "fixtures/phase3/managed-state-v1/base/codex/.codex/skills/legacy-owned/SKILL.md"
    );
    let state = temp.path().join(".baron/managed-state");
    let manifest = r#"{
  "schema_version": 1,
  "installed_version": "4.2.2",
  "records": [{"adapter":"codex","relative_path":".codex/skills/legacy-owned/SKILL.md","base_sha256":"c38610dfee14c115a11ca1b77a78f4c0591301b387ba499d0930c2464ed1ebee","installed_version":"4.2.2","merge_kind":"full_text"}]
}
"#;
    write(&state.join("manifest.json"), manifest);
    write(
        &state.join("base/codex/.codex/skills/legacy-owned/SKILL.md"),
        content,
    );
    write(&legacy, content);

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    let canonical = temp.path().join(".baron/core/skills/legacy-owned/SKILL.md");
    assert_eq!(fs::read(&canonical).unwrap(), content);
    assert!(!legacy.exists());
}

#[test]
fn modified_former_managed_skill_blocks_migration_and_preserves_bytes() {
    let temp = tempdir().unwrap();
    let legacy = temp.path().join(".codex/skills/legacy-owned/SKILL.md");
    let baseline = b"# Baron baseline\n";
    let modified = b"# User changed former Baron skill\n";
    let state = temp.path().join(".baron/managed-state");
    let hash = sha256(baseline);
    write(
        &state.join("manifest.json"),
        format!(
            "{{\n  \"schema_version\": 1,\n  \"installed_version\": \"4.2.2\",\n  \"records\": [{{\"adapter\":\"codex\",\"relative_path\":\".codex/skills/legacy-owned/SKILL.md\",\"base_sha256\":\"{hash}\",\"installed_version\":\"4.2.2\",\"merge_kind\":\"full_text\"}}]\n}}\n"
        ),
    );
    write(
        &state.join("base/codex/.codex/skills/legacy-owned/SKILL.md"),
        baseline,
    );
    write(&legacy, modified);

    let error = install_adapter(temp.path(), AgentAdapter::Codex)
        .unwrap_err()
        .to_string();

    assert!(error.contains("conflict") || error.contains("modified"));
    assert_eq!(fs::read(&legacy).unwrap(), modified);
}

#[test]
fn ambiguous_skill_collision_is_preserved_without_a_legacy_claim() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".codex/skills/frontend-design/SKILL.md");
    let original = b"# User-owned skill\n";
    write(&path, original);

    let report = install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(!report
        .core
        .conflicts
        .iter()
        .any(|entry| entry.ends_with(".codex/skills/frontend-design/SKILL.md")));
    let baseline = load_managed_baseline(temp.path()).unwrap();
    assert!(!baseline
        .records
        .iter()
        .any(|record| record.relative_path == path));
}

#[test]
fn unregistered_user_skill_is_preserved_without_claiming_managed_ownership() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".claude/skills/personal/SKILL.md");
    let original = b"# Personal Claude skill\n";
    write(&path, original);

    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    assert_eq!(fs::read(&path).unwrap(), original);
    let baseline = load_managed_baseline(temp.path()).unwrap();
    assert!(!baseline
        .records
        .iter()
        .any(|record| record.relative_path == Path::new(".claude/skills/personal/SKILL.md")));
}

#[test]
fn installation_order_keeps_core_owner_stable() {
    fn manifest_after(order: &[AgentAdapter]) -> String {
        let temp = tempdir().unwrap();
        for adapter in order {
            install_adapter(temp.path(), *adapter).unwrap();
        }
        fs::read_to_string(temp.path().join(".baron/managed-state/manifest.json")).unwrap()
    }

    let codex_claude = manifest_after(&[AgentAdapter::Codex, AgentAdapter::Claude]);
    let claude_codex = manifest_after(&[AgentAdapter::Claude, AgentAdapter::Codex]);
    assert_eq!(codex_claude, claude_codex);
    assert!(codex_claude.contains("\"owner\": \"core\""));

    let generic_codex_claude = manifest_after(&[AgentAdapter::Codex, AgentAdapter::Claude]);
    let generic_claude_codex = manifest_after(&[AgentAdapter::Claude, AgentAdapter::Codex]);
    assert_eq!(generic_codex_claude, generic_claude_codex);
    assert!(generic_codex_claude.contains("\"owner\": \"core\""));

    fn legacy_generic_manifest_after(order: &[AgentAdapter]) -> String {
        let temp = tempdir().unwrap();
        materialize_v1_fixture(temp.path());
        write(
            &temp.path().join(".baron/core/skills/legacy/SKILL.md"),
            include_bytes!(
                "fixtures/phase3/managed-state-v1/base/agent/.baron/core/skills/legacy/SKILL.md"
            ),
        );
        for adapter in order {
            install_adapter(temp.path(), *adapter).unwrap();
        }
        fs::read_to_string(temp.path().join(".baron/managed-state/manifest.json")).unwrap()
    }

    let legacy_codex_claude =
        legacy_generic_manifest_after(&[AgentAdapter::Codex, AgentAdapter::Claude]);
    let legacy_claude_codex =
        legacy_generic_manifest_after(&[AgentAdapter::Claude, AgentAdapter::Codex]);
    assert_eq!(legacy_codex_claude, legacy_claude_codex);
    assert!(legacy_codex_claude.contains("\"owner\": \"core\""));
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
