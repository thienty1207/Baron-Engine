use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use regex::Regex;
use rusqlite::{params, Connection, OpenFlags, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::identity::CapsuleMetadata;
use crate::vault::{load_capsule_metadata, VaultContext};

const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryScope {
    Project,
    GlobalVerified,
    GlobalCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    Fact,
    Decision,
    Task,
    Plan,
    Harness,
    Proof,
    Trace,
    Session,
    Research,
    Note,
    Question,
    Handoff,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryConfidence {
    Verified,
    Likely,
    Candidate,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    Active,
    Warning,
    Candidate,
    Contested,
    Superseded,
    Expired,
}

/// Baron 4.0 keeps abstraction level separate from trust. A polished
/// summary (L3) is never trusted merely because it is polished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryAbstraction {
    L0Evidence,
    L1Fact,
    L2Decision,
    L3Invariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryTrustState {
    Candidate,
    Verified,
    Contested,
    Superseded,
    Expired,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub scope: MemoryScope,
    pub project_id: Option<String>,
    pub project_slug: Option<String>,
    pub kind: MemoryKind,
    pub path: String,
    pub title: String,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub confidence: MemoryConfidence,
    pub status: MemoryStatus,
    pub updated_at: Option<String>,
    pub content_hash: String,
}

impl MemoryRecord {
    pub fn abstraction_level(&self) -> MemoryAbstraction {
        if self
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case("invariant"))
        {
            return MemoryAbstraction::L3Invariant;
        }
        match self.kind {
            MemoryKind::Session
            | MemoryKind::Research
            | MemoryKind::Note
            | MemoryKind::Question => MemoryAbstraction::L0Evidence,
            MemoryKind::Fact | MemoryKind::Proof | MemoryKind::Trace => MemoryAbstraction::L1Fact,
            MemoryKind::Decision
            | MemoryKind::Task
            | MemoryKind::Plan
            | MemoryKind::Harness
            | MemoryKind::Handoff
            | MemoryKind::Global => MemoryAbstraction::L2Decision,
        }
    }

    pub fn trust_state(&self) -> MemoryTrustState {
        if self.status == MemoryStatus::Candidate || self.confidence == MemoryConfidence::Candidate
        {
            return MemoryTrustState::Candidate;
        }
        if self.status == MemoryStatus::Contested {
            return MemoryTrustState::Contested;
        }
        if self.status == MemoryStatus::Superseded {
            return MemoryTrustState::Superseded;
        }
        if self.status == MemoryStatus::Expired {
            return MemoryTrustState::Expired;
        }
        if self.confidence == MemoryConfidence::Stale || self.status == MemoryStatus::Warning {
            return MemoryTrustState::Expired;
        }
        if self.confidence == MemoryConfidence::Verified {
            return MemoryTrustState::Verified;
        }
        MemoryTrustState::Unknown
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryIndexReport {
    pub total_sources: usize,
    pub reused_sources: usize,
    pub refreshed_sources: usize,
    pub deleted_sources: usize,
    pub total_records: usize,
    pub current_project_records: usize,
    pub global_verified_records: usize,
    pub global_candidate_records: usize,
    pub cross_project_records: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConsolidationItem {
    pub kind: String,
    pub record_ids: Vec<String>,
    pub reason: String,
    pub action: String,
    #[serde(default)]
    pub preferred_record_id: Option<String>,
    #[serde(default)]
    pub authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryConsolidationReport {
    pub schema_version: u32,
    pub generated_at: String,
    pub project_id: String,
    pub record_count: usize,
    pub duplicate_groups: usize,
    pub duplicate_records: usize,
    pub conflict_groups: usize,
    pub superseded_records: usize,
    pub candidate_records: usize,
    pub items: Vec<MemoryConsolidationItem>,
    pub writes_performed: bool,
    #[serde(default)]
    pub staged_path: Option<String>,
}

/// Read-only Phase 67 analysis. It stages merge/conflict/supersession decisions
/// as evidence; it never rewrites Vault Markdown or promotes a candidate.
pub fn analyze_memory_consolidation(context: &VaultContext) -> Result<MemoryConsolidationReport> {
    let records = load_memory_records(context)?
        .into_iter()
        .filter(|record| {
            record.scope != MemoryScope::Project
                || record.project_id.as_deref() == Some(context.project_id.as_str())
        })
        .collect::<Vec<_>>();
    let mut exact_groups = BTreeMap::<String, Vec<&MemoryRecord>>::new();
    let mut title_groups = BTreeMap::<String, Vec<&MemoryRecord>>::new();
    for record in &records {
        let identity = format!(
            "{}|{}|{}",
            record.project_id.as_deref().unwrap_or("global"),
            record.kind.as_str(),
            normalize_memory_text(&record.excerpt)
        );
        exact_groups.entry(identity).or_default().push(record);
        let title = format!(
            "{}|{}|{}",
            record.project_id.as_deref().unwrap_or("global"),
            record.kind.as_str(),
            normalize_memory_text(&record.title)
        );
        title_groups.entry(title).or_default().push(record);
    }
    let mut items = Vec::new();
    let mut duplicate_groups = 0;
    let mut duplicate_records = 0;
    for group in exact_groups.values().filter(|group| group.len() > 1) {
        duplicate_groups += 1;
        duplicate_records += group.len() - 1;
        items.push(MemoryConsolidationItem {
            kind: "duplicate".to_string(),
            record_ids: group.iter().map(|record| record.id.clone()).collect(),
            reason: "normalized project-scoped evidence is identical; retain source lineage"
                .to_string(),
            action: "stage_merge_candidate; retain every source lineage".to_string(),
            preferred_record_id: preferred_record(group).map(|record| record.id.clone()),
            authority: authority_explanation(group),
        });
    }
    let mut conflict_groups = 0;
    for group in title_groups.values().filter(|group| {
        group.len() > 1
            && group
                .iter()
                .map(|record| normalize_memory_text(&record.excerpt))
                .collect::<BTreeSet<_>>()
                .len()
                > 1
    }) {
        conflict_groups += 1;
        items.push(MemoryConsolidationItem {
            kind: "conflict".to_string(),
            record_ids: group.iter().map(|record| record.id.clone()).collect(),
            reason: "same project/kind/title has divergent evidence".to_string(),
            action: "keep_contested; require source/decision authority".to_string(),
            preferred_record_id: preferred_record(group).map(|record| record.id.clone()),
            authority: authority_explanation(group),
        });
    }
    let superseded_records = records
        .iter()
        .filter(|record| {
            record.confidence == MemoryConfidence::Stale
                || matches!(
                    record.status,
                    MemoryStatus::Expired | MemoryStatus::Superseded
                )
        })
        .count();
    for record in records.iter().filter(|record| {
        record.confidence == MemoryConfidence::Stale
            || matches!(
                record.status,
                MemoryStatus::Expired | MemoryStatus::Superseded
            )
    }) {
        items.push(MemoryConsolidationItem {
            kind: "supersession_candidate".to_string(),
            record_ids: vec![record.id.clone()],
            reason: "source is stale or explicitly marked interrupted/draft".to_string(),
            action: "retain_audit_trail; exclude_from_current_truth".to_string(),
            preferred_record_id: None,
            authority: "stale_or_expired_never_overrides_current_verified_evidence".to_string(),
        });
    }
    let candidate_records = records
        .iter()
        .filter(|record| {
            record.scope == MemoryScope::GlobalCandidate
                || record.confidence == MemoryConfidence::Candidate
                || record.status == MemoryStatus::Candidate
        })
        .count();
    Ok(MemoryConsolidationReport {
        schema_version: 1,
        generated_at: Utc::now().to_rfc3339(),
        project_id: context.project_id.clone(),
        record_count: records.len(),
        duplicate_groups,
        duplicate_records,
        conflict_groups,
        superseded_records,
        candidate_records,
        items,
        writes_performed: false,
        staged_path: None,
    })
}

/// Persist a consolidation proposal as a disposable, reviewable Vault
/// artifact. This does not edit source Markdown, change SQLite records, or
/// promote any candidate. Repeated staging is atomic and idempotent for the
/// same report content.
pub fn stage_memory_consolidation(
    context: &VaultContext,
    report: &MemoryConsolidationReport,
) -> Result<PathBuf> {
    let path = context
        .project_root
        .join("Artifacts")
        .join("memory-consolidation-candidates.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let mut staged = report.clone();
    staged.staged_path = Some(path.display().to_string());
    let content = serde_json::to_vec_pretty(&staged)?;
    fs::write(&temp, content)
        .with_context(|| format!("Could not stage consolidation report: {}", temp.display()))?;
    fs::rename(&temp, &path).with_context(|| {
        format!(
            "Could not activate consolidation report: {}",
            path.display()
        )
    })?;
    Ok(path)
}

fn authority_rank(record: &MemoryRecord) -> u8 {
    if matches!(
        record.status,
        MemoryStatus::Candidate
            | MemoryStatus::Contested
            | MemoryStatus::Superseded
            | MemoryStatus::Expired
    ) || matches!(
        record.confidence,
        MemoryConfidence::Candidate | MemoryConfidence::Stale
    ) {
        return 0;
    }
    let kind_rank = match record.kind {
        MemoryKind::Decision | MemoryKind::Proof => 4,
        MemoryKind::Fact | MemoryKind::Trace => 3,
        MemoryKind::Plan | MemoryKind::Harness | MemoryKind::Handoff => 2,
        MemoryKind::Task | MemoryKind::Global => 1,
        _ => 0,
    };
    let confidence_rank = match record.confidence {
        MemoryConfidence::Verified => 2,
        MemoryConfidence::Likely => 1,
        _ => 0,
    };
    kind_rank * 3 + confidence_rank
}

fn preferred_record<'a>(group: &[&'a MemoryRecord]) -> Option<&'a MemoryRecord> {
    group
        .iter()
        .copied()
        .filter(|record| authority_rank(record) > 0)
        .max_by(|left, right| {
            authority_rank(left)
                .cmp(&authority_rank(right))
                .then_with(|| left.updated_at.cmp(&right.updated_at))
                .then_with(|| left.id.cmp(&right.id))
        })
}

fn authority_explanation(group: &[&MemoryRecord]) -> String {
    preferred_record(group)
        .map(|record| {
            format!(
                "preferred_by_authority={} rank={} observed_at={}",
                record.id,
                authority_rank(record),
                record.updated_at.as_deref().unwrap_or("unknown")
            )
        })
        .unwrap_or_else(|| "no_verified_authority; keep_contested_or_candidate".to_string())
}

fn normalize_memory_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| character.to_lowercase())
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone)]
struct SourceDescriptor {
    absolute_path: PathBuf,
    relative_path: String,
    scope: MemoryScope,
    project_id: Option<String>,
    project_slug: Option<String>,
    kind: MemoryKind,
}

#[derive(Debug, Clone)]
struct ExistingSource {
    modified_ns: i64,
    size: i64,
    content_hash: String,
}

pub fn build_memory_index(context: &VaultContext) -> Result<MemoryIndexReport> {
    if let Some(parent) = context.index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let sources = discover_sources(context)?;
    let mut connection = Connection::open(&context.index_path)
        .with_context(|| format!("Could not open {}", context.index_path.display()))?;
    create_schema(&connection)?;
    let existing = load_existing_sources(&connection)?;
    let discovered_paths = sources
        .iter()
        .map(|source| source.relative_path.clone())
        .collect::<BTreeSet<_>>();
    let mut reused_sources = 0;
    let mut refreshed_sources = 0;
    let mut deleted_sources = 0;

    let transaction = connection.transaction()?;
    for source in &sources {
        let metadata = fs::metadata(&source.absolute_path)?;
        let modified_ns = modified_ns(&metadata);
        let size = metadata.len() as i64;
        if existing
            .get(&source.relative_path)
            .is_some_and(|item| item.modified_ns == modified_ns && item.size == size)
        {
            reused_sources += 1;
            continue;
        }

        let content = fs::read_to_string(&source.absolute_path)
            .with_context(|| format!("Could not read {}", source.absolute_path.display()))?;
        let content_hash = hash(&content);
        if existing
            .get(&source.relative_path)
            .is_some_and(|item| item.content_hash == content_hash)
        {
            update_source_metadata(&transaction, source, modified_ns, size, &content_hash)?;
            reused_sources += 1;
            continue;
        }

        let records = parse_source(context, source, &content, &metadata);
        replace_source(
            &transaction,
            source,
            modified_ns,
            size,
            &content_hash,
            &records,
        )?;
        refreshed_sources += 1;
    }

    for stale_path in existing
        .keys()
        .filter(|path| !discovered_paths.contains(*path))
    {
        transaction.execute("DELETE FROM records WHERE source_path = ?1", [stale_path])?;
        transaction.execute("DELETE FROM sources WHERE path = ?1", [stale_path])?;
        deleted_sources += 1;
    }
    transaction.commit()?;

    let records = load_memory_records(context)?;
    Ok(MemoryIndexReport {
        total_sources: sources.len(),
        reused_sources,
        refreshed_sources,
        deleted_sources,
        total_records: records.len(),
        current_project_records: records
            .iter()
            .filter(|record| record.project_id.as_deref() == Some(&context.project_id))
            .count(),
        global_verified_records: records
            .iter()
            .filter(|record| record.scope == MemoryScope::GlobalVerified)
            .count(),
        global_candidate_records: records
            .iter()
            .filter(|record| record.scope == MemoryScope::GlobalCandidate)
            .count(),
        cross_project_records: records
            .iter()
            .filter(|record| {
                record.scope == MemoryScope::Project
                    && record.project_id.as_deref() != Some(&context.project_id)
            })
            .count(),
    })
}

pub fn load_memory_records(context: &VaultContext) -> Result<Vec<MemoryRecord>> {
    if !context.index_path.exists() {
        return Ok(Vec::new());
    }
    let connection =
        Connection::open_with_flags(&context.index_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("Could not open {}", context.index_path.display()))?;
    validate_read_schema(&connection)?;
    let mut statement = connection.prepare(
        "SELECT id, scope, project_id, project_slug, kind, path, title, excerpt, tags,
                confidence, status, updated_at, content_hash
         FROM records
         ORDER BY path, id",
    )?;
    let rows = statement.query_map([], |row| {
        let tags: String = row.get(8)?;
        Ok(MemoryRecord {
            id: row.get(0)?,
            scope: MemoryScope::from_str(&row.get::<_, String>(1)?),
            project_id: row.get(2)?,
            project_slug: row.get(3)?,
            kind: MemoryKind::from_str(&row.get::<_, String>(4)?),
            path: row.get(5)?,
            title: row.get(6)?,
            excerpt: row.get(7)?,
            tags: tags
                .split(',')
                .filter(|tag| !tag.is_empty())
                .map(ToString::to_string)
                .collect(),
            confidence: MemoryConfidence::from_str(&row.get::<_, String>(9)?),
            status: MemoryStatus::from_str(&row.get::<_, String>(10)?),
            updated_at: row.get(11)?,
            content_hash: row.get(12)?,
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

fn validate_read_schema(connection: &Connection) -> Result<()> {
    let current: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current != SCHEMA_VERSION {
        anyhow::bail!("Memory index schema is incompatible; rebuild it with `baron memory index`.");
    }
    Ok(())
}

fn create_schema(connection: &Connection) -> Result<()> {
    let current: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current != SCHEMA_VERSION {
        connection.execute_batch(
            "DROP TABLE IF EXISTS records;
             DROP TABLE IF EXISTS sources;",
        )?;
    }
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS sources (
            path TEXT PRIMARY KEY,
            scope TEXT NOT NULL,
            project_id TEXT,
            project_slug TEXT,
            kind TEXT NOT NULL,
            modified_ns INTEGER NOT NULL,
            size INTEGER NOT NULL,
            content_hash TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS records (
            id TEXT PRIMARY KEY,
            source_path TEXT NOT NULL,
            scope TEXT NOT NULL,
            project_id TEXT,
            project_slug TEXT,
            kind TEXT NOT NULL,
            path TEXT NOT NULL,
            title TEXT NOT NULL,
            excerpt TEXT NOT NULL,
            tags TEXT NOT NULL,
            confidence TEXT NOT NULL,
            status TEXT NOT NULL,
            updated_at TEXT,
            content_hash TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS records_project_id ON records(project_id);
        CREATE INDEX IF NOT EXISTS records_scope ON records(scope);
        CREATE INDEX IF NOT EXISTS records_source_path ON records(source_path);",
    )?;
    connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

fn load_existing_sources(connection: &Connection) -> Result<BTreeMap<String, ExistingSource>> {
    let mut statement =
        connection.prepare("SELECT path, modified_ns, size, content_hash FROM sources")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ExistingSource {
                modified_ns: row.get(1)?,
                size: row.get(2)?,
                content_hash: row.get(3)?,
            },
        ))
    })?;
    let mut sources = BTreeMap::new();
    for row in rows {
        let (path, source) = row?;
        sources.insert(path, source);
    }
    Ok(sources)
}

fn discover_sources(context: &VaultContext) -> Result<Vec<SourceDescriptor>> {
    let mut sources = Vec::new();
    let projects_root = context.vault_root.join("Projects");
    if projects_root.exists() {
        let mut projects =
            fs::read_dir(&projects_root)?.collect::<std::result::Result<Vec<_>, _>>()?;
        projects.sort_by_key(|entry| entry.file_name());
        for entry in projects {
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let metadata =
                load_capsule_metadata(&entry.path())?.unwrap_or_else(|| legacy_metadata(&entry));
            collect_project_sources(context, &metadata, &entry.path(), &mut sources)?;
        }
    }
    for (path, scope) in [
        (&context.approved_global_path, MemoryScope::GlobalVerified),
        (
            &context.global_candidates_path,
            MemoryScope::GlobalCandidate,
        ),
    ] {
        if path.exists() {
            sources.push(descriptor(
                context,
                path.clone(),
                scope,
                None,
                None,
                MemoryKind::Global,
            ));
        }
    }
    sources.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(sources)
}

fn legacy_metadata(entry: &fs::DirEntry) -> CapsuleMetadata {
    let slug = entry.file_name().to_string_lossy().to_string();
    CapsuleMetadata {
        schema_version: 1,
        project_id: format!("legacy:{}", hash(&slug)),
        project_slug: slug,
    }
}

fn collect_project_sources(
    context: &VaultContext,
    metadata: &CapsuleMetadata,
    project_root: &Path,
    sources: &mut Vec<SourceDescriptor>,
) -> Result<()> {
    for (path, kind) in [
        (project_root.join("Facts.md"), MemoryKind::Fact),
        (project_root.join("Decisions.md"), MemoryKind::Decision),
        (project_root.join("Tasks.md"), MemoryKind::Task),
        (project_root.join("Open Questions.md"), MemoryKind::Question),
        (project_root.join("Handoff.md"), MemoryKind::Handoff),
    ] {
        if path.exists() {
            sources.push(descriptor(
                context,
                path,
                MemoryScope::Project,
                Some(metadata.project_id.clone()),
                Some(metadata.project_slug.clone()),
                kind,
            ));
        }
    }
    for (directory, kind) in [
        ("Plans", MemoryKind::Plan),
        ("ProductHarness", MemoryKind::Harness),
        ("Sessions", MemoryKind::Session),
        ("Research", MemoryKind::Research),
        ("Notes", MemoryKind::Note),
        ("Proofs", MemoryKind::Proof),
        ("Traces", MemoryKind::Trace),
    ] {
        collect_markdown_sources(
            context,
            &project_root.join(directory),
            metadata,
            kind,
            sources,
        )?;
    }
    Ok(())
}

fn collect_markdown_sources(
    context: &VaultContext,
    root: &Path,
    metadata: &CapsuleMetadata,
    kind: MemoryKind,
    sources: &mut Vec<SourceDescriptor>,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(root)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown_sources(context, &path, metadata, kind, sources)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            if kind == MemoryKind::Harness
                && path.file_name().and_then(|value| value.to_str()) == Some("DOMAIN_LANGUAGE.md")
            {
                // Domain language has a dedicated status-aware bounded renderer.
                // Generic line indexing would turn its full prose into ordinary
                // memory excerpts and bypass the compact-context boundary.
                continue;
            }
            sources.push(descriptor(
                context,
                path,
                MemoryScope::Project,
                Some(metadata.project_id.clone()),
                Some(metadata.project_slug.clone()),
                kind,
            ));
        }
    }
    Ok(())
}

fn descriptor(
    context: &VaultContext,
    absolute_path: PathBuf,
    scope: MemoryScope,
    project_id: Option<String>,
    project_slug: Option<String>,
    kind: MemoryKind,
) -> SourceDescriptor {
    let relative_path = absolute_path
        .strip_prefix(&context.vault_root)
        .unwrap_or(&absolute_path)
        .to_string_lossy()
        .replace('\\', "/");
    SourceDescriptor {
        absolute_path,
        relative_path,
        scope,
        project_id,
        project_slug,
        kind,
    }
}

fn parse_source(
    _context: &VaultContext,
    source: &SourceDescriptor,
    content: &str,
    metadata: &fs::Metadata,
) -> Vec<MemoryRecord> {
    let mut title = String::from("Memory");
    let mut records = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut in_frontmatter = false;
    let mut frontmatter_confidence = None;
    let mut frontmatter_status = None;
    let mut frontmatter_tags = Vec::new();
    let updated_at = metadata
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Secs, true));
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            if let Some(value) = trimmed.strip_prefix("confidence:") {
                frontmatter_confidence = Some(MemoryConfidence::from_str(value.trim()));
            } else if let Some(value) = trimmed.strip_prefix("status:") {
                frontmatter_status = Some(MemoryStatus::from_str(value.trim()));
            } else if let Some(value) = trimmed.strip_prefix("tags:") {
                frontmatter_tags.extend(
                    value
                        .trim()
                        .trim_matches(['[', ']'])
                        .split(',')
                        .map(str::trim)
                        .filter(|tag| !tag.is_empty())
                        .map(|tag| tag.trim_matches(['\'', '"']).to_string()),
                );
            }
            continue;
        }
        if trimmed.starts_with('#') {
            title = trimmed.trim_start_matches('#').trim().to_string();
            continue;
        }
        let excerpt = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .unwrap_or(trimmed)
            .trim();
        if excerpt.is_empty()
            || excerpt.starts_with("Only durable")
            || excerpt.starts_with("Candidates are")
            || excerpt.starts_with("Source file:")
            || excerpt.starts_with("Secrets:")
            || excerpt.len() < 8
        {
            continue;
        }
        let excerpt = redact_index_excerpt(excerpt);
        let confidence = frontmatter_confidence
            .unwrap_or_else(|| classify_confidence(source.scope, source.kind, &excerpt));
        let status = frontmatter_status.unwrap_or(match confidence {
            MemoryConfidence::Candidate => MemoryStatus::Candidate,
            MemoryConfidence::Stale => MemoryStatus::Warning,
            MemoryConfidence::Verified | MemoryConfidence::Likely => MemoryStatus::Active,
        });
        let id_source = format!(
            "{}|{:?}|{:?}|{}|{}",
            source.relative_path, source.scope, source.project_id, title, excerpt
        );
        let content_hash = hash(&id_source);
        if !seen_ids.insert(content_hash.clone()) {
            continue;
        }
        records.push(MemoryRecord {
            id: content_hash.clone(),
            scope: source.scope,
            project_id: source.project_id.clone(),
            project_slug: source.project_slug.clone(),
            kind: source.kind,
            path: source.relative_path.clone(),
            title: title.clone(),
            excerpt,
            tags: tags_for(source.kind, confidence)
                .into_iter()
                .chain(frontmatter_tags.iter().cloned())
                .collect(),
            confidence,
            status,
            updated_at: updated_at.clone(),
            content_hash,
        });
    }
    records
}

fn redact_index_excerpt(value: &str) -> String {
    let patterns = [
        (
            r"(?i)(api[_-]?key|token|secret|password|private[_-]?key)\s*[:=]\s*[^\s,;]+",
            "$1=[REDACTED]",
        ),
        (r"(?i)bearer\s+[A-Za-z0-9._~+/=-]+", "Bearer [REDACTED]"),
    ];
    patterns
        .iter()
        .fold(value.to_string(), |current, (pattern, replacement)| {
            Regex::new(pattern)
                .map(|regex| regex.replace_all(&current, *replacement).to_string())
                .unwrap_or(current)
        })
}

fn replace_source(
    transaction: &Transaction<'_>,
    source: &SourceDescriptor,
    modified_ns: i64,
    size: i64,
    content_hash: &str,
    records: &[MemoryRecord],
) -> Result<()> {
    transaction.execute(
        "DELETE FROM records WHERE source_path = ?1",
        [&source.relative_path],
    )?;
    update_source_metadata(transaction, source, modified_ns, size, content_hash)?;
    for record in records {
        transaction.execute(
            "INSERT INTO records (
                id, source_path, scope, project_id, project_slug, kind, path, title, excerpt,
                tags, confidence, status, updated_at, content_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                record.id,
                source.relative_path,
                record.scope.as_str(),
                record.project_id,
                record.project_slug,
                record.kind.as_str(),
                record.path,
                record.title,
                record.excerpt,
                record.tags.join(","),
                record.confidence.as_str(),
                record.status.as_str(),
                record.updated_at,
                record.content_hash
            ],
        )?;
    }
    Ok(())
}

fn update_source_metadata(
    transaction: &Transaction<'_>,
    source: &SourceDescriptor,
    modified_ns: i64,
    size: i64,
    content_hash: &str,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO sources (
            path, scope, project_id, project_slug, kind, modified_ns, size, content_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(path) DO UPDATE SET
            scope = excluded.scope,
            project_id = excluded.project_id,
            project_slug = excluded.project_slug,
            kind = excluded.kind,
            modified_ns = excluded.modified_ns,
            size = excluded.size,
            content_hash = excluded.content_hash",
        params![
            source.relative_path,
            source.scope.as_str(),
            source.project_id,
            source.project_slug,
            source.kind.as_str(),
            modified_ns,
            size,
            content_hash
        ],
    )?;
    Ok(())
}

fn modified_ns(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn classify_confidence(scope: MemoryScope, kind: MemoryKind, excerpt: &str) -> MemoryConfidence {
    if scope == MemoryScope::GlobalCandidate {
        return MemoryConfidence::Candidate;
    }
    let lower = excerpt.to_lowercase();
    if kind == MemoryKind::Session
        || lower.contains("stale")
        || lower.contains("draft")
        || lower.contains("interrupted")
    {
        return MemoryConfidence::Stale;
    }
    if lower.contains("verified")
        || lower.contains("proof")
        || lower.contains("passed")
        || lower.contains("test")
    {
        MemoryConfidence::Verified
    } else {
        MemoryConfidence::Likely
    }
}

fn tags_for(kind: MemoryKind, confidence: MemoryConfidence) -> Vec<String> {
    vec![kind.as_str().to_string(), confidence.as_str().to_string()]
}

fn hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}

impl MemoryScope {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryScope::Project => "project",
            MemoryScope::GlobalVerified => "global_verified",
            MemoryScope::GlobalCandidate => "global_candidate",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "global_verified" => MemoryScope::GlobalVerified,
            "global_candidate" => MemoryScope::GlobalCandidate,
            _ => MemoryScope::Project,
        }
    }
}

impl MemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryKind::Fact => "fact",
            MemoryKind::Decision => "decision",
            MemoryKind::Task => "task",
            MemoryKind::Plan => "plan",
            MemoryKind::Harness => "harness",
            MemoryKind::Proof => "proof",
            MemoryKind::Trace => "trace",
            MemoryKind::Session => "session",
            MemoryKind::Research => "research",
            MemoryKind::Note => "note",
            MemoryKind::Question => "question",
            MemoryKind::Handoff => "handoff",
            MemoryKind::Global => "global",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "decision" => MemoryKind::Decision,
            "task" => MemoryKind::Task,
            "plan" => MemoryKind::Plan,
            "harness" => MemoryKind::Harness,
            "proof" => MemoryKind::Proof,
            "trace" => MemoryKind::Trace,
            "session" => MemoryKind::Session,
            "research" => MemoryKind::Research,
            "note" => MemoryKind::Note,
            "question" => MemoryKind::Question,
            "handoff" => MemoryKind::Handoff,
            "global" => MemoryKind::Global,
            _ => MemoryKind::Fact,
        }
    }
}

impl MemoryConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryConfidence::Verified => "verified",
            MemoryConfidence::Likely => "likely",
            MemoryConfidence::Candidate => "candidate",
            MemoryConfidence::Stale => "stale",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "verified" => MemoryConfidence::Verified,
            "candidate" => MemoryConfidence::Candidate,
            "stale" => MemoryConfidence::Stale,
            _ => MemoryConfidence::Likely,
        }
    }
}

impl MemoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryStatus::Active => "active",
            MemoryStatus::Warning => "warning",
            MemoryStatus::Candidate => "candidate",
            MemoryStatus::Contested => "contested",
            MemoryStatus::Superseded => "superseded",
            MemoryStatus::Expired => "expired",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "warning" => MemoryStatus::Warning,
            "candidate" => MemoryStatus::Candidate,
            "contested" => MemoryStatus::Contested,
            "superseded" => MemoryStatus::Superseded,
            "expired" => MemoryStatus::Expired,
            _ => MemoryStatus::Active,
        }
    }
}

impl MemoryAbstraction {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryAbstraction::L0Evidence => "l0_evidence",
            MemoryAbstraction::L1Fact => "l1_fact",
            MemoryAbstraction::L2Decision => "l2_decision",
            MemoryAbstraction::L3Invariant => "l3_invariant",
        }
    }
}

impl MemoryTrustState {
    pub fn as_str(self) -> &'static str {
        match self {
            MemoryTrustState::Candidate => "candidate",
            MemoryTrustState::Verified => "verified",
            MemoryTrustState::Contested => "contested",
            MemoryTrustState::Superseded => "superseded",
            MemoryTrustState::Expired => "expired",
            MemoryTrustState::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(
        kind: MemoryKind,
        confidence: MemoryConfidence,
        status: MemoryStatus,
    ) -> MemoryRecord {
        MemoryRecord {
            id: "record".to_string(),
            scope: MemoryScope::Project,
            project_id: Some("project".to_string()),
            project_slug: Some("project".to_string()),
            kind,
            path: "Facts.md".to_string(),
            title: "Fact".to_string(),
            excerpt: "evidence".to_string(),
            tags: Vec::new(),
            confidence,
            status,
            updated_at: None,
            content_hash: "hash".to_string(),
        }
    }

    #[test]
    fn abstraction_and_trust_are_separate_axes() {
        let verified_fact = record(
            MemoryKind::Fact,
            MemoryConfidence::Verified,
            MemoryStatus::Active,
        );
        assert_eq!(verified_fact.abstraction_level(), MemoryAbstraction::L1Fact);
        assert_eq!(verified_fact.trust_state(), MemoryTrustState::Verified);

        let candidate_decision = record(
            MemoryKind::Decision,
            MemoryConfidence::Candidate,
            MemoryStatus::Candidate,
        );
        assert_eq!(
            candidate_decision.abstraction_level(),
            MemoryAbstraction::L2Decision
        );
        assert_eq!(
            candidate_decision.trust_state(),
            MemoryTrustState::Candidate
        );
    }

    #[test]
    fn consolidation_only_stages_dedup_conflict_and_stale_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        let context = crate::vault::ensure_vault(&vault, &repo).unwrap();
        std::fs::write(
            context.project_root.join("Facts.md"),
            "# Direction\n\n- Verified source uses Rust.\n- Verified source uses Rust.\n- Draft source uses another language.\n",
        )
        .unwrap();
        std::fs::create_dir_all(context.project_root.join("Sessions")).unwrap();
        std::fs::create_dir_all(context.project_root.join("Notes")).unwrap();
        for name in ["duplicate.md", "duplicate-2.md"] {
            std::fs::write(
                context.project_root.join("Notes").join(name),
                "# Direction\n\n- Verified source uses Rust.\n",
            )
            .unwrap();
        }
        std::fs::write(
            context.project_root.join("Sessions/old.md"),
            "# Session\n\n- interrupted attempt must remain evidence\n",
        )
        .unwrap();
        build_memory_index(&context).unwrap();
        let report = analyze_memory_consolidation(&context).unwrap();
        assert!(report.duplicate_groups >= 1);
        assert!(report.superseded_records >= 1);
        assert!(!report.writes_performed);
        let staged = stage_memory_consolidation(&context, &report).unwrap();
        let staged_report: MemoryConsolidationReport =
            serde_json::from_str(&std::fs::read_to_string(&staged).unwrap()).unwrap();
        assert_eq!(
            staged_report.staged_path.as_deref(),
            Some(staged.to_str().unwrap())
        );
        assert!(!staged_report.writes_performed);
    }

    #[test]
    fn frontmatter_preserves_abstraction_tags_and_trust_without_auto_promotion() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("vault");
        std::fs::create_dir_all(&repo).unwrap();
        let context = crate::vault::ensure_vault(&vault, &repo).unwrap();
        std::fs::write(
            context.project_root.join("Decisions.md"),
            "---\nconfidence: candidate\nstatus: contested\ntags: [invariant, owner-review]\n---\n# Current direction\n- Candidate decision must remain reviewable.\n",
        )
        .unwrap();
        build_memory_index(&context).unwrap();
        let records = load_memory_records(&context).unwrap();
        let record = records
            .iter()
            .find(|record| record.excerpt.contains("reviewable"))
            .unwrap();
        assert_eq!(record.abstraction_level(), MemoryAbstraction::L3Invariant);
        assert_eq!(record.trust_state(), MemoryTrustState::Candidate);
        assert!(record.tags.iter().any(|tag| tag == "owner-review"));
    }
}
