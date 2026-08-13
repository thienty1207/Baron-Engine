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
use crate::firewall::{recall, recall_v4};
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
    #[serde(default)]
    pub links: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiSearchHit {
    pub document: String,
    pub heading: String,
    pub citation: String,
    pub excerpt: String,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub link_path: Vec<String>,
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
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalGraphSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub language: String,
    pub file: String,
    pub line: usize,
    pub confidence: String,
    #[serde(default)]
    pub span: String,
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
    #[serde(default)]
    pub imports: Vec<String>,
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
            let trust = format!("{:?}", record.trust_state()).to_lowercase();
            let layer = record.abstraction_level().as_str().to_string();
            ResumeMemoryHit {
                id: record.id,
                layer,
                kind: format!("{:?}", record.kind).to_lowercase(),
                excerpt: redact_sensitive(&record.excerpt),
                path: record.path,
                trust,
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

/// Baron 4.0 candidate Resume Brief. It preserves the 3.8 contract and adds
/// stronger project-filtered reranking plus explicit abstraction/trust labels.
pub fn build_resume_brief_v4(
    context: &VaultContext,
    task: Option<&str>,
    max_chars: usize,
) -> Result<ResumeBrief> {
    let mut brief = build_resume_brief(context, task, max_chars)?;
    let query = task
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("current work resume checkpoint");
    let recalled = recall_v4(context, query, 12)?;
    brief.memory_hits = recalled
        .results
        .into_iter()
        .filter(|hit| hit.record.project_id.as_deref() == Some(context.project_id.as_str()))
        .take(8)
        .map(|hit| {
            let record = hit.record;
            let trust = format!("{:?}", record.trust_state()).to_lowercase();
            let layer = record.abstraction_level().as_str().to_string();
            ResumeMemoryHit {
                id: record.id,
                layer: format!("l{}-{}", layer_rank(&layer), layer),
                kind: format!("{:?}", record.kind).to_lowercase(),
                excerpt: redact_sensitive(&record.excerpt),
                path: record.path,
                trust,
                why: hit.notes,
            }
        })
        .collect();
    brief.schema_version = KNOWLEDGE_SCHEMA_VERSION.saturating_add(1);
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
    for document in &index.documents {
        for section in &document.sections {
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
                heading: section.heading.clone(),
                citation: section.citation.clone(),
                excerpt: redact_sensitive(&bounded_text(&section.content, 900)),
                links: section.links.clone(),
                link_path: Vec::new(),
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

/// Candidate Wiki retrieval with heading/path/phrase/character overlap and
/// bounded link expansion. It still loads only the project-local disposable
/// index and marks every result with its source citation.
pub fn search_wiki_v4(
    repo_root: impl AsRef<Path>,
    query: &str,
    limit: usize,
) -> Result<Vec<WikiSearchHit>> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let index = load_wiki_index(&repo_root)?;
    let tokens = query_tokens(query);
    let normalized = query.to_lowercase();
    let current_revision = compute_code_source_fingerprint(&repo_root).unwrap_or_default();
    let mut hits = Vec::new();
    for document in &index.documents {
        for section in &document.sections {
            let haystack = format!(
                "{} {} {} {}",
                document.title,
                section.heading,
                section.content,
                section.links.join(" ")
            )
            .to_lowercase();
            let lexical = tokens
                .iter()
                .filter(|token| haystack.contains(*token))
                .count();
            let phrase = if !normalized.trim().is_empty() && haystack.contains(&normalized) {
                20
            } else {
                0
            };
            let trigram = text_trigram_overlap(&normalized, &haystack).min(24);
            let heading_bonus = tokens
                .iter()
                .filter(|token| section.heading.to_lowercase().contains(*token))
                .count()
                * 8;
            let link_bonus = section
                .links
                .iter()
                .filter(|link| {
                    normalized
                        .split_whitespace()
                        .any(|token| link.to_lowercase().contains(token))
                })
                .count()
                * 5;
            let score = lexical * 14 + phrase + trigram + heading_bonus + link_bonus;
            if score == 0 {
                continue;
            }
            hits.push(WikiSearchHit {
                document: document.path.clone(),
                heading: section.heading.clone(),
                citation: section.citation.clone(),
                excerpt: redact_sensitive(&bounded_text(&section.content, 900)),
                links: section.links.clone(),
                link_path: Vec::new(),
                score,
                stale: !current_revision.is_empty() && current_revision != index.source_revision,
            });
        }
    }
    // Traverse at most two bounded link hops. The path is retained as
    // inspectable evidence so a linked answer cannot masquerade as direct
    // lexical evidence. Broken/external links simply produce no hit.
    let mut frontier = hits.iter().take(8).cloned().collect::<Vec<_>>();
    for depth in 1..=2 {
        let parents = frontier.clone();
        frontier.clear();
        for parent in parents {
            for link in &parent.links {
                for document in &index.documents {
                    if !wiki_link_matches(link, &document.path) {
                        continue;
                    }
                    for section in &document.sections {
                        let linked_text = format!(
                            "{} {} {} {}",
                            document.title,
                            section.heading,
                            section.content,
                            section.links.join(" ")
                        )
                        .to_lowercase();
                        let linked_tokens = tokens
                            .iter()
                            .filter(|token| linked_text.contains(*token))
                            .count();
                        let linked_phrase =
                            normalized.trim().len() > 2 && linked_text.contains(&normalized);
                        if linked_tokens == 0 && !linked_phrase {
                            continue;
                        }
                        let mut link_path = parent.link_path.clone();
                        link_path.push(format!("{} -> {}", parent.document, document.path));
                        let hit = WikiSearchHit {
                            document: document.path.clone(),
                            heading: section.heading.clone(),
                            citation: section.citation.clone(),
                            excerpt: redact_sensitive(&bounded_text(&section.content, 900)),
                            links: section.links.clone(),
                            link_path,
                            score: parent.score / (depth + 1) + linked_tokens * 10,
                            stale: !current_revision.is_empty()
                                && current_revision != index.source_revision,
                        };
                        frontier.push(hit.clone());
                        hits.push(hit);
                    }
                }
            }
        }
    }
    let mut seen = BTreeSet::new();
    hits.retain(|hit| seen.insert(format!("{}#{}", hit.document, hit.heading)));
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
    let mut contents = BTreeMap::<String, String>::new();
    for path in source_paths {
        let relative = safe_relative(&repo_root, &path)?;
        let content = fs::read_to_string(&path).unwrap_or_default();
        let language = language_for(&path).to_string();
        files.push(LocalGraphFile {
            path: relative.clone(),
            language: language.clone(),
            content_hash: sha256(content.as_bytes()),
            imports: extract_imports(&content),
        });
        contents.insert(relative.clone(), content.clone());
        symbols.extend(extract_symbols(&relative, &content, &language));
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
    let symbols_by_name = symbols
        .iter()
        .map(|symbol| (symbol.name.clone(), symbol.id.clone()))
        .collect::<BTreeMap<_, _>>();
    let identifier_pattern = Regex::new(r"\b[A-Za-z_][A-Za-z0-9_]*\b").expect("identifier regex");
    let identifiers_by_file = contents
        .iter()
        .map(|(file, content)| {
            let identifiers = identifier_pattern
                .find_iter(content)
                .map(|value| value.as_str().to_string())
                .collect::<BTreeSet<_>>();
            (file.clone(), identifiers)
        })
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for symbol in &symbols {
        let Some(identifiers) = identifiers_by_file.get(&symbol.file) else {
            continue;
        };
        for name in identifiers.intersection(&names) {
            if *name == symbol.name {
                continue;
            }
            if let Some(target_id) = symbols_by_name.get(name) {
                edges.push(LocalGraphEdge {
                    from: symbol.id.clone(),
                    to: target_id.clone(),
                    relation: "references".to_string(),
                    confidence: "inferred".to_string(),
                });
            }
        }
    }
    edges.extend(extract_call_edges(&symbols, &repo_root));
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from && left.to == right.to && left.relation == right.relation
    });
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

/// Candidate CodeGraph query adds source-aware relation labels and import/call
/// hints. The current source remains authoritative; these edges are advisory.
pub fn search_local_code_graph_v4(
    repo_root: impl AsRef<Path>,
    query: &str,
    limit: usize,
) -> Result<Vec<LocalGraphSearchHit>> {
    let repo_root = canonical_repo(repo_root.as_ref())?;
    let graph = load_local_code_graph(&repo_root)?;
    let tokens = query_tokens(query);
    let mut hits = Vec::new();
    for symbol in graph.symbols {
        let imports = graph
            .files
            .iter()
            .find(|file| file.path == symbol.file)
            .map(|file| file.imports.clone())
            .unwrap_or_default();
        let haystack = format!(
            "{} {} {} {}",
            symbol.name,
            symbol.file,
            symbol.span,
            imports.join(" ")
        )
        .to_lowercase();
        let exact = tokens
            .iter()
            .filter(|token| haystack.contains(*token))
            .count();
        let fuzzy = text_trigram_overlap(&query.to_lowercase(), &haystack).min(20);
        let score = exact * 20 + fuzzy;
        if score == 0 {
            continue;
        }
        let relations = graph
            .edges
            .iter()
            .filter(|edge| edge.from == symbol.id || edge.to == symbol.id)
            .take(12)
            .map(|edge| {
                format!(
                    "{} {} [{}]",
                    edge.relation,
                    if edge.from == symbol.id {
                        &edge.to
                    } else {
                        &edge.from
                    },
                    edge.confidence
                )
            })
            .collect::<Vec<_>>();
        hits.push((score, LocalGraphSearchHit {
            symbol,
            imports,
            relations,
            why: "v4 lexical/fuzzy symbol match; relations remain advisory until current source verification".to_string(),
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
        let imports = graph
            .files
            .iter()
            .find(|file| file.path == symbol.file)
            .map(|file| file.imports.clone())
            .unwrap_or_default();
        let haystack =
            format!("{} {} {}", symbol.name, symbol.file, imports.join(" ")).to_lowercase();
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
            imports,
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
        links: extract_markdown_links(content),
    }
}

fn extract_symbols(file: &str, content: &str, language: &str) -> Vec<LocalGraphSymbol> {
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
            "function",
            Regex::new(r"^\s*func\s+(?:\([^)]*\)\s+)?([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
        ),
        (
            "class",
            Regex::new(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap(),
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
                    let column = line.find(name).unwrap_or(0) + 1;
                    symbols.push(LocalGraphSymbol {
                        id: sha256(format!("{file}|{}|{name}", line_number + 1).as_bytes()),
                        name: name.to_string(),
                        kind: (*kind).to_string(),
                        language: language.to_string(),
                        file: file.to_string(),
                        line: line_number + 1,
                        confidence: "extracted".to_string(),
                        span: format!(
                            "L{}-L{}:C{}-C{}",
                            line_number + 1,
                            line_number + 1,
                            column,
                            column + name.len().saturating_sub(1)
                        ),
                    });
                    break;
                }
            }
        }
    }
    symbols
}

fn extract_markdown_links(content: &str) -> Vec<String> {
    let pattern = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").expect("link regex");
    pattern
        .captures_iter(content)
        .filter_map(|captures| {
            captures
                .get(1)
                .map(|value| value.as_str().trim().to_string())
        })
        .filter(|value| !value.starts_with("http://") && !value.starts_with("https://"))
        .take(32)
        .collect()
}

fn wiki_link_matches(link: &str, target: &str) -> bool {
    let normalized_link = link
        .split('#')
        .next()
        .unwrap_or(link)
        .trim_start_matches("./")
        .replace('\\', "/");
    let normalized_target = target.replace('\\', "/");
    !normalized_link.is_empty()
        && (normalized_link == normalized_target
            || normalized_target.ends_with(&format!("/{normalized_link}"))
            || normalized_link.ends_with(&format!("/{normalized_target}"))
            || Path::new(&normalized_link).file_name() == Path::new(&normalized_target).file_name())
}

fn extract_imports(content: &str) -> Vec<String> {
    let patterns = [
        Regex::new(r"^\s*use\s+([^;]+)").expect("rust import regex"),
        Regex::new(r#"^\s*(?:import|export)\s+.*?from\s+['"]([^'"]+)"#).expect("js import regex"),
        Regex::new(r#"^\s*import\s+['"]([^'"]+)['"]"#).expect("js side effect import regex"),
        Regex::new(r"^\s*from\s+([^\s]+)\s+import\s+").expect("python import regex"),
        Regex::new(r#"^\s*import\s+"([^"]+)""#).expect("go import regex"),
    ];
    patterns
        .iter()
        .flat_map(|pattern| pattern.captures_iter(content))
        .filter_map(|captures| {
            captures
                .get(1)
                .map(|value| value.as_str().trim().to_string())
        })
        .take(64)
        .collect()
}

fn extract_call_edges(symbols: &[LocalGraphSymbol], repo_root: &Path) -> Vec<LocalGraphEdge> {
    let names = symbols
        .iter()
        .map(|symbol| symbol.name.clone())
        .collect::<BTreeSet<_>>();
    let call_pattern = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").expect("call regex");
    let mut contents = BTreeMap::<String, String>::new();
    for symbol in symbols {
        contents.entry(symbol.file.clone()).or_insert_with(|| {
            fs::read_to_string(repo_root.join(&symbol.file)).unwrap_or_default()
        });
    }
    let calls_by_file = contents
        .iter()
        .map(|(file, content)| {
            let calls = call_pattern
                .captures_iter(content)
                .filter_map(|captures| captures.get(1).map(|value| value.as_str().to_string()))
                .filter(|name| {
                    names.contains(name)
                        && !matches!(
                            name.as_str(),
                            "if" | "for" | "while" | "match" | "switch" | "catch"
                        )
                })
                .collect::<BTreeSet<_>>();
            (file.clone(), calls)
        })
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for symbol in symbols {
        let Some(calls) = calls_by_file.get(&symbol.file) else {
            continue;
        };
        for name in calls {
            if name == &symbol.name {
                continue;
            }
            if let Some(target) = symbols.iter().find(|candidate| candidate.name == *name) {
                edges.push(LocalGraphEdge {
                    from: symbol.id.clone(),
                    to: target.id.clone(),
                    relation: "calls".to_string(),
                    confidence: "syntax-evidence".to_string(),
                });
            }
        }
    }
    edges
}

fn text_trigram_overlap(left: &str, right: &str) -> usize {
    let grams = |value: &str| {
        let chars = value
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<Vec<_>>();
        chars
            .windows(3)
            .map(|window| window.iter().collect::<String>())
            .collect::<BTreeSet<_>>()
    };
    let left = grams(left);
    let right = grams(right);
    left.intersection(&right).count()
}

fn layer_rank(layer: &str) -> usize {
    if layer.starts_with("l3") {
        3
    } else if layer.starts_with("l2") {
        2
    } else if layer.starts_with("l1") {
        1
    } else {
        0
    }
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
            "# Project\n\nUse the memory resume brief. See [Architecture](docs/ARCHITECTURE.md).\n",
        )
        .unwrap();
        fs::write(
            repo.join("docs/ARCHITECTURE.md"),
            "# Architecture\n\n## Memory\nVault remains truth.\n",
        )
        .unwrap();
        fs::write(
            repo.join("src.rs"),
            "use crate::helper;\nfn build_memory() {}\nfn resume() { build_memory(); helper(); }\n",
        )
        .unwrap();
        fs::write(repo.join("helper.rs"), "fn helper() {}\n").unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(
            repo.join("src/app.ts"),
            "import {resume} from './resume';\nexport function app() { return resume(); }\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/tool.py"),
            "from helper import run\ndef tool():\n    return run()\n",
        )
        .unwrap();
        fs::write(
            repo.join("src/main.go"),
            "package main\nfunc main() { helper() }\nfunc helper() {}\n",
        )
        .unwrap();
        let wiki = index_wiki(&repo).unwrap();
        assert_eq!(wiki.documents, 2);
        assert_eq!(search_wiki(&repo, "memory", 5).unwrap().len(), 2);
        let linked = search_wiki_v4(&repo, "memory", 5).unwrap();
        assert!(linked.iter().any(|hit| !hit.links.is_empty()));
        let traversed = search_wiki_v4(&repo, "architecture", 5).unwrap();
        assert!(traversed
            .iter()
            .any(|hit| hit.document == "docs/ARCHITECTURE.md"));
        let graph = build_local_code_graph(&repo).unwrap();
        assert!(graph.symbols.iter().any(|symbol| symbol.name == "resume"));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.name == "app" && symbol.language == "typescript"));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.name == "tool" && symbol.language == "python"));
        assert!(graph
            .symbols
            .iter()
            .any(|symbol| symbol.name == "main" && symbol.language == "go"));
        assert!(graph
            .symbols
            .iter()
            .all(|symbol| symbol.span.contains(":C")));
        assert!(graph.files.iter().any(|file| !file.imports.is_empty()));
        assert!(graph.edges.iter().any(|edge| edge.relation == "calls"));
        let v4_hits = search_local_code_graph_v4(&repo, "resume", 5).unwrap();
        assert!(v4_hits.iter().any(|hit| !hit.imports.is_empty()));
        assert!(v4_hits
            .iter()
            .flat_map(|hit| hit.relations.iter())
            .any(|relation| relation.contains("calls")));
        let graph = build_local_code_graph(&repo).unwrap();
        assert!(graph.symbols.iter().any(|symbol| symbol.name == "resume"));
        assert!(!search_local_code_graph(&repo, "resume", 5)
            .unwrap()
            .is_empty());
        let second = index_wiki(&repo).unwrap();
        assert_eq!(second.changed_documents, 0);
    }
}
