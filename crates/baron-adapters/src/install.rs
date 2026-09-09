#[cfg(unix)]
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use baron_core::safe_io::{acquire_project_lock, read_bytes, read_text, replace_file};

use crate::managed::{upsert_managed_block, upsert_routing_block, write_managed_file};
use crate::update::managed_manifest_exists;
use crate::{
    core_reconcile_managed_assets, ensure_managed_baseline, load_managed_baseline,
    managed_baseline_content, managed_content_for_kind, migrate_managed_ownership, AgentAdapter,
    ManagedAssetPayload, ManagedMergeKind,
};

static CORE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../assets/core");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReport {
    pub adapter: String,
    pub managed_files: Vec<String>,
    pub preserved_custom_assets: bool,
    #[serde(default)]
    pub core: CoreInstallReport,
    #[serde(default)]
    pub preserved_paths: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

/// Evidence for the canonical `.baron/core/**` publication that precedes a
/// Codex or Claude adapter install. Core conflicts are reported separately so
/// an adapter can still preserve and install its own integration files while
/// never claiming or overwriting an ambiguous semantic asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CoreInstallReport {
    #[serde(default)]
    pub managed_files: Vec<String>,
    #[serde(default)]
    pub changed_paths: Vec<String>,
    #[serde(default)]
    pub preserved_paths: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
}

/// Render the embedded Baron semantic runtime as one Core-owned payload set.
/// Every nested reference, script, and support file is included so installed
/// relative paths continue to resolve from their canonical skill directory.
pub fn core_managed_payloads() -> Result<Vec<ManagedAssetPayload>> {
    let mut payloads = Vec::new();
    collect_embedded_asset_payloads(
        "skills",
        Path::new(".baron/core/skills"),
        "core",
        &mut payloads,
    )?;
    collect_embedded_asset_payloads(
        "agents",
        Path::new(".baron/core/agents"),
        "core",
        &mut payloads,
    )?;
    payloads.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(payloads)
}

pub fn install_adapter(
    repo_root: impl AsRef<Path>,
    adapter: AgentAdapter,
) -> Result<InstallReport> {
    let repo_root = repo_root.as_ref();
    let core = Some(ensure_core_runtime(repo_root)?);
    let _lock = acquire_project_lock(repo_root)?;
    preflight_native_hooks(repo_root, adapter)?;
    migrate_managed_ownership(repo_root)?;
    let mut report = match adapter {
        AgentAdapter::Codex => install_codex(repo_root),
        AgentAdapter::Claude => install_claude(repo_root),
    }?;
    let mut payloads = managed_payloads_for_adapter(adapter)?;
    payloads.retain(|payload| {
        let relative = payload.relative_path.to_string_lossy().replace('\\', "/");
        !report.conflicts.iter().any(|path| path == &relative)
    });
    ensure_managed_baseline(repo_root, &payloads, env!("CARGO_PKG_VERSION"))?;
    if let Some(core) = core {
        report.core = core;
    }
    Ok(report)
}

/// Ensure the packaged semantic runtime is present at `.baron/core/**`.
///
/// Existing managed Core records are reconciled through the Phase 3 three-way
/// planner. A fresh repository publishes only files that are absent or byte
/// identical; unknown or modified targets remain untouched and are reported as
/// conflicts. Adapter-owned records are retained in the same baseline but are
/// deliberately excluded from Core reconciliation.
pub fn ensure_core_runtime(repo_root: impl AsRef<Path>) -> Result<CoreInstallReport> {
    let repo_root = repo_root.as_ref();
    let _lock = acquire_project_lock(repo_root)?;
    migrate_managed_ownership(repo_root)?;
    let payloads = core_managed_payloads()?;
    let managed_files = payloads
        .iter()
        .map(|payload| payload.relative_path.to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>();

    if managed_manifest_exists(repo_root)? {
        let report =
            core_reconcile_managed_assets(repo_root, &payloads, env!("CARGO_PKG_VERSION"))?;
        return Ok(CoreInstallReport {
            managed_files,
            changed_paths: report
                .applied_paths
                .into_iter()
                .map(|path| path.to_string_lossy().replace('\\', "/"))
                .collect(),
            preserved_paths: report.preserved_paths,
            conflicts: report
                .conflicts
                .into_iter()
                .map(|path| path.to_string_lossy().replace('\\', "/"))
                .collect(),
        });
    }

    let mut installable = Vec::new();
    let mut writes = Vec::new();
    let mut conflicts = Vec::new();
    for payload in &payloads {
        let path = repo_root.join(&payload.relative_path);
        match read_bytes(&path)? {
            None => {
                writes.push((path, payload.content.as_bytes().to_vec()));
                installable.push(payload.clone());
            }
            Some(existing) if existing == payload.content.as_bytes() => {
                installable.push(payload.clone());
            }
            Some(_) => {
                conflicts.push(payload.relative_path.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    let mut applied = Vec::new();
    for (path, content) in &writes {
        if let Err(error) =
            replace_file(path, content).and_then(|_| apply_embedded_mode(path, content))
        {
            rollback_core_files(&applied)?;
            return Err(error.context("Canonical Core publication failed and was rolled back"));
        }
        applied.push(path.clone());
    }
    if !installable.is_empty() {
        if let Err(error) =
            ensure_managed_baseline(repo_root, &installable, env!("CARGO_PKG_VERSION"))
        {
            rollback_core_files(&applied)?;
            return Err(
                error.context("Canonical Core baseline publication failed and was rolled back")
            );
        }
    }

    Ok(CoreInstallReport {
        managed_files,
        changed_paths: applied
            .into_iter()
            .map(|path| {
                path.strip_prefix(repo_root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect(),
        preserved_paths: conflicts.clone(),
        conflicts,
    })
}

fn rollback_core_files(paths: &[PathBuf]) -> Result<()> {
    for path in paths.iter().rev() {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                std::fs::remove_file(path)
                    .with_context(|| format!("Could not roll back Core file {}", path.display()))?;
            }
            Ok(_) => bail!(
                "Core rollback target is not a regular file: {}",
                path.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

/// Renders exactly the Baron-owned portions of an adapter installation without
/// reading or writing a target repository. This is the upstream side of the
/// safe three-way update planner.
pub fn managed_payloads_for_adapter(adapter: AgentAdapter) -> Result<Vec<ManagedAssetPayload>> {
    let adapter_name = adapter_name(adapter).to_string();
    let payloads = match adapter {
        AgentAdapter::Codex => vec![
            payload(
                &adapter_name,
                "AGENTS.md",
                ManagedMergeKind::MarkerBlock,
                &managed_block(&codex_startup_contract()),
            ),
            payload(
                &adapter_name,
                ".agents/skills/baron-engine/SKILL.md",
                ManagedMergeKind::FullText,
                &codex_bridge_skill(),
            ),
            payload(
                &adapter_name,
                ".agents/skills/baron-engine/agents/openai.yaml",
                ManagedMergeKind::FullText,
                &codex_openai_metadata(),
            ),
            payload(
                &adapter_name,
                ".codex/INDEX.md",
                ManagedMergeKind::FullText,
                &codex_index(),
            ),
            payload(
                &adapter_name,
                ".codex/agents/INDEX.md",
                ManagedMergeKind::RoutingBlock,
                &routing_block(&agents_index()),
            ),
            codex_agent_payload("code-reviewer"),
            codex_agent_payload("security-auditor"),
            codex_agent_payload("test-engineer"),
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
                &managed_block(&claude_startup_contract()),
            ),
            payload(
                &adapter_name,
                ".claude/skills/baron-engine/SKILL.md",
                ManagedMergeKind::FullText,
                &claude_bridge_skill(),
            ),
            claude_agent_payload("code-reviewer"),
            claude_agent_payload("security-auditor"),
            claude_agent_payload("test-engineer"),
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
                &routing_block(&skills_index(".baron/core/skills")),
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
        ],
    };

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
    }
}

fn claude_context_command() -> String {
    "# Baron Context\n\nRun `baron capability check --adapter claude`, `baron runtime check --adapter claude`, `baron autopilot status`, and then `baron context --claude` silently. For architecture, dependency, impact, entrypoint, ownership, call-flow, refactor, or cross-module work, run task context first. If its Optional Code Map section requests it, silently run `baron automation code-map refresh` then `baron automation code-map query \"<task>\"`; verify selected source files before using any hit as proof. Capability presence and graph hints are not execution evidence.\n".to_string()
}

fn claude_status_command() -> String {
    "# Baron Status\n\nRun `baron plan status`, `baron harness status`, `baron proof status`, and inspect the latest trace score.\n".to_string()
}

fn native_hooks_document(adapter: &str) -> Result<String> {
    let mut hooks = serde_json::Map::new();
    for (event, command, matcher) in [
        ("SessionStart", "session-start", None),
        ("UserPromptSubmit", "user-prompt-submit", None),
        ("PreCompact", "pre-compact", None),
        // Older hosts delivered a cheap checkpoint after mutating tools. Keep
        // this compatibility accelerator while routing it through the
        // canonical PreCompact handler.
        ("PostToolUse", "pre-compact", Some("Edit|Write|apply_patch")),
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
    let mut preserved_paths = Vec::new();
    let mut conflicts = Vec::new();
    upsert_managed_block(&repo.join("AGENTS.md"), &codex_startup_contract())?;
    for (relative_path, content) in [
        (".agents/skills/baron-engine/SKILL.md", codex_bridge_skill()),
        (
            ".agents/skills/baron-engine/agents/openai.yaml",
            codex_openai_metadata(),
        ),
        (".codex/INDEX.md", codex_index()),
    ] {
        if let Some(path) = safe_write_codex_projection(repo, relative_path, &content)? {
            conflicts.push(path.clone());
            preserved_paths.push(path);
        }
    }
    upsert_routing_block(
        &repo.join(".codex/agents/INDEX.md"),
        &agents_index(),
        "## Custom Agents",
        "Register optional project-specific agents below without replacing the core gates.",
    )?;
    for name in ["code-reviewer", "security-auditor", "test-engineer"] {
        if let Some(path) = safe_write_codex_projection(
            repo,
            &format!(".codex/agents/{name}.toml"),
            &codex_agent_wrapper(name),
        )? {
            conflicts.push(path.clone());
            preserved_paths.push(path);
        }
    }
    install_native_hooks(&repo.join(".codex/hooks.json"), "codex")?;
    Ok(report_with_details(
        "codex",
        &[
            "AGENTS.md",
            ".agents/skills/baron-engine/SKILL.md",
            ".agents/skills/baron-engine/agents/openai.yaml",
            ".codex/INDEX.md",
            ".codex/agents/INDEX.md",
            ".codex/agents/code-reviewer.toml",
            ".codex/agents/security-auditor.toml",
            ".codex/agents/test-engineer.toml",
            ".codex/hooks.json",
        ],
        preserved_paths,
        conflicts,
    ))
}

fn install_claude(repo: &Path) -> Result<InstallReport> {
    let mut preserved_paths = Vec::new();
    let mut conflicts = Vec::new();
    upsert_managed_block(&repo.join("CLAUDE.md"), &claude_startup_contract())?;
    for (relative_path, content) in [
        (
            ".claude/skills/baron-engine/SKILL.md",
            claude_bridge_skill(),
        ),
        (
            ".claude/commands/baron-context.md",
            claude_context_command(),
        ),
        (".claude/commands/baron-status.md", claude_status_command()),
    ] {
        if let Some(path) = safe_write_adapter_projection(repo, relative_path, &content, "claude")?
        {
            conflicts.push(path.clone());
            preserved_paths.push(path);
        }
    }
    upsert_routing_block(
        &repo.join(".claude/skills/INDEX.md"),
        &skills_index(".baron/core/skills"),
        "## Custom Skills",
        "Register project-specific skills below. Custom skills must not duplicate Superpowers workflow ownership.",
    )?;
    upsert_routing_block(
        &repo.join(".claude/agents/INDEX.md"),
        &agents_index(),
        "## Custom Agents",
        "Register optional project-specific agents below without replacing the core gates.",
    )?;
    for name in ["code-reviewer", "security-auditor", "test-engineer"] {
        let relative_path = format!(".claude/agents/{name}.md");
        if let Some(path) = safe_write_adapter_projection(
            repo,
            &relative_path,
            &claude_agent_wrapper(name),
            "claude",
        )? {
            conflicts.push(path.clone());
            preserved_paths.push(path);
        }
    }
    install_native_hooks(&repo.join(".claude/settings.json"), "claude")?;
    Ok(report_with_details(
        "claude",
        &[
            "CLAUDE.md",
            ".claude/skills/baron-engine/SKILL.md",
            ".claude/agents/code-reviewer.md",
            ".claude/agents/security-auditor.md",
            ".claude/agents/test-engineer.md",
            ".claude/commands/baron-context.md",
            ".claude/commands/baron-status.md",
            ".claude/skills/INDEX.md",
            ".claude/agents/INDEX.md",
            ".claude/settings.json",
        ],
        preserved_paths,
        conflicts,
    ))
}

fn install_native_hooks(path: &Path, adapter: &str) -> Result<()> {
    let mut root = native_hooks_root(path)?;
    let root_object = root
        .as_object_mut()
        .context("Native hook configuration must be a JSON object")?;
    let hooks = root_object
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    let hooks = hooks
        .as_object_mut()
        .context("Native hook registry must be a JSON object")?;

    for (event, command, matcher) in [
        ("SessionStart", "session-start", None),
        ("UserPromptSubmit", "user-prompt-submit", None),
        ("PreCompact", "pre-compact", None),
        ("PostToolUse", "pre-compact", Some("Edit|Write|apply_patch")),
        ("Stop", "stop", None),
    ] {
        let entries = hooks.entry(event).or_insert_with(|| serde_json::json!([]));
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

fn preflight_native_hooks(repo_root: &Path, adapter: AgentAdapter) -> Result<()> {
    let path = match adapter {
        AgentAdapter::Codex => repo_root.join(".codex/hooks.json"),
        AgentAdapter::Claude => repo_root.join(".claude/settings.json"),
    };
    let _ = native_hooks_root(&path)?;
    Ok(())
}

fn native_hooks_root(path: &Path) -> Result<serde_json::Value> {
    let content = read_text(path)?.unwrap_or_else(|| "{}".to_string());
    let root: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Native hook configuration is malformed: {}", path.display()))?;
    let root_object = root
        .as_object()
        .context("Native hook configuration must be a JSON object")?;
    if let Some(hooks) = root_object.get("hooks") {
        let hooks_object = hooks
            .as_object()
            .context("Native hook registry must be a JSON object")?;
        for (event, entries) in hooks_object {
            entries
                .as_array()
                .with_context(|| format!("Native hook event must contain an array: {event}"))?;
        }
    }
    Ok(root)
}

fn claude_startup_contract() -> String {
    r#"# Baron Claude Code Contract

Baron Core is the project's state, routing, memory, continuity, and verification authority. This managed section governs Baron lifecycle/orchestration only and coexists with user, nested, plugin, and host instructions.

- The user's current explicit instructions take precedence over generic Baron guidance unless a real safety, integrity, migration, or required confirmation gate prevents execution; preserve that intent.
- For normal engineering work, construct a structured PrepareRequestV1 and silently run `baron control-plane prepare --adapter claude --json`; never interpolate arbitrary task text into a shell command. Consume the existing PreparePacketV1 and follow its work shape, risk, intent, trusted Task State, bounded route explanation, selected skills, selected agents, verification, blockers, warnings, and next action. Do not ask the user to run hidden Baron commands.
- Resume compatible Baron Task State, continuity, recovery, and its safe next action before restarting investigation. Preserve unknowns and genuine blockers instead of inventing certainty.
- Load only route-selected canonical resources from `.baron/core/skills/<skill>/` and `.baron/core/agents/<agent>`; resolve references, scripts, and assets relative to that root. Do not recursively load the full `.baron/core/**` tree, and do not create a competing workflow or keyword router.
- Use proportional lifecycle depth: read-only explanation stays read-only, focused edits use bounded verification, and durable/risky work uses the full plan, Harness, proof, trace, checkpoint, and recovery path.
- At session start silently run `baron capability check --adapter claude`, `baron runtime check --adapter claude`, `baron context --claude`, `baron continuity status`, `baron autopilot status`, `baron plan status`, and `baron harness status`; capability presence is not execution evidence.
- For architecture, dependency, impact, ownership, entrypoint, call-flow, refactor, or cross-module work, use task context first. If requested, run `baron automation code-map refresh` and `baron automation code-map query "<task>"`, then verify selected source files before edits or proof.
- Classify authority with `baron authority classify "<requested outcome>"`. For `read_only` or `ambiguous`, do not create or update plan, Harness, proof, trace, review, friction, or learning state. Classify by outcome, so `review and apply fixes` remains a change request. If identity or metadata mismatches, run `baron automation reconcile`; never repair Baron metadata by hand. Never run public `baron update`.
- Before medium/high-risk intake, read repo, Vault, current plan, Harness, continuity, and decisions before asking; ask exactly one missing high-value question at a time. Record intent with `baron harness intent`, inspect `baron harness intent-status`, and do not pass `--confirmed` until the user explicitly confirms.
- When the user explicitly expands the product to a new platform, silently run `baron init --<platform>` so Baron adds a non-destructive extension without rearranging existing code.
- Use Superpowers as the workflow core and dispatch only selected Core agents. Claude subagents return bounded findings/evidence to this parent; they must not create a competing plan, restart prepare, switch adapters, write competing memory, invoke another agent, or claim overall completion.
- Before edits, direction changes, interruptions, and final state, record `baron continuity checkpoint`. If work fails, blocks, or is interrupted, record `baron continuity recover` with cause, last successful step, evidence, affected files, blocker, safe next action, and retry conditions; preserve the failed attempt.
- Verify before claiming completion. Run required checks, record execution with `baron proof record`, record mandatory gates with `baron control-plane record-gate`, run `baron autopilot review`, and run `baron trace score`; missing proof or failed trace remains incomplete.
- Claude host auto memory is non-authoritative host-local context. Never import it into trusted Baron memory or let it override Baron trusted project state, current intent, decisions, continuity, or recovery. Preserve any explicit host auto-memory preference unless a separately proven architecture requirement changes it.
- Preserve user CLAUDE text, custom skills, user agents, commands, settings, hooks, Codex files, and unknown fields. Native hooks are optional accelerators; this contract is the correctness fallback. Read only selected skill/agent indexes and never require a normal user to operate hidden Baron CLI commands.
- Read `docs/baron/platform/PROJECT_PROFILE.md`, `docs/baron/architecture/CURRENT_ARCHITECTURE.md`, and `docs/baron/harness/DOMAIN_LANGUAGE.md` before structural work; use canonical terms only when evidence/status support them. Product Harness owns this document; Superpowers still owns workflow. Use `baron control-plane route` for explainable routing and `baron review finding`/`baron review close` only with fix evidence and verification.
"#
    .to_string()
}

fn claude_bridge_skill() -> String {
    r#"---
name: baron-engine
description: Explicit Claude bridge to the canonical Baron Core lifecycle and selected resources.
disable-model-invocation: true
---

# Baron Engine Claude Bridge

Baron Core is the semantic and orchestration authority. The root `CLAUDE.md` contract owns automatic lifecycle behavior; this skill is an explicit diagnostic/native bridge and must not create a second routing path.

For a meaningful task, construct `PrepareRequestV1` as structured data and invoke `baron control-plane prepare --adapter claude --json`; pass task text through the supported structured transport, never shell interpolation. Consume the existing `PreparePacketV1` and follow its bounded work shape, risk, intent, Task State, route explanation, selected skills, selected agents, verification, blockers, and next action.

A selected skill identifier maps to `.baron/core/skills/<name>/SKILL.md`. Load only that canonical root and resolve its `references/`, `scripts/`, `assets/`, and nested files relative to it. A selected agent identifier maps to `.baron/core/agents/<name>` and remains subject to the parent session's lifecycle and proof gates.

Do not preload or recursively read all of `.baron/core/**`; this bridge contains no copied Baron skill body and no independent keyword router. Preserve explicit user intent and surface safety, integrity, identity, or migration blockers. If Core or the Baron runtime is unavailable, stop and report the blocker instead of guessing.
"#
    .to_string()
}

fn codex_startup_contract() -> String {
    r#"# Baron Codex Contract

Baron Core is the project state, routing, memory, and verification authority.

- The user's current explicit instructions take precedence over generic Baron skill guidance unless a required safety or integrity constraint blocks the request; preserve that intent.
- For a normal engineering task, construct a structured PrepareRequestV1 and silently run `baron control-plane prepare --adapter codex --json`; never interpolate arbitrary task text into a shell command. Consume the existing PreparePacketV1 and follow its work shape, risk, intent, Task State, bounded route explanation (including why skills, agents, or profile evidence influenced selection), selected skills, selected agents, verification, blockers, and next action. Do not ask the user to run hidden Baron commands.
- Resume compatible Task State, continuity, recovery, and safe next action before restarting investigation. Preserve unknowns and genuine blockers.
- Load only selected canonical resources from `.baron/core/skills/<skill>/` and `.baron/core/agents/<agent>`; resolve references, scripts, and assets relative to that root. Do not recursively load the full `.baron/core/**` tree and do not create a competing workflow.
- Use proportional lifecycle depth: read-only explanation may remain read-only, focused edits use bounded verification, and durable/risky work uses the full Baron plan, Harness, proof, trace, and recovery path.
- At session start silently run `baron capability check --adapter codex`, `baron runtime check --adapter codex`, `baron context --codex`, `baron continuity status`, `baron autopilot status`, `baron plan status`, and `baron harness status`; presence is not execution evidence.
- For architecture, dependency, impact, ownership, entrypoint, call-flow, refactor, or cross-module work, use task context first; when requested, run `baron automation code-map refresh` and `baron automation code-map query "<task>"`, then verify selected source files.
- Classify authority with `baron authority classify "<requested outcome>"`. For `read_only` or `ambiguous`, do not create or update plan, Harness, proof, trace, review, friction, or learning state. Classify by outcome, so `review and apply fixes` remains a change request. If identity or metadata mismatches, run `baron automation reconcile`; never repair Baron metadata by hand. Never run public `baron update`.
- Before medium/high-risk intake, read repo, Vault, current plan, Harness, continuity, and decisions before asking; ask exactly one missing high-value question at a time. Record intent with `baron harness intent`, inspect `baron harness intent-status`, and do not pass `--confirmed` until the user explicitly confirms.
- Use Superpowers as the workflow core. Dispatch only selected Core agents. Child work is bounded evidence returned to the parent: it must not start another lifecycle, overwrite parent intent, create a competing plan, switch adapters, or claim completion.
- Checkpoint meaningful edits, direction changes, interruptions, and final state. On failure record `baron continuity recover`, preserve the failed attempt, and include the last successful step, evidence, affected files, blocker, safe next action, and retry conditions.
- Verify before claiming completion. Run required checks, record execution with `baron proof record`, record mandatory gates with `baron control-plane record-gate`, and run `baron autopilot review`; proof and trace failures remain blockers. Do not fabricate unknowns or success.
- Preserve user files, custom skills/agents, hooks, settings, nested instructions, and Claude-owned files. Hooks are optional accelerators; the same idempotent instruction protocol is the fallback.
- Read `docs/baron/platform/PROJECT_PROFILE.md`, `docs/baron/architecture/CURRENT_ARCHITECTURE.md`, and `docs/baron/harness/DOMAIN_LANGUAGE.md` before structural work; use canonical terms from the domain language. Product Harness owns this document; Superpowers still owns workflow. Use `baron control-plane route` for explainable routing and `baron init --<platform>` only when the user explicitly expands the product. For findings use `baron review finding`, then `baron review close` only with fix evidence and verification. Run `baron trace score` before claiming completion.
"#
    .to_string()
}

fn codex_bridge_skill() -> String {
    r#"---
name: baron-engine
description: Native Codex bridge to the canonical Baron Core lifecycle and selected resources.
---

# Baron Engine Codex Bridge

Baron Core is the project orchestration authority and semantic source of truth. The root `AGENTS.md` contract owns automatic lifecycle behavior.

For a meaningful user task, construct `PrepareRequestV1` as structured data and invoke `baron control-plane prepare --adapter codex --json`; pass task text through the supported structured transport, never through shell interpolation. Consume the existing `PreparePacketV1` and follow its work shape, risk, intent, Task State, bounded route explanation, selected skills, selected agents, verification, blockers, and next action.

A selected skill identifier maps to `.baron/core/skills/<name>/SKILL.md`. Load only the selected skill entrypoint and resolve its `references/`, `scripts/`, `assets/`, and nested files relative to `.baron/core/skills/<name>/`. A selected agent identifier maps to `.baron/core/agents/<name>` and remains subject to the parent session's lifecycle and proof gates.

Do not preload or recursively read all of `.baron/core/**`; this bridge contains no copied Baron skill body. Preserve explicit user intent and surface safety, integrity, identity, or migration blockers clearly. If Core or the Baron runtime is unavailable, stop and report the blocker instead of guessing.
"#
    .to_string()
}

fn codex_openai_metadata() -> String {
    "name: baron-engine\ndescription: Explicit Codex bridge to Baron Core; AGENTS.md owns automatic routing.\npolicy:\n  allow_implicit_invocation: false\n  products:\n    - codex\n"
        .to_string()
}

fn codex_agent_payload(name: &str) -> ManagedAssetPayload {
    payload(
        "codex",
        &format!(".codex/agents/{name}.toml"),
        ManagedMergeKind::FullText,
        &codex_agent_wrapper(name),
    )
}

fn codex_agent_wrapper(name: &str) -> String {
    format!(
        "name = \"{name}\"\ndescription = \"Baron Core {name} quality-gate projection for Codex.\"\ndeveloper_instructions = \"\"\"\nCanonical semantic source: `.baron/core/agents/{name}.toml`.\n\nUse the Core definition and bounded Baron evidence for the `{name}` role. Return concise findings, proof, verification, and remaining uncertainty to the parent session. Do not plan, implement, switch adapters, invoke other subagents, or change lifecycle ownership. The parent owns intent, checkpoints, completion, and recovery.\n\"\"\"\n"
    )
}

fn claude_agent_payload(name: &str) -> ManagedAssetPayload {
    payload(
        "claude",
        &format!(".claude/agents/{name}.md"),
        ManagedMergeKind::FullText,
        &claude_agent_wrapper(name),
    )
}

fn claude_agent_wrapper(name: &str) -> String {
    format!(
        "---\nname: {name}\ndescription: Baron Core {name} quality-gate projection for Claude.\n---\n\nCanonical semantic source: `.baron/core/agents/{name}.toml`.\n\nUse the Core definition and return bounded findings, evidence, verification, and uncertainty to the parent session. This child must not create a plan, switch adapters, restart prepare, write competing memory, invoke another agent, or claim overall completion. The parent session owns intent, checkpoints, lifecycle, proof, and recovery.\n"
    )
}

fn safe_write_codex_projection(
    repo: &Path,
    relative_path: &str,
    content: &str,
) -> Result<Option<String>> {
    safe_write_adapter_projection(repo, relative_path, content, "codex")
}

fn safe_write_adapter_projection(
    repo: &Path,
    relative_path: &str,
    content: &str,
    adapter: &str,
) -> Result<Option<String>> {
    let path = repo.join(relative_path);
    let existing = read_bytes(&path)?;
    if existing.as_deref() == Some(content.as_bytes()) {
        return Ok(None);
    }
    if let Some(existing) = existing {
        let unchanged_owned = if managed_manifest_exists(repo)? {
            load_managed_baseline(repo)
                .ok()
                .and_then(|baseline| {
                    baseline
                        .records
                        .into_iter()
                        .find(|record| record.relative_path == Path::new(relative_path))
                })
                .and_then(|record| {
                    managed_baseline_content(repo, &record)
                        .ok()
                        .map(|base| (record, base))
                })
                .is_some_and(|(record, base)| {
                    record.owner.as_str() == adapter && existing == base.as_bytes()
                })
        } else {
            false
        };
        if !unchanged_owned {
            return Ok(Some(relative_path.replace('\\', "/")));
        }
    }
    write_managed_file(&path, content)?;
    Ok(None)
}

fn codex_index() -> String {
    "# Baron Codex Workspace\n\n\
Start with root `AGENTS.md`. Baron Core lives at `.baron/core/**`; the native bridge is `.agents/skills/baron-engine/`; Codex quality-agent projections live under `.codex/agents/`. Do not recursively load every skill; read only the route-selected Core resources. Core routing covers Superpowers, `frontend-design`, `vibe-security-scan`, `api-and-interface-design`, `observability-and-instrumentation`, `performance-optimization`, and `deprecation-and-migration`; these names identify canonical Core resources rather than copied Codex files. `.codex/hooks.json` is an optional accelerator and never the sole correctness path.\n"
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
  | `vibe-security-scan` | optional defensive security domain | auth, API, secrets, RLS, uploads, payment, dependencies, permissions | visual-only or copy-only tasks | severity, evidence, fix, verification | must not replace `security-auditor` final gate |\n\
  | `binary-reverse-analysis` | optional defensive reverse domain | binary triage, disassembly, static reverse engineering | live exploitation, persistence, evasion, CTF/pwn | scope, artifact hash, tool evidence, uncertainty | must not install tools or replace security-auditor |\n\
  | `apk-mobile-analysis` | optional defensive mobile domain | APK/Android manifest, permissions, static mobile review | credential theft, live exploitation, auto-bootstrap | artifact hash, static evidence, verification | must not replace vibe-security-scan or security-auditor |\n\
  | `malware-triage` | optional defensive malware domain | offline sample triage, indicators, containment notes | payload execution, persistence, evasion, live delivery | hash, isolated evidence, safe handling | must not execute samples or replace security-auditor |\n\n\
| `api-and-interface-design` | optional API/interface domain | API contracts, request/response shape, SDK/public interface, compatibility, or deep module boundary | implementation-only tasks that do not change boundaries | contract risks, versioning impact, boundary evidence, verification | must not replace Superpowers planning or tests |\n\
| `observability-and-instrumentation` | optional operations domain | logs, metrics, tracing, alerts, SLOs, audit events, diagnostics | tasks with no runtime/operations impact | signal list, gaps, proof hooks | must not fabricate production behavior |\n\
| `performance-optimization` | optional performance domain | latency, runtime speed, bundle size, cache, loading, database/query performance | cosmetic-only or security-only tasks | measured or potential impact, verification | must not fabricate metrics |\n\
| `deprecation-and-migration` | optional migration domain | legacy behavior, migrations, deprecations, compatibility, rollout/rollback | greenfield work with no compatibility risk | migration plan, compatibility proof, rollback | must not bypass proof gates |\n\n\
| `database-engineering` | optional database domain | relational models, constraints, indexes, query plans, transactions, migrations, backfills, integrity | pipeline-only, frontend-only, incidental SQL | schema/query/migration evidence, integrity and rollback checks | conflicts with mobile/reverse-analysis domains; must not replace Superpowers |\n\
| `mobile-application-engineering` | optional mobile application domain | Android/iOS app lifecycle, navigation, offline, permissions, storage, device behavior | APK/binary reverse, decompile, malware analysis | lifecycle/device/network evidence, platform build checks | conflicts with APK/binary/malware analysis; must not replace Superpowers |\n\n\
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
        core: CoreInstallReport::default(),
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
