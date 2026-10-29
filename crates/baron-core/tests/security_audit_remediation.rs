use baron_core::security_audit::SHARED_ATTACK_CLASSES;
use baron_core::security_audit_remediation::{
    validate_security_audit_remediation, RemediationCheckStatusV1, RemediationFixStatusV1,
    RemediationReauditStatusV1, SecurityAuditRemediationCycleV1, SecurityAuditRemediationRequestV1,
    SecurityAuditRemediationStateV1, SecurityAuditRemediationV1,
    SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
};
use baron_core::security_audit_workflow::{
    AuditCandidateReviewV1, AuditCandidateV1, AuditCoverageDisposition, AuditCoverageLedgerV1,
    AuditCoverageUnitV1, AuditEvidenceRefV1, AuditFindingVerdictV1, AuditMode, AuditPhase,
    AuditPhaseReceiptV1, AuditRecordReviewV1, AuditRunBindingV1, AuditRunV1, AuditSeverity,
    SECURITY_AUDIT_LIFECYCLE_SURFACES, SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};

fn binding() -> AuditRunBindingV1 {
    AuditRunBindingV1 {
        project_id: "project-remediation".to_string(),
        source_revision: "a".repeat(40),
        source_dirty: true,
        dirty_state_sha256: Some("b".repeat(64)),
        scope: vec!["src".to_string()],
        languages: vec!["rust".to_string()],
        profile: "backend".to_string(),
        task_id: "task-remediation".to_string(),
        task_state_sha256: "c".repeat(64),
        operation_id: "operation-remediation".to_string(),
    }
}

fn binding_digest(binding: &AuditRunBindingV1) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(binding).unwrap()))
}

fn evidence() -> AuditEvidenceRefV1 {
    AuditEvidenceRefV1 {
        path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 20,
        content_sha256: "d".repeat(64),
    }
}

fn candidate() -> AuditCandidateV1 {
    AuditCandidateV1 {
        root_cause_key: "tenant-ownership-query".to_string(),
        attack_class: "authentication_authorization_and_ownership".to_string(),
        language: "rust".to_string(),
        subsystem: "api".to_string(),
        protected_boundary: "tenant-api-to-database".to_string(),
        lifecycle: "authorization_and_ownership".to_string(),
        source_path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 20,
        claim: "tenant ownership is not enforced before loading a record".to_string(),
        author_ids: vec!["hunter-a".to_string()],
        evidence_refs: vec![evidence()],
    }
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

fn coverage_ledger() -> AuditCoverageLedgerV1 {
    let mut units = Vec::new();
    for lifecycle in SECURITY_AUDIT_LIFECYCLE_SURFACES {
        for attack_class in SHARED_ATTACK_CLASSES {
            units.push(AuditCoverageUnitV1 {
                unit_id: format!("unit-{lifecycle}-{attack_class}"),
                scope_path: "src/api.rs".to_string(),
                subsystem: "api".to_string(),
                boundary: "tenant-api-to-database".to_string(),
                lifecycle: lifecycle.to_string(),
                language: "rust".to_string(),
                attack_class: attack_class.to_string(),
                disposition: AuditCoverageDisposition::Covered,
                reason: None,
                evidence_refs: vec!["evidence://repo/src/api.rs#L10-L20".to_string()],
            });
        }
    }
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    AuditCoverageLedgerV1 {
        schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        units,
    }
}

fn confirmed_verdict() -> AuditFindingVerdictV1 {
    AuditFindingVerdictV1::Confirmed {
        severity: AuditSeverity::High,
        affected_principal_or_attacker: "authenticated user from another tenant".to_string(),
        protected_resource: "tenant-owned record".to_string(),
        impact: "cross-tenant disclosure".to_string(),
        source_path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 20,
        evidence_refs: vec![evidence()],
        recommended_correction: "enforce tenant ownership in the server-side query".to_string(),
        safe_verification: "test cross-tenant access with an isolated fixture".to_string(),
    }
}

fn completed_run(assessment: AuditFindingVerdictV1) -> (AuditRunV1, String) {
    let mut run = AuditRunV1::new(
        "00000000-0000-4000-8000-000000000041",
        binding(),
        AuditMode::Focused,
    )
    .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::Reconnaissance))
        .unwrap();
    run.set_coverage_ledger(coverage_ledger()).unwrap();
    let fingerprint = run.add_candidate(candidate()).unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::CoverageLedHunting))
        .unwrap();
    run.add_candidate_review(
        &fingerprint,
        AuditCandidateReviewV1 {
            reviewer_id: "verifier-a".to_string(),
            context_id: "fresh-context-a".to_string(),
            source_revision: run.binding.source_revision.clone(),
            binding_sha256: run.binding_fingerprint().unwrap(),
            candidate_revision: 0,
            assessment,
        },
    )
    .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::StructuredOutput))
        .unwrap();
    if matches!(
        run.findings[0]
            .final_verdict(run.mode, &run.binding)
            .unwrap(),
        AuditFindingVerdictV1::Confirmed { .. }
    ) {
        run.record_independent_record_review(
            &fingerprint,
            AuditRecordReviewV1 {
                reviewer_id: "record-verifier".to_string(),
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
    (run, fingerprint)
}

fn remediation(run: &AuditRunV1, fingerprint: &str) -> SecurityAuditRemediationV1 {
    SecurityAuditRemediationV1 {
        schema_version: SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
        remediation_id: "00000000-0000-4000-8000-000000000042".to_string(),
        source_audit_run_id_sha256: run.run_id_sha256(),
        source_binding_sha256: run.binding_fingerprint().unwrap(),
        finding_fingerprint: fingerprint.to_string(),
        status: SecurityAuditRemediationStateV1::InProgress,
        cycles: Vec::new(),
        blocker: None,
        next_action: "Begin only after Core grants safe remediation authorization".to_string(),
    }
}

fn changed_binding(before: &AuditRunBindingV1, marker: u32) -> AuditRunBindingV1 {
    let mut after = before.clone();
    after.dirty_state_sha256 = Some(format!("{marker:064x}"));
    after
}

fn cycle(
    number: u8,
    before: &AuditRunBindingV1,
    after: &AuditRunBindingV1,
) -> SecurityAuditRemediationCycleV1 {
    SecurityAuditRemediationCycleV1 {
        cycle_number: number,
        source_before_binding_sha256: binding_digest(before),
        source_after_binding: Some(after.clone()),
        changed_paths: vec!["src/api.rs".to_string()],
        source_diff_sha256: Some("1".repeat(64)),
        fixer_id: Some("fixer-agent".to_string()),
        fix_status: RemediationFixStatusV1::Applied,
        fix_summary: "enforce ownership in the query".to_string(),
        regression_status: RemediationCheckStatusV1::Passed,
        regression_evidence_sha256: Some(format!("{:064x}", u32::from(number) + 10)),
        regression_binding_sha256: Some(binding_digest(after)),
        re_audit_status: RemediationReauditStatusV1::NotRun,
        re_audit_binding_sha256: None,
        re_audit_run_id_sha256: None,
        security_auditor_status: RemediationCheckStatusV1::NotRun,
        security_auditor_evidence_sha256: None,
        security_auditor_id: None,
        security_auditor_context_id: None,
        security_auditor_binding_sha256: None,
        proof_record_id_sha256: None,
        trace_record_id_sha256: None,
        blocker: None,
    }
}

#[test]
fn post_fix_binding_must_preserve_project_task_operation_and_scope() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let mut foreign_binding = run.binding.clone();
    foreign_binding.project_id = "another-project".to_string();
    foreign_binding.task_id = "another-task".to_string();
    foreign_binding.operation_id = "another-operation".to_string();
    foreign_binding.scope = vec!["other".to_string()];
    foreign_binding.source_dirty = true;
    foreign_binding.dirty_state_sha256 = Some("e".repeat(64));
    let attempt = cycle(1, &before, &foreign_binding);
    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![attempt])).is_err()
    );
}

fn request(
    run: AuditRunV1,
    fingerprint: &str,
    cycles: Vec<SecurityAuditRemediationCycleV1>,
) -> SecurityAuditRemediationRequestV1 {
    let mut remediation = remediation(&run, fingerprint);
    remediation.cycles = cycles;
    SecurityAuditRemediationRequestV1 {
        schema_version: SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
        audit_run: run,
        remediation,
    }
}

#[test]
fn confirmed_finding_is_schema_eligible_but_never_auto_authorized_or_closed() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let result = validate_security_audit_remediation(&request(run, &fingerprint, vec![])).unwrap();

    assert!(result.schema_eligible);
    assert!(!result.auto_fix_authorized);
    assert!(!result.closure_authorized);
    assert_eq!(result.validation_scope, "schema_contract_only");
    assert!(result
        .blockers
        .contains(&"trusted_evidence_and_sandbox_backend_unverified".to_string()));
}

#[test]
fn non_confirmed_finding_cannot_enter_the_remediation_loop() {
    let rejected = AuditFindingVerdictV1::Rejected {
        claim: candidate().claim,
        disproof: "the current handler checks tenant ownership".to_string(),
        evidence_refs: vec![evidence()],
    };
    let (run, fingerprint) = completed_run(rejected.clone());
    assert!(validate_security_audit_remediation(&request(run, &fingerprint, vec![])).is_err());

    let needs_validation = AuditFindingVerdictV1::NeedsValidation {
        unresolved_fact: "ownership middleware behavior is unavailable".to_string(),
        blocker: "current middleware source is outside the repository".to_string(),
    };
    let (run, fingerprint) = completed_run(needs_validation);
    assert!(validate_security_audit_remediation(&request(run, &fingerprint, vec![])).is_err());
}

#[test]
fn remediation_must_be_bound_to_the_exact_audit_run_and_operation() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let mut request = request(run, &fingerprint, vec![]);
    request.remediation.source_binding_sha256 = "f".repeat(64);
    assert!(validate_security_audit_remediation(&request).is_err());
}

#[test]
fn changed_paths_must_remain_inside_the_authorized_audit_scope() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after = changed_binding(&before, 101);
    let mut changed = cycle(1, &before, &after);
    changed.changed_paths = vec!["../outside/secrets.rs".to_string()];
    let result = request(run, &fingerprint, vec![changed]);
    assert!(validate_security_audit_remediation(&result).is_err());
}

#[test]
fn regression_must_pass_before_reaudit_or_gate_claims() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after = changed_binding(&before, 102);
    let after_digest = binding_digest(&after);
    let mut failed = cycle(1, &before, &after);
    failed.regression_status = RemediationCheckStatusV1::Failed;
    failed.re_audit_status = RemediationReauditStatusV1::NotConfirmed;
    failed.re_audit_binding_sha256 = Some(after_digest);
    failed.re_audit_run_id_sha256 = Some("f".repeat(64));
    let result = request(run, &fingerprint, vec![failed]);
    assert!(validate_security_audit_remediation(&result).is_err());
}

#[test]
fn regression_pass_must_bind_to_the_exact_post_fix_source() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after = changed_binding(&before, 103);
    let mut attempt = cycle(1, &before, &after);
    attempt.regression_binding_sha256 = Some("f".repeat(64));
    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![attempt])).is_err()
    );
}

#[test]
fn applied_fix_requires_a_source_diff_digest() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after = changed_binding(&before, 104);
    let mut attempt = cycle(1, &before, &after);
    attempt.source_diff_sha256 = None;
    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![attempt])).is_err()
    );
}

#[test]
fn fix_actor_must_be_distinct_from_audit_authors_and_reviewers() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after = changed_binding(&before, 105);
    let mut attempt = cycle(1, &before, &after);
    attempt.fixer_id = Some("hunter-a".to_string());
    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![attempt])).is_err()
    );
}

#[test]
fn security_gate_claim_requires_fresh_distinct_reviewer_on_exact_post_fix_source() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after_binding = changed_binding(&before, 106);
    let after = binding_digest(&after_binding);
    let mut attempt = cycle(1, &before, &after_binding);
    attempt.re_audit_status = RemediationReauditStatusV1::NotConfirmed;
    attempt.re_audit_binding_sha256 = Some(after.clone());
    attempt.re_audit_run_id_sha256 = Some("f".repeat(64));
    attempt.security_auditor_status = RemediationCheckStatusV1::Passed;
    attempt.security_auditor_evidence_sha256 = Some("a".repeat(64));
    attempt.security_auditor_id = Some("security-auditor".to_string());
    attempt.security_auditor_context_id = Some("fresh-security-context".to_string());
    attempt.security_auditor_binding_sha256 = Some(after.clone());
    let valid = request(run.clone(), &fingerprint, vec![attempt.clone()]);
    assert!(validate_security_audit_remediation(&valid).is_ok());

    attempt.security_auditor_binding_sha256 = Some(binding_digest(&before));
    assert!(validate_security_audit_remediation(&request(
        run.clone(),
        &fingerprint,
        vec![attempt.clone()]
    ))
    .is_err());

    attempt.security_auditor_binding_sha256 = Some(after.clone());
    attempt.security_auditor_id = Some("fixer-agent".to_string());
    assert!(validate_security_audit_remediation(&request(
        run.clone(),
        &fingerprint,
        vec![attempt.clone()]
    ))
    .is_err());

    attempt.security_auditor_id = Some("security-auditor".to_string());
    attempt.security_auditor_context_id = Some("fresh-record-context".to_string());
    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![attempt])).is_err()
    );
}

#[test]
fn security_reviewer_must_be_distinct_from_every_fixer_in_the_timeline() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let first_after = changed_binding(&before, 201);
    let second_after = changed_binding(&first_after, 202);
    let mut first = cycle(1, &before, &first_after);
    first.fixer_id = Some("fixer-cycle-one".to_string());
    first.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
    first.re_audit_binding_sha256 = Some(binding_digest(&first_after));
    first.re_audit_run_id_sha256 = Some("a".repeat(64));

    let mut second = cycle(2, &first_after, &second_after);
    second.fixer_id = Some("fixer-cycle-two".to_string());
    second.re_audit_status = RemediationReauditStatusV1::NotConfirmed;
    second.re_audit_binding_sha256 = Some(binding_digest(&second_after));
    second.re_audit_run_id_sha256 = Some("b".repeat(64));
    second.security_auditor_status = RemediationCheckStatusV1::Passed;
    second.security_auditor_evidence_sha256 = Some("c".repeat(64));
    second.security_auditor_id = Some("fixer-cycle-one".to_string());
    second.security_auditor_context_id = Some("fresh-final-review-context".to_string());
    second.security_auditor_binding_sha256 = Some(binding_digest(&second_after));

    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![first, second]))
            .is_err()
    );
}

#[test]
fn regression_evidence_cannot_be_reused_across_cycles() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let first_after = changed_binding(&before, 203);
    let second_after = changed_binding(&first_after, 204);
    let mut first = cycle(1, &before, &first_after);
    first.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
    first.re_audit_binding_sha256 = Some(binding_digest(&first_after));
    first.re_audit_run_id_sha256 = Some("a".repeat(64));
    let mut second = cycle(2, &first_after, &second_after);
    second.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
    second.re_audit_binding_sha256 = Some(binding_digest(&second_after));
    second.re_audit_run_id_sha256 = Some("b".repeat(64));
    second.regression_evidence_sha256 = first.regression_evidence_sha256.clone();

    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![first, second]))
            .is_err()
    );
}

#[test]
fn reaudit_run_identifier_cannot_be_reused_across_cycles() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let first_after = changed_binding(&before, 205);
    let second_after = changed_binding(&first_after, 206);
    let mut first = cycle(1, &before, &first_after);
    first.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
    first.re_audit_binding_sha256 = Some(binding_digest(&first_after));
    first.re_audit_run_id_sha256 = Some("a".repeat(64));
    let mut second = cycle(2, &first_after, &second_after);
    second.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
    second.re_audit_binding_sha256 = Some(binding_digest(&second_after));
    second.re_audit_run_id_sha256 = first.re_audit_run_id_sha256.clone();

    assert!(
        validate_security_audit_remediation(&request(run, &fingerprint, vec![first, second]))
            .is_err()
    );
}

#[test]
fn failed_regression_is_terminal_and_cannot_be_bypassed_by_a_later_cycle() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let first_after = changed_binding(&before, 301);
    let mut failed = cycle(1, &before, &first_after);
    failed.regression_status = RemediationCheckStatusV1::Failed;
    let mut missing_cycle_blocker = request(run.clone(), &fingerprint, vec![failed.clone()]);
    missing_cycle_blocker.remediation.status = SecurityAuditRemediationStateV1::Blocked;
    missing_cycle_blocker.remediation.blocker = Some("regression failed".to_string());
    assert!(validate_security_audit_remediation(&missing_cycle_blocker).is_err());

    failed.blocker = Some("regression failed".to_string());
    let second_after = changed_binding(&first_after, 302);
    let retry = cycle(2, &first_after, &second_after);
    let mut result = request(run, &fingerprint, vec![failed, retry]);
    result.remediation.status = SecurityAuditRemediationStateV1::Blocked;
    result.remediation.blocker = Some("regression failed".to_string());
    assert!(validate_security_audit_remediation(&result).is_err());
}

#[test]
fn not_confirmed_reaudit_must_finish_gate_before_any_new_fix_cycle() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let first_after = changed_binding(&before, 303);
    let mut first = cycle(1, &before, &first_after);
    first.re_audit_status = RemediationReauditStatusV1::NotConfirmed;
    first.re_audit_binding_sha256 = Some(binding_digest(&first_after));
    first.re_audit_run_id_sha256 = Some("f".repeat(64));
    let second_after = changed_binding(&first_after, 304);
    let retry = cycle(2, &first_after, &second_after);
    let result = request(run, &fingerprint, vec![first, retry]);
    assert!(validate_security_audit_remediation(&result).is_err());
}

#[test]
fn remediation_cycles_are_sequential_and_cannot_exceed_three() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let mut cycles = Vec::new();
    let mut before = run.binding.clone();
    for number in 1..=4 {
        let after = changed_binding(&before, number as u32 + 100);
        cycles.push(cycle(number, &before, &after));
        before = after;
    }
    assert!(validate_security_audit_remediation(&request(run, &fingerprint, cycles)).is_err());
}

#[test]
fn third_still_confirmed_cycle_must_stop_instead_of_looping_forever() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let mut before = run.binding.clone();
    let mut cycles = Vec::new();
    for number in 1..=3 {
        let after = changed_binding(&before, u32::from(number) + 200);
        let mut attempt = cycle(number, &before, &after);
        attempt.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
        attempt.re_audit_binding_sha256 = Some(binding_digest(&after));
        attempt.re_audit_run_id_sha256 = Some(format!("{:064x}", u32::from(number) + 300));
        if number == 3 {
            attempt.blocker = Some("still confirmed after third re-audit".to_string());
        }
        cycles.push(attempt);
        before = after;
    }
    let mut result = request(run, &fingerprint, cycles);
    result.remediation.status = SecurityAuditRemediationStateV1::InProgress;
    result.remediation.next_action = "try another fix".to_string();
    assert!(validate_security_audit_remediation(&result).is_err());
}

#[test]
fn three_still_confirmed_cycles_are_valid_only_as_blocked_and_incomplete() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let mut before = run.binding.clone();
    let mut cycles = Vec::new();
    for number in 1..=3 {
        let after = changed_binding(&before, u32::from(number) + 500);
        let mut attempt = cycle(number, &before, &after);
        attempt.re_audit_status = RemediationReauditStatusV1::StillConfirmed;
        attempt.re_audit_binding_sha256 = Some(binding_digest(&after));
        attempt.re_audit_run_id_sha256 = Some(format!("{:064x}", u32::from(number) + 600));
        if number == 3 {
            attempt.blocker = Some("still confirmed after third re-audit".to_string());
        }
        cycles.push(attempt);
        before = after;
    }
    let mut result = request(run, &fingerprint, cycles);
    result.remediation.status = SecurityAuditRemediationStateV1::Blocked;
    result.remediation.blocker = Some("still confirmed after third re-audit".to_string());
    result.remediation.next_action =
        "stop automatic remediation and report the open finding".to_string();

    let validation = validate_security_audit_remediation(&result).unwrap();
    assert_eq!(validation.cycle_count, 3);
    assert_eq!(validation.status_claim, "blocked");
    assert!(!validation.auto_fix_authorized);
    assert!(!validation.closure_authorized);
}

#[test]
fn blocked_cycle_must_be_final_and_cannot_claim_progress() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after = changed_binding(&before, 701);
    let mut blocked_cycle = cycle(1, &before, &after);
    blocked_cycle.blocker = Some("SPEC-08-05 sandbox is unavailable".to_string());

    let mut in_progress = request(run.clone(), &fingerprint, vec![blocked_cycle.clone()]);
    in_progress.remediation.next_action = "run the fix".to_string();
    assert!(validate_security_audit_remediation(&in_progress).is_err());

    let mut blocked = request(run.clone(), &fingerprint, vec![blocked_cycle.clone()]);
    blocked.remediation.status = SecurityAuditRemediationStateV1::Blocked;
    blocked.remediation.blocker = Some("SPEC-08-05 sandbox is unavailable".to_string());
    assert!(validate_security_audit_remediation(&blocked).is_ok());

    let second_after = changed_binding(&after, 702);
    let resumed = cycle(2, &after, &second_after);
    blocked.remediation.cycles.push(resumed);
    assert!(validate_security_audit_remediation(&blocked).is_err());
}

#[test]
fn serialized_gate_proof_and_trace_claims_cannot_close_a_finding() {
    let (run, fingerprint) = completed_run(confirmed_verdict());
    let before = run.binding.clone();
    let after_binding = changed_binding(&before, 801);
    let after = binding_digest(&after_binding);
    let mut finished_claim = cycle(1, &before, &after_binding);
    finished_claim.re_audit_status = RemediationReauditStatusV1::NotConfirmed;
    finished_claim.re_audit_binding_sha256 = Some(after.clone());
    finished_claim.re_audit_run_id_sha256 = Some("f".repeat(64));
    finished_claim.security_auditor_status = RemediationCheckStatusV1::Passed;
    finished_claim.security_auditor_evidence_sha256 = Some("a".repeat(64));
    finished_claim.security_auditor_id = Some("security-auditor".to_string());
    finished_claim.security_auditor_context_id = Some("fresh-security-context".to_string());
    finished_claim.security_auditor_binding_sha256 = Some(after.clone());
    finished_claim.proof_record_id_sha256 = Some("b".repeat(64));
    finished_claim.trace_record_id_sha256 = Some("c".repeat(64));
    let mut request = request(run, &fingerprint, vec![finished_claim]);
    request.remediation.status = SecurityAuditRemediationStateV1::InProgress;

    let result = validate_security_audit_remediation(&request).unwrap();
    assert!(result.schema_eligible);
    assert!(!result.auto_fix_authorized);
    assert!(!result.closure_authorized);
    assert!(result
        .blockers
        .contains(&"trusted_evidence_and_sandbox_backend_unverified".to_string()));
}
