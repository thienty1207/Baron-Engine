use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{json, Value};
use tempfile::tempdir;

#[test]
fn control_plane_route_explains_security_skill_and_quality_gates() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    std::fs::create_dir_all(&repo).unwrap();
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
        .args([
            "control-plane",
            "route",
            "auth login with tenant RLS",
            repo.to_str().unwrap(),
            "--risk",
            "high",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Baron Control Plane Route"))
        .stdout(predicate::str::contains("security-audit"))
        .stdout(predicate::str::contains("security-auditor"))
        .stdout(predicate::str::contains("security-sensitive task"));
}

#[test]
fn control_plane_gate_evidence_is_required_and_recordable() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    std::fs::create_dir_all(&repo).unwrap();
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
        .args([
            "control-plane",
            "evidence",
            repo.to_str().unwrap(),
            "--required",
            "code-reviewer",
            "--required",
            "security-auditor",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Passed: `no`"))
        .stdout(predicate::str::contains("security-auditor"));

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "control-plane",
            "record-gate",
            "security-auditor",
            "security gate reviewed auth flow with evidence",
            repo.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("security-auditor"))
        .stdout(predicate::str::contains("Gate evidence recorded"));
}

#[test]
fn initialized_codex_status_and_prepare_agree_on_generated_quality_agents() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    let home = temp.path().join("baron-home");
    std::fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .env("BARON_HOME", &home)
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();

    let status = Command::cargo_bin("baron")
        .unwrap()
        .env("BARON_HOME", &home)
        .args(["control-plane", "status", repo.to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status = String::from_utf8(status).unwrap();

    let request = json!({
        "schema_version": 1,
        "task": "implement auth login with tenant RLS permissions and verify security tests"
    });
    let output = Command::cargo_bin("baron")
        .unwrap()
        .env("BARON_HOME", &home)
        .args([
            "control-plane",
            "prepare",
            "--adapter",
            "codex",
            "--vault",
            vault.to_str().unwrap(),
            "--json",
        ])
        .current_dir(&repo)
        .write_stdin(serde_json::to_vec(&request).unwrap())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let packet: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(packet["ok"], true);
    let prepare_has_control_plane_blocker = packet["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|blocker| blocker["code"] == "control_plane_invalid");
    let status_passed = status.contains("Passed: `yes`");
    assert_eq!(
        status_passed, !prepare_has_control_plane_blocker,
        "status: {status}; prepare blockers: {}",
        packet["blockers"]
    );
    assert!(status_passed, "{status}");
    assert!(!prepare_has_control_plane_blocker);
    for agent in ["code-reviewer", "security-auditor", "test-engineer"] {
        assert!(packet["route"]["mandatory_agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|selected| selected["name"] == agent));
    }
}
