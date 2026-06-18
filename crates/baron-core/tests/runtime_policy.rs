use std::fs;

use baron_core::capability::{
    check_capabilities, record_runtime_execution, register_provider, runtime_backend_report,
    BackendSafety, CapabilityExecutionEvidence, CapabilityProvider, CheckOptions, Presence,
    ProviderKind, Requirement,
};
use baron_core::config::{initialize_project, AdapterKind};
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
    initialize_project(&repo, AdapterKind::Generic, &vault).unwrap();
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

    let report = runtime_backend_report(&repo, AdapterKind::Generic).unwrap();

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
fn safe_present_provider_still_requires_execution_evidence_before_completion() {
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
        }],
    )
    .unwrap();
    let after = runtime_backend_report(&repo, AdapterKind::Codex).unwrap();
    assert!(after.passed);
    assert_eq!(after.providers[0].execution_evidence, Presence::Present);
}

#[test]
fn missing_optional_backend_degrades_without_blocking_release_gate() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("demo");
    let vault = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Generic, &vault).unwrap();
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
            adapter: AdapterKind::Generic,
            capability: None,
            allow_network: false,
        },
    )
    .unwrap();

    let report = runtime_backend_report(&repo, AdapterKind::Generic).unwrap();

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
