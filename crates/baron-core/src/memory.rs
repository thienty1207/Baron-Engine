use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::vault::VaultContext;

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
    pub total_records: usize,
    pub current_project_records: usize,
    pub global_verified_records: usize,
    pub global_candidate_records: usize,
    pub cross_project_records: usize,
}

pub fn build_memory_index(context: &VaultContext) -> Result<MemoryIndexReport> {
    if let Some(parent) = context.index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let records = scan_vault_records(context)?;
    let connection = Connection::open(&context.index_path)
        .with_context(|| format!("Could not open {}", context.index_path.display()))?;
    create_schema(&connection)?;
    connection.execute("DELETE FROM records", [])?;
    for record in &records {
        connection.execute(
            "INSERT INTO records (
                id, scope, project_slug, kind, path, title, excerpt, tags,
                confidence, status, updated_at, content_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                record.id,
                record.scope.as_str(),
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

    Ok(MemoryIndexReport {
        total_records: records.len(),
        current_project_records: records
            .iter()
            .filter(|record| record.project_slug.as_deref() == Some(&context.project_slug))
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
                    && record.project_slug.as_deref() != Some(&context.project_slug)
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
        "SELECT id, scope, project_slug, kind, path, title, excerpt, tags,
                confidence, status, updated_at, content_hash
         FROM records",
    )?;
    let rows = statement.query_map([], |row| {
        let tags: String = row.get(7)?;
        Ok(MemoryRecord {
            id: row.get(0)?,
            scope: MemoryScope::from_str(&row.get::<_, String>(1)?),
            project_slug: row.get(2)?,
            kind: MemoryKind::from_str(&row.get::<_, String>(3)?),
            path: row.get(4)?,
            title: row.get(5)?,
            excerpt: row.get(6)?,
            tags: tags
                .split(',')
                .filter(|tag| !tag.is_empty())
                .map(ToString::to_string)
                .collect(),
            confidence: MemoryConfidence::from_str(&row.get::<_, String>(8)?),
            status: MemoryStatus::from_str(&row.get::<_, String>(9)?),
            updated_at: row.get(10)?,
            content_hash: row.get(11)?,
        })
    })?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

fn create_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS records (
            id TEXT PRIMARY KEY,
            scope TEXT NOT NULL,
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
        );",
    )?;
    Ok(())
}

fn scan_vault_records(context: &VaultContext) -> Result<Vec<MemoryRecord>> {
    let mut records = Vec::new();
    let projects_root = context.vault_root.join("Projects");
    if projects_root.exists() {
        for entry in fs::read_dir(&projects_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let project_slug = entry.file_name().to_string_lossy().to_string();
            records.extend(scan_project_records(context, &project_slug, &entry.path())?);
        }
    }
    records.extend(scan_global_records(
        context,
        &context.approved_global_path,
        MemoryScope::GlobalVerified,
    )?);
    records.extend(scan_global_records(
        context,
        &context.global_candidates_path,
        MemoryScope::GlobalCandidate,
    )?);
    Ok(records)
}

fn scan_project_records(
    context: &VaultContext,
    project_slug: &str,
    project_root: &Path,
) -> Result<Vec<MemoryRecord>> {
    let mut files = Vec::new();
    for (path, kind) in [
        (project_root.join("Facts.md"), MemoryKind::Fact),
        (project_root.join("Decisions.md"), MemoryKind::Decision),
        (project_root.join("Tasks.md"), MemoryKind::Task),
    ] {
        files.push((path, kind));
    }
    collect_markdown_files(&project_root.join("Plans"), MemoryKind::Plan, &mut files)?;
    collect_markdown_files(
        &project_root.join("ProductHarness"),
        MemoryKind::Harness,
        &mut files,
    )?;
    collect_markdown_files(
        &project_root.join("Sessions"),
        MemoryKind::Session,
        &mut files,
    )?;
    collect_markdown_files(
        &project_root.join("Research"),
        MemoryKind::Research,
        &mut files,
    )?;
    collect_markdown_files(&project_root.join("Notes"), MemoryKind::Note, &mut files)?;
    files.push((project_root.join("Open Questions.md"), MemoryKind::Question));
    files.push((project_root.join("Handoff.md"), MemoryKind::Handoff));
    collect_markdown_files(&project_root.join("Proofs"), MemoryKind::Proof, &mut files)?;
    collect_markdown_files(&project_root.join("Traces"), MemoryKind::Trace, &mut files)?;

    let mut records = Vec::new();
    for (path, kind) in files.into_iter().take(200) {
        records.extend(scan_markdown_file(
            context,
            &path,
            MemoryScope::Project,
            Some(project_slug.to_string()),
            kind,
        )?);
    }
    Ok(records)
}

fn scan_global_records(
    context: &VaultContext,
    path: &Path,
    scope: MemoryScope,
) -> Result<Vec<MemoryRecord>> {
    scan_markdown_file(context, path, scope, None, MemoryKind::Global)
}

fn collect_markdown_files(
    root: &Path,
    kind: MemoryKind,
    files: &mut Vec<(PathBuf, MemoryKind)>,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown_files(&path, kind, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push((path, kind));
        }
    }
    Ok(())
}

fn scan_markdown_file(
    context: &VaultContext,
    path: &Path,
    scope: MemoryScope,
    project_slug: Option<String>,
    kind: MemoryKind,
) -> Result<Vec<MemoryRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    let mut title = String::from("Memory");
    let mut records = Vec::new();
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
        {
            continue;
        }
        if excerpt.len() < 8 {
            continue;
        }
        records.push(record(
            context,
            scope,
            project_slug.clone(),
            kind,
            path,
            &title,
            excerpt,
        ));
    }
    Ok(records)
}

fn record(
    context: &VaultContext,
    scope: MemoryScope,
    project_slug: Option<String>,
    kind: MemoryKind,
    path: &Path,
    title: &str,
    excerpt: &str,
) -> MemoryRecord {
    let relative_path = path
        .strip_prefix(&context.vault_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let confidence = classify_confidence(scope, kind, excerpt);
    let status = match confidence {
        MemoryConfidence::Candidate => MemoryStatus::Candidate,
        MemoryConfidence::Stale => MemoryStatus::Warning,
        MemoryConfidence::Verified | MemoryConfidence::Likely => MemoryStatus::Active,
    };
    let id_source = format!(
        "{:?}|{:?}|{}|{}",
        scope, project_slug, relative_path, excerpt
    );
    let content_hash = hash(&id_source);
    MemoryRecord {
        id: content_hash.clone(),
        scope,
        project_slug,
        kind,
        path: relative_path,
        title: title.to_string(),
        excerpt: excerpt.to_string(),
        tags: tags_for(kind, confidence),
        confidence,
        status,
        updated_at: None,
        content_hash,
    }
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
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
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
