//! Baron 4.1 benchmark contract and reproducible local evidence.
//!
//! The contract deliberately treats Tencent as an external comparison input.
//! A missing runner or baseline is a hard `target_not_achieved` result, never a
//! fabricated score.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{bail, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::code_graph::compute_code_source_fingerprint;
use crate::firewall::recall_v5;
use crate::intelligence41::{
    analyze_graph_impact, build_grounded_handoff, learn_session_candidates, refresh_temporal_ledger,
};
use crate::knowledge::{
    build_local_code_graph, index_wiki, search_local_code_graph_v5, search_wiki_v5,
};
use crate::memory::build_memory_index;
use crate::vault::VaultContext;

pub const EVALUATION41_SCHEMA_VERSION: u32 = 1;
pub const TARGET_SCORE: u8 = 95;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkContract41 {
    pub schema_version: u32,
    pub contract_id: String,
    pub generated_at: String,
    pub source_revision: String,
    pub fixture_revision: String,
    pub holdout_hash: String,
    pub surfaces: Vec<String>,
    pub allowed_context_tokens: usize,
    pub time_budget_ms: u64,
    pub peak_memory_budget_bytes: u64,
    pub cost_normalized: bool,
    pub tencent_target: String,
    pub tencent_revision: String,
    pub fixture_cases: Vec<BenchmarkCase41>,
    pub holdout_case_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkCase41 {
    pub id: String,
    pub surface: String,
    pub query: String,
    pub expected_signals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceEvidence41 {
    pub surface: String,
    pub score: u8,
    pub cases: usize,
    pub passed_cases: usize,
    pub evidence: Vec<String>,
    pub hard_failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TencentEvidence41 {
    pub status: String,
    pub source: Option<String>,
    pub revision: Option<String>,
    pub scores: BTreeMap<String, u8>,
    #[serde(default)]
    pub same_corpus: bool,
    #[serde(default)]
    pub confidence_95: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkReport41 {
    pub schema_version: u32,
    pub report_id: String,
    pub generated_at: String,
    pub contract: BenchmarkContract41,
    pub project_id: String,
    pub source_revision: String,
    pub baron_scores: Vec<SurfaceEvidence41>,
    pub tencent: TencentEvidence41,
    pub same_corpus_win: bool,
    pub statistical_confidence_95: bool,
    pub repetitions: usize,
    pub metrics: BenchmarkMetrics41,
    pub target_achieved: bool,
    pub hard_failures: Vec<String>,
    pub json_path: String,
    pub markdown_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkMetrics41 {
    pub execution_profile: String,
    pub elapsed_ms: u128,
    pub index_elapsed_ms: u128,
    pub query_elapsed_ms: u128,
    pub estimated_tokens: usize,
    pub cache_bytes: u64,
    pub peak_memory_bytes: Option<u64>,
    pub cost_status: String,
}

pub fn benchmark_contract_path(repo_root: &Path) -> PathBuf {
    repo_root.join("docs/assessment/baron-4.1-contract.json")
}

pub fn benchmark_report_path(repo_root: &Path) -> PathBuf {
    repo_root.join("docs/assessment/baron-4.1-benchmark.json")
}

pub fn freeze_contract(context: &VaultContext) -> Result<(BenchmarkContract41, String, String)> {
    let json_path = benchmark_contract_path(&context.repo_root);
    let markdown_path = json_path.with_extension("md");
    let refreeze = std::env::var("BARON_41_REFREEZE_CONTRACT")
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
    if json_path.is_file() && !refreeze {
        let content = fs::read_to_string(&json_path)?;
        let contract: BenchmarkContract41 = serde_json::from_str(&content)?;
        validate_frozen_contract(&contract, &context.project_id)?;
        if !markdown_path.is_file() {
            fs::write(&markdown_path, format_contract_markdown(&contract))?;
        }
        return Ok((
            contract,
            json_path.display().to_string(),
            markdown_path.display().to_string(),
        ));
    }
    let source_revision = compute_code_source_fingerprint(&context.repo_root)
        .unwrap_or_else(|_| "unknown".to_string());
    let fixture_cases = development_fixture_cases();
    let holdout_cases = holdout_case_ids();
    let fixture_revision = sha256(serde_json::to_vec(&fixture_cases)?);
    let holdout_hash = sha256(serde_json::to_vec(&holdout_cases)?);
    let mut contract = BenchmarkContract41 {
        schema_version: EVALUATION41_SCHEMA_VERSION,
        contract_id: String::new(),
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        source_revision,
        fixture_revision,
        holdout_hash,
        surfaces: vec![
            "long_term_memory_l0_l3".to_string(),
            "semantic_retrieval_grounded_synthesis".to_string(),
            "automatic_session_learning".to_string(),
            "wiki".to_string(),
            "codegraph".to_string(),
        ],
        allowed_context_tokens: 8_000,
        time_budget_ms: 10_000,
        peak_memory_budget_bytes: 512 * 1024 * 1024,
        cost_normalized: true,
        tencent_target: "TencentDB-Agent-Memory".to_string(),
        tencent_revision:
            "v2.0.0@0aff21a2d9f2b8a0354aaa80a2e586aab4054562 (surface baseline still required)"
                .to_string(),
        fixture_cases,
        holdout_case_count: holdout_cases.len(),
    };
    contract.contract_id = frozen_contract_id(&contract, &context.project_id)?;
    validate_frozen_contract(&contract, &context.project_id)?;
    write_json_atomic(&json_path, &contract)?;
    let markdown = format_contract_markdown(&contract);
    fs::write(&markdown_path, markdown)?;
    Ok((
        contract,
        json_path.display().to_string(),
        markdown_path.display().to_string(),
    ))
}

fn validate_frozen_contract(contract: &BenchmarkContract41, project_id: &str) -> Result<()> {
    if contract.schema_version != EVALUATION41_SCHEMA_VERSION {
        bail!(
            "Frozen 4.1 contract schema {} is unsupported; create an explicit new contract",
            contract.schema_version
        );
    }
    let required_surfaces = [
        "long_term_memory_l0_l3",
        "semantic_retrieval_grounded_synthesis",
        "automatic_session_learning",
        "wiki",
        "codegraph",
    ];
    if contract.surfaces.len() != required_surfaces.len()
        || !required_surfaces
            .iter()
            .all(|surface| contract.surfaces.iter().any(|value| value == surface))
    {
        bail!("Frozen 4.1 contract does not contain the required five surfaces");
    }
    if contract.fixture_cases.is_empty() || contract.holdout_case_count == 0 {
        bail!("Frozen 4.1 contract must contain development and holdout cases");
    }
    if contract.holdout_hash.len() != 64 || contract.source_revision.len() != 64 {
        bail!("Frozen 4.1 contract has an invalid source or holdout fingerprint");
    }
    let expected_id = frozen_contract_id(contract, project_id)?;
    if contract.contract_id != expected_id {
        bail!(
            "Frozen 4.1 contract integrity mismatch; set BARON_41_REFREEZE_CONTRACT=1 to create a new reviewed contract"
        );
    }
    Ok(())
}

fn frozen_contract_id(contract: &BenchmarkContract41, project_id: &str) -> Result<String> {
    let payload = (
        project_id,
        contract.schema_version,
        &contract.source_revision,
        &contract.fixture_revision,
        &contract.holdout_hash,
        &contract.surfaces,
        contract.allowed_context_tokens,
        contract.time_budget_ms,
        contract.peak_memory_budget_bytes,
        contract.cost_normalized,
        &contract.tencent_target,
        &contract.tencent_revision,
        &contract.fixture_cases,
        contract.holdout_case_count,
    );
    Ok(sha256(serde_json::to_vec(&payload)?))
}

pub fn run_benchmark(context: &VaultContext) -> Result<BenchmarkReport41> {
    let started = Instant::now();
    let (contract, _, _) = freeze_contract(context)?;
    let query_for = |surface: &str, fallback: &str| {
        contract
            .fixture_cases
            .iter()
            .find(|case| case.surface == surface)
            .map(|case| case.query.clone())
            .unwrap_or_else(|| fallback.to_string())
    };
    let index_started = Instant::now();
    build_memory_index(context)?;
    let _ = index_wiki(&context.repo_root)?;
    let graph = build_local_code_graph(&context.repo_root)?;
    let index_elapsed_ms = index_started.elapsed().as_millis();
    let query_started = Instant::now();
    let mut scores = Vec::new();

    let memory_query = query_for(
        "long_term_memory_l0_l3",
        "current work proof next safe action",
    );
    let memory = recall_v5(context, &memory_query, 8)?;
    scores.push(score_surface_checks(
        "long_term_memory_l0_l3",
        vec![
            ("current-project memory hit", !memory.results.is_empty()),
            (
                "all returned memory stays project-scoped or approved-global",
                memory.results.iter().all(|hit| {
                    hit.record.project_id.as_deref() == Some(context.project_id.as_str())
                        || hit.record.scope == crate::memory::MemoryScope::GlobalVerified
                }),
            ),
            (
                "cross-project candidate is blocked and measured",
                memory.blocked_cross_project > 0,
            ),
        ],
        vec![format!("memory_hits={}", memory.results.len())],
        Vec::new(),
    ));

    let semantic_query = query_for(
        "semantic_retrieval_grounded_synthesis",
        "tìm kiếm ngữ nghĩa memory retrieval",
    );
    let retrieval = recall_v5(context, &semantic_query, 8)?;
    let retrieval_english = recall_v5(context, "semantic retrieval", 8)?;
    let handoff = build_grounded_handoff(context, Some(&semantic_query), 8_000)?;
    scores.push(score_surface_checks(
        "semantic_retrieval_grounded_synthesis",
        vec![
            (
                "Vietnamese semantic query returns memory",
                !retrieval.results.is_empty(),
            ),
            (
                "English semantic query returns memory",
                !retrieval_english.results.is_empty(),
            ),
            (
                "ranker leaves explainable v5 evidence",
                retrieval
                    .results
                    .iter()
                    .any(|hit| hit.notes.iter().any(|note| note.starts_with("v5-"))),
            ),
            (
                "grounded handoff contains cited bounded claims",
                !handoff.claims.is_empty()
                    && handoff
                        .claims
                        .iter()
                        .all(|claim| !claim.citation.is_empty() && !claim.evidence_hash.is_empty())
                    && handoff.bounded_chars <= 8_000,
            ),
        ],
        retrieval
            .results
            .iter()
            .take(4)
            .map(|hit| format!("{}:{}", hit.record.path, hit.score))
            .chain(std::iter::once(format!(
                "handoff_claims={}",
                handoff.claims.len()
            )))
            .collect(),
        Vec::new(),
    ));

    let learning = learn_session_candidates(context)?;
    let quarantined = learning
        .candidates
        .iter()
        .filter(|candidate| !candidate.risk_flags.is_empty())
        .count();
    scores.push(score_surface_checks(
        "automatic_session_learning",
        vec![
            ("at least one imported session source", learning.sources > 0),
            ("session messages were parsed", learning.messages > 0),
            (
                "evidence-linked learning candidates were extracted",
                !learning.candidates.is_empty(),
            ),
            (
                "candidates carry L0-L3 proposal and dedup identity",
                learning.candidates.iter().all(|candidate| {
                    matches!(
                        candidate.layer.as_str(),
                        "L0Evidence"
                            | "L1FactCandidate"
                            | "L2DecisionCandidate"
                            | "L3InvariantCandidate"
                    ) && !candidate.dedup_key.is_empty()
                }),
            ),
            (
                "learning remains candidate-only and never creates Skills",
                learning.skills_created == 0
                    && learning
                        .candidates
                        .iter()
                        .all(|candidate| !candidate.approved),
            ),
            (
                "risky learning candidates are quarantined",
                learning
                    .candidates
                    .iter()
                    .filter(|candidate| !candidate.risk_flags.is_empty())
                    .all(|candidate| candidate.confidence == "quarantined"),
            ),
        ],
        vec![
            format!("sources={}", learning.sources),
            format!("messages={}", learning.messages),
            format!("candidates={}", learning.candidates.len()),
            format!("quarantined={quarantined}"),
            "skills_created=0".to_string(),
        ],
        Vec::new(),
    ));

    let (_, temporal) = refresh_temporal_ledger(context)?;

    let graph_query = query_for("codegraph", "context memory");
    let graph_hits = search_local_code_graph_v5(&context.repo_root, &graph_query, 8)?;
    let impact = analyze_graph_impact(&graph, &graph_query, 8);
    scores.push(score_surface_checks(
        "codegraph",
        vec![
            ("repository files are discovered", !graph.files.is_empty()),
            ("symbols are extracted", !graph.symbols.is_empty()),
            ("typed relation edges are present", !graph.edges.is_empty()),
            (
                "semantic CodeGraph query returns symbols",
                !graph_hits.is_empty(),
            ),
            (
                "bounded impact traversal returns paths",
                !impact.paths.is_empty(),
            ),
            (
                "edge budget is enforced",
                graph.edges.len() <= crate::knowledge::MAX_GRAPH_EDGES,
            ),
        ],
        vec![
            format!("files={}", graph.files.len()),
            format!("symbols={}", graph.symbols.len()),
            format!("edges={}", graph.edges.len()),
            format!("impact_paths={}", impact.paths.len()),
        ],
        Vec::new(),
    ));

    // Force the Wiki path into the report even when a fixture repository has no
    // matching document. A missing hit is evidence, not a hidden success.
    let wiki_query = query_for("wiki", "memory architecture");
    let wiki_hits = search_wiki_v5(&context.repo_root, &wiki_query, 8)?;
    scores.push(score_surface_checks(
        "wiki",
        vec![
            ("Wiki query returns sections", !wiki_hits.is_empty()),
            (
                "Wiki returns more than one grounded section",
                wiki_hits.len() >= 2,
            ),
            (
                "Wiki retains link/entity evidence",
                wiki_hits
                    .iter()
                    .any(|hit| !hit.links.is_empty() || !hit.entities.is_empty()),
            ),
            (
                "Wiki citations remain project-relative",
                wiki_hits
                    .iter()
                    .all(|hit| !hit.document.starts_with('/') && !hit.document.contains("..")),
            ),
        ],
        vec![
            format!("wiki_hits={}", wiki_hits.len()),
            format!("temporal_active={}", temporal.active),
            format!("temporal_superseded={}", temporal.superseded),
        ],
        Vec::new(),
    ));

    let tencent = load_tencent_baseline();
    let mut hard_failures = scores
        .iter()
        .flat_map(|score| score.hard_failures.clone())
        .collect::<Vec<_>>();
    for score in &scores {
        if score.score < TARGET_SCORE {
            hard_failures.push(format!(
                "{} score {} is below {}",
                score.surface, score.score, TARGET_SCORE
            ));
        }
    }
    let (repetitions, statistical_confidence_95, confidence_detail) =
        load_confidence_evidence(&contract);
    let same_corpus_win = tencent.status == "available"
        && tencent.same_corpus
        && tencent.confidence_95
        && statistical_confidence_95
        && scores.iter().all(|score| {
            tencent
                .scores
                .get(&score.surface)
                .is_some_and(|remote| score.score >= remote.saturating_add(2))
        });
    if tencent.status != "available" {
        hard_failures
            .push("Tencent baseline is unavailable; comparison cannot claim a win".to_string());
    }
    if !same_corpus_win {
        hard_failures.push("Baron did not prove a same-corpus Tencent win".to_string());
    }
    if !statistical_confidence_95 {
        hard_failures.push(format!(
            "Independent repeated-run confidence evidence is missing or invalid: {confidence_detail}"
        ));
    }
    let metrics = BenchmarkMetrics41 {
        execution_profile: if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        },
        elapsed_ms: started.elapsed().as_millis(),
        index_elapsed_ms,
        query_elapsed_ms: query_started.elapsed().as_millis(),
        estimated_tokens: handoff.estimated_tokens,
        cache_bytes: measure_cache_bytes(context),
        peak_memory_bytes: measure_peak_memory_bytes(),
        cost_status: if handoff.cost_status == "within_budget" {
            "within_context_budget".to_string()
        } else {
            "over_context_budget".to_string()
        },
    };
    // Phase 86 is a release gate, not a feature smoke test. Keep runtime and
    // observability budgets as hard failures so a high functional score cannot
    // be promoted while the measured run is too slow, unobservable, or over
    // the context/cost envelope.
    if metrics.query_elapsed_ms > contract.time_budget_ms as u128 {
        hard_failures.push(format!(
            "benchmark query time {} ms exceeds {} ms budget (total={}, index={})",
            metrics.query_elapsed_ms,
            contract.time_budget_ms,
            metrics.elapsed_ms,
            metrics.index_elapsed_ms
        ));
    }
    if metrics.peak_memory_bytes.is_none() {
        hard_failures.push(
            "peak process memory measurement is unavailable; independent resource evidence is required"
                .to_string(),
        );
    }
    if let Some(peak_memory_bytes) = metrics.peak_memory_bytes {
        if peak_memory_bytes > contract.peak_memory_budget_bytes {
            hard_failures.push(format!(
                "peak process memory {} bytes exceeds {} byte budget",
                peak_memory_bytes, contract.peak_memory_budget_bytes
            ));
        }
    }
    if !metrics.cost_status.starts_with("within_context_budget") {
        hard_failures.push(format!(
            "context/cost budget was not met: {}",
            metrics.cost_status
        ));
    }
    let target_achieved = hard_failures.is_empty();
    let report_id = sha256(
        format!(
            "{}|{}|{}|{}",
            context.project_id,
            contract.contract_id,
            serde_json::to_string(&scores)?,
            target_achieved
        )
        .as_bytes(),
    );
    let json_path = benchmark_report_path(&context.repo_root);
    let markdown_path = json_path.with_extension("md");
    let mut report = BenchmarkReport41 {
        schema_version: EVALUATION41_SCHEMA_VERSION,
        report_id,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        contract,
        project_id: context.project_id.clone(),
        source_revision: graph.source_revision,
        baron_scores: scores,
        tencent,
        same_corpus_win,
        statistical_confidence_95,
        repetitions,
        metrics,
        target_achieved,
        hard_failures,
        json_path: json_path.display().to_string(),
        markdown_path: markdown_path.display().to_string(),
    };
    write_json_atomic(&json_path, &report)?;
    fs::write(&markdown_path, format_report_markdown(&report))?;
    report.json_path = json_path.display().to_string();
    Ok(report)
}

fn measure_cache_bytes(context: &VaultContext) -> u64 {
    fn walk(path: &Path) -> u64 {
        let Ok(entries) = fs::read_dir(path) else {
            return 0;
        };
        entries
            .flatten()
            .map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path)
                } else {
                    entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
                }
            })
            .sum()
    }
    walk(&context.repo_root.join(".baron/cache")) + walk(&context.baron_artifacts_root)
}

/// Return the current process high-water resident memory where the host makes
/// that measurement available. A missing value is deliberately surfaced as a
/// benchmark hard failure rather than silently treated as zero.
fn measure_peak_memory_bytes() -> Option<u64> {
    #[cfg(windows)]
    {
        use std::ffi::c_void;

        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }

        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> *mut c_void;
        }
        #[link(name = "psapi")]
        extern "system" {
            fn GetProcessMemoryInfo(
                process: *mut c_void,
                counters: *mut ProcessMemoryCounters,
                size: u32,
            ) -> i32;
        }

        let mut counters = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            pagefile_usage: 0,
            peak_pagefile_usage: 0,
        };
        // SAFETY: both pointers refer to valid process-owned objects and the
        // Windows API writes no more than the declared structure size.
        let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
        (ok != 0).then_some(counters.peak_working_set_size as u64)
    }

    #[cfg(unix)]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
        // SAFETY: `usage` points to writable storage sized for libc::rusage.
        let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
        if result != 0 {
            return None;
        }
        // SAFETY: getrusage returned success and initialized the structure.
        let usage = unsafe { usage.assume_init() };
        let max_rss = usage.ru_maxrss as i128;
        if max_rss <= 0 {
            return None;
        }
        #[cfg(target_os = "macos")]
        {
            return u64::try_from(max_rss).ok();
        }
        #[cfg(not(target_os = "macos"))]
        {
            return u64::try_from(max_rss.saturating_mul(1024)).ok();
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn score_surface_checks(
    surface: &str,
    checks: Vec<(&str, bool)>,
    evidence: Vec<String>,
    mut hard_failures: Vec<String>,
) -> SurfaceEvidence41 {
    let cases = checks.len();
    let passed_cases = checks.iter().filter(|(_, passed)| *passed).count();
    for (label, passed) in checks {
        if !passed {
            hard_failures.push(format!("{surface} check failed: {label}"));
        }
    }
    let score = passed_cases
        .saturating_mul(100)
        .checked_div(cases.max(1))
        .unwrap_or_default() as u8;
    SurfaceEvidence41 {
        surface: surface.to_string(),
        score,
        cases,
        passed_cases,
        evidence,
        hard_failures,
    }
}

fn load_tencent_baseline() -> TencentEvidence41 {
    let Some(path) = std::env::var_os("BARON_TENCENT_BASELINE_JSON") else {
        return TencentEvidence41 {
            status: "unavailable".to_string(),
            source: None,
            revision: None,
            scores: BTreeMap::new(),
            same_corpus: false,
            confidence_95: false,
            detail: "Set BARON_TENCENT_BASELINE_JSON to a reviewed same-corpus score file; no implicit Tencent score is invented.".to_string(),
        };
    };
    let path = PathBuf::from(path);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            return TencentEvidence41 {
                status: "unavailable".to_string(),
                source: Some(path.display().to_string()),
                revision: None,
                scores: BTreeMap::new(),
                same_corpus: false,
                confidence_95: false,
                detail: format!("Could not read Tencent baseline: {error}"),
            }
        }
    };
    #[derive(Deserialize)]
    struct Input {
        scores: BTreeMap<String, u8>,
        revision: Option<String>,
        same_corpus: Option<bool>,
        confidence_95: Option<bool>,
    }
    match serde_json::from_str::<Input>(&content) {
        Ok(input)
            if required_tencent_surfaces_present(&input.scores)
                && input
                    .revision
                    .as_deref()
                    .is_some_and(|revision| revision.contains("v2.0.0"))
                && input.same_corpus == Some(true)
                && input.confidence_95 == Some(true) => TencentEvidence41 {
            status: "available".to_string(),
            source: Some(path.display().to_string()),
            revision: input.revision,
            scores: input.scores,
            same_corpus: true,
            confidence_95: true,
            detail: "Loaded from an explicit reviewed v2.0.0 same-corpus baseline with 95% confidence evidence.".to_string(),
        },
        Ok(input) if !input.scores.is_empty() => TencentEvidence41 {
            status: "unavailable".to_string(),
            source: Some(path.display().to_string()),
            revision: input.revision,
            scores: input.scores,
            same_corpus: input.same_corpus.unwrap_or(false),
            confidence_95: input.confidence_95.unwrap_or(false),
            detail: "Tencent baseline was rejected: require all five surfaces, revision v2.0.0, same_corpus=true, and confidence_95=true.".to_string(),
        },
        Ok(_) => TencentEvidence41 {
            status: "unavailable".to_string(),
            source: Some(path.display().to_string()),
            revision: None,
            scores: BTreeMap::new(),
            same_corpus: false,
            confidence_95: false,
            detail: "Tencent baseline file contained no scores.".to_string(),
        },
        Err(error) => TencentEvidence41 {
            status: "unavailable".to_string(),
            source: Some(path.display().to_string()),
            revision: None,
            scores: BTreeMap::new(),
            same_corpus: false,
            confidence_95: false,
            detail: format!("Could not parse Tencent baseline: {error}"),
        },
    }
}

fn load_confidence_evidence(contract: &BenchmarkContract41) -> (usize, bool, String) {
    let Some(path) = std::env::var_os("BARON_41_CONFIDENCE_EVIDENCE_JSON") else {
        return (
            1,
            false,
            "Set BARON_41_CONFIDENCE_EVIDENCE_JSON to an independently scored repeated-run file"
                .to_string(),
        );
    };
    let path = PathBuf::from(path);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => return (1, false, format!("could not read evidence: {error}")),
    };
    #[derive(Deserialize)]
    struct Input {
        contract_id: String,
        source_revision: String,
        repetitions: usize,
        confidence_95: bool,
    }
    match serde_json::from_str::<Input>(&content) {
        Ok(input)
            if input.contract_id == contract.contract_id
                && input.source_revision == contract.source_revision
                && input.repetitions >= 3
                && input.confidence_95 =>
        {
            (
                input.repetitions,
                true,
                format!("validated independent evidence from {}", path.display()),
            )
        }
        Ok(input) => (
            input.repetitions,
            false,
            "evidence contract/source mismatch, fewer than three repetitions, or confidence_95=false"
                .to_string(),
        ),
        Err(error) => (1, false, format!("could not parse evidence: {error}")),
    }
}

fn required_tencent_surfaces_present(scores: &BTreeMap<String, u8>) -> bool {
    [
        "long_term_memory_l0_l3",
        "semantic_retrieval_grounded_synthesis",
        "automatic_session_learning",
        "wiki",
        "codegraph",
    ]
    .iter()
    .all(|surface| scores.contains_key(*surface))
        && scores.values().all(|score| *score <= 100)
}

fn format_contract_markdown(contract: &BenchmarkContract41) -> String {
    let cases = contract
        .fixture_cases
        .iter()
        .map(|case| format!("{} ({})", case.id, case.surface))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "# Baron 4.1 Benchmark Contract\n\n- Contract: `{}`\n- Source revision: `{}`\n- Fixture revision: `{}`\n- Holdout hash: `{}`\n- Tencent target: `{}` `{}`\n- Surfaces: {}\n- Context token budget: {}\n- Time budget: {} ms\n- Peak memory budget: {} bytes\n- Cost normalized: `{}`\n\nThe holdout is hash-sealed and must not be used for tuning. A missing Tencent runner or baseline is a hard `target_not_achieved` result.\n",
        contract.contract_id,
        contract.source_revision,
        contract.fixture_revision,
        contract.holdout_hash,
        contract.tencent_target,
        contract.tencent_revision,
        contract.surfaces.join(", "),
        contract.allowed_context_tokens,
        contract.time_budget_ms,
        contract.peak_memory_budget_bytes,
        contract.cost_normalized
    )
    + &format!(
        "\n## Development fixtures\n\n- Cases: {}\n- Sealed holdout cases: {}\n",
        cases, contract.holdout_case_count
    )
}

fn format_report_markdown(report: &BenchmarkReport41) -> String {
    let mut output = format!(
        "# Baron 4.1 Benchmark\n\n- Report: `{}`\n- Contract: `{}`\n- Project: `{}`\n- Source revision: `{}`\n- Target achieved: `{}`\n- Same-corpus Tencent win: `{}`\n- Statistical confidence 95%: `{}`\n- Repetitions: `{}`\n\n",
        report.report_id,
        report.contract.contract_id,
        report.project_id,
        report.source_revision,
        report.target_achieved,
        report.same_corpus_win,
        report.statistical_confidence_95,
        report.repetitions
    );
    output.push_str("## Baron surfaces\n\n");
    for score in &report.baron_scores {
        output.push_str(&format!(
            "- {}: **{}/100** ({} / {} cases)\n",
            score.surface, score.score, score.passed_cases, score.cases
        ));
        for evidence in &score.evidence {
            output.push_str(&format!("  - evidence: {evidence}\n"));
        }
        for failure in &score.hard_failures {
            output.push_str(&format!("  - hard failure: {failure}\n"));
        }
    }
    output.push_str(&format!(
        "\n## Tencent baseline\n\n- Status: `{}`\n- Revision: `{}`\n- Same corpus: `{}`\n- 95% confidence evidence: `{}`\n- Detail: {}\n",
        report.tencent.status,
        report.tencent.revision.as_deref().unwrap_or("unknown"),
        report.tencent.same_corpus,
        report.tencent.confidence_95,
        report.tencent.detail
    ));
    output.push_str(&format!(
        "\n## Local run metrics\n\n- Execution profile: `{}`\n- Total elapsed: `{}` ms\n- Index elapsed: `{}` ms\n- Query elapsed: `{}` ms\n- Estimated tokens: `{}`\n- Cache/artifact bytes: `{}`\n- Peak memory: `{}`\n- Cost status: `{}`\n",
        report.metrics.execution_profile,
        report.metrics.elapsed_ms,
        report.metrics.index_elapsed_ms,
        report.metrics.query_elapsed_ms,
        report.metrics.estimated_tokens,
        report.metrics.cache_bytes,
        report
            .metrics
            .peak_memory_bytes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not measured".to_string()),
        report.metrics.cost_status
    ));
    if !report.hard_failures.is_empty() {
        output.push_str("\n## Hard failures\n\n");
        for failure in &report.hard_failures {
            output.push_str(&format!("- {failure}\n"));
        }
    }
    output
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("baron-tmp");
    fs::write(&temp, serde_json::to_string_pretty(value)?)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp, path)?;
    Ok(())
}

fn sha256(value: impl AsRef<[u8]>) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_ref());
    format!("{:x}", digest.finalize())
}

fn development_fixture_cases() -> Vec<BenchmarkCase41> {
    vec![
        BenchmarkCase41 {
            id: "memory-l0-l3-resume".to_string(),
            surface: "long_term_memory_l0_l3".to_string(),
            query: "current work proof next safe action".to_string(),
            expected_signals: vec![
                "project_identity".to_string(),
                "layer_l0_l3".to_string(),
                "trust_and_freshness".to_string(),
            ],
        },
        BenchmarkCase41 {
            id: "semantic-vietnamese-english".to_string(),
            surface: "semantic_retrieval_grounded_synthesis".to_string(),
            query: "tìm kiếm ngữ nghĩa memory retrieval".to_string(),
            expected_signals: vec![
                "bm25".to_string(),
                "vector".to_string(),
                "rrf".to_string(),
                "citation".to_string(),
            ],
        },
        BenchmarkCase41 {
            id: "session-evidence-candidates".to_string(),
            surface: "automatic_session_learning".to_string(),
            query: "session decision proof blocker next action".to_string(),
            expected_signals: vec![
                "source_hash".to_string(),
                "evidence_span".to_string(),
                "candidate_only".to_string(),
            ],
        },
        BenchmarkCase41 {
            id: "wiki-entities-links".to_string(),
            surface: "wiki".to_string(),
            query: "memory architecture".to_string(),
            expected_signals: vec![
                "entities".to_string(),
                "typed_links".to_string(),
                "citation".to_string(),
            ],
        },
        BenchmarkCase41 {
            id: "codegraph-impact".to_string(),
            surface: "codegraph".to_string(),
            query: "context memory".to_string(),
            expected_signals: vec![
                "symbols".to_string(),
                "calls".to_string(),
                "references".to_string(),
                "impact_paths".to_string(),
            ],
        },
    ]
}

fn holdout_case_ids() -> Vec<String> {
    [
        "stale-conflict",
        "renamed-deleted",
        "vietnamese-english",
        "rust-ts-js-py-go",
        "same-name-project-isolation",
    ]
    .iter()
    .map(|value| (*value).to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{initialize_project, AdapterKind};
    use crate::vault::ensure_vault;
    use std::path::Path;
    use tempfile::tempdir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn frozen_contract_is_hash_sealed_and_lists_five_surfaces() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        write(&repo.join("src/lib.rs"), "pub fn entry() {}\n");
        let (contract, json, markdown) = freeze_contract(&context).unwrap();
        assert_eq!(contract.surfaces.len(), 5);
        assert_eq!(contract.fixture_cases.len(), 5);
        assert_eq!(contract.holdout_case_count, 5);
        assert_eq!(
            contract.holdout_hash,
            sha256(serde_json::to_vec(&holdout_case_ids()).unwrap())
        );
        assert!(Path::new(&json).exists());
        assert!(Path::new(&markdown).exists());
    }

    #[test]
    fn frozen_contract_is_not_rewritten_without_explicit_refreeze() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        write(&repo.join("src/lib.rs"), "pub fn before() {}\n");
        let (first, _, _) = freeze_contract(&context).unwrap();
        write(
            &repo.join("src/lib.rs"),
            "pub fn after() { println!(\"source changed\"); }\n",
        );
        let (second, _, _) = freeze_contract(&context).unwrap();
        assert_eq!(first.contract_id, second.contract_id);
        assert_eq!(first.source_revision, second.source_revision);
    }

    #[test]
    fn missing_tencent_baseline_cannot_claim_target_achieved() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        std::env::remove_var("BARON_TENCENT_BASELINE_JSON");
        let report = run_benchmark(&context).unwrap();
        assert_eq!(report.tencent.status, "unavailable");
        assert!(!report.target_achieved);
        assert!(report
            .hard_failures
            .iter()
            .any(|failure| failure.contains("Tencent")));
    }

    #[test]
    fn tencent_baseline_contract_requires_all_surfaces() {
        let mut scores = BTreeMap::new();
        scores.insert("wiki".to_string(), 92);
        assert!(!required_tencent_surfaces_present(&scores));
        scores.insert("long_term_memory_l0_l3".to_string(), 92);
        scores.insert("semantic_retrieval_grounded_synthesis".to_string(), 92);
        scores.insert("automatic_session_learning".to_string(), 92);
        scores.insert("codegraph".to_string(), 92);
        assert!(required_tencent_surfaces_present(&scores));
    }

    #[test]
    fn seeded_development_fixture_exercises_all_local_surfaces() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let other_repo = temp.path().join("other-repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&other_repo).unwrap();
        initialize_project(&repo, AdapterKind::Codex, &vault).unwrap();
        initialize_project(&other_repo, AdapterKind::Codex, &vault).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let other_context = ensure_vault(&vault, &other_repo).unwrap();
        write(
            &context.project_root.join("Facts.md"),
            "# Verified Memory\n\n- Verified proof: current work uses semantic retrieval and a safe next action.\n",
        );
        write(
            &context.project_root.join("Decisions.md"),
            "# Decision\n\n- The memory architecture decision keeps Vault Markdown as durable truth.\n",
        );
        write(
            &other_context.project_root.join("Facts.md"),
            "# Other Project\n\n- Verified proof: current work uses semantic retrieval and a safe next action.\n",
        );
        write(
            &context.project_root.join("Sessions/Imported/session.md"),
            "### User\n\nWe decided the memory architecture keeps Vault Markdown as truth.\n\n### Assistant\n\nThe proof test passed; next action is verify the semantic retrieval handoff.\n",
        );
        write(
            &repo.join("docs/ARCHITECTURE.md"),
            "# Architecture\n\nSee [Memory](MEMORY.md).\n\n## Memory\nVault Markdown is durable truth.\n",
        );
        write(
            &repo.join("docs/MEMORY.md"),
            "# Memory\n\nThe architecture memory handoff carries proof citations.\n",
        );
        write(
            &repo.join("src/lib.rs"),
            "fn context_memory() { memory(); }\nfn memory() {}\n",
        );
        let report = run_benchmark(&context).unwrap();
        let scores = report
            .baron_scores
            .iter()
            .map(|score| (score.surface.as_str(), score.score))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(scores.get("long_term_memory_l0_l3"), Some(&100));
        assert_eq!(
            scores.get("semantic_retrieval_grounded_synthesis"),
            Some(&100)
        );
        assert_eq!(scores.get("automatic_session_learning"), Some(&100));
        assert_eq!(scores.get("wiki"), Some(&100));
        assert_eq!(scores.get("codegraph"), Some(&100));
        assert!(!report.target_achieved);
        assert!(!report.same_corpus_win);
    }
}
