use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use crate::managed::{upsert_managed_block, upsert_routing_block, write_managed_file};
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
    upsert_managed_block(&repo.join("AGENTS.md"), &startup_contract("Codex", "codex"))?;
    write_managed_file(&repo.join(".codex/INDEX.md"), &codex_index())?;
    upsert_routing_block(
        &repo.join(".codex/skills/INDEX.md"),
        &skills_index(".codex/skills"),
        "## Custom Skills",
        "Register project-specific skills below. Custom skills must not duplicate Superpowers workflow ownership.",
    )?;
    upsert_routing_block(
        &repo.join(".codex/agents/INDEX.md"),
        &agents_index(),
        "## Custom Agents",
        "Register optional project-specific agents below without replacing the core gates.",
    )?;
    write_asset_subtree("skills", &repo.join(".codex/skills"))?;
    write_asset_subtree("agents", &repo.join(".codex/agents"))?;
    install_native_hooks(&repo.join(".codex/hooks.json"), "codex")?;
    Ok(report(
        "codex",
        &[
            "AGENTS.md",
            ".codex/INDEX.md",
            ".codex/skills/INDEX.md",
            ".codex/agents/INDEX.md",
            ".codex/hooks.json",
        ],
    ))
}

fn install_claude(repo: &Path) -> Result<InstallReport> {
    upsert_managed_block(
        &repo.join("CLAUDE.md"),
        &startup_contract("Claude", "claude"),
    )?;
    write_managed_file(
        &repo.join(".claude/commands/baron-context.md"),
        "# Baron Context\n\nRun `baron capability check --adapter claude` and then `baron context --claude` silently. Follow the bounded context bundle. Capability presence is not execution evidence.\n",
    )?;
    write_managed_file(
        &repo.join(".claude/commands/baron-status.md"),
        "# Baron Status\n\nRun `baron plan status`, `baron harness status`, `baron proof status`, and inspect the latest trace score.\n",
    )?;
    upsert_routing_block(
        &repo.join(".claude/skills/INDEX.md"),
        &skills_index(".claude/skills"),
        "## Custom Skills",
        "Register project-specific skills below. Custom skills must not duplicate Superpowers workflow ownership.",
    )?;
    write_asset_subtree("skills", &repo.join(".claude/skills"))?;
    write_claude_agents(repo)?;
    install_native_hooks(&repo.join(".claude/settings.json"), "claude")?;
    Ok(report(
        "claude",
        &[
            "CLAUDE.md",
            ".claude/commands/baron-context.md",
            ".claude/commands/baron-status.md",
            ".claude/skills/INDEX.md",
            ".claude/settings.json",
        ],
    ))
}

fn install_native_hooks(path: &Path, adapter: &str) -> Result<()> {
    let mut root = fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !root.is_object() {
        root = serde_json::json!({});
    }
    let root_object = root
        .as_object_mut()
        .context("Native hook configuration must be a JSON object")?;
    let hooks = root_object
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }
    let hooks = hooks
        .as_object_mut()
        .context("Native hook registry must be a JSON object")?;

    for (event, command, matcher) in [
        ("SessionStart", "session-start", None),
        ("UserPromptSubmit", "prompt", None),
        ("PostToolUse", "checkpoint", Some("Edit|Write|apply_patch")),
        ("Stop", "stop", None),
    ] {
        let entries = hooks.entry(event).or_insert_with(|| serde_json::json!([]));
        if !entries.is_array() {
            *entries = serde_json::json!([]);
        }
        let entries = entries
            .as_array_mut()
            .context("Native hook event must contain an array")?;
        entries.retain(|entry| !entry.to_string().contains("baron automation hook"));
        let mut group = serde_json::json!({
            "hooks": [{
                "type": "command",
                "command": format!("baron automation hook {command} --adapter {adapter}"),
                "commandWindows": format!("baron automation hook {command} --adapter {adapter}"),
                "timeout": 120
            }]
        });
        if let Some(matcher) = matcher {
            group["matcher"] = serde_json::Value::String(matcher.to_string());
        }
        entries.push(group);
    }
    write_managed_file(path, &format!("{}\n", serde_json::to_string_pretty(&root)?))
}

fn install_generic(repo: &Path) -> Result<InstallReport> {
    upsert_managed_block(
        &repo.join("AGENT.md"),
        &startup_contract("generic agents", "agent"),
    )?;
    write_managed_file(
        &repo.join("baron-context.md"),
        "# Baron Context Contract\n\nRun `baron capability check --adapter agent` and then `baron context --agent` at session start. Treat output as bounded orientation, not as a replacement for repository evidence. Capability presence is not execution evidence.\n",
    )?;
    write_managed_file(
        &repo.join("baron-context.json"),
        &serde_json::to_string_pretty(&serde_json::json!({
            "engine": "baron",
            "adapter": "agent",
            "capabilityCheckCommand": "baron capability check --adapter agent",
            "contextCommand": "baron context --agent",
            "automatic": true,
            "sourceOfTruth": ["repository", "vault-markdown"]
        }))?,
    )?;
    upsert_routing_block(
        &repo.join(".baron/core/skills/INDEX.md"),
        &skills_index(".baron/core/skills"),
        "## Custom Skills",
        "Register project-specific skills below. Custom skills must not duplicate Superpowers workflow ownership.",
    )?;
    upsert_routing_block(
        &repo.join(".baron/core/agents/INDEX.md"),
        &agents_index(),
        "## Custom Agents",
        "Register optional project-specific agents below without replacing the core gates.",
    )?;
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

fn startup_contract(agent: &str, adapter: &str) -> String {
    format!(
        "# Baron Automatic Agent Contract\n\n\
You are using Baron through the {agent} adapter.\n\n\
- At session start, silently run `baron capability check --adapter {adapter}`; capability presence is not execution evidence.\n\
- At session start, silently run `baron context --{adapter}` and follow its bounded context.\n\
- At session start and after interruption, silently inspect `baron continuity status`; resume from its current task, last checkpoint, proof status, trace status, and next action before guessing.\n\
- Respect `.baron/project.toml` platform focus when present; use it to prioritize domain knowledge without creating new workflow ownership.\n\
- Silently inspect `baron plan status` and `baron harness status` before meaningful implementation.\n\
- Before selecting skills or agents for meaningful work, silently run `baron control-plane route \"<task>\"` and follow the selected/skipped explanation.\n\
- Start or resume a Baron plan before editing code for a meaningful task.\n\
- Create harness intake for medium/high-risk work.\n\
- Before edits, direction changes, interruptions, and final responses for meaningful work, record `baron continuity checkpoint \"<current state and next action>\"`.\n\
- Use Superpowers as the workflow core for planning, TDD, debugging, review, and verification.\n\
- Read the routed skill and agent indexes; do not recursively load every skill or agent.\n\
- After each mandatory quality gate actually runs, record it with `baron control-plane record-gate <agent> \"<evidence summary>\"`.\n\
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
Run `baron control-plane route \"<task>\"` before loading optional skills.\n\n\
| Skill | Ownership | Trigger | Exclusion | Evidence | Conflicts |\n\
| --- | --- | --- | --- | --- | --- |\n\
| Superpowers | workflow core | planning, TDD, debugging, review, verification | never optional | plan/proof/trace discipline | no other skill may claim workflow ownership |\n\
| `frontend-design` | optional frontend domain | UI, layout, responsive, accessibility, browser-facing flows | backend-only, CLI-only, security-only tasks | files/screens reviewed, UI verification | must not replace Superpowers or quality gates |\n\
| `vibe-security-scan` | optional defensive security domain | auth, API, secrets, RLS, uploads, payment, dependencies, permissions | visual-only or copy-only tasks | severity, evidence, fix, verification | must not replace `security-auditor` final gate |\n\n\
| `api-and-interface-design` | optional API/interface domain | API contracts, request/response shape, SDK/public interface, compatibility | implementation-only tasks that do not change boundaries | contract risks, versioning impact, verification | must not replace Superpowers planning or tests |\n\
| `observability-and-instrumentation` | optional operations domain | logs, metrics, tracing, alerts, SLOs, audit events, diagnostics | tasks with no runtime/operations impact | signal list, gaps, proof hooks | must not fabricate production behavior |\n\
| `performance-optimization` | optional performance domain | latency, runtime speed, bundle size, cache, loading, database/query performance | cosmetic-only or security-only tasks | measured or potential impact, verification | must not fabricate metrics |\n\
| `deprecation-and-migration` | optional migration domain | legacy behavior, migrations, deprecations, compatibility, rollout/rollback | greenfield work with no compatibility risk | migration plan, compatibility proof, rollback | must not bypass proof gates |\n\n\
Skill root: `{root}`.\n"
    )
}

fn agents_index() -> String {
    "# Baron Agent Routing\n\n\
Use the three core quality agents as gates, not as workflow owners. Do not dispatch agents recursively.\n\n\
Run `baron control-plane route \"<task>\"` before dispatch. After a gate actually runs, record evidence with `baron control-plane record-gate`.\n\n\
| Agent | Ownership | Trigger | Exclusion | Evidence | Conflicts |\n\
| --- | --- | --- | --- | --- | --- |\n\
| `code-reviewer` | core quality gate | meaningful code change, medium/high-risk work | pure docs/status-only updates unless requested | findings or no-issue review with files/proof/trace gaps | must not plan, implement, or call subagents |\n\
| `security-auditor` | core security gate | auth, permission, tenant/RLS, secrets, upload, payment, dependency, security-sensitive work | non-security low-risk work | severity, evidence, impact, fix, verification | must not provide weaponized exploit steps or call subagents |\n\
| `test-engineer` | core verification gate | implementation, bugfix, release, proof, regression concern | none for meaningful implementation | exact commands, outcomes, missing coverage | must not replace actual test/proof execution |\n\
| `web-performance-auditor` | optional web performance gate | Core Web Vitals, Lighthouse, LCP, INP, CLS, bundle/loading/rendering performance | non-web or non-performance tasks | metric source or potential-impact label | optional web performance only; not included in mandatory gates |\n"
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
        (
            "web-performance-auditor",
            "Optional web performance auditor. Use only for web performance tasks. Never fabricate metrics; mark static findings as potential impact. Not included in mandatory gates.",
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
    upsert_routing_block(
        &repo.join(".claude/agents/INDEX.md"),
        &agents_index(),
        "## Custom Agents",
        "Register optional project-specific agents below without replacing the core gates.",
    )
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
