use std::collections::BTreeSet;

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::memory::{
    load_memory_records, MemoryConfidence, MemoryKind, MemoryRecord, MemoryScope, MemoryStatus,
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
        if lexical_score == 0 && title_score == 0 && path_score == 0 && concept_score == 0 {
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

        let mut score =
            (lexical_score * 12 + title_score * 6 + path_score * 3 + concept_score * 30) as i64;
        let mut notes = Vec::new();
        if lexical_score > 0 {
            notes.push(format!("lexical:{lexical_score}"));
        }
        if concept_score > 0 {
            notes.push(format!("concept:{concept_score}"));
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
