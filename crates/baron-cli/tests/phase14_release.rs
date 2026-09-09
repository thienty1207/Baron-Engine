//! Phase 14 local release inventory and source-independent smoke fixtures.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use baron_adapters::core_managed_payloads;
use baron_core::config::{load_project_config, ProjectPlatform};
use tempfile::tempdir;

fn release_binary() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    root.join("target/release")
        .join(if cfg!(windows) { "baron.exe" } else { "baron" })
}

#[test]
fn embedded_release_inventory_contains_core_and_both_bridge_contracts() {
    let payloads = core_managed_payloads().unwrap();
    assert!(payloads.iter().any(|item| {
        item.relative_path == Path::new(".baron/core/skills/superpowers/SKILL.md")
    }));
    assert!(payloads.iter().any(|item| {
        item.relative_path == Path::new(".baron/core/skills/database-engineering/SKILL.md")
    }));
    assert!(payloads.iter().any(|item| {
        item.relative_path
            == Path::new(".baron/core/skills/mobile-application-engineering/SKILL.md")
    }));
    for agent in [
        "code-reviewer.toml",
        "security-auditor.toml",
        "test-engineer.toml",
    ] {
        assert!(payloads
            .iter()
            .any(|item| { item.relative_path == Path::new(".baron/core/agents").join(agent) }));
    }
}

/// Run after `cargo build --release --workspace`; the normal workspace test
/// sweep intentionally does not require a pre-existing release binary.
#[test]
#[ignore = "run after the Phase 14 local release build"]
fn release_binary_smokes_codex_claude_and_database_without_source_tree_assets() {
    let binary = release_binary();
    assert!(
        binary.is_file(),
        "release binary is missing: {}",
        binary.display()
    );
    let temp = tempdir().unwrap();
    let repo = temp.path().join("release-smoke");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    let run = |args: &[&str]| {
        ProcessCommand::new(&binary)
            .current_dir(temp.path())
            .args(args)
            .output()
            .unwrap()
    };
    let codex = run(&[
        "init",
        repo.to_str().unwrap(),
        "--codex",
        "--vault",
        vault.to_str().unwrap(),
    ]);
    assert!(codex.status.success(), "Codex init failed: {:?}", codex);
    let claude = run(&["init", repo.to_str().unwrap(), "--claude"]);
    assert!(claude.status.success(), "Claude init failed: {:?}", claude);
    let database = run(&["init", repo.to_str().unwrap(), "--database"]);
    assert!(
        database.status.success(),
        "Database init failed: {:?}",
        database
    );

    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".agents/skills/baron-engine/SKILL.md").is_file());
    assert!(repo.join(".claude/skills/baron-engine/SKILL.md").is_file());
    assert_eq!(
        load_project_config(&repo).unwrap().platform,
        Some(ProjectPlatform::Database)
    );
}
