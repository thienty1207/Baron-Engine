use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::vault::VaultContext;

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionReplayIndexReport {
    pub indexed_sources: usize,
    pub indexed_messages: usize,
    pub index_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionReplaySearchResult {
    pub message_id: String,
    pub project_id: String,
    pub project_slug: String,
    pub source_path: String,
    pub ordinal: i64,
    pub role: String,
    pub text: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionReplayMessage {
    pub message_id: String,
    pub role: String,
    pub text: String,
    pub source_path: String,
    pub ordinal: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionReplayContext {
    pub project_id: String,
    pub project_slug: String,
    pub source_path: String,
    pub messages: Vec<SessionReplayMessage>,
}

pub fn index_session_replay(vault: &VaultContext) -> Result<SessionReplayIndexReport> {
    if let Some(parent) = session_index_path(vault).parent() {
        fs::create_dir_all(parent)?;
    }
    let connection = Connection::open(session_index_path(vault))?;
    create_schema(&connection)?;
    connection.execute(
        "DELETE FROM session_messages WHERE project_id = ?1",
        [&vault.project_id],
    )?;
    connection.execute(
        "DELETE FROM session_messages_fts WHERE project_id = ?1",
        [&vault.project_id],
    )?;

    let sources = imported_session_sources(vault)?;
    let mut indexed_messages = 0;
    for source in &sources {
        let content = fs::read_to_string(source)
            .with_context(|| format!("Could not read {}", source.display()))?;
        let messages = parse_imported_session(&content);
        let relative = source
            .strip_prefix(&vault.project_root)
            .unwrap_or(source)
            .to_string_lossy()
            .replace('\\', "/");
        for (index, (role, text)) in messages.iter().enumerate() {
            let ordinal = index as i64;
            let message_id = hash(&format!(
                "{}|{}|{}|{}",
                vault.project_id, relative, ordinal, text
            ));
            connection.execute(
                "INSERT OR REPLACE INTO session_messages (
                    message_id, project_id, project_slug, source_path, ordinal, role, text, content_hash
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    message_id,
                    vault.project_id,
                    vault.project_slug,
                    relative,
                    ordinal,
                    role,
                    text,
                    hash(text)
                ],
            )?;
            connection.execute(
                "INSERT INTO session_messages_fts (
                    message_id, project_id, project_slug, source_path, role, text
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    message_id,
                    vault.project_id,
                    vault.project_slug,
                    relative,
                    role,
                    text
                ],
            )?;
            indexed_messages += 1;
        }
    }
    Ok(SessionReplayIndexReport {
        indexed_sources: sources.len(),
        indexed_messages,
        index_path: session_index_path(vault),
    })
}

pub fn search_session_replay(
    vault: &VaultContext,
    query: &str,
    limit: usize,
) -> Result<Vec<SessionReplaySearchResult>> {
    if limit == 0 || !session_index_path(vault).exists() {
        return Ok(Vec::new());
    }
    let terms = query_terms(query);
    if terms.is_empty() {
        return Ok(Vec::new());
    }
    let connection = Connection::open(session_index_path(vault))?;
    create_schema(&connection)?;
    let mut statement = connection.prepare(
        "SELECT message_id, project_id, project_slug, source_path, ordinal, role, text
         FROM session_messages
         WHERE project_id = ?1
         ORDER BY source_path, ordinal",
    )?;
    let rows = statement.query_map([&vault.project_id], |row| {
        Ok(SessionReplaySearchResult {
            message_id: row.get(0)?,
            project_id: row.get(1)?,
            project_slug: row.get(2)?,
            source_path: row.get(3)?,
            ordinal: row.get(4)?,
            role: row.get(5)?,
            text: row.get(6)?,
            score: 0.0,
        })
    })?;
    let mut scored = Vec::new();
    for row in rows {
        let mut item = row?;
        let lower = item.text.to_lowercase();
        let score = terms
            .iter()
            .filter(|term| lower.contains(term.as_str()))
            .count() as f64;
        if score > 0.0 {
            item.score = score;
            scored.push(item);
        }
    }
    scored.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.source_path.cmp(&right.source_path))
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
    scored.truncate(limit);
    Ok(scored)
}

pub fn replay_session_context(
    vault: &VaultContext,
    message_id: &str,
    radius: usize,
) -> Result<SessionReplayContext> {
    if !session_index_path(vault).exists() {
        bail!("Session replay index does not exist. Run `baron session-replay index` first.");
    }
    let connection = Connection::open(session_index_path(vault))?;
    create_schema(&connection)?;
    let mut statement = connection.prepare(
        "SELECT project_id, project_slug, source_path, ordinal
         FROM session_messages
         WHERE message_id = ?1 AND project_id = ?2",
    )?;
    let target = statement
        .query_row(params![message_id, vault.project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .with_context(|| format!("Could not find message id `{message_id}` for current project"))?;
    let radius = radius as i64;
    let mut messages = Vec::new();
    let mut statement = connection.prepare(
        "SELECT message_id, role, text, source_path, ordinal
         FROM session_messages
         WHERE project_id = ?1 AND source_path = ?2 AND ordinal BETWEEN ?3 AND ?4
         ORDER BY ordinal",
    )?;
    let rows = statement.query_map(
        params![
            vault.project_id,
            target.2,
            target.3.saturating_sub(radius),
            target.3.saturating_add(radius)
        ],
        |row| {
            Ok(SessionReplayMessage {
                message_id: row.get(0)?,
                role: row.get(1)?,
                text: row.get(2)?,
                source_path: row.get(3)?,
                ordinal: row.get(4)?,
            })
        },
    )?;
    for row in rows {
        messages.push(row?);
    }
    Ok(SessionReplayContext {
        project_id: target.0,
        project_slug: target.1,
        source_path: target.2,
        messages,
    })
}

pub fn render_session_replay_hits(hits: &[SessionReplaySearchResult]) -> String {
    let mut output = String::from("## Session Replay\n\n");
    if hits.is_empty() {
        output.push_str("- No relevant prior conversation messages found for this project.\n");
        return output;
    }
    for hit in hits.iter().take(3) {
        output.push_str(&format!(
            "- `{}` {}: {}\n",
            hit.source_path,
            hit.role,
            one_line(&hit.text, 220)
        ));
    }
    output.push('\n');
    output
}

fn create_schema(connection: &Connection) -> Result<()> {
    let current: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current != SCHEMA_VERSION {
        connection.execute_batch(
            "DROP TABLE IF EXISTS session_messages;
             DROP TABLE IF EXISTS session_messages_fts;",
        )?;
    }
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS session_messages (
            message_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            project_slug TEXT NOT NULL,
            source_path TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            role TEXT NOT NULL,
            text TEXT NOT NULL,
            content_hash TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS session_messages_project ON session_messages(project_id);
        CREATE INDEX IF NOT EXISTS session_messages_source ON session_messages(project_id, source_path, ordinal);
        CREATE VIRTUAL TABLE IF NOT EXISTS session_messages_fts USING fts5(
            message_id UNINDEXED,
            project_id UNINDEXED,
            project_slug UNINDEXED,
            source_path UNINDEXED,
            role UNINDEXED,
            text
        );",
    )?;
    connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

fn imported_session_sources(vault: &VaultContext) -> Result<Vec<PathBuf>> {
    let root = vault.project_root.join("Sessions/Imported");
    let mut sources = Vec::new();
    collect_markdown(&root, &mut sources)?;
    sources.sort();
    Ok(sources)
}

fn collect_markdown(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown(&path, output)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            output.push(path);
        }
    }
    Ok(())
}

fn parse_imported_session(content: &str) -> Vec<(String, String)> {
    let mut messages = Vec::new();
    let mut current_role: Option<String> = None;
    let mut current = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(role) = heading_role(trimmed) {
            flush_message(&mut messages, &mut current_role, &mut current);
            current_role = Some(role.to_string());
            continue;
        }
        if current_role.is_some() {
            current.push_str(line);
            current.push('\n');
        }
    }
    flush_message(&mut messages, &mut current_role, &mut current);
    messages
}

fn heading_role(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    if lower.starts_with("### user") || lower.starts_with("### human") {
        Some("user")
    } else if lower.starts_with("### assistant") {
        Some("assistant")
    } else {
        None
    }
}

fn flush_message(
    messages: &mut Vec<(String, String)>,
    role: &mut Option<String>,
    content: &mut String,
) {
    let text = content.trim();
    if let Some(role) = role.take() {
        if !text.is_empty() {
            messages.push((role, text.to_string()));
        }
    }
    content.clear();
}

fn session_index_path(vault: &VaultContext) -> PathBuf {
    vault.baron_artifacts_root.join("session-replay.sqlite")
}

fn query_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| !character.is_alphanumeric())
        .map(|term| term.trim().to_lowercase())
        .filter(|term| term.len() >= 2)
        .collect()
}

fn one_line(value: &str, limit: usize) -> String {
    let mut output = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if output.len() > limit {
        output.truncate(limit.saturating_sub(3));
        output.push_str("...");
    }
    output
}

fn hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}
