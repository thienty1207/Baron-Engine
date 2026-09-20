use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn hidden_code_map_commands_are_available_to_ai_without_crowding_public_help() {
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
        .args(["automation", "code-map", "status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"provider\": \"graphify-local\""));
    let status = Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["automation", "code-map", "status", "--json"])
        .output()
        .unwrap();
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    let provider_present = status["present"].as_bool().unwrap();
    assert_eq!(
        status["action"].as_str().unwrap(),
        if provider_present {
            "refresh"
        } else {
            "survey_fallback"
        }
    );
    assert!(!repo.join(".baron/cache/code-graph").exists());

    if provider_present {
        eprintln!("skipped missing-provider refresh assertion: real Graphify is installed");
        return;
    }

    Command::cargo_bin("baron")
        .unwrap()
        .current_dir(&repo)
        .args(["automation", "code-map", "refresh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Survey fallback"));
    assert!(!repo.join(".baron/cache/code-graph").exists());
}
