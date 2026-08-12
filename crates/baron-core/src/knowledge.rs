//! Baron 3.8 local knowledge surfaces.
//!
//! The module deliberately keeps Markdown/Vault and repository files as the
//! source of truth.  Wiki, local graph, and benchmark files are disposable
//! project-local accelerators.  No network, embedding service, or remote graph
//! is required for the default path.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::code_graph::compute_code_source_fingerprint;
use crate::config::{load_project_config, PROJECT_SCHEMA_VERSION};
use crate::firewall::recall;
use crate::identity::project_id_for_path;
use crate::memory::{MemoryConfidence, MemoryKind, MemoryRecord};
use crate::vault::VaultContext;

pub const KNOWLEDGE_SCHEMA_VERSION: u32 = 1;
pub const MAX_RESUME_CHARS: usize = 9_000;
pub const MAX_WIKI_RESULTS: usize = 20;
pub const MAX_GRAPH_RESULTS: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLayer {
    Evidence,
    Verified,
    Decision,
    Invariant,
}

impl MemoryLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Verified => "verified",
            Self::Decision => "decision",
            Self::Invariant => "invariant",
        }
    }
}

pub fn memory_layer(record: &MemoryRecord) -> MemoryLayer {
    match record.kind {
        MemoryKind::Decision | MemoryKind::Plan | MemoryKind::Harness => MemoryLayer::Decision,
        MemoryKind::Proof | MemoryKind::Trace => MemoryLayer::Verified,
        MemoryKind::Fact
            if record.confidence == MemoryConfidence::Verified
                && record.status != crate::memory::MemoryStatus::Candidate =>
        {
            MemoryLayer::Invariant
        }
        MemoryKind::Session | MemoryKind::Research | MemoryKind::Note | MemoryKind::Question => {
            MemoryLayer::Evidence
        }
        _ => MemoryLayer::Verified,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeBrief {
    pub schema_version: u32,
    pub project_id: String,
    pub project_slug: String,
    pub source_revision: String,
    pub current_objective: String,
    pub current_phase: String,
    pub last_checkpoint: String,
    pub confirmed_decisions: Vec<String>,
    pub open_blocker: String,
    pub affected_files: Vec<String>,
    pub proof_status: String,
    pub trace_status: String,
    pub unknowns: Vec<String>,
    pub next_action: String,
    pub memory_hits: Vec<ResumeMemoryHit>,
    pub bounded_chars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeMemoryHit {
    pub id: String,
    pub layer: String,
    pub kind: String,
    pub excerpt: String,
    pub path: String,
    pub trust: String,
    pub why: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeBenchmark {
    pub schema_version: u32,
    pub project_id: String,
    pub source_revision: String,
    pub memory_records: usize,
    pub brief_chars: usize,
    pub estimated_tokens: usize,
    pub bounded: bool,
    pub project_isolated: bool,
    pub missing_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiIndex {
    pub schema_version: u32,
    pub project_id: String,
    pub source_revision: String,
    pub documents: Vec<WikiDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiDocument {
    pub id: String,
    pub project_id: String,
    pub path: String,
    pub title: String,
    pub source_hash: String,
    pub updated_at: String,
    pub sections: Vec<WikiSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiSection {
    pub heading: String,
    pub content: String,
    pub citation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiSearchHit {
    pub document: String,
    pub heading: String,
    pub citation: String,
    pub excerpt: String,
    pub score: usize,
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiIndexReport {
    pub project_id: String,
    pub source_revision: String,
    pub documents: usize,
    pub sections: usize,
    pub changed_documents: usize,
    pub cache_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCodeGraph {
    pub schema_version: u32,
    pub project_id: String,
    pub source_revision: String,
    pub files: Vec<LocalGraphFile>,
    pub symbols: Vec<LocalGraphSymbol>,
    pub edges: Vec<LocalGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalGraphFile {
    pub path: String,
    pub language: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalGraphSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub confidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalGraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub confidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalGraphSearchHit {
    pub symbol: LocalGraphSymbol,
    pub relations: Vec<String>,
    pub why: String,
}

pub fn build_resume_brief(
    context: &VaultContext,
    task: Option<&str>,
    max_chars: usize,
) -> Result<ResumeBrief> {
    let max_chars = max_chars.clamp(1_200, MAX_RESUME_CHARS);
    let records = crate::memory::load_memory_records(context)?;
    let recall_query = task
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("current work resume checkpoint");
    let recalled = recall(context, recall_query, 12)?;
    let current = |path: &str| read_bounded(&context.repo_root.join(path), 2_000);
    let plan = current("docs/baron/plans/CURRENT.md");
    let continuity = current("docs/baron/continuity/CURRENT.md");
    let recovery = current("docs/baron/continuity/CURRENT_RECOVERY.md");
    let harness = current("docs/baron/harness/CURRENT.md");
    let proof = current("docs/baron/proofs/INDEX.md");
    let trace = current("docs/baron/traces/INDEX.md");
    let source_revision = compute_code_source_fingerprint(&context.repo_root)
        .unwrap_or_else(|_| "unknown".to_string());

    let current_objective = first_field(&plan, &["- Title: ", "- Objective: "])
        .or_else(|| first_field(&harness, &["- Title: ", "- Current behavior: "]))
        .unwrap_or("unknown")
        .to_string();
    let current_phase = first_field(&plan, &["- Status: ", "- Phase: "])
        .unwrap_or("unknown")
        .trim_matches('`')
        .to_string();
    let last_checkpoint = first_field(&continuity, &["- Latest checkpoint: ", "- Note: "])
        .unwrap_or("none recorded")
        .to_string();
    let open_blocker = section_first_line(&recovery, "## Root Cause")
        .or_else(|| first_field(&plan, &["- Blocker: "]))
        .unwrap_or("none recorded")
        .to_string();
    let next_action = section_first_line(&recovery, "## Safe Next Action")
        .or_else(|| first_field(&plan, &["- Next action: "]))
        .unwrap_or("inspect current context and continue only with evidence")
        .to_string();
    let proof_status = summarize_state(&proof, "proof");
    let trace_status = summarize_state(&trace, "trace");
    let affected_files = extract_list(&continuity, "- Changed files: ")
        .into_iter()
        .take(12)
        .collect::<Vec<_>>();
    let unknowns = extract_unknowns(&harness, &continuity);
    let confirmed_decisions = records
        .iter()
        .filter(|record| {
            record.project_id.as_deref() == Some(context.project_id.as_str())
                && record.kind == MemoryKind::Decision
                && record.confidence != MemoryConfidence::Candidate
        })
        .take(8)
        .map(|record| record.excerpt.clone())
        .collect::<Vec<_>>();
    let memory_hits = recalled
        .results
        .into_iter()
        .filter(|hit| hit.record.project_id.as_deref() == Some(context.project_id.as_str()))
        .take(8)
        .map(|hit| {
            let record = hit.record;
            let layer = memory_layer(&record).as_str().to_string();
            ResumeMemoryHit {
                id: record.id,
                layer,
                kind: format!("{:?}", record.kind).to_lowercase(),
                excerpt: redact_sensitive(&record.excerpt),
                path: record.path,
                trust: format!("{:?}/{:?}", record.confidence, record.status).to_lowercase(),
                why: hit.notes,
            }
        })
        .collect::<Vec<_>>();

    let mut brief = ResumeBrief {
        schema_version: KNOWLEDGE_SCHEMA_VERSION,
        project_id: context.project_id.clone(),
        project_slug: context.project_slug.clone(),
        source_revision,
        current_objective,
        current_phase,
        last_checkpoint,
        confirmed_decisions,
        open_blocker,
        affected_files,
        proof_status,
        trace_status,
        unknowns,
        next_action,
        memory_hits,
        bounded_chars: 0,
    };
    brief.bounded_chars = render_resume_brief(&brief, max_chars).chars().count();
    Ok(brief)
}

pub fn render_resume_brief(brief: &ResumeBrief, max_chars: usize) -> String {
    let max_chars = max_chars.clamp(1_200, MAX_RESUME_CHARS);
    let mut output = String::new();
    output.push_str("# Baron Resume Brief\n\n");
    output.push_str(&format!("- Schema: `{}`\n", brief.schema_version));
    output.push_str(&format!("- Project: `{}`\n", brief.project_slug));
    output.push_str(&format!("- Project ID: `{}`\n", brief.project_id));
    output.push_str(&format!("- Source revision: `{}`\n", brief.source_revision));
    output.push_str(&format!(
        "- Current objective: {}\n",
        brief.current_objective
    ));
    output.push_str(&format!(
        "- Current phase/status: `{}`\n",
        brief.current_phase
    ));
    output.push_str(&format!("- Last checkpoint: {}\n", brief.last_checkpoint));
    output.push_str(&format!("- Open blocker: {}\n", brief.open_blocker));
    output.push_str(&format!("- Proof status: {}\n", brief.proof_status));
    output.push_str(&format!("- Trace status: {}\n", brief.trace_status));
    output.push_str(&format!("- Next safe action: {}\n\n", brief.next_action));
    output.push_str("## Confirmed Decisions\n\n");
    append_list(&mut output, &brief.confirmed_decisions, "none recorded");
    output.push_str("\n## Affected Files\n\n");
    append_list(&mut output, &brief.affected_files, "none recorded");
    output.push_str("\n## Unknowns\n\n");
    append_list(&mut output, &brief.unknowns, "none recorded");
    output.push_str("\n## Relevant Memory\n\n");
    if brief.memory_hits.is_empty() {
        output.push_str("- no trusted current-project memory matched\n");
    } else {
        for hit in &brief.memory_hits {
            output.push_str(&format!(
                "- [{}] {} ({}; source `{}`)\n",
                hit.layer, hit.excerpt, hit.trust, hit.path
            ));
        }
    }
    output.push_str("\n## Resume Rules\n\n");
    output.push_str("- Verify the current repository against this brief before editing.\n");
    output.push_str("- Treat unknown, stale, candidate, and inferred information as untrusted.\n");
    output.push_str("- Do not claim completion without current execution evidence.\n");
    if output.chars().count() > max_chars {
        let mut truncated = output
            .chars()
            .take(max_chars.saturating_sub(80))
            .collect::<String>();
        truncated.push_str(
            "\n\n[Resume brief truncated by Baron; use targeted recall for more detail.]\n",
        );
        return truncated;
    }
    output
}

pub fn benchmark_resume(context: &VaultContext, task: Option<&str>) -> Result<ResumeBenchmark> {
    let records = crate::memory::load_memory_records(context)?;
    let brief = build_resume_brief(context, task, MAX_RESUME_CHARS)?;
    let rendered = render_resume_brief(&brief, MAX_RESUME_CHARS);
    let missing_fields = [
        ("objective", brief.current_objective.as_str()),
        ("checkpoint", brief.last_checkpoint.as_str()),
        ("proof", brief.proof_status.as_str()),
        ("trace", brief.trace_status.as_str()),
        ("next_action", brief.next_action.as_str()),
    ]
    .iter()
    .filter(|(_, value)| value.trim().is_empty() || *value == "unknown")
    .map(|(name, _)| (*name).to_string())
    .collect::<Vec<_>>();
    Ok(ResumeBenchmark {
        schema_version: KNOWLEDGE_SCHEMA_VERSION,
        project_id: context.project_id.clone(),
        source_revision: brief.source_revision,
        memory_records: records.len(),
        brief_chars: rendered.chars().count(),
        estimated_tokens: rendered.chars().count().div_ceil(4),
        bounded: rendered.chars().count() <= MAX_RESUME_CHARS,
        project_isolated: true,
        missing_fields,
    })
}

pub fn index_wiki(repo_root: impl AsRef<Path>) -> Result<WikiIndexReport> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let project_id = project_id(&repo_root);
    let source_revision =
        compute_code_source_fingerprint(&repo_root).unwrap_or_else(|_| "unknown".to_string());
    let paths = discover_wiki_paths(&repo_root)?;
    let old = load_wiki_index(&repo_root).ok();
    let old_hashes = old
        .as_ref()
        .map(|index| {
            index
                .documents
                .iter()
                .map(|document| (document.path.clone(), document.source_hash.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut documents = Vec::new();
    let mut changed_documents = 0;
    for path in paths {
        let relative = safe_relative(&repo_root, &path)?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Could not read Wiki source {}", path.display()))?;
        let source_hash = sha256(content.as_bytes());
        if old_hashes.get(&relative) != Some(&source_hash) {
            changed_documents += 1;
        }
        documents.push(parse_wiki_document(
            &project_id,
            &relative,
            &content,
            &path,
        )?);
    }
    documents.sort_by(|left, right| left.path.cmp(&right.path));
    let index = WikiIndex {
        schema_version: KNOWLEDGE_SCHEMA_VERSION,
        project_id: project_id.clone(),
        source_revision: source_revision.clone(),
        documents,
    };
    let cache_path = wiki_cache_path(&repo_root);
    atomic_write_json(&cache_path, &index)?;
    let sections = index.documents.iter().map(|doc| doc.sections.len()).sum();
    Ok(WikiIndexReport {
        project_id,
        source_revision,
        documents: index.documents.len(),
        sections,
        changed_documents,
        cache_path: cache_path.display().to_string(),
    })
}

pub fn load_wiki_index(repo_root: impl AsRef<Path>) -> Result<WikiIndex> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let path = wiki_cache_path(&repo_root);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Wiki index is missing; run `baron wiki index`: {}",
            path.display()
        )
    })?;
    let index: WikiIndex = serde_json::from_str(&content)
        .with_context(|| format!("Could not parse Wiki index {}", path.display()))?;
    if index.schema_version != KNOWLEDGE_SCHEMA_VERSION
        || index.project_id != project_id(&repo_root)
    {
        bail!("Wiki index is stale or belongs to another project; rebuild it with `baron wiki index`.");
    }
    Ok(index)
}

pub fn search_wiki(
    repo_root: impl AsRef<Path>,
    query: &str,
    limit: usize,
) -> Result<Vec<WikiSearchHit>> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let index = load_wiki_index(&repo_root)?;
    let tokens = query_tokens(query);
    let current_revision = compute_code_source_fingerprint(&repo_root).unwrap_or_default();
    let mut hits = Vec::new();
    for document in index.documents {
        for section in document.sections {
            let haystack = format!("{} {} {}", document.title, section.heading, section.content);
            let score = tokens
                .iter()
                .filter(|token| haystack.to_lowercase().contains(*token))
                .count();
            if score == 0 {
                continue;
            }
            hits.push(WikiSearchHit {
                document: document.path.clone(),
                heading: section.heading,
                citation: section.citation,
                excerpt: redact_sensitive(&bounded_text(&section.content, 900)),
                score,
                stale: !current_revision.is_empty() && current_revision != index.source_revision,
            });
        }
    }
    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.document.cmp(&right.document))
    });
    hits.truncate(limit.clamp(1, MAX_WIKI_RESULTS));
    Ok(hits)
}

pub fn wiki_cache_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".baron/cache/wiki/index.json")
}

pub fn build_local_code_graph(repo_root: impl AsRef<Path>) -> Result<LocalCodeGraph> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let project_id = project_id(&repo_root);
    let source_revision =
        compute_code_source_fingerprint(&repo_root).unwrap_or_else(|_| "unknown".to_string());
    let source_paths = discover_source_paths(&repo_root)?;
    let mut files = Vec::new();
    let mut symbols = Vec::new();
    for path in source_paths {
        let relative = safe_relative(&repo_root, &path)?;
        let content = fs::read_to_string(&path).unwrap_or_default();
        let language = language_for(&path).to_string();
        files.push(LocalGraphFile {
            path: relative.clone(),
            language,
            content_hash: sha256(content.as_bytes()),
        });
        symbols.extend(extract_symbols(&relative, &content));
    }
    symbols.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.name.cmp(&right.name))
    });
    let names = symbols
        .iter()
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for symbol in &symbols {
        let content = fs::read_to_string(repo_root.join(&symbol.file)).unwrap_or_default();
        for name in names
            .iter()
            .filter(|name| **name != symbol.name && content.contains(*name))
        {
            if let Some(target) = symbols.iter().find(|candidate| candidate.name == *name) {
                edges.push(LocalGraphEdge {
                    from: symbol.id.clone(),
                    to: target.id.clone(),
                    relation: "references".to_string(),
                    confidence: "inferred".to_string(),
                });
            }
        }
    }
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
    });
    edges.dedup_by(|left, right| left.from == right.from && left.to == right.to);
    let graph = LocalCodeGraph {
        schema_version: KNOWLEDGE_SCHEMA_VERSION,
        project_id,
        source_revision,
        files,
        symbols,
        edges,
    };
    let path = local_graph_cache_path(&repo_root);
    atomic_write_json(&path, &graph)?;
    Ok(graph)
}

pub fn load_local_code_graph(repo_root: impl AsRef<Path>) -> Result<LocalCodeGraph> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let path = local_graph_cache_path(&repo_root);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Local CodeGraph is missing; run `baron knowledge codegraph-index`: {}",
            path.display()
        )
    })?;
    let graph: LocalCodeGraph = serde_json::from_str(&content)?;
    if graph.project_id != project_id(&repo_root)
        || graph.schema_version != KNOWLEDGE_SCHEMA_VERSION
    {
        bail!("Local CodeGraph belongs to another project or schema; rebuild it.");
    }
    Ok(graph)
}

pub fn search_local_code_graph(
    repo_root: impl AsRef<Path>,
    query: &str,
    limit: usize,
) -> Result<Vec<LocalGraphSearchHit>> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let graph = load_local_code_graph(&repo_root)?;
    let tokens = query_tokens(query);
    let mut hits = Vec::new();
    for symbol in graph.symbols {
        let haystack = format!("{} {}", symbol.name, symbol.file).to_lowercase();
        let score = tokens
            .iter()
            .filter(|token| haystack.contains(*token))
            .count();
        if score == 0 {
            continue;
        }
        let relations = graph
            .edges
            .iter()
            .filter(|edge| edge.from == symbol.id || edge.to == symbol.id)
            .take(8)
            .map(|edge| {
                format!(
                    "{} {}",
                    edge.relation,
                    if edge.from == symbol.id {
                        &edge.to
                    } else {
                        &edge.from
                    }
                )
            })
            .collect::<Vec<_>>();
        hits.push((score, LocalGraphSearchHit {
            symbol,
            relations,
            why: "matched current project symbol/path; graph edges are advisory until source verification".to_string(),
        }));
    }
    hits.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.symbol.file.cmp(&right.1.symbol.file))
    });
    Ok(hits
        .into_iter()
        .take(limit.clamp(1, MAX_GRAPH_RESULTS))
        .map(|(_, hit)| hit)
        .collect())
}

pub fn local_graph_cache_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".baron/cache/code-graph/local.json")
}

pub fn redact_sensitive(value: &str) -> String {
    let patterns = [
        (
            r"(?i)(api[_-]?key|token|secret|password|private[_-]?key)\s*[:=]\s*[^\s,;]+",
            "$1=[REDACTED]",
        ),
        (r"(?i)bearer\s+[A-Za-z0-9._~+/=-]+", "Bearer [REDACTED]"),
        (
            r"-----BEGIN [A-Z ]+PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+PRIVATE KEY-----",
            "[PRIVATE KEY REDACTED]",
        ),
    ];
    patterns
        .iter()
        .fold(value.to_string(), |current, (pattern, replacement)| {
            Regex::new(pattern)
                .map(|regex| regex.replace_all(&current, *replacement).to_string())
                .unwrap_or(current)
        })
}

fn parse_wiki_document(
    project_id: &str,
    relative: &str,
    content: &str,
    path: &Path,
) -> Result<WikiDocument> {
    let mut sections = Vec::new();
    let mut heading = "Document".to_string();
    let mut body = Vec::new();
    let mut start_line = 1usize;
    for (index, line) in content.lines().enumerate() {
        if line.trim_start().starts_with('#') {
            if !body.is_empty() {
                sections.push(wiki_section(
                    relative,
                    &heading,
                    &body.join("\n"),
                    start_line,
                    index,
                ));
                body.clear();
            }
            heading = line.trim_start_matches('#').trim().to_string();
            start_line = index + 1;
        } else {
            body.push(line.to_string());
        }
    }
    if !body.is_empty() {
        sections.push(wiki_section(
            relative,
            &heading,
            &body.join("\n"),
            start_line,
            content.lines().count(),
        ));
    }
    if sections.is_empty() {
        sections.push(wiki_section(
            relative,
            &heading,
            content,
            1,
            content.lines().count().max(1),
        ));
    }
    let title = sections
        .first()
        .map(|section| section.heading.clone())
        .unwrap_or_else(|| relative.to_string());
    let updated_at = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map(DateTime::<Utc>::from)
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|_| "unknown".to_string());
    Ok(WikiDocument {
        id: sha256(format!("{project_id}|{relative}").as_bytes()),
        project_id: project_id.to_string(),
        path: relative.to_string(),
        title,
        source_hash: sha256(content.as_bytes()),
        updated_at,
        sections,
    })
}

fn wiki_section(path: &str, heading: &str, content: &str, start: usize, end: usize) -> WikiSection {
    WikiSection {
        heading: heading.to_string(),
        content: redact_sensitive(&bounded_text(content.trim(), 6_000)),
        citation: format!("{path}#L{start}-L{end}"),
    }
}

fn extract_symbols(file: &str, content: &str) -> Vec<LocalGraphSymbol> {
    let patterns = [
        (
            "function",
            Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
        ),
        (
            "class",
            Regex::new(r"^\s*(?:export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
        ),
        (
            "function",
            Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)")
                .unwrap(),
        ),
        (
            "function",
            Regex::new(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
        ),
        (
            "type",
            Regex::new(
                r"^\s*(?:pub\s+)?(?:struct|enum|trait|interface|type)\s+([A-Za-z_][A-Za-z0-9_]*)",
            )
            .unwrap(),
        ),
    ];
    let mut symbols = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        for (kind, pattern) in &patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(name) = captures.get(1).map(|value| value.as_str()) {
                    symbols.push(LocalGraphSymbol {
                        id: sha256(format!("{file}|{}|{name}", line_number + 1).as_bytes()),
                        name: name.to_string(),
                        kind: (*kind).to_string(),
                        file: file.to_string(),
                        line: line_number + 1,
                        confidence: "extracted".to_string(),
                    });
                    break;
                }
            }
        }
    }
    symbols
}

fn discover_wiki_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    visit_files(repo_root, &mut |path| {
        let relative = path.strip_prefix(repo_root).unwrap_or(path);
        let components = relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(name) => Some(name.to_string_lossy().to_ascii_lowercase()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let allowed = path.extension().and_then(|value| value.to_str()) == Some("md")
            && (relative.file_name().and_then(|value| value.to_str()) == Some("README.md")
                || components.iter().any(|value| value == "docs"));
        if allowed && !is_skipped_path(relative) {
            paths.push(path.to_path_buf());
        }
        Ok(())
    })?;
    paths.sort();
    Ok(paths)
}

fn discover_source_paths(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let extensions = ["rs", "ts", "tsx", "js", "jsx", "py", "go"];
    let mut paths = Vec::new();
    visit_files(repo_root, &mut |path| {
        let relative = path.strip_prefix(repo_root).unwrap_or(path);
        if extensions.contains(
            &path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default(),
        ) && !is_skipped_path(relative)
        {
            paths.push(path.to_path_buf());
        }
        Ok(())
    })?;
    paths.sort();
    Ok(paths)
}

fn visit_files(root: &Path, visitor: &mut impl FnMut(&Path) -> Result<()>) -> Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if is_skipped_path(path.strip_prefix(root).unwrap_or(&path)) {
            continue;
        }
        if entry.file_type()?.is_dir() {
            visit_files(&path, visitor)?;
        } else {
            visitor(&path)?;
        }
    }
    Ok(())
}

fn is_skipped_path(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(value) => matches!(
            value.to_string_lossy().to_ascii_lowercase().as_str(),
            ".git"
                | ".baron"
                | ".codex"
                | ".claude"
                | "target"
                | "node_modules"
                | "vendor"
                | "dist"
                | "build"
                | ".next"
                | ".cache"
                | "tmp"
        ),
        _ => false,
    })
}

fn safe_relative(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .context("Knowledge source escaped repository")?;
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!("Knowledge source path is unsafe");
    }
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn canonical_repo(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("Could not resolve repository path: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!(
            "Repository path is not a directory: {}",
            canonical.display()
        );
    }
    Ok(canonical)
}

fn project_id(repo_root: &Path) -> String {
    load_project_config(repo_root)
        .ok()
        .filter(|config| {
            config.schema_version >= PROJECT_SCHEMA_VERSION && !config.project_id.trim().is_empty()
        })
        .map(|config| config.project_id)
        .or_else(|| project_id_for_path(repo_root).ok())
        .unwrap_or_else(|| "unknown".to_string())
}

fn language_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "go" => "go",
        _ => "unknown",
    }
}

fn first_field<'a>(content: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    content.lines().find_map(|line| {
        prefixes
            .iter()
            .find_map(|prefix| line.strip_prefix(prefix))
            .map(str::trim)
    })
}

fn section_first_line<'a>(content: &'a str, heading: &str) -> Option<&'a str> {
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if line.trim() == heading {
            return lines.find(|value| !value.trim().is_empty()).map(str::trim);
        }
    }
    None
}

fn summarize_state(content: &str, label: &str) -> String {
    content
        .lines()
        .find(|line| line.to_ascii_lowercase().contains(label))
        .map(|line| line.trim().trim_start_matches('-').trim().to_string())
        .unwrap_or_else(|| format!("{label} evidence missing"))
}

fn extract_list(content: &str, prefix: &str) -> Vec<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().trim_matches('`').to_string())
                .filter(|item| !item.is_empty() && item != "none")
                .collect()
        })
        .unwrap_or_default()
}

fn extract_unknowns(harness: &str, continuity: &str) -> Vec<String> {
    harness
        .lines()
        .chain(continuity.lines())
        .filter_map(|line| {
            let lower = line.to_ascii_lowercase();
            (lower.contains("unknown") || lower.contains("blocker"))
                .then(|| line.trim().trim_start_matches('-').trim().to_string())
        })
        .filter(|line| !line.is_empty())
        .take(8)
        .collect()
}

fn read_bounded(path: &Path, max_chars: usize) -> String {
    fs::read_to_string(path)
        .map(|content| bounded_text(&content, max_chars))
        .unwrap_or_default()
}

fn bounded_text(value: &str, max_chars: usize) -> String {
    let value = redact_sensitive(value);
    if value.chars().count() <= max_chars {
        value
    } else {
        value.chars().take(max_chars).collect::<String>() + " …"
    }
}

fn append_list(output: &mut String, values: &[String], empty: &str) {
    if values.is_empty() {
        output.push_str(&format!("- {empty}\n"));
    } else {
        for value in values {
            output.push_str(&format!("- {}\n", bounded_text(value, 800)));
        }
    }
}

fn query_tokens(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() >= 2)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sha256(value: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(value);
    format!("{:x}", digest.finalize())
}

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&temp, serde_json::to_vec_pretty(value)?)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::vault::ensure_vault;

    #[test]
    fn resume_brief_is_bounded_and_redacts_secrets() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        fs::create_dir_all(repo.join("docs/baron/continuity")).unwrap();
        fs::write(
            repo.join("docs/baron/continuity/CURRENT.md"),
            "# Resume\n\n- Latest checkpoint: auth work\n- Changed files: `src/auth.rs`\n",
        )
        .unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        fs::write(
            context.project_root.join("Facts.md"),
            "# Facts\n\n- token=secret-value must never be stored\n",
        )
        .unwrap();
        crate::memory::build_memory_index(&context).unwrap();
        let brief = build_resume_brief(&context, Some("auth"), 1_500).unwrap();
        let rendered = render_resume_brief(&brief, 1_500);
        assert!(rendered.chars().count() <= 1_500 + 80);
        assert!(!rendered.contains("secret-value"));
        assert!(rendered.contains("Project ID"));
    }

    #[test]
    fn wiki_and_local_graph_are_project_bound_and_incremental() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("docs")).unwrap();
        fs::write(
            repo.join("README.md"),
            "# Project\n\nUse the memory resume brief.\n",
        )
        .unwrap();
        fs::write(
            repo.join("docs/ARCHITECTURE.md"),
            "# Architecture\n\n## Memory\nVault remains truth.\n",
        )
        .unwrap();
        fs::write(
            repo.join("src.rs"),
            "fn build_memory() {}\nfn resume() { build_memory(); }\n",
        )
        .unwrap();
        let wiki = index_wiki(&repo).unwrap();
        assert_eq!(wiki.documents, 2);
        assert_eq!(search_wiki(&repo, "memory", 5).unwrap().len(), 2);
        let graph = build_local_code_graph(&repo).unwrap();
        assert!(graph.symbols.iter().any(|symbol| symbol.name == "resume"));
        assert!(search_local_code_graph(&repo, "resume", 5).unwrap().len() == 1);
        let second = index_wiki(&repo).unwrap();
        assert_eq!(second.changed_documents, 0);
    }
}
