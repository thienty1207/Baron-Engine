use baron_core::capability::{
    check_capabilities, register_provider, CapabilityExecutionEvidence, CapabilityProvider,
    CheckOptions, ProviderKind, Requirement,
};
use baron_core::config::AdapterKind;
use baron_core::operation::{OperationContext, SupportedAdapter};
use baron_core::security_audit::SHARED_ATTACK_CLASSES;
use baron_core::security_audit_safety::{assess_audit_sandbox, inspect_audit_capabilities};
use baron_core::security_audit_workflow::{
    AuditCandidateReviewV1, AuditCandidateV1, AuditCoverageDisposition, AuditCoverageLedgerV1,
    AuditCoverageUnitV1, AuditEvidenceRefV1, AuditExecutionOutcome, AuditExecutionReceiptV1,
    AuditFailureClassV1, AuditFailureV1, AuditFindingVerdictV1, AuditMode, AuditPhase,
    AuditPhaseReceiptV1, AuditRecordReviewV1, AuditRunBindingV1, AuditRunStatusV1, AuditRunV1,
    AuditSeverity, SECURITY_AUDIT_LIFECYCLE_SURFACES, SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
};
use serde_json::Value;
use tempfile::TempDir;

fn binding() -> AuditRunBindingV1 {
    AuditRunBindingV1 {
        project_id: "project-safety-test".to_string(),
        source_revision: "a".repeat(40),
        source_dirty: false,
        dirty_state_sha256: None,
        scope: vec!["src".to_string()],
        languages: vec!["rust".to_string()],
        profile: "backend".to_string(),
        task_id: "task-safety-test".to_string(),
        task_state_sha256: "b".repeat(64),
        operation_id: "operation-safety-test".to_string(),
    }
}

fn evidence() -> AuditEvidenceRefV1 {
    AuditEvidenceRefV1 {
        path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 12,
        content_sha256: "c".repeat(64),
    }
}

fn ledger(disposition: AuditCoverageDisposition) -> AuditCoverageLedgerV1 {
    let mut units = Vec::new();
    for lifecycle in SECURITY_AUDIT_LIFECYCLE_SURFACES {
        for attack_class in SHARED_ATTACK_CLASSES {
            units.push(AuditCoverageUnitV1 {
                unit_id: format!("unit-{lifecycle}-{attack_class}"),
                scope_path: "src/api.rs".to_string(),
                subsystem: "api".to_string(),
                boundary: "request-to-database".to_string(),
                lifecycle: lifecycle.to_string(),
                language: "rust".to_string(),
                attack_class: attack_class.to_string(),
                disposition,
                reason: (disposition != AuditCoverageDisposition::Covered)
                    .then(|| "explicitly deferred for validation".to_string()),
                evidence_refs: vec!["evidence://repo/src/api.rs#L10-L12".to_string()],
            });
        }
    }
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    AuditCoverageLedgerV1 {
        schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        units,
    }
}

fn capability_fixture(probe_presence: bool) -> (TempDir, OperationContext) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let baron = root.join(".baron");
    std::fs::create_dir_all(&baron).unwrap();
    let project_id = baron_core::identity::project_id_for_path(root).unwrap();
    std::fs::write(
        baron.join("project.toml"),
        format!(
            "schema_version = 4\nproject_id = \"{project_id}\"\nidentity_binding = \"{}\"\nproject_slug = \"safety-test\"\nadapters = [\"codex\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
            "b".repeat(64)
        ),
    )
    .unwrap();
    register_provider(
        root,
        CapabilityProvider {
            name: "synthetic-agent".to_string(),
            capability: "security-audit".to_string(),
            kind: ProviderKind::AgentAdapter,
            requirement: Requirement::Required,
            command: None,
            scan_target: None,
            adapters: vec![AdapterKind::Codex],
            description: "synthetic safety fixture provider".to_string(),
        },
    )
    .unwrap();
    if probe_presence {
        check_capabilities(
            root,
            CheckOptions {
                adapter: AdapterKind::Codex,
                capability: Some("security-audit".to_string()),
                allow_network: false,
            },
        )
        .unwrap();
    }
    let operation = OperationContext::new(SupportedAdapter::Codex)
        .with_session_id("session-safety")
        .with_request_id("request-safety")
        .with_task_id("task-safety")
        .with_operation_id("operation-safety");
    (temp, operation)
}

fn phase_receipt(run: &AuditRunV1, phase: AuditPhase) -> AuditPhaseReceiptV1 {
    AuditPhaseReceiptV1 {
        phase,
        binding_sha256: run.binding_fingerprint().unwrap(),
        operation_id: run.binding.operation_id.clone(),
        timestamp: "2026-10-29T12:00:00Z".to_string(),
        artifact_ref: format!("audit-artifact://{}", phase.as_str()),
    }
}

fn complete_run(
    number: u64,
    disposition: AuditCoverageDisposition,
    with_finding: bool,
) -> AuditRunV1 {
    let mut run = AuditRunV1::new(
        format!("00000000-0000-4000-8000-{number:012x}"),
        binding(),
        AuditMode::Focused,
    )
    .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::Reconnaissance))
        .unwrap();
    run.set_coverage_ledger(ledger(disposition)).unwrap();

    let fingerprint = if with_finding {
        let candidate = AuditCandidateV1 {
            root_cause_key: "tenant-ownership-check".to_string(),
            attack_class: "authentication_authorization_and_ownership".to_string(),
            language: "rust".to_string(),
            subsystem: "api".to_string(),
            protected_boundary: "request-to-database".to_string(),
            lifecycle: "authorization_and_ownership".to_string(),
            source_path: "src/api.rs".to_string(),
            start_line: 10,
            end_line: 12,
            claim: "tenant ownership is not checked before reading the record".to_string(),
            author_ids: vec!["hunter".to_string()],
            evidence_refs: vec![evidence()],
        };
        Some(run.add_candidate(candidate).unwrap())
    } else {
        None
    };
    run.complete_phase(phase_receipt(&run, AuditPhase::CoverageLedHunting))
        .unwrap();

    if let Some(fingerprint) = fingerprint {
        run.add_candidate_review(
            &fingerprint,
            AuditCandidateReviewV1 {
                reviewer_id: "verifier".to_string(),
                context_id: "fresh-review-context".to_string(),
                source_revision: run.binding.source_revision.clone(),
                binding_sha256: run.binding_fingerprint().unwrap(),
                candidate_revision: 0,
                assessment: AuditFindingVerdictV1::Confirmed {
                    severity: AuditSeverity::High,
                    affected_principal_or_attacker: "authenticated user from another tenant"
                        .to_string(),
                    protected_resource: "tenant record".to_string(),
                    impact: "cross-tenant disclosure".to_string(),
                    source_path: "src/api.rs".to_string(),
                    start_line: 10,
                    end_line: 12,
                    evidence_refs: vec![evidence()],
                    recommended_correction: "enforce tenant ownership in the query".to_string(),
                    safe_verification: "test access using an isolated fixture".to_string(),
                },
            },
        )
        .unwrap();
    }
    run.complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::StructuredOutput))
        .unwrap();
    if let Some(fingerprint) = run
        .findings
        .first()
        .map(|finding| finding.fingerprint(&run.binding).unwrap())
    {
        run.record_independent_record_review(
            &fingerprint,
            AuditRecordReviewV1 {
                reviewer_id: "record-reviewer".to_string(),
                context_id: "fresh-record-context".to_string(),
                source_revision: run.binding.source_revision.clone(),
                binding_sha256: run.binding_fingerprint().unwrap(),
                candidate_revision: 0,
                candidate_fingerprint: fingerprint.clone(),
                accepted: true,
                correction_requires_revalidation: false,
                evidence_refs: vec![evidence()],
            },
        )
        .unwrap();
    }
    run.complete_phase(phase_receipt(
        &run,
        AuditPhase::IndependentRecordVerification,
    ))
    .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::Reporting))
        .unwrap();
    run
}

#[test]
fn run_states_distinguish_in_progress_complete_incomplete_blocked_and_failed() {
    for expected in ["in_progress", "complete", "incomplete", "blocked", "failed"] {
        let encoded = serde_json::to_string(expected).unwrap();
        let decoded = serde_json::from_str::<AuditRunStatusV1>(&encoded);
        assert!(
            decoded.is_ok(),
            "run state `{expected}` must be represented"
        );
    }
}

#[test]
fn failed_execution_and_timeout_never_become_negative_findings() {
    let mut failed = AuditRunV1::new(
        "00000000-0000-4000-8000-000000000001",
        binding(),
        AuditMode::Focused,
    )
    .unwrap();
    failed
        .record_execution_receipt(AuditExecutionReceiptV1 {
            operation: "target test command".to_string(),
            operation_id: failed.binding.operation_id.clone(),
            source_revision: failed.binding.source_revision.clone(),
            binding_sha256: failed.binding_fingerprint().unwrap(),
            outcome: AuditExecutionOutcome::Failed,
            timestamp: "2026-10-29T12:00:00Z".to_string(),
            artifact_ref: "audit-artifact://failed-command".to_string(),
        })
        .unwrap();
    assert_eq!(failed.status.as_str(), "failed");
    let failed_report = serde_json::to_value(failed.report().unwrap()).unwrap();
    assert!(failed_report.get("failure").is_some());
    assert!(serde_json::from_str::<AuditExecutionOutcome>("\"timed_out\"").is_ok());

    let report = failed.report().unwrap();
    assert!(!report
        .findings
        .iter()
        .any(|finding| finding.verdict.is_none()));
    assert!(report
        .residual_checks
        .iter()
        .any(|check| check.contains("failed") || check.contains("timed_out")));

    let mut timed_out = AuditRunV1::new(
        "00000000-0000-4000-8000-000000000005",
        binding(),
        AuditMode::Focused,
    )
    .unwrap();
    timed_out
        .record_execution_receipt(AuditExecutionReceiptV1 {
            operation: "target fuzz command".to_string(),
            operation_id: timed_out.binding.operation_id.clone(),
            source_revision: timed_out.binding.source_revision.clone(),
            binding_sha256: timed_out.binding_fingerprint().unwrap(),
            outcome: AuditExecutionOutcome::TimedOut,
            timestamp: "2026-10-29T12:00:00Z".to_string(),
            artifact_ref: "audit-artifact://timed-out-command".to_string(),
        })
        .unwrap();
    assert_eq!(timed_out.status.as_str(), "failed");
    assert_eq!(
        timed_out.failure.as_ref().map(|failure| failure.class),
        Some(AuditFailureClassV1::TargetCommandTimedOut)
    );
}

#[test]
fn blocked_and_interrupted_runs_cannot_resume_without_source_bound_validation() {
    let mut run = AuditRunV1::new(
        "00000000-0000-4000-8000-000000000006",
        binding(),
        AuditMode::Focused,
    )
    .unwrap();
    run.block("sandbox_unavailable").unwrap();
    assert_eq!(run.status, AuditRunStatusV1::Blocked);
    assert!(run.resume().is_err());
    assert_eq!(run.status, AuditRunStatusV1::Blocked);

    run.mark_incomplete(AuditFailureV1 {
        class: AuditFailureClassV1::Interrupted,
        code: "user_interrupted".to_string(),
    })
    .unwrap();
    assert_eq!(run.status.as_str(), "incomplete");
    assert!(run.resume().is_err());
    assert_eq!(run.status.as_str(), "incomplete");
    assert!(run.failure.is_some());
}

#[test]
fn failed_runs_require_non_sensitive_codes_and_do_not_resume_in_place() {
    let mut run = AuditRunV1::new(
        "00000000-0000-4000-8000-000000000007",
        binding(),
        AuditMode::Focused,
    )
    .unwrap();
    assert!(run
        .mark_failed(AuditFailureV1 {
            class: AuditFailureClassV1::MalformedEvidence,
            code: "token=secret-value".to_string(),
        })
        .is_err());
    run.mark_failed(AuditFailureV1 {
        class: AuditFailureClassV1::MalformedEvidence,
        code: "evidence_schema_invalid".to_string(),
    })
    .unwrap();
    assert_eq!(run.status, AuditRunStatusV1::Failed);
    assert!(run.resume().is_err());
}

#[test]
fn registered_capability_is_not_present_or_executed() {
    let (temp, operation) = capability_fixture(false);
    let usage = inspect_audit_capabilities(temp.path(), &operation, &[]).unwrap();
    assert_eq!(usage.len(), 1);
    assert!(usage[0].registered);
    assert!(!usage[0].present);
    assert!(!usage[0].executed);
    assert_eq!(
        usage[0].blocker.as_deref(),
        Some("provider_presence_unverified")
    );
}

#[test]
fn present_capability_is_not_executed_without_current_verified_receipt() {
    let (temp, operation) = capability_fixture(true);
    let usage = inspect_audit_capabilities(temp.path(), &operation, &[]).unwrap();
    assert!(usage[0].registered);
    assert!(usage[0].present);
    assert!(!usage[0].executed);
    assert_eq!(
        usage[0].blocker.as_deref(),
        Some("current_operation_execution_unverified")
    );
}

#[test]
fn stale_receipt_does_not_satisfy_current_operation() {
    let (temp, operation) = capability_fixture(true);
    let evidence = [CapabilityExecutionEvidence {
        capability: "security-audit".to_string(),
        provider: "synthetic-agent".to_string(),
        summary: "previous operation executed provider".to_string(),
        receipt_id: Some("receipt-from-previous-request".to_string()),
        task_id: Some("old-task".to_string()),
        operation_id: Some("old-operation".to_string()),
        gate_kind: Some("capability_execution".to_string()),
        session_id: Some("old-session".to_string()),
        request_id: Some("old-request".to_string()),
    }];
    let usage = inspect_audit_capabilities(temp.path(), &operation, &evidence).unwrap();
    assert!(usage[0].present);
    assert!(!usage[0].executed);
}

#[test]
fn missing_sandbox_controls_block_without_running_target() {
    let (temp, operation) = capability_fixture(false);
    let readiness = assess_audit_sandbox(temp.path(), &operation).unwrap();
    assert!(!readiness.execution_allowed);
    assert_eq!(readiness.controls.len(), 10);
    assert!(readiness.controls.iter().all(|control| !control.enforced));
    assert!(readiness
        .blockers
        .contains(&"sandbox_backend_unverified".to_string()));
}

#[test]
fn deferred_coverage_cannot_produce_clean_report() {
    let run = complete_run(2, AuditCoverageDisposition::Deferred, false);
    let report = run.report().unwrap();
    let serialized = serde_json::to_value(report).unwrap();
    assert_eq!(run.status.as_str(), "incomplete");
    assert_eq!(serialized.get("clean"), Some(&Value::Bool(false)));
}

#[test]
fn confirmed_finding_does_not_prevent_audit_completion() {
    let run = complete_run(3, AuditCoverageDisposition::Covered, true);
    let report = run.report().unwrap();
    assert_eq!(run.status, AuditRunStatusV1::Complete);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(
        serde_json::to_value(report).unwrap().get("clean"),
        Some(&Value::Bool(false))
    );
}

#[test]
fn application_task_remains_blocked_until_trusted_closure() {
    let run = complete_run(4, AuditCoverageDisposition::Covered, false);
    let report = run.report().unwrap();
    assert_eq!(
        serde_json::to_value(report)
            .unwrap()
            .get("application_task_blocked"),
        Some(&Value::Bool(true))
    );
}
