use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::authority::classify_request;
use crate::autopilot::pending_approval_warning;
use crate::capability::runtime_backend_report;
use crate::config::{
    find_project_root, load_project_config, resolve_vault_path_for_repo, ProjectPlatform,
};
use crate::context::compile_context_for_operation;
use crate::continuity::continuity_status;
use crate::control_plane::{
    gate_evidence_status_strict, route_task_for_operation, validate_control_plane,
};
use crate::harness::harness_status;
use crate::intent::intent_status;
use crate::operation::{OperationContext, SupportedAdapter};
use crate::plan::plan_status;
use crate::platform::{platform_name, render_platform_context};
use crate::proof::latest_proof;
use crate::risk::RiskLane;
use crate::trace::latest_trace_score;
use crate::vault::ensure_vault;
use crate::work_shape::{decide_work_shape, DurabilityNeed, JudgmentNeed, LifecycleDepth};

pub const PREPARE_SCHEMA_VERSION: u32 = 1;
pub const PREPARE_MAX_INPUT_BYTES: usize = 128 * 1024;
pub const PREPARE_MAX_TASK_CHARS: usize = 96 * 1024;
pub const PREPARE_MAX_OUTPUT_CONTEXT_CHARS: usize = 8_000;
const MAX_IDENTIFIER_CHARS: usize = 256;
const MAX_STATUS_CHARS: usize = 3_600;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PrepareRequestV1 {
    pub schema_version: u32,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrepareErrorCode {
    InvalidInput,
    UnsupportedAdapter,
    ProjectState,
    Internal,
}

impl PrepareErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::UnsupportedAdapter => "unsupported_adapter",
            Self::ProjectState => "project_state",
            Self::Internal => "internal",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::InvalidInput => 2,
            Self::UnsupportedAdapter => 3,
            Self::ProjectState => 4,
            Self::Internal => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareError {
    pub code: PrepareErrorCode,
    pub message: String,
    pub details: BTreeMap<String, String>,
}

impl PrepareError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(PrepareErrorCode::InvalidInput, message)
    }

    pub fn unsupported_adapter(message: impl Into<String>) -> Self {
        Self::new(PrepareErrorCode::UnsupportedAdapter, message)
    }

    pub fn project_state(message: impl Into<String>) -> Self {
        Self::new(PrepareErrorCode::ProjectState, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(PrepareErrorCode::Internal, message)
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    pub fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }

    pub fn envelope(&self) -> PrepareErrorEnvelope {
        PrepareErrorEnvelope {
            schema_version: PREPARE_SCHEMA_VERSION,
            ok: false,
            error: PrepareErrorBody {
                error_code: self.code.as_str().to_string(),
                message: self.message.clone(),
                details: self.details.clone(),
            },
        }
    }

    fn new(code: PrepareErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: BTreeMap::new(),
        }
    }
}

impl fmt::Display for PrepareError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for PrepareError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrepareErrorEnvelope {
    pub schema_version: u32,
    pub ok: bool,
    pub error: PrepareErrorBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrepareErrorBody {
    pub error_code: String,
    pub message: String,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparePacketV1 {
    pub schema_version: u32,
    pub ok: bool,
    pub project_id: String,
    pub adapter: String,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub task: PrepareTask,
    pub authority: PrepareAuthority,
    pub intent: PrepareIntent,
    pub work_shape: PrepareWorkShape,
    pub risk: RiskLane,
    pub profile: PrepareProfile,
    pub route: PrepareRoute,
    pub context: PrepareContext,
    pub continuity: PrepareContinuity,
    pub verification: PrepareVerification,
    pub blockers: Vec<PrepareIssue>,
    pub warnings: Vec<PrepareIssue>,
    pub unknowns: Vec<String>,
    pub next_action: PrepareNextAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareTask {
    pub id: String,
    pub summary: String,
    pub text: String,
    pub resumed: bool,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareAuthority {
    pub classification: String,
    pub mutation_allowed: bool,
    pub reason: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareIntent {
    pub available: bool,
    pub id: Option<String>,
    pub title: Option<String>,
    pub confirmed: bool,
    pub current_behavior: Option<String>,
    pub target_behavior: Option<String>,
    pub scope: Option<String>,
    pub non_goals: Vec<String>,
    pub constraints: Vec<String>,
    pub decisions: Vec<String>,
    pub required_proof: Option<String>,
    pub unknowns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareWorkShape {
    pub durability: String,
    pub judgment: String,
    pub lifecycle: String,
    pub proof_required: bool,
    pub work_shape: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareProfile {
    pub platform: String,
    pub extensions: Vec<String>,
    pub context: String,
    pub unknown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareRoute {
    pub explanation: String,
    pub selected_skills: Vec<PrepareRouteItem>,
    pub mandatory_agents: Vec<PrepareAgent>,
    pub optional_agents: Vec<PrepareRouteItem>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareRouteItem {
    pub name: String,
    pub reason: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareAgent {
    pub name: String,
    pub required: bool,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareContext {
    pub target: String,
    pub text: String,
    pub truncated: bool,
    pub max_chars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareContinuity {
    pub available: bool,
    pub resumed: bool,
    pub plan_status: Option<String>,
    pub plan_next_action: Option<String>,
    pub recovery_outcome: Option<String>,
    pub safe_next_action: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareVerification {
    pub proof_required: bool,
    pub required_trace_tier: String,
    pub mandatory_gates: Vec<String>,
    pub gate_evidence_passed: bool,
    pub missing_gate_evidence: Vec<String>,
    pub latest_proof: Option<PrepareProof>,
    pub trace: Option<PrepareTrace>,
    pub runtime_passed: bool,
    pub runtime_blockers: Vec<String>,
    pub runtime_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareProof {
    pub id: String,
    pub summary: String,
    pub capability_gate_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareTrace {
    pub achieved: String,
    pub required: String,
    pub passed: bool,
    pub missing_fields: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareIssue {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrepareNextAction {
    pub action: String,
    pub requires_confirmation: bool,
}

pub fn decode_request(bytes: &[u8]) -> Result<PrepareRequestV1, PrepareError> {
    if bytes.len() > PREPARE_MAX_INPUT_BYTES {
        return Err(PrepareError::invalid_input(format!(
            "prepare input exceeds the {} byte limit",
            PREPARE_MAX_INPUT_BYTES
        ))
        .with_detail("max_bytes", PREPARE_MAX_INPUT_BYTES.to_string()));
    }
    let mut request: PrepareRequestV1 = serde_json::from_slice(bytes).map_err(|error| {
        PrepareError::invalid_input("prepare input is not valid JSON")
            .with_detail("parser", error.to_string())
    })?;
    validate_request(&mut request)?;
    Ok(request)
}

pub fn prepare(
    request: PrepareRequestV1,
    adapter_input: &str,
    repo_start: impl AsRef<Path>,
    vault_override: Option<PathBuf>,
) -> Result<PreparePacketV1, PrepareError> {
    let mut request = request;
    validate_request(&mut request)?;
    let adapter = SupportedAdapter::parse(adapter_input)
        .map_err(|error| PrepareError::unsupported_adapter(error.to_string()))?;
    let operation = OperationContext {
        adapter,
        session_id: request.session_id.clone(),
        request_id: request.request_id.clone(),
    };
    let repo_root = find_project_root(repo_start).map_err(project_error)?;
    let config = load_project_config(&repo_root).map_err(project_error)?;
    if config.project_id.trim().is_empty() {
        return Err(PrepareError::project_state(
            "Baron project state does not contain a project identity",
        ));
    }
    let vault_path =
        resolve_vault_path_for_repo(vault_override, &repo_root).map_err(project_error)?;
    let vault = ensure_vault(&vault_path, &repo_root).map_err(project_error)?;

    let authority = classify_request(&request.task);
    let work_shape = decide_work_shape(&repo_root, &request.task).map_err(internal_error)?;
    let control_plane = validate_control_plane(&repo_root).map_err(project_error)?;
    let route = route_task_for_operation(&repo_root, &request.task, work_shape.risk, &operation)
        .map_err(project_error)?;

    let intent_source = intent_status(&repo_root).map_err(project_error)?;
    let plan_source = plan_status(&repo_root).map_err(project_error)?;
    let harness_source = harness_status(&repo_root).map_err(project_error)?;
    let continuity_source = continuity_status(&repo_root, &vault).map_err(project_error)?;
    let intent = project_intent(&intent_source);
    let continuity = project_continuity(&plan_source, &continuity_source);
    let profile_context = render_platform_context(&repo_root, Some(&request.task));
    let profile = project_profile(&config, profile_context);
    let context_text =
        compile_context_for_operation(&repo_root, &vault_path, &operation, Some(&request.task))
            .map_err(project_error)?;
    let (context_text, context_truncated) =
        bounded(&context_text, PREPARE_MAX_OUTPUT_CONTEXT_CHARS);

    let gate_status =
        gate_evidence_status_strict(&repo_root, &route.mandatory_agents).map_err(project_error)?;
    let proof = latest_proof(&repo_root).map_err(project_error)?;
    let trace = latest_trace_score(&repo_root).map_err(project_error)?;
    let runtime =
        runtime_backend_report(&repo_root, operation.adapter_kind()).map_err(project_error)?;

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut unknowns = Vec::new();
    if !control_plane.passed {
        blockers.push(PrepareIssue {
            code: "control_plane_invalid".to_string(),
            message: control_plane.diagnostics.join("; "),
        });
    }
    if matches!(work_shape.judgment, JudgmentNeed::UserConfirmation) && !intent.confirmed {
        blockers.push(PrepareIssue {
            code: "intent_confirmation_required".to_string(),
            message: "The current work shape requires confirmed intent before mutation."
                .to_string(),
        });
    }
    if continuity.available && !continuity.resumed && continuity.safe_next_action.is_none() {
        blockers.push(PrepareIssue {
            code: "continuity_unresolved".to_string(),
            message: "Continuity state exists but does not expose a safe next action.".to_string(),
        });
    }
    if !runtime.blocking_gaps.is_empty() {
        blockers.push(PrepareIssue {
            code: "runtime_requirements_unmet".to_string(),
            message: runtime.blocking_gaps.join("; "),
        });
    }
    if !gate_status.passed && work_shape.proof_required {
        warnings.push(PrepareIssue {
            code: "gate_evidence_missing".to_string(),
            message: format!(
                "Required gate evidence is not yet recorded for: {}",
                gate_status.missing_agents.join(", ")
            ),
        });
    }
    if !intent.available {
        unknowns.push("no current intent brief is recorded".to_string());
    } else if !intent.confirmed {
        warnings.push(PrepareIssue {
            code: "intent_unconfirmed".to_string(),
            message: "The current intent brief is present but not confirmed.".to_string(),
        });
    }
    if profile.unknown {
        unknowns.push("no configured platform profile is available".to_string());
    }
    if runtime.warnings.is_empty() {
        // Keep the packet shape stable without manufacturing a warning.
    } else {
        warnings.extend(runtime.warnings.iter().take(8).map(|warning| PrepareIssue {
            code: "runtime_warning".to_string(),
            message: warning.clone(),
        }));
    }
    if !harness_source.contains("- Title: ") {
        unknowns.push("no current Product Harness story is recorded".to_string());
    }
    if let Some(message) = pending_approval_warning(&repo_root, &vault) {
        warnings.push(PrepareIssue {
            code: "autopilot_approval_pending".to_string(),
            message,
        });
    }

    let task_text = bounded(&request.task, PREPARE_MAX_OUTPUT_CONTEXT_CHARS);
    let task_id = task_id_for_request(&config.project_id, &request);
    let selected_skills = route
        .selected_skills
        .iter()
        .map(|item| PrepareRouteItem {
            name: item.name.clone(),
            reason: item.reason.clone(),
            path: format!(".baron/core/skills/{}", item.name),
        })
        .collect::<Vec<_>>();
    let mandatory_agents = route
        .mandatory_agents
        .iter()
        .map(|name| PrepareAgent {
            name: name.clone(),
            required: true,
            path: format!(".baron/core/agents/{name}.toml"),
        })
        .collect::<Vec<_>>();
    let optional_agents = route
        .optional_agents
        .iter()
        .map(|item| PrepareRouteItem {
            name: item.name.clone(),
            reason: item.reason.clone(),
            path: format!(".baron/core/agents/{}.toml", item.name),
        })
        .collect::<Vec<_>>();

    Ok(PreparePacketV1 {
        schema_version: PREPARE_SCHEMA_VERSION,
        ok: true,
        project_id: config.project_id,
        adapter: adapter.as_str().to_string(),
        session_id: request.session_id,
        request_id: request.request_id,
        task: PrepareTask {
            id: task_id,
            summary: task_summary(&request.task),
            text: task_text.0,
            resumed: continuity.resumed,
            truncated: task_text.1,
        },
        authority: PrepareAuthority {
            classification: authority.authority.as_str().to_string(),
            mutation_allowed: authority.mutation_allowed,
            reason: authority.reason,
            next_action: authority.next_action,
        },
        intent,
        work_shape: project_work_shape(&work_shape),
        risk: work_shape.risk,
        profile,
        route: PrepareRoute {
            explanation: route.explanation,
            selected_skills,
            mandatory_agents,
            optional_agents,
            skipped: route.skipped,
        },
        context: PrepareContext {
            target: adapter.as_str().to_string(),
            text: context_text,
            truncated: context_truncated,
            max_chars: PREPARE_MAX_OUTPUT_CONTEXT_CHARS,
        },
        continuity,
        verification: PrepareVerification {
            proof_required: work_shape.proof_required,
            required_trace_tier: work_shape.risk.required_trace_tier().to_string(),
            mandatory_gates: route.mandatory_agents,
            gate_evidence_passed: gate_status.passed,
            missing_gate_evidence: gate_status.missing_agents,
            latest_proof: proof.map(|record| PrepareProof {
                id: record.id,
                summary: bounded(&record.summary, 1_000).0,
                capability_gate_passed: record.capability_gate_passed,
            }),
            trace: trace.map(|score| PrepareTrace {
                achieved: score.achieved.as_str().to_string(),
                required: score.required.as_str().to_string(),
                passed: score.passed,
                missing_fields: score.missing_fields.into_iter().take(16).collect(),
                warnings: score.warnings.into_iter().take(16).collect(),
            }),
            runtime_passed: runtime.passed,
            runtime_blockers: runtime.blocking_gaps.into_iter().take(16).collect(),
            runtime_warnings: runtime.warnings.into_iter().take(16).collect(),
        },
        blockers,
        warnings,
        unknowns,
        next_action: PrepareNextAction {
            action: work_shape.next_action,
            requires_confirmation: matches!(work_shape.judgment, JudgmentNeed::UserConfirmation),
        },
    })
}

fn validate_request(request: &mut PrepareRequestV1) -> Result<(), PrepareError> {
    if request.schema_version != PREPARE_SCHEMA_VERSION {
        return Err(PrepareError::invalid_input(format!(
            "unsupported prepare schema version {}; expected {}",
            request.schema_version, PREPARE_SCHEMA_VERSION
        ))
        .with_detail(
            "expected_schema_version",
            PREPARE_SCHEMA_VERSION.to_string(),
        ));
    }
    if request.task.trim().is_empty() {
        return Err(PrepareError::invalid_input("task must not be empty"));
    }
    if request.task.chars().count() > PREPARE_MAX_TASK_CHARS {
        return Err(PrepareError::invalid_input(format!(
            "task exceeds the {} character limit",
            PREPARE_MAX_TASK_CHARS
        ))
        .with_detail("max_task_chars", PREPARE_MAX_TASK_CHARS.to_string()));
    }
    normalize_identifier("session_id", &mut request.session_id)?;
    normalize_identifier("request_id", &mut request.request_id)?;
    Ok(())
}

fn normalize_identifier(name: &str, value: &mut Option<String>) -> Result<(), PrepareError> {
    let Some(current) = value.as_ref() else {
        return Ok(());
    };
    let trimmed = current.trim().to_string();
    if trimmed.is_empty() {
        *value = None;
    } else if trimmed.chars().count() > MAX_IDENTIFIER_CHARS {
        return Err(PrepareError::invalid_input(format!(
            "{name} exceeds the {MAX_IDENTIFIER_CHARS} character limit"
        )));
    } else {
        *value = Some(trimmed);
    }
    Ok(())
}

fn project_error(error: impl fmt::Display) -> PrepareError {
    PrepareError::project_state(error.to_string())
}

fn internal_error(error: impl fmt::Display) -> PrepareError {
    PrepareError::internal(error.to_string())
}

/// Computes the stable task identity used by both the control-plane prepare
/// command and native hook preparation. Keeping this helper public prevents
/// the hook bridge from inventing a second task identity algorithm.
pub fn task_id_for_request(project_id: &str, request: &PrepareRequestV1) -> String {
    let mut digest = Sha256::new();
    digest.update(project_id.as_bytes());
    digest.update([0]);
    digest.update(request.task.as_bytes());
    digest.update([0]);
    if let Some(session_id) = &request.session_id {
        digest.update(session_id.as_bytes());
    }
    digest.update([0]);
    if let Some(request_id) = &request.request_id {
        digest.update(request_id.as_bytes());
    }
    let digest = digest.finalize();
    let prefix = digest
        .iter()
        .take(10)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("task-{prefix}")
}

fn task_summary(task: &str) -> String {
    let first_line = task.lines().next().unwrap_or(task).trim();
    bounded(first_line, 240).0
}

fn bounded(value: &str, limit: usize) -> (String, bool) {
    let output = value.chars().take(limit).collect::<String>();
    (output, value.chars().count() > limit)
}

fn project_work_shape(decision: &crate::work_shape::WorkShapeDecision) -> PrepareWorkShape {
    PrepareWorkShape {
        durability: durability_name(decision.durability).to_string(),
        judgment: judgment_name(decision.judgment).to_string(),
        lifecycle: lifecycle_name(decision.lifecycle).to_string(),
        proof_required: decision.proof_required,
        work_shape: decision.work_shape.as_str().to_string(),
        reasons: decision.reasons.clone(),
    }
}

fn durability_name(value: DurabilityNeed) -> &'static str {
    match value {
        DurabilityNeed::None => "none",
        DurabilityNeed::Ephemeral => "ephemeral",
        DurabilityNeed::Durable => "durable",
    }
}

fn judgment_name(value: JudgmentNeed) -> &'static str {
    match value {
        JudgmentNeed::None => "none",
        JudgmentNeed::UserConfirmation => "user_confirmation",
    }
}

fn lifecycle_name(value: LifecycleDepth) -> &'static str {
    match value {
        LifecycleDepth::ReadOnly => "read_only",
        LifecycleDepth::Focused => "focused",
        LifecycleDepth::Full => "full",
    }
}

fn project_profile(config: &crate::config::ProjectConfig, context: String) -> PrepareProfile {
    let platform = config.platform.map(platform_name).unwrap_or("unknown");
    PrepareProfile {
        platform: platform.to_string(),
        extensions: config
            .platform_extensions
            .iter()
            .map(|platform| platform_name(*platform).to_string())
            .collect(),
        context: bounded(&context, 3_200).0,
        unknown: config.platform.is_none()
            || matches!(config.platform, Some(ProjectPlatform::Unknown)),
    }
}

fn project_intent(source: &str) -> PrepareIntent {
    let available = source.contains("# Baron Intent Brief");
    PrepareIntent {
        available,
        id: bullet_value(source, "- ID: "),
        title: bullet_value(source, "- Title: "),
        confirmed: bullet_value(source, "- Confirmation: ")
            .map(|value| value == "confirmed")
            .unwrap_or(false),
        current_behavior: section_value(source, "## Current Behavior"),
        target_behavior: section_value(source, "## Target Behavior"),
        scope: section_value(source, "## Scope"),
        non_goals: section_list(source, "## Non-Goals"),
        constraints: section_list(source, "## Constraints"),
        decisions: section_list(source, "## Decisions"),
        required_proof: section_value(source, "## Required Proof"),
        unknowns: section_list(source, "## Remaining Unknowns"),
    }
}

fn project_continuity(plan_source: &str, continuity_source: &str) -> PrepareContinuity {
    let plan_status = bullet_value(plan_source, "- Status: ");
    let plan_next_action = bullet_value(plan_source, "- Next action: ");
    let recovery_outcome = bullet_value(continuity_source, "- Outcome: ");
    let safe_next_action = section_value(continuity_source, "## Safe Next Action");
    let plan_interrupted = matches!(plan_status.as_deref(), Some("interrupted" | "in_progress"));
    let recovery_interrupted = matches!(
        recovery_outcome.as_deref(),
        Some("failed" | "blocked" | "interrupted")
    );
    let current_task_resume =
        continuity_source.contains("- Current task: ") && safe_next_action.is_some();
    let resumed = (plan_interrupted || recovery_interrupted || current_task_resume)
        && safe_next_action.is_some();
    let available = plan_status.is_some()
        || recovery_outcome.is_some()
        || continuity_source.contains("- Current task: ");
    PrepareContinuity {
        available,
        resumed,
        plan_status,
        plan_next_action,
        recovery_outcome,
        safe_next_action,
        summary: bounded(continuity_source, MAX_STATUS_CHARS).0,
    }
}

fn bullet_value(source: &str, prefix: &str) -> Option<String> {
    source.lines().find_map(|line| {
        line.strip_prefix(prefix)
            .map(|value| value.trim().trim_matches('`').trim().to_string())
    })
}

fn section_value(source: &str, heading: &str) -> Option<String> {
    let mut values = Vec::new();
    let mut active = false;
    for line in source.lines() {
        if line.trim() == heading {
            active = true;
            continue;
        }
        if active && line.starts_with("## ") {
            break;
        }
        if active && !line.trim().is_empty() {
            values.push(line.trim().trim_start_matches("- ").to_string());
        }
    }
    if values.is_empty() {
        None
    } else {
        Some(bounded(&values.join("\n"), 1_600).0)
    }
}

fn section_list(source: &str, heading: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut active = false;
    for line in source.lines() {
        if line.trim() == heading {
            active = true;
            continue;
        }
        if active && line.starts_with("## ") {
            break;
        }
        if active {
            if let Some(value) = line.trim().strip_prefix("- ") {
                values.push(bounded(value.trim(), 600).0);
            }
        }
    }
    values.into_iter().take(16).collect()
}
