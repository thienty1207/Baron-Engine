//! Fail-closed capability, sandbox, and recovery support for security audits.
//!
//! This module does not run target code. It only reads bounded Baron state,
//! verifies previously issued Baron receipts, fingerprints the declared
//! source scope, and publishes explicit artifacts without replacement.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::capability::{
    load_capability_state, load_registry, normalize_identifier, observation_matches_provider,
    CapabilityExecutionEvidence, Presence,
};
use crate::config::load_project_config;
use crate::execution_receipt::{
    load_verified_receipts_strict, receipt_matches_verified_context, ReceiptContext,
};
use crate::identity::project_id_for_path;
use crate::operation::OperationContext;
use crate::safe_io::{create_new_file, existing_directory_chain, open_regular_file_for_read};
use crate::security_audit::exclusion_rule;
use crate::security_audit_workflow::{
    candidate_fingerprint, validate_audit_run, AuditPhase, AuditRunBindingV1, AuditRunStatusV1,
    AuditRunV1, AuditValidationScopeV1,
};

pub const AUDIT_RECOVERY_CHECKPOINT_SCHEMA_VERSION: u32 = 1;
pub const MAX_AUDIT_CHECKPOINT_JSON_BYTES: usize = 12 * 1024 * 1024;
pub const MAX_AUDIT_CHECKPOINT_SOURCE_FILES: usize = 20_000;
pub const MAX_AUDIT_CHECKPOINT_FILE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_AUDIT_CHECKPOINT_SOURCE_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_AUDIT_CHECKPOINT_PENDING_WORK: usize = 256;

const MAX_AUDIT_CHECKPOINT_PATH_BYTES: usize = 1024;
const MAX_AUDIT_CHECKPOINT_ACTION_BYTES: usize = 1024;
const MAX_AUDIT_CHECKPOINT_ROOT_BYTES: usize = 4096;
const MAX_AUDIT_CAPABILITY_PROVIDERS: usize = 512;
const MAX_AUDIT_CAPABILITY_EVIDENCE: usize = 512;
const MAX_AUDIT_CHECKPOINT_CHANGED_PATHS: usize = 256;

const REQUIRED_SANDBOX_CONTROLS: [AuditSandboxControlV1; 10] = [
    AuditSandboxControlV1::NetworkDenied,
    AuditSandboxControlV1::SanitizedEnvironment,
    AuditSandboxControlV1::ReadOnlySourceAndToolchain,
    AuditSandboxControlV1::AssignedScratchOnly,
    AuditSandboxControlV1::CpuLimit,
    AuditSandboxControlV1::MemoryLimit,
    AuditSandboxControlV1::ProcessLimit,
    AuditSandboxControlV1::DiskLimit,
    AuditSandboxControlV1::FileSizeLimit,
    AuditSandboxControlV1::TimeLimit,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCapabilityUsageV1 {
    pub capability: String,
    pub provider: String,
    pub registered: bool,
    /// `true` means the exact provider definition has a compatible cached
    /// observation of presence. This is historical observation, not a live
    /// probe and not execution evidence.
    pub present: bool,
    pub executed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AuditSandboxControlV1 {
    NetworkDenied,
    SanitizedEnvironment,
    ReadOnlySourceAndToolchain,
    AssignedScratchOnly,
    CpuLimit,
    MemoryLimit,
    ProcessLimit,
    DiskLimit,
    FileSizeLimit,
    TimeLimit,
}

impl AuditSandboxControlV1 {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NetworkDenied => "network_denial_unverified",
            Self::SanitizedEnvironment => "sanitized_environment_unverified",
            Self::ReadOnlySourceAndToolchain => "read_only_source_toolchain_unverified",
            Self::AssignedScratchOnly => "assigned_scratch_unverified",
            Self::CpuLimit => "cpu_limit_unverified",
            Self::MemoryLimit => "memory_limit_unverified",
            Self::ProcessLimit => "process_limit_unverified",
            Self::DiskLimit => "disk_limit_unverified",
            Self::FileSizeLimit => "file_size_limit_unverified",
            Self::TimeLimit => "time_limit_unverified",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditSandboxControlStatusV1 {
    pub control: AuditSandboxControlV1,
    pub enforced: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditSandboxReadinessV1 {
    pub schema_version: u32,
    pub controls: Vec<AuditSandboxControlStatusV1>,
    pub execution_allowed: bool,
    pub blockers: Vec<String>,
}

impl Default for AuditSandboxReadinessV1 {
    fn default() -> Self {
        Self::unverified()
    }
}

impl AuditSandboxReadinessV1 {
    /// A conservative readiness value used when the safety capability itself
    /// cannot be assessed. No target execution is authorized by this state.
    pub fn unverified() -> Self {
        let controls = REQUIRED_SANDBOX_CONTROLS
            .iter()
            .copied()
            .map(|control| AuditSandboxControlStatusV1 {
                control,
                enforced: false,
            })
            .collect::<Vec<_>>();
        let mut blockers = vec!["sandbox_backend_unverified".to_string()];
        blockers.extend(
            controls
                .iter()
                .map(|control| control.control.code().to_string()),
        );
        Self {
            schema_version: AUDIT_RECOVERY_CHECKPOINT_SCHEMA_VERSION,
            controls,
            execution_allowed: false,
            blockers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditSourceFileFingerprintV1 {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRecoveryCheckpointV1 {
    pub schema_version: u32,
    pub checkpoint_sha256: String,
    pub created_at: String,
    pub run: AuditRunV1,
    pub binding_sha256: String,
    /// Hash of the existing local identity binding. The binding itself is
    /// never copied into a checkpoint.
    pub project_identity_sha256: String,
    pub phase: AuditPhase,
    pub coverage_sha256: Option<String>,
    pub candidate_fingerprints: Vec<String>,
    pub source_files: Vec<AuditSourceFileFingerprintV1>,
    pub source_content_sha256: String,
    pub git_dirty_state_sha256: String,
    pub pending_work: Vec<String>,
    pub next_safe_action: String,
}

impl AuditRecoveryCheckpointV1 {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == AUDIT_RECOVERY_CHECKPOINT_SCHEMA_VERSION,
            "unsupported security-audit checkpoint schema"
        );
        validate_audit_run(&self.run).context("checkpoint run is invalid")?;
        ensure!(
            !contains_secret_like_text(&serde_json::to_value(&self.run)?),
            "checkpoint run contains credential-like evidence"
        );
        ensure!(
            is_sha256(&self.checkpoint_sha256),
            "checkpoint digest is invalid"
        );
        ensure!(
            is_sha256(&self.binding_sha256),
            "checkpoint binding digest is invalid"
        );
        ensure!(
            self.binding_sha256 == self.run.binding_fingerprint()?,
            "checkpoint binding does not match the audit run"
        );
        ensure!(
            is_sha256(&self.project_identity_sha256),
            "project identity digest is invalid"
        );
        ensure!(
            chrono::DateTime::parse_from_rfc3339(&self.created_at).is_ok(),
            "checkpoint timestamp is invalid"
        );
        ensure!(
            self.phase == self.run.current_phase,
            "checkpoint phase does not match the audit run"
        );
        let expected_coverage = self
            .run
            .coverage_ledger
            .as_ref()
            .map(sha256_json)
            .transpose()?;
        ensure!(
            self.coverage_sha256 == expected_coverage,
            "checkpoint coverage does not match the audit run"
        );
        let mut fingerprints = self
            .run
            .findings
            .iter()
            .map(|finding| candidate_fingerprint(&self.run.binding, &finding.candidate))
            .collect::<Result<Vec<_>>>()?;
        fingerprints.sort();
        ensure!(
            fingerprints == self.candidate_fingerprints
                && strictly_sorted_unique(&self.candidate_fingerprints),
            "checkpoint candidate fingerprints do not match the audit run"
        );
        ensure!(
            !self.source_files.is_empty()
                && self.source_files.len() <= MAX_AUDIT_CHECKPOINT_SOURCE_FILES,
            "checkpoint source inventory is empty or exceeds its item limit"
        );
        ensure!(
            strictly_sorted_unique_by(&self.source_files, |file| &file.path),
            "checkpoint source paths must be sorted and unique"
        );
        let mut total_bytes = 0_u64;
        for file in &self.source_files {
            validate_checkpoint_relative_path(&file.path)?;
            ensure!(
                redact_sensitive_text(&file.path) == file.path,
                "checkpoint source path contains credential-like text"
            );
            ensure!(
                file.size_bytes <= MAX_AUDIT_CHECKPOINT_FILE_BYTES,
                "checkpoint source file exceeds its size limit"
            );
            ensure!(is_sha256(&file.sha256), "checkpoint source hash is invalid");
            total_bytes = total_bytes.saturating_add(file.size_bytes);
            ensure!(
                total_bytes <= MAX_AUDIT_CHECKPOINT_SOURCE_BYTES,
                "checkpoint source inventory exceeds its total size limit"
            );
        }
        ensure!(
            self.source_content_sha256 == sha256_json(&self.source_files)?,
            "checkpoint source inventory digest is invalid"
        );
        ensure!(
            is_sha256(&self.git_dirty_state_sha256),
            "checkpoint Git dirty-state digest is invalid"
        );
        ensure!(
            self.pending_work.len() <= MAX_AUDIT_CHECKPOINT_PENDING_WORK
                && self.pending_work.iter().all(|item| {
                    !item.trim().is_empty()
                        && item.len() <= MAX_AUDIT_CHECKPOINT_ACTION_BYTES
                        && !item.chars().any(char::is_control)
                        && redact_sensitive_text(item) == *item
                }),
            "checkpoint pending work is invalid or exceeds its limit"
        );
        ensure!(
            !self.next_safe_action.trim().is_empty()
                && self.next_safe_action.len() <= MAX_AUDIT_CHECKPOINT_ACTION_BYTES
                && !self.next_safe_action.chars().any(char::is_control),
            "checkpoint next action is invalid or exceeds its limit"
        );
        ensure!(
            redact_sensitive_text(&self.next_safe_action) == self.next_safe_action,
            "checkpoint next action contains credential-like text"
        );
        let mut normalized = self.clone();
        normalized.checkpoint_sha256.clear();
        ensure!(
            self.checkpoint_sha256 == sha256_json(&normalized)?,
            "checkpoint integrity digest does not match"
        );
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditResumeAssessmentV1 {
    pub schema_version: u32,
    pub run_status: AuditRunStatusV1,
    pub source_revision_matches: bool,
    pub dirty_state_matches: bool,
    pub project_identity_matches: bool,
    pub source_content_matches: bool,
    pub fresh_run_required: bool,
    pub can_resume: bool,
    pub changed_paths: Vec<String>,
    pub changed_paths_truncated: bool,
    pub blockers: Vec<String>,
    pub next_safe_action: String,
}

/// Read registered/present/executed capability facts without probing tools,
/// installing providers, or trusting summary-only execution claims.
pub fn inspect_audit_capabilities(
    repo_root: impl AsRef<Path>,
    operation: &OperationContext,
    evidence: &[CapabilityExecutionEvidence],
) -> Result<Vec<AuditCapabilityUsageV1>> {
    ensure!(
        evidence.len() <= MAX_AUDIT_CAPABILITY_EVIDENCE,
        "security-audit capability evidence exceeds its item limit"
    );
    let repo_root = repo_root.as_ref();
    let registry = load_registry(repo_root).context("could not safely load capability registry")?;
    ensure!(
        registry.schema_version == 1,
        "capability registry schema is unsupported"
    );
    ensure!(
        registry.providers.len() <= MAX_AUDIT_CAPABILITY_PROVIDERS,
        "capability registry exceeds the security-audit item limit"
    );
    let state =
        load_capability_state(repo_root).context("could not safely load capability state cache")?;
    if let Some(state) = &state {
        ensure!(
            state.observations.len() <= MAX_AUDIT_CAPABILITY_PROVIDERS,
            "capability state cache exceeds the security-audit item limit"
        );
    }
    let adapter = operation.adapter_kind();
    let matching_state = state
        .as_ref()
        .filter(|state| state.adapter.supported() == Some(adapter));
    let receipts = load_verified_receipts_strict(repo_root)
        .context("security-audit execution receipts could not be verified")?;
    let operation_binding = operation_receipt_binding(operation);
    let mut usages = Vec::new();
    let mut registered_pairs = BTreeSet::new();
    let mut provider_names = BTreeSet::new();

    for provider in &registry.providers {
        let Some(capability) = normalize_identifier(&provider.capability) else {
            continue;
        };
        let Some(provider_name) = normalize_identifier(&provider.name) else {
            continue;
        };
        ensure!(
            provider_names.insert(provider_name.clone()),
            "capability registry contains duplicate provider identities"
        );
        registered_pairs.insert((capability.clone(), provider_name.clone()));
        let compatible = provider.adapters.is_empty() || provider.adapters.contains(&adapter);
        let present = compatible
            && matching_state.is_some_and(|state| {
                state.observations.iter().any(|observation| {
                    observation.compatible
                        && observation.presence == Presence::Present
                        && observation_matches_provider(observation, provider)
                })
            });
        let executed = present
            && operation_binding.as_ref().is_some_and(|binding| {
                evidence.iter().any(|item| {
                    normalize_identifier(&item.capability).as_deref() == Some(capability.as_str())
                        && normalize_identifier(&item.provider).as_deref()
                            == Some(provider_name.as_str())
                        && item.summary.len() <= 4096
                        && !item.summary.trim().is_empty()
                        && item.receipt_id.as_deref().is_some_and(|receipt_id| {
                            receipts.iter().any(|receipt| {
                                receipt.receipt_id == receipt_id.trim()
                                    && receipt.capability == capability
                                    && receipt.provider == provider_name
                                    && item.task_id.as_deref() == operation.task_id.as_deref()
                                    && item.operation_id.as_deref()
                                        == operation.operation_id.as_deref()
                                    && item.session_id.as_deref() == operation.session_id.as_deref()
                                    && item.request_id.as_deref() == operation.request_id.as_deref()
                                    && item.gate_kind.as_deref() == Some("capability_execution")
                                    && receipt_matches_verified_context(receipt, binding)
                                        .unwrap_or(false)
                            })
                        })
                })
            });
        usages.push(AuditCapabilityUsageV1 {
            capability,
            provider: provider_name,
            registered: true,
            present,
            executed,
            blocker: if !present {
                Some("provider_presence_unverified".to_string())
            } else if !executed {
                Some("current_operation_execution_unverified".to_string())
            } else {
                None
            },
        });
    }

    for item in evidence {
        let (Some(capability), Some(provider)) = (
            normalize_identifier(&item.capability),
            normalize_identifier(&item.provider),
        ) else {
            continue;
        };
        if !registered_pairs.contains(&(capability.clone(), provider.clone())) {
            usages.push(AuditCapabilityUsageV1 {
                capability,
                provider,
                registered: false,
                present: false,
                executed: false,
                blocker: Some("provider_not_registered".to_string()),
            });
        }
    }
    usages.sort_by(|left, right| {
        left.capability
            .cmp(&right.capability)
            .then_with(|| left.provider.cmp(&right.provider))
    });
    usages.dedup_by(|left, right| {
        left.capability == right.capability && left.provider == right.provider
    });
    Ok(usages)
}

/// Report every required OS control as unverified until a dedicated trusted
/// sandbox backend can prove it. This function never invokes a target process.
pub fn assess_audit_sandbox(
    _repo_root: impl AsRef<Path>,
    _operation: &OperationContext,
) -> Result<AuditSandboxReadinessV1> {
    Ok(AuditSandboxReadinessV1::unverified())
}

/// Construct a checkpoint only when the current Git revision, Baron project
/// identity, and bounded source inventory match the run's declared binding.
pub fn create_audit_checkpoint(
    repo_root: impl AsRef<Path>,
    run: AuditRunV1,
    pending_work: Vec<String>,
    next_safe_action: impl Into<String>,
) -> Result<AuditRecoveryCheckpointV1> {
    let root = canonical_repository_root(repo_root.as_ref())?;
    validate_audit_run(&run).context("audit run is invalid")?;
    let next_safe_action = next_safe_action.into();
    validate_checkpoint_text(&pending_work, &next_safe_action)?;
    let next_safe_action = redact_sensitive_text(&next_safe_action);
    let pending_work = pending_work
        .into_iter()
        .map(|item| redact_sensitive_text(&item))
        .collect::<Vec<_>>();
    ensure_run_has_no_secret_like_text(&run)?;
    let (project_id, identity_sha256) = current_project_identity(&root)?;
    ensure!(
        project_id == run.binding.project_id,
        "audit run belongs to another Baron project identity"
    );
    let revision = current_git_revision(&root)?;
    ensure!(
        revision == run.binding.source_revision,
        "audit run source revision is not the current Git revision"
    );
    let source_files = snapshot_source_scope(&root, &run.binding)?;
    let source_content_sha256 = sha256_json(&source_files)?;
    let git_dirty_state_sha256 = current_git_dirty_state(&root, &run.binding.scope)?;
    ensure_dirty_state_matches_binding(&run.binding, &git_dirty_state_sha256)?;
    let source_files_after = snapshot_source_scope(&root, &run.binding)?;
    let dirty_state_after = current_git_dirty_state(&root, &run.binding.scope)?;
    let revision_after = current_git_revision(&root)?;
    ensure!(
        source_files == source_files_after
            && git_dirty_state_sha256 == dirty_state_after
            && revision == revision_after,
        "source changed while the recovery checkpoint was being created"
    );
    ensure_dirty_state_matches_binding(&run.binding, &dirty_state_after)?;
    let mut candidate_fingerprints = run
        .findings
        .iter()
        .map(|finding| candidate_fingerprint(&run.binding, &finding.candidate))
        .collect::<Result<Vec<_>>>()?;
    candidate_fingerprints.sort();
    let coverage_sha256 = run.coverage_ledger.as_ref().map(sha256_json).transpose()?;
    let mut checkpoint = AuditRecoveryCheckpointV1 {
        schema_version: AUDIT_RECOVERY_CHECKPOINT_SCHEMA_VERSION,
        checkpoint_sha256: String::new(),
        created_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        binding_sha256: run.binding_fingerprint()?,
        project_identity_sha256: identity_sha256,
        phase: run.current_phase,
        coverage_sha256,
        candidate_fingerprints,
        source_files,
        source_content_sha256,
        git_dirty_state_sha256,
        pending_work,
        next_safe_action,
        run,
    };
    checkpoint.checkpoint_sha256 = checkpoint_content_digest(&checkpoint)?;
    checkpoint.validate()?;
    Ok(checkpoint)
}

pub fn validate_audit_checkpoint_json(input: &[u8]) -> Result<AuditRecoveryCheckpointV1> {
    ensure!(
        !input.is_empty() && input.len() <= MAX_AUDIT_CHECKPOINT_JSON_BYTES,
        "security-audit checkpoint is empty or exceeds its size limit"
    );
    let checkpoint: AuditRecoveryCheckpointV1 = serde_json::from_slice(input).map_err(|_| {
        anyhow::anyhow!("security-audit checkpoint JSON is malformed or incompatible")
    })?;
    checkpoint.validate()?;
    Ok(checkpoint)
}

pub fn read_audit_checkpoint_file(path: impl AsRef<Path>) -> Result<AuditRecoveryCheckpointV1> {
    let bytes = read_bounded_regular_file(
        path.as_ref(),
        MAX_AUDIT_CHECKPOINT_JSON_BYTES,
        "security-audit checkpoint",
    )?
    .context("security-audit checkpoint file is missing")?;
    validate_audit_checkpoint_json(&bytes)
}

/// Compare a checkpoint with the current project identity, Git revision, and
/// exact scoped file inventory. A valid result is a resume hint, not an
/// automatic state transition or proof that a prior check passed.
pub fn assess_audit_resume(
    repo_root: impl AsRef<Path>,
    checkpoint: &AuditRecoveryCheckpointV1,
) -> Result<AuditResumeAssessmentV1> {
    checkpoint.validate()?;
    let root = canonical_repository_root(repo_root.as_ref())?;
    let mut blockers = Vec::new();
    let current_identity = current_project_identity(&root);
    let project_identity_matches_before =
        current_identity.as_ref().is_ok_and(|(project_id, digest)| {
            project_id == &checkpoint.run.binding.project_id
                && digest == &checkpoint.project_identity_sha256
        });
    let current_revision = current_git_revision(&root);
    let current_dirty_state = current_git_dirty_state(&root, &checkpoint.run.binding.scope);
    let current_files = snapshot_source_scope(&root, &checkpoint.run.binding)?;
    let current_identity_after = current_project_identity(&root);
    let current_revision_after = current_git_revision(&root);
    let current_dirty_state_after = current_git_dirty_state(&root, &checkpoint.run.binding.scope);
    let current_files_after = snapshot_source_scope(&root, &checkpoint.run.binding)?;
    let project_identity_matches = project_identity_matches_before
        && current_identity_after
            .as_ref()
            .is_ok_and(|(project_id, digest)| {
                project_id == &checkpoint.run.binding.project_id
                    && digest == &checkpoint.project_identity_sha256
            });
    if !project_identity_matches {
        blockers.push("project_identity_changed".to_string());
    }
    let source_revision_matches = current_revision
        .as_ref()
        .is_ok_and(|revision| revision == &checkpoint.run.binding.source_revision)
        && current_revision_after
            .as_ref()
            .is_ok_and(|revision| revision == &checkpoint.run.binding.source_revision);
    if !source_revision_matches {
        blockers.push("source_revision_changed_or_unavailable".to_string());
    }
    let dirty_state_matches = current_dirty_state
        .as_ref()
        .is_ok_and(|digest| digest == &checkpoint.git_dirty_state_sha256)
        && current_dirty_state_after
            .as_ref()
            .is_ok_and(|digest| digest == &checkpoint.git_dirty_state_sha256);
    if !dirty_state_matches {
        blockers.push("source_dirty_state_changed_or_unavailable".to_string());
    }
    let source_content_matches = current_files == checkpoint.source_files
        && current_files_after == checkpoint.source_files
        && current_files == current_files_after;
    let current_files = current_files_after;
    let (changed_paths, changed_paths_truncated) = if source_content_matches {
        (Vec::new(), false)
    } else {
        changed_source_paths(&checkpoint.source_files, &current_files)
    };
    if !source_content_matches {
        blockers.push("source_content_changed".to_string());
    }
    let fresh_run_required = !project_identity_matches
        || !source_revision_matches
        || !dirty_state_matches
        || !source_content_matches;
    let can_resume = !fresh_run_required
        && (checkpoint.run.status == AuditRunStatusV1::InProgress
            || (checkpoint.run.status == AuditRunStatusV1::Incomplete
                && checkpoint.run.failure.as_ref().is_some_and(|failure| {
                    failure.class
                        == crate::security_audit_workflow::AuditFailureClassV1::Interrupted
                })));
    if !can_resume && !fresh_run_required {
        blockers.push("run_state_requires_new_audit_decision".to_string());
    }
    blockers.sort();
    let next_safe_action = if fresh_run_required {
        "start a fresh audit from the current source".to_string()
    } else if can_resume {
        redact_sensitive_text(&checkpoint.next_safe_action)
    } else {
        "reassess audit blockers before continuing".to_string()
    };
    Ok(AuditResumeAssessmentV1 {
        schema_version: AUDIT_RECOVERY_CHECKPOINT_SCHEMA_VERSION,
        run_status: checkpoint.run.status,
        source_revision_matches,
        dirty_state_matches,
        project_identity_matches,
        source_content_matches,
        fresh_run_required,
        can_resume,
        changed_paths,
        changed_paths_truncated,
        blockers,
        next_safe_action,
    })
}

/// Publish one checkpoint with explicit create-only semantics. Identical
/// retries succeed; an existing different artifact is preserved.
pub fn publish_audit_checkpoint(
    output_path: impl AsRef<Path>,
    checkpoint: &AuditRecoveryCheckpointV1,
) -> Result<bool> {
    checkpoint.validate()?;
    let bytes = serde_json::to_vec_pretty(checkpoint)?;
    ensure!(
        bytes.len() <= MAX_AUDIT_CHECKPOINT_JSON_BYTES,
        "security-audit checkpoint exceeds its publication limit"
    );
    publish_create_only(output_path.as_ref(), &bytes)
}

pub fn redact_audit_report(
    report: &crate::security_audit_workflow::AuditReportV1,
) -> Result<Value> {
    let mut value = serde_json::to_value(report)?;
    redact_json_strings(&mut value);
    Ok(value)
}

/// Publish a validated audit report as redacted create-only JSON.
pub fn publish_audit_report(
    output_path: impl AsRef<Path>,
    report: &crate::security_audit_workflow::AuditReportV1,
) -> Result<bool> {
    ensure!(
        report.validation_scope == AuditValidationScopeV1::SchemaAndWorkflowContractOnly,
        "unsupported security-audit report validation scope"
    );
    ensure!(
        !report.clean && report.application_task_blocked && !report.proof_gate_trace_claimed,
        "security-audit report contains an unsupported completion claim"
    );
    let value = redact_audit_report(report)?;
    let bytes = serde_json::to_vec_pretty(&value)?;
    ensure!(
        bytes.len() <= MAX_AUDIT_CHECKPOINT_JSON_BYTES,
        "security-audit report exceeds its publication limit"
    );
    publish_create_only(output_path.as_ref(), &bytes)
}

fn operation_receipt_binding(operation: &OperationContext) -> Option<ReceiptContext> {
    Some(ReceiptContext::new(
        operation.task_id.as_deref()?,
        operation.operation_id.as_deref()?,
        operation.adapter.as_str(),
        operation.session_id.as_deref()?,
        operation.request_id.as_deref()?,
        "capability_execution",
    ))
}

fn canonical_repository_root(path: &Path) -> Result<PathBuf> {
    ensure!(
        path.as_os_str().len() <= MAX_AUDIT_CHECKPOINT_ROOT_BYTES,
        "repository path exceeds its size limit"
    );
    let root = path
        .canonicalize()
        .context("security-audit repository path is unavailable")?;
    ensure!(
        root.is_dir(),
        "security-audit repository path is not a directory"
    );
    Ok(root)
}

fn current_project_identity(root: &Path) -> Result<(String, String)> {
    let config =
        load_project_config(root).context("Baron project identity could not be safely read")?;
    let path_project_id =
        project_id_for_path(root).context("Baron project path identity could not be calculated")?;
    ensure!(
        !config.identity_binding.trim().is_empty()
            && config.identity_binding.len() <= 256
            && !config.identity_binding.chars().any(char::is_control),
        "Baron project identity binding is missing or invalid"
    );
    ensure!(
        config.project_id == path_project_id,
        "Baron project ID does not match the current checkout path"
    );
    Ok((
        config.project_id,
        sha256_bytes(config.identity_binding.as_bytes()),
    ))
}

fn current_git_revision(root: &Path) -> Result<String> {
    let prefix = git_output(root, &["rev-parse", "--show-prefix"])?;
    ensure!(
        prefix.trim().is_empty(),
        "repository path is not the Git worktree root"
    );
    let revision = git_output(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let revision = revision.trim().to_ascii_lowercase();
    ensure!(
        matches!(revision.len(), 40 | 64) && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "Git did not return a full source revision"
    );
    Ok(revision)
}

fn current_git_dirty_state(root: &Path, scope: &[String]) -> Result<String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "core.fsmonitor=false",
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
        ])
        .args(scope)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        );
    let output = command
        .output()
        .context("Git dirty-state metadata is unavailable")?;
    ensure!(
        output.status.success() && output.stdout.len() <= 16 * 1024 * 1024,
        "Git dirty-state metadata could not be verified or exceeds its limit"
    );
    Ok(sha256_bytes(&output.stdout))
}

fn ensure_dirty_state_matches_binding(
    binding: &AuditRunBindingV1,
    current_dirty_state_sha256: &str,
) -> Result<()> {
    let clean_state_sha256 = sha256_bytes(&[]);
    if binding.source_dirty {
        ensure!(
            current_dirty_state_sha256 != clean_state_sha256
                && binding.dirty_state_sha256.as_deref() == Some(current_dirty_state_sha256),
            "audit run dirty-state binding does not match the current scoped source"
        );
    } else {
        ensure!(
            current_dirty_state_sha256 == clean_state_sha256
                && binding.dirty_state_sha256.is_none(),
            "clean audit run does not match the current scoped source state"
        );
    }
    Ok(())
}

fn git_output(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .output()
        .context("Git metadata command is unavailable")?;
    ensure!(
        output.status.success(),
        "Git metadata could not be verified"
    );
    String::from_utf8(output.stdout).context("Git metadata output is not valid UTF-8")
}

fn snapshot_source_scope(
    root: &Path,
    binding: &AuditRunBindingV1,
) -> Result<Vec<AuditSourceFileFingerprintV1>> {
    binding
        .validate()
        .context("audit source binding is invalid")?;
    let mut files = BTreeMap::new();
    let mut total_bytes = 0_u64;
    let mut visited = 0_usize;
    for scope_path in &binding.scope {
        let relative = validate_checkpoint_relative_path(scope_path)?;
        let absolute = root.join(relative);
        visit_scope_path(root, &absolute, &mut files, &mut total_bytes, &mut visited)?;
    }
    ensure!(
        !files.is_empty(),
        "audit source scope contains no fingerprintable source files"
    );
    Ok(files.into_values().collect())
}

fn visit_scope_path(
    root: &Path,
    path: &Path,
    files: &mut BTreeMap<String, AuditSourceFileFingerprintV1>,
    total_bytes: &mut u64,
    visited: &mut usize,
) -> Result<()> {
    *visited = visited.saturating_add(1);
    ensure!(
        *visited <= MAX_AUDIT_CHECKPOINT_SOURCE_FILES,
        "audit source inventory exceeds its entry limit"
    );
    let metadata = fs::symlink_metadata(path).with_context(|| {
        format!(
            "audit source path is unavailable: {}",
            display_relative(root, path)
        )
    })?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "audit source scope contains a symlink"
    );
    if metadata.is_dir() {
        ensure!(
            existing_directory_chain(path).context("audit source directory safety check failed")?,
            "audit source directory is unavailable"
        );
        if exclusion_rule(path).is_some() {
            return Ok(());
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(path).context("audit source directory cannot be inventoried")? {
            entries.push(
                entry
                    .context("audit source directory contains an unreadable entry")?
                    .path(),
            );
            ensure!(
                entries.len() <= MAX_AUDIT_CHECKPOINT_SOURCE_FILES,
                "audit source directory exceeds its entry limit"
            );
        }
        entries.sort();
        for entry in entries {
            visit_scope_path(root, &entry, files, total_bytes, visited)?;
        }
        return Ok(());
    }
    ensure!(
        metadata.is_file(),
        "audit source scope contains a special file"
    );
    let relative = safe_relative_path(root, path)?;
    if files.contains_key(&relative) {
        return Ok(());
    }
    ensure!(
        metadata.len() <= MAX_AUDIT_CHECKPOINT_FILE_BYTES,
        "audit source file exceeds its fingerprint size limit"
    );
    *total_bytes = total_bytes.saturating_add(metadata.len());
    ensure!(
        *total_bytes <= MAX_AUDIT_CHECKPOINT_SOURCE_BYTES,
        "audit source inventory exceeds its total size limit"
    );
    let file = open_regular_file_for_read(path)
        .context("audit source file safety check failed")?
        .context("audit source file disappeared during inventory")?;
    let opened_metadata = file
        .metadata()
        .context("audit source file metadata is unavailable")?;
    ensure!(
        opened_metadata.len() == metadata.len()
            && opened_metadata.modified().ok() == metadata.modified().ok(),
        "audit source file changed during inventory"
    );
    let mut digest = Sha256::new();
    let read = std::io::copy(
        &mut file.take(MAX_AUDIT_CHECKPOINT_FILE_BYTES.saturating_add(1)),
        &mut DigestWriter(&mut digest),
    )
    .context("audit source file could not be fingerprinted")?;
    ensure!(
        read == metadata.len() && read <= MAX_AUDIT_CHECKPOINT_FILE_BYTES,
        "audit source file changed or exceeded its fingerprint size limit"
    );
    let after = fs::symlink_metadata(path).context("audit source file changed during inventory")?;
    ensure!(
        after.is_file()
            && !after.file_type().is_symlink()
            && after.len() == read
            && after.modified().ok() == opened_metadata.modified().ok(),
        "audit source file changed during inventory"
    );
    files.insert(
        relative.clone(),
        AuditSourceFileFingerprintV1 {
            path: relative,
            size_bytes: read,
            sha256: format!("{:x}", digest.finalize()),
        },
    );
    Ok(())
}

struct DigestWriter<'a>(&'a mut Sha256);

impl std::io::Write for DigestWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.update(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn safe_relative_path(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .context("audit source path escaped repository root")?;
    let value = relative
        .to_str()
        .context("audit source path is not valid UTF-8")?
        .replace('\\', "/");
    validate_checkpoint_relative_path(&value)?;
    Ok(value)
}

fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .map(|path| path.replace('\\', "/"))
        .unwrap_or_else(|| "<outside-scope>".to_string())
}

fn validate_checkpoint_relative_path(value: &str) -> Result<PathBuf> {
    ensure!(
        !value.is_empty()
            && value.len() <= MAX_AUDIT_CHECKPOINT_PATH_BYTES
            && !value.chars().any(char::is_control),
        "checkpoint source path is empty, unsafe, or too long"
    );
    let path = Path::new(value);
    ensure!(
        !path.is_absolute(),
        "checkpoint source path must be relative"
    );
    for component in path.components() {
        ensure!(
            matches!(component, Component::Normal(_) | Component::CurDir),
            "checkpoint source path contains traversal"
        );
    }
    Ok(path.to_path_buf())
}

fn changed_source_paths(
    expected: &[AuditSourceFileFingerprintV1],
    current: &[AuditSourceFileFingerprintV1],
) -> (Vec<String>, bool) {
    let expected = expected
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let current = current
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let mut changed = BTreeSet::new();
    for (path, old) in &expected {
        if current.get(path).is_none_or(|new| *new != *old) {
            changed.insert((*path).to_string());
        }
    }
    for path in current.keys() {
        if !expected.contains_key(path) {
            changed.insert((*path).to_string());
        }
    }
    let truncated = changed.len() > MAX_AUDIT_CHECKPOINT_CHANGED_PATHS;
    (
        changed
            .into_iter()
            .take(MAX_AUDIT_CHECKPOINT_CHANGED_PATHS)
            .collect(),
        truncated,
    )
}

fn publish_create_only(path: &Path, bytes: &[u8]) -> Result<bool> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    ensure!(
        existing_directory_chain(parent).context("artifact parent safety check failed")?,
        "artifact parent directory must already exist"
    );
    let parent = parent
        .canonicalize()
        .context("artifact parent directory is unavailable")?;
    let name = path
        .file_name()
        .filter(|name| !name.is_empty())
        .context("artifact file name is invalid")?;
    let target = parent.join(name);
    if let Some(existing) =
        read_bounded_regular_file(&target, bytes.len(), "existing audit artifact")?
    {
        ensure!(
            existing == bytes,
            "existing audit artifact differs; refusing to overwrite it"
        );
        return Ok(false);
    }
    match create_new_file(&target, bytes) {
        Ok(()) => Ok(true),
        Err(error) => {
            if let Some(existing) =
                read_bounded_regular_file(&target, bytes.len(), "existing audit artifact")?
            {
                ensure!(
                    existing == bytes,
                    "existing audit artifact differs; refusing to overwrite it"
                );
                Ok(false)
            } else {
                Err(error).context("audit artifact could not be published atomically")
            }
        }
    }
}

fn read_bounded_regular_file(path: &Path, limit: usize, label: &str) -> Result<Option<Vec<u8>>> {
    let Some(mut file) = open_regular_file_for_read(path)
        .with_context(|| format!("{label} file could not be opened safely"))?
    else {
        return Ok(None);
    };
    let metadata = file
        .metadata()
        .with_context(|| format!("{label} file metadata is unavailable"))?;
    let limit_u64 = u64::try_from(limit).context("bounded file read limit is invalid")?;
    ensure!(
        metadata.len() <= limit_u64,
        "{label} file exceeds the byte limit"
    );
    let capacity = usize::try_from(metadata.len())
        .context("bounded file size cannot be represented in memory")?;
    let mut bytes = Vec::with_capacity(capacity);
    {
        let mut limited_reader = (&mut file).take(limit_u64.saturating_add(1));
        limited_reader
            .read_to_end(&mut bytes)
            .with_context(|| format!("{label} file could not be read"))?;
    }
    ensure!(
        bytes.len() <= limit,
        "{label} file grew beyond the byte limit while being read"
    );
    let after = file
        .metadata()
        .with_context(|| format!("{label} file metadata changed during read"))?;
    ensure!(
        after.len() == metadata.len()
            && after.modified().ok() == metadata.modified().ok()
            && bytes.len() as u64 == metadata.len(),
        "{label} file changed while being read"
    );
    Ok(Some(bytes))
}

fn ensure_run_has_no_secret_like_text(run: &AuditRunV1) -> Result<()> {
    let value = serde_json::to_value(run)?;
    ensure!(
        !contains_secret_like_text(&value),
        "audit checkpoint contains credential-like evidence; redact evidence before checkpointing"
    );
    Ok(())
}

fn contains_secret_like_text(value: &Value) -> bool {
    match value {
        Value::String(text) => redact_sensitive_text(text) != *text,
        Value::Array(items) => items.iter().any(contains_secret_like_text),
        Value::Object(map) => map.values().any(contains_secret_like_text),
        _ => false,
    }
}

fn redact_json_strings(value: &mut Value) {
    match value {
        Value::String(text) => *text = redact_sensitive_text(text),
        Value::Array(items) => items.iter_mut().for_each(redact_json_strings),
        Value::Object(map) => map.values_mut().for_each(redact_json_strings),
        _ => {}
    }
}

fn redact_sensitive_text(input: &str) -> String {
    let mut text = input.to_string();
    for regex in secret_redaction_patterns() {
        text = regex
            .replace_all(&text, |captures: &regex::Captures<'_>| {
                if captures.len() > 1 {
                    format!(
                        "{}=[REDACTED]",
                        captures.get(1).map_or("secret", |m| m.as_str())
                    )
                } else {
                    "[REDACTED]".to_string()
                }
            })
            .into_owned();
    }
    text
}

fn secret_redaction_patterns() -> &'static [regex::Regex] {
    static PATTERNS: std::sync::OnceLock<Vec<regex::Regex>> = std::sync::OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
        r"(?i)-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b",
        r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b",
        r"\bsk-[A-Za-z0-9_-]{16,}\b",
        r"(?i)\bBearer\s+[A-Za-z0-9._~+/-]+=*",
        r#"(?i)\b([A-Z0-9_]*(?:API[_-]?KEY|ACCESS[_-]?KEY|TOKEN|SECRET|PASSWORD|PASSWD|CREDENTIAL)[A-Z0-9_]*)\s*[:=]\s*["']?[^\s,;"']+"#,
        r#"(?i)\b(api[_-]?key|access[_-]?key|token|secret|password|passwd|credential|authorization)\s*[:=]\s*["']?[^\s,;"']+"#,
        ]
        .into_iter()
        .map(|pattern| {
            regex::Regex::new(pattern).expect("static secret-redaction pattern is valid")
        })
        .collect()
    })
}

fn validate_checkpoint_text(pending_work: &[String], next_safe_action: &str) -> Result<()> {
    ensure!(
        pending_work.len() <= MAX_AUDIT_CHECKPOINT_PENDING_WORK,
        "checkpoint pending work exceeds its item limit"
    );
    for item in pending_work {
        ensure!(
            !item.trim().is_empty()
                && item.len() <= MAX_AUDIT_CHECKPOINT_ACTION_BYTES
                && !item.chars().any(char::is_control),
            "checkpoint pending-work item is invalid"
        );
    }
    ensure!(
        !next_safe_action.trim().is_empty()
            && next_safe_action.len() <= MAX_AUDIT_CHECKPOINT_ACTION_BYTES
            && !next_safe_action.chars().any(char::is_control),
        "checkpoint next action is invalid"
    );
    Ok(())
}

fn checkpoint_content_digest(checkpoint: &AuditRecoveryCheckpointV1) -> Result<String> {
    let mut normalized = checkpoint.clone();
    normalized.checkpoint_sha256.clear();
    sha256_json(&normalized)
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256_bytes(&serde_json::to_vec(value)?))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value == value.to_ascii_lowercase()
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn strictly_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn strictly_sorted_unique_by<T, F>(values: &[T], key: F) -> bool
where
    F: Fn(&T) -> &str,
{
    values.windows(2).all(|pair| key(&pair[0]) < key(&pair[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security_audit_workflow::{AuditMode, AuditRunBindingV1, AuditRunV1};
    use tempfile::TempDir;

    fn synthetic_git_project() -> TempDir {
        let temp = tempfile::tempdir().expect("temp project");
        let root = temp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn safe() {}\n").unwrap();
        fs::write(root.join(".gitignore"), ".baron/\n").unwrap();
        let baron = root.join(".baron");
        fs::create_dir_all(&baron).unwrap();
        let project_id = project_id_for_path(root).unwrap();
        fs::write(
            baron.join("project.toml"),
            format!(
                "schema_version = 4\nproject_id = \"{project_id}\"\nidentity_binding = \"{}\"\nproject_slug = \"safety-test\"\nadapters = [\"codex\"]\nactive_adapter = \"codex\"\n\n[automation]\ncontext = true\nplan = true\nharness = true\nproof = true\ntrace = true\n",
                "b".repeat(64)
            ),
        )
        .unwrap();
        git_test(root, &["init", "--quiet"]);
        git_test(
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
        git_test(
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

    fn git_test(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .output()
            .expect("git executable for isolated synthetic fixture");
        assert!(output.status.success(), "synthetic git setup failed");
    }

    fn bound_run(root: &Path, number: &str) -> AuditRunV1 {
        let revision = current_git_revision(root).unwrap();
        let binding = AuditRunBindingV1 {
            project_id: project_id_for_path(root).unwrap(),
            source_revision: revision,
            source_dirty: false,
            dirty_state_sha256: None,
            scope: vec!["src".to_string()],
            languages: vec!["rust".to_string()],
            profile: "backend".to_string(),
            task_id: format!("task-{number}"),
            task_state_sha256: "c".repeat(64),
            operation_id: format!("operation-{number}"),
        };
        AuditRunV1::new(
            format!("00000000-0000-4000-8000-{number}"),
            binding,
            AuditMode::Focused,
        )
        .unwrap()
    }

    #[test]
    fn report_redaction_removes_secret_values_without_reflecting_them() {
        assert_eq!(
            redact_sensitive_text("API_KEY=sk-12345678901234567890"),
            "API_KEY=[REDACTED]"
        );
        assert_eq!(
            redact_sensitive_text("Authorization: Bearer abc.def.ghi"),
            "Authorization=[REDACTED]"
        );
    }

    #[test]
    fn report_publication_redacts_secrets_and_preserves_conflicts() {
        let temp = synthetic_git_project();
        let run = bound_run(temp.path(), "000000000019");
        let mut report = run.report().unwrap();
        report.binding.profile = "API_KEY=sk-12345678901234567890".to_string();
        let redacted = redact_audit_report(&report).unwrap();
        assert!(redacted.to_string().contains("[REDACTED]"));
        assert!(!redacted.to_string().contains("sk-12345678901234567890"));
        let output = temp.path().join("report.json");
        assert!(publish_audit_report(&output, &report).unwrap());
        let saved = fs::read_to_string(&output).unwrap();
        assert!(saved.contains("[REDACTED]"));
        assert!(!saved.contains("sk-12345678901234567890"));
        fs::write(&output, b"user edit").unwrap();
        assert!(publish_audit_report(&output, &report).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"user edit");
    }

    #[test]
    fn changed_source_content_requires_fresh_audit() {
        let temp = synthetic_git_project();
        let run = bound_run(temp.path(), "000000000011");
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            run,
            vec!["complete static review".to_string()],
            "review src/lib.rs",
        )
        .unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
        let assessment = assess_audit_resume(temp.path(), &checkpoint).unwrap();
        assert!(assessment.fresh_run_required);
        assert!(!assessment.can_resume);
        assert_eq!(assessment.changed_paths, vec!["src/lib.rs"]);
    }

    #[test]
    fn current_git_revision_accepts_only_the_worktree_root() {
        let temp = synthetic_git_project();

        let revision = current_git_revision(temp.path()).unwrap();
        assert!(matches!(revision.len(), 40 | 64));

        let nested = temp.path().join("src");
        let error = current_git_revision(&nested).unwrap_err().to_string();
        assert!(
            error.contains("repository path is not the Git worktree root"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn changed_git_revision_requires_fresh_audit() {
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000018"),
            vec!["review findings".to_string()],
            "continue static analysis",
        )
        .unwrap();
        fs::write(temp.path().join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
        git_test(
            temp.path(),
            &[
                "-c",
                "user.name=Safety Test",
                "-c",
                "user.email=safety@example.invalid",
                "add",
                "src/lib.rs",
            ],
        );
        git_test(
            temp.path(),
            &[
                "-c",
                "user.name=Safety Test",
                "-c",
                "user.email=safety@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "changed fixture",
            ],
        );
        let assessment = assess_audit_resume(temp.path(), &checkpoint).unwrap();
        assert!(assessment.fresh_run_required);
        assert!(!assessment.source_revision_matches);
        assert!(assessment.changed_paths.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn checkpoint_integrity_rejects_unknown_schema_and_mutation() {
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000012"),
            vec!["review findings".to_string()],
            "continue static analysis",
        )
        .unwrap();
        let mut bytes = serde_json::to_vec(&checkpoint).unwrap();
        bytes.resize(MAX_AUDIT_CHECKPOINT_JSON_BYTES + 1, b' ');
        assert!(validate_audit_checkpoint_json(&bytes).is_err());
        let mut tampered = checkpoint;
        tampered.schema_version += 1;
        assert!(tampered.validate().is_err());
    }

    #[test]
    fn checkpoint_json_rejects_missing_malformed_and_unknown_schema() {
        assert!(validate_audit_checkpoint_json(&[]).is_err());
        assert!(validate_audit_checkpoint_json(b"{not-json").is_err());
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000017"),
            vec!["review findings".to_string()],
            "continue static analysis",
        )
        .unwrap();
        let mut value = serde_json::to_value(checkpoint).unwrap();
        value["schema_version"] = serde_json::json!(99);
        assert!(validate_audit_checkpoint_json(&serde_json::to_vec(&value).unwrap()).is_err());
        value["schema_version"] = serde_json::json!(AUDIT_RECOVERY_CHECKPOINT_SCHEMA_VERSION);
        value["unexpected"] = serde_json::json!("must be rejected");
        assert!(validate_audit_checkpoint_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn checkpoint_publication_is_create_only_idempotent_and_preserves_edits() {
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000013"),
            vec!["review findings".to_string()],
            "continue static analysis",
        )
        .unwrap();
        let output = temp.path().join("audit-checkpoint.json");
        assert!(publish_audit_checkpoint(&output, &checkpoint).unwrap());
        assert!(!publish_audit_checkpoint(&output, &checkpoint).unwrap());
        fs::write(&output, b"user edit").unwrap();
        assert!(publish_audit_checkpoint(&output, &checkpoint).is_err());
        assert_eq!(fs::read(&output).unwrap(), b"user edit");
    }

    #[test]
    fn checkpoint_file_reader_rejects_oversized_file_before_json_parse() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("oversized-checkpoint.json");
        let file = fs::File::create(&input).unwrap();
        file.set_len(MAX_AUDIT_CHECKPOINT_JSON_BYTES as u64 + 1)
            .unwrap();
        drop(file);

        let error = read_audit_checkpoint_file(&input).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("checkpoint file exceeds the byte limit"),
            "{error:#}"
        );
        assert_eq!(
            fs::metadata(&input).unwrap().len(),
            MAX_AUDIT_CHECKPOINT_JSON_BYTES as u64 + 1
        );
    }

    #[test]
    fn checkpoint_publication_bounds_existing_artifact_comparison() {
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000031"),
            Vec::new(),
            "continue review",
        )
        .unwrap();
        let output = temp.path().join("oversized-checkpoint.json");
        let file = fs::File::create(&output).unwrap();
        file.set_len(MAX_AUDIT_CHECKPOINT_JSON_BYTES as u64 + 1)
            .unwrap();
        drop(file);

        let error = publish_audit_checkpoint(&output, &checkpoint).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("existing audit artifact file exceeds the byte limit"),
            "{error:#}"
        );
        assert_eq!(
            fs::metadata(&output).unwrap().len(),
            MAX_AUDIT_CHECKPOINT_JSON_BYTES as u64 + 1
        );
    }

    #[test]
    fn source_inventory_rejects_symlinks_and_oversized_files() {
        let temp = synthetic_git_project();
        let run = bound_run(temp.path(), "000000000014");
        let link = temp.path().join("src/external");
        #[cfg(unix)]
        let symlink_created =
            std::os::unix::fs::symlink(temp.path().join("src/lib.rs"), &link).is_ok();
        #[cfg(windows)]
        let symlink_created =
            std::os::windows::fs::symlink_file(temp.path().join("src/lib.rs"), &link).is_ok();
        if symlink_created {
            assert!(snapshot_source_scope(temp.path(), &run.binding).is_err());
            fs::remove_file(&link).unwrap();
        }
        fs::write(
            temp.path().join("src/large.rs"),
            vec![b'x'; MAX_AUDIT_CHECKPOINT_FILE_BYTES as usize + 1],
        )
        .unwrap();
        assert!(snapshot_source_scope(temp.path(), &run.binding).is_err());
    }

    #[test]
    fn unchanged_checkpoint_is_a_hint_and_uses_current_identity() {
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000015"),
            vec!["validate source evidence".to_string()],
            "continue at reconnaissance",
        )
        .unwrap();
        let assessment = assess_audit_resume(temp.path(), &checkpoint).unwrap();
        assert!(!assessment.fresh_run_required);
        assert!(assessment.can_resume);
        assert_eq!(assessment.next_safe_action, "continue at reconnaissance");
        assert_eq!(assessment.run_status, AuditRunStatusV1::InProgress);
    }

    #[test]
    fn an_identity_change_invalidates_checkpoint_reuse() {
        let temp = synthetic_git_project();
        let checkpoint = create_audit_checkpoint(
            temp.path(),
            bound_run(temp.path(), "000000000016"),
            vec!["continue".to_string()],
            "review current source",
        )
        .unwrap();
        let config_path = temp.path().join(".baron/project.toml");
        let mut config = fs::read_to_string(&config_path).unwrap();
        config = config.replace(&"b".repeat(64), &"d".repeat(64));
        fs::write(config_path, config).unwrap();
        let assessment = assess_audit_resume(temp.path(), &checkpoint).unwrap();
        assert!(assessment.fresh_run_required);
        assert!(!assessment.project_identity_matches);
        assert!(assessment
            .blockers
            .contains(&"project_identity_changed".to_string()));
    }

    #[test]
    fn checkpoint_creation_rejects_run_with_stale_dirty_state_binding() {
        let temp = synthetic_git_project();
        let root = temp.path();
        let stale_run = bound_run(root, "000000000029");

        fs::write(root.join("src/lib.rs"), "pub fn changed() {}\n").unwrap();
        assert!(create_audit_checkpoint(root, stale_run, Vec::new(), "continue review").is_err());

        let current_dirty_state = current_git_dirty_state(root, &["src".to_string()]).unwrap();
        let mut current_run = bound_run(root, "000000000030");
        current_run.binding.source_dirty = true;
        current_run.binding.dirty_state_sha256 = Some(current_dirty_state);
        let checkpoint =
            create_audit_checkpoint(root, current_run, Vec::new(), "continue review").unwrap();
        assert!(assess_audit_resume(root, &checkpoint).unwrap().can_resume);
    }
}
