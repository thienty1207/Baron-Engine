use std::fs;
use std::path::{Path, PathBuf};

use baron_adapters::{
    install_adapter, load_managed_baseline, managed_payloads_for_adapter,
    reconcile_installed_managed_assets, record_managed_baseline, AgentAdapter, ManagedAssetPayload,
    ManagedMergeKind,
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

// Portability: target assertions are cross-platform and use deterministic
// fixtures; platform-specific filesystem behavior is covered in current tests.

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn payload(adapter: &str, path: &str, content: &str) -> ManagedAssetPayload {
    ManagedAssetPayload {
        adapter: adapter.to_string(),
        relative_path: PathBuf::from(path),
        merge_kind: ManagedMergeKind::FullText,
        content: content.to_string(),
    }
}

fn write_legacy_baseline(repo: &Path, adapter: &str, relative_path: &str, content: &str) {
    let hash = Sha256::digest(content.as_bytes());
    let hash = format!("{hash:x}");
    let state = repo.join(".baron/managed-state");
    write(
        &state.join("manifest.json"),
        &format!(
            "{{\n  \"schema_version\": 1,\n  \"installed_version\": \"4.2.2\",\n  \"records\": [{{\n    \"adapter\": \"{adapter}\",\n    \"relative_path\": \"{relative_path}\",\n    \"base_sha256\": \"{hash}\",\n    \"installed_version\": \"4.2.2\",\n    \"merge_kind\": \"full_text\"\n  }}]\n}}\n"
        ),
    );
    write(
        &state.join("base").join(adapter).join(relative_path),
        content,
    );
}

fn materialize_duplicate_state(repo: &Path) {
    let state = repo.join(".baron/managed-state");
    write(
        &state.join("manifest.json"),
        include_str!("fixtures/phase1/legacy-managed/duplicate/manifest.json"),
    );
    for adapter in ["agent", "codex"] {
        write(
            &state.join(format!("base/{adapter}/.baron/core/skills/legacy/SKILL.md")),
            include_str!(
                "fixtures/phase1/legacy-managed/base/agent/.baron/core/skills/legacy/SKILL.md"
            ),
        );
    }
}

fn materialize_unchanged_legacy_skill(repo: &Path) {
    let content = "# Legacy unchanged managed skill\n";
    write_legacy_baseline(
        repo,
        "codex",
        ".codex/skills/legacy-owned/SKILL.md",
        content,
    );
    write(&repo.join(".codex/skills/legacy-owned/SKILL.md"), content);
}

fn materialize_modified_managed_skill(repo: &Path) {
    let upstream = include_str!("../../../assets/core/skills/superpowers/SKILL.md");
    write_legacy_baseline(
        repo,
        "codex",
        ".codex/skills/superpowers/SKILL.md",
        upstream,
    );
    write(
        &repo.join(".codex/skills/superpowers/SKILL.md"),
        "# Locally modified former managed skill\n",
    );
}

/// EXPECTED RED: the current installer owns complete adapter-local copies.
/// The target keeps one Core tree and projects only thin Codex/Claude bridges.
#[test]
fn target_codex_and_claude_share_one_canonical_core_tree() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    assert!(temp
        .path()
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(temp
        .path()
        .join(".baron/core/agents/code-reviewer.toml")
        .is_file());
    assert!(!temp
        .path()
        .join(".codex/skills/superpowers/SKILL.md")
        .exists());
    assert!(!temp
        .path()
        .join(".claude/skills/superpowers/SKILL.md")
        .exists());
}

/// Promoted Phase 8 target: the semantic agent source is canonical Core and
/// Codex receives only a thin native wrapper over that source.
#[test]
fn target_core_has_one_canonical_semantic_agent_source() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    let source = temp.path().join(".baron/core/agents/code-reviewer.toml");
    assert!(source.is_file());
    let semantic = fs::read_to_string(source).unwrap();
    assert!(semantic.contains("code-reviewer"));
    let wrapper = temp.path().join(".codex/agents/code-reviewer.toml");
    assert!(wrapper.is_file());
    let wrapper_text = fs::read_to_string(wrapper).unwrap();
    assert!(wrapper_text.contains(".baron/core/agents/code-reviewer.toml"));
    assert!(wrapper_text.lines().count() < 30);
    let claude_wrapper = temp.path().join(".claude/agents/code-reviewer.md");
    assert!(claude_wrapper.is_file());
    assert!(fs::read_to_string(claude_wrapper)
        .unwrap()
        .contains(".baron/core/agents/code-reviewer.toml"));
}

/// Phase 3 target promotion: ambiguous ownership is user-owned content and is
/// preserved while the installer reports the collision.
#[test]
fn target_codex_preserves_a_colliding_custom_skill() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".codex/skills/frontend-design/SKILL.md");
    write(&path, "# User-owned skill collision\n");
    let before = fs::read(&path).unwrap();

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert_eq!(fs::read(path).unwrap(), before);
}

/// Phase 3 target promotion: ambiguous ownership is user-owned content and is
/// preserved while the installer reports the collision.
#[test]
fn target_claude_preserves_a_colliding_custom_skill() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".claude/skills/frontend-design/SKILL.md");
    write(&path, "# User-owned skill collision\n");
    let before = fs::read(&path).unwrap();

    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    assert_eq!(fs::read(path).unwrap(), before);
}

/// Phase 2 regression: invalid UTF-8 is never treated as missing content.
/// The safe reader propagates the error and leaves the old file untouched.
#[test]
fn target_invalid_utf8_managed_file_is_never_overwritten() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("AGENTS.md");
    let before = vec![0xff, 0xfe, 0xfd, 0x00];
    fs::write(&path, &before).unwrap();

    let result = install_adapter(temp.path(), AgentAdapter::Codex);

    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

/// Phase 2 regression: malformed hook JSON is preserved and reported.
#[test]
fn target_malformed_native_hooks_are_never_reset() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".codex/hooks.json");
    let before = b"{ not json\n".to_vec();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, &before).unwrap();

    let result = install_adapter(temp.path(), AgentAdapter::Codex);

    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

/// Phase 2 regression: malformed Claude settings are preserved and reported.
#[test]
fn target_malformed_claude_settings_are_never_reset() {
    let temp = tempdir().unwrap();
    let path = temp.path().join(".claude/settings.json");
    let before = b"{ not json\n".to_vec();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, &before).unwrap();

    let result = install_adapter(temp.path(), AgentAdapter::Claude);

    assert!(result.is_err());
    assert_eq!(fs::read(path).unwrap(), before);
}

/// Phase 2 regression: a directory at a managed file target is reported.
#[test]
fn target_directory_where_managed_file_is_expected_is_reported() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("AGENTS.md");
    fs::create_dir_all(&target).unwrap();
    let payload = payload("codex", "AGENTS.md", "managed content\n");
    record_managed_baseline(temp.path(), std::slice::from_ref(&payload), "4.2.2").unwrap();

    let result = reconcile_installed_managed_assets(temp.path(), &[payload], "4.2.2");

    assert!(result.is_err());
    assert!(target.is_dir());
}

/// Phase 2 regression: a deterministic legacy temp collision is preserved.
#[test]
fn target_temp_name_collision_does_not_destroy_existing_temp_content() {
    let temp = tempdir().unwrap();
    let collision = temp
        .path()
        .join(".baron/managed-state/base/codex/AGENTS.baron-tmp");
    write(&collision, "sentinel temp content\n");
    let payload = payload("codex", "AGENTS.md", "managed content\n");

    let result = record_managed_baseline(temp.path(), &[payload], "4.2.2");

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(collision).unwrap(),
        "sentinel temp content\n"
    );
}

/// Phase 3 target promotion: the manifest rejects two owners for one live
/// path.
#[test]
fn target_duplicate_live_path_owners_are_rejected() {
    let temp = tempdir().unwrap();
    materialize_duplicate_state(temp.path());

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();

    assert!(error.contains("live") || error.contains("owner"));
}

/// Phase 3 target promotion: legacy Core records publish with an explicit
/// Core owner.
#[test]
fn target_legacy_core_baseline_has_an_explicit_core_owner() {
    let temp = tempdir().unwrap();
    materialize_unchanged_legacy_skill(temp.path());
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let manifest =
        fs::read_to_string(temp.path().join(".baron/managed-state/manifest.json")).unwrap();

    assert!(manifest.contains("\"owner\""));
    assert!(manifest.contains("\"core\""));
}

/// Phase 3 target promotion: an unchanged former managed copy moves to Core
/// without changing its bytes.
#[test]
fn target_unchanged_legacy_skill_is_moved_to_canonical_core() {
    let temp = tempdir().unwrap();
    let legacy = temp.path().join(".codex/skills/legacy-owned/SKILL.md");
    materialize_unchanged_legacy_skill(temp.path());

    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert!(temp
        .path()
        .join(".baron/core/skills/legacy-owned/SKILL.md")
        .is_file());
    assert!(!legacy.exists());
}

/// Phase 3 target promotion: a modified former managed copy blocks migration
/// and remains available for review.
#[test]
fn target_modified_former_managed_skill_is_preserved_for_review() {
    let temp = tempdir().unwrap();
    let legacy = temp.path().join(".codex/skills/superpowers/SKILL.md");
    materialize_modified_managed_skill(temp.path());
    let before = fs::read(&legacy).unwrap();

    let error = install_adapter(temp.path(), AgentAdapter::Codex).unwrap_err();
    assert!(error.to_string().contains("conflict") || error.to_string().contains("modified"));

    assert_eq!(fs::read(legacy).unwrap(), before);
}

/// Promoted Phase 9 target: the high-level installer exposes Core payloads and
/// only native bridge/wrapper projections for each supported adapter.
#[test]
fn target_adapter_payloads_do_not_duplicate_the_complete_core_tree() {
    let codex = managed_payloads_for_adapter(AgentAdapter::Codex).unwrap();
    let claude = managed_payloads_for_adapter(AgentAdapter::Claude).unwrap();
    assert!(codex.iter().all(|payload| {
        let path = payload.relative_path.to_string_lossy().replace('\\', "/");
        !path.starts_with(".codex/skills/") || path.ends_with("/INDEX.md")
    }));
    assert!(claude.iter().all(|payload| {
        let path = payload.relative_path.to_string_lossy().replace('\\', "/");
        !path.starts_with(".claude/skills/")
            || path.ends_with("/INDEX.md")
            || path == ".claude/skills/baron-engine/SKILL.md"
    }));
}
