use std::fs;

use baron_core::config::{initialize_project, AdapterKind};
use baron_core::operation::{LifecycleIdentity, OperationContext, SupportedAdapter};
use baron_core::plan::start_or_resume_plan_for_identity;
use baron_core::security_audit::SHARED_ATTACK_CLASSES;
use baron_core::security_audit_workflow::{
    candidate_fingerprint, deduplicate_candidates, record_security_audit_continuity_checkpoint,
    validate_audit_run_json, validate_coverage_ledger, AuditCandidateReviewV1, AuditCandidateV1,
    AuditCoverageDisposition, AuditCoverageLedgerV1, AuditCoverageUnitV1, AuditEvidenceRefV1,
    AuditExecutionOutcome, AuditExecutionReceiptV1, AuditFindingRecordV1, AuditFindingVerdictV1,
    AuditMode, AuditPhase, AuditPhaseReceiptV1, AuditRecordReviewV1, AuditReportExecutionStatusV1,
    AuditRunBindingV1, AuditRunV1, AuditSeverity, AuditValidationScopeV1,
    SECURITY_AUDIT_LIFECYCLE_SURFACES, SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
};
use baron_core::vault::ensure_vault;
use tempfile::tempdir;

fn valid_binding() -> AuditRunBindingV1 {
    AuditRunBindingV1 {
        project_id: "project-123".to_string(),
        source_revision: "a".repeat(40),
        source_dirty: true,
        dirty_state_sha256: Some("b".repeat(64)),
        scope: vec!["src".to_string()],
        languages: vec!["rust".to_string()],
        profile: "backend".to_string(),
        task_id: "task-456".to_string(),
        task_state_sha256: "c".repeat(64),
        operation_id: "operation-789".to_string(),
    }
}

fn test_run_id(number: u64) -> String {
    format!("00000000-0000-4000-8000-{number:012x}")
}

fn binding_sha256(binding: &AuditRunBindingV1) -> String {
    AuditRunV1::new(test_run_id(999), binding.clone(), AuditMode::Focused)
        .unwrap()
        .binding_fingerprint()
        .unwrap()
}

fn unit(id: &str, disposition: AuditCoverageDisposition) -> AuditCoverageUnitV1 {
    AuditCoverageUnitV1 {
        unit_id: id.to_string(),
        scope_path: "src/api.rs".to_string(),
        subsystem: "api".to_string(),
        boundary: "authenticated-request-to-database".to_string(),
        lifecycle: "authorization_and_ownership".to_string(),
        language: "rust".to_string(),
        attack_class: "authentication_authorization_and_ownership".to_string(),
        disposition,
        reason: (disposition != AuditCoverageDisposition::Covered)
            .then(|| "explicitly deferred by test fixture".to_string()),
        evidence_refs: vec!["evidence://repo/src/api.rs#L10-L20".to_string()],
    }
}

fn ledger(units: Vec<AuditCoverageUnitV1>) -> AuditCoverageLedgerV1 {
    AuditCoverageLedgerV1 {
        schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        units,
    }
}

fn full_ledger(disposition: AuditCoverageDisposition) -> AuditCoverageLedgerV1 {
    let mut units = Vec::new();
    for lifecycle in SECURITY_AUDIT_LIFECYCLE_SURFACES {
        for attack_class in SHARED_ATTACK_CLASSES {
            let id = format!("unit-{lifecycle}-{attack_class}");
            let mut coverage = unit(&id, disposition);
            coverage.lifecycle = lifecycle.to_string();
            coverage.attack_class = attack_class.to_string();
            units.push(coverage);
        }
    }
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    ledger(units)
}

fn phase_receipt(run: &AuditRunV1, phase: AuditPhase) -> AuditPhaseReceiptV1 {
    AuditPhaseReceiptV1 {
        phase,
        binding_sha256: run.binding_fingerprint().unwrap(),
        operation_id: run.binding.operation_id.clone(),
        timestamp: "2026-10-28T12:00:00Z".to_string(),
        artifact_ref: format!("audit-artifact://{}", phase.as_str()),
    }
}

fn evidence(start_line: u32, end_line: u32) -> AuditEvidenceRefV1 {
    AuditEvidenceRefV1 {
        path: "src/api.rs".to_string(),
        start_line,
        end_line,
        content_sha256: "d".repeat(64),
    }
}

fn candidate(root_cause_key: &str) -> AuditCandidateV1 {
    AuditCandidateV1 {
        root_cause_key: root_cause_key.to_string(),
        attack_class: "authentication_authorization_and_ownership".to_string(),
        language: "rust".to_string(),
        subsystem: "api".to_string(),
        protected_boundary: "authenticated-request-to-database".to_string(),
        lifecycle: "authorization_and_ownership".to_string(),
        source_path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 20,
        claim: format!("candidate claim for {root_cause_key}"),
        author_ids: vec!["hunter-a".to_string()],
        evidence_refs: vec![evidence(10, 20)],
    }
}

fn confirmed_assessment() -> AuditFindingVerdictV1 {
    AuditFindingVerdictV1::Confirmed {
        severity: AuditSeverity::High,
        affected_principal_or_attacker: "authenticated user without tenant ownership".to_string(),
        protected_resource: "another tenant's record".to_string(),
        impact: "cross-tenant record disclosure".to_string(),
        source_path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 20,
        evidence_refs: vec![evidence(10, 20)],
        recommended_correction: "enforce ownership in the server-side query".to_string(),
        safe_verification: "add a regression test for cross-tenant access".to_string(),
    }
}

fn candidate_review(reviewer_id: &str, context_id: &str) -> AuditCandidateReviewV1 {
    AuditCandidateReviewV1 {
        reviewer_id: reviewer_id.to_string(),
        context_id: context_id.to_string(),
        source_revision: "a".repeat(40),
        binding_sha256: binding_sha256(&valid_binding()),
        candidate_revision: 0,
        assessment: confirmed_assessment(),
    }
}

fn finish_through_hunting(run: &mut AuditRunV1, finding: AuditCandidateV1) -> String {
    run.complete_phase(phase_receipt(run, AuditPhase::Reconnaissance))
        .unwrap();
    run.set_coverage_ledger(full_ledger(AuditCoverageDisposition::Covered))
        .unwrap();
    let fingerprint = run.add_candidate(finding).unwrap();
    run.complete_phase(phase_receipt(run, AuditPhase::CoverageLedHunting))
        .unwrap();
    fingerprint
}

#[test]
fn coverage_ledger_requires_supported_schema_and_unique_bounded_units() {
    let mut valid = full_ledger(AuditCoverageDisposition::Covered);
    let deferred = unit("unit-deferred", AuditCoverageDisposition::Deferred);
    valid.units.push(deferred);
    valid
        .units
        .sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    assert!(validate_coverage_ledger(&valid, &valid_binding()).is_ok());

    let duplicate = ledger(vec![
        unit("same", AuditCoverageDisposition::Covered),
        unit("same", AuditCoverageDisposition::Skipped),
    ]);
    assert!(validate_coverage_ledger(&duplicate, &valid_binding()).is_err());

    let mut incomplete = full_ledger(AuditCoverageDisposition::Covered);
    incomplete.units.pop();
    assert!(validate_coverage_ledger(&incomplete, &valid_binding()).is_err());

    let mut unsupported = valid;
    unsupported.schema_version += 1;
    assert!(validate_coverage_ledger(&unsupported, &valid_binding()).is_err());
}

#[test]
fn uncovered_units_require_an_explicit_reason() {
    for disposition in [
        AuditCoverageDisposition::Skipped,
        AuditCoverageDisposition::Deferred,
        AuditCoverageDisposition::OutOfScope,
    ] {
        let mut uncovered = full_ledger(AuditCoverageDisposition::Covered);
        uncovered.units[0].disposition = disposition;
        uncovered.units[0].reason = None;
        assert!(validate_coverage_ledger(&uncovered, &valid_binding()).is_err());
    }
}

#[test]
fn audit_run_rejects_phase_skips_and_requires_coverage_before_hunting_completes() {
    let mut run = AuditRunV1::new(test_run_id(1), valid_binding(), AuditMode::Standard).unwrap();
    assert_eq!(run.current_phase, AuditPhase::Reconnaissance);

    let skipped = phase_receipt(&run, AuditPhase::CandidateValidation);
    assert!(run.complete_phase(skipped).is_err());

    let reconnaissance = phase_receipt(&run, AuditPhase::Reconnaissance);
    run.complete_phase(reconnaissance).unwrap();
    assert_eq!(run.current_phase, AuditPhase::CoverageLedHunting);

    let missing_ledger = phase_receipt(&run, AuditPhase::CoverageLedHunting);
    assert!(run.complete_phase(missing_ledger).is_err());

    run.set_coverage_ledger(full_ledger(AuditCoverageDisposition::Covered))
        .unwrap();
    let hunting = phase_receipt(&run, AuditPhase::CoverageLedHunting);
    run.complete_phase(hunting).unwrap();
    assert_eq!(run.current_phase, AuditPhase::CandidateValidation);
}

#[test]
fn phase_receipts_are_bound_to_exact_revision_scope_and_operation() {
    let mut run = AuditRunV1::new(test_run_id(2), valid_binding(), AuditMode::Focused).unwrap();
    let mut receipt = phase_receipt(&run, AuditPhase::Reconnaissance);
    receipt.operation_id = "another-operation".to_string();
    assert!(run.complete_phase(receipt).is_err());

    let mut receipt = phase_receipt(&run, AuditPhase::Reconnaissance);
    receipt.binding_sha256 = "0".repeat(64);
    assert!(run.complete_phase(receipt).is_err());
}

#[test]
fn run_binding_rejects_unrooted_scope_and_incomplete_dirty_state_identity() {
    let mut binding = valid_binding();
    binding.scope = vec!["../outside".to_string()];
    assert!(AuditRunV1::new(test_run_id(3), binding, AuditMode::Focused).is_err());

    let mut binding = valid_binding();
    binding.dirty_state_sha256 = None;
    assert!(AuditRunV1::new(test_run_id(4), binding, AuditMode::Focused).is_err());
}

#[test]
fn candidate_author_cannot_verify_their_own_claim_and_stale_reviews_fail_closed() {
    let mut run = AuditRunV1::new(test_run_id(5), valid_binding(), AuditMode::Standard).unwrap();
    let fingerprint = finish_through_hunting(&mut run, candidate("root-cause-a"));
    assert!(run
        .add_candidate_review(&fingerprint, candidate_review("hunter-a", "ctx-self"))
        .is_err());

    let mut stale = candidate_review("verifier-a", "ctx-stale");
    stale.source_revision = "e".repeat(40);
    assert!(run.add_candidate_review(&fingerprint, stale).is_err());
}

#[test]
fn candidate_review_is_bound_to_dirty_state_and_task_state_at_same_git_revision() {
    let original_binding = valid_binding();
    let mut changed_binding = original_binding.clone();
    changed_binding.dirty_state_sha256 = Some("e".repeat(64));
    changed_binding.task_state_sha256 = "f".repeat(64);
    assert_eq!(
        original_binding.source_revision,
        changed_binding.source_revision
    );

    let mut run = AuditRunV1::new(test_run_id(11), changed_binding, AuditMode::Standard).unwrap();
    let finding = candidate("root-cause-a");
    assert_ne!(
        candidate_fingerprint(&original_binding, &finding).unwrap(),
        candidate_fingerprint(&run.binding, &finding).unwrap()
    );
    let fingerprint = finish_through_hunting(&mut run, finding);
    let stale_review = candidate_review("verifier-a", "ctx-original-dirty-tree");

    assert!(run
        .add_candidate_review(&fingerprint, stale_review)
        .is_err());
}

#[test]
fn independent_record_review_is_bound_to_the_complete_run_binding() {
    let original_binding = valid_binding();
    let mut changed_binding = original_binding.clone();
    changed_binding.dirty_state_sha256 = Some("e".repeat(64));
    let mut run = AuditRunV1::new(
        test_run_id(15),
        changed_binding.clone(),
        AuditMode::Standard,
    )
    .unwrap();
    let fingerprint = finish_through_hunting(&mut run, candidate("root-cause-a"));
    let mut candidate_review = candidate_review("verifier-a", "ctx-review-dirty-run");
    candidate_review.binding_sha256 = binding_sha256(&changed_binding);
    run.add_candidate_review(&fingerprint, candidate_review)
        .unwrap();

    let stale_record_review = AuditRecordReviewV1 {
        reviewer_id: "record-verifier".to_string(),
        context_id: "ctx-record-stale-run".to_string(),
        source_revision: changed_binding.source_revision.clone(),
        binding_sha256: binding_sha256(&original_binding),
        candidate_revision: 0,
        candidate_fingerprint: fingerprint.clone(),
        accepted: true,
        correction_requires_revalidation: false,
        evidence_refs: vec![evidence(10, 20)],
    };

    assert!(run
        .record_independent_record_review(&fingerprint, stale_record_review)
        .is_err());
}

#[test]
fn confirmed_review_cannot_move_candidate_to_unrelated_lines() {
    let mut run = AuditRunV1::new(test_run_id(12), valid_binding(), AuditMode::Standard).unwrap();
    let fingerprint = finish_through_hunting(&mut run, candidate("root-cause-a"));
    for (context_id, start_line, end_line) in [
        ("ctx-unrelated-confirmation", 30, 35),
        ("ctx-expanded-confirmation", 10, 40),
    ] {
        let mut review = candidate_review("verifier-a", context_id);
        review.assessment = AuditFindingVerdictV1::Confirmed {
            severity: AuditSeverity::High,
            affected_principal_or_attacker: "authenticated user".to_string(),
            protected_resource: "another tenant record".to_string(),
            impact: "cross-tenant disclosure".to_string(),
            source_path: "src/api.rs".to_string(),
            start_line,
            end_line,
            evidence_refs: vec![evidence(start_line, end_line)],
            recommended_correction: "enforce tenant ownership".to_string(),
            safe_verification: "add a tenant isolation regression".to_string(),
        };

        assert!(run.add_candidate_review(&fingerprint, review).is_err());
    }
}

#[test]
fn rejected_review_requires_the_exact_claim_and_relevant_in_scope_disproof() {
    let mut run = AuditRunV1::new(test_run_id(13), valid_binding(), AuditMode::Standard).unwrap();
    let original_candidate = candidate("root-cause-a");
    let fingerprint = finish_through_hunting(&mut run, original_candidate.clone());

    for (index, (claim, path, start_line, end_line)) in [
        ("a different candidate claim", "src/api.rs", 10, 20),
        (
            original_candidate.claim.as_str(),
            "src/unrelated.rs",
            200,
            210,
        ),
        (
            original_candidate.claim.as_str(),
            "docs/unrelated.md",
            10,
            20,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let reviewer = format!("verifier-{index}");
        let context = format!("ctx-unrelated-rejection-{index}");
        let mut review = candidate_review(&reviewer, &context);
        review.assessment = AuditFindingVerdictV1::Rejected {
            claim: claim.to_string(),
            disproof: "the claim is not supported by the current source".to_string(),
            evidence_refs: vec![AuditEvidenceRefV1 {
                path: path.to_string(),
                start_line,
                end_line,
                content_sha256: "d".repeat(64),
            }],
        };
        assert!(run.add_candidate_review(&fingerprint, review).is_err());
    }
}

#[test]
fn run_id_rejects_free_form_secret_like_text() {
    assert!(AuditRunV1::new(
        "sk-live-super-secret-looking-value",
        valid_binding(),
        AuditMode::Focused
    )
    .is_err());
}

#[test]
fn needs_validation_schema_has_no_severity_field() {
    let input = br#"{"verdict":"needs_validation","unresolved_fact":"tenant authorization is not visible","blocker":"missing current authorization source","severity":"High"}"#;
    assert!(serde_json::from_slice::<AuditFindingVerdictV1>(input).is_err());
    let valid = br#"{"verdict":"needs_validation","unresolved_fact":"tenant authorization is not visible","blocker":"missing current authorization source"}"#;
    assert!(serde_json::from_slice::<AuditFindingVerdictV1>(valid).is_ok());
}

#[test]
fn duplicate_candidates_merge_evidence_but_keep_distinct_root_causes() {
    let binding = valid_binding();
    let mut duplicate = candidate("root-cause-a");
    duplicate.author_ids = vec!["hunter-b".to_string()];
    duplicate.evidence_refs = vec![evidence(12, 16)];
    let distinct = candidate("root-cause-b");

    let merged = deduplicate_candidates(
        &binding,
        vec![
            candidate("root-cause-a"),
            duplicate.clone(),
            distinct.clone(),
        ],
    )
    .unwrap();

    assert_eq!(merged.len(), 2);
    let first_fp = candidate_fingerprint(&binding, &merged[0]).unwrap();
    let matching = merged
        .iter()
        .find(|item| item.root_cause_key == "root-cause-a")
        .unwrap();
    assert_eq!(candidate_fingerprint(&binding, matching).unwrap(), first_fp);
    assert_eq!(matching.author_ids, vec!["hunter-a", "hunter-b"]);
    assert_eq!(matching.evidence_refs.len(), 2);
    let reversed = deduplicate_candidates(
        &binding,
        vec![distinct, duplicate, candidate("root-cause-a")],
    )
    .unwrap();
    assert_eq!(merged, reversed);
}

#[test]
fn complete_run_requires_six_phases_and_independent_record_verification() {
    let mut run = AuditRunV1::new(test_run_id(6), valid_binding(), AuditMode::Standard).unwrap();
    let fingerprint = finish_through_hunting(&mut run, candidate("root-cause-a"));
    run.add_candidate_review(&fingerprint, candidate_review("verifier-a", "ctx-review-a"))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::StructuredOutput))
        .unwrap();

    let missing_record_review = phase_receipt(&run, AuditPhase::IndependentRecordVerification);
    assert!(run.complete_phase(missing_record_review).is_err());

    let mut unrelated_record_evidence = AuditRecordReviewV1 {
        reviewer_id: "record-verifier".to_string(),
        context_id: "ctx-record-a".to_string(),
        source_revision: "a".repeat(40),
        binding_sha256: run.binding_fingerprint().unwrap(),
        candidate_revision: 0,
        candidate_fingerprint: fingerprint.clone(),
        accepted: true,
        correction_requires_revalidation: false,
        evidence_refs: vec![evidence(10, 20)],
    };
    unrelated_record_evidence.evidence_refs[0].path = "src/unrelated.rs".to_string();
    assert!(run
        .record_independent_record_review(&fingerprint, unrelated_record_evidence)
        .is_err());

    run.record_independent_record_review(
        &fingerprint,
        AuditRecordReviewV1 {
            reviewer_id: "record-verifier".to_string(),
            context_id: "ctx-record-a".to_string(),
            source_revision: "a".repeat(40),
            binding_sha256: run.binding_fingerprint().unwrap(),
            candidate_revision: 0,
            candidate_fingerprint: fingerprint.clone(),
            accepted: true,
            correction_requires_revalidation: false,
            evidence_refs: vec![evidence(10, 20)],
        },
    )
    .unwrap();
    run.complete_phase(phase_receipt(
        &run,
        AuditPhase::IndependentRecordVerification,
    ))
    .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::Reporting))
        .unwrap();

    assert_eq!(run.phase_receipts.len(), 6);
    assert!(validate_audit_run_json(&serde_json::to_vec(&run).unwrap()).is_ok());
    let report = run.report().unwrap();
    assert_eq!(
        report.validation_scope,
        AuditValidationScopeV1::SchemaAndWorkflowContractOnly
    );
    assert_eq!(
        report.execution_status,
        AuditReportExecutionStatusV1::NotRun
    );
    assert!(!report.proof_gate_trace_claimed);
    assert_eq!(
        report.coverage_ledger.as_ref().unwrap().units.len(),
        SECURITY_AUDIT_LIFECYCLE_SURFACES.len() * SHARED_ATTACK_CLASSES.len()
    );
    assert_eq!(
        report.coverage.covered,
        SECURITY_AUDIT_LIFECYCLE_SURFACES.len() * SHARED_ATTACK_CLASSES.len()
    );
    assert_eq!(report.findings.len(), 1);
    assert!(matches!(
        report.findings[0].verdict.as_ref(),
        Some(AuditFindingVerdictV1::Confirmed {
            severity: AuditSeverity::High,
            ..
        })
    ));
    assert!(run
        .record_execution_receipt(AuditExecutionReceiptV1 {
            operation: "late operation claim".to_string(),
            operation_id: run.binding.operation_id.clone(),
            source_revision: run.binding.source_revision.clone(),
            binding_sha256: run.binding_fingerprint().unwrap(),
            outcome: AuditExecutionOutcome::Completed,
            timestamp: "2026-10-28T12:00:00Z".to_string(),
            artifact_ref: "audit-artifact://late".to_string(),
        })
        .is_err());
}

#[test]
fn serialized_execution_receipts_remain_unverified_and_never_claim_proof() {
    let mut run = AuditRunV1::new(test_run_id(9), valid_binding(), AuditMode::Focused).unwrap();
    run.record_execution_receipt(AuditExecutionReceiptV1 {
        operation: "claimed audit operation".to_string(),
        operation_id: run.binding.operation_id.clone(),
        source_revision: run.binding.source_revision.clone(),
        binding_sha256: run.binding_fingerprint().unwrap(),
        outcome: AuditExecutionOutcome::Completed,
        timestamp: "2026-10-28T12:00:00Z".to_string(),
        artifact_ref: "audit-artifact://unverified".to_string(),
    })
    .unwrap();

    let report = run.report().unwrap();
    assert_eq!(
        report.execution_status,
        AuditReportExecutionStatusV1::EvidenceRecordedUnverified
    );
    assert!(!report.proof_gate_trace_claimed);
}

#[test]
fn partial_report_retains_unvalidated_candidates_and_their_exact_context() {
    let mut run = AuditRunV1::new(test_run_id(10), valid_binding(), AuditMode::Focused).unwrap();
    let fingerprint = finish_through_hunting(&mut run, candidate("root-cause-unreviewed"));

    let report = run.report().unwrap();
    assert_eq!(report.findings.len(), 1);
    let finding = &report.findings[0];
    assert_eq!(finding.fingerprint, fingerprint);
    assert_eq!(finding.candidate.source_path, "src/api.rs");
    assert_eq!(
        finding.candidate.attack_class,
        "authentication_authorization_and_ownership"
    );
    assert!(finding.verdict.is_none());
    assert!(report
        .residual_checks
        .contains(&"candidate_validation_incomplete".to_string()));
}

#[test]
fn continuity_checkpoint_is_project_and_operation_bound_and_contains_only_summary() {
    let temp = tempdir().unwrap();
    let repo = temp.path().join("project");
    let vault_root = temp.path().join("Vault");
    fs::create_dir_all(&repo).unwrap();
    initialize_project(&repo, AdapterKind::Codex, &vault_root).unwrap();
    let vault = ensure_vault(&vault_root, &repo).unwrap();
    let task = "review authorization boundary";
    let identity = LifecycleIdentity::resolve(
        &vault.project_id,
        task,
        SupportedAdapter::Codex,
        Some("security-audit-session"),
        Some("security-audit-request"),
    )
    .unwrap();
    start_or_resume_plan_for_identity(&repo, &vault, task, &identity).unwrap();

    let mut binding = valid_binding();
    binding.project_id = vault.project_id.clone();
    binding.task_id = identity.task_id().to_string();
    binding.operation_id = identity.operation_id().to_string();
    let run_id = test_run_id(14);
    let mut run = AuditRunV1::new(run_id.clone(), binding, AuditMode::Focused).unwrap();
    run.block("sandbox_backend_unverified").unwrap();
    assert!(run
        .record_execution_receipt(AuditExecutionReceiptV1 {
            operation: "blocked operation claim".to_string(),
            operation_id: run.binding.operation_id.clone(),
            source_revision: run.binding.source_revision.clone(),
            binding_sha256: run.binding_fingerprint().unwrap(),
            outcome: AuditExecutionOutcome::Completed,
            timestamp: "2026-10-28T12:00:00Z".to_string(),
            artifact_ref: "audit-artifact://blocked".to_string(),
        })
        .is_err());

    let other_identity = LifecycleIdentity::resolve(
        &vault.project_id,
        "different operation",
        SupportedAdapter::Codex,
        Some("security-audit-session-b"),
        Some("security-audit-request-b"),
    )
    .unwrap();
    assert!(record_security_audit_continuity_checkpoint(
        &repo,
        &vault,
        &OperationContext::from_identity(&other_identity),
        &run,
    )
    .is_err());
    assert!(!repo.join("docs/baron/continuity/CURRENT.md").exists());

    let packet = record_security_audit_continuity_checkpoint(
        &repo,
        &vault,
        &OperationContext::from_identity(&identity),
        &run,
    )
    .unwrap();
    let repo_packet = fs::read_to_string(packet.repo_path).unwrap();
    let vault_packet = fs::read_to_string(packet.vault_path).unwrap();
    for content in [repo_packet, vault_packet] {
        assert!(!content.contains(&run_id));
        assert!(content.contains(&run.run_id_sha256()));
        assert!(content.contains("sandbox_backend_unverified"));
        assert!(content.contains("candidate records 0"));
    }
}

#[test]
fn deep_mode_requires_two_distinct_fresh_candidate_verifiers() {
    let mut run = AuditRunV1::new(test_run_id(7), valid_binding(), AuditMode::Deep).unwrap();
    let fingerprint = finish_through_hunting(&mut run, candidate("root-cause-a"));
    run.add_candidate_review(&fingerprint, candidate_review("verifier-a", "ctx-a"))
        .unwrap();
    assert!(run
        .complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .is_err());
    run.add_candidate_review(&fingerprint, candidate_review("verifier-b", "ctx-b"))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .unwrap();
}

#[test]
fn record_correction_reopens_candidate_validation_and_supersedes_later_phases() {
    let mut run = AuditRunV1::new(test_run_id(8), valid_binding(), AuditMode::Standard).unwrap();
    let finding = candidate("root-cause-a");
    let fingerprint = finish_through_hunting(&mut run, finding.clone());
    run.add_candidate_review(&fingerprint, candidate_review("verifier-a", "ctx-review-a"))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::StructuredOutput))
        .unwrap();
    run.record_independent_record_review(
        &fingerprint,
        AuditRecordReviewV1 {
            reviewer_id: "record-verifier".to_string(),
            context_id: "ctx-record-a".to_string(),
            source_revision: "a".repeat(40),
            binding_sha256: run.binding_fingerprint().unwrap(),
            candidate_revision: 0,
            candidate_fingerprint: fingerprint.clone(),
            accepted: false,
            correction_requires_revalidation: true,
            evidence_refs: vec![evidence(10, 20)],
        },
    )
    .unwrap();
    run.reopen_candidate_for_revalidation(&fingerprint, finding)
        .unwrap();

    assert_eq!(run.current_phase, AuditPhase::CandidateValidation);
    assert_eq!(run.phase_receipts.len(), 2);
    assert_eq!(run.superseded_phase_receipts.len(), 2);
    assert_eq!(run.findings[0].candidate_revision, 1);
    assert!(run.findings[0].candidate_reviews.is_empty());
}

#[test]
fn mixed_candidate_reviews_resolve_to_needs_validation_not_a_confirmed_finding() {
    let candidate = candidate("root-cause-a");
    let mut rejection = candidate_review("verifier-b", "ctx-b");
    rejection.assessment = AuditFindingVerdictV1::Rejected {
        claim: candidate.claim.clone(),
        disproof: "the authorization guard is present on the current path".to_string(),
        evidence_refs: vec![evidence(10, 20)],
    };
    let record = AuditFindingRecordV1 {
        candidate,
        candidate_revision: 0,
        candidate_reviews: vec![candidate_review("verifier-a", "ctx-a"), rejection],
        independent_record_review: None,
    };
    assert!(matches!(
        record
            .final_verdict(AuditMode::Deep, &valid_binding())
            .unwrap(),
        AuditFindingVerdictV1::NeedsValidation { .. }
    ));
}
