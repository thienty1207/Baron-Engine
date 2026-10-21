use std::fs;
use std::path::Path;

use assert_cmd::Command;
use baron_core::continuity::record_continuity_checkpoint_for_operation;
use baron_core::intent::{record_intent_for_operation, IntentBriefInput};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::{interrupt_plan_for_identity, start_or_resume_plan_for_identity};
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
fn prepare_resumes_exact_interrupted_operation_and_rejects_unbound_current_intent() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-resume");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    fs::create_dir_all(repo.join("docs/baron/continuity")).unwrap();
    fs::create_dir_all(repo.join("docs/baron/harness")).unwrap();
    let vault_context = baron_core::vault::ensure_vault(&vault, &repo).unwrap();
    let task = "implement the authorization policy after deciding which policy we should use";
    let session_id = "prepare-login-session";
    let request_id = "prepare-login-request";
    let identity = LifecycleIdentity::resolve(
        &vault_context.project_id,
        task,
        SupportedAdapter::Codex,
        Some(session_id),
        Some(request_id),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault_context, task, &identity).unwrap();
    interrupt_plan_for_identity(
        &repo,
        &vault_context,
        "resume authorization policy implementation",
        &identity,
    )
    .unwrap();
    record_continuity_checkpoint_for_operation(
        &repo,
        &vault_context,
        "resume authorization policy implementation",
        &OperationContext::from_identity(&identity),
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/continuity/CURRENT_RECOVERY.md"),
        "# Recovery\n\n- Outcome: `interrupted`\n\n## Safe Next Action\n\nresume authorization policy implementation\n",
    )
    .unwrap();
    fs::write(
        repo.join("docs/baron/harness/CURRENT_INTENT.md"),
        "# Baron Intent Brief\n\n- ID: `intent-authorization`\n- Title: Decide authorization policy\n- Confirmation: `confirmed`\n\n## Constraints\n\n- preserve existing auth behavior\n\n## Non-Goals\n\n- no provider migration\n",
    )
    .unwrap();

    let packet = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": task,
            "session_id": session_id,
            "request_id": request_id
        }),
    ));
    assert_eq!(packet["task"]["resumed"], true);
    assert_eq!(packet["continuity"]["resumed"], true);
    assert_eq!(
        packet["continuity"]["safe_next_action"],
        "Interrupted: resume authorization policy implementation"
    );
    assert_eq!(packet["intent"]["confirmed"], false);
    assert!(packet["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["code"] == "intent_confirmation_required"));
    assert!(packet["route"]["mandatory_agents"]
        .as_array()
        .unwrap()
        .iter()
        .any(|agent| agent["name"] == "security-auditor"));
    assert_eq!(packet["risk"], "high");
    assert_eq!(packet["verification"]["required_trace_tier"], "detailed");
}

#[test]
fn prepare_reads_exact_operation_intent_when_current_projects_another_session() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-operation-intent");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");
    let vault_context = baron_core::vault::ensure_vault(&vault, &repo).unwrap();
    let task = "implement the authorization policy after deciding which policy we should use";
    let session_a = "authorization-session-a";
    let request_a = "authorization-request-a";
    let session_b = "authorization-session-b";
    let request_b = "authorization-request-b";
    let identity_a = LifecycleIdentity::resolve(
        &vault_context.project_id,
        task,
        SupportedAdapter::Codex,
        Some(session_a),
        Some(request_a),
    )
    .unwrap();
    let identity_b = LifecycleIdentity::resolve(
        &vault_context.project_id,
        task,
        SupportedAdapter::Claude,
        Some(session_b),
        Some(request_b),
    )
    .unwrap();
    let intent_input = |title: &str, confirmed| IntentBriefInput {
        title: title.to_string(),
        current_behavior: "Authorization policy is not yet selected.".to_string(),
        target_behavior: "Implement only the explicitly selected policy.".to_string(),
        scope: "Authorization decision and implementation.".to_string(),
        non_goals: vec!["Do not broaden permissions.".to_string()],
        constraints: vec!["Preserve current authentication behavior.".to_string()],
        decisions: vec!["Policy selection is operation-specific.".to_string()],
        required_proof: "Authorization tests pass.".to_string(),
        unknowns: vec!["Deployment policy remains unknown.".to_string()],
        confirmed,
    };
    record_intent_for_operation(
        &repo,
        &vault_context,
        task,
        &identity_a,
        intent_input("Authorization choice A", true),
    )
    .unwrap();
    record_intent_for_operation(
        &repo,
        &vault_context,
        task,
        &identity_b,
        intent_input("Authorization choice B", false),
    )
    .unwrap();

    let current = fs::read_to_string(repo.join("docs/baron/harness/CURRENT_INTENT.md")).unwrap();
    assert!(current.contains(&format!("- Operation ID: `{}`", identity_b.operation_id())));
    let packet_a = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": task,
            "session_id": session_a,
            "request_id": request_a
        }),
    ));
    assert_eq!(packet_a["intent"]["title"], "Authorization choice A");
    assert_eq!(packet_a["intent"]["confirmed"], true);
    assert!(!packet_a["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["code"] == "intent_confirmation_required"));

    let packet_b = packet(run_prepare(
        &repo,
        "claude",
        json!({
            "schema_version": 1,
            "task": task,
            "session_id": session_b,
            "request_id": request_b
        }),
    ));
    assert_eq!(packet_b["intent"]["title"], "Authorization choice B");
    assert_eq!(packet_b["intent"]["confirmed"], false);
    assert!(packet_b["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["code"] == "intent_confirmation_required"));
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
