use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn historical_value() -> String {
    ['r', 'e', 'a', 's', 'o', 'n', 'i', 'x'].iter().collect()
}

fn write_legacy_config(repo: &std::path::Path, vault: &std::path::Path) -> String {
    let value = historical_value();
    fs::create_dir_all(repo.join(".baron")).unwrap();
    fs::write(
        repo.join(".baron/project.toml"),
        format!(
            "schema_version = 4\nproject_id = \"legacy-project\"\nproject_slug = \"legacy-project\"\nadapters = [\"{value}\"]\nactive_adapter = \"{value}\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n"
        ),
    )
    .unwrap();
    fs::write(
        repo.join(".baron/local.toml"),
        format!("vault_path = {:?}\n", vault.to_string_lossy().to_string()),
    )
    .unwrap();
    value
}

#[test]
fn unsupported_legacy_config_is_preserved_until_explicit_supported_init() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-project");
    let vault = temp.path().join("Vault");
    let historical = write_legacy_config(&repo, &vault);

    Command::cargo_bin("baron")
        .unwrap()
        .args(["adapter", "status", repo.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active adapter: `unknown`"))
        .stdout(predicate::str::contains("Registered adapters: none"));

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

    let config = fs::read_to_string(repo.join(".baron/project.toml")).unwrap();
    assert!(config.contains("active_adapter = \"codex\""));
    assert!(config.contains(&historical));
    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join("AGENTS.md").is_file());
    assert!(!repo.join("CLAUDE.md").exists());
}

#[test]
fn unsupported_legacy_cli_value_cannot_be_selected_or_installed() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    fs::create_dir_all(&repo).unwrap();
    let historical = historical_value();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--codex"))
        .stdout(predicate::str::contains("--claude"))
        .stdout(predicate::str::contains("--agent").not());

    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", repo.to_str().unwrap()])
        .arg(format!("--{historical}"))
        .assert()
        .failure();
}

#[test]
fn unsupported_legacy_config_cannot_trigger_an_implicit_update_adapter() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("legacy-project");
    let vault = temp.path().join("Vault");
    write_legacy_config(&repo, &vault);

    Command::cargo_bin("baron")
        .unwrap()
        .args(["update", repo.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No supported adapter is configured",
        ));
}

#[test]
fn supported_codex_and_claude_init_preserve_one_core_and_thin_bridges() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

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

    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join("CLAUDE.md").is_file());
    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(!repo.join(".codex/skills/superpowers").exists());
    assert!(!repo.join(".claude/skills/superpowers").exists());
    let retired_root = repo.join(format!(".{}", historical_value()));
    assert!(!retired_root.exists());
}
