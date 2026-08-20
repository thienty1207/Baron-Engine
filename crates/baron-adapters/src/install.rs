use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use crate::managed::{upsert_managed_block, upsert_routing_block, write_managed_file};
use crate::{
    ensure_managed_baseline, managed_content_for_kind, AgentAdapter, ManagedAssetPayload,
    ManagedMergeKind,
};

static CORE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../assets/core");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReport {
    pub adapter: String,
    pub managed_files: Vec<String>,
    pub preserved_custom_assets: bool,
    #[serde(default)]
    pub preserved_paths: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

pub fn install_adapter(
    repo_root: impl AsRef<Path>,
    adapter: AgentAdapter,
) -> Result<InstallReport> {
    let repo_root = repo_root.as_ref();
    let report = match adapter {
        AgentAdapter::Codex => install_codex(repo_root),
        AgentAdapter::Claude => install_claude(repo_root),
        AgentAdapter::Generic => install_generic(repo_root),
        AgentAdapter::Reasonix => install_reasonix(repo_root),
    }?;
    let payloads = managed_payloads_for_adapter(adapter)?;
    ensure_managed_baseline(repo_root, &payloads, env!("CARGO_PKG_VERSION"))?;
    Ok(report)
}

/// Renders exactly the Baron-owned portions of an adapter installation without
/// reading or writing a target repository. This is the upstream side of the
/// safe three-way update planner.
pub fn managed_payloads_for_adapter(adapter: AgentAdapter) -> Result<Vec<ManagedAssetPayload>> {
    let adapter_name = adapter_name(adapter).to_string();
    let mut payloads = match adapter {
        AgentAdapter::Codex => vec![
            payload(
                &adapter_name,
                "AGENTS.md",
                ManagedMergeKind::MarkerBlock,
                &managed_block(&startup_contract("Codex", "codex")),
            ),
            payload(
                &adapter_name,
                ".codex/INDEX.md",
                ManagedMergeKind::FullText,
                &codex_index(),
            ),
            payload(
                &adapter_name,
                ".codex/skills/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&skills_index(".codex/skills")),
            ),
            payload(
                &adapter_name,
                ".codex/agents/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&agents_index()),
            ),
            payload(
                &adapter_name,
                ".codex/hooks.json",
                ManagedMergeKind::JsonOwnedEntries,
                &managed_content_for_kind(
                    &native_hooks_document("codex")?,
                    ManagedMergeKind::JsonOwnedEntries,
                )?,
            ),
        ],
        AgentAdapter::Claude => vec![
            payload(
                &adapter_name,
                "CLAUDE.md",
                ManagedMergeKind::MarkerBlock,
                &managed_block(&startup_contract("Claude", "claude")),
            ),
            payload(
                &adapter_name,
                ".claude/commands/baron-context.md",
                ManagedMergeKind::FullText,
                &claude_context_command(),
            ),
            payload(
                &adapter_name,
                ".claude/commands/baron-status.md",
                ManagedMergeKind::FullText,
                &claude_status_command(),
            ),
            payload(
                &adapter_name,
                ".claude/skills/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&skills_index(".claude/skills")),
            ),
            payload(
                &adapter_name,
                ".claude/agents/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&agents_index()),
            ),
            payload(
                &adapter_name,
                ".claude/settings.json",
                ManagedMergeKind::JsonOwnedEntries,
                &managed_content_for_kind(
                    &native_hooks_document("claude")?,
                    ManagedMergeKind::JsonOwnedEntries,
                )?,
            ),
            payload(
                &adapter_name,
                ".claude/agents/code-reviewer.md",
                ManagedMergeKind::FullText,
                &claude_agent_content("code-reviewer", "Review findings first. Focus on correctness, regressions, maintainability, architecture fit, and missing tests. Use evidence."),
            ),
            payload(
                &adapter_name,
                ".claude/agents/security-auditor.md",
                ManagedMergeKind::FullText,
                &claude_agent_content("security-auditor", "Report defensive security findings with severity, evidence, impact, fix, and verification. Never provide weaponized exploitation."),
            ),
            payload(
                &adapter_name,
                ".claude/agents/test-engineer.md",
                ManagedMergeKind::FullText,
                &claude_agent_content("test-engineer", "Identify the smallest sufficient proof, missing coverage, and exact verification evidence. Never replace tests with confidence."),
            ),
            payload(
                &adapter_name,
                ".claude/agents/web-performance-auditor.md",
                ManagedMergeKind::FullText,
                &claude_agent_content("web-performance-auditor", "Optional web performance auditor. Use only for web performance tasks. Never fabricate metrics; mark static findings as potential impact. Not included in mandatory gates."),
            ),
        ],
        AgentAdapter::Generic => vec![
            payload(
                &adapter_name,
                "AGENT.md",
                ManagedMergeKind::MarkerBlock,
                &managed_block(&startup_contract("generic agents", "agent")),
            ),
            payload(
                &adapter_name,
                "baron-context.md",
                ManagedMergeKind::FullText,
                &generic_context_markdown(),
            ),
            payload(
                &adapter_name,
                "baron-context.json",
                ManagedMergeKind::FullText,
                &generic_context_json()?,
            ),
            payload(
                &adapter_name,
                ".baron/core/skills/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&skills_index(".baron/core/skills")),
            ),
            payload(
                &adapter_name,
                ".baron/core/agents/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&agents_index()),
            ),
        ],
        AgentAdapter::Reasonix => vec![
            payload(
                &adapter_name,
                "REASONIX.md",
                ManagedMergeKind::MarkerBlock,
                &managed_block(&startup_contract("DeepSeek Reasonix", "reasonix")),
            ),
            payload(
                &adapter_name,
                ".reasonix/INDEX.md",
                ManagedMergeKind::FullText,
                &reasonix_index(),
            ),
            payload(
                &adapter_name,
                ".reasonix/skills/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&skills_index(".reasonix/skills")),
            ),
            payload(
                &adapter_name,
                ".reasonix/agents/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&agents_index()),
            ),
            payload(
                &adapter_name,
                ".reasonix/commands/baron-context.md",
                ManagedMergeKind::FullText,
                &reasonix_context_command(),
            ),
            payload(
                &adapter_name,
                ".reasonix/commands/baron-status.md",
                ManagedMergeKind::FullText,
                &reasonix_status_command(),
            ),
            payload(
                &adapter_name,
                ".reasonix/settings.json",
                ManagedMergeKind::JsonOwnedEntries,
                &managed_content_for_kind(
                    &reasonix_hooks_document()?,
                    ManagedMergeKind::JsonOwnedEntries,
                )?,
            ),
        ],
    };

    match adapter {
        AgentAdapter::Codex => {
            collect_embedded_asset_payloads(
                "skills",
                Path::new(".codex/skills"),
                &adapter_name,
                &mut payloads,
            )?;
            collect_embedded_asset_payloads(
                "agents",
                Path::new(".codex/agents"),
                &adapter_name,
                &mut payloads,
            )?;
        }
        AgentAdapter::Claude => {
            collect_embedded_asset_payloads(
                "skills",
                Path::new(".claude/skills"),
                &adapter_name,
                &mut payloads,
            )?;
        }
        AgentAdapter::Generic => {
            collect_embedded_asset_payloads(
                "skills",
                Path::new(".baron/core/skills"),
                &adapter_name,
                &mut payloads,
            )?;
            collect_embedded_asset_payloads(
                "agents",
                Path::new(".baron/core/agents"),
                &adapter_name,
                &mut payloads,
            )?;
        }
        AgentAdapter::Reasonix => {
            collect_embedded_asset_payloads(
                "skills",
                Path::new(".reasonix/skills"),
                &adapter_name,
                &mut payloads,
            )?;
            collect_embedded_asset_payloads(
                "agents",
                Path::new(".reasonix/agents"),
                &adapter_name,
                &mut payloads,
            )?;
        }
    }

    Ok(payloads)
}

fn payload(
    adapter: &str,
    relative_path: &str,
    merge_kind: ManagedMergeKind,
    content: &str,
) -> ManagedAssetPayload {
    ManagedAssetPayload {
        adapter: adapter.to_string(),
        relative_path: PathBuf::from(relative_path),
        merge_kind,
        content: content.to_string(),
    }
}

fn managed_block(body: &str) -> String {
    format!(
        "<!-- BARON:MANAGED:START -->\n{}\n<!-- BARON:MANAGED:END -->",
        body.trim()
    )
}

fn routing_block(body: &str) -> String {
    format!(
        "<!-- BARON:ROUTING:START -->\n{}\n<!-- BARON:ROUTING:END -->",
        body.trim()
    )
}

fn collect_embedded_asset_payloads(
    source: &str,
    destination: &Path,
    adapter: &str,
    payloads: &mut Vec<ManagedAssetPayload>,
) -> Result<()> {
    let directory = CORE_ASSETS
        .get_dir(source)
        .with_context(|| format!("Embedded Baron asset directory missing: {source}"))?;
    collect_embedded_directory_payloads(directory, destination, adapter, payloads)?;
    Ok(())
}

fn collect_embedded_directory_payloads(
    directory: &Dir<'_>,
    destination: &Path,
    adapter: &str,
    payloads: &mut Vec<ManagedAssetPayload>,
) -> Result<()> {
    for file in directory.files() {
        let relative = file
            .path()
            .strip_prefix(directory.path())
            .unwrap_or(file.path());
        let content = std::str::from_utf8(file.contents()).with_context(|| {
            format!(
                "Embedded Baron asset is not UTF-8: {}",
                file.path().display()
            )
        })?;
        payloads.push(ManagedAssetPayload {
            adapter: adapter.to_string(),
            relative_path: destination.join(relative),
            merge_kind: ManagedMergeKind::FullText,
            content: content.to_string(),
        });
    }
    for child in directory.dirs() {
        let relative = child
            .path()
            .strip_prefix(directory.path())
            .unwrap_or(child.path());
        collect_embedded_directory_payloads(child, &destination.join(relative), adapter, payloads)?;
    }
    Ok(())
}

fn adapter_name(adapter: AgentAdapter) -> &'static str {
    match adapter {
        AgentAdapter::Codex => "codex",
        AgentAdapter::Claude => "claude",
        AgentAdapter::Generic => "agent",
        AgentAdapter::Reasonix => "reasonix",
    }
}

fn claude_context_command() -> String {
    "# Baron Context\n\nRun `baron capability check --adapter claude`, `baron runtime check --adapter claude`, `baron autopilot status`, and then `baron context --claude` silently. For architecture, dependency, impact, entrypoint, ownership, call-flow, refactor, or cross-module work, run task context first. If its Optional Code Map section requests it, silently run `baron automation code-map refresh` then `baron automation code-map query \"<task>\"`; verify selected source files before using any hit as proof. Capability presence and graph hints are not execution evidence.\n".to_string()
}

fn claude_status_command() -> String {
    "# Baron Status\n\nRun `baron plan status`, `baron harness status`, `baron proof status`, and inspect the latest trace score.\n".to_string()
}

fn generic_context_markdown() -> String {
    "# Baron Context Contract\n\nRun `baron capability check --adapter agent`, `baron runtime check --adapter agent`, `baron autopilot status`, and then `baron context --agent` at session start. For architecture, dependency, impact, entrypoint, ownership, call-flow, refactor, or cross-module work, run task context first. If its Optional Code Map section requests it, silently run `baron automation code-map refresh` then `baron automation code-map query \"<task>\"`; verify selected source files before using any hit as proof. Treat output as bounded orientation, not as a replacement for repository evidence. Capability presence and graph hints are not execution evidence.\n".to_string()
}

fn generic_context_json() -> Result<String> {
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "engine": "baron",
        "adapter": "agent",
        "capabilityCheckCommand": "baron capability check --adapter agent",
        "runtimeCheckCommand": "baron runtime check --adapter agent",
        "autopilotStatusCommand": "baron autopilot status",
        "autopilotReviewCommand": "baron autopilot review \"<summary>\"",
        "contextCommand": "baron context --agent",
        "codeMapRefreshCommand": "baron automation code-map refresh",
        "codeMapQueryCommand": "baron automation code-map query \"<task>\"",
        "codeMapRule": "Use only when task context requests a code map; verify selected source before proof.",
        "automatic": true,
        "sourceOfTruth": ["repository", "vault-markdown"]
    }))?)
}

fn reasonix_context_command() -> String {
    "# Baron Context\n\nRun `baron capability check --adapter reasonix`, `baron runtime check --adapter reasonix`, `baron autopilot status`, and then `baron context --reasonix`. Use the shared Baron Vault for durable memory; Reasonix-local state is only an adapter surface. Read `.reasonix/INDEX.md`, then use `baron control-plane route \"<task>\"` and read only the selected entries from `.reasonix/skills/INDEX.md` and `.reasonix/agents/INDEX.md`; do not recursively load the full tree. For architecture, dependency, impact, entrypoint, ownership, call-flow, refactor, or cross-module work, run task context first and verify selected source files before treating graph hints as proof.\n".to_string()
}

fn reasonix_status_command() -> String {
    "# Baron Status\n\nRun `baron adapter status`, `baron plan status`, `baron harness status`, `baron proof status`, and inspect the latest trace score. The Reasonix skill and agent views under `.reasonix/skills` and `.reasonix/agents` are materialized from the same embedded Baron core as Codex; route them narrowly instead of loading everything. Switch adapters with `baron adapter switch --to <codex|claude|agent|reasonix>`; the project ID, Vault, memory, and history remain shared.\n".to_string()
}

fn reasonix_hooks_document() -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "_baron": {"managed": true, "adapter": "reasonix"},
            "hooks": {
                "SessionStart": [{
                    "command": "baron automation hook session-start --adapter reasonix",
                    "description": "Load shared Baron context",
                    "timeout": 15000
                }],
                "UserPromptSubmit": [{
                    "command": "baron automation hook prompt --adapter reasonix",
                    "description": "Record the submitted task in the shared Vault",
                    "timeout": 5000
                }],
                "Stop": [{
                    "command": "baron automation hook stop --adapter reasonix",
                    "description": "Persist the shared handoff without inferring completion",
                    "timeout": 5000
                }]
            }
        }))?
    ))
}

fn claude_agent_content(name: &str, instructions: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: Baron core quality gate\n---\n\n# {name}\n\n{instructions}\n\nSuperpowers remains the workflow core. Do not orchestrate other agents.\n"
    )
}

fn native_hooks_document(adapter: &str) -> Result<String> {
    let mut hooks = serde_json::Map::new();
    for (event, command, matcher) in [
        ("SessionStart", "session-start", None),
        ("UserPromptSubmit", "prompt", None),
        ("PostToolUse", "checkpoint", Some("Edit|Write|apply_patch")),
        ("Stop", "stop", None),
    ] {
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
        hooks.insert(event.to_string(), serde_json::Value::Array(vec![group]));
    }
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({ "hooks": hooks }))?
    ))
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
        &claude_context_command(),
    )?;
    write_managed_file(
        &repo.join(".claude/commands/baron-status.md"),
        &claude_status_command(),
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
    write_managed_file(&repo.join("baron-context.md"), &generic_context_markdown())?;
    write_managed_file(&repo.join("baron-context.json"), &generic_context_json()?)?;
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

fn install_reasonix(repo: &Path) -> Result<InstallReport> {
    let mut preserved_paths = Vec::new();
    let mut conflicts = Vec::new();

    install_reasonix_contract(
        &repo.join("REASONIX.md"),
        &startup_contract("DeepSeek Reasonix", "reasonix"),
        &mut preserved_paths,
        &mut conflicts,
    )?;
    install_reasonix_file(
        &repo.join(".reasonix/INDEX.md"),
        &reasonix_index(),
        &mut preserved_paths,
        &mut conflicts,
    )?;
    install_reasonix_routing_file(
        &repo.join(".reasonix/skills/INDEX.md"),
        &skills_index(".reasonix/skills"),
    )?;
    install_reasonix_routing_file(&repo.join(".reasonix/agents/INDEX.md"), &agents_index())?;
    write_asset_subtree_preserving(
        "skills",
        &repo.join(".reasonix/skills"),
        repo,
        &mut preserved_paths,
        &mut conflicts,
    )?;
    write_asset_subtree_preserving(
        "agents",
        &repo.join(".reasonix/agents"),
        repo,
        &mut preserved_paths,
        &mut conflicts,
    )?;
    install_reasonix_file(
        &repo.join(".reasonix/commands/baron-context.md"),
        &reasonix_context_command(),
        &mut preserved_paths,
        &mut conflicts,
    )?;
    install_reasonix_file(
        &repo.join(".reasonix/commands/baron-status.md"),
        &reasonix_status_command(),
        &mut preserved_paths,
        &mut conflicts,
    )?;
    install_reasonix_settings(
        &repo.join(".reasonix/settings.json"),
        &mut preserved_paths,
        &mut conflicts,
    )?;

    Ok(report_with_details(
        "reasonix",
        &[
            "REASONIX.md",
            ".reasonix/INDEX.md",
            ".reasonix/skills/INDEX.md",
            ".reasonix/agents/INDEX.md",
            ".reasonix/commands/baron-context.md",
            ".reasonix/commands/baron-status.md",
            ".reasonix/settings.json",
        ],
        preserved_paths,
        conflicts,
    ))
}

fn install_reasonix_routing_file(path: &Path, content: &str) -> Result<()> {
    upsert_routing_block(
        path,
        content,
        "## Custom Reasonix Routing",
        "Register project-specific Reasonix routing below without replacing Baron core gates.",
    )
}

fn write_asset_subtree_preserving(
    source: &str,
    destination: &Path,
    repo_root: &Path,
    preserved_paths: &mut Vec<String>,
    conflicts: &mut Vec<String>,
) -> Result<()> {
    let directory = CORE_ASSETS
        .get_dir(source)
        .with_context(|| format!("Embedded Baron asset directory missing: {source}"))?;
    write_directory_preserving(
        directory,
        destination,
        repo_root,
        preserved_paths,
        conflicts,
    )
}

fn write_directory_preserving(
    directory: &Dir<'_>,
    destination: &Path,
    repo_root: &Path,
    preserved_paths: &mut Vec<String>,
    conflicts: &mut Vec<String>,
) -> Result<()> {
    fs::create_dir_all(destination)?;
    for file in directory.files() {
        let relative = file
            .path()
            .strip_prefix(directory.path())
            .unwrap_or(file.path());
        let path = destination.join(relative);
        if path.exists() {
            let unchanged = path.is_file() && fs::read(&path)? == file.contents();
            if !unchanged {
                let relative = path
                    .strip_prefix(repo_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                preserved_paths.push(relative.clone());
                conflicts.push(relative);
            }
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, file.contents())
                .with_context(|| format!("Could not write {}", path.display()))?;
            apply_embedded_mode(&path, file.contents())?;
        }
    }
    for child in directory.dirs() {
        let relative = child
            .path()
            .strip_prefix(directory.path())
            .unwrap_or(child.path());
        write_directory_preserving(
            child,
            &destination.join(relative),
            repo_root,
            preserved_paths,
            conflicts,
        )?;
    }
    Ok(())
}

fn install_reasonix_contract(
    path: &Path,
    body: &str,
    preserved_paths: &mut Vec<String>,
    conflicts: &mut Vec<String>,
) -> Result<()> {
    if !path.exists() {
        upsert_managed_block(path, body)?;
        return Ok(());
    }
    let existing = fs::read_to_string(path)
        .with_context(|| format!("Could not read existing {}", path.display()))?;
    if existing.contains("BARON:MANAGED:START") && existing.contains("BARON:MANAGED:END") {
        upsert_managed_block(path, body)?;
    } else {
        let relative = path.to_string_lossy().replace('\\', "/");
        preserved_paths.push(relative.clone());
        conflicts.push(relative);
    }
    Ok(())
}

fn install_reasonix_file(
    path: &Path,
    content: &str,
    preserved_paths: &mut Vec<String>,
    conflicts: &mut Vec<String>,
) -> Result<()> {
    if !path.exists() {
        write_managed_file(path, content)?;
        return Ok(());
    }
    let existing = fs::read_to_string(path)
        .with_context(|| format!("Could not read existing {}", path.display()))?;
    if existing == content {
        return Ok(());
    }
    let relative = path.to_string_lossy().replace('\\', "/");
    preserved_paths.push(relative.clone());
    conflicts.push(relative);
    Ok(())
}

fn install_reasonix_settings(
    path: &Path,
    preserved_paths: &mut Vec<String>,
    conflicts: &mut Vec<String>,
) -> Result<()> {
    if !path.exists() {
        write_managed_file(path, &reasonix_hooks_document()?)?;
        return Ok(());
    }
    let existing = fs::read_to_string(path)
        .with_context(|| format!("Could not read existing {}", path.display()))?;
    let mut root: serde_json::Value = match serde_json::from_str(&existing) {
        Ok(value) => value,
        Err(_) => {
            let relative = path.to_string_lossy().replace('\\', "/");
            preserved_paths.push(relative.clone());
            conflicts.push(relative);
            return Ok(());
        }
    };
    let managed = root
        .get("_baron")
        .and_then(|value| value.get("managed"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !managed {
        let relative = path.to_string_lossy().replace('\\', "/");
        preserved_paths.push(relative.clone());
        conflicts.push(relative);
        return Ok(());
    }
    let root_object = root
        .as_object_mut()
        .context("Reasonix settings must be a JSON object")?;
    root_object.insert(
        "_baron".to_string(),
        serde_json::json!({"managed": true, "adapter": "reasonix"}),
    );
    let hooks = root_object
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }
    let hooks_object = hooks
        .as_object_mut()
        .context("Reasonix hook registry must be a JSON object")?;
    for (event, command, description, timeout) in [
        (
            "SessionStart",
            "session-start",
            "Load shared Baron context",
            15000,
        ),
        (
            "UserPromptSubmit",
            "prompt",
            "Record the submitted task in the shared Vault",
            5000,
        ),
        (
            "Stop",
            "stop",
            "Persist the shared handoff without inferring completion",
            5000,
        ),
    ] {
        let entries = hooks_object
            .entry(event.to_string())
            .or_insert_with(|| serde_json::json!([]));
        if !entries.is_array() {
            *entries = serde_json::json!([]);
        }
        let entries = entries
            .as_array_mut()
            .context("Reasonix hook event must contain an array")?;
        entries.retain(|entry| !entry.to_string().contains("baron automation hook"));
        entries.push(serde_json::json!({
            "command": format!("baron automation hook {command} --adapter reasonix"),
            "description": description,
            "timeout": timeout
        }));
    }
    write_managed_file(path, &format!("{}\n", serde_json::to_string_pretty(&root)?))?;
    Ok(())
}

fn startup_contract(agent: &str, adapter: &str) -> String {
    format!(
        "# Baron Automatic Agent Contract\n\n\
    You are using Baron through the {agent} adapter.\n\n\
    - Before any durable Baron or repository write, silently run `baron authority classify \"<requested outcome>\"`; classify by the outcome, so `review and apply fixes` is a change request even though it contains `review`.\n\
    - Silently run `baron work-shape \"<task>\" --json` before meaningful work. Read-only tasks create no lifecycle state; focused reversible changes use focused proof; risky, coordinated, ambiguous, or multi-session work keeps the full plan, intent, recovery, gate, proof, and trace path.\n\
    - When authority is `read_only` or `ambiguous`, inspect only what is needed to answer and do not create or update plan, Harness, proof, trace, review, friction, or learning state. Ambiguous authority stays read-only until the user explicitly requests a change.\n\
    - If Baron reports project, Vault, capsule, schema, or identity mismatch, stop durable work and run `baron automation reconcile`; never repair Baron metadata by hand or continue with guessed state. If local reconciliation cannot restore coherent state, tell the user that a human must run public `baron update`. Never run public `baron update`: only a human may authorize a release download or runtime replacement.\n\
    - At session start, silently run `baron capability check --adapter {adapter}`; capability presence is not execution evidence.\n\
    - At session start, silently run `baron runtime check --adapter {adapter}`; unsafe, missing, or unverified required backends cannot satisfy proof.\n\
    - At session start, silently run `baron context --{adapter}` and follow its bounded context.\n\
    - For architecture, dependency, impact, ownership, entrypoint, call-flow, refactor, or cross-module work, silently run task context first. If its Optional Code Map section requests it, silently run `baron automation code-map refresh` and then `baron automation code-map query \"<task>\"`; verify selected source files before edits, proof, traces, or durable memory. Inferred graph results are reading hints only.\n\
    - At session start and after interruption, silently inspect `baron continuity status`; resume from its current task, last checkpoint, proof status, trace status, and next action before guessing.\n\
    - At session start and after interruption, silently inspect `baron autopilot status`; treat learning candidates as candidates, not facts.\n\
    - Respect `.baron/project.toml` platform focus when present; use it to prioritize domain knowledge without creating new workflow ownership.\n\
    - Read `docs/baron/platform/PROJECT_PROFILE.md` and `docs/baron/architecture/CURRENT_ARCHITECTURE.md` before structural work; load only the task-relevant profile.\n\
    - For cross-module, product, API, data-model, or interface work, silently inspect `docs/baron/harness/DOMAIN_LANGUAGE.md`. Use canonical terms only when their evidence and status support them; record a new term only when it changes cross-module understanding; mark disagreement as `ambiguous` rather than silently choosing a definition. Product Harness owns this document; Superpowers still owns workflow.\n\
    - When the user explicitly expands the product to a new platform, silently run `baron init --<platform>` so Baron adds an extension and regenerates non-destructive architecture contracts; never rearrange existing code automatically.\n\
    - Silently inspect `baron plan status` and `baron harness status` before meaningful implementation.\n\
    - For a new or materially changed task, read repo, Vault, current plan, Harness, continuity, and decisions before asking; ask exactly one missing high-value question at a time and mark unresolved facts unknown.\n\
    - Silently inspect `baron harness intent-status`. Before medium/high-risk intake, record the current/target behavior, scope, proof, constraints, decisions, non-goals, and unknowns with `baron harness intent`; do not pass `--confirmed` until the user explicitly confirms the reflected intent.\n\
    - Before selecting skills or agents for meaningful work, silently run `baron control-plane route \"<task>\"` and follow the selected/skipped explanation.\n\
    - Use Baron-managed skills and agents as local self-contained assets; if a custom asset looks weak, conflicting, or externally dependent, run `baron asset audit` before trusting it.\n\
    - When prior conversation context may matter, rely on `baron context --{adapter} --task \"<task>\"` or `baron session-replay search \"<query>\"`; do not dump full session histories.\n\
    - Start or resume a Baron plan before editing code for a meaningful task.\n\
    - Create harness intake for medium/high-risk work.\n\
    - Before edits, direction changes, interruptions, and final responses for meaningful work, record `baron continuity checkpoint \"<current state and next action>\"`.\n\
    - If work fails, blocks, or remains interrupted, silently record `baron continuity recover \"<root cause>\" --outcome <failed|blocked|interrupted> --last-success \"<last successful step>\" --next-action \"<safe next action>\"` with available evidence, affected files, and retry conditions; preserve the failed attempt even after a later retry succeeds.\n\
    - Before final response after meaningful work, run `baron autopilot review \"<task summary, proof state, remaining risks>\"`; it may propose learning, but it must not rewrite trusted facts or runtime assets without approval.\n\
    - Use Superpowers as the workflow core for planning, TDD, debugging, review, and verification.\n\
    - Read the routed skill and agent indexes; do not recursively load every skill or agent.\n\
    - For execution-required proof, use Baron-owned `baron proof execute --capability <capability> --provider <provider> -- <executable> <args...>`; record the returned receipt with `baron proof record \"<summary>\" --receipt <receipt-id>`. A sentence or hand-written receipt is not execution proof.\n\
    - After each mandatory quality gate actually runs, record it with `baron control-plane record-gate <agent> \"<evidence summary>\" --receipt <receipt-id>`; the legacy form remains reported evidence only.\n\
    - For concrete reviewer findings, silently run `baron review finding \"<summary>\" --severity <level> --evidence \"<evidence>\"`; keep findings open until the fix exists.\n\
    - Close a finding only with `baron review close <id> --fix-evidence \"<what changed>\" --verification \"<command/result>\"`; fix evidence and verification are both mandatory.\n\
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

fn reasonix_index() -> String {
    "# Baron Reasonix Workspace\n\nStart with `REASONIX.md`. Read `.reasonix/skills/INDEX.md` and `.reasonix/agents/INDEX.md` only after `baron control-plane route \"<task>\"` selects a relevant capability. Reasonix is an adapter over the shared Baron core: use the same project ID, Vault, memory, Wiki, CodeGraph, plan, proof, trace, continuity, and autopilot state as every other adapter.\n".to_string()
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
  | `vibe-security-scan` | optional defensive security domain | auth, API, secrets, RLS, uploads, payment, dependencies, permissions | visual-only or copy-only tasks | severity, evidence, fix, verification | must not replace `security-auditor` final gate |\n\
  | `binary-reverse-analysis` | optional defensive reverse domain | binary triage, disassembly, static reverse engineering | live exploitation, persistence, evasion, CTF/pwn | scope, artifact hash, tool evidence, uncertainty | must not install tools or replace security-auditor |\n\
  | `apk-mobile-analysis` | optional defensive mobile domain | APK/Android manifest, permissions, static mobile review | credential theft, live exploitation, auto-bootstrap | artifact hash, static evidence, verification | must not replace vibe-security-scan or security-auditor |\n\
  | `malware-triage` | optional defensive malware domain | offline sample triage, indicators, containment notes | payload execution, persistence, evasion, live delivery | hash, isolated evidence, safe handling | must not execute samples or replace security-auditor |\n\n\
| `api-and-interface-design` | optional API/interface domain | API contracts, request/response shape, SDK/public interface, compatibility, or deep module boundary | implementation-only tasks that do not change boundaries | contract risks, versioning impact, boundary evidence, verification | must not replace Superpowers planning or tests |\n\
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
        apply_embedded_mode(&path, file.contents())?;
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

fn desired_embedded_mode(contents: &[u8]) -> Option<u32> {
    contents.starts_with(b"#!").then_some(0o755)
}

#[cfg(unix)]
fn apply_embedded_mode(path: &Path, contents: &[u8]) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if let Some(mode) = desired_embedded_mode(contents) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .with_context(|| format!("Could not set executable mode on {}", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn apply_embedded_mode(_path: &Path, contents: &[u8]) -> Result<()> {
    let _ = desired_embedded_mode(contents);
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
        write_managed_file(
            &repo.join(".claude/agents").join(format!("{name}.md")),
            &claude_agent_content(name, instructions),
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
    report_with_details(adapter, files, Vec::new(), Vec::new())
}

fn report_with_details(
    adapter: &str,
    files: &[&str],
    preserved_paths: Vec<String>,
    conflicts: Vec<String>,
) -> InstallReport {
    InstallReport {
        adapter: adapter.to_string(),
        managed_files: files.iter().map(|value| value.to_string()).collect(),
        preserved_custom_assets: true,
        preserved_paths,
        conflicts,
    }
}

#[allow(dead_code)]
fn _normalize(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::desired_embedded_mode;

    #[test]
    fn embedded_shebang_assets_are_executable_but_docs_are_not() {
        assert_eq!(desired_embedded_mode(b"#!/usr/bin/env bash\n"), Some(0o755));
        assert_eq!(desired_embedded_mode(b"# Skill\n"), None);
        assert_eq!(desired_embedded_mode(b""), None);
    }
}
