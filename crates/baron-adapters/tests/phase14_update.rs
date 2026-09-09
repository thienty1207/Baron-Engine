//! Phase 14 managed-state and Core/projection update fixtures.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use baron_adapters::{
    core_managed_payloads, core_reconcile_managed_assets, install_adapter, load_managed_baseline,
    managed_payloads_for_adapter, migrate_managed_ownership, plan_managed_update,
    replace_managed_baseline, AgentAdapter, ManagedAssetPayload, ManagedMergeKind, ManagedOwner,
};
use sha2::Digest;
use tempfile::tempdir;

fn payload(adapter: &str, path: &str, content: &str) -> ManagedAssetPayload {
    ManagedAssetPayload {
        adapter: adapter.to_string(),
        relative_path: PathBuf::from(path),
        merge_kind: ManagedMergeKind::FullText,
        content: content.to_string(),
    }
}

fn combined_payloads() -> Vec<ManagedAssetPayload> {
    let mut payloads = core_managed_payloads().unwrap();
    for adapter in [AgentAdapter::Codex, AgentAdapter::Claude] {
        for candidate in managed_payloads_for_adapter(adapter).unwrap() {
            if payloads
                .iter()
                .all(|existing| existing.relative_path != candidate.relative_path)
            {
                payloads.push(candidate);
            }
        }
    }
    payloads
}

#[test]
fn future_managed_writer_refuses_mutation_and_preserves_bytes() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("future-managed");
    let manifest = repo.join(".baron/managed-state/manifest.json");
    fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    let future = r#"{
  "schema_version": 2,
  "installed_version": "9.9.9",
  "minimum_writer_schema": 3,
  "records": []
}
"#;
    fs::write(&manifest, future).unwrap();
    let before = fs::read(&manifest).unwrap();

    let error =
        replace_managed_baseline(&repo, &[payload("codex", "AGENTS.md", "managed")], "4.2.2")
            .unwrap_err()
            .to_string();

    assert!(error.contains("minimum writer schema"));
    assert_eq!(fs::read(&manifest).unwrap(), before);
}

#[test]
fn current_update_payloads_have_one_core_owner_and_thin_adapter_projections() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    let payloads = combined_payloads();
    let unique_paths = payloads
        .iter()
        .map(|item| item.relative_path.clone())
        .collect::<HashSet<_>>();
    assert_eq!(unique_paths.len(), payloads.len());
    let core_paths = payloads
        .iter()
        .filter(|item| item.adapter == "core")
        .map(|item| item.relative_path.clone())
        .collect::<HashSet<_>>();
    assert!(!core_paths.is_empty());
    assert!(payloads.iter().any(|item| item.adapter == "codex"));
    assert!(payloads.iter().any(|item| item.adapter == "claude"));

    let plan = plan_managed_update(temp.path(), &payloads).unwrap();
    let plan_paths = plan
        .actions
        .iter()
        .map(|item| item.relative_path.clone())
        .collect::<HashSet<_>>();
    assert_eq!(plan_paths.len(), plan.actions.len());
    let baseline = load_managed_baseline(temp.path()).unwrap();
    assert!(
        baseline
            .records
            .iter()
            .filter(|item| matches!(item.owner, ManagedOwner::Core))
            .count()
            >= core_paths.len()
    );
}

#[test]
fn modified_core_update_reports_conflict_without_changing_core_or_baseline() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let target = temp.path().join(".baron/core/skills/superpowers/SKILL.md");
    let modified = format!(
        "{}\nuser modification\n",
        fs::read_to_string(&target).unwrap()
    );
    fs::write(&target, &modified).unwrap();
    let before_target = fs::read(&target).unwrap();
    let before_manifest = fs::read(temp.path().join(".baron/managed-state/manifest.json")).unwrap();

    let report =
        core_reconcile_managed_assets(temp.path(), &core_managed_payloads().unwrap(), "4.2.2")
            .unwrap();

    assert!(report
        .conflicts
        .iter()
        .any(|path| path.ends_with("superpowers/SKILL.md")));
    assert_eq!(fs::read(&target).unwrap(), before_target);
    assert_eq!(
        fs::read(temp.path().join(".baron/managed-state/manifest.json")).unwrap(),
        before_manifest
    );
}

#[test]
fn v1_migration_is_deterministic_and_repeated_retry_is_a_no_op() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-managed");
    let source = repo.join(".codex/skills/legacy/SKILL.md");
    let baseline = repo.join(".baron/managed-state/base/codex/.codex/skills/legacy/SKILL.md");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::create_dir_all(baseline.parent().unwrap()).unwrap();
    let content = "legacy skill\n";
    fs::write(&source, content).unwrap();
    fs::write(&baseline, content).unwrap();
    let hash = format!("{:x}", sha2::Sha256::digest(content.as_bytes()));
    fs::write(
        repo.join(".baron/managed-state/manifest.json"),
        format!(
            "{{\n  \"schema_version\": 1,\n  \"installed_version\": \"4.2.2\",\n  \"records\": [{{\"adapter\":\"codex\",\"relative_path\":\".codex/skills/legacy/SKILL.md\",\"base_sha256\":\"{hash}\",\"installed_version\":\"4.2.2\",\"merge_kind\":\"full_text\"}}]\n}}\n"
        ),
    )
    .unwrap();

    let first = migrate_managed_ownership(&repo).unwrap();
    let manifest = repo.join(".baron/managed-state/manifest.json");
    let after_first = fs::read(&manifest).unwrap();
    let second = migrate_managed_ownership(&repo).unwrap();

    assert_eq!(first.published_schema, Some(2));
    assert!(repo.join(".baron/core/skills/legacy/SKILL.md").is_file());
    assert!(!source.exists());
    assert_eq!(second.published_schema, Some(2));
    assert_eq!(fs::read(manifest).unwrap(), after_first);
}
