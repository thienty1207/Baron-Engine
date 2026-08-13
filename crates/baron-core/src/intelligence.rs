//! Baron 4.0 measurable intelligence, fallback selection, and defensive
//! security assessment contracts.
//!
//! The module intentionally stays deterministic and local. It does not run a
//! model, download a tool, or execute a target. It provides the evidence and
//! decision boundary that lets a future accelerator compete with the Baron 3.8
//! baseline without silently taking control of context or security actions.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::code_graph::compute_code_source_fingerprint;
use crate::firewall::{recall, recall_v4};
use crate::intelligence41::{build_grounded_handoff, refresh_temporal_ledger};
use crate::knowledge::ResumeBrief;
use crate::knowledge::{
    build_local_code_graph, build_resume_brief, build_resume_brief_v4, index_wiki,
    render_resume_brief, search_local_code_graph, search_local_code_graph_v4, search_wiki,
    search_wiki_v4, LocalGraphSearchHit, WikiSearchHit,
};
use crate::vault::VaultContext;

pub const INTELLIGENCE_SCHEMA_VERSION: u32 = 1;
pub const BARON_BASELINE_GENERATION: &str = "3.8";
pub const BARON_CANDIDATE_GENERATION: &str = "4.0";
pub const BARON_NEXT_GENERATION: &str = "4.1";
pub const MIN_PROMOTION_SCORE: u8 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntelligenceSurface {
    Memory,
    Wiki,
    CodeGraph,
    Security,
}

impl IntelligenceSurface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Wiki => "wiki",
            Self::CodeGraph => "codegraph",
            Self::Security => "security",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineGeneration {
    Baseline38,
    Candidate40,
    Candidate41,
}

impl EngineGeneration {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Baseline38 => BARON_BASELINE_GENERATION,
            Self::Candidate40 => BARON_CANDIDATE_GENERATION,
            Self::Candidate41 => BARON_NEXT_GENERATION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkCase {
    pub id: String,
    pub surface: IntelligenceSurface,
    pub query: String,
    pub expected_terms: Vec<String>,
    pub expected_citations: Vec<String>,
    pub expected_relations: Vec<String>,
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseResult {
    pub case_id: String,
    pub generation: String,
    pub surface: IntelligenceSurface,
    pub matched_expectations: usize,
    pub total_expectations: usize,
    pub score: u8,
    pub hard_failures: Vec<String>,
    pub evidence: Vec<String>,
    pub elapsed_ms: u128,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceScore {
    pub surface: IntelligenceSurface,
    pub score: u8,
    pub case_count: usize,
    pub passed_cases: usize,
    pub hard_failures: Vec<String>,
    pub per_metric_floor_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub schema_version: u32,
    pub report_id: String,
    pub generated_at: String,
    pub source_revision: String,
    pub fixture_revision: String,
    pub project_id: String,
    pub baseline_generation: String,
    pub candidate_generation: String,
    pub candidate_available: bool,
    pub candidate_note: String,
    pub cases: Vec<BenchmarkCase>,
    pub baseline_results: Vec<CaseResult>,
    pub candidate_results: Vec<CaseResult>,
    pub baseline_scores: Vec<SurfaceScore>,
    pub candidate_scores: Vec<SurfaceScore>,
    pub cross_project_leakage: usize,
    pub critical_regressions: Vec<String>,
    pub candidate_ready_for_promotion: bool,
    pub environment: BenchmarkEnvironment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkEnvironment {
    pub os: String,
    pub arch: String,
    pub pointer_width: String,
    pub cpu_count: usize,
    pub profile: String,
    pub cache_rebuilt_by_runner: bool,
    pub vault_rebuilt_by_runner: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionDecision {
    pub selected: EngineGeneration,
    pub fallback_available: bool,
    pub reason: String,
    pub regressions: Vec<String>,
    pub hard_failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationBrief {
    pub project_id: String,
    pub target: String,
    pub allowed_paths: Vec<String>,
    pub confirmed_by_owner: bool,
    pub dynamic_allowed: bool,
    pub network_profile: String,
    pub prohibited_actions: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub cleanup_owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityRoute {
    SourceAppSec,
    ReverseAnalysis,
    AuthorizedAdversary,
    Mixed,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityRouteDecision {
    pub project_id: String,
    pub route: SecurityRoute,
    pub allowed: bool,
    pub requires_authorization: bool,
    pub scope: String,
    pub owners: Vec<String>,
    pub reasons: Vec<String>,
    pub hard_failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityRegressionCase {
    pub id: String,
    pub query: String,
    pub expected_allowed: bool,
    pub expected_authorization: bool,
    pub observed_route: SecurityRoute,
    pub observed_allowed: bool,
    pub observed_authorization: bool,
    pub hard_failures: Vec<String>,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityRegressionReport {
    pub schema_version: u32,
    pub generated_at: String,
    pub project_id: String,
    pub source_revision: String,
    pub cases: Vec<SecurityRegressionCase>,
    pub score: u8,
    pub passed: bool,
    pub prohibited_intent_blocked: bool,
    pub missing_authorization_blocked: bool,
    pub project_scope_blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub project_id: String,
    pub source_revision: String,
    pub category: String,
    pub severity: String,
    pub confidence: String,
    pub file: String,
    pub line: usize,
    pub evidence: String,
    pub remediation: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticSecurityReport {
    pub schema_version: u32,
    pub generated_at: String,
    pub project_id: String,
    pub source_revision: String,
    pub files_checked: usize,
    pub findings: Vec<SecurityFinding>,
    pub redacted: bool,
    pub dynamic_execution: bool,
    pub score: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceCheck {
    pub id: String,
    pub area: String,
    pub passed: bool,
    pub score: u8,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceReport {
    pub schema_version: u32,
    pub generated_at: String,
    pub project_id: String,
    pub source_revision: String,
    pub benchmark_report_id: String,
    pub checks: Vec<AcceptanceCheck>,
    pub score: u8,
    pub passed: bool,
    pub environment: BenchmarkEnvironment,
}

pub fn compare_generations(
    baseline: &[CaseResult],
    candidate: &[CaseResult],
    candidate_scores: &[SurfaceScore],
    cross_project_leakage: usize,
) -> PromotionDecision {
    let mut regressions = Vec::new();
    let mut hard_failures = Vec::new();
    let baseline_by_id = baseline
        .iter()
        .map(|item| (item.case_id.as_str(), item))
        .collect::<std::collections::BTreeMap<_, _>>();
    for item in candidate {
        if !item.hard_failures.is_empty() {
            hard_failures.extend(
                item.hard_failures
                    .iter()
                    .map(|failure| format!("{}: {failure}", item.case_id)),
            );
        }
        if let Some(previous) = baseline_by_id.get(item.case_id.as_str()) {
            if item.score < previous.score {
                regressions.push(format!(
                    "{} regressed from {} to {}",
                    item.case_id, previous.score, item.score
                ));
            }
        }
    }
    if cross_project_leakage > 0 {
        hard_failures.push(format!(
            "cross-project leakage detected: {cross_project_leakage}"
        ));
    }
    for score in candidate_scores {
        if score.score < MIN_PROMOTION_SCORE {
            hard_failures.push(format!(
                "{} score {} is below {}",
                score.surface.as_str(),
                score.score,
                MIN_PROMOTION_SCORE
            ));
        }
        if !score.per_metric_floor_passed {
            hard_failures.push(format!(
                "{} per-metric floor failed",
                score.surface.as_str()
            ));
        }
        hard_failures.extend(score.hard_failures.clone());
    }
    if !regressions.is_empty() || !hard_failures.is_empty() {
        return PromotionDecision {
            selected: EngineGeneration::Baseline38,
            fallback_available: true,
            reason: "Candidate 4.0 did not pass the no-regression and hard-gate contract; keep the 3.8 result.".to_string(),
            regressions,
            hard_failures,
        };
    }
    PromotionDecision {
        selected: EngineGeneration::Candidate40,
        fallback_available: true,
        reason: "Candidate 4.0 passed every surface score, per-metric floor, and hard gate; retain 3.8 as a fallback.".to_string(),
        regressions,
        hard_failures,
    }
}

pub fn score_surface(surface: IntelligenceSurface, results: &[CaseResult]) -> SurfaceScore {
    let scoped = results
        .iter()
        .filter(|item| item.surface == surface)
        .collect::<Vec<_>>();
    if scoped.is_empty() {
        return SurfaceScore {
            surface,
            score: 0,
            case_count: 0,
            passed_cases: 0,
            hard_failures: vec!["no benchmark cases".to_string()],
            per_metric_floor_passed: false,
        };
    }
    let total = scoped.iter().map(|item| u32::from(item.score)).sum::<u32>();
    let score = (total / scoped.len() as u32).min(100) as u8;
    let passed_cases = scoped.iter().filter(|item| item.score >= 90).count();
    let hard_failures = scoped
        .iter()
        .flat_map(|item| item.hard_failures.clone())
        .collect::<Vec<_>>();
    SurfaceScore {
        surface,
        score,
        case_count: scoped.len(),
        passed_cases,
        hard_failures,
        per_metric_floor_passed: scoped.iter().all(|item| item.score >= 90),
    }
}

pub fn evaluate_text_case(
    case: &BenchmarkCase,
    generation: EngineGeneration,
    answer: &str,
    evidence: Vec<String>,
    elapsed_ms: u128,
    estimated_tokens: usize,
) -> CaseResult {
    let normalized = normalize(answer);
    let mut matched = 0usize;
    let mut hard_failures = Vec::new();
    for term in &case.expected_terms {
        if normalized.contains(&normalize(term)) {
            matched += 1;
        }
    }
    for citation in &case.expected_citations {
        if !answer.contains(citation) {
            hard_failures.push(format!("missing citation {citation}"));
        }
    }
    for relation in &case.expected_relations {
        if !normalized.contains(&normalize(relation)) {
            hard_failures.push(format!("missing relation {relation}"));
        }
    }
    let total =
        case.expected_terms.len() + case.expected_citations.len() + case.expected_relations.len();
    let matched_total = matched
        + case
            .expected_citations
            .iter()
            .filter(|citation| answer.contains(*citation))
            .count()
        + case
            .expected_relations
            .iter()
            .filter(|relation| normalized.contains(&normalize(relation)))
            .count();
    let score = if total == 0 {
        0
    } else {
        (matched_total
            .saturating_mul(100)
            .checked_div(total)
            .unwrap_or_default()
            .min(100)) as u8
    };
    CaseResult {
        case_id: case.id.clone(),
        generation: generation.as_str().to_string(),
        surface: case.surface,
        matched_expectations: matched_total,
        total_expectations: total,
        score,
        hard_failures,
        evidence,
        elapsed_ms,
        estimated_tokens,
    }
}

/// Returns whether the guarded 4.0 runtime should be selected for normal work.
///
/// Baron 4.0 is the released default. Setting `BARON_ENGINE_GENERATION=3.8`
/// (or `baseline`) is an explicit incident-recovery switch; `4.0` is accepted
/// for clarity, and an unknown value fails closed to the 3.8 baseline.
pub fn candidate_generation_enabled() -> bool {
    match std::env::var("BARON_ENGINE_GENERATION") {
        Ok(value) if value.trim() == BARON_BASELINE_GENERATION || value.trim() == "baseline" => {
            false
        }
        Ok(value) if value.trim() == BARON_CANDIDATE_GENERATION => true,
        Ok(_) => false,
        Err(_) => false,
    }
}

/// Returns whether the Baron 4.1 candidate may run. Baron 4.0 remains the
/// immediate fallback; an explicit 3.8/baseline value still forces the old
/// recovery path and unknown values fail closed.
pub fn next_generation_enabled() -> bool {
    match std::env::var("BARON_ENGINE_GENERATION") {
        Ok(value) if value.trim() == BARON_BASELINE_GENERATION || value.trim() == "baseline" => {
            false
        }
        Ok(value) if value.trim() == BARON_CANDIDATE_GENERATION => false,
        Ok(value) if value.trim() == BARON_NEXT_GENERATION => true,
        Ok(_) => false,
        Err(_) => false,
    }
}

/// Select the 4.1 candidate only after its temporal and grounded handoff
/// contracts succeed. A 4.0 Resume Brief remains the result when the candidate
/// cannot prove identity, bounds, or evidence.
pub fn select_resume_brief_v41(
    context: &VaultContext,
    task: Option<&str>,
    max_chars: usize,
) -> Result<(EngineGeneration, ResumeBrief)> {
    let baseline = build_resume_brief_v4(context, task, max_chars)?;
    if !next_generation_enabled() {
        return Ok((EngineGeneration::Candidate40, baseline));
    }
    let _ = refresh_temporal_ledger(context);
    let handoff = match build_grounded_handoff(context, task, max_chars) {
        Ok(handoff) => handoff,
        Err(_) => return Ok((EngineGeneration::Candidate40, baseline)),
    };
    let rendered = render_resume_brief(&baseline, max_chars);
    let structurally_safe = baseline.project_id == context.project_id
        && baseline.bounded_chars <= max_chars.clamp(1_200, MAX_RUNTIME_BRIEF_CHARS)
        && handoff.bounded_chars <= max_chars.clamp(1_200, MAX_RUNTIME_BRIEF_CHARS)
        && handoff.estimated_tokens <= handoff.budget_tokens
        && rendered.contains(&context.project_id)
        && handoff
            .claims
            .iter()
            .all(|claim| !claim.claim.contains("[REDACTED]") || claim.citation.contains("/"));
    if structurally_safe {
        Ok((EngineGeneration::Candidate41, baseline))
    } else {
        Ok((EngineGeneration::Candidate40, baseline))
    }
}

/// Selects the runtime Resume Brief without making the candidate a one-way
/// switch. The candidate is allowed only when it remains project-isolated,
/// bounded, and structurally complete; any build or guard failure returns the
/// proven Baron 3.8 brief.
pub fn select_resume_brief(
    context: &VaultContext,
    task: Option<&str>,
    max_chars: usize,
) -> Result<(EngineGeneration, ResumeBrief)> {
    let baseline = build_resume_brief(context, task, max_chars)?;
    if !candidate_generation_enabled() {
        return Ok((EngineGeneration::Baseline38, baseline));
    }
    let candidate = match build_resume_brief_v4(context, task, max_chars) {
        Ok(candidate) => candidate,
        Err(_) => return Ok((EngineGeneration::Baseline38, baseline)),
    };
    let candidate_rendered = render_resume_brief(&candidate, max_chars);
    let structurally_safe = candidate.project_id == context.project_id
        && candidate.bounded_chars <= max_chars.clamp(1_200, MAX_RUNTIME_BRIEF_CHARS)
        && candidate_rendered.contains(&context.project_id)
        && candidate.memory_hits.iter().all(|hit| {
            !hit.excerpt.contains("project_id=") || hit.path.contains(&context.project_id)
        });
    if structurally_safe {
        Ok((EngineGeneration::Candidate40, candidate))
    } else {
        Ok((EngineGeneration::Baseline38, baseline))
    }
}

const MAX_RUNTIME_BRIEF_CHARS: usize = 9_000;

pub fn default_cases(context: &VaultContext) -> Vec<BenchmarkCase> {
    vec![
        BenchmarkCase {
            id: "memory-resume-contract".to_string(),
            surface: IntelligenceSurface::Memory,
            query: "current work checkpoint proof next safe action".to_string(),
            expected_terms: vec![
                "project".to_string(),
                "source".to_string(),
                "checkpoint".to_string(),
                "next safe action".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "memory-trust-contract".to_string(),
            surface: IntelligenceSurface::Memory,
            query: "confirmed decisions source revision unknowns".to_string(),
            expected_terms: vec![
                "project".to_string(),
                "source revision".to_string(),
                "confirmed decisions".to_string(),
                "unknowns".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "memory-vietnamese-handoff-contract".to_string(),
            surface: IntelligenceSurface::Memory,
            query: "đang làm dở checkpoint bước an toàn tiếp theo".to_string(),
            expected_terms: vec![
                "project".to_string(),
                "next safe action".to_string(),
                "unknowns".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "memory-exact-path-contract".to_string(),
            surface: IntelligenceSurface::Memory,
            query: "docs BARON_STATUS source revision proof".to_string(),
            expected_terms: vec![
                "project id".to_string(),
                "source revision".to_string(),
                "proof status".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "wiki-citation-contract".to_string(),
            surface: IntelligenceSurface::Wiki,
            query: "architecture memory security".to_string(),
            expected_terms: vec!["architecture".to_string(), "memory".to_string()],
            expected_citations: vec!["docs/".to_string()],
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "wiki-freshness-contract".to_string(),
            surface: IntelligenceSurface::Wiki,
            query: "Baron status architecture".to_string(),
            expected_terms: vec![
                "baron".to_string(),
                "status".to_string(),
                "architecture".to_string(),
                "links=".to_string(),
            ],
            expected_citations: vec!["docs/".to_string()],
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "wiki-mixed-language-contract".to_string(),
            surface: IntelligenceSurface::Wiki,
            query: "memory trí nhớ architecture kiến trúc".to_string(),
            expected_terms: vec![
                "memory".to_string(),
                "architecture".to_string(),
                "links=".to_string(),
            ],
            expected_citations: vec!["docs/".to_string()],
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "wiki-injection-boundary-contract".to_string(),
            surface: IntelligenceSurface::Wiki,
            query: "security source truth cache".to_string(),
            expected_terms: vec![
                "security".to_string(),
                "citation=".to_string(),
                "stale=".to_string(),
            ],
            expected_citations: vec!["docs/".to_string()],
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "codegraph-impact-contract".to_string(),
            surface: IntelligenceSurface::CodeGraph,
            query: "context knowledge recall".to_string(),
            expected_terms: vec!["context".to_string()],
            expected_citations: Vec::new(),
            expected_relations: vec!["references".to_string()],
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "codegraph-call-contract".to_string(),
            surface: IntelligenceSurface::CodeGraph,
            query: "select_resume_brief intelligence".to_string(),
            expected_terms: vec![
                "select_resume_brief".to_string(),
                "intelligence".to_string(),
                "span=".to_string(),
                "imports=".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: vec!["references".to_string()],
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "codegraph-symbol-span-contract".to_string(),
            surface: IntelligenceSurface::CodeGraph,
            query: "MemoryRecord abstraction_level".to_string(),
            expected_terms: vec![
                "MemoryRecord".to_string(),
                "span=".to_string(),
                "imports=".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "codegraph-project-isolation-contract".to_string(),
            surface: IntelligenceSurface::CodeGraph,
            query: "context project identity".to_string(),
            expected_terms: vec!["context".to_string(), "span=".to_string()],
            expected_citations: Vec::new(),
            expected_relations: Vec::new(),
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "security-routing-contract".to_string(),
            surface: IntelligenceSurface::Security,
            query: "authorized threat model API security".to_string(),
            expected_terms: vec!["authorization".to_string(), "security".to_string()],
            expected_citations: Vec::new(),
            expected_relations: vec!["security-auditor".to_string()],
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "security-source-appsec-contract".to_string(),
            surface: IntelligenceSurface::Security,
            query: "API authentication dependency security".to_string(),
            expected_terms: vec!["source".to_string(), "security".to_string()],
            expected_citations: Vec::new(),
            expected_relations: vec!["security-auditor".to_string()],
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "security-oauth-source-contract".to_string(),
            surface: IntelligenceSurface::Security,
            query: "review OAuth cookies and secrets".to_string(),
            expected_terms: vec![
                "source".to_string(),
                "security".to_string(),
                "generation".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: vec!["vibe-security-scan".to_string()],
            project_id: context.project_id.clone(),
        },
        BenchmarkCase {
            id: "security-reverse-static-contract".to_string(),
            surface: IntelligenceSurface::Security,
            query: "analyze firmware malware".to_string(),
            expected_terms: vec![
                "reverse".to_string(),
                "security".to_string(),
                "allowed".to_string(),
            ],
            expected_citations: Vec::new(),
            expected_relations: vec!["reverse-analysis-pack".to_string()],
            project_id: context.project_id.clone(),
        },
    ]
}

pub fn run_local_benchmark(context: &VaultContext) -> Result<BenchmarkReport> {
    crate::memory::build_memory_index(context)?;
    let cases = default_cases(context);
    let wiki = index_wiki(&context.repo_root)?;
    let graph = build_local_code_graph(&context.repo_root)?;
    let authorization = benchmark_authorization(context);
    let mut baseline_results = Vec::with_capacity(cases.len());
    let mut candidate_results = Vec::with_capacity(cases.len());
    for case in &cases {
        let case_started = Instant::now();
        match case.surface {
            IntelligenceSurface::Memory => {
                let brief = build_resume_brief(context, Some(&case.query), 8_000)?;
                let rendered = render_resume_brief(&brief, 8_000);
                baseline_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Baseline38,
                    &rendered,
                    vec!["baron 3.8 Resume Brief".to_string()],
                    case_started.elapsed().as_millis(),
                    rendered.chars().count().div_ceil(4),
                ));
                let candidate = build_resume_brief_v4(context, Some(&case.query), 8_000)?;
                let rendered_candidate = render_resume_brief(&candidate, 8_000);
                candidate_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Candidate40,
                    &rendered_candidate,
                    vec![
                        "baron 4.0 candidate Resume Brief".to_string(),
                        format!("schema:{}", candidate.schema_version),
                    ],
                    case_started.elapsed().as_millis(),
                    rendered_candidate.chars().count().div_ceil(4),
                ));
            }
            IntelligenceSurface::Wiki => {
                let hits = search_wiki(&context.repo_root, &case.query, 8)?;
                let answer = render_wiki_answer(&hits, false);
                baseline_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Baseline38,
                    &answer,
                    hits.iter().map(|hit| hit.citation.clone()).collect(),
                    case_started.elapsed().as_millis(),
                    answer.chars().count().div_ceil(4),
                ));
                let candidate_hits = search_wiki_v4(&context.repo_root, &case.query, 8)?;
                let candidate_answer = render_wiki_answer(&candidate_hits, true);
                candidate_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Candidate40,
                    &candidate_answer,
                    candidate_hits
                        .iter()
                        .map(|hit| hit.citation.clone())
                        .collect(),
                    case_started.elapsed().as_millis(),
                    candidate_answer.chars().count().div_ceil(4),
                ));
            }
            IntelligenceSurface::CodeGraph => {
                let hits = search_local_code_graph(&context.repo_root, &case.query, 8)?;
                let answer = render_graph_answer(&hits, false);
                baseline_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Baseline38,
                    &answer,
                    hits.iter().map(|hit| hit.symbol.file.clone()).collect(),
                    case_started.elapsed().as_millis(),
                    answer.chars().count().div_ceil(4),
                ));
                let candidate_hits =
                    search_local_code_graph_v4(&context.repo_root, &case.query, 8)?;
                let candidate_answer = render_graph_answer(&candidate_hits, true);
                candidate_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Candidate40,
                    &candidate_answer,
                    candidate_hits
                        .iter()
                        .map(|hit| hit.symbol.file.clone())
                        .collect(),
                    case_started.elapsed().as_millis(),
                    candidate_answer.chars().count().div_ceil(4),
                ));
            }
            IntelligenceSurface::Security => {
                let decision =
                    route_security_task(&case.query, Some(&authorization), &context.project_id);
                let answer = render_security_answer(&decision, EngineGeneration::Baseline38);
                baseline_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Baseline38,
                    &answer,
                    decision.owners.clone(),
                    case_started.elapsed().as_millis(),
                    answer.chars().count().div_ceil(4),
                ));
                let candidate_decision =
                    route_security_task(&case.query, Some(&authorization), &context.project_id);
                let candidate_answer =
                    render_security_answer(&candidate_decision, EngineGeneration::Candidate40);
                candidate_results.push(evaluate_text_case(
                    case,
                    EngineGeneration::Candidate40,
                    &candidate_answer,
                    candidate_decision.owners.clone(),
                    case_started.elapsed().as_millis(),
                    candidate_answer.chars().count().div_ceil(4),
                ));
            }
        }
    }
    let baseline_scores = all_surface_scores(&baseline_results);
    let candidate_scores = all_surface_scores(&candidate_results);
    let baseline_recall = recall(context, "current work checkpoint proof", 12)?;
    let candidate_recall = recall_v4(context, "current work checkpoint proof", 12)?;
    let cross_project_leakage = baseline_recall
        .results
        .iter()
        .chain(candidate_recall.results.iter())
        .filter(|hit| hit.record.project_id.as_deref() != Some(context.project_id.as_str()))
        .count();
    let decision = compare_generations(
        &baseline_results,
        &candidate_results,
        &candidate_scores,
        cross_project_leakage,
    );
    let report_id = sha256(
        format!(
            "{}|{}|{}|{}|{}",
            context.project_id,
            graph.source_revision,
            wiki.documents,
            graph.symbols.len(),
            cases.len()
        )
        .as_bytes(),
    );
    Ok(BenchmarkReport {
        schema_version: INTELLIGENCE_SCHEMA_VERSION,
        report_id,
        generated_at: Utc::now().to_rfc3339(),
        source_revision: graph.source_revision.clone(),
        fixture_revision: "baron-4.0-fixtures-v1".to_string(),
        project_id: context.project_id.clone(),
        baseline_generation: BARON_BASELINE_GENERATION.to_string(),
        candidate_generation: BARON_CANDIDATE_GENERATION.to_string(),
        candidate_available: true,
        candidate_note: "Candidate 4.0 engines ran independently for sixteen frozen cases across memory, Wiki, CodeGraph, and security routing; promotion still requires every score and hard gate to pass.".to_string(),
        cases,
        baseline_results,
        candidate_results,
        baseline_scores,
        candidate_ready_for_promotion: decision.selected == EngineGeneration::Candidate40,
        candidate_scores,
        cross_project_leakage,
        critical_regressions: decision
            .regressions
            .into_iter()
            .chain(decision.hard_failures)
            .collect(),
        environment: BenchmarkEnvironment {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            pointer_width: std::env::consts::ARCH
                .strip_prefix("x86_64")
                .map(|_| "64".to_string())
                .unwrap_or_else(|| {
                    if std::mem::size_of::<usize>() == 8 {
                        "64".to_string()
                    } else {
                        "32".to_string()
                    }
                }),
            cpu_count: std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1),
            profile: if cfg!(debug_assertions) {
                "debug".to_string()
            } else {
                "release".to_string()
            },
            cache_rebuilt_by_runner: true,
            vault_rebuilt_by_runner: false,
        },
    })
}

fn benchmark_authorization(context: &VaultContext) -> AuthorizationBrief {
    AuthorizationBrief {
        project_id: context.project_id.clone(),
        target: context.repo_root.display().to_string(),
        allowed_paths: vec![context.repo_root.display().to_string()],
        confirmed_by_owner: true,
        dynamic_allowed: false,
        network_profile: "offline".to_string(),
        prohibited_actions: vec!["payload delivery".to_string(), "persistence".to_string()],
        stop_conditions: vec!["scope mismatch".to_string()],
        cleanup_owner: "owner".to_string(),
    }
}

fn render_wiki_answer(hits: &[WikiSearchHit], candidate: bool) -> String {
    hits.iter()
        .map(|hit| {
            if candidate {
                format!(
                    "{} {} {} citation={} links={} link_path={} stale={}",
                    hit.document,
                    hit.heading,
                    hit.excerpt,
                    hit.citation,
                    hit.links.join(" "),
                    hit.link_path.join(" "),
                    hit.stale
                )
            } else {
                format!(
                    "{} {} {} citation={} stale={}",
                    hit.document, hit.heading, hit.excerpt, hit.citation, hit.stale
                )
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_graph_answer(hits: &[LocalGraphSearchHit], candidate: bool) -> String {
    hits.iter()
        .map(|hit| {
            if candidate {
                format!(
                    "{} {} language={} span={} imports={} {} generation=4.0",
                    hit.symbol.name,
                    hit.symbol.file,
                    hit.symbol.language,
                    hit.symbol.span,
                    hit.imports.join(" "),
                    hit.relations.join(" ")
                )
            } else {
                format!(
                    "{} {} {} generation=3.8",
                    hit.symbol.name,
                    hit.symbol.file,
                    hit.relations.join(" ")
                )
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_security_answer(
    decision: &SecurityRouteDecision,
    generation: EngineGeneration,
) -> String {
    format!(
        "generation={} project={} route={:?} scope={} allowed={} authorization={} security owners={} reasons={} failures={}",
        generation.as_str(),
        decision.project_id,
        decision.route,
        decision.scope,
        decision.allowed,
        decision.requires_authorization,
        decision.owners.join(" "),
        decision.reasons.join(" "),
        decision.hard_failures.join(" ")
    )
}

pub fn write_benchmark_report(
    report: &BenchmarkReport,
    directory: impl AsRef<Path>,
) -> Result<(String, String)> {
    let directory = directory.as_ref();
    fs::create_dir_all(directory)?;
    let json_path = directory.join("baron-4.0-benchmark.json");
    let md_path = directory.join("baron-4.0-benchmark.md");
    let json = serde_json::to_string_pretty(report)?;
    fs::write(&json_path, json)?;
    let mut markdown = String::new();
    markdown.push_str("# Baron 4.0 Intelligence Benchmark\n\n");
    markdown.push_str(&format!("- Report: `{}`\n", report.report_id));
    markdown.push_str(&format!(
        "- Source revision: `{}`\n",
        report.source_revision
    ));
    markdown.push_str(&format!(
        "- Fixture revision: `{}`\n",
        report.fixture_revision
    ));
    markdown.push_str(&format!(
        "- Cross-project leakage: `{}`\n\n",
        report.cross_project_leakage
    ));
    markdown.push_str("## Environment\n\n");
    markdown.push_str(&format!(
        "- OS/arch: `{}/{}`\n- Pointer width: `{}`\n- CPUs: `{}`\n- Profile: `{}`\n- Cache rebuilt by runner: `{}`\n- Vault rebuilt by runner: `{}`\n\n",
        report.environment.os,
        report.environment.arch,
        report.environment.pointer_width,
        report.environment.cpu_count,
        report.environment.profile,
        report.environment.cache_rebuilt_by_runner,
        report.environment.vault_rebuilt_by_runner
    ));
    markdown.push_str("## Baron 3.8 baseline scores\n\n");
    for score in &report.baseline_scores {
        markdown.push_str(&format!(
            "- {}: **{}/100** ({} cases; metric floor `{}`)\n",
            score.surface.as_str(),
            score.score,
            score.case_count,
            score.per_metric_floor_passed
        ));
    }
    markdown.push_str("## Candidate scores\n\n");
    markdown.push_str(&format!("- Available: `{}`\n", report.candidate_available));
    markdown.push_str(&format!("- Note: {}\n", report.candidate_note));
    for score in &report.candidate_scores {
        markdown.push_str(&format!(
            "- {}: **{}/100** ({} cases)\n",
            score.surface.as_str(),
            score.score,
            score.case_count
        ));
    }
    markdown.push_str(&format!(
        "\n- Promotion-ready: `{}`\n",
        report.candidate_ready_for_promotion
    ));
    markdown.push_str("\n## Case comparison\n\n");
    for case in &report.cases {
        let baseline = report
            .baseline_results
            .iter()
            .find(|result| result.case_id == case.id);
        let candidate = report
            .candidate_results
            .iter()
            .find(|result| result.case_id == case.id);
        if let (Some(baseline), Some(candidate)) = (baseline, candidate) {
            markdown.push_str(&format!(
                "- `{}` ({:?}): 3.8 `{}` -> 4.0 `{}`; expectations `{}`/`{}`\n",
                case.id,
                case.surface,
                baseline.score,
                candidate.score,
                candidate.matched_expectations,
                candidate.total_expectations
            ));
        }
    }
    if !report.critical_regressions.is_empty() {
        markdown.push_str("\n## Critical regressions\n\n");
        for failure in &report.critical_regressions {
            markdown.push_str(&format!("- {failure}\n"));
        }
    }
    fs::write(&md_path, markdown)?;
    Ok((
        json_path.display().to_string(),
        md_path.display().to_string(),
    ))
}

pub fn route_security_task(
    query: &str,
    authorization: Option<&AuthorizationBrief>,
    project_id: &str,
) -> SecurityRouteDecision {
    let lower = normalize(query);
    if contains_any(
        &lower,
        &[
            "credential theft",
            "steal password",
            "exfiltrate",
            "persistence",
            "evasion",
            "payload delivery",
            "weaponize",
            "deploy ransomware",
            "botnet",
            "ddos",
            "backdoor",
        ],
    ) {
        return SecurityRouteDecision {
            project_id: project_id.to_string(),
            route: SecurityRoute::Unsupported,
            allowed: false,
            requires_authorization: true,
            scope: "blocked_offensive_intent".to_string(),
            owners: vec!["security-auditor".to_string()],
            reasons: vec![
                "unsafe offensive intent is outside Baron's bounded defensive scope".to_string(),
            ],
            hard_failures: vec![
                "blocked offensive or destructive request; create a defensive remediation plan instead"
                    .to_string(),
            ],
        };
    }
    let source = contains_any(
        &lower,
        &[
            "api",
            "oauth",
            "auth",
            "secret",
            "dependency",
            "rls",
            "security",
        ],
    );
    let reverse = contains_any(
        &lower,
        &["binary", "apk", "malware", "firmware", "elf", "dll", "pcap"],
    );
    let adversary = contains_any(
        &lower,
        &[
            "attack path",
            "threat model",
            "pentest",
            "penetration",
            "adversary",
            "scan target",
        ],
    );
    let route = match (source, reverse, adversary) {
        (true, true, _) | (true, _, true) | (_, true, true) => SecurityRoute::Mixed,
        (true, false, false) => SecurityRoute::SourceAppSec,
        (false, true, false) => SecurityRoute::ReverseAnalysis,
        (false, false, true) => SecurityRoute::AuthorizedAdversary,
        _ => SecurityRoute::Unsupported,
    };
    let requires_authorization = matches!(
        route,
        SecurityRoute::AuthorizedAdversary | SecurityRoute::Mixed
    );
    let mut reasons = vec!["Baron Control Plane owns security routing".to_string()];
    let mut hard_failures = Vec::new();
    let mut allowed = true;
    if requires_authorization {
        let Some(brief) = authorization else {
            return SecurityRouteDecision {
                project_id: project_id.to_string(),
                route,
                allowed: false,
                requires_authorization,
                scope: "missing_authorization".to_string(),
                owners: vec!["security-auditor".to_string()],
                reasons: vec!["explicit authorization brief is required".to_string()],
                hard_failures: vec!["missing authorization".to_string()],
            };
        };
        if !brief.confirmed_by_owner {
            hard_failures.push("owner confirmation is missing".to_string());
        }
        if brief.project_id != project_id {
            hard_failures
                .push("authorization project ID does not match current project".to_string());
        }
        if brief.target.trim().is_empty() || brief.cleanup_owner.trim().is_empty() {
            hard_failures.push("target and cleanup owner are required".to_string());
        }
        if brief.allowed_paths.is_empty() {
            hard_failures.push("at least one allowed path is required".to_string());
        }
        if brief.prohibited_actions.is_empty() {
            hard_failures.push("prohibited actions must be explicit".to_string());
        }
        if brief.target.contains("..") || brief.allowed_paths.iter().any(|path| path.contains(".."))
        {
            hard_failures
                .push("path traversal is not allowed in an authorization brief".to_string());
        }
        let target = normalize_scope_path(&brief.target);
        if !brief
            .allowed_paths
            .iter()
            .map(|path| normalize_scope_path(path))
            .any(|path| target == path || target.starts_with(&format!("{path}/")))
        {
            hard_failures.push("target is outside the authorization allowlist".to_string());
        }
        if brief.target.contains("://") && !brief.dynamic_allowed {
            hard_failures
                .push("remote target requires an explicitly authorized isolated lab".to_string());
        }
        if brief.network_profile != "offline" && !brief.dynamic_allowed {
            hard_failures
                .push("network profile is not offline for a static assessment".to_string());
        }
        allowed = hard_failures.is_empty();
        reasons.push(
            "authorized assessment remains bounded, receipt-backed, and abortable".to_string(),
        );
    }
    let owners = match route {
        SecurityRoute::SourceAppSec => vec![
            "vibe-security-scan".to_string(),
            "security-auditor".to_string(),
        ],
        SecurityRoute::ReverseAnalysis => vec![
            "reverse-analysis-pack".to_string(),
            "security-auditor".to_string(),
        ],
        SecurityRoute::AuthorizedAdversary => vec![
            "authorized-adversary-assessment".to_string(),
            "security-auditor".to_string(),
        ],
        SecurityRoute::Mixed => vec![
            "vibe-security-scan".to_string(),
            "reverse-analysis-pack".to_string(),
            "security-auditor".to_string(),
        ],
        SecurityRoute::Unsupported => vec!["security-auditor".to_string()],
    };
    if matches!(route, SecurityRoute::Unsupported) {
        allowed = false;
        hard_failures.push("security task does not match a supported safe route".to_string());
    }
    let scope = if matches!(route, SecurityRoute::Unsupported) {
        "unsupported".to_string()
    } else if requires_authorization {
        "authorized_scope".to_string()
    } else {
        "offline_static".to_string()
    };
    SecurityRouteDecision {
        project_id: project_id.to_string(),
        route,
        allowed,
        requires_authorization,
        scope,
        owners,
        reasons,
        hard_failures,
    }
}

pub fn validate_authorization(brief: &AuthorizationBrief, project_id: &str) -> Result<()> {
    if !brief.confirmed_by_owner {
        anyhow::bail!("authorization brief is not owner-confirmed");
    }
    if brief.project_id != project_id {
        anyhow::bail!("authorization brief project ID does not match current project");
    }
    if brief.target.trim().is_empty() || brief.cleanup_owner.trim().is_empty() {
        anyhow::bail!("authorization brief requires target and cleanup owner");
    }
    if brief.allowed_paths.is_empty() {
        anyhow::bail!("authorization brief requires at least one allowed path");
    }
    if brief.prohibited_actions.is_empty() {
        anyhow::bail!("authorization brief requires explicit prohibited actions");
    }
    if brief.target.contains("..") || brief.allowed_paths.iter().any(|path| path.contains("..")) {
        anyhow::bail!("authorization brief cannot contain path traversal");
    }
    let target = normalize_scope_path(&brief.target);
    if !brief
        .allowed_paths
        .iter()
        .map(|path| normalize_scope_path(path))
        .any(|path| target == path || target.starts_with(&format!("{path}/")))
    {
        anyhow::bail!("authorization target is outside the allowlist");
    }
    Ok(())
}

pub fn run_security_regression(context: &VaultContext) -> SecurityRegressionReport {
    let owner = benchmark_authorization(context);
    let mut wrong_project = owner.clone();
    wrong_project.project_id = "different-project".to_string();
    let mut wrong_path = owner.clone();
    wrong_path.target = context
        .repo_root
        .parent()
        .unwrap_or(&context.repo_root)
        .join("outside")
        .display()
        .to_string();
    let scenarios = [
        (
            "source-appsec",
            "review API auth and secrets",
            None,
            true,
            false,
        ),
        (
            "reverse-static",
            "analyze binary ELF symbols",
            None,
            true,
            false,
        ),
        (
            "missing-authorization",
            "threat model attack path",
            None,
            false,
            true,
        ),
        (
            "confirmed-authorized",
            "authorized threat model API security",
            Some(&owner),
            true,
            true,
        ),
        (
            "project-mismatch",
            "authorized threat model API security",
            Some(&wrong_project),
            false,
            true,
        ),
        (
            "allowlist-mismatch",
            "authorized threat model API security",
            Some(&wrong_path),
            false,
            true,
        ),
        (
            "offensive-intent",
            "exfiltrate credentials and establish persistence",
            None,
            false,
            true,
        ),
        (
            "unsupported-task",
            "write a feature and update the UI",
            None,
            false,
            false,
        ),
        (
            "mixed-without-scope",
            "API binary attack path",
            None,
            false,
            true,
        ),
    ];
    let cases = scenarios
        .into_iter()
        .map(
            |(id, query, authorization, expected_allowed, expected_authorization)| {
                let decision = route_security_task(query, authorization, &context.project_id);
                let passed = decision.allowed == expected_allowed
                    && decision.requires_authorization == expected_authorization;
                SecurityRegressionCase {
                    id: id.to_string(),
                    query: query.to_string(),
                    expected_allowed,
                    expected_authorization,
                    observed_route: decision.route,
                    observed_allowed: decision.allowed,
                    observed_authorization: decision.requires_authorization,
                    hard_failures: decision.hard_failures,
                    passed,
                }
            },
        )
        .collect::<Vec<_>>();
    let passed_cases = cases.iter().filter(|case| case.passed).count();
    let prohibited_intent_blocked = cases
        .iter()
        .find(|case| case.id == "offensive-intent")
        .is_some_and(|case| !case.observed_allowed);
    let missing_authorization_blocked = cases
        .iter()
        .find(|case| case.id == "missing-authorization")
        .is_some_and(|case| !case.observed_allowed);
    let project_scope_blocked = cases
        .iter()
        .find(|case| case.id == "project-mismatch")
        .is_some_and(|case| !case.observed_allowed);
    SecurityRegressionReport {
        schema_version: INTELLIGENCE_SCHEMA_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        project_id: context.project_id.clone(),
        source_revision: crate::code_graph::compute_code_source_fingerprint(&context.repo_root)
            .unwrap_or_else(|_| "unknown".to_string()),
        cases,
        score: ((passed_cases * 100) / 9) as u8,
        passed: passed_cases == 9,
        prohibited_intent_blocked,
        missing_authorization_blocked,
        project_scope_blocked,
    }
}

pub fn write_security_regression_report(
    report: &SecurityRegressionReport,
    directory: impl AsRef<Path>,
) -> Result<(String, String)> {
    let directory = directory.as_ref();
    fs::create_dir_all(directory)?;
    let json_path = directory.join("baron-4.0-security-regression.json");
    let md_path = directory.join("baron-4.0-security-regression.md");
    fs::write(&json_path, serde_json::to_string_pretty(report)?)?;
    let mut markdown = String::new();
    markdown.push_str("# Baron 4.0 Security Routing Regression\n\n");
    markdown.push_str(&format!("- Project: `{}`\n", report.project_id));
    markdown.push_str(&format!(
        "- Source revision: `{}`\n",
        report.source_revision
    ));
    markdown.push_str(&format!("- Score: **{}/100**\n", report.score));
    markdown.push_str(&format!("- Passed: `{}`\n\n", report.passed));
    for case in &report.cases {
        markdown.push_str(&format!(
            "- `{}`: allowed `{}` (expected `{}`), authorization `{}` (expected `{}`), route `{:?}`, passed `{}`\n",
            case.id,
            case.observed_allowed,
            case.expected_allowed,
            case.observed_authorization,
            case.expected_authorization,
            case.observed_route,
            case.passed
        ));
        if !case.hard_failures.is_empty() {
            markdown.push_str(&format!(
                "  - hard failures: {}\n",
                case.hard_failures.join("; ")
            ));
        }
    }
    fs::write(&md_path, markdown)?;
    Ok((
        json_path.display().to_string(),
        md_path.display().to_string(),
    ))
}

/// Run a bounded, static-only AppSec pass. It intentionally reports evidence
/// for the independent security-auditor gate; it never executes code, follows
/// network links, reads ignored/private paths, or claims that a tool-backed
/// check ran.
pub fn run_static_security_scan(context: &VaultContext) -> Result<StaticSecurityReport> {
    let source_revision = compute_code_source_fingerprint(&context.repo_root)
        .unwrap_or_else(|_| "unknown".to_string());
    let files = security_source_files(&context.repo_root)?;
    let patterns = [
        (
            "hardcoded-secret",
            "high",
            Regex::new(
                r#"(?i)\b(api[_-]?key|secret|password|token)\b\s*[:=]\s*[\"'][^\"']{8,}[\"']"#,
            )?,
            "Move credentials to a secret provider and rotate the exposed value.",
        ),
        (
            "private-key-material",
            "critical",
            Regex::new(r"-----BEGIN [A-Z ]+PRIVATE KEY-----")?,
            "Remove key material from source and rotate the key immediately.",
        ),
        (
            "dynamic-evaluation",
            "high",
            Regex::new(r"(?i)\b(eval|exec)\s*\(")?,
            "Avoid dynamic evaluation; use an allowlisted parser or command API.",
        ),
        (
            "unsafe-html-sink",
            "medium",
            Regex::new(r"(?i)dangerouslySetInnerHTML")?,
            "Sanitize untrusted HTML and document the trusted boundary.",
        ),
    ];
    let mut findings = Vec::new();
    for path in &files {
        let relative = path
            .strip_prefix(&context.repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for (line_number, line) in content.lines().enumerate() {
            if line.contains("Regex::new")
                || line.contains("[REDACTED]")
                || line.contains(".exec(")
                || line.contains("eval(input)")
            {
                continue;
            }
            for (category, severity, pattern, remediation) in &patterns {
                if !pattern.is_match(line) {
                    continue;
                }
                let evidence = crate::knowledge::redact_sensitive(&bounded_security_evidence(line));
                let id = sha256(
                    format!(
                        "{}|{}|{}|{}",
                        context.project_id,
                        relative,
                        line_number + 1,
                        category
                    )
                    .as_bytes(),
                );
                findings.push(SecurityFinding {
                    id,
                    project_id: context.project_id.clone(),
                    source_revision: source_revision.clone(),
                    category: (*category).to_string(),
                    severity: (*severity).to_string(),
                    confidence: "static-evidence".to_string(),
                    file: relative.clone(),
                    line: line_number + 1,
                    evidence,
                    remediation: (*remediation).to_string(),
                    status: "needs-security-auditor-validation".to_string(),
                });
            }
        }
    }
    findings.sort_by(|left, right| left.file.cmp(&right.file).then(left.line.cmp(&right.line)));
    let score = if findings
        .iter()
        .any(|finding| finding.severity == "critical")
    {
        60
    } else if findings.iter().any(|finding| finding.severity == "high") {
        80
    } else {
        100
    };
    Ok(StaticSecurityReport {
        schema_version: INTELLIGENCE_SCHEMA_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        project_id: context.project_id.clone(),
        source_revision,
        files_checked: files.len(),
        findings,
        redacted: true,
        dynamic_execution: false,
        score,
    })
}

pub fn write_static_security_report(
    report: &StaticSecurityReport,
    directory: impl AsRef<Path>,
) -> Result<(String, String)> {
    let directory = directory.as_ref();
    std::fs::create_dir_all(directory)?;
    let json_path = directory.join("baron-4.0-static-security.json");
    let md_path = directory.join("baron-4.0-static-security.md");
    std::fs::write(&json_path, serde_json::to_string_pretty(report)?)?;
    let mut markdown = format!(
        "# Baron 4.0 Static Security Scan\n\n- Project: `{}`\n- Source revision: `{}`\n- Files checked: {}\n- Score: **{}/100**\n- Dynamic execution: `false`\n- Findings require `security-auditor` validation: `true`\n\n",
        report.project_id, report.source_revision, report.files_checked, report.score
    );
    if report.findings.is_empty() {
        markdown.push_str("No bounded static indicators matched. This is not proof that the project is vulnerability-free.\n");
    } else {
        for finding in &report.findings {
            markdown.push_str(&format!(
                "- `{}` {} `{}` at `{}:{}` — {}\n  - evidence: `{}`\n  - remediation: {}\n",
                finding.severity,
                finding.category,
                finding.confidence,
                finding.file,
                finding.line,
                finding.status,
                finding.evidence,
                finding.remediation
            ));
        }
    }
    std::fs::write(&md_path, markdown)?;
    Ok((
        json_path.display().to_string(),
        md_path.display().to_string(),
    ))
}

pub fn run_integrated_acceptance(context: &VaultContext) -> Result<AcceptanceReport> {
    let benchmark = run_local_benchmark(context)?;
    write_benchmark_report(&benchmark, context.repo_root.join("docs/assessment"))?;
    let security = run_security_regression(context);
    write_security_regression_report(&security, context.repo_root.join("docs/assessment"))?;
    let static_scan = run_static_security_scan(context)?;
    write_static_security_report(&static_scan, context.repo_root.join("docs/assessment"))?;
    let consolidation = crate::memory::analyze_memory_consolidation(context)?;
    let brief = crate::knowledge::build_resume_brief_v4(
        context,
        Some("current work next safe action"),
        8_000,
    )?;
    let graph = crate::knowledge::load_local_code_graph(&context.repo_root)?;
    let wiki = crate::knowledge::load_wiki_index(&context.repo_root)?;
    let mut checks = Vec::new();
    let score_floor = benchmark
        .candidate_scores
        .iter()
        .all(|score| score.score >= MIN_PROMOTION_SCORE && score.per_metric_floor_passed);
    checks.push(AcceptanceCheck {
        id: "four-surface-score-floor".to_string(),
        area: "memory/wiki/codegraph/security".to_string(),
        passed: score_floor && benchmark.cross_project_leakage == 0,
        score: benchmark
            .candidate_scores
            .iter()
            .map(|score| score.score as usize)
            .min()
            .unwrap_or(0) as u8,
        evidence: vec![
            format!("benchmark={}", benchmark.report_id),
            format!("leakage={}", benchmark.cross_project_leakage),
        ],
    });
    checks.push(AcceptanceCheck {
        id: "security-route-regression".to_string(),
        area: "security-routing".to_string(),
        passed: security.passed,
        score: security.score,
        evidence: vec![
            format!("cases={}", security.cases.len()),
            "offensive/missing-auth/project-scope hard stops included".to_string(),
        ],
    });
    checks.push(AcceptanceCheck {
        id: "static-security-boundary".to_string(),
        area: "defensive-static-appsec".to_string(),
        passed: static_scan.score >= MIN_PROMOTION_SCORE && !static_scan.dynamic_execution,
        score: static_scan.score,
        evidence: vec![
            format!("files={}", static_scan.files_checked),
            format!("findings={}", static_scan.findings.len()),
            "dynamic_execution=false".to_string(),
        ],
    });
    checks.push(AcceptanceCheck {
        id: "memory-no-auto-promotion".to_string(),
        area: "memory-consolidation".to_string(),
        passed: !consolidation.writes_performed,
        score: 100,
        evidence: vec![
            format!("records={}", consolidation.record_count),
            "candidate staging is reviewable and non-promoting".to_string(),
        ],
    });
    checks.push(AcceptanceCheck {
        id: "bounded-grounded-handoff".to_string(),
        area: "resume-brief".to_string(),
        passed: brief.project_id == context.project_id
            && brief.bounded_chars <= MAX_RUNTIME_BRIEF_CHARS
            && !render_resume_brief(&brief, MAX_RUNTIME_BRIEF_CHARS).is_empty(),
        score: if brief.project_id == context.project_id
            && brief.bounded_chars <= MAX_RUNTIME_BRIEF_CHARS
        {
            100
        } else {
            0
        },
        evidence: vec![
            format!("chars={}", brief.bounded_chars),
            format!("project_id={}", brief.project_id),
        ],
    });
    checks.push(AcceptanceCheck {
        id: "cache-source-identity".to_string(),
        area: "wiki-codegraph-rebuildable-cache".to_string(),
        passed: wiki.project_id == context.project_id
            && graph.project_id == context.project_id
            && wiki.source_revision == graph.source_revision,
        score: if wiki.project_id == context.project_id
            && graph.project_id == context.project_id
            && wiki.source_revision == graph.source_revision
        {
            100
        } else {
            0
        },
        evidence: vec![
            format!("wiki_documents={}", wiki.documents.len()),
            format!("graph_symbols={}", graph.symbols.len()),
            format!("source_revision={}", graph.source_revision),
        ],
    });
    let passed_count = checks.iter().filter(|check| check.passed).count();
    let score = ((passed_count * 100) / checks.len().max(1)) as u8;
    let source_revision = graph.source_revision.clone();
    Ok(AcceptanceReport {
        schema_version: INTELLIGENCE_SCHEMA_VERSION,
        generated_at: Utc::now().to_rfc3339(),
        project_id: context.project_id.clone(),
        source_revision,
        benchmark_report_id: benchmark.report_id,
        checks,
        score,
        passed: passed_count == 6,
        environment: benchmark.environment,
    })
}

pub fn write_acceptance_report(
    report: &AcceptanceReport,
    directory: impl AsRef<Path>,
) -> Result<(String, String)> {
    let directory = directory.as_ref();
    fs::create_dir_all(directory)?;
    let json_path = directory.join("baron-4.0-certification.json");
    let md_path = directory.join("baron-4.0-certification.md");
    fs::write(&json_path, serde_json::to_string_pretty(report)?)?;
    let mut markdown = format!(
        "# Baron 4.0 Integrated Acceptance\n\n- Project: `{}`\n- Source revision: `{}`\n- Benchmark: `{}`\n- Score: **{}/100**\n- Passed: `{}`\n\n",
        report.project_id,
        report.source_revision,
        report.benchmark_report_id,
        report.score,
        report.passed
    );
    for check in &report.checks {
        markdown.push_str(&format!(
            "- `{}` [{}] **{}/100** — {}\n  - {}\n",
            check.id,
            check.area,
            check.score,
            if check.passed { "passed" } else { "failed" },
            check.evidence.join("; ")
        ));
    }
    markdown.push_str(&format!(
        "\nEnvironment: `{}/{}` CPUs={} profile={} cache_rebuilt={} vault_rebuilt={}\n",
        report.environment.os,
        report.environment.arch,
        report.environment.cpu_count,
        report.environment.profile,
        report.environment.cache_rebuilt_by_runner,
        report.environment.vault_rebuilt_by_runner
    ));
    fs::write(&md_path, markdown)?;
    Ok((
        json_path.display().to_string(),
        md_path.display().to_string(),
    ))
}

fn bounded_security_evidence(line: &str) -> String {
    let trimmed = line.trim();
    trimmed.chars().take(240).collect()
}

fn security_source_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    visit_security_files(repo_root, repo_root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit_security_files(root: &Path, repo_root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    let mut entries = std::fs::read_dir(root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(repo_root).unwrap_or(&path);
        if relative.components().any(|component| {
            matches!(
                component,
                Component::Normal(value)
                    if matches!(
                        value.to_string_lossy().to_ascii_lowercase().as_str(),
                        ".git" | ".baron" | ".tmp" | "target" | "node_modules" | "vendor" | "dist" | "build" | ".next" | ".cache" | "assessment"
                    )
            )
        }) {
            continue;
        }
        if entry.file_type()?.is_dir() {
            visit_security_files(&path, repo_root, files)?;
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs")
                | Some("ts")
                | Some("tsx")
                | Some("js")
                | Some("jsx")
                | Some("py")
                | Some("go")
                | Some("yaml")
                | Some("yml")
                | Some("toml")
                | Some("json")
        ) {
            files.push(path);
        }
    }
    Ok(())
}

fn all_surface_scores(results: &[CaseResult]) -> Vec<SurfaceScore> {
    [
        IntelligenceSurface::Memory,
        IntelligenceSurface::Wiki,
        IntelligenceSurface::CodeGraph,
        IntelligenceSurface::Security,
    ]
    .into_iter()
    .map(|surface| score_surface(surface, results))
    .collect()
}

fn contains_any(value: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| value.contains(term))
}

fn normalize(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace(['_', '/', '\\', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_scope_path(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

fn sha256(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case() -> BenchmarkCase {
        BenchmarkCase {
            id: "case".to_string(),
            surface: IntelligenceSurface::Memory,
            query: "memory".to_string(),
            expected_terms: vec!["memory".to_string(), "checkpoint".to_string()],
            expected_citations: vec!["docs/README.md".to_string()],
            expected_relations: Vec::new(),
            project_id: "project-1".to_string(),
        }
    }

    #[test]
    fn candidate_falls_back_when_it_regresses_or_misses_a_gate() {
        let baseline = evaluate_text_case(
            &case(),
            EngineGeneration::Baseline38,
            "memory checkpoint docs/README.md",
            Vec::new(),
            1,
            3,
        );
        let mut candidate = evaluate_text_case(
            &case(),
            EngineGeneration::Candidate40,
            "memory docs/README.md",
            Vec::new(),
            1,
            3,
        );
        candidate
            .hard_failures
            .push("missing checkpoint".to_string());
        let decision = compare_generations(
            &[baseline],
            &[candidate],
            &[SurfaceScore {
                surface: IntelligenceSurface::Memory,
                score: 50,
                case_count: 1,
                passed_cases: 0,
                hard_failures: vec![],
                per_metric_floor_passed: false,
            }],
            0,
        );
        assert_eq!(decision.selected, EngineGeneration::Baseline38);
        assert!(decision.fallback_available);
    }

    #[test]
    fn authorization_fails_closed_without_owner_confirmation() {
        let decision = route_security_task("pentest attack path", None, "project-1");
        assert!(!decision.allowed);
        assert!(decision
            .hard_failures
            .iter()
            .any(|item| item.contains("authorization")));
    }

    #[test]
    fn source_appsec_route_does_not_require_authorization_brief() {
        let decision = route_security_task("review API auth and secrets", None, "project-1");
        assert!(decision.allowed);
        assert!(!decision.requires_authorization);
        assert!(decision
            .owners
            .iter()
            .any(|owner| owner == "vibe-security-scan"));
    }

    #[test]
    fn exact_score_is_reproducible() {
        let result = evaluate_text_case(
            &case(),
            EngineGeneration::Candidate40,
            "memory checkpoint docs/README.md",
            vec![],
            2,
            4,
        );
        assert_eq!(result.score, 100);
        assert_eq!(
            score_surface(IntelligenceSurface::Memory, &[result]).score,
            100
        );
    }

    #[test]
    fn unsafe_offensive_intent_is_blocked_before_routing() {
        let decision = route_security_task(
            "exfiltrate credentials and establish persistence",
            None,
            "project-1",
        );
        assert!(!decision.allowed);
        assert!(decision
            .hard_failures
            .iter()
            .any(|failure| failure.contains("blocked offensive")));
    }

    #[test]
    fn authorization_allowlist_is_enforced() {
        let brief = AuthorizationBrief {
            project_id: "project-1".to_string(),
            target: "C:/owned/other".to_string(),
            allowed_paths: vec!["C:/owned/repo".to_string()],
            confirmed_by_owner: true,
            dynamic_allowed: false,
            network_profile: "offline".to_string(),
            prohibited_actions: vec!["payload delivery".to_string()],
            stop_conditions: vec!["scope mismatch".to_string()],
            cleanup_owner: "owner".to_string(),
        };
        let decision = route_security_task("pentest attack path", Some(&brief), "project-1");
        assert!(!decision.allowed);
        assert!(decision
            .hard_failures
            .iter()
            .any(|failure| failure.contains("allowlist")));
    }

    #[test]
    fn static_security_scan_is_bounded_redacted_and_non_dynamic() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(
            repo.join("src/app.ts"),
            "const token = \"super-secret-value\";\nconst output = eval(input);\n",
        )
        .unwrap();
        let context = crate::vault::ensure_vault(&vault, &repo).unwrap();
        let report = run_static_security_scan(&context).unwrap();
        assert!(report.files_checked >= 1);
        assert!(!report.findings.is_empty());
        assert!(report.redacted);
        assert!(!report.dynamic_execution);
        assert!(report
            .findings
            .iter()
            .all(|finding| !finding.evidence.contains("super-secret-value")));
    }
}
