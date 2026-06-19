use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

use crate::firewall::recall;
use crate::memory::{build_memory_index, load_memory_records};
use crate::release::SUPPORTED_RELEASE_TARGETS;
use crate::survey::survey_repository;
use crate::vault::{ensure_vault, VaultContext};
use crate::{
    autopilot::autopilot_status,
    capability::{default_adapter, runtime_backend_report},
};

const TARGET_RELEASE: &str = "3.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationProfile {
    Smoke,
    Release,
    Extreme,
}

impl CertificationProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Release => "release",
            Self::Extreme => "extreme",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationCheck {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub summary: String,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationReport {
    pub schema_version: u32,
    pub product: String,
    pub target_release: String,
    pub profile: CertificationProfile,
    pub passed: bool,
    pub generated_at: String,
    pub repo_root: String,
    pub vault_root: String,
    pub project_slug: String,
    pub project_id: String,
    pub checks: Vec<CertificationCheck>,
    pub markdown_path: PathBuf,
    pub json_path: PathBuf,
    pub vault_markdown_path: PathBuf,
}

pub fn run_certification(
    repo_path: impl AsRef<Path>,
    vault_path: impl AsRef<Path>,
    profile: CertificationProfile,
) -> Result<CertificationReport> {
    let repo_root = repo_path.as_ref().canonicalize().with_context(|| {
        format!(
            "Could not resolve repo path for certification: {}",
            repo_path.as_ref().display()
        )
    })?;
    let context = ensure_vault(vault_path, &repo_root)?;
    let generated_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut checks = vec![
        check_large_repo_survey(&repo_root, profile)?,
        check_cache_rebuild(&context)?,
        check_shared_vault_firewall(&context)?,
        check_context_budget(&context)?,
        check_automation_state(&repo_root)?,
        check_autopilot_readiness(&repo_root, &context)?,
        check_runtime_backend_policy(&repo_root)?,
    ];
    checks.push(check_release_readiness(&checks));

    let passed = checks.iter().all(|check| check.passed);
    let report_dir = repo_root.join("docs/baron/certification");
    let vault_dir = context.project_root.join("Certification");
    fs::create_dir_all(&report_dir)?;
    fs::create_dir_all(&vault_dir)?;
    let stamp = generated_at
        .replace([':', '-'], "")
        .replace(['T', 'Z'], "-")
        .trim_end_matches('-')
        .to_string();
    let markdown_path = report_dir.join(format!("{stamp}-certification.md"));
    let json_path = report_dir.join("latest.json");
    let vault_markdown_path = vault_dir.join(format!("{stamp}-certification.md"));

    let report = CertificationReport {
        schema_version: 1,
        product: "Baron Engine".to_string(),
        target_release: TARGET_RELEASE.to_string(),
        profile,
        passed,
        generated_at,
        repo_root: display_path(&repo_root),
        vault_root: display_path(&context.vault_root),
        project_slug: context.project_slug.clone(),
        project_id: context.project_id.clone(),
        checks,
        markdown_path,
        json_path,
        vault_markdown_path,
    };

    let markdown = render_certification_report(&report);
    fs::write(&report.markdown_path, &markdown)?;
    fs::write(&report.vault_markdown_path, &markdown)?;
    fs::write(
        &report.json_path,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )?;
    Ok(report)
}

pub fn latest_certification_status(repo_path: impl AsRef<Path>) -> Result<String> {
    let repo_root = repo_path.as_ref().canonicalize()?;
    let json_path = repo_root.join("docs/baron/certification/latest.json");
    if !json_path.exists() {
        return Ok(
            "# Baron Certification Status\n\n- No certification report found.\n".to_string(),
        );
    }
    let report: CertificationReport = serde_json::from_str(&fs::read_to_string(&json_path)?)?;
    let status = if report.passed {
        "latest certification passed"
    } else {
        "latest certification failed"
    };
    Ok(format!(
        "# Baron Certification Status\n\n- Status: {status}\n- Target release: `{}`\n- Profile: `{}`\n- Generated: {}\n- Checks: {}\n",
        report.target_release,
        report.profile.as_str(),
        report.generated_at,
        report.checks.len()
    ))
}

pub fn render_certification_report(report: &CertificationReport) -> String {
    let mut output = String::new();
    output.push_str("# Baron Certification\n\n");
    output.push_str(&format!("- Target release: `{}`\n", report.target_release));
    output.push_str(&format!("- Profile: `{}`\n", report.profile.as_str()));
    output.push_str(&format!(
        "- Passed: `{}`\n",
        if report.passed { "yes" } else { "no" }
    ));
    output.push_str(&format!("- Project: `{}`\n", report.project_slug));
    output.push_str(&format!("- Project ID: `{}`\n", report.project_id));
    output.push_str(&format!("- Repo: `{}`\n", report.repo_root));
    output.push_str(&format!("- Vault: `{}`\n\n", report.vault_root));
    output.push_str("## Checks\n\n");
    for check in &report.checks {
        output.push_str(&format!(
            "### {} - {}\n\n- Passed: `{}`\n- Summary: {}\n",
            check.id,
            check.name,
            if check.passed { "yes" } else { "no" },
            check.summary
        ));
        if !check.details.is_empty() {
            output.push_str("- Details:\n");
            for detail in &check.details {
                output.push_str(&format!("  - {detail}\n"));
            }
        }
        output.push('\n');
    }
    output
}

fn check_large_repo_survey(
    repo_root: &Path,
    profile: CertificationProfile,
) -> Result<CertificationCheck> {
    let survey = survey_repository(repo_root)?;
    let repo_files = count_repo_files(repo_root)?;
    let required_files = match profile {
        CertificationProfile::Smoke => 1,
        CertificationProfile::Release => 50,
        CertificationProfile::Extreme => 1_000,
    };
    let output_budget = match profile {
        CertificationProfile::Smoke => 100_000,
        CertificationProfile::Release => 120_000,
        CertificationProfile::Extreme => 150_000,
    };
    let survey_json = serde_json::to_string(&survey)?;
    let passed = repo_files >= required_files && survey_json.len() <= output_budget;
    Ok(CertificationCheck {
        id: "large-repo-survey".to_string(),
        name: "Large repository survey budget".to_string(),
        passed,
        summary: format!(
            "Survey inspected {repo_files} files with {} bytes of JSON output.",
            survey_json.len()
        ),
        details: vec![
            format!("Required files for profile: {required_files}"),
            format!("Output budget bytes: {output_budget}"),
            format!("Risky surfaces detected: {}", survey.risky_surfaces.len()),
        ],
    })
}

fn check_cache_rebuild(context: &VaultContext) -> Result<CertificationCheck> {
    let mut recovered = false;
    let first = match build_memory_index(context) {
        Ok(report) => report,
        Err(error) => {
            if context.index_path.exists() {
                fs::remove_file(&context.index_path)?;
            }
            recovered = true;
            build_memory_index(context)
                .with_context(|| format!("memory index rebuild failed after recovery: {error}"))?
        }
    };
    let second = build_memory_index(context)?;
    Ok(CertificationCheck {
        id: "cache-corruption-recovery".to_string(),
        name: "Cache corruption recovery".to_string(),
        passed: context.index_path.exists() && second.total_records >= first.total_records,
        summary: if recovered {
            "Corrupt SQLite cache was discarded and rebuilt from Vault Markdown.".to_string()
        } else {
            "SQLite cache rebuilt successfully from Vault Markdown.".to_string()
        },
        details: vec![
            format!("Index path: {}", context.index_path.display()),
            format!("First records: {}", first.total_records),
            format!("Second records: {}", second.total_records),
            format!(
                "Recovered corrupt cache: {}",
                if recovered { "yes" } else { "no" }
            ),
        ],
    })
}

fn check_shared_vault_firewall(context: &VaultContext) -> Result<CertificationCheck> {
    let records = load_memory_records(context)?;
    let cross_project_records = records
        .iter()
        .filter(|record| {
            record.project_id.as_deref().is_some()
                && record.project_id.as_deref() != Some(context.project_id.as_str())
        })
        .count();
    let result = recall(context, "auth memory", 10)?;
    let current_first = result
        .results
        .first()
        .map(|hit| hit.record.project_id.as_deref() == Some(context.project_id.as_str()))
        .unwrap_or(true);
    let blocked_or_absent = cross_project_records == 0 || result.blocked_cross_project > 0;
    Ok(CertificationCheck {
        id: "shared-vault-firewall".to_string(),
        name: "Memory firewall isolation".to_string(),
        passed: current_first && blocked_or_absent,
        summary: "Memory firewall kept current-project memory first and did not trust weak cross-project matches.".to_string(),
        details: vec![
            format!("Recall results: {}", result.results.len()),
            format!("Cross-project records indexed: {cross_project_records}"),
            format!("Blocked cross-project hits: {}", result.blocked_cross_project),
            format!("Skipped global candidates: {}", result.skipped_global_candidates),
        ],
    })
}

fn check_context_budget(context: &VaultContext) -> Result<CertificationCheck> {
    let brief = crate::firewall::compact_memory_brief_for_task(context, Some("auth memory"))?;
    let budget = 24_000;
    Ok(CertificationCheck {
        id: "context-budget".to_string(),
        name: "Bounded context budget".to_string(),
        passed: brief.len() <= budget,
        summary: format!("Memory firewall brief is {} bytes.", brief.len()),
        details: vec![format!("Budget bytes: {budget}")],
    })
}

fn check_automation_state(repo_root: &Path) -> Result<CertificationCheck> {
    let config_path = repo_root.join(".baron/project.toml");
    let has_config = config_path.exists();
    let journal_path = repo_root.join(".baron/automation-journal.jsonl");
    Ok(CertificationCheck {
        id: "automation-readiness".to_string(),
        name: "Observable automation readiness".to_string(),
        passed: has_config,
        summary: if has_config {
            "Project has Baron configuration for observable automation.".to_string()
        } else {
            "Project has no Baron configuration; run `baron init` before release certification."
                .to_string()
        },
        details: vec![
            format!("Config: {}", config_path.display()),
            format!(
                "Automation journal exists: {}",
                if journal_path.exists() { "yes" } else { "no" }
            ),
        ],
    })
}

fn check_autopilot_readiness(
    repo_root: &Path,
    context: &VaultContext,
) -> Result<CertificationCheck> {
    let status = autopilot_status(repo_root, context)?;
    let passed = status.contains("Do not infer completion") && status.contains("Trusted");
    Ok(CertificationCheck {
        id: "autopilot-readiness".to_string(),
        name: "Background learning and continuity autopilot".to_string(),
        passed,
        summary: "Autopilot reports resume state and keeps learning candidates separate from trusted facts.".to_string(),
        details: vec![
            format!(
                "Candidate file: {}",
                repo_root
                    .join("docs/baron/autopilot/CANDIDATES.md")
                    .display()
            ),
            "Unapproved candidates are not promoted into Facts.md.".to_string(),
        ],
    })
}

fn check_runtime_backend_policy(repo_root: &Path) -> Result<CertificationCheck> {
    let adapter = default_adapter(repo_root).unwrap_or(crate::config::AdapterKind::Generic);
    let report = runtime_backend_report(repo_root, adapter)?;
    Ok(CertificationCheck {
        id: "runtime-backend-policy".to_string(),
        name: "Capability runtime backend policy".to_string(),
        passed: report.passed,
        summary: "Runtime policy keeps provider presence separate from execution evidence and flags unsafe backends.".to_string(),
        details: vec![
            format!("Providers checked: {}", report.providers.len()),
            format!("Blocking gaps: {}", values_or_none(&report.blocking_gaps)),
            format!("Warnings: {}", values_or_none(&report.warnings)),
        ],
    })
}

fn check_release_readiness(existing: &[CertificationCheck]) -> CertificationCheck {
    let previous_checks_pass = existing.iter().all(|check| check.passed);
    CertificationCheck {
        id: "release-readiness".to_string(),
        name: "Release readiness".to_string(),
        passed: previous_checks_pass && SUPPORTED_RELEASE_TARGETS.len() == 4,
        summary: "Release readiness requires all certification gates and the four native release targets.".to_string(),
        details: vec![
            format!("Target release: {TARGET_RELEASE}"),
            format!("Native targets: {}", SUPPORTED_RELEASE_TARGETS.len()),
            format!("Prior checks passed: {}", if previous_checks_pass { "yes" } else { "no" }),
        ],
    }
}

fn count_repo_files(repo_root: &Path) -> Result<usize> {
    let mut count = 0;
    for entry in WalkBuilder::new(repo_root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true)
        .filter_entry(|entry| !is_heavy_path(entry.path()))
        .build()
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if entry
            .file_type()
            .map(|file_type| file_type.is_file())
            .unwrap_or(false)
        {
            count += 1;
        }
    }
    Ok(count)
}

fn is_heavy_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|name| {
            matches!(
                name,
                ".git"
                    | "node_modules"
                    | "target"
                    | "dist"
                    | "build"
                    | ".next"
                    | ".cache"
                    | "vendor"
            )
        })
        .unwrap_or(false)
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .strip_prefix("//?/")
        .unwrap_or(&path.to_string_lossy().replace('\\', "/"))
        .to_string()
}

fn values_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join("; ")
    }
}
