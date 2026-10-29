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
fn prepare_emits_bounded_security_audit_plan_without_claiming_execution() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-security-audit");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    let payload = json!({
        "schema_version": 1,
        "task": "implement request validation in src/api/users.ts",
        "session_id": "security-audit-session",
        "request_id": "security-audit-request"
    });
    let typescript_packet = packet(run_prepare(&repo, "codex", payload));
    let security_audit = &typescript_packet["security_audit"];

    assert_eq!(security_audit["execution_status"], "not_run");
    assert_eq!(security_audit["checks_run"], json!([]));
    assert_eq!(security_audit["audit_mode"], "focused");
    assert!(security_audit["scope"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == "src/api/users.ts"));
    assert!(security_audit["detected_languages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|language| language == "typescript"));
    assert_eq!(
        security_audit["language_coverage"][0]["language"],
        "typescript"
    );
    assert_eq!(
        security_audit["language_coverage"][0]["confidence"],
        "planned"
    );
    assert_eq!(
        security_audit["language_coverage"][0]["in_scope_paths"],
        json!(["src/api/users.ts"])
    );
    assert_eq!(
        security_audit["language_coverage"][0]["lifecycle_surfaces"],
        json!([
            "inputs_and_entrypoints",
            "validation_and_transforms",
            "authorization_and_ownership",
            "sensitive_sinks_and_persistence",
            "errors_logging_and_outputs",
            "configuration_and_deployment"
        ])
    );
    assert!(!security_audit["language_coverage"][0]["coverage_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|gap| gap == "lifecycle_surface_mapping_pending_spec_08_03"));
    assert_eq!(security_audit["workflow_schema_version"], 1);
    assert_eq!(
        security_audit["required_phases"],
        json!([
            "reconnaissance",
            "coverage_led_hunting",
            "candidate_validation",
            "structured_output",
            "independent_record_verification",
            "reporting"
        ])
    );
    assert_eq!(security_audit["minimum_candidate_verifiers"], 1);
    assert_eq!(
        security_audit["validator_command"],
        "baron security-audit validate-run"
    );
    assert_eq!(
        security_audit["remediation_policy"]["validator_command"],
        "baron security-audit validate-remediation"
    );
    assert_eq!(security_audit["remediation_policy"]["max_cycles"], 3);
    assert_eq!(
        security_audit["remediation_policy"]["execution_status"],
        "not_run"
    );
    assert_eq!(
        security_audit["remediation_policy"]["auto_fix_authorized"],
        false
    );
    assert_eq!(
        security_audit["remediation_policy"]["closure_authorized"],
        false
    );
    assert_eq!(
        security_audit["remediation_policy"]["blocker"],
        "sandbox_backend_unverified"
    );
    assert_eq!(
        security_audit["sandbox_readiness"]["execution_allowed"],
        false
    );
    assert_eq!(
        security_audit["sandbox_readiness"]["controls"]
            .as_array()
            .unwrap()
            .len(),
        10
    );
    assert!(security_audit["sandbox_readiness"]["controls"]
        .as_array()
        .unwrap()
        .iter()
        .all(|control| control["enforced"] == false));
    assert!(security_audit["required_validation"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "independently_re_read_confirmed_records"));
    assert!(!security_audit["capability_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "audit_workflow_pending_spec_08_03"));
    assert!(security_audit["capability_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "sandbox_backend_unverified"));
    assert!(typescript_packet["verification"]["mandatory_gates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|agent| agent != "security-auditor"));

    let mut legacy_packet = typescript_packet.clone();
    legacy_packet
        .as_object_mut()
        .unwrap()
        .remove("security_audit");
    let decoded: baron_core::prepare::PreparePacketV1 =
        serde_json::from_value(legacy_packet).expect("older prepare packets remain readable");
    assert!(decoded.security_audit.is_none());

    for (index, (path, language)) in [
        ("src/handler.py", "python"),
        ("src/handler.cpp", "cpp"),
        ("src/handler.h", "cpp"),
        ("src/Handler.java", "java"),
        ("src/handler.rs", "rust"),
        ("src/handler.go", "go"),
        ("src/handler.ts", "typescript"),
        ("src/module.mts", "typescript"),
        ("src/Handler.cs", "csharp"),
    ]
    .into_iter()
    .enumerate()
    {
        let packet = packet(run_prepare(
            &repo,
            "codex",
            json!({
                "schema_version": 1,
                "task": format!("implement request validation in {path}"),
                "session_id": format!("language-session-{index}"),
                "request_id": format!("language-request-{index}")
            }),
        ));
        assert!(
            packet["security_audit"]["detected_languages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == language),
            "PreparePacket did not identify {language} for {path}"
        );
    }

    let broad = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": "fix authorization bypass in src/auth.ts",
            "session_id": "security-boundary-session",
            "request_id": "security-boundary-request"
        }),
    ));
    assert_eq!(broad["security_audit"]["audit_mode"], "deep");
    assert_eq!(broad["security_audit"]["minimum_candidate_verifiers"], 2);
    assert!(broad["verification"]["mandatory_gates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|agent| agent == "security-auditor"));

    let unknown_language = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": "implement a backend feature",
            "session_id": "unknown-language-session",
            "request_id": "unknown-language-request"
        }),
    ));
    assert!(unknown_language["security_audit"]["detected_languages"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(unknown_language["security_audit"]["coverage_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "no_supported_language_evidence_in_scope"));
    assert!(unknown_language["security_audit"]["capability_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "language_coverage_unknown"));

    let out_of_scope = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": "implement request validation in ../outside.rs",
            "session_id": "outside-scope-session",
            "request_id": "outside-scope-request"
        }),
    ));
    assert_eq!(
        out_of_scope["security_audit"]["scope"],
        json!(["planned_application_or_security_boundary"])
    );
    assert!(out_of_scope["security_audit"]["detected_languages"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(out_of_scope["security_audit"]["coverage_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "no_supported_language_evidence_in_scope"));
}

#[test]
fn prepare_discovers_all_languages_in_a_mixed_monorepo() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-mixed-language-repo");
    let vault = temp.path().join("Vault");
    let files = [
        ("services/python/src/app.py", "def app(): return 'ok'\n"),
        ("services/python/requirements.txt", "fastapi==0.115.0\n"),
        (
            "services/cpp/CMakeLists.txt",
            "project(cpp_service LANGUAGES CXX)\n",
        ),
        ("services/cpp/src/main.cpp", "int main() { return 0; }\n"),
        (
            "services/java/pom.xml",
            "<project><artifactId>api</artifactId></project>\n",
        ),
        ("services/java/src/Main.java", "class Main {}\n"),
        (
            "services/rust/Cargo.toml",
            "[package]\nname = \"service\"\nversion = \"0.1.0\"\n",
        ),
        ("services/rust/src/main.rs", "fn main() {}\n"),
        (
            "services/go/go.mod",
            "module example.com/service\n\ngo 1.23\n",
        ),
        ("services/go/main.go", "package main\nfunc main() {}\n"),
        (
            "services/typescript/package.json",
            "{\"dependencies\": {\"typescript\": \"^5.0.0\"}}\n",
        ),
        ("services/typescript/tsconfig.json", "{}\n"),
        (
            "services/typescript/src/app.ts",
            "export const app = true;\n",
        ),
        (
            "services/dotnet/Api.csproj",
            "<Project Sdk=\"Microsoft.NET.Sdk.Web\"></Project>\n",
        ),
        ("services/dotnet/Program.cs", "class Program {}\n"),
        ("generated/Generated.java", "class Generated {}\n"),
        ("vendor/library/ignored.go", "package ignored\n"),
        ("tests/fixtures/example.py", "def fixture(): pass\n"),
        ("target/debug/generated.rs", "fn generated() {}\n"),
    ];
    for (path, content) in files {
        let path = repo.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    init(&repo, &vault, "--codex");

    let packet = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": "implement a request handler in services/python/src/app.py",
            "session_id": "mixed-language-session",
            "request_id": "mixed-language-request"
        }),
    ));
    let security_audit = &packet["security_audit"];
    let mut detected = security_audit["detected_languages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|language| language.as_str().unwrap())
        .collect::<Vec<_>>();
    detected.sort_unstable();

    assert_eq!(
        detected,
        [
            "cpp",
            "csharp",
            "go",
            "java",
            "python",
            "rust",
            "typescript"
        ]
    );
    assert_eq!(
        security_audit["language_coverage"]
            .as_array()
            .unwrap()
            .len(),
        7
    );
    let languages = security_audit["language_coverage"].as_array().unwrap();
    let python_coverage = languages
        .iter()
        .find(|coverage| coverage["language"] == "python")
        .unwrap();
    let rust_coverage = languages
        .iter()
        .find(|coverage| coverage["language"] == "rust")
        .unwrap();
    assert_eq!(python_coverage["in_scope"], true);
    assert_eq!(
        python_coverage["in_scope_paths"],
        json!(["services/python/src/app.py"])
    );
    assert!(python_coverage["coverage_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|gap| gap == "framework_detection_catalog_non_exhaustive_requires_validation"));
    assert!(security_audit["coverage_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|gap| gap == "framework_detection_catalog_non_exhaustive_requires_validation"));
    assert!(security_audit["capability_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|blocker| blocker == "language_coverage_incomplete"));
    assert_eq!(rust_coverage["in_scope"], false);
    assert_eq!(rust_coverage["in_scope_paths"], json!([]));
    assert_eq!(security_audit["execution_status"], "not_run");
    assert_eq!(security_audit["checks_run"], json!([]));
    assert!(security_audit["scope_exclusions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| rule["path"] == "tests/fixtures" && rule["rule_id"] == "fixture_data"));
    assert!(security_audit["stack_signals"]
        .as_array()
        .unwrap()
        .iter()
        .any(|signal| signal["name"] == "fastapi" && signal["kind"] == "framework"));
    assert!(!security_audit["capability_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "audit_workflow_pending_spec_08_03"));
}

#[test]
fn prepare_omits_security_audit_plan_for_docs_only_work() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-docs-only");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    init(&repo, &vault, "--codex");

    let packet = packet(run_prepare(
        &repo,
        "codex",
        json!({
            "schema_version": 1,
            "task": "update README installation examples",
            "session_id": "docs-only-session",
            "request_id": "docs-only-request"
        }),
    ));

    assert!(packet.get("security_audit").is_none());
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
