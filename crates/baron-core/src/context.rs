use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::firewall::compact_memory_brief;
use crate::memory::build_memory_index;
use crate::survey::{survey_repository, ProjectType, RepoSurvey};
use crate::vault::ensure_vault;

const MAX_CONTEXT_CHARS: usize = 20_000;
const MAX_EXECUTION_STATE_CHARS: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTarget {
    Codex,
    Claude,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RiskLane {
    Low,
    Medium,
    High,
    Unknown,
}

pub fn compile_context(
    repo_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    target: ContextTarget,
) -> Result<String> {
    compile_context_for_task(repo_path, vault_path, target, None)
}

pub fn compile_context_for_task(
    repo_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    target: ContextTarget,
    task: Option<&str>,
) -> Result<String> {
    let repo_path = repo_path.as_ref();
    let survey = survey_repository(repo_path)?;
    let vault = ensure_vault(vault_path, repo_path)?;
    build_memory_index(&vault)?;
    let memory_brief = compact_memory_brief(&vault)?;
    let risk = classify_risk(task, &survey);

    let mut output = String::new();
    output.push_str(&format!("# Baron Context Bundle - {}\n\n", target.title()));
    output.push_str(&format!("- Project: `{}`\n", vault.project_slug));
    output.push_str(&format!("- Adapter target: `{}`\n", target.slug()));
    output.push_str("- Source of truth: repo files plus Vault Markdown\n");
    output.push_str(
        "- Memory firewall: current project first; weak cross-project memory blocked\n\n",
    );

    output.push_str("## Adapter Guidance\n\n");
    output.push_str(&format!("- {}\n", target.guidance()));
    output.push_str("- Treat unknown facts as unknown instead of guessing.\n");
    output.push_str("- This bundle is context only; it does not install adapter files.\n\n");

    output.push_str("## Task Focus\n\n");
    match task.map(str::trim).filter(|value| !value.is_empty()) {
        Some(task) => output.push_str(&format!("- Task: `{}`\n", task)),
        None => output.push_str("- Task: `not specified`\n"),
    }
    output.push_str(&format!("- Risk lane: `{}`\n", risk.label()));
    output.push_str(&format!("- {}\n\n", risk.guidance()));

    output.push_str(&render_survey_context(&survey));
    output.push_str(&render_execution_state(repo_path));
    output.push_str(&render_execution_evidence(repo_path));

    output.push_str(&memory_brief.replacen(
        "# Memory Firewall Brief",
        "## Memory Firewall Brief",
        1,
    ));
    if !output.ends_with('\n') {
        output.push('\n');
    }

    output.push_str("\n## Skipped Context\n\n");
    output.push_str(
        "- full Vault scan; only bounded current-project and approved global memory was loaded\n",
    );
    output.push_str("- unrelated project memory unless explicitly requested through recall\n");
    output.push_str("- full documentation bodies and historical session folders\n");
    output.push_str("- adapter file generation; that belongs to Phase 4\n");
    output.push_str("- No target repo files were written.\n");

    Ok(truncate_context(output))
}

pub fn compile_context_why(
    repo_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    target: ContextTarget,
) -> Result<String> {
    let repo_path = repo_path.as_ref();
    let survey = survey_repository(repo_path)?;
    let vault = ensure_vault(vault_path, repo_path)?;
    let report = build_memory_index(&vault)?;

    let mut output = String::new();
    output.push_str("# Context Selection Why\n\n");
    output.push_str(&format!("- Target: `{}`\n", target.slug()));
    output.push_str(&format!("- Project: `{}`\n\n", vault.project_slug));
    output.push_str("## Loaded\n\n");
    output.push_str("- Loaded: repo survey because stack, entrypoints, commands, risks, and read-first files orient the agent.\n");
    output.push_str(&format!(
        "- Loaded: Memory Firewall Brief because {} current-project records are indexed.\n",
        report.current_project_records
    ));
    if execution_state_path(repo_path).is_some() {
        output.push_str("- Loaded: bounded execution state because the repo exposes a current plan/status file.\n");
    }
    if repo_path.join("docs/baron/harness/CURRENT.md").is_file() {
        output
            .push_str("- Loaded: bounded Product Harness state because a current story exists.\n");
    }
    output.push_str(&format!(
        "- Loaded: {} adapter guidance because output shape changes by agent tool.\n\n",
        target.slug()
    ));
    output.push_str("## Skipped\n\n");
    output
        .push_str("- Skipped: full Vault scan to prevent shared-vault noise and context growth.\n");
    output.push_str(
        "- Skipped: weak cross-project memory because the current project firewall is active.\n",
    );
    output.push_str(
        "- Skipped: full docs and session history because compact context must stay bounded.\n",
    );
    output.push_str(
        "- Skipped: adapter file generation because Phase 3 only compiles stdout context.\n",
    );
    output.push_str(&format!(
        "- Survey unknowns retained: {}.\n",
        survey.unknowns.len()
    ));
    output.push_str("\nNo target repo files were written.\n");
    Ok(output)
}

fn render_survey_context(survey: &RepoSurvey) -> String {
    let mut output = String::new();
    output.push_str("## Project Atlas\n\n");
    output.push_str(&format!("- Repo root: `{}`\n", survey.repo_root));
    output.push_str(&format!(
        "- Git: {}\n",
        if survey.git_present {
            "detected"
        } else {
            "not detected"
        }
    ));
    output.push_str(&format!(
        "- Project type: `{}`\n\n",
        project_type_label(survey.project_type)
    ));

    output.push_str("### Stack Hints\n\n");
    if survey.stack_hints.is_empty() {
        output.push_str("- none detected\n");
    } else {
        for item in survey.stack_hints.iter().take(8) {
            output.push_str(&format!("- {} (`{}`)\n", item.label, item.path));
        }
    }

    output.push_str("\n### Entrypoints\n\n");
    if survey.entrypoints.is_empty() {
        output.push_str("- none detected\n");
    } else {
        for item in survey.entrypoints.iter().take(8) {
            output.push_str(&format!("- {} (`{}`)\n", item.label, item.path));
        }
    }

    output.push_str("\n### Commands\n\n");
    if survey.commands.is_empty() {
        output.push_str("- none detected; do not guess commands\n");
    } else {
        for command in survey.commands.iter().take(8) {
            output.push_str(&format!(
                "- {}: `{}` (source: `{}`)\n",
                command.kind, command.command, command.source
            ));
        }
    }

    output.push_str("\n### Risky Surfaces\n\n");
    if survey.risky_surfaces.is_empty() {
        output.push_str("- none detected from paths\n");
    } else {
        for item in survey.risky_surfaces.iter().take(8) {
            output.push_str(&format!("- {} (`{}`)\n", item.label, item.path));
        }
    }

    output.push_str("\n### Read First\n\n");
    if survey.read_first.is_empty() {
        output.push_str("- none detected\n");
    } else {
        for path in survey.read_first.iter().take(8) {
            output.push_str(&format!("- `{}`\n", path));
        }
    }

    output.push_str("\n### Survey Unknowns\n\n");
    if survey.unknowns.is_empty() {
        output.push_str("- none\n\n");
    } else {
        for unknown in survey.unknowns.iter().take(8) {
            output.push_str(&format!("- {}\n", unknown));
        }
        output.push('\n');
    }
    output
}

fn render_execution_state(repo_path: &Path) -> String {
    let mut output = String::new();
    output.push_str("## Execution State\n\n");
    let Some(path) = execution_state_path(repo_path) else {
        output.push_str("- no current plan/status file detected\n\n");
        return output;
    };
    let relative = path.strip_prefix(repo_path).unwrap_or(&path);
    output.push_str(&format!("- Source: `{}`\n\n", normalize_path(relative)));
    match fs::read_to_string(&path) {
        Ok(content) => {
            let excerpt = truncate_chars(&content, MAX_EXECUTION_STATE_CHARS);
            output.push_str("```markdown\n");
            output.push_str(excerpt.trim());
            output.push_str("\n```\n\n");
        }
        Err(_) => output.push_str("- file could not be read; execution state is unknown\n\n"),
    }
    output
}

fn execution_state_path(repo_path: &Path) -> Option<std::path::PathBuf> {
    [
        "docs/baron/plans/CURRENT.md",
        "docs/superpowers/plans/CURRENT.md",
        "docs/BARON_STATUS.md",
        "notes/build-log/CURRENT.md",
    ]
    .iter()
    .map(|path| repo_path.join(path))
    .find(|path| path.is_file())
}

fn render_execution_evidence(repo_path: &Path) -> String {
    let mut output = String::new();
    output.push_str("## Product Harness State\n\n");
    output.push_str(&bounded_file(
        &repo_path.join("docs/baron/harness/CURRENT.md"),
        1_200,
        "- no current Product Harness story\n",
    ));
    output.push_str("\n## Proof And Trace State\n\n");
    output.push_str("### Proof\n\n");
    output.push_str(&bounded_file(
        &repo_path.join("docs/baron/proofs/INDEX.md"),
        800,
        "- no proof recorded\n",
    ));
    output.push_str("\n### Trace\n\n");
    output.push_str(&bounded_file(
        &repo_path.join("docs/baron/traces/INDEX.md"),
        800,
        "- no trace recorded\n",
    ));
    output.push('\n');
    output
}

fn bounded_file(path: &Path, limit: usize, missing: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => format!("{}\n", truncate_chars(content.trim(), limit)),
        Err(_) => missing.to_string(),
    }
}

fn classify_risk(task: Option<&str>, survey: &RepoSurvey) -> RiskLane {
    let task = task.unwrap_or("").to_lowercase();
    let high_terms = [
        "auth",
        "login",
        "password",
        "token",
        "permission",
        "tenant",
        "rls",
        "payment",
        "billing",
        "migration",
        "security",
        "secret",
        "upload",
        "data loss",
    ];
    if high_terms.iter().any(|term| task.contains(term))
        || survey.risky_surfaces.iter().any(|item| {
            let label = item.label.to_lowercase();
            high_terms.iter().any(|term| label.contains(term))
        })
    {
        return RiskLane::High;
    }
    if task.is_empty() {
        return RiskLane::Unknown;
    }
    if ["docs", "readme", "copy", "typo"]
        .iter()
        .any(|term| task.contains(term))
    {
        RiskLane::Low
    } else {
        RiskLane::Medium
    }
}

fn truncate_context(mut output: String) -> String {
    if output.chars().count() <= MAX_CONTEXT_CHARS {
        return output;
    }
    output = truncate_chars(&output, MAX_CONTEXT_CHARS.saturating_sub(120));
    output.push_str("\n\n[Context truncated by Baron to preserve the bounded context contract.]\n");
    output
}

fn truncate_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn project_type_label(project_type: ProjectType) -> &'static str {
    match project_type {
        ProjectType::Frontend => "frontend",
        ProjectType::Backend => "backend",
        ProjectType::Fullstack => "fullstack",
        ProjectType::Tool => "tool",
        ProjectType::Desktop => "desktop",
        ProjectType::Mobile => "mobile",
        ProjectType::Unknown => "unknown",
    }
}

impl ContextTarget {
    pub fn slug(self) -> &'static str {
        match self {
            ContextTarget::Codex => "codex",
            ContextTarget::Claude => "claude",
            ContextTarget::Generic => "agent",
        }
    }

    fn title(self) -> &'static str {
        match self {
            ContextTarget::Codex => "Codex",
            ContextTarget::Claude => "Claude",
            ContextTarget::Generic => "Generic Agent",
        }
    }

    fn guidance(self) -> &'static str {
        match self {
            ContextTarget::Codex => {
                "For Codex: treat AGENTS.md and routed .codex indexes as the workspace contract when present."
            }
            ContextTarget::Claude => {
                "For Claude: treat CLAUDE.md and routed Claude command/hook surfaces as the workspace contract when present."
            }
            ContextTarget::Generic => {
                "For generic agents: use portable Markdown/JSON context and do not assume tool-specific hooks."
            }
        }
    }
}

impl RiskLane {
    fn label(self) -> &'static str {
        match self {
            RiskLane::Low => "low",
            RiskLane::Medium => "medium",
            RiskLane::High => "high",
            RiskLane::Unknown => "unknown",
        }
    }

    fn guidance(self) -> &'static str {
        match self {
            RiskLane::Low => "Keep verification proportional and record the exact result.",
            RiskLane::Medium => {
                "Require focused tests and explicit verification before claiming completion."
            }
            RiskLane::High => {
                "Require focused tests, security evidence, and explicit verification before claiming completion."
            }
            RiskLane::Unknown => {
                "Task risk is unknown; inspect the request and risky surfaces before changing code."
            }
        }
    }
}
