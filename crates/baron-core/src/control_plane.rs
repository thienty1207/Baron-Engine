use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat};
use serde::Deserialize;

use crate::risk::RiskLane;
use crate::vault::VaultContext;

const CORE_AGENTS: [&str; 3] = ["code-reviewer", "security-auditor", "test-engineer"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneReport {
    pub passed: bool,
    pub workflow_owner: Option<String>,
    pub mandatory_agents: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingDecision {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteReport {
    pub selected_skills: Vec<RoutingDecision>,
    pub mandatory_agents: Vec<String>,
    pub skipped: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEvidence {
    pub agent: String,
    pub repo_path: PathBuf,
    pub vault_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateEvidenceStatus {
    pub passed: bool,
    pub missing_agents: Vec<String>,
}

#[derive(Debug, Clone)]
struct SkillContract {
    name: String,
    description: String,
    body: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentContract {
    name: String,
    description: Option<String>,
    developer_instructions: Option<String>,
}

pub fn validate_control_plane(repo_root: impl AsRef<Path>) -> Result<ControlPlaneReport> {
    let repo_root = repo_root.as_ref();
    let skills = load_skills(repo_root)?;
    let agents = load_agents(repo_root)?;
    let mut diagnostics = Vec::new();

    let mut workflow_owners = skills
        .iter()
        .filter(|skill| owns_workflow(skill))
        .map(|skill| skill.name.clone())
        .collect::<Vec<_>>();
    workflow_owners.sort();
    workflow_owners.dedup();
    if !workflow_owners.iter().any(|name| name == "superpowers") {
        diagnostics.push("missing Superpowers workflow ownership".to_string());
    }
    if workflow_owners.len() > 1 {
        diagnostics.push(format!(
            "duplicate workflow ownership: {}",
            workflow_owners.join(", ")
        ));
    }

    let agent_names = agents
        .iter()
        .map(|agent| agent.name.as_str())
        .collect::<Vec<_>>();
    for required in CORE_AGENTS {
        if !agent_names.contains(&required) {
            diagnostics.push(format!("missing mandatory quality gate: {required}"));
        }
    }
    for agent in &agents {
        let instructions = agent
            .developer_instructions
            .as_deref()
            .unwrap_or_default()
            .to_lowercase();
        if instructions.contains("invoke other subagents")
            && !instructions.contains("do not invoke other subagents")
        {
            diagnostics.push(format!(
                "recursive subagent orchestration is forbidden: {}",
                agent.name
            ));
        }
        if agent
            .description
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
        {
            diagnostics.push(format!(
                "agent has weak contract description: {}",
                agent.name
            ));
        }
    }

    Ok(ControlPlaneReport {
        passed: diagnostics.is_empty(),
        workflow_owner: workflow_owners.first().cloned(),
        mandatory_agents: CORE_AGENTS.iter().map(|value| value.to_string()).collect(),
        diagnostics,
    })
}

pub fn route_task(repo_root: impl AsRef<Path>, task: &str, risk: RiskLane) -> Result<RouteReport> {
    let report = validate_control_plane(repo_root)?;
    let task_lower = task.to_lowercase();
    let frontend = contains_any(
        &task_lower,
        &[
            "frontend",
            "ui",
            "layout",
            "responsive",
            "component",
            "homepage",
            "dashboard",
            "browser",
        ],
    );
    let security = risk == RiskLane::High
        || contains_any(
            &task_lower,
            &[
                "auth",
                "login",
                "permission",
                "tenant",
                "rls",
                "security",
                "secret",
                "jwt",
                "cors",
                "upload",
                "payment",
                "api",
            ],
        );

    let mut selected_skills = vec![RoutingDecision {
        name: "superpowers".to_string(),
        reason: "workflow core for planning, TDD, debugging, review, and verification".to_string(),
    }];
    let mut skipped = Vec::new();
    if frontend {
        selected_skills.push(RoutingDecision {
            name: "frontend-design".to_string(),
            reason: "task touches frontend/UI surface".to_string(),
        });
    } else {
        skipped
            .push("frontend-design skipped: task does not match frontend/UI trigger".to_string());
    }
    if security {
        selected_skills.push(RoutingDecision {
            name: "vibe-security-scan".to_string(),
            reason: "security-sensitive task requires defensive security scan guidance".to_string(),
        });
    } else {
        skipped.push(
            "vibe-security-scan skipped: task does not match security-sensitive trigger"
                .to_string(),
        );
    }

    let mandatory_agents = if security || risk == RiskLane::High {
        CORE_AGENTS.iter().map(|value| value.to_string()).collect()
    } else if risk == RiskLane::Medium || frontend {
        ["code-reviewer", "test-engineer"]
            .iter()
            .map(|value| value.to_string())
            .collect()
    } else {
        vec!["test-engineer".to_string()]
    };
    let explanation = if !report.passed {
        format!(
            "control plane has diagnostics; routing is advisory until fixed: {}",
            report.diagnostics.join("; ")
        )
    } else if security {
        "security-sensitive task: load security skill and require all three core quality gates"
            .to_string()
    } else if frontend {
        "frontend task: load frontend skill and require code review plus verification gates"
            .to_string()
    } else {
        "general task: keep Superpowers as workflow core and use proportional quality gates"
            .to_string()
    };

    Ok(RouteReport {
        selected_skills,
        mandatory_agents,
        skipped,
        explanation,
    })
}

pub fn record_gate_evidence(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
    agent: &str,
    summary: &str,
) -> Result<GateEvidence> {
    let repo_path = repo_root.as_ref().join("docs/baron/control-plane/GATES.md");
    let vault_path = vault.project_root.join("ControlPlane/GATES.md");
    let item = format!("- {} - `{}` - {}", now(), agent.trim(), summary.trim());
    append(&repo_path, "# Baron Quality Gate Evidence\n\n", &item)?;
    append(&vault_path, "# Baron Quality Gate Evidence\n\n", &item)?;
    Ok(GateEvidence {
        agent: agent.trim().to_string(),
        repo_path,
        vault_path,
    })
}

pub fn gate_evidence_status(
    repo_root: impl AsRef<Path>,
    required_agents: &[String],
) -> Result<GateEvidenceStatus> {
    let content = fs::read_to_string(repo_root.as_ref().join("docs/baron/control-plane/GATES.md"))
        .unwrap_or_default();
    let missing_agents = required_agents
        .iter()
        .filter(|agent| !content.contains(&format!("`{agent}`")))
        .cloned()
        .collect::<Vec<_>>();
    Ok(GateEvidenceStatus {
        passed: missing_agents.is_empty(),
        missing_agents,
    })
}

fn owns_workflow(skill: &SkillContract) -> bool {
    let name = skill.name.to_lowercase();
    let combined = format!(
        "{}\n{}",
        skill.description.to_lowercase(),
        skill.body.to_lowercase()
    );
    name == "superpowers"
        || combined.contains("is the workflow core")
        || combined.contains("project workflow core")
        || combined.contains("my workflow core")
        || combined.contains("replaces superpowers")
        || combined.contains("replace superpowers")
}

fn load_skills(repo_root: &Path) -> Result<Vec<SkillContract>> {
    let mut skills = Vec::new();
    for root in skill_roots(repo_root) {
        if !root.exists() {
            continue;
        }
        for entry in
            fs::read_dir(&root).with_context(|| format!("Could not read {}", root.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let path = entry.path().join("SKILL.md");
            if path.exists() {
                skills.push(parse_skill(&path)?);
            }
        }
    }
    Ok(skills)
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
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

fn now() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Secs, false)
}

fn load_agents(repo_root: &Path) -> Result<Vec<AgentContract>> {
    let mut agents = Vec::new();
    for root in agent_roots(repo_root) {
        if !root.exists() {
            continue;
        }
        for entry in
            fs::read_dir(&root).with_context(|| format!("Could not read {}", root.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Could not read {}", path.display()))?;
                agents.push(
                    toml::from_str(&content)
                        .with_context(|| format!("Could not parse {}", path.display()))?,
                );
            } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("Could not read {}", path.display()))?;
                if let Some(agent) = parse_markdown_agent(&content) {
                    agents.push(agent);
                }
            }
        }
    }
    Ok(agents)
}

fn skill_roots(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join(".codex/skills"),
        repo_root.join(".claude/skills"),
        repo_root.join(".baron/core/skills"),
    ]
}

fn agent_roots(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root.join(".codex/agents"),
        repo_root.join(".claude/agents"),
        repo_root.join(".baron/core/agents"),
    ]
}

fn parse_skill(path: &Path) -> Result<SkillContract> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?;
    let (frontmatter, body) = split_frontmatter(&content);
    let name = frontmatter_value(frontmatter, "name")
        .or_else(|| {
            path.parent()
                .and_then(|parent| parent.file_name())
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string());
    let description = frontmatter_value(frontmatter, "description").unwrap_or_default();
    Ok(SkillContract {
        name,
        description,
        body: body.to_string(),
    })
}

fn parse_markdown_agent(content: &str) -> Option<AgentContract> {
    let (frontmatter, body) = split_frontmatter(content);
    let name = frontmatter_value(frontmatter, "name")?;
    let description = frontmatter_value(frontmatter, "description");
    Some(AgentContract {
        name,
        description,
        developer_instructions: Some(body.to_string()),
    })
}

fn split_frontmatter(content: &str) -> (&str, &str) {
    let Some(rest) = content.strip_prefix("---\n") else {
        return ("", content);
    };
    let Some(end) = rest.find("\n---") else {
        return ("", content);
    };
    (&rest[..end], &rest[end + 4..])
}

fn frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    frontmatter
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}
