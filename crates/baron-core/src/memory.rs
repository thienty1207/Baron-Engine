use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{params, Connection, Transaction};
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
    let connection = Connection::open(&context.index_path)
        .with_context(|| format!("Could not open {}", context.index_path.display()))?;
    create_schema(&connection)?;
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
    let updated_at = metadata
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .map(|value| value.to_rfc3339_opts(SecondsFormat::Secs, true));
    for line in content.lines() {
        let trimmed = line.trim();
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
            || excerpt.len() < 8
        {
            continue;
        }
        let confidence = classify_confidence(source.scope, source.kind, excerpt);
        let status = match confidence {
            MemoryConfidence::Candidate => MemoryStatus::Candidate,
            MemoryConfidence::Stale => MemoryStatus::Warning,
            MemoryConfidence::Verified | MemoryConfidence::Likely => MemoryStatus::Active,
        };
        let id_source = format!(
            "{}|{:?}|{:?}|{}|{}",
            source.relative_path, source.scope, source.project_id, title, excerpt
        );
        let content_hash = hash(&id_source);
        records.push(MemoryRecord {
            id: content_hash.clone(),
            scope: source.scope,
            project_id: source.project_id.clone(),
            project_slug: source.project_slug.clone(),
            kind: source.kind,
            path: source.relative_path.clone(),
            title: title.clone(),
            excerpt: excerpt.to_string(),
            tags: tags_for(source.kind, confidence),
            confidence,
            status,
            updated_at: updated_at.clone(),
            content_hash,
        });
    }
    records
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
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "warning" => MemoryStatus::Warning,
            "candidate" => MemoryStatus::Candidate,
            _ => MemoryStatus::Active,
        }
    }
}
