use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use baron_adapters::{
    core_managed_payloads, core_reconcile_managed_assets, install_adapter, load_managed_baseline,
    managed_payloads_for_adapter, AgentAdapter, ManagedOwner,
};
use tempfile::tempdir;

fn collect_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        if !current.exists() {
            return;
        }
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(relative, fs::read(path).unwrap());
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn core_payloads_are_canonical_and_include_nested_resources() {
    let payloads = core_managed_payloads().unwrap();
    assert!(!payloads.is_empty());
    assert!(payloads.iter().all(|payload| {
        payload.adapter == "core"
            && payload
                .relative_path
                .to_string_lossy()
                .replace('\\', "/")
                .starts_with(".baron/core/")
    }));
    assert!(payloads.iter().any(|payload| {
        payload.relative_path
            == Path::new(
                ".baron/core/skills/api-and-interface-design/references/deep-module-boundaries.md",
            )
    }));
    assert!(payloads.iter().any(|payload| {
        payload.relative_path
            == Path::new(".baron/core/skills/superpowers/brainstorming/scripts/helper.js")
    }));
}

#[test]
fn adapter_payloads_contain_only_integration_surfaces() {
    for adapter in [AgentAdapter::Codex, AgentAdapter::Claude] {
        let payloads = managed_payloads_for_adapter(adapter).unwrap();
        assert!(payloads.iter().all(|payload| {
            let path = payload.relative_path.to_string_lossy().replace('\\', "/");
            let codex_wrapper = matches!(
                path.as_str(),
                ".codex/agents/code-reviewer.toml"
                    | ".codex/agents/security-auditor.toml"
                    | ".codex/agents/test-engineer.toml"
            );
            let claude_bridge = path == ".claude/skills/baron-engine/SKILL.md";
            let claude_wrapper = matches!(
                path.as_str(),
                ".claude/agents/code-reviewer.md"
                    | ".claude/agents/security-auditor.md"
                    | ".claude/agents/test-engineer.md"
            );
            (!path.starts_with(".codex/skills/") || path.ends_with("/INDEX.md"))
                && (!path.starts_with(".codex/agents/")
                    || path.ends_with("/INDEX.md")
                    || codex_wrapper)
                && (!path.starts_with(".claude/skills/")
                    || path.ends_with("/INDEX.md")
                    || claude_bridge)
                && (!path.starts_with(".claude/agents/")
                    || path.ends_with("/INDEX.md")
                    || claude_wrapper)
                && !path.starts_with(".baron/core/")
        }));
    }
}

#[test]
fn fresh_codex_and_claude_install_share_one_core_and_owner_baseline() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    let core = temp.path().join(".baron/core");
    assert!(core.join("skills/superpowers/SKILL.md").is_file());
    assert!(core
        .join("skills/api-and-interface-design/references/deep-module-boundaries.md")
        .is_file());
    assert!(core.join("skills/database-engineering/SKILL.md").is_file());
    assert!(core
        .join("skills/mobile-application-engineering/SKILL.md")
        .is_file());
    assert!(core.join("agents/code-reviewer.toml").is_file());
    assert!(!temp
        .path()
        .join(".codex/skills/superpowers/SKILL.md")
        .exists());
    assert!(!temp
        .path()
        .join(".claude/skills/superpowers/SKILL.md")
        .exists());
    assert!(temp
        .path()
        .join(".codex/agents/code-reviewer.toml")
        .is_file());
    assert!(temp
        .path()
        .join(".claude/agents/code-reviewer.md")
        .is_file());

    let baseline = load_managed_baseline(temp.path()).unwrap();
    let core_paths = baseline
        .records
        .iter()
        .filter(|record| record.relative_path.starts_with(".baron/core"))
        .collect::<Vec<_>>();
    assert!(!core_paths.is_empty());
    assert!(core_paths
        .iter()
        .all(|record| matches!(record.owner, ManagedOwner::Core)));
}

#[test]
fn installation_order_does_not_change_canonical_core_bytes() {
    let orders = [
        [AgentAdapter::Codex, AgentAdapter::Claude],
        [AgentAdapter::Claude, AgentAdapter::Codex],
    ];
    let mut snapshots = Vec::new();
    for order in orders {
        let temp = tempdir().unwrap();
        for adapter in order {
            install_adapter(temp.path(), adapter).unwrap();
        }
        snapshots.push(collect_files(&temp.path().join(".baron/core")));
    }
    assert_eq!(snapshots[0], snapshots[1]);
}

#[test]
fn legacy_generic_then_codex_and_claude_orders_share_the_same_core() {
    let orders = [
        vec![AgentAdapter::Codex],
        vec![AgentAdapter::Claude],
        vec![AgentAdapter::Codex, AgentAdapter::Claude],
        vec![AgentAdapter::Claude, AgentAdapter::Codex],
    ];
    for order in orders {
        let temp = tempdir().unwrap();
        for adapter in order {
            install_adapter(temp.path(), adapter).unwrap();
        }
        assert!(temp
            .path()
            .join(".baron/core/skills/superpowers/SKILL.md")
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
}

#[test]
fn unchanged_core_reinstall_is_a_no_op_and_a_simulated_upstream_change_updates_it() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let unchanged = install_adapter(temp.path(), AgentAdapter::Claude).unwrap();
    assert!(unchanged.core.changed_paths.is_empty());
    assert!(unchanged.core.conflicts.is_empty());

    let mut upstream = core_managed_payloads().unwrap();
    let target = upstream
        .iter_mut()
        .find(|payload| {
            payload.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
        })
        .unwrap();
    target
        .content
        .push_str("\n# Phase 4 simulated upstream revision\n");
    let report =
        core_reconcile_managed_assets(temp.path(), &upstream, env!("CARGO_PKG_VERSION")).unwrap();
    assert!(report
        .applied_paths
        .iter()
        .any(|path| path.ends_with(".baron/core/skills/superpowers/SKILL.md")));
    assert!(
        fs::read_to_string(temp.path().join(".baron/core/skills/superpowers/SKILL.md"))
            .unwrap()
            .contains("simulated upstream revision")
    );
}

#[test]
fn unknown_core_collision_is_preserved_without_a_core_claim() {
    let temp = tempdir().unwrap();
    let target = temp.path().join(".baron/core/skills/superpowers/SKILL.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"# ambiguous local skill\n").unwrap();

    let report = install_adapter(temp.path(), AgentAdapter::Codex).unwrap();

    assert_eq!(fs::read(&target).unwrap(), b"# ambiguous local skill\n");
    assert!(report
        .core
        .conflicts
        .iter()
        .any(|path| path == ".baron/core/skills/superpowers/SKILL.md"));
    let baseline = load_managed_baseline(temp.path()).unwrap();
    assert!(!baseline.records.iter().any(|record| {
        record.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
    }));
}

#[test]
fn core_contains_three_mandatory_quality_agents_and_one_workflow_owner() {
    let payloads = core_managed_payloads().unwrap();
    let agents = payloads
        .iter()
        .filter_map(|payload| {
            let path = payload.relative_path.to_string_lossy().replace('\\', "/");
            path.strip_prefix(".baron/core/agents/").map(str::to_owned)
        })
        .collect::<BTreeSet<_>>();
    for required in [
        "code-reviewer.toml",
        "security-auditor.toml",
        "test-engineer.toml",
    ] {
        assert!(agents.contains(required));
    }
    let superpowers = payloads
        .iter()
        .find(|payload| {
            payload.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
        })
        .unwrap();
    assert!(superpowers.content.contains("workflow"));
}

#[test]
fn modified_core_asset_is_preserved_and_reported() {
    let temp = tempdir().unwrap();
    install_adapter(temp.path(), AgentAdapter::Codex).unwrap();
    let target = temp.path().join(".baron/core/skills/superpowers/SKILL.md");
    fs::write(&target, b"# user modification\n").unwrap();

    let report = install_adapter(temp.path(), AgentAdapter::Claude).unwrap();

    assert_eq!(fs::read(&target).unwrap(), b"# user modification\n");
    assert!(report
        .core
        .conflicts
        .iter()
        .any(|path| path.ends_with(".baron/core/skills/superpowers/SKILL.md")));
}
