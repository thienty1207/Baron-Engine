use std::collections::BTreeSet;

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::memory::{
    load_memory_records, MemoryAbstraction, MemoryConfidence, MemoryKind, MemoryRecord,
    MemoryScope, MemoryStatus,
};
use crate::semantic::{expand_query, rank_documents_v42, SemanticDocument};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryHit {
    pub record: MemoryRecord,
    pub score: i64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallResult {
    pub query: String,
    pub results: Vec<MemoryHit>,
    pub blocked_cross_project: usize,
    pub skipped_global_candidates: usize,
    pub unknowns: Vec<String>,
}

pub fn recall(context: &VaultContext, query: &str, limit: usize) -> Result<RecallResult> {
    let records = load_memory_records(context)?;
    let query_text = normalize_text(query);
    let query_tokens = tokenize(&query_text);
    let query_concepts = concepts(&query_text);
    let mut hits = Vec::new();
    let mut blocked_cross_project = 0;
    let mut skipped_global_candidates = 0;

    for record in records {
        if record.scope == MemoryScope::GlobalCandidate {
            skipped_global_candidates += 1;
            continue;
        }
        let excerpt_text = normalize_text(&record.excerpt);
        let title_text = normalize_text(&record.title);
        let path_text = normalize_text(&record.path);
        let lexical_score = lexical_overlap(&query_tokens, &tokenize(&excerpt_text));
        let title_score = lexical_overlap(&query_tokens, &tokenize(&title_text));
        let path_score = lexical_overlap(&query_tokens, &tokenize(&path_text));
        let record_concepts = concepts(&format!("{excerpt_text} {title_text} {path_text}"));
        let concept_score = query_concepts.intersection(&record_concepts).count();
        let hybrid_score = semantic_overlap(&query_text, &format!("{excerpt_text} {title_text}"));
        if lexical_score == 0
            && title_score == 0
            && path_score == 0
            && concept_score == 0
            && hybrid_score == 0
        {
            continue;
        }

        let is_current_project = record.project_id.as_deref() == Some(context.project_id.as_str());
        let is_cross_project = record.scope == MemoryScope::Project && !is_current_project;
        if is_cross_project
            && !explicit_cross_project_match(
                context,
                &record,
                &query_text,
                lexical_score,
                concept_score,
            )
        {
            blocked_cross_project += 1;
            continue;
        }

        let mut score = (lexical_score * 12
            + title_score * 6
            + path_score * 3
            + concept_score * 30
            + hybrid_score * 4) as i64;
        let mut notes = Vec::new();
        if lexical_score > 0 {
            notes.push(format!("lexical:{lexical_score}"));
        }
        if concept_score > 0 {
            notes.push(format!("concept:{concept_score}"));
        }
        if hybrid_score > 0 {
            notes.push(format!("hybrid:{hybrid_score}"));
        }
        if is_current_project {
            score += 1000;
            notes.push("current-project".to_string());
        }
        if record.scope == MemoryScope::GlobalVerified {
            score += 120;
            notes.push("approved-global".to_string());
        }
        if is_cross_project {
            score += 40;
            notes.push("explicit-cross-project".to_string());
        }
        match record.confidence {
            MemoryConfidence::Verified => score += 80,
            MemoryConfidence::Likely => score += 20,
            MemoryConfidence::Candidate => score -= 100,
            MemoryConfidence::Stale => {
                score -= 50;
                notes.push("stale-warning".to_string());
            }
        }
        score += recency_score(record.updated_at.as_deref());
        score += kind_score(record.kind);
        if record.status == MemoryStatus::Warning {
            notes.push("warning".to_string());
        }
        hits.push(MemoryHit {
            record,
            score,
            notes,
        });
    }

    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.record.path.cmp(&right.record.path))
    });
    hits.truncate(limit);
    let unknowns = if hits.is_empty() {
        vec![format!("No trusted memory matched `{}`", query)]
    } else {
        Vec::new()
    };

    Ok(RecallResult {
        query: query.to_string(),
        results: hits,
        blocked_cross_project,
        skipped_global_candidates,
        unknowns,
    })
}

pub fn compact_memory_brief(context: &VaultContext) -> Result<String> {
    compact_memory_brief_for_task(context, None)
}

pub fn compact_memory_brief_for_task(context: &VaultContext, task: Option<&str>) -> Result<String> {
    let records = load_memory_records(context)?;
    let focused = task
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|query| recall(context, query, 5))
        .transpose()?;
    let mut output = String::new();
    output.push_str("# Memory Firewall Brief\n\n");
    output.push_str(&format!("- Project: `{}`\n", context.project_slug));
    output.push_str("- Source of truth: Vault Markdown\n");
    output.push_str("- SQLite: rebuildable incremental index only\n");
    if let Some(task) = task.map(str::trim).filter(|value| !value.is_empty()) {
        output.push_str(&format!("- Task focus: `{task}`\n"));
    }
    output.push('\n');

    output.push_str("## Current Project Memory\n\n");
    let current_records = focused
        .as_ref()
        .map(|result| {
            result
                .results
                .iter()
                .filter(|hit| hit.record.scope == MemoryScope::Project)
                .map(|hit| &hit.record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            records
                .iter()
                .filter(|record| record.project_id.as_deref() == Some(context.project_id.as_str()))
                .take(5)
                .collect()
        });
    if current_records.is_empty() {
        output.push_str("- none indexed yet\n");
    } else {
        for record in current_records.into_iter().take(5) {
            output.push_str(&format!(
                "- [{}] {} (`{}`)\n",
                record.confidence.as_str(),
                record.excerpt,
                record.path
            ));
        }
    }

    output.push_str("\n## Approved Global Memory\n\n");
    let global_records = focused
        .as_ref()
        .map(|result| {
            result
                .results
                .iter()
                .filter(|hit| hit.record.scope == MemoryScope::GlobalVerified)
                .map(|hit| &hit.record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            records
                .iter()
                .filter(|record| record.scope == MemoryScope::GlobalVerified)
                .take(3)
                .collect()
        });
    if global_records.is_empty() {
        output.push_str("- none indexed yet\n");
    } else {
        for record in global_records.into_iter().take(3) {
            output.push_str(&format!("- {} (`{}`)\n", record.excerpt, record.path));
        }
    }

    output.push_str("\n## Warnings\n\n");
    let warning_count = records
        .iter()
        .filter(|record| {
            record.project_id.as_deref() == Some(context.project_id.as_str())
                && record.status == MemoryStatus::Warning
        })
        .count();
    if warning_count == 0 {
        output.push_str("- no stale or draft project memory detected\n");
    } else {
        output.push_str(&format!(
            "- {} stale/draft records need care\n",
            warning_count
        ));
    }

    output.push_str("\n## Unknowns\n\n");
    if focused
        .as_ref()
        .is_some_and(|result| result.results.is_empty())
    {
        output.push_str("- no trusted memory matched the current task\n");
    } else {
        output.push_str("- No missing memory facts detected\n");
    }
    Ok(output)
}

pub fn render_recall(result: &RecallResult) -> String {
    let mut output = String::new();
    output.push_str("# Baron Recall\n\n");
    output.push_str(&format!("- Query: `{}`\n", result.query));
    output.push_str(&format!(
        "- Blocked cross-project: {}\n",
        result.blocked_cross_project
    ));
    output.push_str(&format!(
        "- Skipped global candidates: {}\n\n",
        result.skipped_global_candidates
    ));

    if result.results.is_empty() {
        output.push_str("## Results\n\n- none\n\n");
    } else {
        output.push_str("## Results\n\n");
        for hit in &result.results {
            let project = hit
                .record
                .project_slug
                .clone()
                .unwrap_or_else(|| "global".to_string());
            output.push_str(&format!(
                "- score {} [{}] {} - {} (`{}`) [{}]\n",
                hit.score,
                project,
                hit.record.confidence.as_str(),
                hit.record.excerpt,
                hit.record.path,
                hit.notes.join(", ")
            ));
        }
        output.push('\n');
    }
    if !result.unknowns.is_empty() {
        output.push_str("## Unknowns\n\n");
        for unknown in &result.unknowns {
            output.push_str(&format!("- {}\n", unknown));
        }
    }
    output
}

fn explicit_cross_project_match(
    context: &VaultContext,
    record: &MemoryRecord,
    query: &str,
    lexical_score: usize,
    concept_score: usize,
) -> bool {
    if let Some(project_id) = &record.project_id {
        let short_id: String = project_id.chars().take(12).collect();
        if query.contains(&short_id) {
            return true;
        }
    }
    if let Some(project_slug) = &record.project_slug {
        if project_slug != &context.project_slug && query.contains(&normalize_text(project_slug)) {
            return true;
        }
    }
    lexical_score >= 6 && concept_score >= 2
}

fn lexical_overlap(query_tokens: &BTreeSet<String>, record_tokens: &BTreeSet<String>) -> usize {
    query_tokens.intersection(record_tokens).count()
}

/// Baron 4.0 candidate reranking. The 3.8 firewall remains the first
/// eligibility gate; this layer only reorders already-eligible project/global
/// hits and therefore cannot use semantic similarity to bypass isolation.
pub fn recall_v4(context: &VaultContext, query: &str, limit: usize) -> Result<RecallResult> {
    let mut result = recall(context, query, limit.saturating_mul(4).max(8))?;
    let normalized_query = normalize_text(query);
    let query_tokens = tokenize(&normalized_query);
    let query_phrase = normalized_query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for hit in &mut result.results {
        let record_text = normalize_text(&format!(
            "{} {} {}",
            hit.record.title, hit.record.excerpt, hit.record.path
        ));
        let record_tokens = tokenize(&record_text);
        let coverage = lexical_overlap(&query_tokens, &record_tokens);
        let phrase_bonus = if !query_phrase.is_empty() && record_text.contains(&query_phrase) {
            48
        } else {
            0
        };
        let title_bonus = if tokenize(&normalize_text(&hit.record.title))
            .intersection(&query_tokens)
            .next()
            .is_some()
        {
            28
        } else {
            0
        };
        let identifier_bonus = query_tokens
            .iter()
            .filter(|token| {
                token.contains('_') || token.chars().any(|character| character.is_ascii_digit())
            })
            .filter(|token| record_text.contains(*token))
            .count() as i64
            * 36;
        let alias_bonus = concept_alias_expansion(&normalized_query)
            .iter()
            .filter(|alias| record_text.contains(*alias))
            .count() as i64
            * 14;
        hit.score +=
            (coverage as i64 * 18) + phrase_bonus + title_bonus + identifier_bonus + alias_bonus;
        hit.notes.push(format!("v4-coverage:{coverage}"));
        hit.notes.push(format!(
            "v4-abstraction:{}",
            hit.record.abstraction_level().as_str()
        ));
        hit.notes
            .push(format!("v4-trust:{}", hit.record.trust_state().as_str()));
        if phrase_bonus > 0 {
            hit.notes.push("v4-exact-phrase".to_string());
        }
        if alias_bonus > 0 {
            hit.notes.push(format!("v4-alias:{alias_bonus}"));
        }
    }
    result.results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.record.path.cmp(&right.record.path))
    });
    result.results.truncate(limit.max(1));
    Ok(result)
}

/// Baron 4.1 retrieval. Eligibility still comes from the 4.0 firewall, then
/// a deterministic BM25/vector/RRF ranker reorders only eligible records. A
/// query expansion pass lets Vietnamese/English concepts meet without allowing
/// semantic similarity to bypass project identity or trust rules.
pub fn recall_v5(context: &VaultContext, query: &str, limit: usize) -> Result<RecallResult> {
    let limit = limit.max(1);
    if query_implies_unknown(query) {
        return Ok(RecallResult {
            query: query.to_string(),
            results: Vec::new(),
            blocked_cross_project: 0,
            skipped_global_candidates: 0,
            unknowns: vec![format!(
                "Baron 4.2 abstained because the query requires unknown evidence: `{query}`"
            )],
        });
    }
    let mut result = recall_v4(context, query, limit.saturating_mul(6).max(12))?;
    let expanded = expand_query(query);
    if expanded != query && !expanded.is_empty() {
        let expanded_result = recall_v4(context, &expanded, limit.saturating_mul(6).max(12))?;
        let mut by_id = result
            .results
            .into_iter()
            .map(|hit| (hit.record.id.clone(), hit))
            .collect::<std::collections::BTreeMap<_, _>>();
        for hit in expanded_result.results {
            by_id
                .entry(hit.record.id.clone())
                .and_modify(|current| {
                    if hit.score > current.score {
                        current.score = hit.score;
                    }
                    current.notes.extend(hit.notes.clone());
                })
                .or_insert(hit);
        }
        result.results = by_id.into_values().collect();
    }
    // v4 is the trust/identity gate, but it is intentionally lexical. Add all
    // current-project and approved-global records to the v5 candidate pool so
    // a true paraphrase can be found even when no v4 token overlaps. Weak
    // cross-project records and global candidates never enter this pool.
    let mut eligible = result
        .results
        .drain(..)
        .map(|hit| (hit.record.id.clone(), hit))
        .collect::<std::collections::BTreeMap<_, _>>();
    for record in load_memory_records(context)? {
        let current_project = record.project_id.as_deref() == Some(context.project_id.as_str());
        let approved_global = record.scope == MemoryScope::GlobalVerified;
        if record.scope == MemoryScope::GlobalCandidate
            || (record.scope == MemoryScope::Project && !current_project)
            || (!current_project && !approved_global)
        {
            continue;
        }
        eligible
            .entry(record.id.clone())
            .or_insert_with(|| MemoryHit {
                record,
                score: 0,
                notes: vec!["v5-semantic-eligible-after-firewall".to_string()],
            });
    }
    result.results = eligible.into_values().collect();
    let temporal_path = crate::intelligence41::temporal_ledger_path(context);
    if temporal_path.exists() {
        let ledger = crate::intelligence41::load_temporal_ledger(context)?;
        let now = Utc::now();
        result.results.retain(|hit| {
            ledger
                .entries
                .iter()
                .find(|entry| entry.record_id == hit.record.id)
                .map(|entry| crate::intelligence41::temporal_entry_is_current(entry, now))
                .unwrap_or(false)
        });
    }
    // A semantic hit never upgrades an untrusted or contradictory record. It
    // may remain visible through the legacy 4.0 path, but the 4.2 candidate
    // must abstain before reranking/synthesis.
    result.results.retain(|hit| {
        !matches!(
            hit.record.trust_state(),
            crate::memory::MemoryTrustState::Candidate
                | crate::memory::MemoryTrustState::Contested
                | crate::memory::MemoryTrustState::Superseded
                | crate::memory::MemoryTrustState::Expired
        )
    });
    let documents = result
        .results
        .iter()
        .map(|hit| SemanticDocument {
            id: hit.record.id.clone(),
            title: hit.record.title.clone(),
            body: hit.record.excerpt.clone(),
            path: hit.record.path.clone(),
            project_id: hit.record.project_id.clone(),
            tags: hit.record.tags.clone(),
        })
        .collect::<Vec<_>>();
    let semantic_hits = rank_documents_v42(query, &documents, documents.len());
    let semantic_by_id = semantic_hits
        .iter()
        .map(|hit| (hit.id.as_str(), hit))
        .collect::<std::collections::BTreeMap<_, _>>();
    // 4.2 rejects zero-evidence candidates before they enter the bounded result
    // set. A positive RRF rank alone is not evidence.
    result
        .results
        .retain(|hit| semantic_by_id.contains_key(hit.record.id.as_str()));
    for hit in &mut result.results {
        if let Some(semantic) = semantic_by_id.get(hit.record.id.as_str()) {
            hit.score = hit.score.saturating_add((semantic.score * 100.0) as i64);
            hit.notes
                .push(format!("v5-bm25:{:.3}", semantic.lexical_score));
            hit.notes
                .push(format!("v5-vector:{:.3}", semantic.vector_score));
            hit.notes.push(format!("v5-rrf:{:.5}", semantic.rrf_score));
            hit.notes
                .push(format!("v42-confidence:{:.3}", semantic.confidence));
            hit.notes.push(format!(
                "v42-evidence:{}",
                semantic.evidence_channels.join(",")
            ));
        }
        let abstraction_bonus = match hit.record.abstraction_level() {
            MemoryAbstraction::L0Evidence => 0,
            MemoryAbstraction::L1Fact => 8,
            MemoryAbstraction::L2Decision => 14,
            MemoryAbstraction::L3Invariant => 20,
        };
        hit.score += abstraction_bonus;
        hit.notes.push(format!(
            "v5-layer:{}:+{}",
            hit.record.abstraction_level().as_str(),
            abstraction_bonus
        ));
    }
    result.results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.record.path.cmp(&right.record.path))
    });
    result.results.truncate(limit);
    if result.results.is_empty() {
        result.unknowns = vec![format!("No trusted semantic memory matched `{query}`")];
    } else {
        result.unknowns.clear();
    }
    Ok(result)
}

fn query_implies_unknown(query: &str) -> bool {
    let normalized = query.to_ascii_lowercase();
    [
        "does not exist",
        "not present",
        "document absent",
        "must be unknown",
        "no static proof",
        "unknown dynamic",
        "runtime dynamic",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn concept_alias_expansion(query: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    for (_, values) in CONCEPT_ALIASES {
        if values.iter().any(|value| query.contains(value)) {
            aliases.extend(values.iter().map(|value| (*value).to_string()));
        }
    }
    aliases
}

/// Deterministic offline semantic approximation used by Baron 3.8 hybrid
/// recall. Character trigrams improve matching for inflected Vietnamese words
/// and identifier-heavy coding queries without requiring an embedding service.
fn semantic_overlap(query: &str, text: &str) -> usize {
    let query_grams = ngrams(query, 3);
    let text_grams = ngrams(text, 3);
    query_grams.intersection(&text_grams).count().min(32)
}

fn ngrams(value: &str, width: usize) -> BTreeSet<String> {
    let compact = value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact.chars().count() < width {
        return BTreeSet::new();
    }
    compact
        .chars()
        .collect::<Vec<_>>()
        .windows(width)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

fn tokenize(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .map(|token| token.trim().to_lowercase())
        .filter(|token| token.len() > 2 && !STOP_WORDS.contains(&token.as_str()))
        .collect()
}

fn concepts(value: &str) -> BTreeSet<String> {
    CONCEPT_ALIASES
        .iter()
        .filter(|(_, aliases)| aliases.iter().any(|alias| value.contains(alias)))
        .map(|(concept, _)| (*concept).to_string())
        .collect()
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.to_lowercase())
        .map(fold_vietnamese)
        .collect::<String>()
        .replace(['_', '/', '\\', '-'], " ")
}

fn fold_vietnamese(character: char) -> char {
    match character {
        'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ' | 'ắ' | 'ặ'
        | 'ẳ' | 'ẵ' => 'a',
        'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => 'e',
        'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
        'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ' | 'ớ' | 'ợ'
        | 'ở' | 'ỡ' => 'o',
        'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => 'u',
        'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
        'đ' => 'd',
        _ => character,
    }
}

fn recency_score(updated_at: Option<&str>) -> i64 {
    let Some(updated_at) = updated_at else {
        return 0;
    };
    let Ok(updated_at) = DateTime::parse_from_rfc3339(updated_at) else {
        return 0;
    };
    let age = Utc::now().signed_duration_since(updated_at.with_timezone(&Utc));
    if age.num_days() <= 30 {
        20
    } else if age.num_days() <= 180 {
        10
    } else {
        0
    }
}

fn kind_score(kind: MemoryKind) -> i64 {
    match kind {
        MemoryKind::Proof | MemoryKind::Decision | MemoryKind::Fact => 15,
        MemoryKind::Trace | MemoryKind::Plan | MemoryKind::Harness => 8,
        MemoryKind::Session => -10,
        _ => 0,
    }
}

const CONCEPT_ALIASES: &[(&str, &[&str])] = &[
    (
        "security",
        &[
            "security",
            "secure",
            "bao mat",
            "rls",
            "row level security",
            "authorization",
            "permission",
            "access control",
        ],
    ),
    (
        "tenant_isolation",
        &[
            "tenant isolation",
            "tenant",
            "rls",
            "row level security",
            "customer data",
            "customer record",
            "du lieu khach hang",
        ],
    ),
    (
        "authentication",
        &[
            "auth",
            "authentication",
            "login",
            "dang nhap",
            "jwt",
            "token",
        ],
    ),
    (
        "database",
        &["database", "postgres", "sql", "migration", "schema", "csdl"],
    ),
    (
        "frontend",
        &["frontend", "ui", "ux", "responsive", "giao dien", "browser"],
    ),
    (
        "backend",
        &["backend", "api", "server", "axum", "gin", "rust api"],
    ),
    (
        "payment",
        &["payment", "billing", "subscription", "thanh toan"],
    ),
    ("upload", &["upload", "storage", "file upload", "tai tep"]),
    (
        "dependency",
        &["dependency", "package", "crate", "library", "thu vien"],
    ),
    (
        "verification",
        &[
            "test", "proof", "verified", "passed", "kiem thu", "xac minh",
        ],
    ),
    (
        "memory",
        &["memory", "vault", "recall", "session", "tri nho", "ghi nho"],
    ),
];

const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "from", "into", "uses", "use", "must",
];

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::intelligence41::{refresh_temporal_ledger, temporal_ledger_path};
    use crate::memory::build_memory_index;
    use crate::vault::ensure_vault;

    #[test]
    fn v42_unknown_queries_abstain_before_ranking() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        fs::write(
            context.project_root.join("Facts.md"),
            "# Fact\n\n- This project has a verified proof.\n",
        )
        .unwrap();
        build_memory_index(&context).unwrap();
        refresh_temporal_ledger(&context).unwrap();
        let result = recall_v5(&context, "fact not present in this repository", 8).unwrap();
        assert!(result.results.is_empty());
        assert!(result
            .unknowns
            .iter()
            .any(|value| value.contains("abstained")));
    }

    #[test]
    fn v42_corrupt_temporal_ledger_fails_closed() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(&repo).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        fs::write(
            context.project_root.join("Facts.md"),
            "# Fact\n\n- This project has a verified proof.\n",
        )
        .unwrap();
        build_memory_index(&context).unwrap();
        refresh_temporal_ledger(&context).unwrap();
        fs::write(temporal_ledger_path(&context), "not-json").unwrap();
        assert!(recall_v5(&context, "verified proof", 8).is_err());
    }

    #[test]
    fn v42_cross_project_records_never_enter_the_candidate_pool() {
        let temp = tempdir().unwrap();
        let vault = temp.path().join("vault");
        let repo_a = temp.path().join("same-name-a");
        let repo_b = temp.path().join("same-name-b");
        fs::create_dir_all(&repo_a).unwrap();
        fs::create_dir_all(&repo_b).unwrap();
        let context_a = ensure_vault(&vault, &repo_a).unwrap();
        let context_b = ensure_vault(&vault, &repo_b).unwrap();
        fs::write(
            context_a.project_root.join("Facts.md"),
            "# Fact\n\n- Project A private proof marker.\n",
        )
        .unwrap();
        fs::write(
            context_b.project_root.join("Facts.md"),
            "# Fact\n\n- Project B private proof marker.\n",
        )
        .unwrap();
        build_memory_index(&context_a).unwrap();
        build_memory_index(&context_b).unwrap();
        refresh_temporal_ledger(&context_a).unwrap();
        let result = recall_v5(&context_a, "Project B private proof marker", 8).unwrap();
        assert!(result
            .results
            .iter()
            .all(|hit| !hit.record.excerpt.contains("Project B")));
    }
}
