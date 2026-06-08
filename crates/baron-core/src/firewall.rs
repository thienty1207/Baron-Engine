use std::collections::BTreeSet;

use anyhow::Result;

use crate::memory::{
    load_memory_records, MemoryConfidence, MemoryRecord, MemoryScope, MemoryStatus,
};
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
    let query_tokens = tokenize(query);
    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();
    let mut blocked_cross_project = 0;
    let mut skipped_global_candidates = 0;

    for record in records {
        if record.scope == MemoryScope::GlobalCandidate {
            skipped_global_candidates += 1;
            continue;
        }
        let lexical_score = lexical_overlap(&query_tokens, &tokenize(&record.excerpt));
        let title_score = lexical_overlap(&query_tokens, &tokenize(&record.title));
        if lexical_score == 0 && title_score == 0 {
            continue;
        }

        let is_cross_project = record.scope == MemoryScope::Project
            && record.project_slug.as_deref() != Some(&context.project_slug);
        if is_cross_project && !explicit_cross_project_match(&record, &query_lower, lexical_score) {
            blocked_cross_project += 1;
            continue;
        }

        let mut score = (lexical_score * 10 + title_score * 4) as i64;
        let mut notes = Vec::new();
        if record.project_slug.as_deref() == Some(&context.project_slug) {
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
                score -= 40;
                notes.push("stale-warning".to_string());
            }
        }
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
    let records = load_memory_records(context)?;
    let mut output = String::new();
    output.push_str("# Memory Firewall Brief\n\n");
    output.push_str(&format!("- Project: `{}`\n", context.project_slug));
    output.push_str("- Source of truth: Vault Markdown\n");
    output.push_str("- SQLite: rebuildable index only\n\n");

    output.push_str("## Current Project Memory\n\n");
    let mut current_count = 0;
    for record in records
        .iter()
        .filter(|record| record.project_slug.as_deref() == Some(&context.project_slug))
        .take(5)
    {
        current_count += 1;
        output.push_str(&format!(
            "- [{}] {} (`{}`)\n",
            record.confidence.as_str(),
            record.excerpt,
            record.path
        ));
    }
    if current_count == 0 {
        output.push_str("- none indexed yet\n");
    }

    output.push_str("\n## Approved Global Memory\n\n");
    let mut global_count = 0;
    for record in records
        .iter()
        .filter(|record| record.scope == MemoryScope::GlobalVerified)
        .take(3)
    {
        global_count += 1;
        output.push_str(&format!("- {} (`{}`)\n", record.excerpt, record.path));
    }
    if global_count == 0 {
        output.push_str("- none indexed yet\n");
    }

    output.push_str("\n## Warnings\n\n");
    let warning_count = records
        .iter()
        .filter(|record| {
            record.project_slug.as_deref() == Some(&context.project_slug)
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
    output.push_str("- No missing memory facts detected\n");
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
                "- score {} [{}] {} - {} (`{}`)\n",
                hit.score,
                project,
                hit.record.confidence.as_str(),
                hit.record.excerpt,
                hit.record.path
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
    record: &MemoryRecord,
    query_lower: &str,
    lexical_score: usize,
) -> bool {
    if let Some(project_slug) = &record.project_slug {
        if query_lower.contains(project_slug) {
            return true;
        }
    }
    lexical_score >= 4
}

fn lexical_overlap(query_tokens: &BTreeSet<String>, record_tokens: &BTreeSet<String>) -> usize {
    query_tokens.intersection(record_tokens).count()
}

fn tokenize(value: &str) -> BTreeSet<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(|token| token.trim().to_lowercase())
        .filter(|token| token.len() > 2 && !STOP_WORDS.contains(&token.as_str()))
        .collect()
}

const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "this", "that", "from", "into", "uses", "use", "must",
];
