//! Baron-owned, deterministic contracts for security-audit runs.
//!
//! These types validate audit state and evidence. They do not inspect or run
//! target code, invoke an AI model, or by themselves prove that a check ran.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use anyhow::{bail, ensure, Result};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::continuity::{record_continuity_checkpoint_for_event, ContinuityPacket};
use crate::operation::OperationContext;
use crate::security_audit::SHARED_ATTACK_CLASSES;
use crate::vault::VaultContext;

pub const SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION: u32 = 1;
pub const MAX_AUDIT_RUN_JSON_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_AUDIT_SCOPE_ITEMS: usize = 128;
pub const MAX_COVERAGE_UNITS: usize = 10_000;
pub const MAX_AUDIT_FINDINGS: usize = 1_000;
pub const MAX_AUDIT_EVIDENCE_REFS: usize = 64;
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_SHORT_TEXT_CHARS: usize = 512;

const AUDIT_PHASES: [AuditPhase; 6] = [
    AuditPhase::Reconnaissance,
    AuditPhase::CoverageLedHunting,
    AuditPhase::CandidateValidation,
    AuditPhase::StructuredOutput,
    AuditPhase::IndependentRecordVerification,
    AuditPhase::Reporting,
];

pub const SECURITY_AUDIT_LIFECYCLE_SURFACES: [&str; 6] = [
    "inputs_and_entrypoints",
    "validation_and_transforms",
    "authorization_and_ownership",
    "sensitive_sinks_and_persistence",
    "errors_logging_and_outputs",
    "configuration_and_deployment",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditMode {
    Focused,
    Standard,
    Deep,
}

impl AuditMode {
    pub const fn minimum_candidate_verifiers(self) -> usize {
        match self {
            Self::Focused | Self::Standard => 1,
            Self::Deep => 2,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focused => "focused",
            Self::Standard => "standard",
            Self::Deep => "deep",
        }
    }
}

/// The evidence boundary currently provided by the workflow contract.
/// SPEC-08-05 must add verification before any report can claim execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditValidationScopeV1 {
    SchemaAndWorkflowContractOnly,
}

/// Whether a serialized run contains receipts. Receipt presence does not
/// establish that the referenced operation or artifact was actually verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditReportExecutionStatusV1 {
    NotRun,
    EvidenceRecordedUnverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditPhase {
    Reconnaissance,
    CoverageLedHunting,
    CandidateValidation,
    StructuredOutput,
    IndependentRecordVerification,
    Reporting,
}

impl AuditPhase {
    pub const fn ordered() -> [Self; 6] {
        AUDIT_PHASES
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reconnaissance => "reconnaissance",
            Self::CoverageLedHunting => "coverage_led_hunting",
            Self::CandidateValidation => "candidate_validation",
            Self::StructuredOutput => "structured_output",
            Self::IndependentRecordVerification => "independent_record_verification",
            Self::Reporting => "reporting",
        }
    }

    fn at(index: usize) -> Self {
        AUDIT_PHASES[index.min(AUDIT_PHASES.len() - 1)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditRunStatusV1 {
    InProgress,
    Blocked,
    Incomplete,
    Failed,
    Complete,
}

impl AuditRunStatusV1 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Incomplete => "incomplete",
            Self::Failed => "failed",
            Self::Complete => "complete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditFailureClassV1 {
    CapabilityUnavailable,
    SandboxUnavailable,
    UnsupportedSurface,
    SourceChanged,
    Interrupted,
    MalformedEvidence,
    VerifierDisagreement,
    TargetCommandFailed,
    TargetCommandTimedOut,
    RegressionFailed,
    ReportPublicationFailed,
    SecretRedactionRequired,
    ResourceLimitReached,
    AgentLimitReached,
    CoverageIncomplete,
    InternalFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditFailureV1 {
    pub class: AuditFailureClassV1,
    /// Stable, non-sensitive machine-readable reason. Raw command output,
    /// user input, and environment values must never be stored here.
    pub code: String,
}

/// Exact source and Baron operation identity for one audit run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRunBindingV1 {
    pub project_id: String,
    /// Full Git object ID (SHA-1 or SHA-256 repository format).
    pub source_revision: String,
    pub source_dirty: bool,
    /// Present only when the source checkout contains local changes.
    pub dirty_state_sha256: Option<String>,
    /// Canonical, authorized repository-relative scope.
    pub scope: Vec<String>,
    /// Canonical, lowercase set of in-scope detected languages.
    pub languages: Vec<String>,
    pub profile: String,
    pub task_id: String,
    /// Digest of the exact Task State used to prepare this run.
    pub task_state_sha256: String,
    pub operation_id: String,
}

impl AuditRunBindingV1 {
    fn canonicalize(&mut self) {
        self.scope = self
            .scope
            .iter()
            .map(|path| {
                let normalized = path.replace('\\', "/");
                if normalized == "." {
                    normalized
                } else {
                    normalized.trim_end_matches('/').to_string()
                }
            })
            .collect();
        self.scope.sort();
        self.scope.dedup();
        self.languages = self
            .languages
            .iter()
            .map(|language| language.trim().to_ascii_lowercase())
            .collect();
        self.languages.sort();
        self.languages.dedup();
    }

    pub fn validate(&self) -> Result<()> {
        validate_identifier("project_id", &self.project_id, MAX_SHORT_TEXT_CHARS)?;
        validate_identifier("task_id", &self.task_id, MAX_SHORT_TEXT_CHARS)?;
        validate_identifier("operation_id", &self.operation_id, MAX_SHORT_TEXT_CHARS)?;
        validate_identifier("profile", &self.profile, MAX_SHORT_TEXT_CHARS)?;
        ensure!(
            matches!(self.source_revision.len(), 40 | 64)
                && is_hex(&self.source_revision)
                && self.source_revision == self.source_revision.to_ascii_lowercase(),
            "source revision must be a full 40- or 64-character Git object ID"
        );
        ensure!(
            is_sha256(&self.task_state_sha256),
            "Task State binding must be a SHA-256 digest"
        );
        if self.source_dirty {
            ensure!(
                self.dirty_state_sha256.is_some(),
                "dirty checkout requires an exact dirty-state digest"
            );
        } else {
            ensure!(
                self.dirty_state_sha256.is_none(),
                "clean checkout cannot carry a dirty-state digest"
            );
        }
        if let Some(dirty_hash) = &self.dirty_state_sha256 {
            ensure!(
                is_sha256(dirty_hash) && dirty_hash == &dirty_hash.to_ascii_lowercase(),
                "dirty-state binding must be a lowercase SHA-256 digest"
            );
        }
        ensure!(
            !self.scope.is_empty() && self.scope.len() <= MAX_AUDIT_SCOPE_ITEMS,
            "audit scope is empty or exceeds its item limit"
        );
        ensure!(
            is_strictly_sorted_unique(&self.scope),
            "audit scope must be canonical, sorted, and unique"
        );
        for scope_path in &self.scope {
            validate_relative_path(scope_path, "audit scope path")?;
        }
        ensure!(
            !self.languages.is_empty() && self.languages.len() <= 32,
            "in-scope language set is empty or exceeds its item limit"
        );
        ensure!(
            is_strictly_sorted_unique(&self.languages),
            "language set must be canonical, lowercase, sorted, and unique"
        );
        for language in &self.languages {
            validate_identifier("language", language, MAX_SHORT_TEXT_CHARS)?;
            ensure!(
                language == &language.to_ascii_lowercase(),
                "language identifiers must be lowercase"
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditCoverageDisposition {
    Covered,
    Skipped,
    Deferred,
    OutOfScope,
}

impl AuditCoverageDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Covered => "covered",
            Self::Skipped => "skipped",
            Self::Deferred => "deferred",
            Self::OutOfScope => "out_of_scope",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCoverageUnitV1 {
    pub unit_id: String,
    pub scope_path: String,
    pub subsystem: String,
    pub boundary: String,
    pub lifecycle: String,
    pub language: String,
    pub attack_class: String,
    pub disposition: AuditCoverageDisposition,
    pub reason: Option<String>,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCoverageLedgerV1 {
    pub schema_version: u32,
    pub units: Vec<AuditCoverageUnitV1>,
}

pub fn validate_coverage_ledger(
    ledger: &AuditCoverageLedgerV1,
    binding: &AuditRunBindingV1,
) -> Result<()> {
    binding.validate()?;
    ensure!(
        ledger.schema_version == SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        "unsupported audit coverage schema version"
    );
    ensure!(
        !ledger.units.is_empty() && ledger.units.len() <= MAX_COVERAGE_UNITS,
        "coverage ledger is empty or exceeds its unit limit"
    );
    ensure!(
        ledger
            .units
            .windows(2)
            .all(|pair| pair[0].unit_id < pair[1].unit_id),
        "coverage units must be sorted by unique unit id"
    );
    let mut ids = BTreeSet::new();
    for unit in &ledger.units {
        validate_identifier("coverage unit id", &unit.unit_id, MAX_SHORT_TEXT_CHARS)?;
        ensure!(
            ids.insert(unit.unit_id.as_str()),
            "duplicate coverage unit id"
        );
        validate_relative_path(&unit.scope_path, "coverage scope path")?;
        validate_identifier("subsystem", &unit.subsystem, MAX_SHORT_TEXT_CHARS)?;
        validate_identifier("boundary", &unit.boundary, MAX_TEXT_CHARS)?;
        validate_identifier("lifecycle", &unit.lifecycle, MAX_SHORT_TEXT_CHARS)?;
        validate_identifier("language", &unit.language, MAX_SHORT_TEXT_CHARS)?;
        ensure!(
            SHARED_ATTACK_CLASSES
                .iter()
                .any(|attack_class| attack_class == &unit.attack_class),
            "coverage unit uses an unknown shared attack class"
        );
        if unit.disposition != AuditCoverageDisposition::OutOfScope {
            ensure!(
                binding.languages.binary_search(&unit.language).is_ok(),
                "in-scope coverage unit language is absent from run binding"
            );
            ensure!(
                binding
                    .scope
                    .iter()
                    .any(|scope| path_is_within(scope, &unit.scope_path)),
                "coverage unit is outside the authorized scope"
            );
        }
        match unit.disposition {
            AuditCoverageDisposition::Covered => {
                ensure!(
                    unit.reason
                        .as_deref()
                        .is_none_or(|reason| reason.trim().is_empty()),
                    "covered unit cannot carry an exclusion reason"
                );
                validate_string_evidence_refs(&unit.evidence_refs)?;
            }
            AuditCoverageDisposition::Skipped
            | AuditCoverageDisposition::Deferred
            | AuditCoverageDisposition::OutOfScope => {
                let reason = unit
                    .reason
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("uncovered unit requires a reason"))?;
                validate_identifier("coverage disposition reason", reason, MAX_TEXT_CHARS)?;
                ensure!(
                    unit.evidence_refs.len() <= MAX_AUDIT_EVIDENCE_REFS,
                    "coverage evidence exceeds its item limit"
                );
                for reference in &unit.evidence_refs {
                    validate_identifier("coverage evidence reference", reference, MAX_TEXT_CHARS)?;
                }
            }
        }
    }
    let covered_dimensions = ledger
        .units
        .iter()
        .map(|unit| {
            (
                unit.language.as_str(),
                unit.lifecycle.as_str(),
                unit.attack_class.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    for language in &binding.languages {
        for lifecycle in SECURITY_AUDIT_LIFECYCLE_SURFACES {
            for attack_class in SHARED_ATTACK_CLASSES {
                ensure!(
                    covered_dimensions.contains(&(language.as_str(), lifecycle, attack_class)),
                    "coverage ledger omits a required language/lifecycle/attack-class unit"
                );
            }
        }
    }
    for scope_path in &binding.scope {
        ensure!(
            ledger
                .units
                .iter()
                .any(|unit| path_is_within(scope_path, &unit.scope_path)),
            "coverage ledger omits a path from the authorized scope"
        );
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEvidenceRefV1 {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub content_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCandidateV1 {
    /// Explicit causal key prevents distinct root causes on the same line from
    /// being collapsed into one finding.
    pub root_cause_key: String,
    pub attack_class: String,
    pub language: String,
    pub subsystem: String,
    pub protected_boundary: String,
    pub lifecycle: String,
    pub source_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub claim: String,
    pub author_ids: Vec<String>,
    pub evidence_refs: Vec<AuditEvidenceRefV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditFindingVerdictV1 {
    Confirmed {
        severity: AuditSeverity,
        affected_principal_or_attacker: String,
        protected_resource: String,
        impact: String,
        source_path: String,
        start_line: u32,
        end_line: u32,
        evidence_refs: Vec<AuditEvidenceRefV1>,
        recommended_correction: String,
        safe_verification: String,
    },
    NeedsValidation {
        unresolved_fact: String,
        blocker: String,
    },
    Rejected {
        claim: String,
        disproof: String,
        evidence_refs: Vec<AuditEvidenceRefV1>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCandidateReviewV1 {
    pub reviewer_id: String,
    /// A fresh context ID must be distinct for each independent review.
    pub context_id: String,
    pub source_revision: String,
    /// Binds review evidence to the complete source and Baron operation state,
    /// including the dirty checkout and exact Task State digest.
    pub binding_sha256: String,
    pub candidate_revision: u32,
    pub assessment: AuditFindingVerdictV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRecordReviewV1 {
    pub reviewer_id: String,
    pub context_id: String,
    pub source_revision: String,
    /// Binds independent record verification to the complete audit run.
    pub binding_sha256: String,
    pub candidate_revision: u32,
    pub candidate_fingerprint: String,
    pub accepted: bool,
    pub correction_requires_revalidation: bool,
    pub evidence_refs: Vec<AuditEvidenceRefV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditFindingRecordV1 {
    pub candidate: AuditCandidateV1,
    pub candidate_revision: u32,
    pub candidate_reviews: Vec<AuditCandidateReviewV1>,
    pub independent_record_review: Option<AuditRecordReviewV1>,
}

impl AuditFindingRecordV1 {
    pub fn fingerprint(&self, binding: &AuditRunBindingV1) -> Result<String> {
        candidate_fingerprint(binding, &self.candidate)
    }

    pub fn final_verdict(
        &self,
        mode: AuditMode,
        binding: &AuditRunBindingV1,
    ) -> Result<AuditFindingVerdictV1> {
        validate_candidate(&self.candidate, binding)?;
        ensure!(
            self.candidate_reviews.len() >= mode.minimum_candidate_verifiers(),
            "candidate lacks the required independent fresh verifiers"
        );
        let mut reviewer_ids = BTreeSet::new();
        let mut context_ids = BTreeSet::new();
        for review in &self.candidate_reviews {
            validate_candidate_review(&self.candidate, self.candidate_revision, review, binding)?;
            ensure!(
                reviewer_ids.insert(review.reviewer_id.as_str()),
                "candidate reviewer identity is duplicated"
            );
            ensure!(
                context_ids.insert(review.context_id.as_str()),
                "candidate review context is not fresh"
            );
        }
        resolve_candidate_reviews(&self.candidate_reviews)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditExecutionOutcome {
    Completed,
    Failed,
    TimedOut,
    Blocked,
}

/// An unverified serialized claim that an operation ran. This is not itself a
/// Baron proof, gate receipt, or passing trace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditExecutionReceiptV1 {
    pub operation: String,
    pub operation_id: String,
    pub source_revision: String,
    pub binding_sha256: String,
    pub outcome: AuditExecutionOutcome,
    pub timestamp: String,
    pub artifact_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditPhaseReceiptV1 {
    pub phase: AuditPhase,
    pub binding_sha256: String,
    pub operation_id: String,
    pub timestamp: String,
    /// Opaque reference only; this contract does not resolve or verify it.
    pub artifact_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRunV1 {
    pub schema_version: u32,
    pub run_id: String,
    pub binding: AuditRunBindingV1,
    pub mode: AuditMode,
    pub status: AuditRunStatusV1,
    pub current_phase: AuditPhase,
    pub phase_receipts: Vec<AuditPhaseReceiptV1>,
    /// Receipts invalidated by a material correction remain visible for audit
    /// history, but never count toward current phase completion.
    pub superseded_phase_receipts: Vec<AuditPhaseReceiptV1>,
    pub coverage_ledger: Option<AuditCoverageLedgerV1>,
    pub findings: Vec<AuditFindingRecordV1>,
    pub execution_receipts: Vec<AuditExecutionReceiptV1>,
    pub blockers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<AuditFailureV1>,
}

impl AuditRunV1 {
    pub fn new(
        run_id: impl Into<String>,
        mut binding: AuditRunBindingV1,
        mode: AuditMode,
    ) -> Result<Self> {
        binding.canonicalize();
        binding.validate()?;
        let run_id = run_id.into();
        validate_audit_run_id(&run_id)?;
        Ok(Self {
            schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
            run_id,
            binding,
            mode,
            status: AuditRunStatusV1::InProgress,
            current_phase: AuditPhase::Reconnaissance,
            phase_receipts: Vec::new(),
            superseded_phase_receipts: Vec::new(),
            coverage_ledger: None,
            findings: Vec::new(),
            execution_receipts: Vec::new(),
            blockers: Vec::new(),
            failure: None,
        })
    }

    pub fn binding_fingerprint(&self) -> Result<String> {
        binding_fingerprint(&self.binding)
    }

    /// A non-reflective identifier for output and continuity summaries.
    pub fn run_id_sha256(&self) -> String {
        format!("{:x}", Sha256::digest(self.run_id.as_bytes()))
    }

    pub fn set_coverage_ledger(&mut self, ledger: AuditCoverageLedgerV1) -> Result<()> {
        ensure_active_phase(self, AuditPhase::CoverageLedHunting)?;
        validate_coverage_ledger(&ledger, &self.binding)?;
        self.coverage_ledger = Some(ledger);
        Ok(())
    }

    pub fn add_candidate(&mut self, candidate: AuditCandidateV1) -> Result<String> {
        ensure_active_phase(self, AuditPhase::CoverageLedHunting)?;
        let ledger = self
            .coverage_ledger
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("create the coverage ledger before candidate review"))?;
        validate_candidate(&candidate, &self.binding)?;
        validate_candidate_coverage(&candidate, ledger)?;
        let fingerprint = candidate_fingerprint(&self.binding, &candidate)?;
        ensure!(
            self.findings.len() < MAX_AUDIT_FINDINGS,
            "audit finding count exceeds its item limit"
        );
        if self.findings.iter().any(|known| {
            known
                .fingerprint(&self.binding)
                .is_ok_and(|known| known == fingerprint)
        }) {
            bail!("duplicate candidate fingerprint; consolidate before submission");
        }
        self.findings.push(AuditFindingRecordV1 {
            candidate,
            candidate_revision: 0,
            candidate_reviews: Vec::new(),
            independent_record_review: None,
        });
        self.findings.sort_by_key(|finding| {
            finding
                .fingerprint(&self.binding)
                .unwrap_or_else(|_| String::new())
        });
        Ok(fingerprint)
    }

    pub fn add_candidate_review(
        &mut self,
        candidate_fingerprint: &str,
        review: AuditCandidateReviewV1,
    ) -> Result<()> {
        ensure_active_phase(self, AuditPhase::CandidateValidation)?;
        let binding = self.binding.clone();
        let finding = self.find_finding_mut(candidate_fingerprint)?;
        validate_candidate_review(
            &finding.candidate,
            finding.candidate_revision,
            &review,
            &binding,
        )?;
        ensure!(
            !finding
                .candidate_reviews
                .iter()
                .any(|known| known.reviewer_id == review.reviewer_id
                    || known.context_id == review.context_id),
            "candidate review is not independent or fresh"
        );
        finding.candidate_reviews.push(review);
        finding
            .candidate_reviews
            .sort_by(|left, right| left.reviewer_id.cmp(&right.reviewer_id));
        Ok(())
    }

    pub fn record_independent_record_review(
        &mut self,
        candidate_fingerprint: &str,
        review: AuditRecordReviewV1,
    ) -> Result<()> {
        ensure_active_phase(self, AuditPhase::IndependentRecordVerification)?;
        let binding = self.binding.clone();
        let mode = self.mode;
        let finding = self.find_finding_mut(candidate_fingerprint)?;
        let final_verdict = finding.final_verdict(mode, &binding)?;
        ensure!(
            matches!(final_verdict, AuditFindingVerdictV1::Confirmed { .. }),
            "independent record review is required only for confirmed records"
        );
        validate_record_review(finding, &review, &binding)?;
        finding.independent_record_review = Some(review);
        Ok(())
    }

    /// Reopen a material correction through candidate validation. Current
    /// later-phase receipts are retained as superseded evidence and do not
    /// authorize completion of the corrected record.
    pub fn reopen_candidate_for_revalidation(
        &mut self,
        fingerprint: &str,
        corrected_candidate: AuditCandidateV1,
    ) -> Result<()> {
        ensure_active_phase(self, AuditPhase::IndependentRecordVerification)?;
        let binding = &self.binding;
        validate_candidate(&corrected_candidate, binding)?;
        ensure!(
            candidate_fingerprint(binding, &corrected_candidate)? == fingerprint,
            "material correction changes the causal candidate identity; start a new candidate"
        );
        let index = self.finding_index(fingerprint)?;
        let finding = &self.findings[index];
        let review = finding
            .independent_record_review
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("candidate has no independent record correction"))?;
        ensure!(
            review.correction_requires_revalidation,
            "record review did not require candidate revalidation"
        );
        self.superseded_phase_receipts
            .extend(self.phase_receipts.drain(2..));
        for finding in &mut self.findings {
            finding.independent_record_review = None;
        }
        let finding = &mut self.findings[index];
        finding.candidate = corrected_candidate;
        finding.candidate_revision = finding.candidate_revision.saturating_add(1);
        finding.candidate_reviews.clear();
        self.current_phase = AuditPhase::CandidateValidation;
        self.status = AuditRunStatusV1::InProgress;
        self.blockers.clear();
        self.failure = None;
        Ok(())
    }

    pub fn record_execution_receipt(&mut self, receipt: AuditExecutionReceiptV1) -> Result<()> {
        ensure!(
            self.status == AuditRunStatusV1::InProgress,
            "blocked or completed audit run cannot accept execution receipts"
        );
        self.binding.validate()?;
        validate_execution_receipt(&receipt, self)?;
        ensure!(
            self.execution_receipts.len() < MAX_COVERAGE_UNITS,
            "audit execution receipt count exceeds its item limit"
        );
        let outcome = receipt.outcome.clone();
        self.execution_receipts.push(receipt);
        match outcome {
            AuditExecutionOutcome::Completed => {}
            AuditExecutionOutcome::Failed => self.mark_failed(AuditFailureV1 {
                class: AuditFailureClassV1::TargetCommandFailed,
                code: "target_command_failed".to_string(),
            })?,
            AuditExecutionOutcome::TimedOut => self.mark_failed(AuditFailureV1 {
                class: AuditFailureClassV1::TargetCommandTimedOut,
                code: "target_command_timed_out".to_string(),
            })?,
            AuditExecutionOutcome::Blocked => self.block("target_command_blocked")?,
        }
        Ok(())
    }

    pub fn mark_incomplete(&mut self, failure: AuditFailureV1) -> Result<()> {
        ensure!(
            !matches!(
                self.status,
                AuditRunStatusV1::Complete | AuditRunStatusV1::Failed
            ),
            "completed or failed audit run cannot be marked incomplete retroactively"
        );
        validate_audit_failure(&failure)?;
        self.status = AuditRunStatusV1::Incomplete;
        self.blockers.clear();
        self.failure = Some(failure);
        Ok(())
    }

    pub fn mark_failed(&mut self, failure: AuditFailureV1) -> Result<()> {
        ensure!(
            self.status != AuditRunStatusV1::Complete,
            "completed audit run cannot be failed retroactively"
        );
        validate_audit_failure(&failure)?;
        self.status = AuditRunStatusV1::Failed;
        self.blockers.clear();
        self.failure = Some(failure);
        Ok(())
    }

    pub fn block(&mut self, blocker: impl Into<String>) -> Result<()> {
        ensure!(
            !matches!(
                self.status,
                AuditRunStatusV1::Complete | AuditRunStatusV1::Failed
            ),
            "completed or failed audit run cannot be blocked retroactively"
        );
        let blocker = blocker.into();
        validate_identifier("audit blocker", &blocker, MAX_TEXT_CHARS)?;
        if !self.blockers.contains(&blocker) {
            validate_blocker_id(&blocker)?;
            self.blockers.push(blocker);
            self.blockers.sort();
        }
        self.status = AuditRunStatusV1::Blocked;
        self.failure = None;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        bail!("audit runs can resume only through source-bound checkpoint validation")
    }

    pub fn complete_phase(&mut self, receipt: AuditPhaseReceiptV1) -> Result<()> {
        ensure!(
            self.status == AuditRunStatusV1::InProgress,
            "blocked or completed audit run cannot advance"
        );
        validate_audit_run(self)?;
        let phase_index = self.phase_receipts.len();
        ensure!(
            phase_index < AUDIT_PHASES.len(),
            "all audit phases are already complete"
        );
        let expected = AUDIT_PHASES[phase_index];
        ensure!(
            self.current_phase == expected && receipt.phase == expected,
            "audit phase transition would skip, repeat, or reorder a required phase"
        );
        validate_phase_receipt(&receipt, self)?;
        self.validate_phase_prerequisites(expected)?;
        self.phase_receipts.push(receipt);
        if self.phase_receipts.len() == AUDIT_PHASES.len() {
            let execution_failure =
                self.execution_receipts
                    .iter()
                    .find_map(|receipt| match &receipt.outcome {
                        AuditExecutionOutcome::Failed => Some(AuditFailureV1 {
                            class: AuditFailureClassV1::TargetCommandFailed,
                            code: "target_command_failed".to_string(),
                        }),
                        AuditExecutionOutcome::TimedOut => Some(AuditFailureV1 {
                            class: AuditFailureClassV1::TargetCommandTimedOut,
                            code: "target_command_timed_out".to_string(),
                        }),
                        AuditExecutionOutcome::Completed | AuditExecutionOutcome::Blocked => None,
                    });
            if let Some(failure) = execution_failure {
                self.mark_failed(failure)?;
            } else if self
                .execution_receipts
                .iter()
                .any(|receipt| receipt.outcome == AuditExecutionOutcome::Blocked)
            {
                self.block("target_command_blocked")?;
            } else if self.coverage_ledger.as_ref().is_some_and(|ledger| {
                ledger.units.iter().any(|unit| {
                    matches!(
                        unit.disposition,
                        AuditCoverageDisposition::Skipped | AuditCoverageDisposition::Deferred
                    )
                })
            }) {
                self.mark_incomplete(AuditFailureV1 {
                    class: AuditFailureClassV1::CoverageIncomplete,
                    code: "required_coverage_unresolved".to_string(),
                })?;
            } else {
                self.status = AuditRunStatusV1::Complete;
            }
        } else {
            self.current_phase = AuditPhase::at(self.phase_receipts.len());
        }
        validate_audit_run(self)
    }

    pub fn report(&self) -> Result<AuditReportV1> {
        validate_audit_run(self)?;
        let coverage = self
            .coverage_ledger
            .as_ref()
            .map(|ledger| summarize_coverage(&ledger.units))
            .unwrap_or_default();
        let mut findings = Vec::new();
        let mut unreviewed_paths = BTreeSet::new();
        let mut residual_checks = self.blockers.iter().cloned().collect::<BTreeSet<_>>();
        for finding in &self.findings {
            let verdict = match finding.final_verdict(self.mode, &self.binding) {
                Ok(verdict) => Some(verdict),
                Err(_) => {
                    residual_checks.insert("candidate_validation_incomplete".to_string());
                    None
                }
            };
            findings.push(AuditReportFindingV1 {
                fingerprint: finding.fingerprint(&self.binding)?,
                root_cause_key: finding.candidate.root_cause_key.clone(),
                candidate: finding.candidate.clone(),
                candidate_revision: finding.candidate_revision,
                candidate_reviews: finding.candidate_reviews.clone(),
                independent_record_review: finding.independent_record_review.clone(),
                verdict,
            });
        }
        if let Some(ledger) = &self.coverage_ledger {
            for unit in &ledger.units {
                if unit.disposition != AuditCoverageDisposition::Covered {
                    unreviewed_paths.insert(unit.scope_path.clone());
                    if let Some(reason) = &unit.reason {
                        residual_checks.insert(reason.clone());
                    }
                }
            }
        }
        if self.status != AuditRunStatusV1::Complete {
            residual_checks.insert("audit_run_incomplete".to_string());
        }
        if !self
            .execution_receipts
            .iter()
            .any(|receipt| receipt.outcome == AuditExecutionOutcome::Completed)
        {
            residual_checks.insert("no_completed_execution_receipt".to_string());
        }
        if let Some(failure) = &self.failure {
            residual_checks.insert(failure.code.clone());
        }
        if findings.iter().any(|finding| {
            matches!(
                finding.verdict.as_ref(),
                Some(
                    AuditFindingVerdictV1::Confirmed { .. }
                        | AuditFindingVerdictV1::NeedsValidation { .. }
                )
            )
        }) {
            residual_checks.insert("open_findings_block_application_task".to_string());
        }
        Ok(AuditReportV1 {
            schema_version: SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
            run_id: self.run_id.clone(),
            validation_scope: AuditValidationScopeV1::SchemaAndWorkflowContractOnly,
            execution_status: if self.execution_receipts.is_empty() {
                AuditReportExecutionStatusV1::NotRun
            } else {
                AuditReportExecutionStatusV1::EvidenceRecordedUnverified
            },
            proof_gate_trace_claimed: false,
            clean: false,
            application_task_blocked: true,
            failure: self.failure.clone(),
            status: self.status,
            mode: self.mode,
            binding: self.binding.clone(),
            completed_phases: self.phase_receipts.len(),
            coverage,
            coverage_ledger: self.coverage_ledger.clone(),
            findings,
            unreviewed_paths: unreviewed_paths.into_iter().collect(),
            residual_checks: residual_checks.into_iter().collect(),
            checks_run: self.execution_receipts.clone(),
            prior_run_limitations: vec![
                "Coverage from another source revision is not evidence for this run.".to_string(),
                "A completed audit describes its declared scope; it does not prove the repository is vulnerability-free.".to_string(),
            ],
        })
    }

    fn find_finding_mut(&mut self, fingerprint: &str) -> Result<&mut AuditFindingRecordV1> {
        let index = self.finding_index(fingerprint)?;
        Ok(&mut self.findings[index])
    }

    fn finding_index(&self, fingerprint: &str) -> Result<usize> {
        self.findings
            .iter()
            .position(|finding| {
                finding
                    .fingerprint(&self.binding)
                    .is_ok_and(|known| known == fingerprint)
            })
            .ok_or_else(|| anyhow::anyhow!("candidate fingerprint is absent from this run"))
    }

    fn validate_phase_prerequisites(&self, phase: AuditPhase) -> Result<()> {
        match phase {
            AuditPhase::Reconnaissance => self.binding.validate(),
            AuditPhase::CoverageLedHunting => {
                let ledger = self.coverage_ledger.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("coverage ledger is required before hunting completes")
                })?;
                validate_coverage_ledger(ledger, &self.binding)?;
                validate_findings_inventory(self)?;
                Ok(())
            }
            AuditPhase::CandidateValidation => {
                validate_findings_for_phase(self, AuditPhase::CandidateValidation)
            }
            AuditPhase::StructuredOutput => {
                let ledger = self
                    .coverage_ledger
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("validated coverage ledger is required"))?;
                validate_coverage_ledger(ledger, &self.binding)?;
                validate_findings_for_phase(self, AuditPhase::StructuredOutput)
            }
            AuditPhase::IndependentRecordVerification => {
                validate_findings_for_phase(self, AuditPhase::IndependentRecordVerification)
            }
            AuditPhase::Reporting => {
                validate_findings_for_phase(self, AuditPhase::Reporting)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCoverageSummaryV1 {
    pub covered: usize,
    pub skipped: usize,
    pub deferred: usize,
    pub out_of_scope: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditReportFindingV1 {
    pub fingerprint: String,
    pub root_cause_key: String,
    pub candidate: AuditCandidateV1,
    pub candidate_revision: u32,
    pub candidate_reviews: Vec<AuditCandidateReviewV1>,
    pub independent_record_review: Option<AuditRecordReviewV1>,
    /// `None` means this candidate remains in the report as an unresolved
    /// lead, with its exact source context preserved and no verdict implied.
    pub verdict: Option<AuditFindingVerdictV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditReportV1 {
    pub schema_version: u32,
    pub run_id: String,
    pub validation_scope: AuditValidationScopeV1,
    pub execution_status: AuditReportExecutionStatusV1,
    pub proof_gate_trace_claimed: bool,
    /// This schema-only workflow does not verify target execution, proof,
    /// quality gates, or trace, so it cannot make a clean-audit claim.
    #[serde(default)]
    pub clean: bool,
    /// A security-audit report never closes the application task by itself.
    #[serde(default = "application_task_blocked_by_default")]
    pub application_task_blocked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<AuditFailureV1>,
    pub status: AuditRunStatusV1,
    pub mode: AuditMode,
    pub binding: AuditRunBindingV1,
    pub completed_phases: usize,
    pub coverage: AuditCoverageSummaryV1,
    /// Full validated ledger is retained so summaries do not erase which
    /// language/lifecycle/attack units were skipped or deferred.
    pub coverage_ledger: Option<AuditCoverageLedgerV1>,
    pub findings: Vec<AuditReportFindingV1>,
    pub unreviewed_paths: Vec<String>,
    pub residual_checks: Vec<String>,
    pub checks_run: Vec<AuditExecutionReceiptV1>,
    pub prior_run_limitations: Vec<String>,
}

fn application_task_blocked_by_default() -> bool {
    true
}

fn validate_audit_failure(failure: &AuditFailureV1) -> Result<()> {
    validate_blocker_id(&failure.code)
}

pub fn validate_audit_run(run: &AuditRunV1) -> Result<()> {
    ensure!(
        run.schema_version == SECURITY_AUDIT_WORKFLOW_SCHEMA_VERSION,
        "unsupported audit run schema version"
    );
    validate_audit_run_id(&run.run_id)?;
    run.binding.validate()?;
    ensure!(
        run.phase_receipts.len() <= AUDIT_PHASES.len(),
        "audit phase receipt count exceeds required phases"
    );
    ensure!(
        run.superseded_phase_receipts.len() <= AUDIT_PHASES.len() * MAX_AUDIT_FINDINGS,
        "superseded phase receipt count exceeds its limit"
    );
    ensure!(
        run.findings.len() <= MAX_AUDIT_FINDINGS,
        "audit finding count exceeds its item limit"
    );
    ensure!(
        run.execution_receipts.len() <= MAX_COVERAGE_UNITS,
        "audit execution receipt count exceeds its item limit"
    );
    ensure!(
        run.current_phase == AuditPhase::at(run.phase_receipts.len()),
        "current phase does not match completed phase receipts"
    );
    for (index, receipt) in run.phase_receipts.iter().enumerate() {
        ensure!(
            receipt.phase == AUDIT_PHASES[index],
            "phase receipt history skips or reorders a required phase"
        );
        validate_phase_receipt(receipt, run)?;
    }
    for receipt in &run.superseded_phase_receipts {
        validate_phase_receipt(receipt, run)?;
    }
    match run.status {
        AuditRunStatusV1::InProgress => {
            ensure!(
                run.phase_receipts.len() < AUDIT_PHASES.len(),
                "all required phases are already complete"
            );
            ensure!(
                run.blockers.is_empty(),
                "in-progress audit cannot retain active blockers"
            );
            ensure!(
                run.failure.is_none(),
                "in-progress audit cannot retain an active failure"
            );
            ensure!(
                run.execution_receipts
                    .iter()
                    .all(|receipt| { receipt.outcome == AuditExecutionOutcome::Completed }),
                "in-progress audit cannot retain a failed, timed out, or blocked execution"
            );
        }
        AuditRunStatusV1::Blocked => {
            ensure!(
                !run.blockers.is_empty(),
                "blocked audit run requires a blocker"
            );
            ensure!(
                run.failure.is_none(),
                "blocked audit cannot retain a failed-run state"
            );
            ensure!(
                run.execution_receipts
                    .iter()
                    .all(|receipt| receipt.outcome != AuditExecutionOutcome::Failed
                        && receipt.outcome != AuditExecutionOutcome::TimedOut),
                "blocked audit cannot hide a failed or timed-out execution"
            );
        }
        AuditRunStatusV1::Incomplete | AuditRunStatusV1::Failed => {
            ensure!(
                run.blockers.is_empty(),
                "incomplete or failed audit cannot retain active blockers"
            );
            let failure = run.failure.as_ref().ok_or_else(|| {
                anyhow::anyhow!("incomplete or failed audit requires a failure classification")
            })?;
            validate_audit_failure(failure)?;
            if let Some(receipt) = run.execution_receipts.iter().find(|receipt| {
                receipt.outcome == AuditExecutionOutcome::Failed
                    || receipt.outcome == AuditExecutionOutcome::TimedOut
            }) {
                let expected = match &receipt.outcome {
                    AuditExecutionOutcome::Failed => AuditFailureClassV1::TargetCommandFailed,
                    AuditExecutionOutcome::TimedOut => AuditFailureClassV1::TargetCommandTimedOut,
                    AuditExecutionOutcome::Completed | AuditExecutionOutcome::Blocked => {
                        unreachable!("receipt was filtered to failed or timed out")
                    }
                };
                ensure!(
                    run.status == AuditRunStatusV1::Failed && failure.class == expected,
                    "failed execution receipt does not match the run failure classification"
                );
            }
        }
        AuditRunStatusV1::Complete => {
            ensure!(
                run.phase_receipts.len() == AUDIT_PHASES.len(),
                "complete audit requires all required phases"
            );
            ensure!(
                run.blockers.is_empty(),
                "complete audit cannot retain active blockers"
            );
            ensure!(
                run.failure.is_none(),
                "complete audit cannot retain an active failure"
            );
        }
    }
    for blocker in &run.blockers {
        validate_blocker_id(blocker)?;
    }
    if let Some(ledger) = &run.coverage_ledger {
        validate_coverage_ledger(ledger, &run.binding)?;
    }
    validate_findings_inventory(run)?;
    let binding_hash = run.binding_fingerprint()?;
    for receipt in &run.execution_receipts {
        validate_execution_receipt(receipt, run)?;
        ensure!(
            receipt.binding_sha256 == binding_hash,
            "execution receipt does not match run binding"
        );
    }
    if run.status == AuditRunStatusV1::Complete {
        ensure!(
            run.execution_receipts
                .iter()
                .all(|receipt| receipt.outcome == AuditExecutionOutcome::Completed),
            "complete audit cannot contain a failed, timed out, or blocked execution"
        );
        ensure!(
            run.coverage_ledger.as_ref().is_none_or(|ledger| {
                ledger.units.iter().all(|unit| {
                    matches!(
                        unit.disposition,
                        AuditCoverageDisposition::Covered | AuditCoverageDisposition::OutOfScope
                    )
                })
            }),
            "complete audit cannot contain skipped or deferred required coverage"
        );
    }
    if run
        .execution_receipts
        .iter()
        .any(|receipt| receipt.outcome == AuditExecutionOutcome::Blocked)
    {
        ensure!(
            run.status == AuditRunStatusV1::Blocked
                && run
                    .blockers
                    .iter()
                    .any(|blocker| blocker == "target_command_blocked"),
            "blocked execution receipt must remain a blocked audit run"
        );
    }
    let completed_count = run.phase_receipts.len();
    if completed_count >= 2 {
        ensure!(
            run.coverage_ledger.is_some(),
            "completed hunting phase has no ledger"
        );
    }
    if completed_count >= 3 {
        validate_findings_for_phase(run, AuditPhase::CandidateValidation)?;
    }
    if completed_count >= 4 {
        validate_findings_for_phase(run, AuditPhase::StructuredOutput)?;
    }
    if completed_count >= 5 {
        validate_findings_for_phase(run, AuditPhase::IndependentRecordVerification)?;
    }
    if completed_count >= 6 {
        validate_findings_for_phase(run, AuditPhase::Reporting)?;
    }
    Ok(())
}

pub fn validate_audit_run_json(input: &[u8]) -> Result<AuditRunV1> {
    ensure!(
        input.len() <= MAX_AUDIT_RUN_JSON_BYTES,
        "audit run JSON exceeds its byte limit"
    );
    let run: AuditRunV1 = serde_json::from_slice(input).map_err(|_| {
        anyhow::anyhow!("audit run JSON is malformed or uses an unsupported field shape")
    })?;
    validate_audit_run(&run)?;
    Ok(run)
}

pub fn candidate_fingerprint(
    binding: &AuditRunBindingV1,
    candidate: &AuditCandidateV1,
) -> Result<String> {
    binding.validate()?;
    validate_candidate(candidate, binding)?;
    let mut digest = Sha256::new();
    digest.update(b"baron-security-audit-candidate-v1\0");
    let binding_sha256 = binding_fingerprint(binding)?;
    for value in [
        binding.project_id.as_str(),
        binding_sha256.as_str(),
        candidate.attack_class.as_str(),
        candidate.language.as_str(),
        candidate.subsystem.as_str(),
        candidate.protected_boundary.as_str(),
        candidate.lifecycle.as_str(),
        candidate.source_path.as_str(),
        candidate.root_cause_key.as_str(),
    ] {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    Ok(format!("finding-{:x}", digest.finalize()))
}

/// Deterministically consolidate only candidates with the same project,
/// revision, attack class, boundary, path, and explicit root-cause key.
pub fn deduplicate_candidates(
    binding: &AuditRunBindingV1,
    mut candidates: Vec<AuditCandidateV1>,
) -> Result<Vec<AuditCandidateV1>> {
    ensure!(
        candidates.len() <= MAX_AUDIT_FINDINGS,
        "candidate count exceeds its item limit"
    );
    for candidate in &candidates {
        validate_candidate(candidate, binding)?;
    }
    for candidate in &mut candidates {
        candidate.evidence_refs.sort_by(evidence_order);
        candidate.evidence_refs.dedup();
    }
    candidates.sort_by(|left, right| {
        candidate_fingerprint(binding, left)
            .unwrap_or_default()
            .cmp(&candidate_fingerprint(binding, right).unwrap_or_default())
            .then_with(|| left.claim.cmp(&right.claim))
            .then_with(|| left.start_line.cmp(&right.start_line))
            .then_with(|| left.end_line.cmp(&right.end_line))
    });
    let mut merged: Vec<AuditCandidateV1> = Vec::new();
    let mut fingerprints = BTreeMap::<String, usize>::new();
    for candidate in candidates {
        let fingerprint = candidate_fingerprint(binding, &candidate)?;
        if let Some(index) = fingerprints.get(&fingerprint).copied() {
            let known = &mut merged[index];
            known.author_ids.extend(candidate.author_ids);
            known.author_ids.sort();
            known.author_ids.dedup();
            known.evidence_refs.extend(candidate.evidence_refs);
            known.evidence_refs.sort_by(evidence_order);
            known.evidence_refs.dedup();
            ensure!(
                known.author_ids.len() <= 32
                    && known.evidence_refs.len() <= MAX_AUDIT_EVIDENCE_REFS,
                "merged candidate exceeds its author or evidence limit"
            );
            known.start_line = known.start_line.min(candidate.start_line);
            known.end_line = known.end_line.max(candidate.end_line);
        } else {
            fingerprints.insert(fingerprint, merged.len());
            merged.push(candidate);
        }
    }
    for candidate in &merged {
        validate_candidate(candidate, binding)?;
    }
    Ok(merged)
}

/// Persist a compact operational checkpoint through Baron continuity. The
/// checkpoint contains no source excerpt or finding claim and is bound to the
/// current task/operation and project identity.
pub fn record_security_audit_continuity_checkpoint(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    operation: &OperationContext,
    run: &AuditRunV1,
) -> Result<ContinuityPacket> {
    validate_audit_run(run)?;
    ensure!(
        run.binding.project_id == vault.project_id,
        "audit run project identity does not match the selected Vault"
    );
    let identity = operation.lifecycle_identity(&vault.project_id)?;
    ensure!(
        identity.task_id() == run.binding.task_id
            && identity.operation_id() == run.binding.operation_id,
        "audit run task or operation binding does not match continuity operation"
    );
    let phase_count = run.phase_receipts.len();
    let run_id_sha256 = run.run_id_sha256();
    let event_key = format!(
        "security-audit:{}:{}:{}:{}",
        run_id_sha256,
        phase_count,
        run.current_phase.as_str(),
        run.status.as_str()
    );
    let note = format!(
        "Security audit run ID sha256 {}; status {}; phase {}; completed phases {}/{}; coverage units {}; candidate records {}; active blockers [{}]; safe next action: reconcile the exact operation state and continue the next required audit phase.",
        run_id_sha256,
        run.status.as_str(),
        run.current_phase.as_str(),
        phase_count,
        AUDIT_PHASES.len(),
        run.coverage_ledger.as_ref().map_or(0, |ledger| ledger.units.len()),
        run.findings.len(),
        run.blockers.join(",")
    );
    record_continuity_checkpoint_for_event(repo_root, vault, &note, operation, &event_key)
}

fn validate_findings_inventory(run: &AuditRunV1) -> Result<()> {
    let mut fingerprints = BTreeSet::new();
    let mut review_contexts = BTreeSet::new();
    for finding in &run.findings {
        let fingerprint = finding.fingerprint(&run.binding)?;
        let ledger = run
            .coverage_ledger
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("candidate finding has no coverage ledger"))?;
        validate_candidate_coverage(&finding.candidate, ledger)?;
        ensure!(
            fingerprints.insert(fingerprint),
            "duplicate finding root-cause fingerprint"
        );
        ensure!(
            finding.candidate_revision <= 1_000_000,
            "candidate revision exceeds its limit"
        );
        ensure!(
            finding.candidate_reviews.len() <= 32,
            "candidate review count exceeds its limit"
        );
        let mut reviewer_ids = BTreeSet::new();
        for review in &finding.candidate_reviews {
            validate_candidate_review(
                &finding.candidate,
                finding.candidate_revision,
                review,
                &run.binding,
            )?;
            ensure!(
                reviewer_ids.insert(review.reviewer_id.as_str()),
                "candidate reviewer identity is duplicated"
            );
            ensure!(
                review_contexts.insert(review.context_id.as_str()),
                "candidate validation context is not fresh across this run"
            );
        }
        if let Some(review) = &finding.independent_record_review {
            validate_record_review(finding, review, &run.binding)?;
            ensure!(
                review_contexts.insert(review.context_id.as_str()),
                "record verification context is not fresh across this run"
            );
        }
    }
    Ok(())
}

fn validate_findings_for_phase(run: &AuditRunV1, phase: AuditPhase) -> Result<()> {
    for finding in &run.findings {
        let verdict = finding.final_verdict(run.mode, &run.binding)?;
        validate_finding_verdict(&verdict, &finding.candidate, &run.binding)?;
        if phase == AuditPhase::IndependentRecordVerification
            && matches!(verdict, AuditFindingVerdictV1::Confirmed { .. })
        {
            let review = finding.independent_record_review.as_ref().ok_or_else(|| {
                anyhow::anyhow!("confirmed finding lacks independent record verification")
            })?;
            ensure!(
                review.accepted && !review.correction_requires_revalidation,
                "independent record verification rejected or reopened the confirmed finding"
            );
        }
        if phase == AuditPhase::Reporting {
            ensure!(
                finding.candidate_reviews.len() >= run.mode.minimum_candidate_verifiers(),
                "report cannot include an unvalidated candidate"
            );
        }
    }
    Ok(())
}

fn validate_candidate(candidate: &AuditCandidateV1, binding: &AuditRunBindingV1) -> Result<()> {
    validate_identifier(
        "candidate root-cause key",
        &candidate.root_cause_key,
        MAX_SHORT_TEXT_CHARS,
    )?;
    validate_identifier(
        "candidate attack class",
        &candidate.attack_class,
        MAX_SHORT_TEXT_CHARS,
    )?;
    ensure!(
        SHARED_ATTACK_CLASSES
            .iter()
            .any(|attack_class| attack_class == &candidate.attack_class),
        "candidate uses an unknown shared attack class"
    );
    validate_identifier(
        "candidate language",
        &candidate.language,
        MAX_SHORT_TEXT_CHARS,
    )?;
    ensure!(
        binding.languages.binary_search(&candidate.language).is_ok(),
        "candidate language is absent from the run binding"
    );
    validate_identifier(
        "candidate subsystem",
        &candidate.subsystem,
        MAX_SHORT_TEXT_CHARS,
    )?;
    validate_identifier(
        "candidate boundary",
        &candidate.protected_boundary,
        MAX_TEXT_CHARS,
    )?;
    validate_identifier(
        "candidate lifecycle",
        &candidate.lifecycle,
        MAX_SHORT_TEXT_CHARS,
    )?;
    validate_relative_path(&candidate.source_path, "candidate source path")?;
    ensure!(
        binding
            .scope
            .iter()
            .any(|scope| path_is_within(scope, &candidate.source_path)),
        "candidate source path is outside the authorized audit scope"
    );
    ensure!(
        candidate.start_line > 0 && candidate.end_line >= candidate.start_line,
        "candidate source line range is invalid"
    );
    validate_identifier("candidate claim", &candidate.claim, MAX_TEXT_CHARS)?;
    ensure!(
        !candidate.author_ids.is_empty() && candidate.author_ids.len() <= 32,
        "candidate author set is empty or exceeds its limit"
    );
    ensure!(
        is_strictly_sorted_unique(&candidate.author_ids),
        "candidate author identities must be canonical, sorted, and unique"
    );
    for author in &candidate.author_ids {
        validate_identifier("candidate author", author, MAX_SHORT_TEXT_CHARS)?;
    }
    validate_evidence_refs(&candidate.evidence_refs)?;
    ensure!(
        candidate
            .evidence_refs
            .iter()
            .all(|evidence| evidence_is_in_scope(evidence, binding)),
        "candidate contains evidence outside the authorized audit scope"
    );
    ensure!(
        candidate.evidence_refs.iter().any(|evidence| {
            evidence.path == candidate.source_path
                && evidence.start_line <= candidate.end_line
                && evidence.end_line >= candidate.start_line
        }),
        "candidate evidence does not identify the cited current source path and lines"
    );
    Ok(())
}

fn validate_candidate_coverage(
    candidate: &AuditCandidateV1,
    ledger: &AuditCoverageLedgerV1,
) -> Result<()> {
    ensure!(
        ledger.units.iter().any(|unit| {
            unit.disposition == AuditCoverageDisposition::Covered
                && unit.language == candidate.language
                && unit.attack_class == candidate.attack_class
                && unit.subsystem == candidate.subsystem
                && unit.boundary == candidate.protected_boundary
                && unit.lifecycle == candidate.lifecycle
                && path_is_within(&unit.scope_path, &candidate.source_path)
        }),
        "candidate has no matching covered language/boundary/lifecycle/attack unit"
    );
    Ok(())
}

fn validate_candidate_review(
    candidate: &AuditCandidateV1,
    candidate_revision: u32,
    review: &AuditCandidateReviewV1,
    binding: &AuditRunBindingV1,
) -> Result<()> {
    validate_identifier(
        "candidate reviewer",
        &review.reviewer_id,
        MAX_SHORT_TEXT_CHARS,
    )?;
    validate_identifier(
        "candidate review context",
        &review.context_id,
        MAX_SHORT_TEXT_CHARS,
    )?;
    ensure!(
        !candidate.author_ids.contains(&review.reviewer_id),
        "candidate author cannot verify their own candidate"
    );
    ensure!(
        review.source_revision == binding.source_revision,
        "candidate review uses stale source revision"
    );
    ensure!(
        review.binding_sha256 == binding_fingerprint(binding)?,
        "candidate review uses a stale or different run binding"
    );
    ensure!(
        review.candidate_revision == candidate_revision,
        "candidate review uses a stale candidate revision"
    );
    validate_finding_verdict(&review.assessment, candidate, binding)
}

fn validate_finding_verdict(
    verdict: &AuditFindingVerdictV1,
    candidate: &AuditCandidateV1,
    binding: &AuditRunBindingV1,
) -> Result<()> {
    match verdict {
        AuditFindingVerdictV1::Confirmed {
            severity: _,
            affected_principal_or_attacker,
            protected_resource,
            impact,
            source_path,
            start_line,
            end_line,
            evidence_refs,
            recommended_correction,
            safe_verification,
        } => {
            validate_identifier(
                "affected principal or attacker",
                affected_principal_or_attacker,
                MAX_TEXT_CHARS,
            )?;
            validate_identifier("protected resource", protected_resource, MAX_TEXT_CHARS)?;
            validate_identifier("confirmed impact", impact, MAX_TEXT_CHARS)?;
            validate_relative_path(source_path, "confirmed source path")?;
            ensure!(
                source_path == &candidate.source_path
                    && binding
                        .scope
                        .iter()
                        .any(|scope| path_is_within(scope, source_path)),
                "confirmed source path is outside the candidate or authorized scope"
            );
            ensure!(
                *start_line >= candidate.start_line && *end_line <= candidate.end_line,
                "confirmed source lines extend outside the candidate range"
            );
            ensure!(
                *start_line > 0 && *end_line >= *start_line,
                "confirmed source line range is invalid"
            );
            validate_evidence_refs(evidence_refs)?;
            ensure!(
                evidence_refs.iter().any(|evidence| {
                    evidence.path == *source_path
                        && evidence.start_line <= candidate.end_line
                        && evidence.end_line >= candidate.start_line
                        && evidence.start_line <= *end_line
                        && evidence.end_line >= *start_line
                }),
                "confirmed finding lacks matching current-source evidence"
            );
            ensure!(
                evidence_refs
                    .iter()
                    .all(|evidence| evidence_is_in_scope(evidence, binding)),
                "confirmed finding contains evidence outside the authorized scope"
            );
            validate_identifier(
                "recommended correction",
                recommended_correction,
                MAX_TEXT_CHARS,
            )?;
            validate_identifier("safe verification", safe_verification, MAX_TEXT_CHARS)?;
        }
        AuditFindingVerdictV1::NeedsValidation {
            unresolved_fact,
            blocker,
        } => {
            validate_identifier("unresolved fact", unresolved_fact, MAX_TEXT_CHARS)?;
            validate_identifier("finding blocker", blocker, MAX_TEXT_CHARS)?;
        }
        AuditFindingVerdictV1::Rejected {
            claim,
            disproof,
            evidence_refs,
        } => {
            validate_identifier("rejected claim", claim, MAX_TEXT_CHARS)?;
            ensure!(
                claim == &candidate.claim,
                "rejected verdict must refer to the exact candidate claim"
            );
            validate_identifier("candidate disproof", disproof, MAX_TEXT_CHARS)?;
            validate_evidence_refs(evidence_refs)?;
            ensure!(
                evidence_refs
                    .iter()
                    .all(|evidence| evidence_is_in_scope(evidence, binding)),
                "rejected verdict contains evidence outside the authorized scope"
            );
            ensure!(
                evidence_refs
                    .iter()
                    .any(|evidence| evidence_supports_candidate(evidence, candidate)),
                "rejected verdict lacks disproof evidence relevant to the candidate"
            );
        }
    }
    Ok(())
}

fn validate_record_review(
    finding: &AuditFindingRecordV1,
    review: &AuditRecordReviewV1,
    binding: &AuditRunBindingV1,
) -> Result<()> {
    validate_identifier("record reviewer", &review.reviewer_id, MAX_SHORT_TEXT_CHARS)?;
    validate_identifier(
        "record review context",
        &review.context_id,
        MAX_SHORT_TEXT_CHARS,
    )?;
    ensure!(
        review.source_revision == binding.source_revision,
        "independent record review uses stale source revision"
    );
    ensure!(
        review.binding_sha256 == binding_fingerprint(binding)?,
        "independent record review uses a stale or different run binding"
    );
    ensure!(
        review.candidate_revision == finding.candidate_revision,
        "independent record review uses a stale candidate revision"
    );
    ensure!(
        finding.fingerprint(binding)? == review.candidate_fingerprint,
        "independent record review is bound to another finding"
    );
    validate_evidence_refs(&review.evidence_refs)?;
    ensure!(
        review
            .evidence_refs
            .iter()
            .all(|evidence| evidence_is_in_scope(evidence, binding)),
        "independent record review contains evidence outside the authorized scope"
    );
    ensure!(
        review
            .evidence_refs
            .iter()
            .any(|evidence| evidence_supports_candidate(evidence, &finding.candidate)),
        "independent record evidence does not support the candidate"
    );
    ensure!(
        !finding.candidate.author_ids.contains(&review.reviewer_id)
            && !finding
                .candidate_reviews
                .iter()
                .any(|candidate_review| candidate_review.reviewer_id == review.reviewer_id),
        "independent record verifier must be distinct from author and candidate verifier"
    );
    ensure!(
        !finding
            .candidate_reviews
            .iter()
            .any(|candidate_review| candidate_review.context_id == review.context_id),
        "independent record verification requires a fresh context"
    );
    Ok(())
}

fn resolve_candidate_reviews(reviews: &[AuditCandidateReviewV1]) -> Result<AuditFindingVerdictV1> {
    ensure!(!reviews.is_empty(), "candidate has no independent review");
    if reviews
        .iter()
        .all(|review| confirmed_assessments_agree(&reviews[0].assessment, &review.assessment))
    {
        return merge_agreed_assessments(reviews);
    }
    if reviews
        .iter()
        .all(|review| matches!(review.assessment, AuditFindingVerdictV1::Rejected { .. }))
    {
        let mut claims = BTreeSet::new();
        let mut disproofs = BTreeSet::new();
        let mut evidence = Vec::new();
        for review in reviews {
            if let AuditFindingVerdictV1::Rejected {
                claim,
                disproof,
                evidence_refs,
            } = &review.assessment
            {
                claims.insert(claim.clone());
                disproofs.insert(disproof.clone());
                evidence.extend(evidence_refs.clone());
            }
        }
        if claims.len() == 1 {
            evidence.sort_by(evidence_order);
            evidence.dedup();
            return Ok(AuditFindingVerdictV1::Rejected {
                claim: claims.into_iter().next().unwrap_or_default(),
                disproof: bounded_join(&disproofs, MAX_TEXT_CHARS),
                evidence_refs: evidence,
            });
        }
    }
    let mut unresolved = BTreeSet::new();
    let mut blockers = BTreeSet::new();
    for review in reviews {
        match &review.assessment {
            AuditFindingVerdictV1::NeedsValidation {
                unresolved_fact,
                blocker,
            } => {
                unresolved.insert(unresolved_fact.clone());
                blockers.insert(blocker.clone());
            }
            AuditFindingVerdictV1::Confirmed { .. } | AuditFindingVerdictV1::Rejected { .. } => {
                unresolved.insert(
                    "independent candidate reviewers reached different conclusions".to_string(),
                );
                blockers.insert("independent_candidate_review_conflict".to_string());
            }
        }
    }
    Ok(AuditFindingVerdictV1::NeedsValidation {
        unresolved_fact: bounded_join(&unresolved, MAX_TEXT_CHARS),
        blocker: bounded_join(&blockers, MAX_TEXT_CHARS),
    })
}

fn confirmed_assessments_agree(
    left: &AuditFindingVerdictV1,
    right: &AuditFindingVerdictV1,
) -> bool {
    match (left, right) {
        (
            AuditFindingVerdictV1::Confirmed {
                severity: left_severity,
                affected_principal_or_attacker: left_principal,
                protected_resource: left_resource,
                impact: left_impact,
                source_path: left_path,
                start_line: left_start,
                end_line: left_end,
                recommended_correction: left_fix,
                safe_verification: left_check,
                ..
            },
            AuditFindingVerdictV1::Confirmed {
                severity: right_severity,
                affected_principal_or_attacker: right_principal,
                protected_resource: right_resource,
                impact: right_impact,
                source_path: right_path,
                start_line: right_start,
                end_line: right_end,
                recommended_correction: right_fix,
                safe_verification: right_check,
                ..
            },
        ) => {
            left_severity == right_severity
                && left_principal == right_principal
                && left_resource == right_resource
                && left_impact == right_impact
                && left_path == right_path
                && left_start == right_start
                && left_end == right_end
                && left_fix == right_fix
                && left_check == right_check
        }
        (
            AuditFindingVerdictV1::NeedsValidation { .. },
            AuditFindingVerdictV1::NeedsValidation { .. },
        ) => true,
        (
            AuditFindingVerdictV1::Rejected { claim: left, .. },
            AuditFindingVerdictV1::Rejected { claim: right, .. },
        ) => left == right,
        _ => false,
    }
}

fn merge_agreed_assessments(reviews: &[AuditCandidateReviewV1]) -> Result<AuditFindingVerdictV1> {
    let first = &reviews[0].assessment;
    match first {
        AuditFindingVerdictV1::Confirmed {
            severity,
            affected_principal_or_attacker,
            protected_resource,
            impact,
            source_path,
            start_line,
            end_line,
            recommended_correction,
            safe_verification,
            ..
        } => {
            let mut evidence = reviews
                .iter()
                .filter_map(|review| match &review.assessment {
                    AuditFindingVerdictV1::Confirmed { evidence_refs, .. } => {
                        Some(evidence_refs.clone())
                    }
                    _ => None,
                })
                .flatten()
                .collect::<Vec<_>>();
            evidence.sort_by(evidence_order);
            evidence.dedup();
            ensure!(
                evidence.len() <= MAX_AUDIT_EVIDENCE_REFS,
                "merged verifier evidence exceeds its item limit"
            );
            Ok(AuditFindingVerdictV1::Confirmed {
                severity: *severity,
                affected_principal_or_attacker: affected_principal_or_attacker.clone(),
                protected_resource: protected_resource.clone(),
                impact: impact.clone(),
                source_path: source_path.clone(),
                start_line: *start_line,
                end_line: *end_line,
                evidence_refs: evidence,
                recommended_correction: recommended_correction.clone(),
                safe_verification: safe_verification.clone(),
            })
        }
        AuditFindingVerdictV1::NeedsValidation { .. } => {
            let mut unresolved = BTreeSet::new();
            let mut blockers = BTreeSet::new();
            for review in reviews {
                if let AuditFindingVerdictV1::NeedsValidation {
                    unresolved_fact,
                    blocker,
                } = &review.assessment
                {
                    unresolved.insert(unresolved_fact.clone());
                    blockers.insert(blocker.clone());
                }
            }
            Ok(AuditFindingVerdictV1::NeedsValidation {
                unresolved_fact: bounded_join(&unresolved, MAX_TEXT_CHARS),
                blocker: bounded_join(&blockers, MAX_TEXT_CHARS),
            })
        }
        AuditFindingVerdictV1::Rejected { .. } => {
            let mut claims = BTreeSet::new();
            let mut disproofs = BTreeSet::new();
            let mut evidence = Vec::new();
            for review in reviews {
                if let AuditFindingVerdictV1::Rejected {
                    claim,
                    disproof,
                    evidence_refs,
                } = &review.assessment
                {
                    claims.insert(claim.clone());
                    disproofs.insert(disproof.clone());
                    evidence.extend(evidence_refs.clone());
                }
            }
            evidence.sort_by(evidence_order);
            evidence.dedup();
            ensure!(
                evidence.len() <= MAX_AUDIT_EVIDENCE_REFS,
                "merged verifier evidence exceeds its item limit"
            );
            Ok(AuditFindingVerdictV1::Rejected {
                claim: claims.into_iter().next().unwrap_or_default(),
                disproof: bounded_join(&disproofs, MAX_TEXT_CHARS),
                evidence_refs: evidence,
            })
        }
    }
}

fn validate_phase_receipt(receipt: &AuditPhaseReceiptV1, run: &AuditRunV1) -> Result<()> {
    validate_identifier(
        "phase receipt operation id",
        &receipt.operation_id,
        MAX_SHORT_TEXT_CHARS,
    )?;
    validate_identifier(
        "phase artifact reference",
        &receipt.artifact_ref,
        MAX_TEXT_CHARS,
    )?;
    validate_timestamp(&receipt.timestamp)?;
    ensure!(
        receipt.operation_id == run.binding.operation_id,
        "phase receipt operation identity does not match run"
    );
    ensure!(
        receipt.binding_sha256 == run.binding_fingerprint()?,
        "phase receipt scope/revision binding does not match run"
    );
    Ok(())
}

fn validate_execution_receipt(receipt: &AuditExecutionReceiptV1, run: &AuditRunV1) -> Result<()> {
    validate_identifier(
        "executed audit operation",
        &receipt.operation,
        MAX_TEXT_CHARS,
    )?;
    validate_identifier(
        "execution receipt operation id",
        &receipt.operation_id,
        MAX_SHORT_TEXT_CHARS,
    )?;
    validate_identifier(
        "execution artifact reference",
        &receipt.artifact_ref,
        MAX_TEXT_CHARS,
    )?;
    validate_timestamp(&receipt.timestamp)?;
    ensure!(
        receipt.operation_id == run.binding.operation_id,
        "execution receipt operation identity does not match run"
    );
    ensure!(
        receipt.source_revision == run.binding.source_revision,
        "execution receipt uses a stale source revision"
    );
    ensure!(
        receipt.binding_sha256 == run.binding_fingerprint()?,
        "execution receipt scope/revision binding does not match run"
    );
    Ok(())
}

fn ensure_active_phase(run: &AuditRunV1, phase: AuditPhase) -> Result<()> {
    ensure!(
        run.status == AuditRunStatusV1::InProgress,
        "audit run is blocked or complete"
    );
    ensure!(
        run.current_phase == phase,
        "audit data is not valid in the current phase"
    );
    Ok(())
}

fn validate_timestamp(timestamp: &str) -> Result<()> {
    validate_identifier("evidence timestamp", timestamp, MAX_SHORT_TEXT_CHARS)?;
    DateTime::parse_from_rfc3339(timestamp)
        .map(|_| ())
        .map_err(|_| anyhow::anyhow!("evidence timestamp must be RFC 3339"))
}

fn validate_evidence_refs(references: &[AuditEvidenceRefV1]) -> Result<()> {
    ensure!(
        !references.is_empty() && references.len() <= MAX_AUDIT_EVIDENCE_REFS,
        "evidence references are empty or exceed their item limit"
    );
    let mut unique = BTreeSet::new();
    for reference in references {
        validate_relative_path(&reference.path, "evidence source path")?;
        ensure!(
            reference.start_line > 0 && reference.end_line >= reference.start_line,
            "evidence source line range is invalid"
        );
        ensure!(
            is_sha256(&reference.content_sha256),
            "evidence source digest must be SHA-256"
        );
        ensure!(
            unique.insert((
                reference.path.as_str(),
                reference.start_line,
                reference.end_line,
                reference.content_sha256.as_str(),
            )),
            "duplicate source evidence reference"
        );
    }
    Ok(())
}

fn validate_string_evidence_refs(references: &[String]) -> Result<()> {
    ensure!(
        !references.is_empty() && references.len() <= MAX_AUDIT_EVIDENCE_REFS,
        "evidence references are empty or exceed their item limit"
    );
    let mut unique = BTreeSet::new();
    for reference in references {
        validate_identifier("coverage evidence reference", reference, MAX_TEXT_CHARS)?;
        ensure!(
            unique.insert(reference),
            "duplicate coverage evidence reference"
        );
    }
    Ok(())
}

fn validate_identifier(label: &str, value: &str, limit: usize) -> Result<()> {
    let value = value.trim();
    ensure!(!value.is_empty(), "{label} must not be empty");
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

fn validate_audit_run_id(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    ensure!(bytes.len() == 36, "audit run id must be a canonical UUID");
    for (index, byte) in bytes.iter().copied().enumerate() {
        if matches!(index, 8 | 13 | 18 | 23) {
            ensure!(byte == b'-', "audit run id must be a canonical UUID");
        } else {
            ensure!(
                byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase(),
                "audit run id must be a canonical lowercase UUID"
            );
        }
    }
    ensure!(
        matches!(bytes[14], b'1'..=b'8') && matches!(bytes[19], b'8' | b'9' | b'a' | b'b'),
        "audit run id must use a supported UUID version and variant"
    );
    Ok(())
}

fn evidence_is_in_scope(evidence: &AuditEvidenceRefV1, binding: &AuditRunBindingV1) -> bool {
    binding
        .scope
        .iter()
        .any(|scope| path_is_within(scope, &evidence.path))
}

fn evidence_supports_candidate(
    evidence: &AuditEvidenceRefV1,
    candidate: &AuditCandidateV1,
) -> bool {
    candidate.evidence_refs.contains(evidence)
        || (evidence.path == candidate.source_path
            && evidence.start_line <= candidate.end_line
            && evidence.end_line >= candidate.start_line)
}

fn validate_blocker_id(value: &str) -> Result<()> {
    validate_identifier("audit blocker id", value, MAX_SHORT_TEXT_CHARS)?;
    ensure!(
        value == value.to_ascii_lowercase()
            && value.bytes().all(|byte| byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || b"._-".contains(&byte)),
        "audit blocker must be a lowercase machine-readable identifier"
    );
    Ok(())
}

fn validate_relative_path(value: &str, label: &str) -> Result<()> {
    validate_identifier(label, value, MAX_SHORT_TEXT_CHARS)?;
    ensure!(
        !value.starts_with('/')
            && !value.starts_with('\\')
            && !value.contains('\\')
            && !value.contains(':')
            && !value.starts_with("\\\\"),
        "{label} must be repository-relative"
    );
    ensure!(
        value == "."
            || value
                .split('/')
                .all(|component| !component.is_empty() && component != "." && component != ".."),
        "{label} is not in canonical slash-separated form"
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

fn path_is_within(scope: &str, candidate: &str) -> bool {
    let scope = scope.replace('\\', "/").trim_end_matches('/').to_string();
    let candidate = candidate.replace('\\', "/");
    scope == "."
        || candidate == scope
        || candidate
            .strip_prefix(&scope)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn binding_fingerprint(binding: &AuditRunBindingV1) -> Result<String> {
    binding.validate()?;
    let encoded = serde_json::to_vec(binding)?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

fn summarize_coverage(units: &[AuditCoverageUnitV1]) -> AuditCoverageSummaryV1 {
    let mut summary = AuditCoverageSummaryV1::default();
    for unit in units {
        match unit.disposition {
            AuditCoverageDisposition::Covered => summary.covered += 1,
            AuditCoverageDisposition::Skipped => summary.skipped += 1,
            AuditCoverageDisposition::Deferred => summary.deferred += 1,
            AuditCoverageDisposition::OutOfScope => summary.out_of_scope += 1,
        }
    }
    summary
}

fn evidence_order(left: &AuditEvidenceRefV1, right: &AuditEvidenceRefV1) -> std::cmp::Ordering {
    left.path
        .cmp(&right.path)
        .then_with(|| left.start_line.cmp(&right.start_line))
        .then_with(|| left.end_line.cmp(&right.end_line))
        .then_with(|| left.content_sha256.cmp(&right.content_sha256))
}

fn is_strictly_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn is_hex(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && is_hex(value)
}

fn bounded_join(values: &BTreeSet<String>, limit: usize) -> String {
    let mut joined = String::new();
    for value in values {
        let separator = if joined.is_empty() { "" } else { "; " };
        if joined.chars().count() + separator.len() + value.chars().count() > limit {
            break;
        }
        joined.push_str(separator);
        joined.push_str(value);
    }
    joined
}
