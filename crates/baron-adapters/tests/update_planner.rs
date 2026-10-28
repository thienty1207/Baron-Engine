use std::fs;
use std::path::{Path, PathBuf};

use baron_adapters::{
    core_managed_payloads, core_reconcile_managed_assets, ensure_managed_baseline, install_adapter,
    load_managed_baseline, managed_baseline_content, managed_payloads_for_adapter,
    plan_managed_update, reconcile_installed_managed_assets, record_managed_baseline,
    replace_managed_baseline, AgentAdapter, ManagedAssetPayload, ManagedMergeKind,
    UpdateDisposition,
};
use sha2::{Digest, Sha256};
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

const RETIRED_VIBE_PATH: &str = ".baron/core/skills/vibe-security-scan/SKILL.md";
const RETIRED_VIBE_CONTENT: &str = "# Previously managed Vibe security skill\n";

fn sha256(content: &str) -> String {
    Sha256::digest(content.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn current_core_payloads_without_vibe() -> Vec<ManagedAssetPayload> {
    core_managed_payloads()
        .unwrap()
        .into_iter()
        .filter(|item| {
            !item
                .relative_path
                .starts_with(".baron/core/skills/vibe-security-scan")
        })
        .collect()
}

fn install_legacy_vibe_baseline(repo: &Path) {
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    let mut legacy_upstream = current_core_payloads_without_vibe();
    legacy_upstream.push(payload_for(
        "core",
        RETIRED_VIBE_PATH,
        ManagedMergeKind::FullText,
        RETIRED_VIBE_CONTENT,
    ));
    let report = core_reconcile_managed_assets(repo, &legacy_upstream, "5.0.0").unwrap();
    assert!(report.conflicts.is_empty());
}

#[test]
fn unchanged_retired_vibe_asset_is_archived_and_retry_is_idempotent() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let legacy = repo.join(RETIRED_VIBE_PATH);
    assert_eq!(fs::read_to_string(&legacy).unwrap(), RETIRED_VIBE_CONTENT);

    let current = current_core_payloads_without_vibe();
    let first = core_reconcile_managed_assets(repo, &current, "5.0.1").unwrap();

    assert!(
        !legacy.exists(),
        "the active legacy runtime path must be retired"
    );
    assert!(first.conflicts.is_empty());
    assert!(repo
        .join(".baron/core/skills/security-audit/SKILL.md")
        .is_file());
    assert_eq!(
        fs::read_to_string(repo.join(".baron/managed-state/retired/vibe-security-scan/SKILL.md"))
            .unwrap(),
        RETIRED_VIBE_CONTENT
    );
    assert!(repo
        .join(".baron/managed-state/retired-vibe-security.json")
        .is_file());
    let baseline = load_managed_baseline(repo).unwrap();
    assert!(!baseline
        .records
        .iter()
        .any(|record| record.relative_path == Path::new(RETIRED_VIBE_PATH)));

    let archive_before =
        fs::read(repo.join(".baron/managed-state/retired/vibe-security-scan/SKILL.md")).unwrap();
    let second = core_reconcile_managed_assets(repo, &current, "5.0.1").unwrap();
    assert!(second.conflicts.is_empty());
    assert!(!legacy.exists());
    assert_eq!(
        fs::read(repo.join(".baron/managed-state/retired/vibe-security-scan/SKILL.md")).unwrap(),
        archive_before
    );
}

#[test]
fn modified_retired_vibe_asset_is_preserved_with_review_conflict() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let legacy = repo.join(RETIRED_VIBE_PATH);
    let modified = "# User-modified legacy security guidance\n";
    fs::write(&legacy, modified).unwrap();
    let custom = repo.join(".codex/skills/vibe-security-scan/SKILL.md");
    fs::create_dir_all(custom.parent().unwrap()).unwrap();
    fs::write(&custom, "user-owned skill\n").unwrap();
    let claude_custom = repo.join(".claude/skills/vibe-security-scan/SKILL.md");
    fs::create_dir_all(claude_custom.parent().unwrap()).unwrap();
    fs::write(&claude_custom, "user-owned Claude skill\n").unwrap();

    let current = current_core_payloads_without_vibe();
    let report = core_reconcile_managed_assets(repo, &current, "5.0.1").unwrap();

    assert_eq!(fs::read_to_string(&legacy).unwrap(), modified);
    assert!(report
        .conflicts
        .iter()
        .any(|path| path == Path::new(RETIRED_VIBE_PATH)));
    assert_eq!(fs::read_to_string(&custom).unwrap(), "user-owned skill\n");
    assert!(repo
        .join(".baron/core/skills/security-audit/SKILL.md")
        .is_file());
    let baseline = load_managed_baseline(repo).unwrap();
    assert!(baseline
        .records
        .iter()
        .any(|record| record.relative_path == Path::new(RETIRED_VIBE_PATH)));

    let claude_install = install_adapter(repo, AgentAdapter::Claude).unwrap();
    assert!(claude_install
        .core
        .conflicts
        .iter()
        .any(|path| { path == ".baron/core/skills/vibe-security-scan/SKILL.md" }));
    let claude_index = fs::read_to_string(repo.join(".claude/skills/INDEX.md")).unwrap();
    assert!(claude_index.contains("security-audit"));
    assert!(!claude_index.contains("vibe-security-scan"));
    assert_eq!(fs::read_to_string(&custom).unwrap(), "user-owned skill\n");
    assert_eq!(
        fs::read_to_string(&claude_custom).unwrap(),
        "user-owned Claude skill\n"
    );
}

#[test]
fn retired_vibe_file_without_baseline_is_preserved_and_reported() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    let unknown_path = ".baron/core/skills/vibe-security-scan/local.md";
    let unknown = repo.join(unknown_path);
    fs::create_dir_all(unknown.parent().unwrap()).unwrap();
    fs::write(&unknown, "untracked local file\n").unwrap();

    let report =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
            .unwrap();

    assert_eq!(
        fs::read_to_string(&unknown).unwrap(),
        "untracked local file\n"
    );
    assert!(report
        .conflicts
        .iter()
        .any(|path| path == Path::new(unknown_path)));
    assert!(repo
        .join(".baron/core/skills/security-audit/SKILL.md")
        .is_file());
}

#[test]
fn first_install_reports_and_preserves_unmanaged_retired_vibe_asset() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let unknown_path = ".baron/core/skills/vibe-security-scan/local.md";
    let unknown = repo.join(unknown_path);
    fs::create_dir_all(unknown.parent().unwrap()).unwrap();
    fs::write(&unknown, "unmanaged legacy data\n").unwrap();

    let report = install_adapter(repo, AgentAdapter::Codex).unwrap();

    assert_eq!(
        fs::read_to_string(&unknown).unwrap(),
        "unmanaged legacy data\n"
    );
    assert!(report
        .core
        .conflicts
        .iter()
        .any(|path| path == unknown_path));
    assert!(repo
        .join(".baron/core/skills/security-audit/SKILL.md")
        .is_file());
    let index = fs::read_to_string(repo.join(".codex/INDEX.md")).unwrap();
    assert!(index.contains("security-audit"));
    assert!(!index.contains("vibe-security-scan"));
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(repo.join(".baron/managed-state/retired-vibe-security.json")).unwrap(),
    )
    .unwrap();
    assert!(receipt["review_required_paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == unknown_path));
}

#[test]
fn resolved_unmanaged_vibe_conflict_clears_review_receipt() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let unknown_path = ".baron/core/skills/vibe-security-scan/local.md";
    let unknown = repo.join(unknown_path);
    fs::create_dir_all(unknown.parent().unwrap()).unwrap();
    fs::write(&unknown, "unmanaged legacy data\n").unwrap();
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    fs::remove_file(&unknown).unwrap();

    let report =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
            .unwrap();

    assert!(report.conflicts.is_empty());
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(repo.join(".baron/managed-state/retired-vibe-security.json")).unwrap(),
    )
    .unwrap();
    assert!(receipt["review_required_paths"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn new_adapter_route_is_not_published_when_security_audit_asset_conflicts() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    let skill = repo.join(".baron/core/skills/security-audit/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(&skill, "user-owned conflicting skill\n").unwrap();

    let error = install_adapter(repo, AgentAdapter::Codex).unwrap_err();

    assert!(error.to_string().contains("security-audit"));
    assert_eq!(
        fs::read_to_string(&skill).unwrap(),
        "user-owned conflicting skill\n"
    );
    assert!(!repo.join(".codex/INDEX.md").exists());
    assert!(!repo.join("AGENTS.md").exists());
}

#[test]
fn retry_recovers_after_legacy_file_was_archived_before_baseline_publication() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let legacy = repo.join(RETIRED_VIBE_PATH);
    let archived = repo.join(".baron/managed-state/retired/vibe-security-scan/SKILL.md");
    fs::create_dir_all(archived.parent().unwrap()).unwrap();
    fs::write(&archived, RETIRED_VIBE_CONTENT).unwrap();
    fs::remove_file(&legacy).unwrap();
    let receipt_path = repo.join(".baron/managed-state/retired-vibe-security.json");
    fs::write(
        &receipt_path,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "legacy_skill_id": "vibe-security-scan",
            "replacement_skill_id": "security-audit",
            "retired_assets": [],
            "pending_assets": [{
                "original_path": RETIRED_VIBE_PATH,
                "archived_path": ".baron/managed-state/retired/vibe-security-scan/SKILL.md",
                "sha256": sha256(RETIRED_VIBE_CONTENT)
            }],
            "review_required_paths": []
        }))
        .unwrap(),
    )
    .unwrap();

    let report =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
            .unwrap();

    assert!(report.conflicts.is_empty());
    assert!(!legacy.exists());
    assert_eq!(fs::read_to_string(&archived).unwrap(), RETIRED_VIBE_CONTENT);
    let baseline = load_managed_baseline(repo).unwrap();
    assert!(!baseline
        .records
        .iter()
        .any(|record| record.relative_path == Path::new(RETIRED_VIBE_PATH)));
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    assert!(receipt["retired_assets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["original_path"] == RETIRED_VIBE_PATH));
    assert!(receipt["pending_assets"].as_array().unwrap().is_empty());
}

#[test]
fn malformed_retirement_receipt_cannot_suppress_accurate_archive_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let receipt = serde_json::json!({
        "schema_version": 1,
        "legacy_skill_id": "vibe-security-scan",
        "replacement_skill_id": "security-audit",
        "retired_assets": [{
            "original_path": RETIRED_VIBE_PATH,
            "archived_path": "../../attacker-controlled.txt",
            "sha256": sha256(RETIRED_VIBE_CONTENT)
        }],
        "pending_assets": [],
        "review_required_paths": []
    });
    fs::write(
        repo.join(".baron/managed-state/retired-vibe-security.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    let legacy = repo.join(RETIRED_VIBE_PATH);

    let result =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1");

    assert!(result.is_err());
    assert_eq!(fs::read_to_string(&legacy).unwrap(), RETIRED_VIBE_CONTENT);
    assert!(load_managed_baseline(repo)
        .unwrap()
        .records
        .iter()
        .any(|record| record.relative_path == Path::new(RETIRED_VIBE_PATH)));
}

#[test]
fn archive_collision_preserves_both_legacy_and_archive_bytes() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let legacy = repo.join(RETIRED_VIBE_PATH);
    let archive = repo.join(".baron/managed-state/retired/vibe-security-scan/SKILL.md");
    fs::create_dir_all(archive.parent().unwrap()).unwrap();
    fs::write(&archive, "unrelated archive data\n").unwrap();

    let report =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
            .unwrap();

    assert!(report
        .conflicts
        .iter()
        .any(|path| path == Path::new(RETIRED_VIBE_PATH)));
    assert_eq!(fs::read_to_string(&legacy).unwrap(), RETIRED_VIBE_CONTENT);
    assert_eq!(
        fs::read_to_string(&archive).unwrap(),
        "unrelated archive data\n"
    );
    assert!(load_managed_baseline(repo)
        .unwrap()
        .records
        .iter()
        .any(|record| record.relative_path == Path::new(RETIRED_VIBE_PATH)));
}

#[test]
fn restored_legacy_source_completes_an_already_archived_review_case() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let legacy = repo.join(RETIRED_VIBE_PATH);
    let archive = repo.join(".baron/managed-state/retired/vibe-security-scan/SKILL.md");
    fs::create_dir_all(archive.parent().unwrap()).unwrap();
    fs::write(&archive, RETIRED_VIBE_CONTENT).unwrap();
    fs::write(&legacy, "locally modified legacy asset\n").unwrap();

    let first = core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
        .unwrap();
    assert!(first
        .conflicts
        .iter()
        .any(|path| path == Path::new(RETIRED_VIBE_PATH)));
    fs::write(&legacy, RETIRED_VIBE_CONTENT).unwrap();

    let second =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
            .unwrap();

    assert!(second.conflicts.is_empty());
    assert!(!legacy.exists());
    assert_eq!(fs::read_to_string(&archive).unwrap(), RETIRED_VIBE_CONTENT);
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(repo.join(".baron/managed-state/retired-vibe-security.json")).unwrap(),
    )
    .unwrap();
    assert!(receipt["pending_assets"].as_array().unwrap().is_empty());
    assert!(receipt["review_required_paths"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn missing_retired_vibe_file_keeps_baseline_content_recoverable() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_legacy_vibe_baseline(repo);
    let legacy = repo.join(RETIRED_VIBE_PATH);
    fs::remove_file(&legacy).unwrap();

    let report =
        core_reconcile_managed_assets(repo, &current_core_payloads_without_vibe(), "5.0.1")
            .unwrap();

    assert!(report
        .conflicts
        .iter()
        .any(|path| path == Path::new(RETIRED_VIBE_PATH)));
    let baseline = load_managed_baseline(repo).unwrap();
    let record = baseline
        .records
        .iter()
        .find(|record| record.relative_path == Path::new(RETIRED_VIBE_PATH))
        .expect("missing legacy target must retain its managed baseline record");
    assert_eq!(
        managed_baseline_content(repo, record).unwrap(),
        RETIRED_VIBE_CONTENT
    );
    assert!(!repo
        .join(".baron/managed-state/retired/vibe-security-scan/SKILL.md")
        .exists());
    let receipt: serde_json::Value = serde_json::from_slice(
        &fs::read(repo.join(".baron/managed-state/retired-vibe-security.json")).unwrap(),
    )
    .unwrap();
    assert!(receipt["review_required_paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == RETIRED_VIBE_PATH));
}

#[test]
fn local_reconcile_restores_only_a_missing_embedded_managed_asset() {
    let temp = tempdir().unwrap();
    let repo = temp.path();
    install_adapter(repo, AgentAdapter::Codex).unwrap();
    let missing = repo.join(".baron/core/skills/superpowers/SKILL.md");
    fs::remove_file(&missing).unwrap();
    let custom = repo.join(".codex/skills/custom/SKILL.md");
    fs::create_dir_all(custom.parent().unwrap()).unwrap();
    fs::write(&custom, "custom user skill").unwrap();

    let report =
        core_reconcile_managed_assets(repo, &core_managed_payloads().unwrap(), "3.3.0").unwrap();

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
    assert_eq!(baseline.schema_version, 2);
    assert_eq!(baseline.minimum_writer_schema, Some(2));
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

    let mut payloads = core_managed_payloads().unwrap();
    payloads.extend(managed_payloads_for_adapter(AgentAdapter::Codex).unwrap());
    let plan = plan_managed_update(repo, &payloads).unwrap();
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

    // The preview is intentionally bounded, so it may stop before any
    // particular unlisted user file depending on filesystem enumeration
    // order. User-path preservation itself is covered by the preceding
    // planner test; this case only asserts the bound and dependency filter.
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
