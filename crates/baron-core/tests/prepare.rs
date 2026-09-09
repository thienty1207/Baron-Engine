use std::fs;

use baron_core::config::{
    initialize_project, initialize_project_with_options, AdapterKind, ProjectPlatform,
};
use baron_core::prepare::{
    decode_request, prepare, PrepareError, PrepareErrorCode, PrepareRequestV1,
    PREPARE_MAX_INPUT_BYTES,
};
use serde_json::json;
use tempfile::tempdir;

// Portability: prepare protocol fixtures are cross-platform and avoid native
// shell, permission, or adapter-hook mechanics.

fn initialized_project(adapter: AdapterKind) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-core");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, adapter, &vault).unwrap();
    (temp, repo)
}

fn initialized_database_project() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("prepare-database");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project_with_options(
        &repo,
        Some(AdapterKind::Codex),
        &vault,
        Some(ProjectPlatform::Database),
    )
    .unwrap();
    (temp, repo)
}

#[test]
fn request_decoder_accepts_unknown_fields_but_enforces_version_and_bounds() {
    let request = decode_request(
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "task": "tiếng Việt\nquotes \" and shell $()",
            "future_field": "ignored"
        }))
        .unwrap()
        .as_slice(),
    )
    .unwrap();
    assert_eq!(request.schema_version, 1);
    assert!(request.task.contains("$()"));

    let unsupported = decode_request(br#"{"schema_version":2,"task":"inspect"}"#)
        .expect_err("unsupported protocol version must fail closed");
    assert_eq!(unsupported.code, PrepareErrorCode::InvalidInput);

    let oversized = vec![b'x'; PREPARE_MAX_INPUT_BYTES + 1];
    let oversized = decode_request(&oversized).expect_err("oversized input must be bounded");
    assert_eq!(oversized.code, PrepareErrorCode::InvalidInput);
}

#[test]
fn error_codes_have_stable_envelopes_and_exit_codes() {
    let cases = [
        (PrepareErrorCode::InvalidInput, "invalid_input", 2),
        (
            PrepareErrorCode::UnsupportedAdapter,
            "unsupported_adapter",
            3,
        ),
        (PrepareErrorCode::ProjectState, "project_state", 4),
        (PrepareErrorCode::Internal, "internal", 5),
    ];

    for (code, expected_name, expected_exit) in cases {
        let error = PrepareError {
            code,
            message: "fixture failure".to_string(),
            details: Default::default(),
        };
        assert_eq!(error.exit_code(), expected_exit);
        let envelope = error.envelope();
        assert_eq!(envelope.schema_version, 1);
        assert!(!envelope.ok);
        assert_eq!(envelope.error.error_code, expected_name);
    }
}

#[test]
fn prepare_uses_explicit_adapter_even_when_project_active_adapter_differs() {
    let (_temp, repo) = initialized_project(AdapterKind::Claude);
    let request = PrepareRequestV1 {
        schema_version: 1,
        task: "review the current API contract".to_string(),
        session_id: Some("session-1".to_string()),
        request_id: Some("request-1".to_string()),
    };

    let packet = prepare(request, "codex", &repo, None).unwrap();
    assert!(packet.ok);
    assert_eq!(packet.adapter, "codex");
    assert_eq!(packet.session_id.as_deref(), Some("session-1"));
    assert_eq!(packet.request_id.as_deref(), Some("request-1"));
    assert!(packet
        .route
        .selected_skills
        .iter()
        .any(|skill| skill.path == ".baron/core/skills/superpowers"));
}

#[test]
fn prepare_does_not_create_parallel_plan_intent_or_recovery_state() {
    let (temp, repo) = initialized_project(AdapterKind::Codex);
    let request = PrepareRequestV1 {
        schema_version: 1,
        task: "inspect the current repository".to_string(),
        session_id: None,
        request_id: None,
    };

    let _packet = prepare(request, "claude", &repo, None).unwrap();
    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
    assert!(!repo.join("docs/baron/harness/CURRENT_INTENT.md").exists());
    assert!(!repo
        .join("docs/baron/continuity/CURRENT_RECOVERY.md")
        .exists());
    assert!(temp.path().join("Vault").exists());
}

#[test]
fn prepare_v1_consumes_profile_aware_database_route() {
    let (_temp, repo) = initialized_database_project();
    let request = PrepareRequestV1 {
        schema_version: 1,
        task: "review relational schema constraints".to_string(),
        session_id: None,
        request_id: None,
    };

    let packet = prepare(request, "codex", &repo, None).unwrap();

    assert_eq!(packet.schema_version, 1);
    assert_eq!(packet.profile.platform, "database");
    assert!(packet
        .route
        .selected_skills
        .iter()
        .any(|skill| skill.name == "database-engineering"));
    assert!(packet.route.explanation.contains("schema-integrity"));
    assert!(packet.route.explanation.contains("work_shape="));
}
