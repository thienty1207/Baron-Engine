use std::fs;
use std::time::Duration;

use baron_core::capability::{
    check_capabilities, record_runtime_execution, register_provider, runtime_backend_report,
    runtime_backend_report_for_operation, BackendSafety, CapabilityExecutionEvidence,
    CapabilityProvider, CheckOptions, Presence, ProviderKind, Requirement,
};
use baron_core::config::{initialize_project, AdapterKind};
use baron_core::execution_receipt::{
    execute_command_with_context, ExecutionRequest, ReceiptContext,
};
use baron_core::operation::{OperationContext, SupportedAdapter};
use tempfile::tempdir;

fn provider(name: &str, capability: &str, command: &str, required: bool) -> CapabilityProvider {
    CapabilityProvider {
        name: name.to_string(),
        capability: capability.to_string(),
        kind: ProviderKind::Cli,
        requirement: if required {
            Requirement::Required
        } else {
            Requirement::Optional
        },
        command: Some(command.to_string()),
        scan_target: None,
        adapters: Vec::new(),
        description: "Provides tool-backed verification for Baron.".to_string(),
    }
}

#[test]
fn runtime_report_flags_unsafe_backend_and_missing_execution_evidence() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    register_provider(
        &repo,
        provider(
            "danger-shell",
            "release-cleanup",
            "powershell -EncodedCommand ZABhAG4AZwBlAHIA",
            true,
        ),
    )
    .unwrap();

    let report = runtime_backend_report(&repo, AdapterKind::Codex).unwrap();

    assert!(!report.passed);
    assert_eq!(report.providers[0].safety, BackendSafety::Unsafe);
    assert_eq!(report.providers[0].execution_evidence, Presence::Missing);
    assert!(report
        .blocking_gaps
        .iter()
        .any(|gap| gap.contains("unsafe backend")));
    assert!(report
        .blocking_gaps
        .iter()
        .any(|gap| gap.contains("execution evidence")));
}

#[test]
fn persisted_summary_evidence_stays_diagnostic_until_a_trusted_receipt_exists() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    register_provider(&repo, provider("cargo-test", "test-suite", "cargo", true)).unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    let before = runtime_backend_report(&repo, AdapterKind::Codex).unwrap();
    assert!(!before.passed);
    assert!(before
        .blocking_gaps
        .iter()
        .any(|gap| gap.contains("execution evidence")));

    record_runtime_execution(
        &repo,
        &[CapabilityExecutionEvidence {
            capability: "test-suite".to_string(),
            provider: "cargo-test".to_string(),
            summary: "cargo test --workspace --all-targets passed".to_string(),
            ..Default::default()
        }],
    )
    .unwrap();
    let after = runtime_backend_report(&repo, AdapterKind::Codex).unwrap();
    assert!(!after.passed);
    assert_eq!(after.providers[0].execution_evidence, Presence::Missing);
}

#[test]
fn missing_optional_backend_degrades_without_blocking_release_gate() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    register_provider(
        &repo,
        provider(
            "optional-lint",
            "lint",
            "baron-definitely-missing-lint",
            false,
        ),
    )
    .unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    let report = runtime_backend_report(&repo, AdapterKind::Codex).unwrap();

    assert!(report.passed);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning.contains("optional")));
    assert!(report
        .recommendations
        .iter()
        .any(|recommendation| recommendation.contains("safe local")));
}

#[test]
fn runtime_report_requires_the_exact_current_operation_receipt() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
    #[cfg(windows)]
    let provider_command = "cmd";
    #[cfg(not(windows))]
    let provider_command = "true";
    register_provider(
        &repo,
        provider("test-runner", "test-suite", provider_command, true),
    )
    .unwrap();
    check_capabilities(
        &repo,
        CheckOptions {
            adapter: AdapterKind::Codex,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    #[cfg(windows)]
    let (executable, arguments) = ("cmd", vec!["/C".to_string(), "exit 0".to_string()]);
    #[cfg(not(windows))]
    let (executable, arguments) = ("sh", vec!["-c".to_string(), "exit 0".to_string()]);
    execute_command_with_context(
        ExecutionRequest {
            capability: "test-suite".to_string(),
            provider: "test-runner".to_string(),
            executable: executable.to_string(),
            arguments,
            working_directory: repo.clone(),
            timeout: Duration::from_secs(5),
        },
        ReceiptContext::new(
            "task-a",
            "operation-a",
            "codex",
            "session-a",
            "request-a",
            "capability_execution",
        ),
    )
    .unwrap();

    let operation_a = OperationContext::new(SupportedAdapter::Codex)
        .with_task_id("task-a")
        .with_operation_id("operation-a")
        .with_session_id("session-a")
        .with_request_id("request-a");
    let report_a = runtime_backend_report_for_operation(&repo, &operation_a).unwrap();
    assert_eq!(report_a.providers[0].execution_evidence, Presence::Present);
    assert!(report_a.passed);

    let operation_b = operation_a.clone().with_operation_id("operation-b");
    let report_b = runtime_backend_report_for_operation(&repo, &operation_b).unwrap();
    assert_eq!(report_b.providers[0].execution_evidence, Presence::Missing);
    assert!(!report_b.passed);
}
