use std::fs;
use std::path::{Path, PathBuf};

use baron_core::migration::{
    execute_agent_bootstrap_migration, inventory_agent_bootstrap, migration_status,
    rollback_migration, MigrationAction, MigrationAssetKind,
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn visit(root: &Path, current: &Path, output: &mut Vec<(String, Vec<u8>)>) {
        if !current.exists() {
            return;
        }
        for entry in fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &path, output);
            } else {
                output.push((
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    fs::read(&path).unwrap(),
                ));
            }
        }
    }
    let mut output = Vec::new();
    visit(root, root, &mut output);
    output.sort_by(|left, right| left.0.cmp(&right.0));
    output
}

fn legacy_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let legacy_capsule = vault.join("Projects/legacy-demo");
    fs::create_dir_all(&repo).unwrap();

    write(
        &repo.join("vault.config.json"),
        &serde_json::to_string_pretty(&serde_json::json!({
            "vault_root": vault,
            "project_slug": "legacy-demo",
            "project_root": legacy_capsule,
            "project_type": "fullstack",
            "runtime_script": "scripts/agent-memory.js",
            "hooks_path": ".githooks"
        }))
        .unwrap(),
    );
    write(
        &repo.join(".agent-bootstrap-manifest.json"),
        "{\n  \"version\": 1,\n  \"entries\": {}\n}\n",
    );
    write(
        &repo.join("AGENTS.md"),
        "# User Rules\n\nKeep this.\n\n<!-- agent-bootstrap:start -->\nlegacy runtime instructions\n<!-- agent-bootstrap:end -->\n",
    );
    write(
        &repo.join("scripts/agent-memory.js"),
        "// agent-bootstrap generated runtime\nconsole.log('legacy');\n",
    );
    write(
        &repo.join(".githooks/post-commit"),
        "#!/bin/sh\nnode scripts/agent-memory.js compact\n",
    );
    write(
        &repo.join("docs/superpowers/plans/2026-05-21/2026-05-21-auth.md"),
        "# Auth Plan\n\n- verified old plan\n",
    );
    write(
        &repo.join("docs/product/PRODUCT.md"),
        "# Product\n\nLegacy product contract.\n",
    );
    write(
        &repo.join("docs/product/traces/legacy-trace.md"),
        "# Legacy Trace\n\n- verified auth implementation path\n",
    );
    write(
        &repo.join("docs/product/proofs/legacy-proof.md"),
        "# Legacy Proof\n\n- auth test passed\n",
    );
    write(
        &repo.join(".codex/skills/rust-api/SKILL.md"),
        "---\nname: rust-api\ndescription: Use when implementing Rust API endpoints.\n---\n\n# Rust API\n\nUse Superpowers for workflow. Require test evidence.\n",
    );
    write(
        &repo.join(".codex/agents/backend-development.toml"),
        "name = \"backend-development\"\ndescription = \"Use when reviewing backend implementation.\"\ndeveloper_instructions = \"Require evidence. Do not orchestrate other agents.\"\n",
    );
    write(
        &repo.join(".codex/agents/unsafe-agent.toml"),
        "name = \"unsafe-agent\"\ndescription = \"Legacy helper\"\ndeveloper_instructions = \"Run agent-bootstrap update and orchestrate subagents.\"\n",
    );
    write(
        &legacy_capsule.join("Facts.md"),
        "# Facts\n\n- Legacy API uses Rust.\n",
    );
    write(
        &legacy_capsule.join("Decisions.md"),
        "# Decisions\n\n- Keep PostgreSQL.\n",
    );
    write(
        &legacy_capsule.join("Research/backend.md"),
        "# Backend Research\n\n- Axum remains a candidate.\n",
    );
    write(
        &legacy_capsule.join("Sessions/session-1.md"),
        "# Session\n\n- Auth work was interrupted.\n",
    );

    (temp, repo, vault)
}

#[test]
fn inventory_is_read_only_and_classifies_legacy_assets() {
    let (_temp, repo, vault) = legacy_fixture();
    let repo_before = snapshot(&repo);
    let vault_before = snapshot(&vault);

    let inventory = inventory_agent_bootstrap(&repo, None).unwrap();

    assert_eq!(inventory.project_slug, "legacy-demo");
    assert_eq!(inventory.source_vault, vault);
    assert!(inventory.items.iter().any(|item| {
        item.relative_path == "scripts/agent-memory.js"
            && item.kind == MigrationAssetKind::LegacyRuntime
            && item.action == MigrationAction::Remove
    }));
    assert!(inventory.items.iter().any(|item| {
        item.relative_path == ".codex/skills/rust-api"
            && item.kind == MigrationAssetKind::CustomSkill
            && item.action == MigrationAction::Import
    }));
    assert!(inventory.items.iter().any(|item| {
        item.relative_path == ".codex/agents/unsafe-agent.toml"
            && item.action == MigrationAction::Quarantine
    }));
    assert_eq!(snapshot(&repo), repo_before);
    assert_eq!(snapshot(&vault), vault_before);
}

#[test]
fn migration_imports_data_quarantines_invalid_assets_and_retires_runtime() {
    let (_temp, repo, vault) = legacy_fixture();

    let receipt = execute_agent_bootstrap_migration(&repo, None, |repo, _vault| {
        write(
            &repo.join(".baron/project.toml"),
            "schema_version = 1\nproject_slug = \"demo\"\nadapters = [\"codex\"]\n",
        );
        write(
            &repo.join("AGENTS.md"),
            "# User Rules\n\nKeep this.\n\n<!-- BARON:MANAGED:START -->\nBaron native\n<!-- BARON:MANAGED:END -->\n",
        );
        Ok(())
    })
    .unwrap();

    assert_eq!(receipt.status, "completed");
    assert!(receipt.imported_count >= 4);
    assert!(repo
        .join("docs/baron/plans/2026-05-21/2026-05-21-auth.md")
        .exists());
    assert!(repo.join("docs/baron/harness/product/PRODUCT.md").exists());
    assert!(repo.join("docs/baron/traces/legacy-trace.md").exists());
    assert!(repo.join("docs/baron/proofs/legacy-proof.md").exists());
    assert!(vault.join("Projects/demo/Facts.md").exists());
    assert!(vault.join("Projects/demo/Research/backend.md").exists());
    assert!(repo.join(".codex/skills/rust-api/SKILL.md").exists());
    assert!(!repo.join(".codex/agents/unsafe-agent.toml").exists());
    assert!(repo
        .join(".baron/quarantine")
        .join(&receipt.migration_id)
        .join(".codex/agents/unsafe-agent.toml")
        .exists());
    assert!(!repo.join("vault.config.json").exists());
    assert!(!repo.join(".agent-bootstrap-manifest.json").exists());
    assert!(!repo.join("scripts/agent-memory.js").exists());
    assert!(!repo.join(".githooks/post-commit").exists());
    assert!(fs::read_to_string(repo.join("AGENTS.md"))
        .unwrap()
        .contains("BARON:MANAGED:START"));
    assert!(receipt.backup_root.join("manifest.json").exists());
    assert!(receipt.backup_root.join("receipt.json").exists());
}

#[test]
fn rollback_restores_legacy_paths_without_touching_unrelated_files() {
    let (_temp, repo, vault) = legacy_fixture();
    let receipt = execute_agent_bootstrap_migration(&repo, None, |repo, _vault| {
        write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
        Ok(())
    })
    .unwrap();
    write(&repo.join("after-migration.txt"), "keep me\n");
    write(
        &repo.join("docs/baron/plans/post-migration-plan.md"),
        "# New Baron Plan\n",
    );
    write(
        &vault.join("Projects/demo/post-migration-memory.md"),
        "# New Baron Memory\n",
    );

    let report = rollback_migration(&repo, &vault, &receipt.migration_id).unwrap();

    assert_eq!(report.status, "rolled_back");
    assert!(repo.join("vault.config.json").exists());
    assert!(repo.join("scripts/agent-memory.js").exists());
    assert!(repo.join(".codex/agents/unsafe-agent.toml").exists());
    assert!(!repo.join(".baron/project.toml").exists());
    assert!(!repo
        .join(".baron/quarantine")
        .join(&receipt.migration_id)
        .exists());
    assert_eq!(
        fs::read_to_string(repo.join("after-migration.txt")).unwrap(),
        "keep me\n"
    );
    assert!(repo
        .join("docs/baron/plans/post-migration-plan.md")
        .exists());
    assert!(vault
        .join("Projects/demo/post-migration-memory.md")
        .exists());
    assert!(migration_status(&repo).unwrap().contains("rolled_back"));
}

#[test]
fn modified_legacy_runtime_is_quarantined_instead_of_deleted() {
    let (_temp, repo, _vault) = legacy_fixture();
    write(
        &repo.join("scripts/agent-memory.js"),
        "// user-customized bridge that must be preserved\n",
    );

    let inventory = inventory_agent_bootstrap(&repo, None).unwrap();
    assert!(inventory.items.iter().any(|item| {
        item.relative_path == "scripts/agent-memory.js"
            && item.action == MigrationAction::Quarantine
    }));

    let receipt = execute_agent_bootstrap_migration(&repo, None, |repo, _vault| {
        write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
        Ok(())
    })
    .unwrap();

    assert!(!repo.join("scripts/agent-memory.js").exists());
    assert_eq!(
        fs::read_to_string(
            repo.join(".baron/quarantine")
                .join(receipt.migration_id)
                .join("scripts/agent-memory.js")
        )
        .unwrap(),
        "// user-customized bridge that must be preserved\n"
    );
}

#[test]
fn failed_install_rolls_back_automatically() {
    let (_temp, repo, vault) = legacy_fixture();

    let result = execute_agent_bootstrap_migration(&repo, None, |repo, _vault| {
        write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
        anyhow::bail!("injected install failure")
    });

    assert!(result.is_err());
    assert!(repo.join("vault.config.json").exists());
    assert!(repo.join("scripts/agent-memory.js").exists());
    assert!(!repo.join(".baron/project.toml").exists());
    let migrations = vault.join("Artifacts/Baron/Migrations");
    let failure_exists = fs::read_dir(migrations)
        .unwrap()
        .any(|entry| entry.unwrap().path().join("failure.json").exists());
    assert!(failure_exists);
    assert!(migration_status(&repo).unwrap().contains("rolled_back"));
}

#[test]
fn explicit_vault_is_destination_while_legacy_config_remains_the_source() {
    let (temp, repo, source_vault) = legacy_fixture();
    let destination_vault = temp.path().join("BaronVault");

    let receipt =
        execute_agent_bootstrap_migration(&repo, Some(&destination_vault), |repo, _vault| {
            write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
            Ok(())
        })
        .unwrap();

    assert_eq!(receipt.source_vault, source_vault);
    assert_eq!(receipt.destination_vault, destination_vault);
    assert!(receipt
        .destination_vault
        .join("Projects/demo/Facts.md")
        .exists());
    assert!(receipt
        .backup_root
        .join("source-vault/legacy-demo/Facts.md")
        .exists());
}

#[test]
fn manifest_paths_cannot_escape_the_repo() {
    let (temp, repo, _vault) = legacy_fixture();
    let outside = temp.path().join("outside.txt");
    write(&outside, "must survive\n");
    let hash = format!("{:x}", Sha256::digest(fs::read(&outside).unwrap()));
    write(
        &repo.join(".agent-bootstrap-manifest.json"),
        &serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "entries": {
                "../outside.txt": {
                    "syncedHash": hash,
                    "status": "managed"
                }
            }
        }))
        .unwrap(),
    );

    let inventory = inventory_agent_bootstrap(&repo, None).unwrap();
    assert!(!inventory
        .items
        .iter()
        .any(|item| item.relative_path == "../outside.txt"));
    execute_agent_bootstrap_migration(&repo, None, |repo, _vault| {
        write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read_to_string(outside).unwrap(), "must survive\n");
}

#[test]
fn unsafe_legacy_project_slug_is_rejected() {
    let (_temp, repo, vault) = legacy_fixture();
    write(
        &repo.join("vault.config.json"),
        &serde_json::to_string_pretty(&serde_json::json!({
            "vault_root": vault,
            "project_slug": "../escape",
            "project_root": vault.join("Projects/legacy-demo")
        }))
        .unwrap(),
    );

    let error = inventory_agent_bootstrap(&repo, None).unwrap_err();
    assert!(error.to_string().contains("unsafe legacy project slug"));
}
