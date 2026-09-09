use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::Deserialize;

use crate::capability::load_capability_state;
use crate::config::{load_project_config, AdapterKind, ProjectPlatform};
use crate::execution_receipt::{
    load_receipts, receipt_is_current, receipt_matches_context, ReceiptContext,
};
use crate::operation::OperationContext;
use crate::platform::platform_name;
use crate::risk::RiskLane;
use crate::safe_io::replace_text;
use crate::survey::survey_repository;
use crate::vault::VaultContext;
use crate::work_shape::decide_work_shape;

const CORE_AGENTS: [&str; 3] = ["code-reviewer", "security-auditor", "test-engineer"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneReport {
    pub passed: bool,
    pub workflow_owner: Option<String>,
    pub mandatory_agents: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteReport {
    pub selected_skills: Vec<RoutingDecision>,
    pub mandatory_agents: Vec<String>,
    pub optional_agents: Vec<RoutingDecision>,
    pub skipped: Vec<String>,
    pub explanation: String,
    pub verification: Vec<RoutingDecision>,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEvidence {
    pub agent: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEvidenceStatus {
    pub passed: bool,
    pub missing_agents: Vec<String>,
}

pub type GateReceiptBinding = ReceiptContext;

#[derive(Debug, Clone)]
struct SkillContract {
    name: String,
    description: String,
    body: String,
    metadata: SkillMetadata,
}

#[derive(Debug, Clone, Default)]
struct SkillMetadata {
    triggers: Vec<String>,
    exclusions: Vec<String>,
    profile_affinities: Vec<String>,
    dependencies: Vec<String>,
    conflicts: Vec<String>,
    evidence: Vec<String>,
    verification: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentContract {
    name: String,
    description: Option<String>,
    developer_instructions: Option<String>,
}

pub fn validate_control_plane(repo_root: impl AsRef<Path>) -> Result<ControlPlaneReport> {
    let repo_root = repo_root.as_ref();
    let skills = load_skills(repo_root)?;
    let agents = load_agents(repo_root)?;
    let mut diagnostics = Vec::new();

    let mut workflow_owners = skills
        .iter()
        .filter(|skill| owns_workflow(skill))
        .map(|skill| skill.name.clone())
        .collect::<Vec<_>>();
    workflow_owners.sort();
    workflow_owners.dedup();
    if !workflow_owners.iter().any(|name| name == "superpowers") {
        diagnostics.push("missing Superpowers workflow ownership".to_string());
    }
    if workflow_owners.len() > 1 {
        diagnostics.push(format!(
            "duplicate workflow ownership: {}",
            workflow_owners.join(", ")
        ));
    }

    let agent_names = agents
        .iter()
        .map(|agent| agent.name.as_str())
        .collect::<Vec<_>>();
    for required in CORE_AGENTS {
        if !agent_names.contains(&required) {
            diagnostics.push(format!("missing mandatory quality gate: {required}"));
        }
    }
    for agent in &agents {
        let instructions = agent
            .developer_instructions
            .as_deref()
            .unwrap_or_default()
            .to_lowercase();
        if instructions.contains("invoke other subagents")
            && !instructions.contains("do not invoke other subagents")
        {
            diagnostics.push(format!(
                "recursive subagent orchestration is forbidden: {}",
                agent.name
            ));
        }
        if agent
            .description
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
        {
            diagnostics.push(format!(
                "agent has weak contract description: {}",
                agent.name
            ));
        }
    }

    Ok(ControlPlaneReport {
        passed: diagnostics.is_empty(),
        workflow_owner: workflow_owners.first().cloned(),
        mandatory_agents: CORE_AGENTS.iter().map(|value| value.to_string()).collect(),
        diagnostics,
    })
}

pub fn route_task(repo_root: impl AsRef<Path>, task: &str, risk: RiskLane) -> Result<RouteReport> {
    route_task_internal(repo_root, task, risk, None)
}

/// Route one explicitly identified operation without consulting the project's
/// serialized `active_adapter` convenience field.
pub fn route_task_for_operation(
    repo_root: impl AsRef<Path>,
    task: &str,
    risk: RiskLane,
    operation: &OperationContext,
) -> Result<RouteReport> {
    route_task_internal(repo_root, task, risk, Some(operation.adapter_kind()))
}

fn route_task_internal(
    repo_root: impl AsRef<Path>,
    task: &str,
    risk: RiskLane,
    operation_adapter: Option<AdapterKind>,
) -> Result<RouteReport> {
    let repo_root = repo_root.as_ref();
    let report = validate_control_plane(repo_root)?;
    let normalized_task = normalize_task(task);
    let context = collect_route_context(repo_root, &normalized_task, risk, operation_adapter);
    let contracts = load_skills(repo_root)?;

    let reverse_analysis = contains_any(
        &normalized_task,
        &[
            "binary reverse",
            "reverse engineering",
            "disassembly",
            "decompile",
            "apk reverse",
            "apk analysis",
            "apk manifest",
            "static apk",
            "android binary",
            "malware triage",
            "malware sample",
            "firmware analysis",
        ],
    );
    let security = risk == RiskLane::High
        || contains_any(
            &normalized_task,
            &[
                "auth",
                "login",
                "permission",
                "tenant",
                "rls",
                "security",
                "secret",
                "jwt",
                "cors",
                "upload",
                "payment",
                "crypto",
                "untrusted input",
                "sensitive data",
                "injection",
            ],
        );
    let frontend = contains_any(
        &normalized_task,
        &[
            "frontend",
            "ui",
            "layout",
            "responsive",
            "component",
            "homepage",
            "dashboard",
            "browser",
        ],
    );
    let api = contains_any(
        &normalized_task,
        &[
            "api",
            "endpoint",
            "rest",
            "graphql",
            "interface",
            "contract",
            "request",
            "response",
            "sdk",
            "versioning",
            "compatibility",
            "public contract",
        ],
    );
    let observability = contains_any(
        &normalized_task,
        &[
            "observability",
            "instrumentation",
            "log",
            "logging",
            "metric",
            "metrics",
            "tracing",
            "alert",
            "slo",
            "monitor",
            "audit event",
            "dashboard telemetry",
        ],
    );
    let performance = contains_any(
        &normalized_task,
        &[
            "performance",
            "latency",
            "slow",
            "speed",
            "throughput",
            "cache",
            "caching",
            "bundle",
            "core web vitals",
            "lcp",
            "inp",
            "cls",
            "lighthouse",
            "pagespeed",
            "rendering",
            "loading",
        ],
    );
    let migration = contains_any(
        &normalized_task,
        &[
            "migration",
            "migrate",
            "deprecate",
            "deprecated",
            "legacy",
            "backward",
            "compatibility",
            "breaking change",
            "schema migration",
            "backfill",
        ],
    );
    let database = database_evidence(&normalized_task, &context.repository_signals);
    let data = data_evidence(&normalized_task, &context.repository_signals);
    let mobile = mobile_evidence(&normalized_task, &context.repository_signals);
    let database_profile_relevant =
        profile_relevant("database-engineering", &normalized_task, &context);
    let mobile_profile_relevant =
        profile_relevant("mobile-application-engineering", &normalized_task, &context);
    let database_route = database || database_profile_relevant;
    let mobile_application = (mobile || mobile_profile_relevant) && !reverse_analysis;

    let mut selected_skills = vec![RoutingDecision {
        name: "superpowers".to_string(),
        reason: "selected; workflow-core; Baron planning, TDD, review, proof, and trace authority"
            .to_string(),
    }];
    let mut skipped = Vec::new();
    let mut selected_names = vec!["superpowers".to_string()];

    let mut candidates = vec![
        CandidateMatch::new("frontend-design", frontend, false),
        CandidateMatch::new("vibe-security-scan", security, risk == RiskLane::High),
        CandidateMatch::new(
            "apk-mobile-analysis",
            reverse_analysis && contains_any(&normalized_task, &["apk", "android binary"]),
            reverse_analysis,
        ),
        CandidateMatch::new(
            "malware-triage",
            reverse_analysis && contains_any(&normalized_task, &["malware", "sample", "firmware"]),
            reverse_analysis,
        ),
        CandidateMatch::new(
            "binary-reverse-analysis",
            reverse_analysis,
            reverse_analysis,
        ),
        CandidateMatch::new("api-and-interface-design", api, false),
        CandidateMatch::new("observability-and-instrumentation", observability, false),
        CandidateMatch::new("performance-optimization", performance, false),
        CandidateMatch::new("deprecation-and-migration", migration, migration),
        CandidateMatch::new(
            "database-engineering",
            database_route,
            context.profile == Some(ProjectPlatform::Database),
        ),
        CandidateMatch::new(
            "mobile-application-engineering",
            mobile_application,
            context.profile == Some(ProjectPlatform::Mobile),
        ),
    ];
    for candidate in &mut candidates {
        let metadata = metadata_for(&candidate.name, &contracts);
        candidate.profile_match = profile_matches(&metadata, &context.profiles);
        candidate.profile_relevant = profile_relevant(&candidate.name, &normalized_task, &context);
        candidate.repo_match = metadata.triggers.iter().any(|trigger| {
            context
                .repository_signals
                .iter()
                .any(|signal| contains_term(signal, trigger))
        }) || metadata.triggers.iter().any(|trigger| {
            context
                .affected_paths
                .iter()
                .any(|path| contains_term(path, trigger) || path.contains(trigger))
        });
        candidate.task_match = metadata
            .triggers
            .iter()
            .any(|trigger| contains_term(&normalized_task, trigger));
        if is_reverse_candidate(&candidate.name) && !reverse_analysis {
            candidate.task_match = false;
            candidate.repo_match = false;
        }
        candidate.excluded = metadata
            .exclusions
            .iter()
            .any(|exclusion| contains_term(&normalized_task, exclusion));
        candidate.metadata = metadata;
        candidate.score = candidate.score();
    }

    if reverse_analysis {
        for candidate in &mut candidates {
            if candidate.name == "mobile-application-engineering"
                && candidate.task_match
                && candidate
                    .metadata
                    .conflicts
                    .iter()
                    .any(|conflict| conflict == "apk-mobile-analysis")
            {
                candidate.excluded = true;
                candidate.conflict_suppressed = true;
            }
        }
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.order.cmp(&right.order))
    });

    const MAX_SKILLS: usize = 5;
    for candidate in candidates {
        if candidate.name == "superpowers" {
            continue;
        }
        let matched = candidate.task_match
            || candidate.repo_match
            || (candidate.profile_match && candidate.profile_relevant);
        if candidate.excluded {
            if candidate.conflict_suppressed {
                skipped.push(format!(
                    "{} conflict-suppressed; reverse-analysis evidence excludes normal mobile application guidance",
                    candidate.name
                ));
            } else {
                skipped.push(format!(
                    "{} excluded; evidence matches an explicit exclusion",
                    candidate.name
                ));
            }
            continue;
        }
        if let Some(conflict) = candidate
            .metadata
            .conflicts
            .iter()
            .find(|conflict| selected_names.iter().any(|selected| selected == *conflict))
        {
            skipped.push(format!(
                "{} conflict-suppressed; selected skill `{}` takes precedence",
                candidate.name, conflict
            ));
            continue;
        }
        if !matched {
            let reason = if candidate.profile_match && !candidate.profile_relevant {
                "profile-influenced but evidence-missing for this task"
            } else {
                "evidence-missing for task, repository, and configured profile"
            };
            let evidence = candidate
                .metadata
                .evidence
                .first()
                .map(|item| format!("; required evidence: {item}"))
                .unwrap_or_default();
            skipped.push(format!(
                "{} skipped; {}{}",
                candidate.name, reason, evidence
            ));
            continue;
        }
        if selected_skills.len() >= MAX_SKILLS {
            skipped.push(format!(
                "{} skipped; bounded-selection limit of {} skills reached",
                candidate.name, MAX_SKILLS
            ));
            continue;
        }
        let mut classifications = vec!["selected".to_string()];
        if candidate.task_match {
            classifications.push("task-evidence".to_string());
        }
        if candidate.repo_match {
            classifications.push("repo-evidence".to_string());
        }
        if candidate.profile_match {
            classifications.push("profile-influenced".to_string());
        }
        if candidate.risk_required {
            classifications.push("risk-required".to_string());
        }
        if !candidate.metadata.dependencies.is_empty() {
            classifications.push("dependency-selected".to_string());
        }
        let domain_reason = candidate_reason(&candidate.name, reverse_analysis);
        selected_names.push(candidate.name.clone());
        selected_skills.push(RoutingDecision {
            name: candidate.name,
            reason: format!("{}; {}", classifications.join(", "), domain_reason),
        });
    }

    if selected_skills.len() > 1 {
        selected_skills[0]
            .reason
            .push_str("; dependency-selected for routed domains");
    }

    let durable_work = matches!(
        context.work_shape.as_deref(),
        Some("durable" | "requires_confirmation")
    );
    let mandatory_agents = if security || reverse_analysis || risk == RiskLane::High {
        CORE_AGENTS.iter().map(|value| value.to_string()).collect()
    } else if risk == RiskLane::Medium || selected_skills.len() > 1 || durable_work {
        ["code-reviewer", "test-engineer"]
            .iter()
            .map(|value| value.to_string())
            .collect()
    } else {
        vec!["test-engineer".to_string()]
    };
    let mut optional_agents = Vec::new();
    if frontend && performance {
        optional_agents.push(RoutingDecision {
            name: "web-performance-auditor".to_string(),
            reason: "selected; task-evidence; web performance evidence requires measured Core Web Vitals review".to_string(),
        });
    } else {
        skipped.push(
            "web-performance-auditor skipped; evidence-missing for a combined web and performance task"
                .to_string(),
        );
    }
    if !mandatory_agents
        .iter()
        .any(|agent| agent == "security-auditor")
    {
        skipped.push(
            "security-auditor not mandatory; skipped; evidence-missing for security and risk authority".to_string(),
        );
    }

    let verification = verification_for(
        &context,
        &contracts,
        &selected_names,
        &normalized_task,
        &RouteFlags {
            database: database_route,
            data,
            mobile: mobile_application,
            migration,
            security,
            risk,
        },
    );
    let profile = context
        .profile
        .map(|value| platform_name(value).to_string());
    let selected_summary = selected_names.join(", ");
    let verification_summary = verification
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let work_shape = context.work_shape.as_deref().unwrap_or("unknown");
    let state_summary = if context.state_signals.is_empty() {
        "unknown"
    } else {
        "evidence"
    };
    let capability_summary = if context.capabilities_available {
        "available"
    } else {
        "unknown"
    };
    let affected_summary = context
        .affected_paths
        .first()
        .map(|path| bounded_text(path, 120))
        .unwrap_or_else(|| "none".to_string());
    let mut explanation = if !report.passed {
        format!(
            "control plane has diagnostics; routing is advisory until fixed: {}",
            report.diagnostics.join("; ")
        )
    } else if database_route && data {
        "database and data task: persistence and pipeline evidence require separate bounded engineering and verification guidance; high-risk work keeps all three core quality gates".to_string()
    } else if security {
        "security-sensitive task: risk evidence requires defensive review and all three core quality gates".to_string()
    } else if database_route {
        "database task: relational/persistence evidence and profile signals select bounded database engineering guidance".to_string()
    } else if data {
        "data task: pipeline/lineage evidence selects data-oriented verification without assuming database ownership".to_string()
    } else if mobile_application {
        "mobile application task: lifecycle/device evidence selects normal mobile engineering guidance".to_string()
    } else if frontend {
        "frontend task: load frontend skill and require code review plus verification gates"
            .to_string()
    } else {
        "general task: keep Superpowers as workflow core and use proportional quality gates"
            .to_string()
    };
    explanation.push_str(&format!(
        " profile={}; work_shape={}; state={}; capabilities={}; affected={}; selected={}; verification={}",
        profile.as_deref().unwrap_or("unknown"),
        work_shape,
        state_summary,
        capability_summary,
        affected_summary,
        selected_summary,
        if verification_summary.is_empty() {
            "focused-tests"
        } else {
            verification_summary.as_str()
        }
    ));
    explanation = bounded_text(&explanation, 1_200);

    Ok(RouteReport {
        selected_skills,
        mandatory_agents,
        optional_agents,
        skipped: skipped.into_iter().take(32).collect(),
        explanation,
        verification,
        profile,
    })
}

#[derive(Debug, Clone)]
struct RouteContext {
    profile: Option<ProjectPlatform>,
    profiles: Vec<String>,
    repository_signals: Vec<String>,
    work_shape: Option<String>,
    state_signals: Vec<String>,
    capabilities_available: bool,
    affected_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct RouteFlags {
    database: bool,
    data: bool,
    mobile: bool,
    migration: bool,
    security: bool,
    risk: RiskLane,
}

#[derive(Debug, Clone)]
struct CandidateMatch {
    name: String,
    direct_signal: bool,
    risk_required: bool,
    order: usize,
    score: i32,
    task_match: bool,
    repo_match: bool,
    profile_match: bool,
    profile_relevant: bool,
    excluded: bool,
    conflict_suppressed: bool,
    metadata: SkillMetadata,
}

impl CandidateMatch {
    fn new(name: &str, direct_signal: bool, risk_required: bool) -> Self {
        Self {
            name: name.to_string(),
            direct_signal,
            risk_required,
            order: candidate_order(name),
            score: 0,
            task_match: false,
            repo_match: false,
            profile_match: false,
            profile_relevant: false,
            excluded: false,
            conflict_suppressed: false,
            metadata: SkillMetadata::default(),
        }
    }

    fn score(&self) -> i32 {
        let mut score = 0;
        if self.task_match || self.direct_signal {
            score += 8;
        }
        if self.repo_match {
            score += 2;
        }
        if self.profile_match && self.profile_relevant {
            score += 3;
        }
        if self.risk_required {
            score += 4;
        }
        score
    }
}

fn candidate_order(name: &str) -> usize {
    [
        "frontend-design",
        "vibe-security-scan",
        "apk-mobile-analysis",
        "malware-triage",
        "binary-reverse-analysis",
        "api-and-interface-design",
        "observability-and-instrumentation",
        "performance-optimization",
        "deprecation-and-migration",
        "database-engineering",
        "mobile-application-engineering",
    ]
    .iter()
    .position(|candidate| *candidate == name)
    .unwrap_or(usize::MAX)
}

fn is_reverse_candidate(name: &str) -> bool {
    matches!(
        name,
        "apk-mobile-analysis" | "malware-triage" | "binary-reverse-analysis"
    )
}

fn collect_route_context(
    repo_root: &Path,
    task: &str,
    risk: RiskLane,
    operation_adapter: Option<AdapterKind>,
) -> RouteContext {
    let config = load_project_config(repo_root).ok();
    let profile = config.as_ref().and_then(|value| value.platform);
    let mut profiles = Vec::new();
    if let Some(config) = &config {
        if let Some(primary) = config.platform {
            profiles.push(platform_name(primary).to_string());
        }
        profiles.extend(
            config
                .platform_extensions
                .iter()
                .map(|value| platform_name(*value).to_string()),
        );
    }
    profiles.sort();
    profiles.dedup();

    let mut repository_signals = Vec::new();
    let should_survey = risk != RiskLane::Low
        || profile.is_none()
        || profile == Some(ProjectPlatform::Unknown)
        || task.len() > 160;
    if should_survey {
        if let Ok(survey) = survey_repository(repo_root) {
            repository_signals.extend(
                survey
                    .stack_hints
                    .iter()
                    .chain(survey.entrypoints.iter())
                    .map(|item| format!("{} {}", item.label, item.path).to_lowercase()),
            );
            repository_signals.extend(
                survey
                    .risky_surfaces
                    .iter()
                    .map(|item| format!("{} {}", item.label, item.path).to_lowercase()),
            );
            repository_signals.push(format!("{:?}", survey.project_type).to_lowercase());
        }
    }

    let mut state_signals = Vec::new();
    for relative in [
        "docs/baron/harness/CURRENT_INTENT.md",
        "docs/baron/plans/CURRENT.md",
        "docs/baron/continuity/CURRENT.md",
        "docs/baron/continuity/CURRENT_RECOVERY.md",
    ] {
        if repo_root.join(relative).is_file() {
            state_signals.push(relative.to_string());
        }
    }
    let capabilities_available = load_capability_state(repo_root)
        .ok()
        .flatten()
        .map(|state| {
            operation_adapter
                .map(|adapter| state.adapter.supported() == Some(adapter))
                .unwrap_or(true)
        })
        .unwrap_or(false);
    let affected_paths = state_signals
        .iter()
        .flat_map(|relative| {
            read_bounded_text(&repo_root.join(relative), 12_000)
                .lines()
                .filter_map(|line| line.strip_prefix("- Changed files: "))
                .flat_map(|line| line.split(',').map(|value| value.trim().to_lowercase()))
                .collect::<Vec<_>>()
        })
        .take(16)
        .collect();
    RouteContext {
        profile,
        profiles,
        repository_signals,
        work_shape: decide_work_shape(repo_root, task)
            .ok()
            .map(|decision| decision.work_shape.as_str().to_string()),
        state_signals,
        capabilities_available,
        affected_paths,
    }
}

fn profile_matches(metadata: &SkillMetadata, profiles: &[String]) -> bool {
    metadata
        .profile_affinities
        .iter()
        .any(|profile| profiles.iter().any(|configured| configured == profile))
}

fn profile_relevant(name: &str, task: &str, context: &RouteContext) -> bool {
    let configured = context.profiles.iter().any(|profile| {
        matches!(
            (profile.as_str(), name),
            ("database", "database-engineering")
                | ("mobile", "mobile-application-engineering")
                | ("data", "observability-and-instrumentation")
        )
    });
    configured
        && (match name {
            "database-engineering" => contains_any(
                task,
                &[
                    "database",
                    "persistence",
                    "repository",
                    "data access",
                    "schema",
                    "query",
                    "transaction",
                ],
            ),
            "mobile-application-engineering" => contains_any(
                task,
                &[
                    "mobile",
                    "android",
                    "ios",
                    "app",
                    "lifecycle",
                    "navigation",
                    "offline",
                    "permission",
                    "device",
                ],
            ),
            _ => true,
        })
}

fn database_evidence(task: &str, repository_signals: &[String]) -> bool {
    let positive = [
        "database",
        "relational",
        "foreign key",
        "fk",
        "primary key",
        "pk",
        "constraint",
        "nullability",
        "unique",
        "referential integrity",
        "normalization",
        "denormalization",
        "index",
        "query plan",
        "n+1",
        "transaction",
        "isolation",
        "lock",
        "deadlock",
        "orm usage",
        "repository",
        "persistence",
        "migrate",
        "migration",
        "backfill",
        "rollback",
    ]
    .iter()
    .any(|term| contains_term(task, term));
    let repo = repository_signals.iter().any(|signal| {
        contains_any(
            signal,
            &[
                "migration",
                "schema",
                "prisma",
                "diesel",
                "sqlx",
                "database",
                "postgres",
                "mysql",
            ],
        )
    });
    positive || repo
}

fn data_evidence(task: &str, repository_signals: &[String]) -> bool {
    [
        "etl",
        "elt",
        "ingestion",
        "kafka",
        "pipeline",
        "batch",
        "stream",
        "transformation",
        "lineage",
        "partition",
        "reproducibility",
        "analytical backfill",
        "data quality",
        "csv",
    ]
    .iter()
    .any(|term| contains_term(task, term))
        || repository_signals
            .iter()
            .any(|signal| contains_any(signal, &["pipeline", "kafka", "airflow", "spark", "etl"]))
}

fn mobile_evidence(task: &str, repository_signals: &[String]) -> bool {
    [
        "mobile app",
        "android app",
        "ios app",
        "lifecycle",
        "navigation",
        "offline",
        "local storage",
        "permission",
        "foreground",
        "background",
        "deep link",
        "emulator",
        "device",
        "battery",
        "connectivity",
    ]
    .iter()
    .any(|term| contains_term(task, term))
        || repository_signals
            .iter()
            .any(|signal| contains_any(signal, &["android", "ios", "flutter", "react native"]))
}

fn metadata_for(name: &str, contracts: &[SkillContract]) -> SkillMetadata {
    if let Some(contract) = contracts.iter().find(|contract| contract.name == name) {
        let metadata = &contract.metadata;
        if !metadata.triggers.is_empty()
            || !metadata.exclusions.is_empty()
            || !metadata.profile_affinities.is_empty()
            || !metadata.dependencies.is_empty()
            || !metadata.conflicts.is_empty()
            || !metadata.evidence.is_empty()
            || !metadata.verification.is_empty()
        {
            return metadata.clone();
        }
    }
    builtin_metadata(name)
}

fn builtin_metadata(name: &str) -> SkillMetadata {
    let values = |items: &[&str]| items.iter().map(|item| (*item).to_string()).collect();
    match name {
        "frontend-design" => SkillMetadata {
            triggers: values(&[
                "frontend",
                "ui",
                "layout",
                "responsive",
                "component",
                "homepage",
                "dashboard",
                "browser",
            ]),
            ..SkillMetadata::default()
        },
        "vibe-security-scan" => SkillMetadata {
            triggers: values(&[
                "auth",
                "login",
                "permission",
                "tenant",
                "rls",
                "security",
                "secret",
                "jwt",
                "cors",
                "upload",
                "payment",
                "crypto",
                "untrusted input",
                "sensitive data",
                "injection",
            ]),
            ..SkillMetadata::default()
        },
        "apk-mobile-analysis" => SkillMetadata {
            triggers: values(&["apk", "android binary", "apk reverse"]),
            exclusions: values(&["live exploitation", "credential theft"]),
            ..SkillMetadata::default()
        },
        "malware-triage" => SkillMetadata {
            triggers: values(&[
                "malware",
                "malware sample",
                "malware triage",
                "firmware analysis",
            ]),
            ..SkillMetadata::default()
        },
        "binary-reverse-analysis" => SkillMetadata {
            triggers: values(&[
                "binary reverse",
                "reverse engineering",
                "disassembly",
                "decompile",
                "firmware",
            ]),
            ..SkillMetadata::default()
        },
        "api-and-interface-design" => SkillMetadata {
            triggers: values(&[
                "api",
                "endpoint",
                "rest",
                "graphql",
                "interface",
                "contract",
                "request",
                "response",
                "sdk",
                "versioning",
                "compatibility",
            ]),
            ..SkillMetadata::default()
        },
        "observability-and-instrumentation" => SkillMetadata {
            triggers: values(&[
                "observability",
                "instrumentation",
                "log",
                "logging",
                "metric",
                "metrics",
                "tracing",
                "alert",
                "slo",
                "monitor",
            ]),
            ..SkillMetadata::default()
        },
        "performance-optimization" => SkillMetadata {
            triggers: values(&[
                "performance",
                "latency",
                "slow",
                "speed",
                "throughput",
                "cache",
                "caching",
                "bundle",
                "lcp",
                "inp",
                "cls",
                "lighthouse",
                "loading",
            ]),
            ..SkillMetadata::default()
        },
        "deprecation-and-migration" => SkillMetadata {
            triggers: values(&[
                "migration",
                "migrate",
                "deprecate",
                "deprecated",
                "legacy",
                "backward",
                "compatibility",
                "breaking change",
                "backfill",
            ]),
            ..SkillMetadata::default()
        },
        "database-engineering" => SkillMetadata {
            triggers: values(&[
                "database",
                "relational",
                "foreign key",
                "primary key",
                "constraint",
                "nullability",
                "uniqueness",
                "referential integrity",
                "normalization",
                "denormalization",
                "index",
                "query plan",
                "n+1",
                "transaction",
                "isolation",
                "lock",
                "deadlock",
                "migrate",
                "migration",
                "backfill",
                "rollback",
                "orm usage",
                "repository",
                "persistence",
            ]),
            exclusions: values(&[
                "csv transform",
                "kafka ingestion",
                "analytics quality",
                "frontend page",
                "pure http handler",
                "incidental sql",
            ]),
            profile_affinities: values(&["database", "backend", "fullstack"]),
            dependencies: values(&["superpowers"]),
            conflicts: values(&["mobile-application-engineering", "apk-mobile-analysis"]),
            verification: values(&[
                "schema constraints",
                "query plan/index",
                "transaction/concurrency",
                "migration dry-run/rollback",
                "integrity check",
            ]),
            evidence: values(&[
                "schema or query evidence",
                "affected data boundary",
                "migration or integrity impact",
            ]),
        },
        "mobile-application-engineering" => SkillMetadata {
            triggers: values(&[
                "mobile app",
                "android app",
                "ios app",
                "lifecycle",
                "navigation",
                "offline",
                "local storage",
                "permission flow",
                "foreground",
                "background",
                "deep link",
                "emulator",
                "device",
                "connectivity",
                "battery",
            ]),
            exclusions: values(&[
                "apk reverse",
                "apk manifest",
                "binary reverse",
                "decompile",
                "disassembly",
                "malware triage",
                "static artifact analysis",
            ]),
            profile_affinities: values(&["mobile", "fullstack"]),
            dependencies: values(&["superpowers"]),
            conflicts: values(&[
                "apk-mobile-analysis",
                "binary-reverse-analysis",
                "malware-triage",
            ]),
            verification: values(&[
                "lifecycle state",
                "offline/network failure",
                "permission state",
                "platform build/release",
            ]),
            evidence: values(&[
                "affected app flow",
                "platform state",
                "device or emulator conditions",
            ]),
        },
        _ => SkillMetadata::default(),
    }
}

fn candidate_reason(name: &str, reverse_analysis: bool) -> &'static str {
    match name {
        "vibe-security-scan" => "security-sensitive task evidence requires defensive scan guidance",
        "apk-mobile-analysis" | "malware-triage" | "binary-reverse-analysis"
            if reverse_analysis =>
        {
            "narrow defensive reverse-analysis guidance"
        }
        "database-engineering" => "database modeling/query/transaction/migration evidence",
        "mobile-application-engineering" => "mobile lifecycle/device/network evidence",
        "api-and-interface-design" => "API/interface boundary evidence",
        "observability-and-instrumentation" => "observability and operational evidence",
        "performance-optimization" => "measured performance evidence",
        "deprecation-and-migration" => "compatibility and migration evidence",
        "frontend-design" => "frontend/UI evidence",
        _ => "domain evidence matches the task",
    }
}

fn verification_for(
    context: &RouteContext,
    contracts: &[SkillContract],
    selected_names: &[String],
    task: &str,
    flags: &RouteFlags,
) -> Vec<RoutingDecision> {
    let mut values = Vec::new();
    let mut add = |name: &str, reason: &str| {
        if !values
            .iter()
            .any(|item: &RoutingDecision| item.name == name)
            && values.len() < 6
        {
            values.push(RoutingDecision {
                name: name.to_string(),
                reason: reason.to_string(),
            });
        }
    };
    if flags.database {
        add("schema-integrity", "selected; profile-influenced; verify constraints, nullability, uniqueness, and referential integrity");
        if contains_any(task, &["index", "query plan", "n+1", "latency", "slow"]) {
            add("query-plan", "selected; task-evidence; capture query-plan/index evidence and bounded query shape");
        }
        if contains_any(
            task,
            &[
                "transaction",
                "isolation",
                "lock",
                "deadlock",
                "concurrency",
                "race",
            ],
        ) {
            add(
                "transaction-concurrency",
                "selected; risk-required; run a deterministic concurrency or isolation test",
            );
        }
        if flags.migration {
            add("migration-rollback", "selected; risk-required; dry-run/backfill validation and rollback or forward-fix evidence");
        }
    }
    if flags.data {
        add(
            "data-integrity",
            "selected; profile-influenced; verify deterministic schema, quality, and lineage",
        );
        if contains_any(task, &["backfill", "replay", "batch", "stream"]) {
            add(
                "replay-backfill",
                "selected; task-evidence; verify idempotent replay and bounded backfill",
            );
        }
    }
    if !flags.database && !flags.data && flags.mobile {
        add(
            "mobile-lifecycle",
            "selected; profile-influenced; verify foreground/background and state restoration",
        );
        if contains_any(task, &["offline", "network", "connectivity"]) {
            add(
                "mobile-network-failure",
                "selected; task-evidence; verify offline, retry, and reconnect behavior",
            );
        }
        if contains_any(task, &["permission", "storage", "deep link"]) {
            add("mobile-platform-boundary", "selected; task-evidence; verify permission, storage, and platform boundary behavior");
        }
    } else if !flags.database && !flags.data {
        add("focused-tests", "selected; evidence-missing only for specialized verification; run focused regression tests");
    }
    for name in selected_names
        .iter()
        .filter(|name| name.as_str() != "superpowers")
    {
        let metadata = metadata_for(name, contracts);
        for hint in metadata.verification.iter().take(2) {
            let verification_name = hint
                .split_whitespace()
                .next()
                .unwrap_or("domain-check")
                .replace('/', "-");
            add(
                &verification_name,
                &format!("selected; metadata-derived verification hint: {hint}"),
            );
        }
    }
    let durable_work = matches!(
        context.work_shape.as_deref(),
        Some("durable" | "requires_confirmation")
    );
    if flags.security || flags.risk == RiskLane::High {
        add(
            "proof-trace",
            "selected; risk-required; record execution proof and detailed trace evidence",
        );
    } else if durable_work {
        add(
            "proof-trace",
            "selected; work-shape-required; durable or recovery work needs execution proof and trace evidence",
        );
    }
    values
}

fn normalize_task(task: &str) -> String {
    task.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn contains_term(value: &str, term: &str) -> bool {
    let term = term.trim();
    if term.is_empty() {
        return false;
    }
    if term.contains('-') {
        return value.contains(term);
    }
    if term.chars().any(char::is_whitespace) {
        return value.contains(term);
    }
    value
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '+')
        .any(|token| token == term)
}

fn bounded_text(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn read_bounded_text(path: &Path, max_bytes: usize) -> String {
    let Ok(file) = fs::File::open(path) else {
        return String::new();
    };
    let mut bytes = Vec::new();
    let _ = file.take(max_bytes as u64).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn record_gate_evidence(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    agent: &str,
    summary: &str,
) -> Result<GateEvidence> {
    let repo_path = repo_root.as_ref().join("docs/baron/control-plane/GATES.md");
    let vault_path = vault.project_root.join("ControlPlane/GATES.md");
    let item = format!("- {} - `{}` - {}", now(), agent.trim(), summary.trim());
    append(&repo_path, "# Baron Quality Gate Evidence\n\n", &item)?;
    append(&vault_path, "# Baron Quality Gate Evidence\n\n", &item)?;
    Ok(GateEvidence {
        agent: agent.trim().to_string(),
        repo_path,
        vault_path,
    })
}

pub fn record_gate_evidence_with_receipt(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    agent: &str,
    summary: &str,
    receipt_id: &str,
) -> Result<GateEvidence> {
    let _ = (repo_root, vault, agent, summary, receipt_id);
    bail!("explicit gate receipt binding is required; use record_gate_evidence_with_receipt_bound")
}

pub fn record_gate_evidence_with_receipt_bound(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    agent: &str,
    summary: &str,
    receipt_id: &str,
    binding: &GateReceiptBinding,
) -> Result<GateEvidence> {
    let repo_root = repo_root.as_ref();
    validate_gate_text(agent, "agent", 120)?;
    validate_gate_text(summary, "summary", 8_000)?;
    if gate_kind_for_agent(agent) != binding.gate_kind.trim() {
        bail!(
            "gate receipt kind `{}` is not authorized for `{}`",
            binding.gate_kind,
            agent.trim()
        );
    }
    let receipt = load_receipts(repo_root)?
        .into_iter()
        .find(|receipt| receipt.receipt_id == receipt_id.trim())
        .with_context(|| format!("Trusted execution receipt not found: {}", receipt_id.trim()))?;
    if !receipt_matches_context(repo_root, &receipt, binding)? {
        bail!(
            "Gate receipt `{}` is stale, failed, mismatched, replayed, or not current-operation evidence",
            receipt.receipt_id
        );
    }
    let repo_path = repo_root.join("docs/baron/control-plane/GATES.md");
    let vault_path = vault.project_root.join("ControlPlane/GATES.md");
    let existing = fs::read_to_string(&repo_path).unwrap_or_default();
    if existing.lines().any(|line| {
        line.contains(&format!("`{}`", agent.trim()))
            && line.contains(&format!("trusted_receipt=`{}`", receipt.receipt_id))
    }) {
        bail!(
            "gate receipt `{}` has already been recorded for `{}`",
            receipt.receipt_id,
            agent.trim()
        );
    }
    let item = format!(
        "- {} - `{}` - {} - trusted_receipt=`{}` gate_kind=`{}` task_id=`{}` operation_id=`{}` adapter=`{}` session_id=`{}` request_id=`{}`",
        now(),
        agent.trim(),
        summary.trim(),
        receipt.receipt_id,
        binding.gate_kind.trim(),
        binding.task_id.trim(),
        binding.operation_id.trim(),
        binding.adapter.trim(),
        binding.session_id.trim(),
        binding.request_id.trim(),
    );
    append(&repo_path, "# Baron Quality Gate Evidence\n\n", &item)?;
    append(&vault_path, "# Baron Quality Gate Evidence\n\n", &item)?;
    Ok(GateEvidence {
        agent: agent.trim().to_string(),
        repo_path,
        vault_path,
    })
}

fn validate_gate_text(value: &str, name: &str, max_chars: usize) -> Result<()> {
    if value.trim().is_empty()
        || value.chars().count() > max_chars
        || value
            .chars()
            .any(|character| matches!(character, '`' | '\n' | '\r'))
    {
        bail!("gate {name} is missing, too long, or contains a reserved marker");
    }
    Ok(())
}

pub fn gate_evidence_status_strict(
    repo_root: impl AsRef<Path>,
    required_agents: &[String],
) -> Result<GateEvidenceStatus> {
    gate_evidence_status_strict_for_context(repo_root, required_agents, None)
}

/// Evaluate quality-gate receipts within the task and adapter scope of the
/// current proof operation.  Individual receipt lines still carry and verify
/// their complete operation/session/request binding; this scope prevents a
/// valid receipt from another task from satisfying the current plan.
pub fn gate_evidence_status_strict_for_scope(
    repo_root: impl AsRef<Path>,
    required_agents: &[String],
    task_id: &str,
    adapter: &str,
) -> Result<GateEvidenceStatus> {
    gate_evidence_status_strict_for_context_and_scope(
        repo_root,
        required_agents,
        None,
        Some((task_id.trim(), adapter.trim())),
    )
}

pub fn gate_evidence_status_strict_for_context(
    repo_root: impl AsRef<Path>,
    required_agents: &[String],
    expected_context: Option<&GateReceiptBinding>,
) -> Result<GateEvidenceStatus> {
    gate_evidence_status_strict_for_context_and_scope(
        repo_root,
        required_agents,
        expected_context,
        None,
    )
}

fn gate_evidence_status_strict_for_context_and_scope(
    repo_root: impl AsRef<Path>,
    required_agents: &[String],
    expected_context: Option<&GateReceiptBinding>,
    expected_scope: Option<(&str, &str)>,
) -> Result<GateEvidenceStatus> {
    let repo_root = repo_root.as_ref();
    let content =
        fs::read_to_string(repo_root.join("docs/baron/control-plane/GATES.md")).unwrap_or_default();
    let receipts = load_receipts(repo_root)?;
    let mut missing_agents = Vec::new();
    for agent in required_agents {
        let needle = format!("`{}`", agent.trim());
        let satisfied = content.lines().any(|line| {
            if !line.contains(&needle) {
                return false;
            }
            let Some((_, value)) = line.split_once("trusted_receipt=`") else {
                return false;
            };
            let Some((receipt_id, _)) = value.split_once('`') else {
                return false;
            };
            receipts
                .iter()
                .find(|receipt| receipt.receipt_id == receipt_id.trim())
                .map(|receipt| {
                    let current = receipt_is_current(repo_root, receipt).unwrap_or(false)
                        && receipt.provenance
                            == crate::execution_receipt::ReceiptProvenance::TrustedCurrentOperation;
                    let binding_matches = if let Some(expected) = expected_context {
                        receipt_matches_context(repo_root, receipt, expected).unwrap_or(false)
                    } else if let Some((expected_task, expected_adapter)) = expected_scope {
                        crate::execution_receipt::receipt_is_current_authority(repo_root, receipt)
                            .unwrap_or(false)
                            && receipt.task_id.as_deref() == Some(expected_task)
                            && receipt.adapter.as_deref() == Some(expected_adapter)
                    } else {
                        crate::execution_receipt::receipt_is_current_authority(repo_root, receipt)
                            .unwrap_or(false)
                    };
                    let line_matches = parse_gate_binding(line)
                        .map(|binding| {
                            gate_kind_for_agent(agent) == binding.gate_kind
                                && binding.gate_kind
                                    == receipt.gate_kind.clone().unwrap_or_default()
                                && binding.task_id == receipt.task_id.clone().unwrap_or_default()
                                && binding.operation_id
                                    == receipt.operation_id.clone().unwrap_or_default()
                                && binding.adapter == receipt.adapter.clone().unwrap_or_default()
                                && binding.session_id
                                    == receipt.session_id.clone().unwrap_or_default()
                                && binding.request_id
                                    == receipt.request_id.clone().unwrap_or_default()
                        })
                        .unwrap_or(false);
                    current && binding_matches && line_matches
                })
                .unwrap_or(false)
        });
        if !satisfied {
            missing_agents.push(agent.clone());
        }
    }
    Ok(GateEvidenceStatus {
        passed: missing_agents.is_empty(),
        missing_agents,
    })
}

pub fn gate_evidence_status(
    repo_root: impl AsRef<Path>,
    required_agents: &[String],
) -> Result<GateEvidenceStatus> {
    gate_evidence_status_strict(repo_root, required_agents)
}

fn parse_gate_binding(line: &str) -> Option<GateReceiptBinding> {
    let value = |name: &str| {
        let marker = format!("{name}=`");
        let start = line.find(&marker)? + marker.len();
        let rest = &line[start..];
        let end = rest.find('`')?;
        Some(rest[..end].to_string())
    };
    Some(GateReceiptBinding {
        gate_kind: value("gate_kind")?,
        task_id: value("task_id")?,
        operation_id: value("operation_id")?,
        adapter: value("adapter")?,
        session_id: value("session_id")?,
        request_id: value("request_id")?,
    })
}

fn gate_kind_for_agent(agent: &str) -> String {
    format!("quality:{}", agent.trim())
}

fn owns_workflow(skill: &SkillContract) -> bool {
    let name = skill.name.to_lowercase();
    let combined = format!(
        "{}\n{}",
        skill.description.to_lowercase(),
        skill.body.to_lowercase()
    );
    name == "superpowers"
        || combined.contains("is the workflow core")
        || combined.contains("project workflow core")
        || combined.contains("my workflow core")
        || combined.contains("replaces superpowers")
}

fn load_skills(repo_root: &Path) -> Result<Vec<SkillContract>> {
    let mut skills = Vec::new();
    for root in skill_roots(repo_root) {
        if !root.exists() {
            continue;
        }
        let mut entries = fs::read_dir(&root)
            .with_context(|| format!("Could not read {}", root.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path().join("SKILL.md");
            if path.exists() {
                skills.push(parse_skill(&path)?);
            }
        }
    }
    Ok(skills)
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| contains_term(value, needle))
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    content.push('\n');
    write(path, &content)
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn load_agents(repo_root: &Path) -> Result<Vec<AgentContract>> {
    let mut agents = Vec::new();
    for root in agent_roots(repo_root) {
        if !root.exists() {
            continue;
        }
        let mut entries = fs::read_dir(&root)
            .with_context(|| format!("Could not read {}", root.display()))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Could not read {}", path.display()))?;
                agents.push(
                    toml::from_str(&content)
                        .with_context(|| format!("Could not parse {}", path.display()))?,
                );
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Could not read {}", path.display()))?;
                if let Some(agent) = parse_markdown_agent(&content) {
                    agents.push(agent);
                }
            }
        }
    }
    Ok(agents)
}

fn skill_roots(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join(".baron/core/skills"),
        repo_root.join(".codex/skills"),
        repo_root.join(".claude/skills"),
    ]
}

fn agent_roots(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join(".codex/agents"),
        repo_root.join(".claude/agents"),
        repo_root.join(".baron/core/agents"),
    ]
}

fn parse_skill(path: &Path) -> Result<SkillContract> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
    let (frontmatter, body) = split_frontmatter(&content);
    let name = frontmatter_value(frontmatter, "name")
        .or_else(|| {
            path.parent()
                .and_then(|parent| parent.file_name())
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string());
    let description = frontmatter_value(frontmatter, "description").unwrap_or_default();
    Ok(SkillContract {
        name,
        description,
        body: body.to_string(),
        metadata: SkillMetadata {
            triggers: frontmatter_list(frontmatter, "routing_triggers"),
            exclusions: frontmatter_list(frontmatter, "routing_exclusions"),
            profile_affinities: frontmatter_list(frontmatter, "profile_affinities"),
            dependencies: frontmatter_list(frontmatter, "routing_dependencies"),
            conflicts: frontmatter_list(frontmatter, "routing_conflicts"),
            evidence: frontmatter_list(frontmatter, "evidence_requirements"),
            verification: frontmatter_list(frontmatter, "verification_hints"),
        },
    })
}

fn parse_markdown_agent(content: &str) -> Option<AgentContract> {
    let (frontmatter, body) = split_frontmatter(content);
    let name = frontmatter_value(frontmatter, "name")?;
    let description = frontmatter_value(frontmatter, "description");
    Some(AgentContract {
        name,
        description,
        developer_instructions: Some(body.to_string()),
    })
}

fn split_frontmatter(content: &str) -> (&str, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return ("", content);
    };
    let Some(end) = rest.find("\n---") else {
        return ("", content);
    };
    (&rest[..end], &rest[end + 4..])
}

fn frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    frontmatter
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

fn frontmatter_list(frontmatter: &str, key: &str) -> Vec<String> {
    frontmatter_value(frontmatter, key)
        .unwrap_or_default()
        .split(',')
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}
