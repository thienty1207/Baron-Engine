use std::fs;
use std::path::{Path, PathBuf};

use baron_adapters::{
    ensure_managed_baseline, install_adapter, load_managed_baseline, managed_payloads_for_adapter,
    plan_managed_update, reconcile_installed_managed_assets, record_managed_baseline,
    replace_managed_baseline, AgentAdapter, ManagedAssetPayload, ManagedMergeKind,
    UpdateDisposition,
};
use tempfile::tempdir;

fn payload(path: &str, kind: ManagedMergeKind, content: &str) -> ManagedAssetPayload {
    payload_for("codex", path, kind, content)
}

fn payload_for(
    adapter: &str,
    path: &str,
    kind: ManagedMergeKind,
    content: &str,
) -> ManagedAssetPayload {
    ManagedAssetPayload {
        adapter: adapter.to_string(),
        relative_path: PathBuf::from(path),
        merge_kind: kind,
        content: content.to_string(),
    }
}

const BASE_MARKER: &str = "<!-- BARON:MANAGED:START -->\nbase\n<!-- BARON:MANAGED:END -->";
const UPSTREAM_MARKER: &str = "<!-- BARON:MANAGED:START -->\nupstream\n<!-- BARON:MANAGED:END -->";

#[test]
fn local_reconcile_restores_only_a_missing_embedded_managed_asset() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    let missing = repo.join(".codex/skills/superpowers/SKILL.md");
    fs::remove_file(&missing).unwrap();
    let custom = repo.join(".codex/skills/custom/SKILL.md");
    fs::create_dir_all(custom.parent().unwrap()).unwrap();
    fs::write(&custom, "custom user skill").unwrap();

    let report = reconcile_installed_managed_assets(
        repo,
        &managed_payloads_for_adapter(AgentAdapter::Codex).unwrap(),
        "3.3.0",
    )
    .unwrap();

    assert!(report.conflicts.is_empty());
    assert!(report
        .applied_paths
        .iter()
        .any(|path| path.ends_with("SKILL.md")));
    assert!(missing.is_file());
    assert_eq!(fs::read_to_string(custom).unwrap(), "custom user skill");
}

#[test]
fn local_reconcile_leaves_an_ambiguous_managed_edit_untouched() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    let agents = repo.join("AGENTS.md");
    fs::write(
        &agents,
        "<!-- BARON:MANAGED:START -->\nlocal change\n<!-- BARON:MANAGED:END -->\n",
    )
    .unwrap();
    let before = fs::read_to_string(&agents).unwrap();
    let mut upstream = managed_payloads_for_adapter(AgentAdapter::Codex).unwrap();
    let agents_payload = upstream
        .iter_mut()
        .find(|payload| payload.relative_path == Path::new("AGENTS.md"))
        .unwrap();
    agents_payload.content = UPSTREAM_MARKER.to_string();

    let report = reconcile_installed_managed_assets(repo, &upstream, "3.3.0").unwrap();

    assert_eq!(report.conflicts, vec![PathBuf::from("AGENTS.md")]);
    assert!(report.applied_paths.is_empty());
    assert_eq!(fs::read_to_string(&agents).unwrap(), before);
}

#[test]
fn first_install_records_relative_managed_baseline_copies() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let baseline = load_managed_baseline(repo).unwrap();
    assert_eq!(baseline.schema_version, 1);
    assert!(baseline.records.iter().any(|record| {
        record.relative_path == Path::new("AGENTS.md")
            && record.merge_kind == ManagedMergeKind::MarkerBlock
    }));
    assert!(baseline
        .records
        .iter()
        .all(|record| !record.relative_path.is_absolute()));
    assert!(baseline.records.iter().all(|record| !record
        .relative_path
        .components()
        .any(|part| matches!(part, std::path::Component::ParentDir))));
    assert!(repo
        .join(".baron/managed-state/base/codex/AGENTS.md")
        .is_file());
}

#[test]
fn rendered_candidate_matches_a_fresh_codex_install_without_reading_the_target() {
    let temp = tempdir().unwrap();
    let repo = temp.path();

    install_adapter(repo, AgentAdapter::Codex).unwrap();

    let plan = plan_managed_update(
        repo,
        &managed_payloads_for_adapter(AgentAdapter::Codex).unwrap(),
    )
    .unwrap();
    assert!(plan
        .actions
        .iter()
        .all(|action| action.disposition == UpdateDisposition::Identical));
}

#[test]
fn planner_includes_a_new_upstream_asset_for_its_own_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let baseline = vec![
        payload_for(
            "codex",
            "AGENTS.md",
            ManagedMergeKind::MarkerBlock,
            BASE_MARKER,
        ),
        payload_for(
            "claude",
            "CLAUDE.md",
            ManagedMergeKind::FullText,
            "base claude",
        ),
    ];
    record_managed_baseline(repo, &baseline, "3.3.0").unwrap();
    fs::write(repo.join("AGENTS.md"), BASE_MARKER).unwrap();
    fs::write(repo.join("CLAUDE.md"), "base claude").unwrap();

    let upstream = vec![
        payload_for(
            "codex",
            "AGENTS.md",
            ManagedMergeKind::MarkerBlock,
            BASE_MARKER,
        ),
        payload_for(
            "claude",
            "CLAUDE.md",
            ManagedMergeKind::FullText,
            "base claude",
        ),
        payload_for(
            "claude",
            ".claude/commands/baron.md",
            ManagedMergeKind::FullText,
            "new claude command",
        ),
    ];

    let plan = plan_managed_update(repo, &upstream).unwrap();

    let action = plan
        .actions
        .iter()
        .find(|action| action.relative_path == Path::new(".claude/commands/baron.md"))
        .expect("a new upstream Claude asset must be in the managed update plan");
    assert_eq!(action.disposition, UpdateDisposition::TakeUpstream);
    assert_eq!(
        action.resolved_content.as_deref(),
        Some("new claude command")
    );
}

#[test]
fn planner_applies_the_safe_three_way_matrix_without_writing_targets() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let base = vec![
        payload("AGENTS.md", ManagedMergeKind::MarkerBlock, BASE_MARKER),
        payload(".codex/core/take.md", ManagedMergeKind::FullText, "base"),
        payload(".codex/core/keep.md", ManagedMergeKind::FullText, "base"),
        payload(".codex/core/same.md", ManagedMergeKind::FullText, "base"),
        payload(
            ".codex/core/conflict.md",
            ManagedMergeKind::FullText,
            "base",
        ),
    ];
    record_managed_baseline(repo, &base, "3.3.0").unwrap();

    fs::write(
        repo.join("AGENTS.md"),
        format!("# User Header\n\n{BASE_MARKER}\n\nKeep this local text.\n"),
    )
    .unwrap();
    fs::create_dir_all(repo.join(".codex/core")).unwrap();
    fs::write(repo.join(".codex/core/take.md"), "base").unwrap();
    fs::write(repo.join(".codex/core/keep.md"), "local").unwrap();
    fs::write(repo.join(".codex/core/same.md"), "same").unwrap();
    fs::write(repo.join(".codex/core/conflict.md"), "local").unwrap();

    let upstream = vec![
        payload("AGENTS.md", ManagedMergeKind::MarkerBlock, UPSTREAM_MARKER),
        payload(
            ".codex/core/take.md",
            ManagedMergeKind::FullText,
            "upstream",
        ),
        payload(".codex/core/keep.md", ManagedMergeKind::FullText, "base"),
        payload(".codex/core/same.md", ManagedMergeKind::FullText, "same"),
        payload(
            ".codex/core/conflict.md",
            ManagedMergeKind::FullText,
            "upstream",
        ),
    ];

    let plan = plan_managed_update(repo, &upstream).unwrap();
    assert_eq!(
        plan.action_for("AGENTS.md").unwrap().disposition,
        UpdateDisposition::AutoMerge
    );
    assert!(plan
        .action_for("AGENTS.md")
        .unwrap()
        .resolved_content
        .as_deref()
        .unwrap()
        .contains("# User Header"));
    assert!(plan
        .action_for("AGENTS.md")
        .unwrap()
        .resolved_content
        .as_deref()
        .unwrap()
        .contains("upstream"));
    assert_eq!(
        plan.action_for(".codex/core/take.md").unwrap().disposition,
        UpdateDisposition::TakeUpstream
    );
    assert_eq!(
        plan.action_for(".codex/core/keep.md").unwrap().disposition,
        UpdateDisposition::KeepLocal
    );
    assert_eq!(
        plan.action_for(".codex/core/same.md").unwrap().disposition,
        UpdateDisposition::Identical
    );
    assert_eq!(
        plan.action_for(".codex/core/conflict.md")
            .unwrap()
            .disposition,
        UpdateDisposition::Conflict
    );
    assert_eq!(
        fs::read_to_string(repo.join("AGENTS.md")).unwrap(),
        format!("# User Header\n\n{BASE_MARKER}\n\nKeep this local text.\n")
    );
}

#[test]
fn planner_excludes_custom_assets_source_plans_harness_and_vault_paths() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let managed = vec![payload(
        ".codex/skills/superpowers/SKILL.md",
        ManagedMergeKind::FullText,
        "base",
    )];
    record_managed_baseline(repo, &managed, "3.3.0").unwrap();
    fs::create_dir_all(repo.join(".codex/skills/custom-domain")).unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    fs::write(repo.join(".codex/skills/custom-domain/SKILL.md"), "custom").unwrap();
    fs::write(repo.join("src/app.rs"), "user source").unwrap();
    fs::write(repo.join("docs/baron/plans/CURRENT.md"), "user plan").unwrap();
    fs::write(repo.join("docs/baron/harness/STORY.md"), "user harness").unwrap();

    let plan = plan_managed_update(repo, &managed).unwrap();
    assert_eq!(plan.actions.len(), 1);
    assert!(plan.preserved_paths.iter().any(|path| path == "src/app.rs"));
    assert!(plan
        .preserved_paths
        .iter()
        .any(|path| path == ".codex/skills/custom-domain/SKILL.md"));
    assert!(plan
        .preserved_paths
        .iter()
        .any(|path| path == "docs/baron/plans/CURRENT.md"));
    assert!(plan
        .preserved_paths
        .iter()
        .any(|path| path == "docs/baron/harness/STORY.md"));
}

#[test]
fn planner_preservation_preview_skips_dependency_artifacts_and_reports_truncation() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let managed = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];
    record_managed_baseline(repo, &managed, "3.3.0").unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::create_dir_all(repo.join("node_modules/large-package")).unwrap();
    fs::write(repo.join("src/app.rs"), "user source").unwrap();
    fs::write(
        repo.join("node_modules/large-package/generated.js"),
        "dependency artifact",
    )
    .unwrap();
    for index in 0..300 {
        fs::write(repo.join("src").join(format!("file-{index}.rs")), "source").unwrap();
    }

    let plan = plan_managed_update(repo, &managed).unwrap();

    assert!(plan.preserved_paths.iter().any(|path| path == "src/app.rs"));
    assert!(!plan
        .preserved_paths
        .iter()
        .any(|path| path.contains("node_modules")));
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("bounded limit")));
}

#[test]
fn successful_baseline_replacement_moves_the_next_update_ancestor_forward() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let old = vec![payload(
        ".codex/core/SKILL.md",
        ManagedMergeKind::FullText,
        "old",
    )];
    record_managed_baseline(repo, &old, "3.3.0").unwrap();
    fs::create_dir_all(repo.join(".codex/core")).unwrap();
    fs::write(repo.join(".codex/core/SKILL.md"), "new").unwrap();
    let new = vec![payload(
        ".codex/core/SKILL.md",
        ManagedMergeKind::FullText,
        "new",
    )];

    replace_managed_baseline(repo, &new, "3.4.0").unwrap();

    let baseline = load_managed_baseline(repo).unwrap();
    assert_eq!(baseline.installed_version, "3.4.0");
    let plan = plan_managed_update(repo, &new).unwrap();
    assert_eq!(
        plan.action_for(".codex/core/SKILL.md").unwrap().disposition,
        UpdateDisposition::Identical
    );
}

#[test]
fn failed_baseline_replacement_keeps_the_previous_manifest_loadable() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let old = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];
    record_managed_baseline(repo, &old, "3.3.0").unwrap();
    let blocked_parent = repo.join(".baron/managed-state/base/codex/blocked");
    fs::write(&blocked_parent, "not a directory").unwrap();
    let replacement = vec![payload(
        "blocked/asset.md",
        ManagedMergeKind::FullText,
        "new managed asset",
    )];

    assert!(replace_managed_baseline(repo, &replacement, "3.4.0").is_err());

    let baseline = load_managed_baseline(repo)
        .expect("a failed replacement must preserve the prior valid manifest");
    assert_eq!(baseline.installed_version, "3.3.0");
}

#[test]
fn unsafe_or_duplicate_managed_paths_are_rejected() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let unsafe_path = vec![payload("../outside.md", ManagedMergeKind::FullText, "no")];
    assert!(record_managed_baseline(repo, &unsafe_path, "3.3.0").is_err());

    let duplicate = vec![
        payload("safe.md", ManagedMergeKind::FullText, "one"),
        payload("safe.md", ManagedMergeKind::FullText, "two"),
    ];
    assert!(record_managed_baseline(repo, &duplicate, "3.3.0").is_err());
}

#[test]
fn tampered_baseline_copy_is_rejected_before_it_can_be_a_merge_ancestor() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let managed = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];
    record_managed_baseline(repo, &managed, "3.3.0").unwrap();
    fs::write(
        repo.join(".baron/managed-state/base/codex/AGENTS.md"),
        "tampered baseline",
    )
    .unwrap();

    let error = plan_managed_update(repo, &managed).unwrap_err().to_string();

    assert!(error.contains("baseline hash mismatch"));
}

#[test]
fn malformed_existing_manifest_fails_closed_instead_of_being_reinitialized() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let managed = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];
    record_managed_baseline(repo, &managed, "3.3.0").unwrap();
    fs::write(repo.join(".baron/managed-state/manifest.json"), "not json").unwrap();

    let error = ensure_managed_baseline(repo, &managed, "3.4.0")
        .unwrap_err()
        .to_string();

    assert!(error.contains("malformed"));
    assert_eq!(
        fs::read_to_string(repo.join(".baron/managed-state/manifest.json")).unwrap(),
        "not json"
    );
}

#[test]
fn planner_rejects_duplicate_or_inverted_managed_markers() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let managed = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];
    record_managed_baseline(repo, &managed, "3.3.0").unwrap();
    fs::write(
        repo.join("AGENTS.md"),
        "<!-- BARON:MANAGED:START -->\none\n<!-- BARON:MANAGED:START -->\ntwo\n<!-- BARON:MANAGED:END -->\n",
    )
    .unwrap();

    let error = plan_managed_update(repo, &managed).unwrap_err().to_string();

    assert!(error.contains("malformed"));
}

#[cfg(unix)]
#[test]
fn managed_state_symlink_is_rejected_before_any_baseline_write() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    symlink(&outside, repo.join(".baron/managed-state")).unwrap();
    let managed = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];

    let error = record_managed_baseline(&repo, &managed, "3.3.0")
        .unwrap_err()
        .to_string();

    assert!(error.contains("symlink or junction"));
    assert!(!outside.join("manifest.json").exists());
}

#[cfg(windows)]
#[test]
fn managed_state_junction_is_rejected_before_any_baseline_write() {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("baron-junction-{}-{nonce}", std::process::id()));
    let repo = root.join("repo");
    let outside = root.join("outside");
    fs::create_dir_all(repo.join(".baron")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let state_link = repo.join(".baron").join("managed-state");
    let script = format!(
        "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
        state_link.display().to_string().replace('\'', "''"),
        outside.display().to_string().replace('\'', "''")
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "junction command failed: {script}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let managed = vec![payload(
        "AGENTS.md",
        ManagedMergeKind::MarkerBlock,
        BASE_MARKER,
    )];

    let error = record_managed_baseline(&repo, &managed, "3.3.0")
        .unwrap_err()
        .to_string();

    assert!(error.contains("symlink or junction"));
    assert!(!outside.join("manifest.json").exists());
    fs::remove_dir(&state_link).unwrap();
    fs::remove_dir_all(&root).unwrap();
}
