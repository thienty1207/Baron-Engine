use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use assert_cmd::Command;
use baron_core::vault::vault_context_without_create;
use predicates::prelude::*;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn non_shadow_init_installs_codex_and_configuration() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adapter initialized: `codex`"));

    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join(".baron/local.toml").exists());
    assert!(repo.join("AGENTS.md").exists());
    assert!(repo.join(".codex/skills/superpowers/SKILL.md").exists());
    assert!(repo.join("docs/baron/harness/DOMAIN_LANGUAGE.md").exists());
    let context = vault_context_without_create(&vault, &repo).unwrap();
    assert!(context.project_root.join("Facts.md").exists());
    assert!(context
        .project_root
        .join("ProductHarness/DOMAIN_LANGUAGE.md")
        .exists());
}

#[test]
fn setup_vault_then_init_codex_from_project_directory() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let home = temp.path().join("home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&vault).unwrap();
    std::env::set_var("BARON_HOME", &home);

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&vault)
        .args(["setup", "--vault"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Setup"))
        .stdout(predicate::str::contains("Default Vault"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["init", "--codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adapter initialized: `codex`"));

    std::env::remove_var("BARON_HOME");
    assert!(home.join("config.toml").exists());
    assert!(repo.join(".baron/project.toml").exists());
    assert!(repo.join("AGENTS.md").exists());
    let context = vault_context_without_create(&vault, &repo).unwrap();
    assert!(context.project_root.join("Facts.md").exists());
}

#[test]
fn init_can_set_platform_focus_separately_or_with_adapter() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let second = temp.path().join("second");
    let vault = temp.path().join("Vault");
    let home = temp.path().join("home");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::create_dir_all(&vault).unwrap();
    std::env::set_var("BARON_HOME", &home);

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&vault)
        .args(["setup", "--vault"])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["init", "--codex"])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["init", "--fullstack"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Primary platform: `fullstack`"));

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("platform = \"fullstack\""));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&second)
        .args(["init", "--codex", "--tool"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adapter initialized: `codex`"))
        .stdout(predicate::str::contains("Platform focus: `tool`"));

    std::env::remove_var("BARON_HOME");
    let second_config = fs::read_to_string(second.join(".baron/project.toml")).unwrap();
    assert!(second_config.contains("platform = \"tool\""));
}

#[test]
fn init_generates_platform_intelligence_and_expands_architecture_safely() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(repo.join("legacy-src")).unwrap();
    fs::write(repo.join("legacy-src/app.ts"), "export const app = true;").unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--fullstack",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", repo.to_str().unwrap(), "--mobile"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extension platforms: `mobile`"));

    assert!(repo.join("legacy-src/app.ts").exists());
    assert!(repo.join("docs/baron/platform/PROJECT_PROFILE.md").exists());
    assert!(repo
        .join("docs/baron/architecture/CURRENT_ARCHITECTURE.md")
        .exists());
    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("platform = \"fullstack\""));
    assert!(config.contains("platform_extensions = [\"mobile\"]"));
}

#[test]
fn repeated_init_registers_codex_and_claude() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    for flag in ["--codex", "--claude"] {
        Command::cargo_bin("baron")
            .unwrap()
            .args([
                "init",
                repo.to_str().unwrap(),
                flag,
                "--vault",
                vault.to_str().unwrap(),
            ])
            .assert()
            .success();
    }

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("codex"));
    assert!(config.contains("claude"));
    assert!(repo.join("AGENTS.md").exists());
    assert!(repo.join("CLAUDE.md").exists());
}

#[test]
fn automation_reconcile_from_nested_path_preserves_ambiguous_local_marker_content() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src/features");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    write(
        &repo.join("AGENTS.md"),
        "# User Header\n\n<!-- BARON:MANAGED:START -->\nstale\n<!-- BARON:MANAGED:END -->\n",
    );

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["automation", "reconcile"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Remote release download: not attempted",
        ));

    let agents = fs::read_to_string(repo.join("AGENTS.md")).unwrap();
    assert!(agents.contains("# User Header"));
    assert!(agents.contains("\nstale\n"));
}

fn snapshot_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                collect(root, &path, files);
            } else if path.is_file() {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(&path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

#[test]
fn update_dry_run_previews_the_safe_merge_without_writing_project_files() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src/features");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
    write(
        &repo.join("AGENTS.md"),
        "# User Header\n\n<!-- BARON:MANAGED:START -->\nlocal edit\n<!-- BARON:MANAGED:END -->\n",
    );
    let before = fs::read_to_string(repo.join("AGENTS.md")).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["update", "--dry-run", "--installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Safe Update Preview"))
        .stdout(predicate::str::contains("No project files were written."))
        .stdout(predicate::str::contains("AGENTS.md"));

    assert_eq!(fs::read_to_string(repo.join("AGENTS.md")).unwrap(), before);
    assert!(!repo.join(".baron/update").exists());
}

#[test]
fn update_dry_run_merges_all_registered_adapters_without_writing_any_repo_file() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src/features");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();

    for adapter in ["--codex", "--claude"] {
        Command::cargo_bin("baron")
            .unwrap()
            .args([
                "init",
                repo.to_str().unwrap(),
                adapter,
                "--vault",
                vault.to_str().unwrap(),
            ])
            .assert()
            .success();
    }
    let before = snapshot_files(&repo);

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .args(["update", "--dry-run", "--installed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Safe Update Preview"))
        .stdout(predicate::str::contains("`AGENTS.md`: `keep_local`").not())
        .stdout(predicate::str::contains("`CLAUDE.md`: `keep_local`").not());

    assert_eq!(snapshot_files(&repo), before);
}

#[test]
fn context_uses_registered_adapter_and_local_vault_automatically() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let nested = repo.join("src");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&nested).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&nested)
        .arg("context")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Context Bundle - Codex"))
        .stdout(predicate::str::contains("Project: `demo`"));
}

#[test]
fn shadow_init_remains_read_only_and_does_not_require_vault() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", repo.to_str().unwrap(), "--agent", "--shadow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No files were written"));

    assert!(!repo.join(".baron/project.toml").exists());
}
