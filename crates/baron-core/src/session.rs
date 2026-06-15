use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::vault::VaultContext;

const MAX_FILES_CHECKED: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionImportReport {
    pub roots_checked: usize,
    pub files_checked: usize,
    pub imported: usize,
    pub deduplicated: usize,
    pub skipped_unmatched: usize,
    pub skipped_noise: usize,
    pub state_path: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportState {
    last_import: Option<String>,
    sources: BTreeMap<String, ImportedSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImportedSource {
    content_hash: String,
    status: String,
    note_path: Option<String>,
}

#[derive(Debug, Clone)]
struct SessionSource {
    path: PathBuf,
    adapter: &'static str,
    modified: std::time::SystemTime,
}

pub fn import_sessions(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    import_limit: usize,
) -> Result<SessionImportReport> {
    let repo_root = repo_root.as_ref().canonicalize().with_context(|| {
        format!(
            "Could not resolve repo path for session import: {}",
            repo_root.as_ref().display()
        )
    })?;
    let roots = discover_roots();
    let state_path = vault
        .project_root
        .join("Artifacts/session-import-state.json");
    let mut state = load_state(&state_path);
    let max_files = MAX_FILES_CHECKED.min(import_limit.saturating_mul(4).max(20));
    let mut sources = discover_sources(&roots, max_files);
    sources.sort_by_key(|source| Reverse(source.modified));
    sources.truncate(MAX_FILES_CHECKED);
    let mut report = SessionImportReport {
        roots_checked: roots.len(),
        files_checked: 0,
        imported: 0,
        deduplicated: 0,
        skipped_unmatched: 0,
        skipped_noise: 0,
        state_path: state_path.clone(),
    };

    for source in sources {
        if report.imported >= import_limit {
            break;
        }
        report.files_checked += 1;
        let content = match fs::read_to_string(&source.path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let content_hash = hash(&content);
        let source_key = normalize_path(&source.path);
        if state
            .sources
            .get(&source_key)
            .is_some_and(|item| item.content_hash == content_hash)
        {
            report.deduplicated += 1;
            continue;
        }

        let records = parse_records(&content);
        if !records
            .iter()
            .any(|record| value_matches_repo(record, &repo_root))
        {
            report.skipped_unmatched += 1;
            state.sources.insert(
                source_key,
                ImportedSource {
                    content_hash,
                    status: "unmatched".to_string(),
                    note_path: None,
                },
            );
            continue;
        }
        let messages = extract_messages(&records);
        if messages.is_empty() {
            report.skipped_noise += 1;
            state.sources.insert(
                source_key,
                ImportedSource {
                    content_hash,
                    status: "noise".to_string(),
                    note_path: None,
                },
            );
            continue;
        }

        let note_path = imported_note_path(vault, &source, &content_hash);
        let note = render_note(source.adapter, &source.path, &content_hash, &messages);
        write(&note_path, &note)?;
        report.imported += 1;
        state.sources.insert(
            source_key,
            ImportedSource {
                content_hash,
                status: "imported".to_string(),
                note_path: Some(normalize_relative(&note_path, &vault.project_root)),
            },
        );
    }
    state.last_import = Some(now());
    write(
        &state_path,
        &format!("{}\n", serde_json::to_string_pretty(&state)?),
    )?;
    Ok(report)
}

pub fn import_state_summary(vault: &VaultContext) -> Result<(usize, usize, Option<String>)> {
    let path = vault
        .project_root
        .join("Artifacts/session-import-state.json");
    let state = load_state(&path);
    let imported = state
        .sources
        .values()
        .filter(|source| source.status == "imported")
        .count();
    let skipped = state.sources.len().saturating_sub(imported);
    Ok((imported, skipped, state.last_import))
}

fn discover_roots() -> Vec<(PathBuf, &'static str)> {
    let mut roots = Vec::new();
    let codex_override = env_path("BARON_CODEX_SESSIONS_ROOT");
    let claude_override = env_path("BARON_CLAUDE_SESSIONS_ROOT");
    if let Some(path) = codex_override.clone() {
        roots.push((path, "codex"));
    }
    if let Some(path) = claude_override.clone() {
        roots.push((path, "claude"));
    }
    if codex_override.is_none() {
        if let Some(path) = env_path("CODEX_HOME") {
            roots.push((path.join("sessions"), "codex"));
        }
    }
    for home in [env_path("USERPROFILE"), env_path("HOME")]
        .into_iter()
        .flatten()
    {
        if codex_override.is_none() {
            roots.push((home.join(".codex/sessions"), "codex"));
        }
        if claude_override.is_none() {
            roots.push((home.join(".claude/projects"), "claude"));
        }
    }
    if claude_override.is_none() {
        if let Some(path) = env_path("CLAUDE_CONFIG_DIR") {
            roots.push((path.join("projects"), "claude"));
        }
    }
    let mut seen = BTreeSet::new();
    roots
        .into_iter()
        .filter(|(path, adapter)| seen.insert((normalize_path(path), *adapter)))
        .collect()
}

fn discover_sources(roots: &[(PathBuf, &'static str)], max_files: usize) -> Vec<SessionSource> {
    let mut sources = Vec::new();
    for (root, adapter) in roots {
        collect_sources(root, adapter, max_files, &mut sources);
    }
    sources
}

fn collect_sources(
    root: &Path,
    adapter: &'static str,
    max_files: usize,
    sources: &mut Vec<SessionSource>,
) {
    if !root.exists() || sources.len() >= max_files {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| {
        Reverse(
            entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(std::time::UNIX_EPOCH),
        )
    });
    for entry in entries {
        if sources.len() >= max_files {
            break;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_sources(&path, adapter, max_files, sources);
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("jsonl" | "log" | "json")
        ) {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            sources.push(SessionSource {
                path,
                adapter,
                modified,
            });
        }
    }
}

fn parse_records(content: &str) -> Vec<Value> {
    content
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn value_matches_repo(value: &Value, repo_root: &Path) -> bool {
    let repo = normalize_path(repo_root);
    let mut values = Vec::new();
    collect_strings(value, &mut values);
    values.iter().any(|value| {
        let normalized = normalize_text_path(value);
        if normalized == repo || normalized.starts_with(&format!("{repo}/")) {
            return true;
        }
        Path::new(value)
            .canonicalize()
            .ok()
            .map(|candidate| normalize_path(&candidate))
            .is_some_and(|candidate| {
                candidate == repo || candidate.starts_with(&format!("{repo}/"))
            })
    })
}

fn extract_messages(records: &[Value]) -> Vec<(String, String)> {
    let mut messages = Vec::new();
    for record in records {
        let record_type = record
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_lowercase();
        if ["tool", "function", "system", "developer"]
            .iter()
            .any(|kind| record_type.contains(kind))
        {
            continue;
        }
        let role = find_role(record).unwrap_or_default();
        if role != "user" && role != "assistant" {
            continue;
        }
        let Some(content) = find_content(record) else {
            continue;
        };
        let mut strings = Vec::new();
        collect_message_strings(content, &mut strings);
        let text = strings
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if text.is_empty() || text.len() > 8_000 {
            continue;
        }
        if role == "assistant" && !useful_assistant_message(&text) {
            continue;
        }
        messages.push((role.to_string(), redact_secrets(&text)));
    }
    messages
}

fn find_role(value: &Value) -> Option<&str> {
    value
        .get("role")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/payload/role").and_then(Value::as_str))
        .or_else(|| value.pointer("/message/role").and_then(Value::as_str))
}

fn find_content(value: &Value) -> Option<&Value> {
    value
        .get("content")
        .or_else(|| value.pointer("/payload/content"))
        .or_else(|| value.pointer("/message/content"))
}

fn collect_message_strings(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(value) => output.push(value.clone()),
        Value::Array(values) => {
            for value in values {
                let kind = value
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                if !["tool", "function", "image"]
                    .iter()
                    .any(|blocked| kind.contains(blocked))
                {
                    collect_message_strings(value, output);
                }
            }
        }
        Value::Object(values) => {
            if let Some(text) = values.get("text").and_then(Value::as_str) {
                output.push(text.to_string());
            } else {
                for (key, value) in values {
                    if matches!(key.as_str(), "content" | "message" | "output_text") {
                        collect_message_strings(value, output);
                    }
                }
            }
        }
        _ => {}
    }
}

fn collect_strings(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(value) => output.push(value.clone()),
        Value::Array(values) => {
            for value in values {
                collect_strings(value, output);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_strings(value, output);
            }
        }
        _ => {}
    }
}

fn useful_assistant_message(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "decision",
        "decided",
        "next",
        "todo",
        "unresolved",
        "question",
        "implemented",
        "verified",
        "proof",
        "chốt",
        "quyết định",
        "tiếp theo",
        "cần",
    ]
    .iter()
    .any(|term| lower.contains(term))
}

fn redact_secrets(value: &str) -> String {
    let assignment = Regex::new(
        r#"(?i)(api[_-]?key|access[_-]?token|token|secret|password|authorization)\s*[:=]\s*["']?[A-Za-z0-9_\-./+=]{6,}"#,
    )
    .expect("valid secret assignment regex");
    let openai = Regex::new(r"\bsk-[A-Za-z0-9_-]{10,}\b").expect("valid key regex");
    let jwt = Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b")
        .expect("valid jwt regex");
    let value = assignment.replace_all(value, "$1=[REDACTED]");
    let value = openai.replace_all(&value, "[REDACTED]");
    jwt.replace_all(&value, "[REDACTED]").to_string()
}

fn imported_note_path(vault: &VaultContext, source: &SessionSource, content_hash: &str) -> PathBuf {
    let stem = source
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("session");
    let short_hash: String = content_hash.chars().take(12).collect();
    vault.project_root.join("Sessions/Imported").join(format!(
        "{}-{}-{}.md",
        source.adapter,
        slugify(stem),
        short_hash
    ))
}

fn render_note(
    adapter: &str,
    source_path: &Path,
    content_hash: &str,
    messages: &[(String, String)],
) -> String {
    let mut output = format!(
        "---\n\
type: baron-imported-session\n\
adapter: {adapter}\n\
source_hash: {content_hash}\n\
imported: {}\n\
---\n\n\
# Imported {} Session\n\n\
- Source file: `{}`\n\
- Secrets: obvious secret patterns redacted\n\n\
## Clean Conversation Memory\n\n",
        now(),
        adapter,
        source_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("session")
    );
    for (role, message) in messages {
        output.push_str(&format!("### {}\n\n{}\n\n", title_case(role), message));
    }
    output
}

fn load_state(path: &Path) -> ImportState {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn normalize_text_path(value: &str) -> String {
    value.replace('\\', "/").to_lowercase()
}

fn normalize_relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut dash = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            dash = false;
        } else if !dash && !slug.is_empty() {
            slug.push('-');
            dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "session".to_string()
    } else {
        slug
    }
}

fn title_case(value: &str) -> &str {
    match value {
        "assistant" => "Assistant",
        _ => "User",
    }
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}
