# Baron Engine

Baron is a Rust-first multi-agent memory operating system for coding agents.

It is not a rewrite of `agent-bootstrap`, and it is not a clone of
`repository-harness`. Baron keeps the best ideas from both, then moves them into
one durable engine:

- repo understanding
- vault-backed long-term memory
- memory firewall for many projects
- context compiler for each agent tool
- active plan state
- product harness
- proof and trace quality gates
- capability-aware tool routing and evidence
- adapter-specific output for Codex, Claude, Cursor, and generic agents

## Stable Release

Current version: `2.1.0`.

Survey, Vault Memory Firewall, Context Compiler, multi-agent adapters, Active
Plan State, Product Harness, proof gates, trace quality, transactional legacy
migration, capability routing, release lifecycle, and the simple user setup flow
are implemented.

Progress is tracked in `docs/BARON_STATUS.md`. Machine-readable progress is in
`docs/BARON_STATUS.json`.

Baron 2.x is the long-horizon version of the engine. It is built for old
projects, many projects sharing one Vault, direct IDE/agent use, and strict
completion claims backed by proof.

## Install

Windows PowerShell:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
& $installer
```

Linux or macOS:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh | sh
```

The installer verifies the downloaded archive against `SHA256SUMS` before
replacing any executable. Update keeps a rollback binary. Uninstall never
deletes project files or Vault memory. See [docs/RELEASE.md](docs/RELEASE.md)
for update, rollback, uninstall, offline install, and manual checksum steps.

## Core Promise

Baron should help an AI agent answer these questions before it edits code:

- What project am I in?
- What should I read first?
- What is the current task and active plan?
- Which memory is verified, stale, cross-project, or unknown?
- How risky is this task?
- What proof is required before claiming completion?
- Which trace should be left for the next agent session?
- Which adapter format does this agent tool need?
- Which registered capability providers are present and compatible?
- Which tool-backed claims have real execution evidence?

## Quick Start

Normal users only need these steps.

### 1. Install

```bash
baron --version
```

### 2. Set Up The Vault

Stand inside the folder you want to use as Baron's long-term memory Vault:

```powershell
cd D:\work\AgentMemory
baron setup --vault
```

Or pass the Vault path directly:

```powershell
baron setup --vault "D:\work\AgentMemory"
```

This writes the machine default Vault to `~/.baron/config.toml`, so future
projects do not need `--vault` every time.

### 3. Initialize The Agent Tool

Stand inside the project:

```powershell
cd D:\work\IT\Web\TomoTy
```

Choose the agent surface:

```bash
baron init --codex
baron init --claude
baron init --agent
```

You can register more than one adapter in the same project by repeating `init`.

### 4. Choose The Platform Focus

Platform focus tells Baron what kind of knowledge the AI should prioritize. It
does not replace Superpowers, skills, agents, proof, or repository evidence.

```bash
baron init --frontend
baron init --backend
baron init --fullstack
baron init --mobile
baron init --desktop
baron init --tool
baron init --library
baron init --data
baron init --cloud
```

Fast path:

```bash
baron init --codex --fullstack
baron init --claude --backend
baron init --agent --tool
```

### 5. Update Later

```bash
baron update
```

## What The AI Runs Automatically

After init, the user should not need to manually run survey, context, memory,
plan, harness, proof, trace, control-plane, or automation commands during normal
work. Baron installs adapter instructions and native hooks where supported, so
AI agents load context, route skills, check memory, record proof, score traces,
and preserve execution state when the task requires it.

The full advanced command surface is documented in
[docs/architecture/COMMAND_SURFACE.md](docs/architecture/COMMAND_SURFACE.md).

## How Baron Stays Smart Under The Hood

Baron keeps the visible user flow small, but the engine underneath is still large. After init, the adapter instructions and supported hooks tell the AI to do the background work automatically when the task needs it.

The hidden engine handles:

- reading the repository shape before editing
- loading a bounded context brief instead of flooding the agent
- keeping project memory separate inside a shared Vault
- importing useful agent session history with redaction and deduplication
- tracking the active plan, product intent, proof, traces, and friction
- routing Superpowers, optional domain skills, and the three quality agents through a strict control plane
- checking whether tool-backed claims have real execution evidence

This is why normal users should not need to learn the internal command list. The advanced command surface exists for AI automation, recovery, diagnostics, and maintainers, and is documented in [docs/architecture/COMMAND_SURFACE.md](docs/architecture/COMMAND_SURFACE.md).

## Automatic Project Configuration

Baron writes small map files under `.baron/` so the project remembers its identity, selected adapters, selected platform focus, and machine Vault path. They are not the memory itself.

Memory, decisions, plans, proof, traces, and session history remain Markdown in the repository and the Vault. SQLite/cache files are rebuildable accelerators.

## Agent Surfaces

`baron init` installs the right surface for the agent tool you choose:

- Codex receives `AGENTS.md`, `.codex/`, Superpowers, optional domain skills, and the three core quality agents.
- Claude receives `CLAUDE.md`, Claude commands, skills, and quality agents.
- Generic agents receive `AGENT.md`, portable context files, and `.baron/core`.

Managed root instructions use Baron markers. User text outside the markers, custom skills, custom agents, custom routing entries, and non-Baron native hooks survive updates.

## Memory, Proof, And Safety

Baron is designed for many old and new projects sharing one Vault without turning memory into soup.

- Current-project memory is preferred.
- Approved global memory can help across projects.
- Cross-project memory is blocked unless the task clearly asks for it.
- Draft, stale, interrupted, or imported-session-only memory is treated with lower confidence.
- Medium and high-risk work needs real proof before completion is trusted.
- Trace quality helps the next agent understand what happened and what remains.

The user normally sees none of this as command work. The AI uses it as the background discipline that keeps long projects from drifting.

## Legacy Migration

Baron can take over projects that used Agent Bootstrap, but migration is an advanced maintenance flow. It inventories the old project, imports useful memory and execution history, validates custom skills and agents, quarantines weak or conflicting assets, and keeps rollback data in the Vault. The old source Vault is never deleted automatically.
## Source Of Truth

- Vault Markdown is the durable source of truth.
- SQLite/cache/index files are rebuildable accelerators only.
- Rust is the main engine runtime.
- Agent-specific files are adapters, not separate brains.

## Release Safety

- The Cargo workspace version is the only Baron release version.
- Windows, Linux, Intel macOS, and Apple Silicon macOS build on native runners.
- Release archives use stable target-specific names.
- `SHA256SUMS` and `release-manifest.json` cover every native archive.
- Install and update verify checksum and binary version before replacement.
- Rollback affects only the Baron executable.
- Uninstall leaves repositories, adapters, `.baron/`, and Vault Markdown intact.
- A Baron certification run must pass before a release claim is trusted.

## Temporary Build Notes

`notes/build-log/` is a temporary working memory folder for building Baron. It is
safe to delete after Baron reaches a mature release, because the durable product
spec, roadmap, and architecture docs live under `docs/`.

For status, read `docs/BARON_STATUS.md` first. For interrupted work, read
`notes/build-log/CURRENT.md` next.
