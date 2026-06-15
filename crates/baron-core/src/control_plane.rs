use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

const CORE_AGENTS: [&str; 3] = ["code-reviewer", "security-auditor", "test-engineer"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneReport {
    pub passed: bool,
    pub workflow_owner: Option<String>,
    pub mandatory_agents: Vec<String>,
    pub diagnostics: Vec<String>,
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
