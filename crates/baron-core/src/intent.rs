use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::operation::LifecycleIdentity;
use crate::risk::{classify_risk, RiskLane};
use crate::safe_io::{acquire_project_lock, append_text, read_text, replace_text};
use crate::vault::VaultContext;

pub const MAX_OPERATION_INTENT_CHARS: usize = 3_600;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IntentBriefInput {
    pub title: String,
    pub current_behavior: String,
    pub target_behavior: String,
    pub scope: String,
    pub non_goals: Vec<String>,
    pub constraints: Vec<String>,
    pub decisions: Vec<String>,
    pub required_proof: String,
    pub unknowns: Vec<String>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentBrief {
    pub id: String,
    pub title: String,
    pub risk: RiskLane,
    pub confirmed: bool,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
}

pub fn record_intent(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    input: IntentBriefInput,
) -> Result<IntentBrief> {
    record_intent_internal(repo_root.as_ref(), vault, input, None)
}

/// Record a confirmed-intent candidate bound to one complete operation.
/// `CURRENT_INTENT.md` remains a presentation projection; identified consumers
/// read the durable per-operation copy instead.
pub fn record_intent_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    canonical_task: &str,
    identity: &LifecycleIdentity,
    input: IntentBriefInput,
) -> Result<IntentBrief> {
    if identity.project_id() != vault.project_id {
        bail!(
            "intent lifecycle identity project `{}` does not match Vault project `{}`",
            identity.project_id(),
            vault.project_id
        );
    }
    identity
        .validate_task(canonical_task)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    record_intent_internal(repo_root.as_ref(), vault, input, Some(identity))
}

/// Return the exact durable intent for an operation, never falling back to
/// the shared CURRENT projection. A present file with a missing or conflicting
/// binding is corrupt authority and therefore an error.
pub fn operation_intent_for_identity(
    repo_root: impl AsRef<Path>,
    identity: &LifecycleIdentity,
    max_chars: usize,
) -> Result<Option<String>> {
    let path = operation_intent_path(repo_root, identity);
    let Some(content) = read_text(&path)? else {
        return Ok(None);
    };
    if !intent_header_matches_identity(&content, identity) {
        bail!(
            "operation-scoped intent does not match its lifecycle identity: {}",
            path.display()
        );
    }
    let authority_limit = max_chars.min(MAX_OPERATION_INTENT_CHARS);
    if content.chars().count() > authority_limit {
        bail!(
            "operation-scoped intent exceeds its bounded authority limit of {authority_limit} characters"
        );
    }
    Ok(Some(content))
}

pub fn operation_intent_path(
    repo_root: impl AsRef<Path>,
    identity: &LifecycleIdentity,
) -> std::path::PathBuf {
    repo_root
        .as_ref()
        .join("docs/baron/harness/intent-operations")
        .join(format!("{}.md", identity.operation_id()))
}

fn record_intent_internal(
    repo_root: &Path,
    vault: &VaultContext,
    mut input: IntentBriefInput,
    identity: Option<&LifecycleIdentity>,
) -> Result<IntentBrief> {
    normalize_input(&mut input);
    validate_input(&input)?;
    let risk = classify_risk(&input.title);
    let date = today();
    let id = match identity {
        Some(identity) => intent_id_for_operation(&input, identity)?,
        None => intent_id(&input)?,
    };
    let filename = format!("{}-{}.md", slugify(&input.title), id);
    let repo_path = repo_root
        .join("docs/baron/harness/intents")
        .join(&date)
        .join(&filename);
    let vault_path = vault
        .project_root
        .join("ProductHarness/Intents")
        .join(&date)
        .join(&filename);
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let operation_path = identity.map(|identity| operation_intent_path(repo_root, identity));
    let operation_vault_path = identity.map(|identity| {
        vault
            .project_root
            .join("ProductHarness/IntentOperations")
            .join(format!("{}.md", identity.operation_id()))
    });
    for path in operation_path.iter().chain(operation_vault_path.iter()) {
        if let Some(existing) = read_text(path)? {
            if !identity.is_some_and(|identity| intent_header_matches_identity(&existing, identity))
            {
                bail!(
                    "existing operation-scoped intent has a conflicting lifecycle identity: {}",
                    path.display()
                );
            }
        }
    }
    let resumed = repo_path.is_file();
    let content = render_intent(&id, &input, risk, identity);
    if identity.is_some() && content.chars().count() > MAX_OPERATION_INTENT_CHARS {
        bail!(
            "operation-scoped intent exceeds its bounded authority limit of {MAX_OPERATION_INTENT_CHARS} characters"
        );
    }
    if !resumed {
        write(&repo_path, &content)?;
        write(&vault_path, &content)?;
        append_unique(
            &repo_root.join("docs/baron/harness/INTENTS.md"),
            "# Baron Intent History\n\n",
            &format!(
                "- [{}]({}) - risk: `{}` - confirmation: `{}`",
                input.title,
                normalize_path(&repo_path, repo_root),
                risk.as_str(),
                confirmation(input.confirmed)
            ),
        )?;
        append_unique(
            &vault.project_root.join("ProductHarness/INTENTS.md"),
            "# Baron Intent History\n\n",
            &format!(
                "- [{}]({}) - risk: `{}` - confirmation: `{}`",
                input.title,
                normalize_path(&vault_path, &vault.project_root),
                risk.as_str(),
                confirmation(input.confirmed)
            ),
        )?;
    }
    if let Some(path) = operation_path {
        write(&path, &content)?;
    }
    if let Some(path) = operation_vault_path {
        write(&path, &content)?;
    }
    write(
        &repo_root.join("docs/baron/harness/CURRENT_INTENT.md"),
        &content,
    )?;
    write(
        &vault.project_root.join("ProductHarness/CURRENT_INTENT.md"),
        &content,
    )?;
    Ok(IntentBrief {
        id,
        title: input.title,
        risk,
        confirmed: input.confirmed,
        repo_path,
        vault_path,
        resumed,
    })
}

pub fn require_confirmed_intent(repo_root: impl AsRef<Path>, title: &str) -> Result<()> {
    let path = repo_root
        .as_ref()
        .join("docs/baron/harness/CURRENT_INTENT.md");
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "A confirmed intent brief is required before medium/high-risk intake. No current intent exists at {}.",
            path.display()
        )
    })?;
    let recorded_title = field(&content, "- Title: ").unwrap_or_default();
    if normalize_match(recorded_title) != normalize_match(title) {
        bail!(
            "The current confirmed intent does not match this intake. Intent: `{}`, intake: `{}`.",
            recorded_title,
            title.trim()
        );
    }
    if !content.contains("- Confirmation: `confirmed`") {
        bail!(
            "The current intent for `{}` is not confirmed.",
            title.trim()
        );
    }
    Ok(())
}

pub fn intent_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let path = repo_root
        .as_ref()
        .join("docs/baron/harness/CURRENT_INTENT.md");
    let body = fs::read_to_string(&path)
        .unwrap_or_else(|_| "- Status: no intent brief recorded\n".to_string());
    Ok(format!(
        "# Baron Intent Status\n\n- Current intent: `{}`\n\n{}",
        path.display(),
        body.trim()
    ))
}

fn render_intent(
    id: &str,
    input: &IntentBriefInput,
    risk: RiskLane,
    identity: Option<&LifecycleIdentity>,
) -> String {
    let identity_header = identity.map_or_else(String::new, |identity| {
        format!(
            "- Project ID: `{}`\n- Task ID: `{}`\n- Operation ID: `{}`\n- Adapter: `{}`\n- Session ID: `{}`\n- Request ID: `{}`\n",
            identity.project_id(),
            identity.task_id(),
            identity.operation_id(),
            identity.adapter().as_str(),
            identity.session_id(),
            identity.request_id()
        )
    });
    format!(
        "# Baron Intent Brief\n\n\
- ID: `{id}`\n\
- Title: {}\n\
- Risk: `{}`\n\
- Confirmation: `{}`\n\
- Updated: {}\n\
{}\n\
## Current Behavior\n\n{}\n\n\
## Target Behavior\n\n{}\n\n\
## Scope\n\n{}\n\n\
## Non-Goals\n\n{}\n\n\
## Constraints\n\n{}\n\n\
## Decisions\n\n{}\n\n\
## Required Proof\n\n{}\n\n\
## Remaining Unknowns\n\n{}\n\n\
## Agent Rules\n\n\
- Read project, Vault, plan, Harness, and prior decisions before asking the user.\n\
- Ask one missing high-value question at a time.\n\
- Do not treat unknowns as facts.\n\
- Medium/high-risk implementation requires this intent to be confirmed.\n",
        input.title,
        risk.as_str(),
        confirmation(input.confirmed),
        now(),
        identity_header,
        input.current_behavior,
        input.target_behavior,
        input.scope,
        markdown_list(&input.non_goals),
        markdown_list(&input.constraints),
        markdown_list(&input.decisions),
        input.required_proof,
        markdown_list(&input.unknowns)
    )
}

fn validate_input(input: &IntentBriefInput) -> Result<()> {
    for (name, value) in [
        ("title", input.title.as_str()),
        ("current behavior", input.current_behavior.as_str()),
        ("target behavior", input.target_behavior.as_str()),
        ("scope", input.scope.as_str()),
        ("required proof", input.required_proof.as_str()),
    ] {
        if value.is_empty() {
            bail!("Intent {name} must not be empty.");
        }
    }
    Ok(())
}

fn normalize_input(input: &mut IntentBriefInput) {
    input.title = single_line(&input.title);
    input.current_behavior = single_line(&input.current_behavior);
    input.target_behavior = single_line(&input.target_behavior);
    input.scope = single_line(&input.scope);
    input.required_proof = single_line(&input.required_proof);
    for values in [
        &mut input.non_goals,
        &mut input.constraints,
        &mut input.decisions,
        &mut input.unknowns,
    ] {
        *values = values
            .iter()
            .map(|value| single_line(value))
            .filter(|value| !value.is_empty())
            .collect();
    }
}

fn intent_id(input: &IntentBriefInput) -> Result<String> {
    let bytes = serde_json::to_vec(input)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("intent-{}", hex_prefix(&digest, 8)))
}

fn intent_id_for_operation(
    input: &IntentBriefInput,
    identity: &LifecycleIdentity,
) -> Result<String> {
    let bytes = serde_json::to_vec(&(input, identity))?;
    let digest = Sha256::digest(bytes);
    Ok(format!("intent-{}", hex_prefix(&digest, 8)))
}

fn intent_header_matches_identity(content: &str, identity: &LifecycleIdentity) -> bool {
    let header = content.split("\n## ").next().unwrap_or_default();
    [
        ("- Project ID: ", identity.project_id()),
        ("- Task ID: ", identity.task_id()),
        ("- Operation ID: ", identity.operation_id()),
        ("- Adapter: ", identity.adapter().as_str()),
        ("- Session ID: ", identity.session_id()),
        ("- Request ID: ", identity.request_id()),
    ]
    .into_iter()
    .all(|(prefix, expected)| {
        let mut values = header.lines().filter_map(|line| line.strip_prefix(prefix));
        values.next().map(|value| value.trim().trim_matches('`')) == Some(expected)
            && values.next().is_none()
    })
}

fn hex_prefix(bytes: &[u8], count: usize) -> String {
    bytes
        .iter()
        .take(count)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn confirmation(confirmed: bool) -> &'static str {
    if confirmed {
        "confirmed"
    } else {
        "needs_confirmation"
    }
}

fn markdown_list(values: &[String]) -> String {
    if values.is_empty() {
        "- none recorded".to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn append_unique(path: &Path, header: &str, item: &str) -> Result<()> {
    let content = match read_text(path)? {
        Some(content) => content,
        None => {
            replace_text(path, header)?;
            header.to_string()
        }
    };
    if content.lines().any(|line| line == item) {
        return Ok(());
    }
    let separator = if content.is_empty() || content.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    append_text(path, &format!("{separator}{item}\n"))
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn field<'a>(content: &'a str, prefix: &str) -> Option<&'a str> {
    content.lines().find_map(|line| line.strip_prefix(prefix))
}

fn normalize_match(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalize_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_string()
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
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
        "intent".to_string()
    } else {
        slug
    }
}
