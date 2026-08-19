use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn reasonix_init_and_switch_keep_one_project_and_vault_identity() {
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
        .success();
    let before = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    let project_id = before
        .lines()
        .find_map(|line| line.strip_prefix("project_id = \""))
        .and_then(|value| value.strip_suffix('"'))
        .unwrap()
        .to_string();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["init", "--reasonix"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reasonix"));
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["adapter", "switch", "--to", "reasonix"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Brain/history: shared"));

    let after = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(after.contains(&format!("project_id = \"{project_id}\"")));
    assert!(after.contains("active_adapter = \"reasonix\""));
    assert!(after.contains("reasonix"));
    assert!(repo.join("REASONIX.md").is_file());
    assert!(repo.join(".reasonix/settings.json").is_file());

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["adapter", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active adapter: `reasonix`"))
        .stdout(predicate::str::contains("Memory/history namespace"))
        .stdout(predicate::str::contains(
            vault.to_string_lossy().to_string(),
        ));
}

#[test]
fn root_adapter_shortcuts_switch_without_the_long_subcommand() {
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
            "--fullstack",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .arg("--reasonix")
        .assert()
        .success()
        .stdout(predicate::str::contains("Baron Adapter Shortcut"))
        .stdout(predicate::str::contains("Active adapter: `reasonix`"))
        .stdout(predicate::str::contains("Brain/history: shared"));

    let after_reasonix = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(after_reasonix.contains("active_adapter = \"reasonix\""));
    assert!(after_reasonix.contains("codex"));
    assert!(after_reasonix.contains("reasonix"));

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .arg("--codex")
        .assert()
        .success()
        .stdout(predicate::str::contains("Active adapter: `codex`"));

    let after_codex = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(after_codex.contains("active_adapter = \"codex\""));
    assert!(after_codex.contains("project_id = \""));
    assert!(repo.join("REASONIX.md").is_file());
    assert!(repo.join(".reasonix/settings.json").is_file());
}

#[test]
fn root_adapter_shortcuts_are_visible_and_mutually_exclusive() {
    Command::cargo_bin("baron")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--codex"))
        .stdout(predicate::str::contains("--reasonix"));

    Command::cargo_bin("baron")
        .unwrap()
        .args(["--codex", "--reasonix"])
        .assert()
        .failure();
}

#[test]
fn reasonix_switch_dry_run_does_not_write_existing_contract() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("REASONIX.md"), "# User contract\n").unwrap();

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
    let before = fs::read_to_string(repo.join("REASONIX.md")).unwrap();
    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["adapter", "switch", "--to", "reasonix", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No project config or adapter files were written",
        ));
    assert_eq!(
        fs::read_to_string(repo.join("REASONIX.md")).unwrap(),
        before
    );
}

#[test]
fn reasonix_switch_refuses_identity_mismatch_before_writing() {
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
        .success();
    let config_path = repo.join(".baron/project.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let mismatched = config
        .lines()
        .map(|line| {
            if line.starts_with("project_id = ") {
                "project_id = \"wrong-project-id\"".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&config_path, mismatched).unwrap();
    let before = fs::read_to_string(&config_path).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["adapter", "switch", "--to", "reasonix"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("identity mismatch")
                .or(predicate::str::contains("project capsule is missing")),
        );
    assert_eq!(fs::read_to_string(config_path).unwrap(), before);
}
