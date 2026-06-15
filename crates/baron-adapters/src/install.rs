use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use crate::managed::{upsert_managed_block, write_managed_file};
use crate::AgentAdapter;

static CORE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../assets/core");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReport {
    pub adapter: String,
    pub managed_files: Vec<String>,
    pub preserved_custom_assets: bool,
}

pub fn install_adapter(
    repo_root: impl AsRef<Path>,
    adapter: AgentAdapter,
) -> Result<InstallReport> {
    let repo_root = repo_root.as_ref();
    match adapter {
        AgentAdapter::Codex => install_codex(repo_root),
        AgentAdapter::Claude => install_claude(repo_root),
        AgentAdapter::Generic => install_generic(repo_root),
    }
}

fn install_codex(repo: &Path) -> Result<InstallReport> {
    upsert_managed_block(&repo.join("AGENTS.md"), &startup_contract("Codex"))?;
    write_managed_file(&repo.join(".codex/INDEX.md"), &codex_index())?;
    write_managed_file(
        &repo.join(".codex/skills/INDEX.md"),
        &skills_index(".codex/skills"),
    )?;
    write_managed_file(&repo.join(".codex/agents/INDEX.md"), &agents_index())?;
    write_asset_subtree("skills", &repo.join(".codex/skills"))?;
    write_asset_subtree("agents", &repo.join(".codex/agents"))?;
    Ok(report(
        "codex",
        &[
            "AGENTS.md",
            ".codex/INDEX.md",
            ".codex/skills/INDEX.md",
            ".codex/agents/INDEX.md",
        ],
    ))
}

fn install_claude(repo: &Path) -> Result<InstallReport> {
    upsert_managed_block(&repo.join("CLAUDE.md"), &startup_contract("Claude"))?;
    write_managed_file(
        &repo.join(".claude/commands/baron-context.md"),
        "# Baron Context\n\nRun `baron capability check` and then `baron context --claude` silently. Follow the bounded context bundle. Capability presence is not execution evidence.\n",
    )?;
    write_managed_file(
        &repo.join(".claude/commands/baron-status.md"),
        "# Baron Status\n\nRun `baron plan status`, `baron harness status`, `baron proof status`, and inspect the latest trace score.\n",
    )?;
    write_managed_file(
        &repo.join(".claude/skills/INDEX.md"),
        &skills_index(".claude/skills"),
    )?;
    write_asset_subtree("skills", &repo.join(".claude/skills"))?;
    write_claude_agents(repo)?;
    Ok(report(
        "claude",
        &[
            "CLAUDE.md",
            ".claude/commands/baron-context.md",
            ".claude/commands/baron-status.md",
            ".claude/skills/INDEX.md",
        ],
    ))
}

fn install_generic(repo: &Path) -> Result<InstallReport> {
    upsert_managed_block(&repo.join("AGENT.md"), &startup_contract("generic agents"))?;
    write_managed_file(
        &repo.join("baron-context.md"),
        "# Baron Context Contract\n\nRun `baron capability check` and then `baron context --agent` at session start. Treat output as bounded orientation, not as a replacement for repository evidence. Capability presence is not execution evidence.\n",
    )?;
    write_managed_file(
        &repo.join("baron-context.json"),
        &serde_json::to_string_pretty(&serde_json::json!({
            "engine": "baron",
            "adapter": "agent",
            "capabilityCheckCommand": "baron capability check",
            "contextCommand": "baron context --agent",
            "automatic": true,
            "sourceOfTruth": ["repository", "vault-markdown"]
        }))?,
    )?;
    write_managed_file(
        &repo.join(".baron/core/skills/INDEX.md"),
        &skills_index(".baron/core/skills"),
    )?;
    write_managed_file(&repo.join(".baron/core/agents/INDEX.md"), &agents_index())?;
    write_asset_subtree("skills", &repo.join(".baron/core/skills"))?;
    write_asset_subtree("agents", &repo.join(".baron/core/agents"))?;
    Ok(report(
        "agent",
        &[
            "AGENT.md",
            "baron-context.md",
            "baron-context.json",
            ".baron/core/skills/INDEX.md",
            ".baron/core/agents/INDEX.md",
        ],
    ))
}

fn startup_contract(agent: &str) -> String {
    format!(
        "# Baron Automatic Agent Contract\n\n\
You are using Baron through the {agent} adapter.\n\n\
- At session start, silently run `baron capability check`; capability presence is not execution evidence.\n\
- At session start, silently run `baron context` and follow its bounded context.\n\
- Silently inspect `baron plan status` and `baron harness status` before meaningful implementation.\n\
- Start or resume a Baron plan before editing code for a meaningful task.\n\
- Create harness intake for medium/high-risk work.\n\
- Use Superpowers as the workflow core for planning, TDD, debugging, review, and verification.\n\
- Read the routed skill and agent indexes; do not recursively load every skill or agent.\n\
- After actually running a registered provider, attach structured capability evidence with `baron proof record`; then record and run `baron trace score` before claiming completion.\n\
- Never complete high-risk work when proof is missing or trace quality fails.\n\
- Treat Vault Markdown as durable memory and unknown facts as unknown.\n"
    )
}

fn codex_index() -> String {
    "# Baron Codex Workspace\n\n\
Start with root `AGENTS.md`. Read `.codex/skills/INDEX.md` and `.codex/agents/INDEX.md` for narrow routing. Superpowers is the workflow core; domain skills and quality agents are routed only when relevant.\n"
        .to_string()
}

fn skills_index(root: &str) -> String {
    format!(
        "# Baron Skill Routing\n\n\
Do not recursively load every skill. Match the task, then read only the narrow skill body.\n\n\
| Skill | Role | Trigger |\n\
| --- | --- | --- |\n\
| Superpowers | workflow core | planning, TDD, debugging, review, verification |\n\
| `frontend-design` | optional domain | UI, layout, responsive, accessibility, browser-facing flows |\n\
| `vibe-security-scan` | optional domain | auth, API, secrets, RLS, uploads, payment, dependencies, permissions |\n\n\
Skill root: `{root}`.\n\n\
## Custom Skills\n\n\
Register project-specific skills below. Custom skills must not duplicate Superpowers workflow ownership.\n"
    )
}

fn agents_index() -> String {
    "# Baron Agent Routing\n\n\
Use the three core quality agents as gates, not as workflow owners. Do not dispatch agents recursively.\n\n\
| Agent | Gate |\n\
| --- | --- |\n\
| `code-reviewer` | correctness, regressions, maintainability, architecture |\n\
| `security-auditor` | exploitable security and sensitive-memory risks |\n\
| `test-engineer` | verification evidence and missing coverage |\n\n\
## Custom Agents\n\n\
Register optional project-specific agents below without replacing the core gates.\n"
        .to_string()
}

fn write_asset_subtree(source: &str, destination: &Path) -> Result<()> {
    let directory = CORE_ASSETS
        .get_dir(source)
        .with_context(|| format!("Embedded Baron asset directory missing: {source}"))?;
    write_directory(directory, destination)
}

fn write_directory(directory: &Dir<'_>, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for file in directory.files() {
        let relative = file
            .path()
            .strip_prefix(directory.path())
            .unwrap_or(file.path());
        let path = destination.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, file.contents())
            .with_context(|| format!("Could not write {}", path.display()))?;
    }
    for child in directory.dirs() {
        let relative = child
            .path()
            .strip_prefix(directory.path())
            .unwrap_or(child.path());
        write_directory(child, &destination.join(relative))?;
    }
    Ok(())
}

fn write_claude_agents(repo: &Path) -> Result<()> {
    let agents = [
        (
            "code-reviewer",
            "Review findings first. Focus on correctness, regressions, maintainability, architecture fit, and missing tests. Use evidence.",
        ),
        (
            "security-auditor",
            "Report defensive security findings with severity, evidence, impact, fix, and verification. Never provide weaponized exploitation.",
        ),
        (
            "test-engineer",
            "Identify the smallest sufficient proof, missing coverage, and exact verification evidence. Never replace tests with confidence.",
        ),
    ];
    for (name, instructions) in agents {
        let content = format!(
            "---\nname: {name}\ndescription: Baron core quality gate\n---\n\n# {name}\n\n{instructions}\n\nSuperpowers remains the workflow core. Do not orchestrate other agents.\n"
        );
        write_managed_file(
            &repo.join(".claude/agents").join(format!("{name}.md")),
            &content,
        )?;
    }
    write_managed_file(&repo.join(".claude/agents/INDEX.md"), &agents_index())
}

fn report(adapter: &str, files: &[&str]) -> InstallReport {
    InstallReport {
        adapter: adapter.to_string(),
        managed_files: files.iter().map(|value| value.to_string()).collect(),
        preserved_custom_assets: true,
    }
}

#[allow(dead_code)]
fn _normalize(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}
