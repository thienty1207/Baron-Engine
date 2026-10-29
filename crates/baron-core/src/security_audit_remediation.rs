//! Deterministic remediation-loop contracts for Baron security audits.
//!
//! This module validates serialized workflow state only. With no verified
//! target-code sandbox backend or trusted current-operation remediation
//! evidence, it cannot authorize edits, accept a test or re-audit claim, or
//! close a finding.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::security_audit_workflow::{
    validate_audit_run, AuditFindingRecordV1, AuditFindingVerdictV1, AuditRunBindingV1,
    AuditRunStatusV1, AuditRunV1, MAX_AUDIT_RUN_JSON_BYTES,
};

pub const SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION: u32 = 1;
pub const MAX_SECURITY_AUDIT_REMEDIATION_CYCLES: usize = 3;
pub const MAX_REMEDIATION_CHANGED_PATHS: usize = 128;
pub const MAX_SECURITY_AUDIT_REMEDIATION_JSON_BYTES: usize = MAX_AUDIT_RUN_JSON_BYTES + 1024 * 1024;

const MAX_TEXT_CHARS: usize = 4_000;
const MAX_SHORT_TEXT_CHARS: usize = 512;
const SPEC_08_05_BLOCKER: &str = "trusted_evidence_and_sandbox_backend_unverified";
const UNVERIFIED_SCHEMA_BLOCKER: &str = "serialized_remediation_evidence_is_unverified";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RemediationFixStatusV1 {
    Planned,
    Applied,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RemediationCheckStatusV1 {
    NotRun,
    Passed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum RemediationReauditStatusV1 {
    NotRun,
    StillConfirmed,
    NotConfirmed,
    NeedsValidation,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SecurityAuditRemediationStateV1 {
    InProgress,
    Blocked,
    Incomplete,
}

impl SecurityAuditRemediationStateV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Incomplete => "incomplete",
        }
    }
}

/// One claimed fix-and-recheck cycle. All evidence references are opaque
/// SHA-256 identifiers, not proof that a command or review actually ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityAuditRemediationCycleV1 {
    pub cycle_number: u8,
    pub source_before_binding_sha256: String,
    pub source_after_binding: Option<AuditRunBindingV1>,
    pub changed_paths: Vec<String>,
    pub source_diff_sha256: Option<String>,
    pub fixer_id: Option<String>,
    pub fix_status: RemediationFixStatusV1,
    pub fix_summary: String,
    pub regression_status: RemediationCheckStatusV1,
    pub regression_evidence_sha256: Option<String>,
    pub regression_binding_sha256: Option<String>,
    pub re_audit_status: RemediationReauditStatusV1,
    pub re_audit_binding_sha256: Option<String>,
    pub re_audit_run_id_sha256: Option<String>,
    pub security_auditor_status: RemediationCheckStatusV1,
    pub security_auditor_evidence_sha256: Option<String>,
    pub security_auditor_id: Option<String>,
    pub security_auditor_context_id: Option<String>,
    pub security_auditor_binding_sha256: Option<String>,
    pub proof_record_id_sha256: Option<String>,
    pub trace_record_id_sha256: Option<String>,
    pub blocker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityAuditRemediationV1 {
    pub schema_version: u32,
    pub remediation_id: String,
    pub source_audit_run_id_sha256: String,
    pub source_binding_sha256: String,
    pub finding_fingerprint: String,
    pub status: SecurityAuditRemediationStateV1,
    pub cycles: Vec<SecurityAuditRemediationCycleV1>,
    pub blocker: Option<String>,
    pub next_action: String,
}

/// A bounded schema-validation request containing the source audit and one
/// remediation timeline. It does not authorize target-code work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityAuditRemediationRequestV1 {
    pub schema_version: u32,
    pub audit_run: AuditRunV1,
    pub remediation: SecurityAuditRemediationV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityAuditRemediationValidationV1 {
    pub schema_version: u32,
    pub validation_scope: String,
    pub source_audit_run_id_sha256: String,
    pub remediation_id_sha256: String,
    pub finding_fingerprint: String,
    pub status_claim: String,
    pub schema_eligible: bool,
    pub cycle_count: usize,
    pub max_cycles: usize,
    pub execution_status: String,
    pub auto_fix_authorized: bool,
    pub closure_authorized: bool,
    pub blockers: Vec<String>,
}

pub fn validate_security_audit_remediation_json(
    input: &[u8],
) -> Result<SecurityAuditRemediationValidationV1> {
    ensure!(
        input.len() <= MAX_SECURITY_AUDIT_REMEDIATION_JSON_BYTES,
        "security-audit remediation input exceeds its byte limit"
    );
    let request: SecurityAuditRemediationRequestV1 = serde_json::from_slice(input)
        .context("could not parse security-audit remediation input")?;
    validate_security_audit_remediation(&request)
}

pub fn validate_security_audit_remediation(
    request: &SecurityAuditRemediationRequestV1,
) -> Result<SecurityAuditRemediationValidationV1> {
    ensure!(
        request.schema_version == SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
        "unsupported security-audit remediation request schema"
    );
    validate_audit_run(&request.audit_run)?;
    ensure!(
        request.audit_run.status == AuditRunStatusV1::Complete,
        "remediation requires a schema-complete source audit run"
    );

    let remediation = &request.remediation;
    ensure!(
        remediation.schema_version == SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
        "unsupported security-audit remediation schema"
    );
    validate_canonical_uuid(&remediation.remediation_id)?;
    validate_sha256(
        &remediation.source_audit_run_id_sha256,
        "source audit run digest",
    )?;
    validate_sha256(&remediation.source_binding_sha256, "source binding digest")?;
    validate_finding_fingerprint(&remediation.finding_fingerprint)?;
    validate_short_text("next action", &remediation.next_action, MAX_TEXT_CHARS)?;
    validate_optional_blocker(remediation.blocker.as_deref())?;
    match remediation.status {
        SecurityAuditRemediationStateV1::InProgress => ensure!(
            remediation.blocker.is_none(),
            "in-progress remediation cannot carry a terminal blocker"
        ),
        SecurityAuditRemediationStateV1::Blocked | SecurityAuditRemediationStateV1::Incomplete => {
            ensure!(
                remediation.blocker.is_some(),
                "blocked or incomplete remediation requires an explicit blocker"
            )
        }
    }

    let run_digest = request.audit_run.run_id_sha256();
    ensure!(
        remediation.source_audit_run_id_sha256 == run_digest,
        "remediation is bound to a different source audit run"
    );
    let source_binding = request.audit_run.binding_fingerprint()?;
    ensure!(
        remediation.source_binding_sha256 == source_binding,
        "remediation is bound to different project, source, or task state"
    );

    let matching_findings = request
        .audit_run
        .findings
        .iter()
        .filter(|finding| {
            finding
                .fingerprint(&request.audit_run.binding)
                .is_ok_and(|fingerprint| fingerprint == remediation.finding_fingerprint)
        })
        .collect::<Vec<_>>();
    ensure!(
        matching_findings.len() == 1,
        "remediation must select exactly one source audit finding"
    );
    let finding = matching_findings[0];
    let verdict = finding.final_verdict(request.audit_run.mode, &request.audit_run.binding)?;
    ensure!(
        matches!(verdict, AuditFindingVerdictV1::Confirmed { .. }),
        "only an independently confirmed finding can enter remediation"
    );
    ensure!(
        finding
            .independent_record_review
            .as_ref()
            .is_some_and(|review| review.accepted && !review.correction_requires_revalidation),
        "confirmed finding lacks an accepted independent record review"
    );

    validate_cycles(
        &remediation.cycles,
        &request.audit_run.binding,
        &run_digest,
        finding,
        remediation.status,
        remediation.blocker.as_deref(),
    )?;

    let mut blockers = vec![
        UNVERIFIED_SCHEMA_BLOCKER.to_string(),
        SPEC_08_05_BLOCKER.to_string(),
    ];
    blockers.sort();
    Ok(SecurityAuditRemediationValidationV1 {
        schema_version: SECURITY_AUDIT_REMEDIATION_SCHEMA_VERSION,
        validation_scope: "schema_contract_only".to_string(),
        source_audit_run_id_sha256: run_digest,
        remediation_id_sha256: sha256(remediation.remediation_id.as_bytes()),
        finding_fingerprint: remediation.finding_fingerprint.clone(),
        status_claim: remediation.status.as_str().to_string(),
        schema_eligible: true,
        cycle_count: remediation.cycles.len(),
        max_cycles: MAX_SECURITY_AUDIT_REMEDIATION_CYCLES,
        execution_status: "not_run".to_string(),
        auto_fix_authorized: false,
        closure_authorized: false,
        blockers,
    })
}

fn validate_cycles(
    cycles: &[SecurityAuditRemediationCycleV1],
    initial_binding: &AuditRunBindingV1,
    source_audit_run_id_sha256: &str,
    finding: &AuditFindingRecordV1,
    remediation_status: SecurityAuditRemediationStateV1,
    remediation_blocker: Option<&str>,
) -> Result<()> {
    ensure!(
        cycles.len() <= MAX_SECURITY_AUDIT_REMEDIATION_CYCLES,
        "remediation exceeds the maximum number of fix/re-audit cycles"
    );

    let mut expected_source_binding = initial_binding.clone();
    let mut expected_source = source_binding_fingerprint(initial_binding)?;
    let mut regression_evidence = BTreeSet::new();
    let mut re_audit_run_ids = BTreeSet::from([source_audit_run_id_sha256.to_string()]);
    let mut must_stop = false;
    for (index, cycle) in cycles.iter().enumerate() {
        if index > 0 {
            let prior = &cycles[index - 1];
            ensure!(
                prior.fix_status == RemediationFixStatusV1::Applied
                    && prior.regression_status == RemediationCheckStatusV1::Passed
                    && prior.re_audit_status == RemediationReauditStatusV1::StillConfirmed
                    && prior.blocker.is_none()
                    && prior.security_auditor_status == RemediationCheckStatusV1::NotRun
                    && prior.security_auditor_evidence_sha256.is_none()
                    && prior.security_auditor_id.is_none()
                    && prior.security_auditor_context_id.is_none()
                    && prior.security_auditor_binding_sha256.is_none()
                    && prior.proof_record_id_sha256.is_none()
                    && prior.trace_record_id_sha256.is_none(),
                "a new fix cycle may follow only a passing regression and still-confirmed re-audit"
            );
        }
        ensure!(
            usize::from(cycle.cycle_number) == index + 1,
            "remediation cycles must be sequential and unique"
        );
        validate_sha256(
            &cycle.source_before_binding_sha256,
            "cycle source-before digest",
        )?;
        ensure!(
            cycle.source_before_binding_sha256 == expected_source,
            "remediation cycle does not continue from the prior source binding"
        );
        validate_short_text("fix summary", &cycle.fix_summary, MAX_TEXT_CHARS)?;
        validate_optional_blocker(cycle.blocker.as_deref())?;
        validate_changed_paths(&cycle.changed_paths, &initial_binding.scope)?;
        validate_fix_actor(cycle, finding)?;
        if cycle_requires_stop(cycle)
            || (cycle.cycle_number as usize == MAX_SECURITY_AUDIT_REMEDIATION_CYCLES
                && cycle.re_audit_status == RemediationReauditStatusV1::StillConfirmed)
        {
            ensure!(
                cycle.blocker.is_some(),
                "a stopped or exhausted remediation cycle requires an explicit blocker"
            );
        }

        match cycle.fix_status {
            RemediationFixStatusV1::Planned => {
                ensure!(
                    cycle.changed_paths.is_empty()
                        && cycle.source_after_binding.is_none()
                        && cycle.source_diff_sha256.is_none(),
                    "planned remediation cycle cannot claim changed source"
                );
                ensure_checks_not_run(cycle)?;
                must_stop = false;
            }
            RemediationFixStatusV1::Applied => {
                ensure!(
                    !cycle.changed_paths.is_empty(),
                    "applied fix requires at least one changed in-scope path"
                );
                let after_binding = cycle.source_after_binding.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("applied fix requires its resulting source binding")
                })?;
                let after = validate_source_binding_transition(
                    initial_binding,
                    &expected_source_binding,
                    after_binding,
                )?;
                let diff = cycle
                    .source_diff_sha256
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("applied fix requires a source diff digest"))?;
                validate_sha256(diff, "source diff digest")?;
                validate_regression(cycle, &after)?;
                validate_reaudit(cycle, &after)?;
                validate_security_gate_claim(cycle, &after, finding)?;
                if let Some(evidence) = cycle.regression_evidence_sha256.as_ref() {
                    ensure!(
                        regression_evidence.insert(evidence),
                        "regression evidence cannot be reused across remediation cycles"
                    );
                }
                if let Some(run_id) = cycle.re_audit_run_id_sha256.as_ref() {
                    ensure!(
                        re_audit_run_ids.insert(run_id.clone()),
                        "re-audit run identifier cannot be reused"
                    );
                }
                expected_source = after;
                expected_source_binding = after_binding.clone();
                must_stop = cycle_requires_stop(cycle);
            }
            RemediationFixStatusV1::Failed | RemediationFixStatusV1::Blocked => {
                if let Some(after_binding) = cycle.source_after_binding.as_ref() {
                    let after = validate_source_binding_transition(
                        initial_binding,
                        &expected_source_binding,
                        after_binding,
                    )?;
                    ensure!(
                        !cycle.changed_paths.is_empty(),
                        "partial source change requires affected paths"
                    );
                    let diff = cycle.source_diff_sha256.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("partial source change requires a diff digest")
                    })?;
                    validate_sha256(diff, "source diff digest")?;
                    expected_source = after;
                    expected_source_binding = after_binding.clone();
                } else {
                    ensure!(
                        cycle.changed_paths.is_empty() && cycle.source_diff_sha256.is_none(),
                        "changed paths require a resulting source binding"
                    );
                }
                ensure_checks_not_run(cycle)?;
                ensure!(
                    cycle.blocker.is_some(),
                    "failed or blocked fix requires an explicit cycle blocker"
                );
                must_stop = true;
            }
        }
    }

    validate_security_reviewer_independence(cycles)?;

    if let Some(last) = cycles.last() {
        if last.blocker.is_some() {
            ensure!(
                remediation_status != SecurityAuditRemediationStateV1::InProgress,
                "a remediation cycle with a blocker cannot remain in progress"
            );
        }
        if remediation_status != SecurityAuditRemediationStateV1::InProgress {
            ensure!(
                last.blocker.is_some(),
                "blocked or incomplete remediation with cycles requires a final cycle blocker"
            );
        }
        if last.fix_status == RemediationFixStatusV1::Applied
            && last.re_audit_status == RemediationReauditStatusV1::StillConfirmed
            && cycles.len() == MAX_SECURITY_AUDIT_REMEDIATION_CYCLES
        {
            must_stop = true;
            ensure!(
                remediation_blocker.is_some(),
                "third still-confirmed cycle must retain an explicit blocker"
            );
        }
    }
    if must_stop {
        ensure!(
            remediation_status != SecurityAuditRemediationStateV1::InProgress,
            "failed, blocked, or exhausted remediation cannot remain in progress"
        );
        ensure!(
            remediation_blocker.is_some(),
            "stopped remediation requires an explicit top-level blocker"
        );
    }
    Ok(())
}

fn validate_regression(
    cycle: &SecurityAuditRemediationCycleV1,
    post_fix_binding: &str,
) -> Result<()> {
    validate_evidence_status(
        cycle.regression_status,
        cycle.regression_evidence_sha256.as_deref(),
        "regression",
    )?;
    match cycle.regression_binding_sha256.as_deref() {
        Some(binding) => {
            validate_sha256(binding, "regression source binding")?;
            ensure!(
                binding == post_fix_binding,
                "regression result must bind to the exact post-fix source"
            );
        }
        None => ensure!(
            cycle.regression_status == RemediationCheckStatusV1::NotRun,
            "regression result requires its source binding"
        ),
    }
    if cycle.regression_status != RemediationCheckStatusV1::Passed {
        ensure!(
            cycle.re_audit_status == RemediationReauditStatusV1::NotRun,
            "re-audit cannot follow a failed, blocked, or unrun regression"
        );
        ensure!(
            cycle.re_audit_binding_sha256.is_none() && cycle.re_audit_run_id_sha256.is_none(),
            "re-audit references cannot be present before regression passes"
        );
        ensure!(
            cycle.security_auditor_status == RemediationCheckStatusV1::NotRun
                && cycle.security_auditor_evidence_sha256.is_none()
                && cycle.proof_record_id_sha256.is_none()
                && cycle.trace_record_id_sha256.is_none(),
            "gate, proof, or trace claims cannot precede a passing regression and re-audit"
        );
    }
    Ok(())
}

fn source_binding_fingerprint(binding: &AuditRunBindingV1) -> Result<String> {
    binding.validate()?;
    Ok(sha256(&serde_json::to_vec(binding)?))
}

fn validate_source_binding_transition(
    initial: &AuditRunBindingV1,
    previous: &AuditRunBindingV1,
    after: &AuditRunBindingV1,
) -> Result<String> {
    after.validate()?;
    ensure!(
        after.project_id == initial.project_id
            && after.task_id == initial.task_id
            && after.operation_id == initial.operation_id
            && after.scope == initial.scope
            && after.languages == initial.languages
            && after.profile == initial.profile,
        "post-fix source binding must preserve project, task operation, profile, languages, and authorized scope"
    );
    let previous_digest = source_binding_fingerprint(previous)?;
    let after_digest = source_binding_fingerprint(after)?;
    ensure!(
        after_digest != previous_digest,
        "applied fix must change the source binding"
    );
    Ok(after_digest)
}

fn validate_security_reviewer_independence(
    cycles: &[SecurityAuditRemediationCycleV1],
) -> Result<()> {
    let fixer_ids = cycles
        .iter()
        .filter_map(|cycle| cycle.fixer_id.as_deref())
        .collect::<BTreeSet<_>>();
    for cycle in cycles {
        if cycle.security_auditor_status != RemediationCheckStatusV1::NotRun {
            let reviewer = cycle.security_auditor_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("security-auditor result requires its reviewer identity")
            })?;
            ensure!(
                !fixer_ids.contains(reviewer),
                "security-auditor must be independent from every remediation fix implementer"
            );
        }
    }
    Ok(())
}

fn validate_reaudit(cycle: &SecurityAuditRemediationCycleV1, post_fix_binding: &str) -> Result<()> {
    match cycle.re_audit_status {
        RemediationReauditStatusV1::NotRun => {
            ensure!(
                cycle.re_audit_binding_sha256.is_none() && cycle.re_audit_run_id_sha256.is_none(),
                "unrun re-audit cannot carry evidence references"
            );
            ensure!(
                cycle.security_auditor_status == RemediationCheckStatusV1::NotRun
                    && cycle.security_auditor_evidence_sha256.is_none()
                    && cycle.security_auditor_id.is_none()
                    && cycle.security_auditor_context_id.is_none()
                    && cycle.security_auditor_binding_sha256.is_none()
                    && cycle.proof_record_id_sha256.is_none()
                    && cycle.trace_record_id_sha256.is_none(),
                "gate, proof, and trace claims require a not-confirmed re-audit"
            );
        }
        status => {
            ensure!(
                cycle.regression_status == RemediationCheckStatusV1::Passed,
                "re-audit requires a passing regression check"
            );
            let binding = cycle
                .re_audit_binding_sha256
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("re-audit requires its source binding"))?;
            let run = cycle
                .re_audit_run_id_sha256
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("re-audit requires a run digest"))?;
            validate_sha256(binding, "re-audit binding digest")?;
            validate_sha256(run, "re-audit run digest")?;
            ensure!(
                binding == post_fix_binding,
                "re-audit must bind to the exact post-fix source"
            );
            if status != RemediationReauditStatusV1::NotConfirmed {
                ensure!(
                    cycle.security_auditor_status == RemediationCheckStatusV1::NotRun
                        && cycle.security_auditor_evidence_sha256.is_none()
                        && cycle.security_auditor_id.is_none()
                        && cycle.security_auditor_context_id.is_none()
                        && cycle.security_auditor_binding_sha256.is_none()
                        && cycle.proof_record_id_sha256.is_none()
                        && cycle.trace_record_id_sha256.is_none(),
                    "gate, proof, and trace claims require a not-confirmed re-audit"
                );
            }
        }
    }
    Ok(())
}

fn validate_fix_actor(
    cycle: &SecurityAuditRemediationCycleV1,
    finding: &AuditFindingRecordV1,
) -> Result<()> {
    match cycle.fix_status {
        RemediationFixStatusV1::Planned => ensure!(
            cycle.fixer_id.is_none(),
            "planned remediation cannot claim an implementation actor"
        ),
        RemediationFixStatusV1::Applied | RemediationFixStatusV1::Failed => ensure!(
            cycle.fixer_id.is_some(),
            "an applied or failed fix requires its implementation actor"
        ),
        RemediationFixStatusV1::Blocked => ensure!(
            cycle.source_after_binding.is_none() || cycle.fixer_id.is_some(),
            "a blocked partial fix requires its implementation actor"
        ),
    }
    if let Some(fixer_id) = cycle.fixer_id.as_deref() {
        validate_short_text("fixer identity", fixer_id, MAX_SHORT_TEXT_CHARS)?;
        ensure_distinct_from_audit_actors(fixer_id, finding, "fixer")?;
    }
    Ok(())
}

fn validate_security_gate_claim(
    cycle: &SecurityAuditRemediationCycleV1,
    post_fix_binding: &str,
    finding: &AuditFindingRecordV1,
) -> Result<()> {
    if cycle.re_audit_status != RemediationReauditStatusV1::NotConfirmed {
        ensure!(
            cycle.security_auditor_status == RemediationCheckStatusV1::NotRun
                && cycle.security_auditor_evidence_sha256.is_none()
                && cycle.security_auditor_id.is_none()
                && cycle.security_auditor_context_id.is_none()
                && cycle.security_auditor_binding_sha256.is_none()
                && cycle.proof_record_id_sha256.is_none()
                && cycle.trace_record_id_sha256.is_none(),
            "gate, proof, and trace claims require a not-confirmed re-audit"
        );
        return Ok(());
    }

    validate_evidence_status(
        cycle.security_auditor_status,
        cycle.security_auditor_evidence_sha256.as_deref(),
        "security-auditor gate",
    )?;
    match cycle.security_auditor_status {
        RemediationCheckStatusV1::NotRun => ensure!(
            cycle.security_auditor_id.is_none()
                && cycle.security_auditor_context_id.is_none()
                && cycle.security_auditor_binding_sha256.is_none()
                && cycle.proof_record_id_sha256.is_none()
                && cycle.trace_record_id_sha256.is_none(),
            "an unrun security gate cannot carry reviewer, binding, proof, or trace claims"
        ),
        RemediationCheckStatusV1::Passed
        | RemediationCheckStatusV1::Failed
        | RemediationCheckStatusV1::Blocked => {
            let reviewer = cycle.security_auditor_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!("security-auditor result requires its reviewer identity")
            })?;
            let context = cycle
                .security_auditor_context_id
                .as_deref()
                .ok_or_else(|| {
                    anyhow::anyhow!("security-auditor result requires its fresh context identity")
                })?;
            let binding = cycle
                .security_auditor_binding_sha256
                .as_deref()
                .ok_or_else(|| {
                    anyhow::anyhow!("security-auditor result requires its source binding")
                })?;
            validate_short_text("security-auditor identity", reviewer, MAX_SHORT_TEXT_CHARS)?;
            validate_short_text(
                "security-auditor context identity",
                context,
                MAX_SHORT_TEXT_CHARS,
            )?;
            validate_sha256(binding, "security-auditor source binding")?;
            ensure!(
                binding == post_fix_binding,
                "security-auditor result must bind to the exact post-fix source"
            );
            ensure_distinct_from_audit_actors(reviewer, finding, "security-auditor")?;
            if let Some(fixer_id) = cycle.fixer_id.as_deref() {
                ensure!(
                    reviewer != fixer_id,
                    "security-auditor must be independent from the fix implementer"
                );
            }
            ensure!(
                finding
                    .candidate_reviews
                    .iter()
                    .all(|review| review.context_id != context)
                    && finding
                        .independent_record_review
                        .as_ref()
                        .is_none_or(|review| review.context_id != context),
                "security-auditor must use a fresh review context"
            );
            if cycle.security_auditor_status != RemediationCheckStatusV1::Passed {
                ensure!(
                    cycle.proof_record_id_sha256.is_none()
                        && cycle.trace_record_id_sha256.is_none(),
                    "proof and trace claims require a passing security-auditor gate"
                );
            }
        }
    }
    if let Some(proof) = cycle.proof_record_id_sha256.as_deref() {
        validate_sha256(proof, "proof record digest")?;
    }
    if let Some(trace) = cycle.trace_record_id_sha256.as_deref() {
        validate_sha256(trace, "trace record digest")?;
    }
    Ok(())
}

fn ensure_distinct_from_audit_actors(
    actor_id: &str,
    finding: &AuditFindingRecordV1,
    label: &str,
) -> Result<()> {
    ensure!(
        !finding
            .candidate
            .author_ids
            .iter()
            .any(|author| author == actor_id),
        "{label} must be independent from the finding author"
    );
    ensure!(
        !finding
            .candidate_reviews
            .iter()
            .any(|review| review.reviewer_id == actor_id),
        "{label} must be independent from candidate validators"
    );
    ensure!(
        finding
            .independent_record_review
            .as_ref()
            .is_none_or(|review| review.reviewer_id != actor_id),
        "{label} must be independent from the record verifier"
    );
    Ok(())
}

fn validate_evidence_status(
    status: RemediationCheckStatusV1,
    evidence: Option<&str>,
    label: &str,
) -> Result<()> {
    match status {
        RemediationCheckStatusV1::NotRun => ensure!(
            evidence.is_none(),
            "{label} cannot carry evidence when it has not run"
        ),
        RemediationCheckStatusV1::Passed
        | RemediationCheckStatusV1::Failed
        | RemediationCheckStatusV1::Blocked => {
            let evidence = evidence.ok_or_else(|| {
                anyhow::anyhow!("{label} result requires a bounded evidence digest")
            })?;
            validate_sha256(evidence, label)?;
        }
    }
    Ok(())
}

fn ensure_checks_not_run(cycle: &SecurityAuditRemediationCycleV1) -> Result<()> {
    ensure!(
        cycle.regression_status == RemediationCheckStatusV1::NotRun
            && cycle.regression_evidence_sha256.is_none()
            && cycle.regression_binding_sha256.is_none()
            && cycle.re_audit_status == RemediationReauditStatusV1::NotRun
            && cycle.re_audit_binding_sha256.is_none()
            && cycle.re_audit_run_id_sha256.is_none()
            && cycle.security_auditor_status == RemediationCheckStatusV1::NotRun
            && cycle.security_auditor_evidence_sha256.is_none()
            && cycle.security_auditor_id.is_none()
            && cycle.security_auditor_context_id.is_none()
            && cycle.security_auditor_binding_sha256.is_none()
            && cycle.proof_record_id_sha256.is_none()
            && cycle.trace_record_id_sha256.is_none(),
        "remediation checks cannot run before a fix is applied"
    );
    Ok(())
}

fn cycle_requires_stop(cycle: &SecurityAuditRemediationCycleV1) -> bool {
    matches!(
        cycle.fix_status,
        RemediationFixStatusV1::Failed | RemediationFixStatusV1::Blocked
    ) || matches!(
        cycle.regression_status,
        RemediationCheckStatusV1::Failed | RemediationCheckStatusV1::Blocked
    ) || matches!(
        cycle.re_audit_status,
        RemediationReauditStatusV1::NeedsValidation
            | RemediationReauditStatusV1::Failed
            | RemediationReauditStatusV1::Blocked
    ) || matches!(
        cycle.security_auditor_status,
        RemediationCheckStatusV1::Failed | RemediationCheckStatusV1::Blocked
    )
}

fn validate_changed_paths(paths: &[String], authorized_scope: &[String]) -> Result<()> {
    ensure!(
        paths.len() <= MAX_REMEDIATION_CHANGED_PATHS,
        "remediation changed-path list exceeds its item limit"
    );
    let mut unique = BTreeSet::new();
    for path in paths {
        validate_relative_path(path, "changed path")?;
        ensure!(path != ".", "changed path must name a repository file");
        ensure!(unique.insert(path), "duplicate remediation changed path");
        ensure!(
            authorized_scope
                .iter()
                .any(|scope| path_is_within(scope, path)),
            "remediation changed path is outside the authorized audit scope"
        );
    }
    Ok(())
}

fn validate_relative_path(value: &str, label: &str) -> Result<()> {
    validate_short_text(label, value, MAX_SHORT_TEXT_CHARS)?;
    ensure!(
        !value.starts_with('/')
            && !value.starts_with('\\')
            && !value.contains('\\')
            && !value.contains(':'),
        "{label} must be a canonical repository-relative path"
    );
    ensure!(
        value == "."
            || value
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != ".."),
        "{label} contains an unsafe path component"
    );
    let path = Path::new(value);
    ensure!(!path.is_absolute(), "{label} must be repository-relative");
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("{label} contains an unsafe path component")
            }
        }
    }
    Ok(())
}

fn path_is_within(scope: &str, path: &str) -> bool {
    let scope = scope.trim_end_matches('/');
    scope == "."
        || path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn validate_optional_blocker(blocker: Option<&str>) -> Result<()> {
    if let Some(blocker) = blocker {
        validate_short_text("remediation blocker", blocker, MAX_SHORT_TEXT_CHARS)?;
    }
    Ok(())
}

fn validate_short_text(label: &str, value: &str, limit: usize) -> Result<()> {
    ensure!(!value.trim().is_empty(), "{label} must not be empty");
    ensure!(
        value.chars().count() <= limit,
        "{label} exceeds its character limit"
    );
    ensure!(
        !value.chars().any(char::is_control),
        "{label} contains control characters"
    );
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    ensure!(
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "{label} must be a lowercase SHA-256 digest"
    );
    Ok(())
}

fn validate_finding_fingerprint(value: &str) -> Result<()> {
    let digest = value.strip_prefix("finding-").ok_or_else(|| {
        anyhow::anyhow!("finding fingerprint must use the canonical finding- prefix")
    })?;
    validate_sha256(digest, "finding fingerprint digest")
}

fn validate_canonical_uuid(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    ensure!(bytes.len() == 36, "remediation id must be a canonical UUID");
    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            ensure!(byte == b'-', "remediation id must be a canonical UUID");
        } else {
            ensure!(
                byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase(),
                "remediation id must be a canonical lowercase UUID"
            );
        }
    }
    ensure!(
        matches!(bytes[14], b'1'..=b'8') && matches!(bytes[19], b'8' | b'9' | b'a' | b'b'),
        "remediation id must use a supported UUID version and variant"
    );
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
