use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use baron_adapters::{
    install_adapter, load_managed_baseline, plan_managed_update,
    reconcile_installed_managed_assets, record_managed_baseline, AgentAdapter, ManagedAssetPayload,
    ManagedMergeKind, UpdateDisposition,
};
use tempfile::tempdir;

// Portability: the default fixtures are cross-platform; symlink and permission
// mechanics are explicitly guarded below for Unix or Windows.

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn files(root: &Path) -> BTreeSet<String> {
    fn visit(root: &Path, current: &Path, output: &mut BTreeSet<String>) {
        let entries = fs::read_dir(current).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, output);
            } else {
                output.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }

    let mut output = BTreeSet::new();
    visit(root, root, &mut output);
    output
}

fn materialize_valid_legacy_state(repo: &Path) {
    let state = repo.join(".baron/managed-state");
    write(
        &state.join("manifest.json"),
        include_str!("fixtures/phase1/legacy-managed/manifest.json"),
    );
    write(
        &state.join("base/agent/.baron/core/skills/legacy/SKILL.md"),
        include_str!(
            "fixtures/phase1/legacy-managed/base/agent/.baron/core/skills/legacy/SKILL.md"
        ),
    );
}

fn materialize_duplicate_legacy_state(repo: &Path) {
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

fn payload(adapter: &str, path: &str, content: &str) -> ManagedAssetPayload {
    ManagedAssetPayload {
        adapter: adapter.to_string(),
        relative_path: PathBuf::from(path),
        merge_kind: ManagedMergeKind::FullText,
        content: content.to_string(),
    }
}

#[test]
fn current_codex_only_fixture_materializes_the_current_codex_surface() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    let report = install_adapter(repo, AgentAdapter::Codex).unwrap();

    assert_eq!(report.adapter, "codex");
    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join(".codex/INDEX.md").is_file());
    // Phase 8 keeps the legacy path untouched when supplied, but fresh
    // installations no longer create a Codex skill-routing tree.
    assert!(!repo.join(".codex/skills").exists());
    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".baron/core/agents/code-reviewer.toml").is_file());
    assert!(!repo.join(".codex/skills/superpowers/SKILL.md").exists());
    let baseline = load_managed_baseline(repo).unwrap();
    assert!(baseline.records.iter().any(|record| {
        matches!(&record.owner, baron_adapters::ManagedOwner::Core)
            && record.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
    }));
}

#[test]
fn current_claude_only_fixture_materializes_the_current_claude_surface() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    let report = install_adapter(repo, AgentAdapter::Claude).unwrap();

    assert_eq!(report.adapter, "claude");
    assert!(repo.join("CLAUDE.md").is_file());
    assert!(repo.join(".claude/commands/baron-context.md").is_file());
    assert!(repo.join(".claude/skills/INDEX.md").is_file());
    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".baron/core/agents/code-reviewer.toml").is_file());
    assert!(!repo.join(".claude/skills/superpowers/SKILL.md").exists());
    let baseline = load_managed_baseline(repo).unwrap();
    assert!(baseline.records.iter().any(|record| {
        matches!(&record.owner, baron_adapters::ManagedOwner::Core)
            && record.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
    }));
}

#[test]
fn current_codex_and_claude_installations_are_order_independent() {
    fn install_in_order(first: AgentAdapter, second: AgentAdapter) -> BTreeSet<String> {
        let temp = tempdir().unwrap();
        install_adapter(temp.path(), first).unwrap();
        install_adapter(temp.path(), second).unwrap();
        assert!(temp.path().join("AGENTS.md").is_file());
        assert!(temp.path().join("CLAUDE.md").is_file());
        files(temp.path())
    }

    let codex_then_claude = install_in_order(AgentAdapter::Codex, AgentAdapter::Claude);
    let claude_then_codex = install_in_order(AgentAdapter::Claude, AgentAdapter::Codex);
    assert_eq!(codex_then_claude, claude_then_codex);
}

#[test]
fn current_codex_preserves_user_text_hooks_and_unlisted_skills() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join("AGENTS.md"),
        "# User instructions\n\nKeep this paragraph.\n",
    );
    write(
        &repo.join(".codex/hooks.json"),
        r#"{
  "third_party": {"enabled": true},
  "hooks": {"SessionStart": [{"hooks": [{"type": "command", "command": "third-party"}]}]}
}
"#,
    );
    write(
        &repo.join(".codex/skills/custom-user/SKILL.md"),
        "# Custom user skill\n",
    );

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("Keep this paragraph."));
    let hooks: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join(".codex/hooks.json")).unwrap()).unwrap();
    assert_eq!(hooks["third_party"]["enabled"], true);
    assert!(hooks["hooks"]["SessionStart"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry.to_string().contains("third-party")));
    assert_eq!(
        fs::read_to_string(repo.join(".codex/skills/custom-user/SKILL.md")).unwrap(),
        "# Custom user skill\n"
    );
}

#[test]
fn current_claude_preserves_user_text_settings_commands_and_skills() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    write(
        &repo.join("CLAUDE.md"),
        "# User instructions\n\nKeep this Claude paragraph.\n",
    );
    write(
        &repo.join(".claude/settings.json"),
        r#"{
  "personal_setting": "keep",
  "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "personal-stop"}]}]}
}
"#,
    );
    write(
        &repo.join(".claude/commands/personal.md"),
        "# Personal Claude command\n",
    );
    write(
        &repo.join(".claude/skills/personal/SKILL.md"),
        "# Personal Claude skill\n",
    );

    install_adapter(repo, AgentAdapter::Claude).unwrap();

    let claude = fs::read_to_string(repo.join("CLAUDE.md")).unwrap();
    assert!(claude.contains("Keep this Claude paragraph."));
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert_eq!(settings["personal_setting"], "keep");
    assert!(settings["hooks"]["Stop"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry.to_string().contains("personal-stop")));
    assert_eq!(
        fs::read_to_string(repo.join(".claude/commands/personal.md")).unwrap(),
        "# Personal Claude command\n"
    );
    assert_eq!(
        fs::read_to_string(repo.join(".claude/skills/personal/SKILL.md")).unwrap(),
        "# Personal Claude skill\n"
    );
}

#[test]
#[ignore = "historical Phase 1 behavior intentionally hardened in Phase 2"]
fn current_installer_replaces_invalid_utf8_managed_text() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let before = vec![0xff, 0xfe, 0xfd, 0x00];
    fs::write(repo.join("AGENTS.md"), &before).unwrap();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let after = fs::read(repo.join("AGENTS.md")).unwrap();
    assert_ne!(after, before);
    assert!(String::from_utf8(after).is_ok());
}

#[test]
#[ignore = "historical Phase 1 behavior intentionally hardened in Phase 2"]
fn current_installer_resets_malformed_native_hooks_to_managed_json() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    fs::create_dir_all(repo.join(".codex")).unwrap();
    fs::write(repo.join(".codex/hooks.json"), b"{ not json\n").unwrap();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let hooks: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join(".codex/hooks.json")).unwrap()).unwrap();
    assert!(hooks.is_object());
}

#[test]
fn legacy_managed_state_fixture_loads_the_existing_schema() {
    let temp = tempdir().unwrap();
    materialize_valid_legacy_state(temp.path());

    let baseline = load_managed_baseline(temp.path()).unwrap();

    assert_eq!(baseline.schema_version, 1);
    assert_eq!(baseline.records.len(), 1);
    assert_eq!(baseline.records[0].adapter, "agent");
    assert_eq!(
        baseline.records[0].relative_path,
        Path::new(".baron/core/skills/legacy/SKILL.md")
    );
}

#[test]
fn legacy_managed_state_missing_manifest_is_reported() {
    let temp = tempdir().unwrap();

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();

    assert!(error.contains("missing"));
}

#[test]
fn legacy_managed_state_unreadable_manifest_bytes_are_reported() {
    let temp = tempdir().unwrap();
    let manifest = temp.path().join(".baron/managed-state/manifest.json");
    if let Some(parent) = manifest.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&manifest, [0xff, 0xfe, 0xfd]).unwrap();

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();

    assert!(error.contains("missing") || error.contains("malformed"));
}

#[test]
fn legacy_managed_state_manifest_directory_is_reported_as_unreadable() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".baron/managed-state/manifest.json")).unwrap();

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();

    assert!(!error.is_empty());
}

#[cfg(unix)]
#[test]
fn unix_unreadable_managed_manifest_is_guarded_by_permission_semantics() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let manifest = temp.path().join(".baron/managed-state/manifest.json");
    write(
        &manifest,
        include_str!("fixtures/phase1/legacy-managed/manifest.json"),
    );
    let mut permissions = fs::metadata(&manifest).unwrap().permissions();
    permissions.set_mode(0o0);
    fs::set_permissions(&manifest, permissions).unwrap();

    // Privileged Unix runners may still read mode-000 files. In that case the
    // platform cannot provide a meaningful unreadable-file assertion.
    if fs::read(&manifest).is_ok() {
        let mut restore = fs::metadata(&manifest).unwrap().permissions();
        restore.set_mode(0o600);
        fs::set_permissions(&manifest, restore).unwrap();
        return;
    }

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();
    let mut restore = fs::metadata(&manifest).unwrap().permissions();
    restore.set_mode(0o600);
    fs::set_permissions(&manifest, restore).unwrap();

    assert!(error.contains("missing") || error.contains("read"));
}

#[test]
fn legacy_managed_state_corrupt_and_truncated_manifests_are_reported() {
    for fixture in ["corrupt", "truncated"] {
        let temp = tempdir().unwrap();
        let manifest = temp.path().join(".baron/managed-state/manifest.json");
        write(
            &manifest,
            match fixture {
                "corrupt" => include_str!("fixtures/phase1/legacy-managed/corrupt/manifest.json"),
                "truncated" => {
                    include_str!("fixtures/phase1/legacy-managed/truncated/manifest.json")
                }
                _ => unreachable!(),
            },
        );

        let error = load_managed_baseline(temp.path()).unwrap_err().to_string();
        assert!(error.contains("malformed"), "fixture {fixture}: {error}");
    }
}

#[test]
fn legacy_managed_state_missing_baseline_copy_is_reported() {
    let temp = tempdir().unwrap();
    write(
        &temp.path().join(".baron/managed-state/manifest.json"),
        include_str!("fixtures/phase1/legacy-managed/manifest.json"),
    );

    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();

    assert!(error.contains("baseline copy is missing"));
}

#[test]
fn legacy_managed_state_distinguishes_unchanged_and_modified_files() {
    let unchanged = tempdir().unwrap();
    materialize_valid_legacy_state(unchanged.path());
    write(
        &unchanged.path().join(".baron/core/skills/legacy/SKILL.md"),
        include_str!(
            "fixtures/phase1/legacy-managed/base/agent/.baron/core/skills/legacy/SKILL.md"
        ),
    );
    let unchanged_plan = plan_managed_update(
        unchanged.path(),
        &[payload(
            "agent",
            ".baron/core/skills/legacy/SKILL.md",
            "# Upstream Core Skill\n",
        )],
    )
    .unwrap();
    assert_eq!(
        unchanged_plan.actions[0].disposition,
        UpdateDisposition::TakeUpstream
    );

    let modified = tempdir().unwrap();
    materialize_valid_legacy_state(modified.path());
    write(
        &modified.path().join(".baron/core/skills/legacy/SKILL.md"),
        "# Local modified Core Skill\n",
    );
    let modified_plan = plan_managed_update(
        modified.path(),
        &[payload(
            "agent",
            ".baron/core/skills/legacy/SKILL.md",
            "# Upstream Core Skill\n",
        )],
    )
    .unwrap();
    assert_eq!(
        modified_plan.actions[0].disposition,
        UpdateDisposition::Conflict
    );
}

#[test]
fn managed_paths_reject_parent_and_absolute_escapes() {
    let temp = tempdir().unwrap();
    let parent_error = baron_adapters::managed_target_path(temp.path(), Path::new("../escape.md"))
        .unwrap_err()
        .to_string();
    assert!(parent_error.contains("relative") || parent_error.contains("escapes"));

    let absolute = if cfg!(windows) {
        PathBuf::from(r"C:\escape.md")
    } else {
        PathBuf::from("/escape.md")
    };
    let absolute_error = baron_adapters::managed_target_path(temp.path(), &absolute)
        .unwrap_err()
        .to_string();
    assert!(absolute_error.contains("relative") || absolute_error.contains("escapes"));

    let payload = payload("codex", "../escape.md", "unsafe");
    let error = record_managed_baseline(temp.path(), &[payload], "4.2.2")
        .unwrap_err()
        .to_string();
    assert!(error.contains("relative") || error.contains("escapes"));

    let escaped_manifest = temp.path().join(".baron/managed-state/manifest.json");
    write(
        &escaped_manifest,
        r#"{
  "schema_version": 1,
  "installed_version": "4.2.2",
  "records": [{
    "adapter": "codex",
    "relative_path": "../outside.md",
    "base_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
    "installed_version": "4.2.2",
    "merge_kind": "full_text"
  }]
}
"#,
    );
    let manifest_error = load_managed_baseline(temp.path()).unwrap_err().to_string();
    assert!(manifest_error.contains("relative") || manifest_error.contains("escapes"));
}

#[cfg(unix)]
#[test]
fn unix_symlinked_managed_path_is_rejected() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let outside = temp.path().join("outside");
    let repo = temp.path().join("repo");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&repo).unwrap();
    symlink(&outside, repo.join("linked")).unwrap();

    let error = baron_adapters::managed_target_path(repo, Path::new("linked/escape.md"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("symlink") || error.contains("junction"));
}

#[cfg(windows)]
#[test]
fn windows_reparse_point_managed_path_is_rejected_when_supported() {
    use std::os::windows::fs::symlink_dir;

    let temp = tempdir().unwrap();
    let outside = temp.path().join("outside");
    let repo = temp.path().join("repo");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&repo).unwrap();
    if symlink_dir(&outside, repo.join("linked")).is_err() {
        return;
    }

    let error = baron_adapters::managed_target_path(repo, Path::new("linked/escape.md"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("symlink") || error.contains("junction"));
}

#[test]
fn legacy_duplicate_fixture_is_available_for_target_owner_tests() {
    let temp = tempdir().unwrap();
    materialize_duplicate_legacy_state(temp.path());
    let error = load_managed_baseline(temp.path()).unwrap_err().to_string();
    assert!(error.contains("live") || error.contains("owner"));
}

#[test]
#[ignore = "historical Phase 1 behavior intentionally hardened in Phase 2"]
fn managed_target_directory_is_reported_when_a_file_is_expected() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("AGENTS.md");
    fs::create_dir_all(&target).unwrap();
    let payload = payload("codex", "AGENTS.md", "managed content\n");
    record_managed_baseline(temp.path(), std::slice::from_ref(&payload), "4.2.2").unwrap();

    let report = reconcile_installed_managed_assets(temp.path(), &[payload], "4.2.2").unwrap();

    assert!(report.applied_paths.is_empty());
    assert!(target.is_dir());
}

#[test]
fn current_managed_plan_observes_a_deterministically_ordered_concurrent_mutation() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let temp = tempdir().unwrap();
    let repo = temp.path().to_path_buf();
    let target = repo.join("AGENTS.md");
    let upstream_payload = payload("codex", "AGENTS.md", "upstream content\n");
    record_managed_baseline(
        &repo,
        &[payload("codex", "AGENTS.md", "baseline content\n")],
        "4.2.2",
    )
    .unwrap();
    write(&target, "baseline content\n");

    let ready = Arc::new(Barrier::new(2));
    let mutated = Arc::new(Barrier::new(2));
    let writer_repo = repo.clone();
    let writer_ready = Arc::clone(&ready);
    let writer_mutated = Arc::clone(&mutated);
    let writer = thread::spawn(move || {
        writer_ready.wait();
        write(
            &writer_repo.join("AGENTS.md"),
            "concurrent local mutation\n",
        );
        writer_mutated.wait();
    });

    let planner_repo = repo.clone();
    let planner_ready = Arc::clone(&ready);
    let planner_mutated = Arc::clone(&mutated);
    let planner = thread::spawn(move || {
        planner_ready.wait();
        planner_mutated.wait();
        plan_managed_update(&planner_repo, &[upstream_payload]).unwrap()
    });

    writer.join().unwrap();
    let plan = planner.join().unwrap();

    assert_eq!(plan.actions[0].disposition, UpdateDisposition::Conflict);
}

#[test]
fn interrupted_legacy_managed_state_fixture_is_deterministic_input() {
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/phase1/legacy-managed/interrupted/manifest.json"
    ))
    .unwrap();

    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["state"], "interrupted");
    assert_eq!(value["last_checkpoint"], "after_baseline_backup");
    assert!(value["records"].as_array().unwrap().is_empty());
}
