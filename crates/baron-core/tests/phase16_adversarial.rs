//! Phase 16 independent adversarial verification fixtures.

use std::fs;
use std::path::{Path, PathBuf};

use baron_core::config::{initialize_project_with_options, load_project_config, AdapterKind};
use baron_core::control_plane::record_gate_evidence;
use baron_core::harness_experiment::{finalize_experiment, record_fresh_rerun};
use baron_core::migration::rollback_migration;
use baron_core::plan::start_or_resume_plan;
use baron_core::review_gate::close_finding;
use baron_core::state_guard::require_coherent_execution_state;
use baron_core::vault::{ensure_vault, VaultContext};
use tempfile::tempdir;

#[cfg(unix)]
fn link_directory(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn link_directory(target: &Path, link: &Path) -> std::io::Result<()> {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => Ok(()),
        Err(symlink_error) => {
            // A junction does not require Developer Mode and exercises the
            // same path traversal boundary on ordinary Windows CI workers.
            let command = format!(
                "New-Item -ItemType Junction -Path '{}' -Target '{}' | Out-Null",
                link.display().to_string().replace('\'', "''"),
                target.display().to_string().replace('\'', "''")
            );
            let status = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", &command])
                .status()?;
            if status.success() && link.is_dir() {
                Ok(())
            } else {
                Err(symlink_error)
            }
        }
    }
}

#[test]
fn linked_baron_directory_is_rejected_before_writing_outside_the_project() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&vault).unwrap();

    // Windows hosts without symlink/junction privileges cannot exercise this
    // filesystem mechanic; the supported-platform test remains deterministic
    // everywhere the link can be created.
    if link_directory(&outside, &repo.join(".baron")).is_err() {
        return;
    }

    let result = initialize_project_with_options(&repo, Some(AdapterKind::Codex), &vault, None);

    assert!(
        result.is_err(),
        "a linked .baron directory must fail closed"
    );
    assert!(
        fs::read_dir(&outside).unwrap().next().is_none(),
        "initialization must not write through a project link"
    );
}

#[test]
fn linked_control_plane_state_is_rejected_before_writing_outside_the_project() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    let vault_root = temp.path().join("vault");
    fs::create_dir_all(repo.join("docs/baron")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&vault_root).unwrap();

    let control_plane = repo.join("docs/baron/control-plane");
    if link_directory(&outside, &control_plane).is_err() {
        return;
    }

    let vault = VaultContext {
        vault_root: vault_root.clone(),
        repo_root: repo.clone(),
        project_id: "phase16-project".to_string(),
        identity_binding: "phase16-binding".to_string(),
        project_slug: "phase16-project".to_string(),
        project_root: vault_root.join("project"),
        baron_artifacts_root: vault_root.join("artifacts"),
        index_path: PathBuf::from("index.sqlite"),
        state_path: PathBuf::from("state.json"),
        approved_global_path: PathBuf::from("approved.md"),
        global_candidates_path: PathBuf::from("candidates.md"),
    };

    let result = record_gate_evidence(&repo, &vault, "test-engineer", "linked path");

    assert!(
        result.is_err(),
        "linked control-plane path must fail closed"
    );
    assert!(
        fs::read_dir(&outside).unwrap().next().is_none(),
        "control-plane recording must not write through a project link"
    );
}

#[test]
fn rollback_rejects_manifest_path_escape_before_deleting_outside_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault = temp.path().join("vault");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let protected = outside.join("protected.txt");
    fs::write(&protected, "must survive\n").unwrap();

    let migration_id = "phase16-escape";
    let backup_root = vault.join("Artifacts/Baron/Migrations").join(migration_id);
    fs::create_dir_all(&backup_root).unwrap();
    let manifest = format!(
        "{{\"migration_id\":\"{migration_id}\",\"repo_root\":{},\"vault_root\":{},\"entries\":[{{\"scope\":\"repo\",\"relative_path\":\"../outside/protected.txt\",\"existed\":true,\"was_directory\":false,\"original_hash\":null}}]}}",
        serde_json::to_string(&repo.canonicalize().unwrap()).unwrap(),
        serde_json::to_string(&vault.canonicalize().unwrap()).unwrap(),
    );
    fs::write(backup_root.join("manifest.json"), manifest).unwrap();

    let result = rollback_migration(&repo, &vault, migration_id);

    assert!(
        result.is_err(),
        "rollback must reject an escaping manifest path"
    );
    assert_eq!(fs::read_to_string(&protected).unwrap(), "must survive\n");
}

#[test]
fn active_plan_path_escape_is_rejected_before_writing_outside_the_project() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let outside = temp.path().join("outside");
    let vault_root = temp.path().join("vault");
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(&vault_root).unwrap();

    let outside_plan = outside.join("plan.md");
    fs::write(
        &outside_plan,
        "---\ntitle: linked plan\nstatus: in_progress\n---\n\n# linked plan\n",
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/plans/CURRENT.md"),
        format!(
            "# Current Baron Plan\n\n- Title: linked plan\n- Plan: `{}`\n- Status: `in_progress`\n- Risk: `low`\n",
            outside_plan.display()
        ),
    )
    .unwrap();
    let before = fs::read(&outside_plan).unwrap();
    let vault = VaultContext {
        vault_root: vault_root.clone(),
        repo_root: repo.clone(),
        project_id: "phase16-project".to_string(),
        identity_binding: "phase16-binding".to_string(),
        project_slug: "phase16-project".to_string(),
        project_root: vault_root.join("project"),
        baron_artifacts_root: vault_root.join("artifacts"),
        index_path: PathBuf::from("index.sqlite"),
        state_path: PathBuf::from("state.json"),
        approved_global_path: PathBuf::from("approved.md"),
        global_candidates_path: PathBuf::from("candidates.md"),
    };

    let result = start_or_resume_plan(&repo, &vault, "linked plan");

    assert!(
        result.is_err(),
        "active plan paths must stay inside the project"
    );
    assert_eq!(fs::read(&outside_plan).unwrap(), before);
}

#[test]
fn project_id_tampering_cannot_select_another_capsule_without_its_binding() {
    let temp = tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    let vault = temp.path().join("vault");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();

    let first_config =
        initialize_project_with_options(&first, Some(AdapterKind::Codex), &vault, None).unwrap();
    initialize_project_with_options(&second, Some(AdapterKind::Claude), &vault, None).unwrap();
    ensure_vault(&vault, &first).unwrap();
    let second_config = load_project_config(&second).unwrap();
    let path = first.join(".baron/project.toml");
    let mut content = fs::read_to_string(&path).unwrap();
    content = content.replace(
        &format!("project_id = \"{}\"", first_config.project_id),
        &format!("project_id = \"{}\"", second_config.project_id),
    );
    fs::write(&path, content).unwrap();

    let result = require_coherent_execution_state(&first, &vault);

    assert!(
        result.is_err(),
        "a foreign project ID must fail capsule binding"
    );
}

#[test]
fn managed_record_ids_reject_path_escape_before_any_write() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("repo");
    let vault_root = temp.path().join("vault");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault_root).unwrap();
    let vault = VaultContext {
        vault_root: vault_root.clone(),
        repo_root: repo.clone(),
        project_id: "phase16-project".to_string(),
        identity_binding: "phase16-binding".to_string(),
        project_slug: "phase16-project".to_string(),
        project_root: vault_root.join("project"),
        baron_artifacts_root: vault_root.join("artifacts"),
        index_path: PathBuf::from("index.sqlite"),
        state_path: PathBuf::from("state.json"),
        approved_global_path: PathBuf::from("approved.md"),
        global_candidates_path: PathBuf::from("candidates.md"),
    };

    assert!(record_fresh_rerun(&repo, &vault, "../outside", true, true, true, true, "ok").is_err());
    assert!(finalize_experiment(&repo, &vault, "C:\\outside", "pending").is_err());
    assert!(close_finding(&repo, &vault, "../outside", "fixed", "verified").is_err());
}
