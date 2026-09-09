use std::fs;
use std::path::Path;

use assert_cmd::Command;
use serde_json::{json, Value};
use tempfile::tempdir;

fn init(repo: &Path, vault: &Path, adapter: &str) {
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

fn run_prepare(repo: &Path, adapter: &str, payload: Value) -> assert_cmd::assert::Assert {
    Command::cargo_bin("baron")
        .unwrap()
        .args(["control-plane", "prepare", "--adapter", adapter, "--json"])
        .current_dir(repo)
        .write_stdin(serde_json::to_vec(&payload).unwrap())
        .assert()
}

fn packet(assert: assert_cmd::assert::Assert) -> Value {
    let output = assert.success().get_output().stdout.clone();
    serde_json::from_slice(&output).expect("prepare stdout must be one JSON packet")
}

fn error_envelope(assert: assert_cmd::assert::Assert, exit_code: i32, code: &str) -> Value {
    let output = assert
        .failure()
        .code(exit_code)
        .stderr(predicates::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let envelope: Value = serde_json::from_slice(&output).expect("error stdout must be JSON");
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["error_code"], code);
    envelope
}

#[test]
fn explicit_codex_and_claude_prepare_packets_use_the_same_project_identity() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-adapters");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    init(&repo, &vault, "--claude");

    let payload = json!({
        "schema_version": 1,
        "task": "Implement login safely for tiếng Việt users; preserve \"quoted\" text.\nNext line.",
        "session_id": "host-session-1",
        "request_id": "request-1"
    });
    let codex = packet(run_prepare(&repo, "codex", payload.clone()));
    let claude = packet(run_prepare(&repo, "claude", payload));

    assert_eq!(codex["ok"], true);
    assert_eq!(claude["ok"], true);
    assert_eq!(codex["adapter"], "codex");
    assert_eq!(claude["adapter"], "claude");
    assert_eq!(codex["project_id"], claude["project_id"]);
    assert_eq!(codex["session_id"], "host-session-1");
    assert_eq!(codex["request_id"], "request-1");
    assert_eq!(codex["schema_version"], 1);
    assert!(codex["task"]["id"].as_str().unwrap().starts_with("task-"));
    assert!(codex["context"]["text"].as_str().unwrap().chars().count() <= 8_000);
    assert!(serde_json::to_vec(&codex).unwrap().len() <= 32_000);
    assert!(codex["route"]["selected_skills"]
        .as_array()
        .unwrap()
        .iter()
        .any(|skill| skill["name"] == "superpowers"
            && skill["path"] == ".baron/core/skills/superpowers"));
}

#[test]
fn prepare_repeated_input_keeps_deterministic_identity_and_route_fields() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-deterministic");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    let payload = json!({
        "schema_version": 1,
        "task": "review API contract and test the response shape",
        "session_id": "session-deterministic",
        "request_id": "request-deterministic"
    });

    let first = packet(run_prepare(&repo, "codex", payload.clone()));
    let second = packet(run_prepare(&repo, "codex", payload));
    assert_eq!(first["project_id"], second["project_id"]);
    assert_eq!(first["task"]["id"], second["task"]["id"]);
    assert_eq!(first["route"], second["route"]);
    assert_eq!(first["verification"], second["verification"]);
}

#[test]
fn prepare_projects_interrupted_continuity_confirmed_intent_and_required_route() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-resume");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    fs::create_dir_all(repo.join("docs/baron/plans")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/continuity")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    fs::write(
        repo.join("docs/baron/plans/CURRENT.md"),
        "# Current Plan\n\n- Title: implement login\n- Status: `interrupted`\n- Next action: resume login verification\n",
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/continuity/CURRENT.md"),
        "# Baron Continuity Resume\n\n- Current task: `implement login`\n- Next action: resume login verification\n",
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        "# Recovery\n\n- Outcome: `interrupted`\n\n## Safe Next Action\n\nresume login verification\n",
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/harness/CURRENT_INTENT.md"),
        "# Baron Intent Brief\n\n- ID: `intent-login`\n- Title: Implement login\n- Confirmation: `confirmed`\n\n## Constraints\n\n- preserve existing auth behavior\n\n## Non-Goals\n\n- no provider migration\n",
    )
    .unwrap();

    let packet = packet(run_prepare(
        &repo,
        "codex",
        json!({"schema_version": 1, "task": "Implement login safely"}),
    ));
    assert_eq!(packet["task"]["resumed"], true);
    assert_eq!(packet["continuity"]["resumed"], true);
    assert_eq!(
        packet["continuity"]["safe_next_action"],
        "resume login verification"
    );
    assert_eq!(packet["intent"]["confirmed"], true);
    assert!(packet["route"]["selected_skills"]
        .as_array()
        .unwrap()
        .iter()
        .any(|skill| skill["name"] == "vibe-security-scan"));
    assert_eq!(packet["risk"], "high");
    assert_eq!(packet["verification"]["required_trace_tier"], "detailed");
}

#[test]
fn prepare_reports_confirmation_blocker_without_fabricating_intent() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-confirmation");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--claude");

    let packet = packet(run_prepare(
        &repo,
        "claude",
        json!({"schema_version": 1, "task": "decide which authorization policy we should use"}),
    ));
    assert!(packet["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["code"] == "intent_confirmation_required"));
    assert_eq!(packet["intent"]["confirmed"], false);
}

#[test]
fn prepare_returns_stable_errors_and_never_executes_task_text() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-errors");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    error_envelope(
        run_prepare(
            &repo,
            "agent",
            json!({"schema_version": 1, "task": "$(touch should-not-exist) ; `echo unsafe` | >"}),
        ),
        3,
        "unsupported_adapter",
    );
    assert!(!repo.join("should-not-exist").exists());

    error_envelope(
        run_prepare(&repo, "codex", json!({"schema_version": 1, "task": ""})),
        2,
        "invalid_input",
    );

    let missing_project = temp.path().join("missing-project");
    fs::create_dir_all(&missing_project).unwrap();
    error_envelope(
        run_prepare(
            &missing_project,
            "codex",
            json!({"schema_version": 1, "task": "inspect this project"}),
        ),
        4,
        "project_state",
    );

    error_envelope(
        run_prepare(
            &repo,
            "codex",
            json!({"schema_version": 1, "task": "x".repeat(200_000)}),
        ),
        2,
        "invalid_input",
    );
}
