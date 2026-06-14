use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};
use serde::{Deserialize, Serialize};

use crate::harness::{current_harness_risk, current_harness_title};
use crate::proof::latest_proof;
use crate::risk::RiskLane;
use crate::vault::VaultContext;

const SCORE_START: &str = "<!-- BARON:TRACE-SCORE:START -->";
const SCORE_END: &str = "<!-- BARON:TRACE-SCORE:END -->";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceOutcome {
    Completed,
    Partial,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceTier {
    Incomplete,
    Minimal,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceRecord {
    pub id: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceScore {
    pub achieved: TraceTier,
    pub required: TraceTier,
    pub passed: bool,
    pub missing_fields: Vec<String>,
}

pub fn record_trace(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    summary: &str,
    outcome: TraceOutcome,
) -> Result<TraceRecord> {
    let repo_root = repo_root.as_ref();
    let now = Local::now();
    let id = now.format("%Y%m%d%H%M%S%3f").to_string();
    let date = now.format("%Y-%m-%d").to_string();
    let risk = if repo_root.join("docs/baron/harness/CURRENT.md").exists() {
        current_harness_risk(repo_root)
    } else {
        current_plan_risk(repo_root)
    };
    let story = current_harness_title(repo_root);
    let plan = current_plan_title(repo_root);
    let proof = latest_proof(repo_root)?;
    let files = changed_files(repo_root);
    let repo_path = repo_root
        .join("docs/baron/traces")
        .join(&date)
        .join(format!("{id}.md"));
    let vault_path = vault
        .project_root
        .join("Traces")
        .join(&date)
        .join(format!("{id}.md"));
    let content = render_trace(TraceView {
        id: &id,
        summary,
        outcome,
        risk,
        plan: plan.as_deref(),
        story: story.as_deref(),
        proof: proof.as_ref().map(|value| value.summary.as_str()),
        files: &files,
    });
    write(&repo_path, &content)?;
    write(&vault_path, &content)?;
    append(
        &repo_root.join("docs/baron/traces/INDEX.md"),
        "# Baron Trace Index\n\n",
        &format!("- `{id}` - {} - {}", outcome.as_str(), summary.trim()),
    )?;
    append(
        &vault.project_root.join("Traces/INDEX.md"),
        "# Baron Trace Index\n\n",
        &format!("- `{id}` - {} - {}", outcome.as_str(), summary.trim()),
    )?;
    Ok(TraceRecord {
        id,
        repo_path,
        vault_path,
    })
}

pub fn score_trace(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    trace_id: Option<&str>,
) -> Result<TraceScore> {
    let repo_root = repo_root.as_ref();
    let repo_path = find_trace(repo_root, trace_id)?;
    let content = fs::read_to_string(&repo_path)?;
    let risk = parse_risk(&content);
    let mut missing = Vec::new();
    if !content.contains("## Task Summary\n\n") || content.contains("## Task Summary\n\n\n") {
        missing.push("task summary".to_string());
    }
    if content.contains("- Outcome: `unknown`") {
        missing.push("outcome".to_string());
    }
    let has_plan = !content.contains("- Current plan: `missing`");
    let has_story = !content.contains("- Current story: `missing`");
    let has_proof = !content.contains("- Proof: `missing`");
    let has_files = content
        .split("## Files Changed")
        .nth(1)
        .map(|value| value.lines().any(|line| line.starts_with("- `")))
        .unwrap_or(false);

    let achieved = if missing.is_empty() && has_plan && has_story && has_proof && has_files {
        TraceTier::Detailed
    } else if missing.is_empty() && has_plan && has_proof {
        TraceTier::Standard
    } else if missing.is_empty() {
        TraceTier::Minimal
    } else {
        TraceTier::Incomplete
    };
    let required = required_tier(risk);
    if required >= TraceTier::Standard && !has_plan {
        missing.push("current plan".to_string());
    }
    if required >= TraceTier::Standard && !has_proof {
        missing.push("proof".to_string());
    }
    if required == TraceTier::Detailed && !has_story {
        missing.push("current story".to_string());
    }
    if required == TraceTier::Detailed && !has_files {
        missing.push("files changed".to_string());
    }
    let proof_valid = if risk == RiskLane::High {
        high_risk_proof_present(&content)
    } else {
        true
    };
    if !proof_valid {
        missing.push("security/data-impact proof".to_string());
    }
    missing.sort();
    missing.dedup();
    let passed = achieved >= required && missing.is_empty();
    let score = TraceScore {
        achieved,
        required,
        passed,
        missing_fields: missing,
    };
    let updated = replace_score(&content, &score);
    write(&repo_path, &updated)?;
    let relative = repo_path
        .strip_prefix(repo_root.join("docs/baron/traces"))
        .unwrap_or(&repo_path);
    let vault_path = vault.project_root.join("Traces").join(relative);
    write(&vault_path, &updated)?;
    let trace_id = trace_field(&content, "- Trace ID: `").unwrap_or_else(|| "unknown".to_string());
    let outcome = trace_field(&content, "- Outcome: `").unwrap_or_else(|| "unknown".to_string());
    let summary = trace_summary(&content);
    update_trace_index(
        &repo_root.join("docs/baron/traces/INDEX.md"),
        &trace_id,
        &outcome,
        &summary,
        &score,
    )?;
    update_trace_index(
        &vault.project_root.join("Traces/INDEX.md"),
        &trace_id,
        &outcome,
        &summary,
        &score,
    )?;
    Ok(score)
}

pub fn latest_trace_score(repo_root: impl AsRef<Path>) -> Result<Option<TraceScore>> {
    let path = match find_trace(repo_root.as_ref(), None) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    let content = fs::read_to_string(path)?;
    let Some(section) = content.split(SCORE_START).nth(1) else {
        return Ok(None);
    };
    let achieved = parse_tier_line(section, "- Achieved: `");
    let required = parse_tier_line(section, "- Required: `");
    let passed = section.contains("- Passed: `yes`");
    let missing = section
        .lines()
        .find_map(|line| line.strip_prefix("- Missing: "))
        .unwrap_or("none")
        .split(", ")
        .filter(|value| *value != "none")
        .map(str::to_string)
        .collect();
    Ok(Some(TraceScore {
        achieved,
        required,
        passed,
        missing_fields: missing,
    }))
}

fn render_trace(view: TraceView<'_>) -> String {
    let mut content = format!(
        "# Baron Execution Trace\n\n\
- Trace ID: `{}`\n\
- Recorded: {}\n\
- Risk: `{}`\n\
- Outcome: `{}`\n\
- Current plan: `{}`\n\
- Current story: `{}`\n\
- Proof: `{}`\n\
- Score status: `unscored`\n\n\
## Task Summary\n\n{}\n\n\
## Files Changed\n\n",
        view.id,
        Local::now().to_rfc3339_opts(SecondsFormat::Secs, false),
        view.risk.as_str(),
        view.outcome.as_str(),
        view.plan.unwrap_or("missing"),
        view.story.unwrap_or("missing"),
        view.proof.unwrap_or("missing"),
        view.summary.trim()
    );
    if view.files.is_empty() {
        content.push_str("- none detected\n");
    } else {
        for file in view.files {
            content.push_str(&format!("- `{file}`\n"));
        }
    }
    content
}

struct TraceView<'a> {
    id: &'a str,
    summary: &'a str,
    outcome: TraceOutcome,
    risk: RiskLane,
    plan: Option<&'a str>,
    story: Option<&'a str>,
    proof: Option<&'a str>,
    files: &'a [String],
}

fn find_trace(repo_root: &Path, trace_id: Option<&str>) -> Result<PathBuf> {
    let root = repo_root.join("docs/baron/traces");
    let mut files = Vec::new();
    collect_markdown(&root, &mut files)?;
    files.retain(|path| path.file_name().and_then(|value| value.to_str()) != Some("INDEX.md"));
    files.sort();
    if let Some(id) = trace_id {
        return files
            .into_iter()
            .find(|path| path.file_stem().and_then(|value| value.to_str()) == Some(id))
            .with_context(|| format!("Trace not found: {id}"));
    }
    files.pop().context("No Baron trace found")
}

fn collect_markdown(root: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_markdown(&path, files)?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            files.push(path);
        }
    }
    Ok(())
}

fn current_plan_title(repo_root: &Path) -> Option<String> {
    let content = fs::read_to_string(repo_root.join("docs/baron/plans/CURRENT.md")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("- Title: "))
        .map(str::to_string)
}

fn current_plan_risk(repo_root: &Path) -> RiskLane {
    let content =
        fs::read_to_string(repo_root.join("docs/baron/plans/CURRENT.md")).unwrap_or_default();
    if content.contains("- Risk: `high`") {
        RiskLane::High
    } else if content.contains("- Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

fn changed_files(repo_root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(repo_root)
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_risk(content: &str) -> RiskLane {
    if content.contains("- Risk: `high`") {
        RiskLane::High
    } else if content.contains("- Risk: `low`") {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

fn required_tier(risk: RiskLane) -> TraceTier {
    match risk {
        RiskLane::Low => TraceTier::Minimal,
        RiskLane::Medium => TraceTier::Standard,
        RiskLane::High => TraceTier::Detailed,
    }
}

fn high_risk_proof_present(content: &str) -> bool {
    let lower = content.to_lowercase();
    let verification = ["passed", "verified", "test"]
        .iter()
        .any(|term| lower.contains(term));
    let impact = [
        "security",
        "authorization",
        "permission",
        "tenant",
        "rls",
        "migration",
        "data impact",
        "payment",
        "upload",
    ]
    .iter()
    .any(|term| lower.contains(term));
    verification && impact
}

fn replace_score(content: &str, score: &TraceScore) -> String {
    let missing = if score.missing_fields.is_empty() {
        "none".to_string()
    } else {
        score.missing_fields.join(", ")
    };
    let block = format!(
        "{SCORE_START}\n## Trace Quality Score\n\n- Achieved: `{}`\n- Required: `{}`\n- Passed: `{}`\n- Missing: {missing}\n{SCORE_END}\n",
        score.achieved.as_str(),
        score.required.as_str(),
        if score.passed { "yes" } else { "no" }
    );
    match (content.find(SCORE_START), content.find(SCORE_END)) {
        (Some(start), Some(end)) if end >= start => {
            let end = end + SCORE_END.len();
            format!("{}{}{}", &content[..start], block, &content[end..])
        }
        _ => format!("{}\n\n{}", content.trim_end(), block),
    }
}

fn trace_field(content: &str, prefix: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.strip_suffix('`'))
        .map(str::to_string)
}

fn trace_summary(content: &str) -> String {
    content
        .split("## Task Summary")
        .nth(1)
        .and_then(|value| value.split("## Files Changed").next())
        .unwrap_or("unknown")
        .trim()
        .replace(['\r', '\n'], " ")
}

fn update_trace_index(
    path: &Path,
    id: &str,
    outcome: &str,
    summary: &str,
    score: &TraceScore,
) -> Result<()> {
    let row = format!(
        "- `{id}` - {outcome} - score: `{}/{}` - passed: `{}` - {summary}",
        score.achieved.as_str(),
        score.required.as_str(),
        if score.passed { "yes" } else { "no" }
    );
    let mut content =
        fs::read_to_string(path).unwrap_or_else(|_| "# Baron Trace Index\n\n".to_string());
    let prefix = format!("- `{id}` -");
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

fn parse_tier_line(content: &str, prefix: &str) -> TraceTier {
    let value = content
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or("incomplete");
    TraceTier::from_str(value)
}

fn append(path: &Path, header: &str, item: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_else(|_| header.to_string());
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(item);
    content.push('\n');
    write(path, &content)
}

fn write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content).with_context(|| format!("Could not write {}", path.display()))
}

impl TraceOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

impl TraceTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Minimal => "minimal",
            Self::Standard => "standard",
            Self::Detailed => "detailed",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "minimal" => Self::Minimal,
            "standard" => Self::Standard,
            "detailed" => Self::Detailed,
            _ => Self::Incomplete,
        }
    }
}
