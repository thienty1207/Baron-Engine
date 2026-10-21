use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn hidden_code_map_commands_are_available_to_ai_without_crowding_public_help() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let unavailable_provider_path = temp.path().join("no-graphify");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&unavailable_provider_path).unwrap();
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
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("automation").not())
        .stdout(predicate::str::contains("code-map").not());

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .env("PATH", &unavailable_provider_path)
        .args(["automation", "code-map", "status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"provider\": \"graphify-local\""));
    let status = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .env("PATH", &unavailable_provider_path)
        .args(["automation", "code-map", "status", "--json"])
        .output()
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["provider"].as_str(), Some("graphify-local"));
    assert_eq!(status["present"].as_bool(), Some(false));
    assert_eq!(status["action"].as_str(), Some("survey_fallback"));
    assert!(!repo.join(".baron/cache/code-graph").exists());

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .env("PATH", &unavailable_provider_path)
        .args(["automation", "code-map", "refresh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Survey fallback"));
    assert!(!repo.join(".baron/cache/code-graph").exists());
}
