use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use tempfile::tempdir;

// Portability: target transport assertions are cross-platform and avoid shell
// command construction in the fixture itself.

fn init(repo: &std::path::Path, vault: &std::path::Path, adapter: &str) {
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

/// Promoted Phase 5 regression: structured prepare transports task text as
/// data without shell interpolation.
#[test]
fn target_prepare_protocol_accepts_adversarial_structured_task_input() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("structured-input");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    let long_text = "long-task-marker ".repeat(4_000);
    let payload = json!({
        "schema_version": 1,
        "repo": repo,
        "adapter": "codex",
        "task": "Quotes: \\\"single\\\" and Unicode tiếng Việt\\nline two; shell: $() && ; | > <; JSON: {\\\"key\\\":true}; ".to_string() + &long_text,
        "constraints": ["preserve user files", "no shell interpolation"],
    });

    Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "prepare", "--adapter", "codex", "--json"])
        .current_dir(&repo)
        .write_stdin(serde_json::to_vec(&payload).unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("schema_version"))
        .stdout(predicate::str::contains("task"));
}

/// Promoted Phase 5 regression: malformed structured input returns a stable
/// machine-readable error without invoking a shell.
#[test]
fn target_prepare_protocol_reports_malformed_payload_as_structured_error() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("malformed-structured-input");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "prepare", "--adapter", "codex", "--json"])
        .current_dir(&repo)
        .write_stdin(b"{ not valid structured input")
        .assert()
        .failure()
        .stdout(predicate::str::contains("error_code"))
        .stderr(predicate::str::is_empty());
}

/// Promoted Phase 7 target: Database is a separate target profile and the
/// existing init surface accepts it for active adapters.
#[test]
fn target_database_profile_flag_is_available() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("database-profile");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "init",
            repo.to_str().unwrap(),
            "--codex",
            "--database",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .success();
}

/// Promoted Phase 11 target: the final CLI exposes only supported adapters.
#[test]
fn target_init_help_removes_the_legacy_adapter_flag() {
    Command::cargo_bin("baron")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--agent").not());
}

/// Promoted Phase 8 target: Codex is a thin native projection over the
/// canonical `.baron/core` runtime and exposes a discoverable bridge.
#[test]
fn target_codex_cli_install_is_a_thin_core_bridge() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("codex-bridge");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".agents/skills/baron-engine/SKILL.md").is_file());
    assert!(!repo.join(".codex/skills/superpowers/SKILL.md").exists());
}

/// Promoted Phase 9 target: Claude is a thin native projection over the same
/// canonical Core and exposes a discoverable bridge.
#[test]
fn target_claude_cli_install_is_a_thin_core_bridge() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("claude-bridge");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");

    assert!(repo
        .join(".baron/core/skills/superpowers/SKILL.md")
        .is_file());
    assert!(repo.join(".claude/skills/baron-engine/SKILL.md").is_file());
    assert!(!repo.join(".claude/skills/superpowers/SKILL.md").exists());
}

/// Promoted Phase 11 target: an unsupported legacy context flag is rejected
/// instead of selecting an active adapter.
#[test]
fn target_generic_context_target_is_not_active() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("generic-context");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();

    Command::cargo_bin("baron")
        .unwrap()
        .args([
            "context",
            repo.to_str().unwrap(),
            "--agent",
            "--vault",
            vault.to_str().unwrap(),
        ])
        .assert()
        .failure();
}
