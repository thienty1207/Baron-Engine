use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{Local, SecondsFormat};

use crate::domain_language::{ensure_domain_language, DomainLanguageStatus};
use crate::intent::require_confirmed_intent;
use crate::operation::OperationContext;
use crate::plan::{active_plan_authority_for_binding, PlanOperationBinding};
use crate::risk::{classify_risk, RiskLane};
use crate::safe_io::{acquire_project_lock, append_text, read_text, replace_text};
use crate::vault::{canonical_project_id, VaultContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessStory {
    pub title: String,
    pub risk: RiskLane,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
    pub resumed: bool,
}

pub fn ensure_harness_workspace(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> Result<DomainLanguageStatus> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_domain_language(repo_root, vault)
}

pub fn start_or_resume_intake(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    title: &str,
) -> Result<HarnessStory> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    // Shared Vault writes always follow checkout -> capsule lock ordering.
    // Separate worktrees can share one capsule, so the checkout lock alone
    // cannot serialize updates to its read-modify-write documents.
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    ensure_harness_workspace(repo_root, vault)?;
    let title = title.trim();
    let risk = classify_risk(title);
    if risk != RiskLane::Low {
        require_confirmed_intent(repo_root, title)?;
    }
    let date = today();
    let slug = slugify(title);
    let repo_path = repo_root
        .join("docs/baron/harness/stories")
        .join(&date)
        .join(format!("{date}-{slug}.md"));
    let vault_path = vault
        .project_root
        .join("ProductHarness/Stories")
        .join(&date)
        .join(format!("{date}-{slug}.md"));
    let resumed = repo_path.exists();
    if !resumed {
        let content = story_content(title, risk);
        write(&repo_path, &content)?;
        write(&vault_path, &content)?;
        append_unique(
            &repo_root.join("docs/baron/harness/STORIES.md"),
            "# Product Harness Stories\n\n",
            &format!(
                "- [{}]({}) - risk: `{}`",
                title,
                normalize(&repo_path, repo_root),
                risk.as_str()
            ),
        )?;
        append_unique(
            &vault.project_root.join("ProductHarness/STORIES.md"),
            "# Product Harness Stories\n\n",
            &format!(
                "- [{}]({}) - risk: `{}`",
                title,
                normalize(&vault_path, &vault.project_root),
                risk.as_str()
            ),
        )?;
    }
    let current = format!(
        "# Current Product Harness\n\n- Title: {title}\n- Risk: `{}`\n- Story: `{}`\n- Updated: {}\n",
        risk.as_str(),
        normalize(&repo_path, repo_root),
        now()
    );
    write(&repo_root.join("docs/baron/harness/CURRENT.md"), &current)?;
    write(
        &vault.project_root.join("ProductHarness/CURRENT.md"),
        &current,
    )?;
    upsert_validation_row(
        &repo_root.join("docs/baron/harness/TEST_MATRIX.md"),
        title,
        risk,
        "pending",
        "pending",
    )?;
    upsert_validation_row(
        &vault.project_root.join("ProductHarness/TEST_MATRIX.md"),
        title,
        risk,
        "pending",
        "pending",
    )?;
    Ok(HarnessStory {
        title: title.to_string(),
        risk,
        repo_path,
        vault_path,
        resumed,
    })
}

pub fn record_decision(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    append(
        &repo_root.join("docs/baron/harness/DECISIONS.md"),
        "# Product Decisions\n\n",
        &format!("- {} - {}", now(), summary.trim()),
    )?;
    append(
        &vault.project_root.join("ProductHarness/DECISIONS.md"),
        "# Product Decisions\n\n",
        &format!("- {} - {}", now(), summary.trim()),
    )
}

pub fn record_friction(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    let item = format!("- [ ] {} - {}", now(), summary.trim());
    append(
        &repo_root.join("docs/baron/harness/FRICTION.md"),
        "# Harness Friction\n\n",
        &item,
    )?;
    append(
        &vault.project_root.join("ProductHarness/FRICTION.md"),
        "# Harness Friction\n\n",
        &item,
    )
}

pub fn harness_status(repo_root: impl AsRef<Path>) -> Result<String> {
    let root = repo_root.as_ref().join("docs/baron/harness");
    let current_path = root.join("CURRENT.md");
    let current = if current_path.exists() {
        fs::read_to_string(&current_path)?
    } else {
        "# Current Product Harness\n\n- none\n".to_string()
    };
    let friction = fs::read_to_string(root.join("FRICTION.md")).unwrap_or_default();
    let open = friction
        .lines()
        .filter(|line| line.starts_with("- [ ]"))
        .count();
    Ok(format!(
        "# Baron Harness Status\n\n{}\n- Open friction: {}\n",
        current.trim(),
        open
    ))
}

pub fn current_harness_risk(repo_root: impl AsRef<Path>) -> RiskLane {
    let content = fs::read_to_string(repo_root.as_ref().join("docs/baron/harness/CURRENT.md"))
        .unwrap_or_default();
    if content.contains("Risk: `high`") {
        RiskLane::High
    } else if content.contains("Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

pub fn current_harness_title(repo_root: impl AsRef<Path>) -> Option<String> {
    let content =
        fs::read_to_string(repo_root.as_ref().join("docs/baron/harness/CURRENT.md")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("- Title: "))
        .map(str::to_string)
}

/// Associate the one unique task story with the exact identified active plan.
/// Harness CURRENT is a latest-view projection and never selects or vetoes an
/// identified operation's story. Same-task concurrent plans and duplicate
/// historical story files remain ambiguous and confer no story authority.
pub fn current_harness_title_for_operation(
    repo_root: impl AsRef<Path>,
    operation: &OperationContext,
) -> Result<Option<String>> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    if operation.task_id.is_none()
        || operation.operation_id.is_none()
        || operation.session_id.is_none()
        || operation.request_id.is_none()
    {
        return Ok(None);
    }
    let project_id = canonical_project_id(repo_root)?;
    let identity = operation.lifecycle_identity(&project_id)?;
    let binding = PlanOperationBinding::from_identity(&identity);
    let Some(authority) = active_plan_authority_for_binding(repo_root, &binding)? else {
        return Ok(None);
    };
    if identity.validate_task(&authority.title).is_err() {
        return Ok(None);
    }
    let Some(index) = read_text(repo_root.join("docs/baron/plans/ACTIVE.md"))? else {
        return Ok(None);
    };
    // Exact plan resolution above validates every ACTIVE entry and its linked
    // frontmatter. This read only rejects ambiguous task ownership; it cannot
    // grant authority to an unindexed plan discovered by a legacy fallback.
    let mut task_entries = Vec::new();
    for line in index.lines() {
        let Some(json) = line
            .strip_prefix("<!-- BARON:ACTIVE-PLAN ")
            .and_then(|value| value.strip_suffix(" -->"))
        else {
            continue;
        };
        let entry: serde_json::Value = serde_json::from_str(json)?;
        if entry["task_id"].as_str() == Some(identity.task_id())
            && entry["status"].as_str() != Some("completed")
        {
            task_entries.push(entry);
        }
    }
    let plan_count =
        active_task_plan_count(&repo_root.join("docs/baron/plans"), identity.task_id())?;
    if task_entries.len() != 1
        || task_entries[0]["operation_id"].as_str() != Some(identity.operation_id())
        || plan_count != 1
    {
        return Ok(None);
    }
    find_unique_operation_story(repo_root, &authority.title, authority.risk)
}

fn find_unique_operation_story(
    repo_root: &Path,
    title: &str,
    plan_risk: RiskLane,
) -> Result<Option<String>> {
    let root = repo_root.join("docs/baron/harness/stories");
    let mut matches = Vec::new();
    collect_operation_story_matches(&root, title, &mut matches)?;
    if matches.is_empty() {
        return Ok(None);
    }
    if matches.len() != 1 {
        bail!("operation harness story is ambiguous for canonical task `{title}`");
    }
    let story = read_text(&matches[0])?.context("managed operation story disappeared")?;
    let risk = classify_risk(title);
    if unique_field(&story, "- Risk: `")?.and_then(|value| value.strip_suffix('`'))
        != Some(risk.as_str())
        || plan_risk != risk
    {
        bail!("managed harness story risk does not match canonical task risk");
    }
    Ok(Some(title.to_string()))
}

fn collect_operation_story_matches(
    root: &Path,
    title: &str,
    matches: &mut Vec<PathBuf>,
) -> Result<()> {
    let expected_header = format!("# Product Story - {title}");
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("managed harness story root contains a linked or non-directory path");
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("managed harness story tree contains a linked path");
        }
        if file_type.is_dir() {
            collect_operation_story_matches(&path, title, matches)?;
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("md")
        {
            let Some(content) = read_text(&path)? else {
                continue;
            };
            if content.lines().next() == Some(expected_header.as_str()) {
                matches.push(path);
            }
        }
    }
    Ok(())
}

fn unique_field<'a>(content: &'a str, prefix: &str) -> Result<Option<&'a str>> {
    let mut fields = content.lines().filter_map(|line| line.strip_prefix(prefix));
    let value = fields.next();
    if fields.next().is_some() {
        bail!("harness ownership field `{prefix}` is duplicated");
    }
    Ok(value)
}

fn active_task_plan_count(root: &Path, task_id: &str) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!("harness ownership cannot traverse a linked managed plan");
        }
        if file_type.is_dir() {
            count += active_task_plan_count(&path, task_id)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md")
            && !matches!(
                path.file_name().and_then(|value| value.to_str()),
                Some("CURRENT.md" | "ACTIVE.md" | "INDEX.md")
            )
        {
            let Some(content) = read_text(path)? else {
                continue;
            };
            let normalized = content.replace("\r\n", "\n");
            let Some(frontmatter) = normalized
                .strip_prefix("---\n")
                .and_then(|value| value.split_once("\n---").map(|(fields, _)| fields))
            else {
                continue;
            };
            if frontmatter_field(frontmatter, "type")? == Some("baron-plan")
                && frontmatter_field(frontmatter, "task_id")? == Some(task_id)
                && frontmatter_field(frontmatter, "status")? != Some("completed")
            {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn frontmatter_field<'a>(frontmatter: &'a str, key: &str) -> Result<Option<&'a str>> {
    let mut values = frontmatter.lines().filter_map(|line| {
        let (candidate, value) = line.split_once(':')?;
        (candidate.trim() == key).then_some(value.trim())
    });
    let value = values.next();
    if values.next().is_some() {
        bail!("managed plan frontmatter field `{key}` is duplicated");
    }
    Ok(value)
}

/// Update validation only after re-establishing exact, unique story ownership
/// under the same project lock used by proof publication.
pub fn update_current_validation_evidence_for_operation(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    operation: &OperationContext,
    evidence: &str,
    verified: bool,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let Some(title) = current_harness_title_for_operation(repo_root, operation)? else {
        return Ok(());
    };
    if canonical_project_id(repo_root)? != vault.project_id {
        bail!("harness operation project does not match Vault project");
    }
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    for path in [
        repo_root.join("docs/baron/harness/TEST_MATRIX.md"),
        vault.project_root.join("ProductHarness/TEST_MATRIX.md"),
    ] {
        upsert_validation_row(
            &path,
            &title,
            classify_risk(&title),
            if verified { "verified" } else { "insufficient" },
            evidence,
        )?;
    }
    Ok(())
}

pub fn update_current_validation_evidence(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    evidence: &str,
    verified: bool,
) -> Result<()> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    let Some(title) = current_harness_title(repo_root) else {
        return Ok(());
    };
    let risk = current_harness_risk(repo_root);
    let _vault_lock = acquire_project_lock(&vault.project_root)?;
    upsert_validation_row(
        &repo_root.join("docs/baron/harness/TEST_MATRIX.md"),
        &title,
        risk,
        if verified { "verified" } else { "insufficient" },
        evidence,
    )?;
    upsert_validation_row(
        &vault.project_root.join("ProductHarness/TEST_MATRIX.md"),
        &title,
        risk,
        if verified { "verified" } else { "insufficient" },
        evidence,
    )
}

fn story_content(title: &str, risk: RiskLane) -> String {
    let proof = match risk {
        RiskLane::Low => "concrete verification result",
        RiskLane::Medium => "focused test/build/smoke proof",
        RiskLane::High => "focused verification plus security/data-impact proof",
    };
    format!(
        "# Product Story - {title}\n\n\
- Status: `in_progress`\n\
- Risk: `{}`\n\
- Created: {}\n\n\
## Goal\n\n{title}\n\n\
## Scope\n\n- Work tied to this story only.\n\n\
## Out Of Scope\n\n- Unrelated cleanup or speculative features.\n\n\
## Required Proof\n\n- [ ] {proof}\n\
- [ ] Trace tier `{}` or stronger\n\n\
## Progress\n\n- {} - Intake created.\n",
        risk.as_str(),
        now(),
        risk.required_trace_tier(),
        now()
    )
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let content = match read_text(path)? {
        Some(content) => content,
        None => {
            replace_text(path, header)?;
            header.to_string()
        }
    };
    let separator = if content.is_empty() || content.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    append_text(path, &format!("{separator}{item}\n"))
}

fn append_unique(path: &Path, header: &str, item: &str) -> Result<()> {
    let content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if content.contains(item) {
        return Ok(());
    }
    append(path, header, item)
}

fn upsert_validation_row(
    path: &Path,
    title: &str,
    risk: RiskLane,
    status: &str,
    evidence: &str,
) -> Result<()> {
    const HEADER: &str = "# Baron Validation Matrix\n\n\
| Story | Risk | Status | Evidence |\n\
| --- | --- | --- | --- |\n";
    let title = table_cell(title);
    let row = format!(
        "| {title} | {} | {} | {} |",
        risk.as_str(),
        table_cell(status),
        table_cell(evidence)
    );
    let mut content = read_text(path)?.unwrap_or_else(|| HEADER.to_string());
    let prefix = format!("| {title} |");
    let mut replaced = false;
    let mut lines = content
        .lines()
        .map(|line| {
            if line.starts_with(&prefix) {
                replaced = true;
                row.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if !replaced {
        lines.push(row);
    }
    content = lines.join("\n");
    content.push('\n');
    write(path, &content)
}

fn table_cell(value: &str) -> String {
    value
        .replace('|', "\\|")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_string()
}

fn write(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content).with_context(|| format!("Could not write {}", path.display()))
}

fn normalize(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
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
        "story".to_string()
    } else {
        slug
    }
}
