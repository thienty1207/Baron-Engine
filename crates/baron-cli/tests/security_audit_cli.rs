use assert_cmd::Command;
use baron_core::identity::project_id_for_path;
use baron_core::security_audit::SHARED_ATTACK_CLASSES;
use baron_core::security_audit_remediation::{
    MAX_SECURITY_AUDIT_REMEDIATION_JSON_BYTES, SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
};
use baron_core::security_audit_safety::MAX_AUDIT_CHECKPOINT_JSON_BYTES;
use baron_core::security_audit_workflow::{
    AuditCandidateReviewV1, AuditCandidateV1, AuditCoverageDisposition, AuditCoverageLedgerV1,
    AuditCoverageUnitV1, AuditEvidenceRefV1, AuditFailureClassV1, AuditFailureV1,
    AuditFindingVerdictV1, AuditMode, AuditPhase, AuditPhaseReceiptV1, AuditRecordReviewV1,
    AuditRunBindingV1, AuditRunV1, AuditSeverity, SECURITY_AUDIT_LIFECYCLE_SURFACES,
    SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

fn valid_run() -> AuditRunV1 {
    AuditRunV1::new(
        "00000000-0000-4000-8000-000000000001",
        AuditRunBindingV1 {
            project_id: "project-cli".to_string(),
            source_revision: "a".repeat(40),
            source_dirty: false,
            dirty_state_sha256: None,
            scope: vec!["src".to_string()],
            languages: vec!["rust".to_string()],
            profile: "backend".to_string(),
            task_id: "task-cli".to_string(),
            task_state_sha256: "b".repeat(64),
            operation_id: "operation-cli".to_string(),
        },
        AuditMode::Focused,
    )
    .unwrap()
}

fn synthetic_repo() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn safe() {}\n").unwrap();
    std::fs::write(root.join(".gitignore"), ".baron/\n").unwrap();
    std::fs::create_dir_all(root.join(".baron")).unwrap();
    let project_id = project_id_for_path(root).unwrap();
    std::fs::write(
        root.join(".baron/project.toml"),
        format!(
            "schema_version = 4\nproject_id = \"{project_id}\"\nidentity_binding = \"{}\"\nproject_slug = \"cli-safety\"\nadapters = [\"codex\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
            "c".repeat(64)
        ),
    )
    .unwrap();
    git_fixture(root, &["init", "--quiet"]);
    git_fixture(
        root,
        &[
            "-c",
            "user.name=Safety Test",
            "-c",
            "user.email=safety@example.invalid",
            "add",
            "src/lib.rs",
            ".gitignore",
        ],
    );
    git_fixture(
        root,
        &[
            "-c",
            "user.name=Safety Test",
            "-c",
            "user.email=safety@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );
    temp
}

fn git_fixture(root: &Path, args: &[&str]) {
    let output = ProcessCommand::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "synthetic Git fixture setup failed"
    );
}

fn run_bound_to_synthetic_repo(root: &Path) -> AuditRunV1 {
    let revision = ProcessCommand::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .output()
        .unwrap();
    assert!(revision.status.success());
    let mut binding = valid_run().binding;
    binding.project_id = project_id_for_path(root).unwrap();
    binding.source_revision = String::from_utf8(revision.stdout)
        .unwrap()
        .trim()
        .to_ascii_lowercase();
    AuditRunV1::new(
        "00000000-0000-4000-8000-000000000031",
        binding,
        AuditMode::Focused,
    )
    .unwrap()
}

fn checkpoint_test_coverage_ledger() -> AuditCoverageLedgerV1 {
    let mut units = Vec::new();
    for lifecycle in SECURITY_AUDIT_LIFECYCLE_SURFACES {
        for attack_class in SHARED_ATTACK_CLASSES {
            units.push(AuditCoverageUnitV1 {
                unit_id: format!("checkpoint-{lifecycle}-{attack_class}"),
                scope_path: "src/lib.rs".to_string(),
                subsystem: "api".to_string(),
                boundary: "synthetic-fixture-boundary".to_string(),
                lifecycle: lifecycle.to_string(),
                language: "rust".to_string(),
                attack_class: attack_class.to_string(),
                disposition: AuditCoverageDisposition::Covered,
                reason: None,
                evidence_refs: vec!["evidence://repo/src/lib.rs#L1-L1".to_string()],
            });
        }
    }
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    AuditCoverageLedgerV1 {
        schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        units,
    }
}

fn checkpoint_test_candidate() -> AuditCandidateV1 {
    AuditCandidateV1 {
        root_cause_key: "checkpoint-retention-fixture".to_string(),
        attack_class: SHARED_ATTACK_CLASSES[0].to_string(),
        language: "rust".to_string(),
        subsystem: "api".to_string(),
        protected_boundary: "synthetic-fixture-boundary".to_string(),
        lifecycle: SECURITY_AUDIT_LIFECYCLE_SURFACES[0].to_string(),
        source_path: "src/lib.rs".to_string(),
        start_line: 1,
        end_line: 1,
        claim: "synthetic candidate used to verify checkpoint retention".to_string(),
        author_ids: vec!["hunter-a".to_string()],
        evidence_refs: vec![AuditEvidenceRefV1 {
            path: "src/lib.rs".to_string(),
            start_line: 1,
            end_line: 1,
            content_sha256: "d".repeat(64),
        }],
    }
}

fn run_at_interrupted_phase_boundary(root: &Path, boundary: usize) -> AuditRunV1 {
    let bound = run_bound_to_synthetic_repo(root);
    let mut run = AuditRunV1::new(
        format!("00000000-0000-4000-8000-{:012x}", boundary + 100),
        bound.binding,
        AuditMode::Focused,
    )
    .unwrap();
    let receipt = |run: &AuditRunV1, phase: AuditPhase| AuditPhaseReceiptV1 {
        phase,
        binding_sha256: run.binding_fingerprint().unwrap(),
        operation_id: run.binding.operation_id.clone(),
        timestamp: "2026-10-29T12:00:00Z".to_string(),
        artifact_ref: format!("audit-artifact://{}", phase.as_str()),
    };

    if boundary >= 1 {
        run.complete_phase(receipt(&run, AuditPhase::Reconnaissance))
            .unwrap();
    }
    if boundary >= 2 {
        run.set_coverage_ledger(checkpoint_test_coverage_ledger())
            .unwrap();
        run.add_candidate(checkpoint_test_candidate()).unwrap();
        run.complete_phase(receipt(&run, AuditPhase::CoverageLedHunting))
            .unwrap();
    }
    if boundary >= 3 {
        let candidate = &run.findings[0].candidate;
        run.add_candidate_review(
            &run.findings[0].fingerprint(&run.binding).unwrap(),
            AuditCandidateReviewV1 {
                reviewer_id: "verifier-a".to_string(),
                context_id: "checkpoint-fresh-review".to_string(),
                source_revision: run.binding.source_revision.clone(),
                binding_sha256: run.binding_fingerprint().unwrap(),
                candidate_revision: 0,
                assessment: AuditFindingVerdictV1::Rejected {
                    claim: candidate.claim.clone(),
                    disproof: "synthetic checkpoint fixture has no security behavior".to_string(),
                    evidence_refs: candidate.evidence_refs.clone(),
                },
            },
        )
        .unwrap();
        run.complete_phase(receipt(&run, AuditPhase::CandidateValidation))
            .unwrap();
    }
    if boundary >= 4 {
        run.complete_phase(receipt(&run, AuditPhase::StructuredOutput))
            .unwrap();
    }
    if boundary >= 5 {
        run.complete_phase(receipt(&run, AuditPhase::IndependentRecordVerification))
            .unwrap();
    }
    if boundary >= 6 {
        run.complete_phase(receipt(&run, AuditPhase::Reporting))
            .unwrap();
    } else {
        run.mark_incomplete(AuditFailureV1 {
            class: AuditFailureClassV1::Interrupted,
            code: "synthetic_process_interruption".to_string(),
        })
        .unwrap();
    }
    run
}

fn valid_run_json() -> Vec<u8> {
    let run = valid_run();
    serde_json::to_vec(&run).unwrap()
}

fn confirmed_remediation_json() -> Vec<u8> {
    let evidence = AuditEvidenceRefV1 {
        path: "src/api.rs".to_string(),
        start_line: 10,
        end_line: 20,
        content_sha256: "d".repeat(64),
    };
    let binding = AuditRunBindingV1 {
        project_id: "project-cli-remediation".to_string(),
        source_revision: "a".repeat(40),
        source_dirty: false,
        dirty_state_sha256: None,
        scope: vec!["src".to_string()],
        languages: vec!["rust".to_string()],
        profile: "backend".to_string(),
        task_id: "task-cli-remediation".to_string(),
        task_state_sha256: "b".repeat(64),
        operation_id: "operation-cli-remediation".to_string(),
    };
    let mut run = AuditRunV1::new(
        "00000000-0000-4000-8000-000000000011",
        binding,
        AuditMode::Focused,
    )
    .unwrap();
    let phase = AuditPhase::Reconnaissance;
    run.complete_phase(AuditPhaseReceiptV1 {
        phase,
        binding_sha256: run.binding_fingerprint().unwrap(),
        operation_id: run.binding.operation_id.clone(),
        timestamp: "2026-10-29T12:00:00Z".to_string(),
        artifact_ref: format!("audit-artifact://{}", phase.as_str()),
    })
    .unwrap();
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
    run.set_coverage_ledger(AuditCoverageLedgerV1 {
        schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        units,
    })
    .unwrap();
    let candidate = AuditCandidateV1 {
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
        evidence_refs: vec![evidence.clone()],
    };
    let fingerprint = run.add_candidate(candidate).unwrap();
    let phase_receipt = |run: &AuditRunV1, phase: AuditPhase| AuditPhaseReceiptV1 {
        phase,
        binding_sha256: run.binding_fingerprint().unwrap(),
        operation_id: run.binding.operation_id.clone(),
        timestamp: "2026-10-29T12:00:00Z".to_string(),
        artifact_ref: format!("audit-artifact://{}", phase.as_str()),
    };
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
            assessment: AuditFindingVerdictV1::Confirmed {
                severity: AuditSeverity::High,
                affected_principal_or_attacker: "authenticated user from another tenant"
                    .to_string(),
                protected_resource: "tenant-owned record".to_string(),
                impact: "cross-tenant disclosure".to_string(),
                source_path: "src/api.rs".to_string(),
                start_line: 10,
                end_line: 20,
                evidence_refs: vec![evidence.clone()],
                recommended_correction: "enforce tenant ownership in the server-side query"
                    .to_string(),
                safe_verification: "test cross-tenant access with an isolated fixture".to_string(),
            },
        },
    )
    .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::CandidateValidation))
        .unwrap();
    run.complete_phase(phase_receipt(&run, AuditPhase::StructuredOutput))
        .unwrap();
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
            evidence_refs: vec![evidence],
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

    let remediation_id = "00000000-0000-4000-8000-000000000012";
    let run_id_sha256 = run.run_id_sha256();
    let source_binding_sha256 = run.binding_fingerprint().unwrap();
    let request = json!({
        "schema_version": SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
        "audit_run": run,
        "remediation": {
            "schema_version": SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
            "remediation_id": remediation_id,
            "source_audit_run_id_sha256": run_id_sha256,
            "source_binding_sha256": source_binding_sha256,
            "finding_fingerprint": fingerprint,
            "status": "in_progress",
            "cycles": [],
            "blocker": null,
            "next_action": "wait for trusted sandbox capability"
        }
    });
    serde_json::to_vec(&request).unwrap()
}

#[test]
fn hidden_validator_reports_schema_only_and_does_not_claim_execution() {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-run"])
        .write_stdin(valid_run_json())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(
        result["validation_scope"],
        "schema_and_workflow_contract_only"
    );
    assert_eq!(result["run_status_claim"], "in_progress");
    assert!(result.get("run_status").is_none());
    assert_eq!(result["execution_status"], "not_run");
    assert_eq!(result["proof_gate_trace_claimed"], false);
    assert!(result.get("run_id").is_none());
    assert_eq!(result["run_id_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn hidden_validator_rejects_unknown_fields_without_echoing_input() {
    let mut run: Value = serde_json::from_slice(&valid_run_json()).unwrap();
    run["unexpected_secret"] = json!("must-not-appear-in-error-output");
    let result = Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-run"])
        .write_stdin(serde_json::to_vec(&run).unwrap())
        .assert()
        .failure();
    assert!(!String::from_utf8_lossy(&result.get_output().stderr)
        .contains("must-not-appear-in-error-output"));
}

#[test]
fn hidden_validator_rejects_needs_validation_with_a_severity_field() {
    let mut run: Value = serde_json::from_slice(&valid_run_json()).unwrap();
    run["findings"] = json!([{
        "candidate": {
            "root_cause_key": "tenant-ownership-check",
            "attack_class": "authentication_authorization_and_ownership",
            "language": "rust",
            "subsystem": "api",
            "protected_boundary": "authenticated-request-to-database",
            "lifecycle": "authorization_and_ownership",
            "source_path": "src/api.rs",
            "start_line": 10,
            "end_line": 20,
            "claim": "tenant ownership is not enforced",
            "author_ids": ["hunter"],
            "evidence_refs": [{
                "path": "src/api.rs",
                "start_line": 10,
                "end_line": 20,
                "content_sha256": "d".repeat(64)
            }]
        },
        "candidate_revision": 0,
        "candidate_reviews": [{
            "reviewer_id": "reviewer",
            "context_id": "fresh-review-context",
            "source_revision": "a".repeat(40),
            "binding_sha256": "f".repeat(64),
            "candidate_revision": 0,
            "assessment": {
                "verdict": "needs_validation",
                "unresolved_fact": "current authorization source is unavailable",
                "blocker": "authoritative ownership check is not visible",
                "severity": "High"
            }
        }],
        "independent_record_review": null
    }]);
    Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-run"])
        .write_stdin(serde_json::to_vec(&run).unwrap())
        .assert()
        .failure();
}

#[test]
fn hidden_validator_rejects_secret_like_run_id_without_echoing_it() {
    let mut run: Value = serde_json::from_slice(&valid_run_json()).unwrap();
    let secret_like_run_id = "sk-live-super-secret-looking-value";
    run["run_id"] = json!(secret_like_run_id);

    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-run"])
        .write_stdin(serde_json::to_vec(&run).unwrap())
        .assert()
        .failure()
        .get_output()
        .clone();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!combined.contains(secret_like_run_id));
}

#[test]
fn remediation_validator_is_schema_only_and_never_authorizes_fix_or_closure() {
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-remediation"])
        .write_stdin(confirmed_remediation_json())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result["validation_scope"], "schema_contract_only");
    assert_eq!(result["execution_status"], "not_run");
    assert_eq!(result["auto_fix_authorized"], false);
    assert_eq!(result["closure_authorized"], false);
    assert!(result["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|blocker| blocker == "trusted_evidence_and_sandbox_backend_unverified"));
    assert!(result.get("run_id").is_none());
    assert!(result.get("claim").is_none());
}

#[test]
fn remediation_validator_rejects_unknown_fields_without_echoing_them() {
    let mut request: Value = serde_json::from_slice(&confirmed_remediation_json()).unwrap();
    request["remediation"]["unexpected_secret"] = json!("do-not-reflect-this-value");
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-remediation"])
        .write_stdin(serde_json::to_vec(&request).unwrap())
        .assert()
        .failure()
        .get_output()
        .clone();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains("do-not-reflect-this-value"));
}

#[test]
fn remediation_validator_rejects_oversized_input() {
    let mut request = confirmed_remediation_json();
    request.resize(MAX_SECURITY_AUDIT_REMEDIATION_JSON_BYTES + 1, b' ');
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args(["security-audit", "validate-remediation"])
        .write_stdin(request)
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn checkpoint_cli_saves_then_revalidates_and_rejects_changed_source() {
    let temp = synthetic_repo();
    let root = temp.path();
    let output_path = root.join("audit-checkpoint.json");
    let run = run_bound_to_synthetic_repo(root);
    let save = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "save-checkpoint",
            "--repo-path",
            root.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--pending-work",
            "review source",
            "--next-action",
            "continue static analysis",
        ])
        .write_stdin(serde_json::to_vec(&run).unwrap())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let save: Value = serde_json::from_slice(&save).unwrap();
    assert_eq!(save["artifact_state"], "created");

    let resumed = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "resume-checkpoint",
            "--repo-path",
            root.to_str().unwrap(),
            "--input",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let resumed: Value = serde_json::from_slice(&resumed).unwrap();
    assert_eq!(resumed["can_resume"], true);
    assert_eq!(resumed["fresh_run_required"], false);

    std::fs::write(root.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
    let changed = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "resume-checkpoint",
            "--repo-path",
            root.to_str().unwrap(),
            "--input",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let changed: Value = serde_json::from_slice(&changed).unwrap();
    assert_eq!(changed["can_resume"], false);
    assert_eq!(changed["fresh_run_required"], true);
    assert_eq!(changed["changed_paths"], json!(["src/lib.rs"]));
}

#[test]
fn checkpoint_cli_revalidates_every_persisted_phase_boundary_after_interruption() {
    for boundary in 0..=AuditPhase::ordered().len() {
        let temp = synthetic_repo();
        let root = temp.path();
        let output_path = root.join(format!("phase-{boundary}-checkpoint.json"));
        let run = run_at_interrupted_phase_boundary(root, boundary);
        let run_bytes = serde_json::to_vec(&run).unwrap();
        let save = Command::cargo_bin("baron")
            .unwrap()
            .args([
                "security-audit",
                "save-checkpoint",
                "--repo-path",
                root.to_str().unwrap(),
                "--output",
                output_path.to_str().unwrap(),
                "--pending-work",
                "resume synthetic audit phase",
                "--next-action",
                "continue from the validated phase boundary",
            ])
            .write_stdin(run_bytes)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let save: Value = serde_json::from_slice(&save).unwrap();
        assert_eq!(save["artifact_state"], "created", "boundary {boundary}");

        let checkpoint_bytes = std::fs::read(&output_path).unwrap();
        let checkpoint: Value = serde_json::from_slice(&checkpoint_bytes).unwrap();
        let run = &checkpoint["run"];
        assert_eq!(
            run["phase_receipts"].as_array().unwrap().len(),
            boundary,
            "persisted phase receipt count at boundary {boundary}"
        );
        assert_eq!(
            checkpoint["phase"],
            AuditPhase::ordered()[boundary.min(AuditPhase::ordered().len() - 1)].as_str(),
            "checkpoint next phase at boundary {boundary}"
        );
        assert_eq!(
            run["status"],
            if boundary == AuditPhase::ordered().len() {
                "complete"
            } else {
                "incomplete"
            },
            "persisted run status at boundary {boundary}"
        );

        let expected_findings = if boundary >= 2 { 1 } else { 0 };
        assert_eq!(
            run["findings"].as_array().unwrap().len(),
            expected_findings,
            "finding count at boundary {boundary}"
        );
        assert_eq!(
            checkpoint["candidate_fingerprints"]
                .as_array()
                .unwrap()
                .len(),
            expected_findings,
            "candidate fingerprint count at boundary {boundary}"
        );
        if expected_findings == 1 {
            assert_eq!(
                run["findings"][0]["candidate_reviews"]
                    .as_array()
                    .unwrap()
                    .len(),
                usize::from(boundary >= 3),
                "candidate review count at boundary {boundary}"
            );
        }
        let expected_coverage_units = if boundary >= 2 {
            SECURITY_AUDIT_LIFECYCLE_SURFACES.len() * SHARED_ATTACK_CLASSES.len()
        } else {
            0
        };
        assert_eq!(
            run["coverage_ledger"]["units"]
                .as_array()
                .map_or(0, Vec::len),
            expected_coverage_units,
            "coverage unit count at boundary {boundary}"
        );

        let mut previous_assessment = None;
        for _ in 0..2 {
            let output = Command::cargo_bin("baron")
                .unwrap()
                .args([
                    "security-audit",
                    "resume-checkpoint",
                    "--repo-path",
                    root.to_str().unwrap(),
                    "--input",
                    output_path.to_str().unwrap(),
                ])
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();
            let assessment: Value = serde_json::from_slice(&output).unwrap();
            assert_eq!(assessment["run_status"], run["status"]);
            assert_eq!(assessment["fresh_run_required"], false);
            assert_eq!(
                assessment["can_resume"],
                boundary < AuditPhase::ordered().len(),
                "resume eligibility at boundary {boundary}"
            );
            if boundary < AuditPhase::ordered().len() {
                assert!(assessment["blockers"].as_array().unwrap().is_empty());
                assert_eq!(
                    assessment["next_safe_action"],
                    "continue from the validated phase boundary"
                );
            } else {
                assert_eq!(
                    assessment["blockers"],
                    json!(["run_state_requires_new_audit_decision"])
                );
            }
            if let Some(previous) = previous_assessment {
                assert_eq!(
                    assessment, previous,
                    "repeated resume assessment at boundary {boundary}"
                );
            }
            previous_assessment = Some(assessment);
        }
        assert_eq!(
            std::fs::read(&output_path).unwrap(),
            checkpoint_bytes,
            "resume assessment must not mutate checkpoint state at boundary {boundary}"
        );
    }
}

#[test]
fn report_cli_publishes_create_only_and_does_not_overwrite_user_edits() {
    let temp = tempfile::tempdir().unwrap();
    let output_path = temp.path().join("audit-report.json");
    let first = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "publish-report",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .write_stdin(valid_run_json())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let first: Value = serde_json::from_slice(&first).unwrap();
    assert_eq!(first["clean"], false);
    assert_eq!(first["application_task_blocked"], true);
    assert_eq!(first["artifact_state"], "created");

    let saved = std::fs::read(&output_path).unwrap();
    let report: Value = serde_json::from_slice(&saved).unwrap();
    assert_eq!(report["clean"], false);
    assert_eq!(report["proof_gate_trace_claimed"], false);

    std::fs::write(&output_path, b"user edit").unwrap();
    let failure = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "publish-report",
            "--output",
            output_path.to_str().unwrap(),
        ])
        .write_stdin(valid_run_json())
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(!String::from_utf8_lossy(&failure.stderr).contains("user edit"));
    assert_eq!(std::fs::read(&output_path).unwrap(), b"user edit");
}

#[test]
fn checkpoint_cli_rejects_malformed_input_without_reflecting_it() {
    let temp = tempfile::tempdir().unwrap();
    let secret = "raw-secret-in-malformed-json";
    let input_path = temp.path().join("malformed-checkpoint.json");
    std::fs::write(&input_path, format!("{{\"value\":\"{secret}\"}}")).unwrap();
    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "resume-checkpoint",
            "--repo-path",
            temp.path().to_str().unwrap(),
            "--input",
            input_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains(secret));
}

#[test]
fn checkpoint_cli_rejects_oversized_file_before_json_parsing() {
    let temp = tempfile::tempdir().unwrap();
    let input_path = temp.path().join("oversized-checkpoint.json");
    let file = std::fs::File::create(&input_path).unwrap();
    file.set_len(MAX_AUDIT_CHECKPOINT_JSON_BYTES as u64 + 1)
        .unwrap();
    drop(file);

    let output = Command::cargo_bin("baron")
        .unwrap()
        .args([
            "security-audit",
            "resume-checkpoint",
            "--repo-path",
            temp.path().to_str().unwrap(),
            "--input",
            input_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(combined.contains("checkpoint file exceeds the byte limit"));
    assert!(!combined.contains("checkpoint JSON is malformed"));
    assert_eq!(
        std::fs::metadata(&input_path).unwrap().len(),
        MAX_AUDIT_CHECKPOINT_JSON_BYTES as u64 + 1
    );
}
