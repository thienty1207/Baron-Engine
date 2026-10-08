use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::identity::{capsule_key, project_id_for_path};
use baron_core::migration::{
    execute_agent_bootstrap_migration, execute_agent_bootstrap_migration_with_outputs,
    inventory_agent_bootstrap, migration_status, rollback_migration, MigrationAction,
    MigrationAssetKind, MigrationInstallOutputs,
};
use baron_core::safe_io::acquire_project_lock_with_timeout;
use baron_core::vault::project_slug;
use sha2::{Digest, Sha256};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn capsule_root(vault: &Path, repo: &Path) -> PathBuf {
    let slug = project_slug(repo);
    let project_id = project_id_for_path(repo).unwrap();
    vault.join("Projects").join(capsule_key(&slug, &project_id))
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

    let receipt = execute_agent_bootstrap_migration_with_outputs(&repo, None, |repo, vault| {
        write(
            &repo.join(".baron/project.toml"),
            "schema_version = 1\nproject_slug = \"demo\"\nadapters = [\"codex\"]\n",
        );
        write(
            &repo.join("AGENTS.md"),
            "# User Rules\n\nKeep this.\n\n<!-- BARON:MANAGED:START -->\nBaron native\n<!-- BARON:MANAGED:END -->\n",
        );
        MigrationInstallOutputs::capture(
            repo,
            vault,
            vec!["AGENTS.md".to_string()],
            Vec::new(),
        )
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
    let project_root = capsule_root(&vault, &repo);
    assert!(project_root.join("Facts.md").exists());
    assert!(project_root.join("Research/backend.md").exists());
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
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let receipt = execute_agent_bootstrap_migration(&repo, None, |_repo, _vault| Ok(())).unwrap();
    write(&repo.join("after-migration.txt"), "keep me\n");
    write(
        &repo.join("docs/baron/plans/post-migration-plan.md"),
        "# New Baron Plan\n",
    );
    let project_root = capsule_root(&vault, &repo);
    write(
        &project_root.join("post-migration-memory.md"),
        "# New Baron Memory\n",
    );

    let report = rollback_migration(&repo, &vault, &receipt.migration_id).unwrap();

    assert_eq!(report.status, "rolled_back");
    assert!(repo.join("vault.config.json").exists());
    assert!(repo.join("scripts/agent-memory.js").exists());
    assert!(repo.join(".codex/agents/unsafe-agent.toml").exists());
    assert!(repo.join(".baron/project.toml").exists());
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
    assert!(project_root.join("post-migration-memory.md").exists());
    assert!(migration_status(&repo).unwrap().contains("rolled_back"));
}

#[test]
fn explicit_rollback_preserves_modified_imports_and_records_recovery() {
    let (_temp, repo, vault) = legacy_fixture();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let receipt = execute_agent_bootstrap_migration(&repo, None, |_repo, _vault| Ok(())).unwrap();
    let imported_memory = capsule_root(&vault, &repo).join("Facts.md");
    let user_update = "# User update after migration\n";
    write(&imported_memory, user_update);

    let result = rollback_migration(&repo, &vault, &receipt.migration_id);

    assert!(
        result.is_err(),
        "rollback must reject a changed managed target"
    );
    assert_eq!(fs::read_to_string(&imported_memory).unwrap(), user_update);
    assert!(!repo.join("vault.config.json").exists());
    assert!(migration_status(&repo).unwrap().contains("needs_recovery"));
    assert!(receipt.backup_root.join("failure.json").is_file());
}

#[test]
fn explicit_rollback_without_handoff_baseline_requires_recovery() {
    let (_temp, repo, vault) = legacy_fixture();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let receipt = execute_agent_bootstrap_migration(&repo, None, |_repo, _vault| Ok(())).unwrap();
    let imported_memory = capsule_root(&vault, &repo).join("Facts.md");
    let imported_bytes = fs::read(&imported_memory).unwrap();
    let manifest_path = receipt.backup_root.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["post_handoff_captured"] = serde_json::Value::Bool(false);
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let result = rollback_migration(&repo, &vault, &receipt.migration_id);

    let error = result.expect_err("rollback must reject an incomplete baseline");
    assert!(
        error
            .to_string()
            .contains("no complete persisted handoff baseline"),
        "rollback should identify the missing baseline, got: {error}"
    );
    assert_eq!(fs::read(&imported_memory).unwrap(), imported_bytes);
    assert!(!repo.join("vault.config.json").exists());
    assert!(migration_status(&repo).unwrap().contains("needs_recovery"));
    assert!(receipt.backup_root.join("failure.json").is_file());
}

#[test]
fn explicit_rollback_cannot_race_an_active_installer_callback() {
    let (_temp, repo, vault) = legacy_fixture();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let (callback_started_tx, callback_started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let migration_repo = repo.clone();
    let migration = thread::spawn(move || {
        execute_agent_bootstrap_migration_with_outputs(
            &migration_repo,
            None,
            move |_repo_root, _vault_root| {
                callback_started_tx.send(()).unwrap();
                release_rx
                    .recv_timeout(Duration::from_secs(10))
                    .map_err(anyhow::Error::from)
                    .context("test did not release the installer callback")?;
                Ok(MigrationInstallOutputs::default())
            },
        )
    });

    callback_started_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("installer callback did not start");
    let migrations_root = vault.join("Artifacts/Baron/Migrations");
    let migration_id = fs::read_dir(&migrations_root)
        .unwrap()
        .next()
        .expect("migration did not reserve its backup root")
        .unwrap()
        .file_name()
        .to_string_lossy()
        .into_owned();
    let imported_memory = capsule_root(&vault, &repo).join("Facts.md");
    assert!(imported_memory.is_file());

    let rollback = rollback_migration(&repo, &vault, &migration_id).expect_err(
        "explicit rollback must not restore while the installer callback can still publish",
    );
    assert!(
        rollback.to_string().contains("migration lifecycle fence"),
        "active rollback should fail at the lifecycle fence, got: {rollback}"
    );
    assert!(
        imported_memory.is_file(),
        "active migration data was rolled back"
    );

    release_tx.send(()).unwrap();
    let receipt = migration
        .join()
        .expect("migration thread panicked")
        .expect("migration should complete after the callback is released");
    assert_eq!(receipt.status, "completed");
    assert!(migration_status(&repo).unwrap().contains("completed"));
}

#[test]
fn installer_callback_cannot_reenter_rollback_on_the_same_thread() {
    let (_temp, repo, vault) = legacy_fixture();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();

    let receipt =
        execute_agent_bootstrap_migration_with_outputs(&repo, None, |repo_root, vault_root| {
            let migrations_root = vault_root.join("Artifacts/Baron/Migrations");
            let migration_id = fs::read_dir(&migrations_root)?
                .filter_map(std::result::Result::ok)
                .find(|entry| entry.path().is_dir())
                .context("migration callback did not find its active backup root")?
                .file_name()
                .to_string_lossy()
                .into_owned();

            rollback_migration(repo_root, vault_root, &migration_id)
                .expect_err("same-thread rollback must be rejected during installer callback");
            Ok(MigrationInstallOutputs::default())
        })
        .unwrap();

    assert_eq!(receipt.status, "completed");
    assert!(migration_status(&repo).unwrap().contains("completed"));
    assert!(capsule_root(&vault, &repo).join("Facts.md").is_file());
}

#[test]
fn installer_return_window_edit_is_not_adopted_as_migration_output() {
    let (_temp, repo, vault) = legacy_fixture();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let agents_path = repo.join("AGENTS.md");
    let (locked_tx, locked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (callback_ready_tx, callback_ready_rx) = mpsc::channel();
    let migration_repo = repo.clone();
    let migration = thread::spawn(move || {
        execute_agent_bootstrap_migration_with_outputs(
            &migration_repo,
            None,
            move |repo_root, vault_root| {
                let installer_path = repo_root.join("AGENTS.md");
                let mut installer_bytes = fs::read(&installer_path)?;
                installer_bytes.extend_from_slice(b"\n# successful installer output\n");
                fs::write(&installer_path, installer_bytes)?;
                let outputs = MigrationInstallOutputs::capture(
                    repo_root,
                    vault_root,
                    vec!["AGENTS.md".to_string()],
                    Vec::new(),
                )?;

                let held_repo = repo_root.to_path_buf();
                let holder_release = release_rx;
                let _ = thread::spawn(move || {
                    let _lock =
                        acquire_project_lock_with_timeout(&held_repo, Duration::from_secs(5))
                            .unwrap();
                    locked_tx.send(()).unwrap();
                    holder_release.recv_timeout(Duration::from_secs(5)).unwrap();
                });
                locked_rx.recv_timeout(Duration::from_secs(5))?;
                callback_ready_tx.send(()).unwrap();
                Ok(outputs)
            },
        )
    });

    callback_ready_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("installer callback did not reach the return window");
    let mut user_edit = fs::read(&agents_path).unwrap();
    user_edit.extend_from_slice(b"\n# user edit after installer success\n");
    fs::write(&agents_path, &user_edit).unwrap();
    release_tx.send(()).unwrap();

    let migration_result = migration.join().unwrap();
    let rollback_result = match migration_result {
        Ok(receipt) => rollback_migration(&repo, &vault, &receipt.migration_id),
        Err(error) => Err(error),
    };

    assert!(
        rollback_result.is_err(),
        "a user edit in the installer-return window must block rollback"
    );
    assert_eq!(
        fs::read(&agents_path).unwrap(),
        user_edit,
        "installer output capture must not adopt and later erase the concurrent instructions edit"
    );
}

#[test]
fn installer_return_window_edit_inside_agents_managed_block_is_preserved() {
    let (_temp, repo, vault) = legacy_fixture();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    let agents_path = repo.join("AGENTS.md");
    let (locked_tx, locked_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (callback_ready_tx, callback_ready_rx) = mpsc::channel();
    let migration_repo = repo.clone();
    let migration = thread::spawn(move || {
        execute_agent_bootstrap_migration_with_outputs(
            &migration_repo,
            None,
            move |repo_root, vault_root| {
                let installer_path = repo_root.join("AGENTS.md");
                let mut installer_bytes = fs::read(&installer_path)?;
                installer_bytes.extend_from_slice(b"\n# successful installer output\n");
                fs::write(&installer_path, installer_bytes)?;
                let outputs = MigrationInstallOutputs::capture(
                    repo_root,
                    vault_root,
                    vec!["AGENTS.md".to_string()],
                    Vec::new(),
                )?;

                let held_repo = repo_root.to_path_buf();
                let holder_release = release_rx;
                let _ = thread::spawn(move || {
                    let _lock =
                        acquire_project_lock_with_timeout(&held_repo, Duration::from_secs(5))
                            .unwrap();
                    locked_tx.send(()).unwrap();
                    holder_release.recv_timeout(Duration::from_secs(5)).unwrap();
                });
                locked_rx.recv_timeout(Duration::from_secs(5))?;
                callback_ready_tx.send(()).unwrap();
                Ok(outputs)
            },
        )
    });

    callback_ready_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("installer callback did not reach the return window");
    let mut user_edit = fs::read(&agents_path).unwrap();
    let current = String::from_utf8(user_edit.clone()).unwrap();
    user_edit = current
        .replace(
            "legacy runtime instructions",
            "user-owned managed-block edit",
        )
        .into_bytes();
    fs::write(&agents_path, &user_edit).unwrap();
    release_tx.send(()).unwrap();

    let result = migration.join().unwrap();
    assert!(
        result.is_err(),
        "migration must fail closed when AGENTS.md changed after installer capture"
    );
    assert_eq!(
        fs::read(&agents_path).unwrap(),
        user_edit,
        "managed-block cleanup must preserve an edit made after installer capture"
    );
    assert!(migration_status(&repo)
        .unwrap()
        .contains("rolled_back_with_conflicts"));
}

#[test]
fn failed_after_installer_output_capture_uses_persisted_hash_for_rollback() {
    let (_temp, repo, _vault) = legacy_fixture();
    let agents_path = repo.join("AGENTS.md");
    let original_agents = fs::read(&agents_path).unwrap();

    let error =
        execute_agent_bootstrap_migration_with_outputs(&repo, None, |repo_root, vault_root| {
            fs::write(repo_root.join("AGENTS.md"), [0xff, 0xfe, 0xfd])?;
            MigrationInstallOutputs::capture(
                repo_root,
                vault_root,
                vec!["AGENTS.md".to_string()],
                Vec::new(),
            )
        })
        .expect_err("invalid installer output should fail during legacy block cleanup");

    assert!(
        error.to_string().contains("not valid UTF-8"),
        "got: {error}"
    );
    assert_eq!(
        fs::read(&agents_path).unwrap(),
        original_agents,
        "rollback must restore acknowledged installer output after a later step fails"
    );
    let status = migration_status(&repo).unwrap();
    let backup_line = status
        .lines()
        .find(|line| line.starts_with("- Backup: `"))
        .unwrap();
    let backup_root = backup_line
        .trim_start_matches("- Backup: `")
        .trim_end_matches('`');
    let failure = fs::read_to_string(PathBuf::from(backup_root).join("failure.json")).unwrap();
    assert!(
        status.contains("- Status: `rolled_back`"),
        "migration status: {status}; failure: {failure}"
    );
}

#[test]
fn user_config_edit_during_installer_callback_is_not_adopted_for_rollback() {
    let (_temp, repo, _vault) = legacy_fixture();
    let project_config = repo.join(".baron/project.toml");
    let mut user_config = Vec::new();

    let error =
        execute_agent_bootstrap_migration_with_outputs(&repo, None, |repo_root, vault_root| {
            write(
                &project_config,
                "schema_version = 1\nproject_slug = \"demo\"\nadapters = [\"codex\"]\n",
            );
            user_config = fs::read(&project_config)?;
            user_config.extend_from_slice(b"\n# concurrent user routing change\n");
            fs::write(&project_config, &user_config)?;

            MigrationInstallOutputs::capture(
                repo_root,
                vault_root,
                vec![".baron/project.toml".to_string()],
                Vec::new(),
            )
        })
        .expect_err("migration must not adopt callback-time project config as installer output");

    assert!(
        error.to_string().contains("user-owned configuration"),
        "the injected post-callback failure did not occur as expected: {error}"
    );
    assert_eq!(
        fs::read(&project_config).unwrap(),
        user_config,
        "rollback must not erase user config bytes captured during the installer callback"
    );
    assert!(migration_status(&repo)
        .unwrap()
        .contains("rolled_back_with_conflicts"));
}

#[test]
fn migration_cannot_claim_project_or_local_config_as_installer_output() {
    for relative_path in [".baron/project.toml", ".baron/local.toml"] {
        let (_temp, repo, _vault) = legacy_fixture();
        initialize_project(&repo, AdapterKind::Codex, &_vault).unwrap();
        let target = repo.join(relative_path);
        if relative_path == ".baron/local.toml" {
            fs::write(&target, b"user-owned local config\n").unwrap();
        }
        let original = fs::read(&target).unwrap();

        let error =
            execute_agent_bootstrap_migration_with_outputs(&repo, None, |repo_root, vault_root| {
                MigrationInstallOutputs::capture(
                    repo_root,
                    vault_root,
                    vec![relative_path.to_string()],
                    Vec::new(),
                )
            })
            .expect_err("user-owned project/local config must not become rollback authority");

        assert!(
            error.to_string().contains("user-owned configuration"),
            "unexpected migration rejection for {relative_path}: {error}"
        );
        assert_eq!(fs::read(&target).unwrap(), original);
    }
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

    let result = execute_agent_bootstrap_migration(&repo, None, |_repo, _vault| {
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
fn failed_install_does_not_clobber_changes_after_handoff() {
    let (_temp, repo, _vault) = legacy_fixture();

    let result = execute_agent_bootstrap_migration(&repo, None, |repo, _vault| {
        write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
        anyhow::bail!("injected install failure after handoff")
    });

    assert!(result.is_err());
    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join("vault.config.json").exists());
    assert!(migration_status(&repo)
        .unwrap()
        .contains("rolled_back_with_conflicts"));
}

#[test]
fn failure_after_partial_import_requires_recovery() {
    let (_temp, repo, vault) = legacy_fixture();
    // Backup can preserve this directory, but importing the trace as a file
    // fails after Vault memory and the repository plan have been published.
    fs::create_dir_all(repo.join("docs/baron/traces/legacy-trace.md")).unwrap();
    let mut installer_called = false;
    let error = execute_agent_bootstrap_migration(&repo, None, |_, _| {
        installer_called = true;
        Ok(())
    })
    .unwrap_err();

    assert!(!installer_called);
    assert!(error.to_string().contains("regular file"));
    assert_eq!(
        fs::read_to_string(capsule_root(&vault, &repo).join("Facts.md")).unwrap(),
        "# Facts\n\n- Legacy API uses Rust.\n"
    );
    assert!(repo
        .join("docs/baron/plans/2026-05-21/2026-05-21-auth.md")
        .is_file());
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(".baron/migration-state.json")).unwrap())
            .unwrap();
    let backup = PathBuf::from(state["backup_root"].as_str().unwrap());
    let failure: serde_json::Value =
        serde_json::from_slice(&fs::read(backup.join("failure.json")).unwrap()).unwrap();
    assert!(backup.join("source-vault/legacy-demo/Facts.md").is_file());
    assert!(failure["rollback"].is_null());
    assert!(failure["rollbackError"].as_str().is_some());
    assert_eq!(failure["status"], "needs_recovery");
    assert_eq!(state["status"], "needs_recovery");
}

#[test]
fn failed_automatic_restoration_reports_rollback_failed() {
    let (_temp, repo, vault) = legacy_fixture();
    let result = execute_agent_bootstrap_migration(&repo, None, |_, _| {
        let backup = fs::read_dir(vault.join("Artifacts/Baron/Migrations"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        // Corrupt the recovery copy without changing the live handoff paths.
        fs::remove_file(backup.join("repo/AGENTS.md")).unwrap();
        anyhow::bail!("install failure with missing recovery copy")
    });
    assert!(result.is_err());
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(".baron/migration-state.json")).unwrap())
            .unwrap();
    let backup = PathBuf::from(state["backup_root"].as_str().unwrap());
    let failure: serde_json::Value =
        serde_json::from_slice(&fs::read(backup.join("failure.json")).unwrap()).unwrap();
    assert!(failure["rollbackError"].as_str().is_some());
    assert_eq!(failure["status"], "rollback_failed");
    assert_eq!(state["status"], "rollback_failed");
    assert!(repo.join("AGENTS.md").is_file());
}

#[test]
fn migration_releases_project_lock_while_install_callback_runs() {
    let (_temp, repo, _vault) = legacy_fixture();
    let contender_repo = repo.clone();

    execute_agent_bootstrap_migration(&repo, None, move |repo, _vault| {
        let contender = std::thread::spawn(move || {
            acquire_project_lock_with_timeout(&contender_repo, Duration::from_millis(300)).is_ok()
        });
        assert!(
            contender.join().unwrap(),
            "migration callback must not run under the project mutation lock"
        );
        write(&repo.join(".baron/project.toml"), "schema_version = 1\n");
        Ok(())
    })
    .unwrap();
}

#[test]
fn unchanged_markdown_containing_the_import_can_roll_back() {
    let (_temp, repo, vault) = legacy_fixture();
    let target = repo.join("docs/baron/plans/2026-05-21/2026-05-21-auth.md");
    let existing =
        "# Existing plan\n\n# Auth Plan\n\n- verified old plan\n\n- Additional user content\n";
    write(&target, existing);

    execute_agent_bootstrap_migration(&repo, None, |_, _| {
        anyhow::bail!("install failed after a no-op Markdown import")
    })
    .unwrap_err();

    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(".baron/migration-state.json")).unwrap())
            .unwrap();
    assert_eq!(state["status"], "rolled_back");
    assert_eq!(fs::read_to_string(target).unwrap(), existing);
    assert!(!capsule_root(&vault, &repo).join("Facts.md").exists());
}

#[test]
fn malformed_handoff_manifest_requires_recovery_and_preserves_the_original_error() {
    let (_temp, repo, vault) = legacy_fixture();
    let error = execute_agent_bootstrap_migration(&repo, None, |_, _| {
        let backup = fs::read_dir(vault.join("Artifacts/Baron/Migrations"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        write(&backup.join("manifest.json"), "{malformed");
        anyhow::bail!("original installer failure")
    })
    .unwrap_err();

    assert_eq!(error.to_string(), "original installer failure");
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(".baron/migration-state.json")).unwrap())
            .unwrap();
    let backup = PathBuf::from(state["backup_root"].as_str().unwrap());
    let failure: serde_json::Value =
        serde_json::from_slice(&fs::read(backup.join("failure.json")).unwrap()).unwrap();
    assert_eq!(state["status"], "needs_recovery");
    assert_eq!(failure["status"], "needs_recovery");
    assert_eq!(failure["error"], "original installer failure");
    assert!(failure["rollback"].is_null());
    assert!(failure["rollbackError"].as_str().is_some());
    assert!(backup.join("repo/AGENTS.md").is_file());
    assert!(capsule_root(&vault, &repo).join("Facts.md").is_file());
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
    let project_root = capsule_root(&receipt.destination_vault, &repo);
    assert!(project_root.join("Facts.md").exists());
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
