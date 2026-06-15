use assert_cmd::Command;
use predicates::prelude::*;
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
        .stdout(predicate::str::contains("vibe-security-scan"))
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
