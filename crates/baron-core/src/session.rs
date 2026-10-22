use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::safe_io::{acquire_project_lock, create_new_text, read_text, replace_text};
use crate::vault::VaultContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionImportReport {
    pub roots_checked: usize,
    #[serde(default)]
    pub files_discovered: usize,
    pub files_checked: usize,
    pub imported: usize,
    pub deduplicated: usize,
    pub skipped_unmatched: usize,
    pub skipped_noise: usize,
    pub learning_candidates: usize,
    pub learning_error: Option<String>,
    #[serde(default)]
    pub learning_v42_candidates: usize,
    #[serde(default)]
    pub learning_v42_quarantined: usize,
    #[serde(default)]
    pub learning_v42_error: Option<String>,
    #[serde(default)]
    pub learning_v42_path: Option<PathBuf>,
    #[serde(default)]
    pub skipped_read_errors: usize,
    #[serde(default)]
    pub invalid_records: usize,
    #[serde(default)]
    pub omissions: Vec<String>,
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

enum PreparedImportAction {
    Unmatched,
    Noise,
    Imported {
        note_path: PathBuf,
        note: String,
        adapter: &'static str,
    },
}

struct PreparedSessionImport {
    source_key: String,
    content_hash: String,
    action: PreparedImportAction,
}

pub fn import_sessions(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    import_limit: usize,
) -> Result<SessionImportReport> {
    let roots = discover_roots();
    let mut report = import_sessions_from_roots(
        repo_root.as_ref(),
        vault,
        import_limit,
        roots,
        || {},
        |_| Ok(()),
    )?;
    match crate::intelligence41::learn_session_candidates(vault) {
        Ok(learning) => report.learning_candidates = learning.candidates.len(),
        Err(error) => report.learning_error = Some(error.to_string()),
    }
    match crate::intelligence42::learn_session_candidates_v42(vault) {
        Ok(learning) => {
            report.learning_v42_candidates = learning.candidates.len();
            report.learning_v42_quarantined = learning.quarantined;
            report.learning_v42_path = Some(PathBuf::from(learning.output_path));
        }
        Err(error) => report.learning_v42_error = Some(error.to_string()),
    }
    Ok(report)
}

fn import_sessions_from_roots<F, G>(
    repo_root: &Path,
    vault: &VaultContext,
    import_limit: usize,
    roots: Vec<(PathBuf, &'static str)>,
    before_state_publish: F,
    mut after_note_publish: G,
) -> Result<SessionImportReport>
where
    F: FnOnce(),
    G: FnMut(&Path) -> Result<()>,
{
    let repo_root = repo_root.canonicalize().with_context(|| {
        format!(
            "Could not resolve repo path for session import: {}",
            repo_root.display()
        )
    })?;
    let mut sources = discover_sources(&roots);
    sources.sort_by_key(|source| Reverse(source.modified));
    let roots_checked = roots.len();
    let state_path = vault
        .project_root
        .join("Artifacts/session-import-state.json");
    let initial_state = {
        let _repo_lock = acquire_project_lock(&repo_root)?;
        let _vault_lock = acquire_project_lock(&vault.project_root)?;
        load_state(&state_path)?
    };
    let mut report = SessionImportReport {
        roots_checked,
        files_discovered: sources.len(),
        files_checked: 0,
        imported: 0,
        deduplicated: 0,
        skipped_unmatched: 0,
        skipped_noise: 0,
        learning_candidates: 0,
        learning_error: None,
        learning_v42_candidates: 0,
        learning_v42_quarantined: 0,
        learning_v42_error: None,
        learning_v42_path: None,
        skipped_read_errors: 0,
        invalid_records: 0,
        omissions: Vec::new(),
        state_path: state_path.clone(),
    };
    let mut prepared = Vec::new();
    let mut prepared_imports = 0usize;

    for source in sources {
        if prepared_imports >= import_limit {
            break;
        }
        report.files_checked += 1;
        let content = match fs::read_to_string(&source.path) {
            Ok(content) => content,
            Err(error) => {
                report.skipped_read_errors += 1;
                report.omissions.push(format!(
                    "{}:read-error:{}",
                    normalize_path(&source.path),
                    error
                ));
                continue;
            }
        };
        let content_hash = hash(&content);
        let source_key = normalize_path(&source.path);
        if initial_state
            .sources
            .get(&source_key)
            .is_some_and(|item| item.content_hash == content_hash)
        {
            report.deduplicated += 1;
            continue;
        }

        let (records, invalid_records) = parse_records(&content);
        report.invalid_records = report.invalid_records.saturating_add(invalid_records);
        // Trusted cwd/repo/project metadata establishes the project for the
        // following user/assistant records. Message bodies never establish a
        // project binding; a new context record closes the active segment.
        let mut active_repo_segment = false;
        let mut matching_records = Vec::new();
        for record in &records {
            if value_matches_repo(record, &repo_root) {
                active_repo_segment = true;
                matching_records.push(record.clone());
            } else if is_session_context_record(record) {
                active_repo_segment = false;
            } else if active_repo_segment {
                matching_records.push(record.clone());
            }
        }
        let action = if matching_records.is_empty() {
            PreparedImportAction::Unmatched
        } else {
            let messages = extract_messages(&matching_records);
            if messages.is_empty() {
                PreparedImportAction::Noise
            } else {
                prepared_imports += 1;
                let note_path = imported_note_path(vault, &source, &content_hash);
                let note = render_note(source.adapter, &source.path, &content_hash, &messages);
                PreparedImportAction::Imported {
                    note_path,
                    note,
                    adapter: source.adapter,
                }
            }
        };
        prepared.push(PreparedSessionImport {
            source_key,
            content_hash,
            action,
        });
    }

    // The test synchronizes here to force both callers to prepare from the
    // same earlier state snapshot. Production passes a no-op, and this hook is
    // outside the mutation locks so it cannot extend their critical section.
    before_state_publish();

    let _repo_lock = acquire_project_lock(&repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let mut state = load_state(&state_path)?;
    let mut folded_source_counts = BTreeMap::new();
    for item in &prepared {
        *folded_source_counts
            .entry(item.source_key.to_lowercase())
            .or_insert(0usize) += 1;
    }
    for item in prepared {
        if report.imported >= import_limit {
            break;
        }
        if state
            .sources
            .get(&item.source_key)
            .is_some_and(|existing| existing.content_hash == item.content_hash)
        {
            report.deduplicated += 1;
            continue;
        }
        let folded_key = item.source_key.to_lowercase();
        let expected_note_path = match &item.action {
            PreparedImportAction::Imported { note_path, .. } => {
                Some(normalize_relative(note_path, &vault.project_root))
            }
            PreparedImportAction::Unmatched | PreparedImportAction::Noise => None,
        };
        if folded_source_counts.get(&folded_key) == Some(&1) {
            if let Some(legacy) = state.sources.get(&folded_key) {
                if legacy.content_hash == item.content_hash
                    && legacy.note_path == expected_note_path
                {
                    let migrated = state
                        .sources
                        .remove(&folded_key)
                        .expect("the matching legacy session source was just observed");
                    state.sources.insert(item.source_key, migrated);
                    report.deduplicated += 1;
                    continue;
                }
            }
        }
        let source_state = match item.action {
            PreparedImportAction::Unmatched => {
                report.skipped_unmatched += 1;
                ImportedSource {
                    content_hash: item.content_hash,
                    status: "unmatched".to_string(),
                    note_path: None,
                }
            }
            PreparedImportAction::Noise => {
                report.skipped_noise += 1;
                ImportedSource {
                    content_hash: item.content_hash,
                    status: "noise".to_string(),
                    note_path: None,
                }
            }
            PreparedImportAction::Imported {
                note_path,
                note,
                adapter,
            } => {
                publish_imported_note(&note_path, &note, adapter, &item.content_hash)?;
                after_note_publish(&note_path)?;
                report.imported += 1;
                ImportedSource {
                    content_hash: item.content_hash,
                    status: "imported".to_string(),
                    note_path: Some(normalize_relative(&note_path, &vault.project_root)),
                }
            }
        };
        state.sources.insert(item.source_key, source_state);
    }
    state.last_import = Some(now());
    write(
        &state_path,
        &format!("{}\n", serde_json::to_string_pretty(&state)?),
    )?;
    report.omissions.sort();
    report.omissions.dedup();
    Ok(report)
}

pub fn import_state_summary(vault: &VaultContext) -> Result<(usize, usize, Option<String>)> {
    let path = vault
        .project_root
        .join("Artifacts/session-import-state.json");
    let _repo_lock = acquire_project_lock(&vault.repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let state = load_state(&path)?;
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

fn discover_sources(roots: &[(PathBuf, &'static str)]) -> Vec<SessionSource> {
    let mut sources = Vec::new();
    for (root, adapter) in roots {
        collect_sources(root, adapter, &mut sources);
    }
    let mut seen = BTreeSet::new();
    sources.retain(|source| seen.insert((normalize_path(&source.path), source.adapter)));
    sources
}

fn collect_sources(root: &Path, adapter: &'static str, sources: &mut Vec<SessionSource>) {
    if !root.exists() {
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
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_sources(&path, adapter, sources);
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

fn parse_records(content: &str) -> (Vec<Value>, usize) {
    let mut records = Vec::new();
    let mut invalid = 0usize;
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => records.push(value),
            Err(_) => invalid = invalid.saturating_add(1),
        }
    }
    (records, invalid)
}

fn value_matches_repo(value: &Value, repo_root: &Path) -> bool {
    if !is_session_context_record(value) {
        return false;
    }

    let record_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let has_session_payload = matches!(
        record_type.as_str(),
        "session_meta" | "session_start" | "session"
    );
    let mut paths = Vec::new();
    let mut malformed = false;
    collect_context_path_fields(value, &["cwd", "repo"], &mut paths, &mut malformed);
    collect_project_path_field(value, &mut paths, &mut malformed);
    if has_session_payload {
        if let Some(payload) = value.get("payload") {
            collect_context_path_fields(payload, &["cwd", "repo"], &mut paths, &mut malformed);
            collect_project_path_field(payload, &mut paths, &mut malformed);
        }
    }

    !malformed
        && !paths.is_empty()
        && paths
            .into_iter()
            .all(|path| path_matches_repo(path, repo_root))
}

fn is_session_context_record(value: &Value) -> bool {
    let record_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    record_type.contains("session")
        || value.get("cwd").is_some()
        || value.get("repo").is_some()
        || value.get("project").is_some()
}

fn collect_context_path_fields<'a>(
    value: &'a Value,
    fields: &[&str],
    output: &mut Vec<&'a str>,
    malformed: &mut bool,
) {
    for field in fields {
        let Some(value) = value.get(field) else {
            continue;
        };
        match value
            .as_str()
            .map(str::trim)
            .filter(|path| !path.is_empty())
        {
            Some(path) => output.push(path),
            None => *malformed = true,
        }
    }
}

fn collect_project_path_field<'a>(
    value: &'a Value,
    output: &mut Vec<&'a str>,
    malformed: &mut bool,
) {
    let Some(project) = value.get("project") else {
        return;
    };
    match project.as_str().map(str::trim) {
        Some(path) if Path::new(path).is_absolute() => output.push(path),
        Some(_) => {} // Project labels are not filesystem path authority.
        None => *malformed = true,
    }
}

fn path_matches_repo(value: &str, repo_root: &Path) -> bool {
    let candidate = Path::new(value);
    if !candidate.is_absolute() {
        return false;
    }
    candidate
        .canonicalize()
        .ok()
        .is_some_and(|candidate| candidate == repo_root || candidate.starts_with(repo_root))
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

fn publish_imported_note(path: &Path, note: &str, adapter: &str, source_hash: &str) -> Result<()> {
    match read_text(path)? {
        Some(existing) => {
            if !imported_note_matches(&existing, adapter, source_hash) {
                bail!(
                    "Existing session import note has conflicting identity; refusing to overwrite: {}",
                    path.display()
                );
            }
        }
        None => create_new_text(path, note).with_context(|| {
            format!(
                "Could not publish imported session note: {}",
                path.display()
            )
        })?,
    }
    Ok(())
}

fn imported_note_matches(content: &str, adapter: &str, source_hash: &str) -> bool {
    let Some(front_matter) = content
        .strip_prefix("---\n")
        .and_then(|content| content.split_once("\n---\n"))
        .map(|(front_matter, _)| front_matter)
    else {
        return false;
    };
    let mut identity = BTreeMap::new();
    for line in front_matter.lines() {
        let Some((key @ ("type" | "adapter" | "source_hash"), value)) = line.split_once(": ")
        else {
            continue;
        };
        if identity.insert(key, value).is_some() {
            return false;
        }
    }
    identity.get("type") == Some(&"baron-imported-session")
        && identity.get("adapter") == Some(&adapter)
        && identity.get("source_hash") == Some(&source_hash)
}

fn load_state(path: &Path) -> Result<ImportState> {
    let Some(content) = read_text(path)? else {
        return Ok(ImportState::default());
    };
    serde_json::from_str(&content)
        .with_context(|| format!("Could not parse session import state: {}", path.display()))
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn normalize_path(path: &Path) -> String {
    // v1 persisted keys lowercased paths. New keys use canonical path spelling
    // so case-sensitive roots stay distinct. A legacy key is migrated only
    // when one current source matches its folded spelling and both its content
    // hash and destination note identity still match; ambiguous legacy entries
    // remain inert and create-only note publication makes a retry safe.
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use super::{import_sessions_from_roots, load_state};
    use crate::vault::ensure_vault;
    use tempfile::tempdir;

    fn enable_case_sensitive_directory(path: &std::path::Path) -> bool {
        #[cfg(windows)]
        {
            path.to_str().is_some_and(|path| {
                std::process::Command::new("fsutil.exe")
                    .args(["file", "setCaseSensitiveInfo"])
                    .arg(path)
                    .arg("enable")
                    .output()
                    .is_ok_and(|output| output.status.success())
            })
        }
        #[cfg(not(windows))]
        {
            let _ = path;
            true
        }
    }

    #[test]
    fn concurrent_imports_preserve_disjoint_source_union() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("Vault");
        let root_a = temp.path().join("source-a");
        let root_b = temp.path().join("source-b");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&root_a).unwrap();
        fs::create_dir_all(&root_b).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let repo_text = repo.canonicalize().unwrap().to_string_lossy().to_string();
        for (root, name, message) in [
            (&root_a, "a.jsonl", "Remember source A decision"),
            (&root_b, "b.jsonl", "Remember source B decision"),
        ] {
            fs::write(
                root.join(name),
                format!(
                    "{}\n{}\n",
                    serde_json::json!({"type":"session_meta","payload":{"cwd":repo_text}}),
                    serde_json::json!({"type":"response_item","payload":{"role":"user","content":message}})
                ),
            )
            .unwrap();
        }
        let barrier = Arc::new(Barrier::new(2));
        let handles = [root_a.clone(), root_b.clone()]
            .into_iter()
            .map(|root| {
                let repo = repo.clone();
                let context = context.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    import_sessions_from_roots(
                        &repo,
                        &context,
                        20,
                        vec![(root, "codex")],
                        move || {
                            barrier.wait();
                        },
                        |_| Ok(()),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }

        let state_path = context
            .project_root
            .join("Artifacts/session-import-state.json");
        let state = load_state(&state_path).unwrap();
        assert_eq!(state.sources.len(), 2);
        assert!(state
            .sources
            .values()
            .all(|source| source.status == "imported"));
        for root in [root_a, root_b] {
            let retry = import_sessions_from_roots(
                &repo,
                &context,
                20,
                vec![(root, "codex")],
                || {},
                |_| Ok(()),
            )
            .unwrap();
            assert_eq!(retry.imported, 0);
            assert_eq!(retry.deduplicated, 1);
        }
    }

    #[test]
    fn retry_after_interruption_preserves_an_edited_imported_note() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("Vault");
        let root = temp.path().join("sessions");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&root).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let repo_text = repo.canonicalize().unwrap().to_string_lossy().to_string();
        fs::write(
            root.join("interrupted.jsonl"),
            format!(
                "{}\n{}\n",
                serde_json::json!({"type":"session_meta","payload":{"cwd":repo_text}}),
                serde_json::json!({"type":"response_item","payload":{"role":"user","content":"Remember the imported decision"}})
            ),
        )
        .unwrap();

        let state_path = context
            .project_root
            .join("Artifacts/session-import-state.json");
        let interrupted = import_sessions_from_roots(
            &repo,
            &context,
            20,
            vec![(root.clone(), "codex")],
            || {},
            |note_path| {
                let mut edited = fs::read_to_string(note_path).unwrap();
                edited.push_str("\nUser addition before retry.\n");
                fs::write(note_path, edited).unwrap();
                Err(anyhow::anyhow!(
                    "simulated interruption after note publication"
                ))
            },
        );
        assert!(interrupted.is_err());
        assert!(!state_path.exists(), "interrupted state was published");

        let notes_dir = context.project_root.join("Sessions/Imported");
        let note_path = fs::read_dir(&notes_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let edited_note = fs::read(&note_path).unwrap();
        assert!(String::from_utf8_lossy(&edited_note).contains("User addition before retry."));

        let retry = import_sessions_from_roots(
            &repo,
            &context,
            20,
            vec![(root, "codex")],
            || {},
            |_| Ok(()),
        )
        .unwrap();

        assert_eq!(retry.imported, 1);
        assert_eq!(fs::read(&note_path).unwrap(), edited_note);
        assert_eq!(load_state(&state_path).unwrap().sources.len(), 1);
    }

    #[test]
    fn case_distinct_session_sources_keep_separate_import_state_keys() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("Vault");
        let roots = temp.path().join("source-roots");
        fs::create_dir_all(&roots).unwrap();
        let case_sensitive_enabled = enable_case_sensitive_directory(&roots);
        #[cfg(windows)]
        if !case_sensitive_enabled {
            eprintln!(
                "Skipping distinct-case session source assertions: Windows could not enable case-sensitive directory semantics for {}",
                roots.display()
            );
        }
        let source_upper = roots.join("Sessions");
        let source_lower = roots.join("sessions");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&source_upper).unwrap();
        fs::create_dir_all(&source_lower).unwrap();
        let distinct_sources =
            source_upper.canonicalize().unwrap() != source_lower.canonicalize().unwrap();
        if case_sensitive_enabled {
            assert!(
                distinct_sources,
                "case-sensitive directory setup did not create distinct sources"
            );
        }
        let context = ensure_vault(&vault, &repo).unwrap();
        let repo_text = repo.canonicalize().unwrap().to_string_lossy().to_string();
        for (root, message) in [
            (&source_upper, "Remember the upper-case source identity"),
            (&source_lower, "Remember the lower-case source identity"),
        ] {
            fs::write(
                root.join("session.jsonl"),
                format!(
                    "{}\n{}\n",
                    serde_json::json!({"type":"session_meta","payload":{"cwd":repo_text}}),
                    serde_json::json!({"type":"response_item","payload":{"role":"user","content":message}})
                ),
            )
            .unwrap();
        }

        let roots = vec![(source_upper, "codex"), (source_lower, "codex")];
        let first =
            import_sessions_from_roots(&repo, &context, 20, roots.clone(), || {}, |_| Ok(()))
                .unwrap();
        let state_path = context
            .project_root
            .join("Artifacts/session-import-state.json");
        let state = load_state(&state_path).unwrap();
        let retry =
            import_sessions_from_roots(&repo, &context, 20, roots, || {}, |_| Ok(())).unwrap();

        let expected_sources = if distinct_sources { 2 } else { 1 };
        assert_eq!(first.imported, expected_sources);
        assert_eq!(state.sources.len(), expected_sources);
        assert_eq!(retry.imported, 0);
        assert_eq!(retry.deduplicated, expected_sources);
    }

    #[test]
    fn legacy_folded_source_key_migrates_without_replacing_edited_note() {
        let temp = tempdir().unwrap();
        let repo = temp.path().join("repo");
        let vault = temp.path().join("Vault");
        let root = temp.path().join("LegacySessions");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&root).unwrap();
        let context = ensure_vault(&vault, &repo).unwrap();
        let repo_text = repo.canonicalize().unwrap().to_string_lossy().to_string();
        fs::write(
            root.join("session.jsonl"),
            format!(
                "{}\n{}\n",
                serde_json::json!({"type":"session_meta","payload":{"cwd":repo_text}}),
                serde_json::json!({"type":"response_item","payload":{"role":"user","content":"Remember the migrated session identity"}})
            ),
        )
        .unwrap();
        let roots = vec![(root, "codex")];
        let first =
            import_sessions_from_roots(&repo, &context, 20, roots.clone(), || {}, |_| Ok(()))
                .unwrap();
        assert_eq!(first.imported, 1);

        let state_path = context
            .project_root
            .join("Artifacts/session-import-state.json");
        let mut state = load_state(&state_path).unwrap();
        let (current_key, imported_source) = state.sources.pop_first().unwrap();
        let legacy_key = current_key.to_lowercase();
        assert_ne!(current_key, legacy_key);
        let note_path = context
            .project_root
            .join(imported_source.note_path.as_deref().unwrap());
        let mut edited_note = fs::read_to_string(&note_path).unwrap();
        edited_note.push_str("\nUser edit preserved across state-key migration.\n");
        fs::write(&note_path, &edited_note).unwrap();
        state.sources.insert(legacy_key.clone(), imported_source);
        fs::write(
            &state_path,
            format!("{}\n", serde_json::to_string_pretty(&state).unwrap()),
        )
        .unwrap();

        let retry =
            import_sessions_from_roots(&repo, &context, 20, roots, || {}, |_| Ok(())).unwrap();
        let migrated = load_state(&state_path).unwrap();

        assert_eq!(retry.imported, 0);
        assert_eq!(retry.deduplicated, 1);
        assert_eq!(fs::read_to_string(&note_path).unwrap(), edited_note);
        assert!(migrated.sources.contains_key(&current_key));
        assert!(!migrated.sources.contains_key(&legacy_key));
    }
}
