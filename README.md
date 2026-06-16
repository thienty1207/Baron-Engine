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

Current version: `2.0.0`.

Survey, Vault Memory Firewall, Context Compiler, multi-agent adapters, Active
Plan State, Product Harness, proof gates, trace quality, and transactional
legacy migration, the Baron Capability Registry, and the native release
lifecycle are implemented. Baron 2.0 adds observable automation, stable project
identity, massive-memory indexing, multilingual recall, automatic session
import, strict skill/agent control-plane routing, self-improving harness audits,
and certification gates.

Progress is tracked in `docs/BARON_STATUS.md`. Machine-readable progress is in
`docs/BARON_STATUS.json`.

Baron 2.0 is the long-horizon version of the engine. It is built for old
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

## Normal User Flow

Most users should not copy a giant command list. Use this flow instead.

### 1. Install Baron

Install once, then confirm the binary is available:

```bash
baron --version
```

### 2. Choose The Vault

The Vault is Baron's long-term memory folder. Pick one durable folder that can be
shared by many projects:

```powershell
$env:BARON_VAULT = "D:\work\AgentMemory"
```

You can also pass it directly:

```bash
baron memory status <project-path> --vault <vault-path>
```

Baron never guesses a Vault path. Memory and context commands require
`--vault <path>`, `BARON_VAULT`, or an initialized `.baron/local.toml`.

### 3. Inspect A Repo Before Writing

Use survey first when you want Baron to read the project and explain what it
sees:

```bash
baron survey <project-path>
baron survey <project-path> --json
```

Preview what an adapter would install without writing files:

```bash
baron init <project-path> --codex --shadow
baron init <project-path> --claude --shadow
baron init <project-path> --agent --shadow
```

Baron detects project shape from the repo itself: web, mobile, backend, tool,
fullstack, desktop, or unknown. There is no separate `--web` or `--mobile` flag
because the Survey Engine derives that from files such as `package.json`,
`Cargo.toml`, `go.mod`, mobile folders, build configs, and entrypoints.

### 4. Initialize A Project

Choose the agent surface the project will use:

```bash
baron init <project-path> --codex --vault <vault-path>
baron init <project-path> --claude --vault <vault-path>
baron init <project-path> --agent --vault <vault-path>
```

Repeat `init` with another adapter when the same project should support more
than one agent tool. Baron preserves user text, custom skills, custom agents,
custom routing entries, and non-Baron hooks.

### 5. Update An Existing Baron Project

Refresh managed Baron files after installing a newer Baron binary:

```bash
baron update <project-path>
```

Refresh only one registered adapter when needed:

```bash
baron update <project-path> --codex
baron update <project-path> --claude
baron update <project-path> --agent
```

### 6. Manual Inspection When Needed

These commands are useful when you want to inspect the system yourself:

```bash
baron context <project-path> --codex --task "<task>" --vault <vault-path>
baron recall "<query>" <project-path> --vault <vault-path>
baron memory compact <project-path> --vault <vault-path>
baron memory import-sessions <project-path> --vault <vault-path>
baron certify run <project-path> --vault <vault-path> --profile smoke
baron certify status <project-path>
```

Plan, harness, proof, trace, capability, control-plane, and automation commands
exist for the agent runtime and for diagnostics. Normal users do not need to run
them by hand during daily work; installed adapters and hooks guide the AI to run
them at the right time.

## Capability Registry

Baron can use optional project tools without becoming dependent on them. A
project registers the ability it needs, such as `security-scan`,
`impact-analysis`, or `deploy-verification`, and then lists one or more
providers for that ability.

Baron keeps three facts separate:

- registered: the project intends to use this provider
- present: the current machine appears equipped to use it
- executed: the provider actually ran for this task and produced recorded
  evidence

This prevents a common AI mistake: seeing that a tool is installed and then
claiming its check passed without running it.

Provider definitions live in `.baron/capabilities.toml`. Current-machine checks
live in the rebuildable `.baron/cache/capability-state.json`. CLI, binary, MCP,
skill, HTTP, and agent-adapter providers are supported.

Agents automatically run `baron capability check` before compact context.
Missing optional tools produce a visible fallback warning. Missing required
tools, or required tools without execution evidence, make Proof insufficient
and prevent Trace from passing.

## Context Compiler

The Context Compiler turns the Survey Engine and Memory Firewall into one
bounded brief for the active agent tool. It prints to stdout and does not
install or overwrite agent files.

It includes:

- adapter-specific guidance for Codex, Claude, or a generic agent
- the current Project Atlas
- detected commands, entrypoints, risky surfaces, and unknowns
- a bounded current execution-state excerpt when one exists
- current-project memory plus relevant approved global memory
- a visible list of context intentionally skipped

Use `--task "<task>"` when the caller knows the current task. Baron then adds a
low, medium, or high risk lane and proportional verification guidance. Use
`--why` to inspect why context was loaded or skipped. Adapter file generation
and managed-file updates are handled by `baron init` and `baron update`.

## Automatic Project Configuration

Baron writes two small configuration files:

- `.baron/project.toml` is safe to commit. It stores project identity,
  registered adapters, and automatic behavior switches.
- `.baron/local.toml` stores the Vault path for the current machine and is
  ignored by `.baron/.gitignore`.

These files are maps, not memory. Facts, decisions, plans, proof, traces, and
session history remain Markdown in the repo and Vault. After initialization,
Baron commands work from nested folders without repeating `--vault`.

Vault resolution order is explicit `--vault`, then `BARON_VAULT`, then
`.baron/local.toml`.

## Multi-Agent Adapters

Run `baron init --codex`, `--claude`, or `--agent` with a Vault path. Repeating
init registers another adapter for the same project. `baron update` refreshes
all registered adapters.

- Codex receives `AGENTS.md`, `.codex/`, Superpowers, optional domain skills,
  and the three core quality agents.
- Claude receives `CLAUDE.md`, Claude commands, skills, and quality agents.
- Generic agents receive `AGENT.md`, portable context files, and `.baron/core`.

Managed root instructions use Baron markers. User text outside the markers,
custom skills, custom agents, custom routing entries, and non-Baron native
hooks survive updates.

## Observable Automatic Workflow

Baron does not rely only on an instruction telling the AI to remember the
workflow.

- Codex receives project hooks in `.codex/hooks.json`.
- Claude receives project hooks in `.claude/settings.json`.
- Session start injects bounded Baron context automatically.
- Prompt and edit checkpoints are written to a small automation journal.
- Stop reconciliation checks active plan, proof, and trace state.
- If meaningful work is unfinished, Baron asks the agent to continue or record
  an interruption instead of silently claiming completion.

Project hooks still require the agent tool to trust the project configuration.
`baron automation status` shows what actually ran. Generic agents keep the
managed startup contract and can use `baron automation reconcile` when their
host has no native hook standard.

## Skill And Agent Control Plane

Baron does not let the agent recursively scan every skill or invent a routing
rule from vibes.

- `baron control-plane route "<task>"` explains which skills and quality gates
  match the task.
- Superpowers remains the only workflow core.
- `frontend-design` and `vibe-security-scan` remain optional domain skills.
- `code-reviewer`, `security-auditor`, and `test-engineer` remain quality
  gates, not planners or routers.
- `baron control-plane record-gate` records evidence that a mandatory gate
  actually ran.

If a custom skill or agent tries to claim workflow ownership, duplicates another
asset, or asks subagents to orchestrate each other recursively, Baron reports a
control-plane diagnostic instead of silently trusting it.

## Self-Improving Harness

Baron now measures workflow friction without silently rewriting its own core
rules.

- `baron harness audit` checks context-read evidence, proof gaps, trace gaps,
  open friction, and documentation drift.
- `baron harness verify-all` scans the validation matrix for pending or weak
  proof in bounded batches.
- `baron harness intervention` records human, reviewer, CI, or agent
  corrections.
- `baron harness propose` groups repeated friction into improvement proposals.
- `baron harness outcome` records whether a proposal actually helped.

Core policy and architecture changes still require human approval. Baron may
propose improvements automatically; it does not rewrite the rules by itself.

## Certification Gate

`baron certify run` is Baron's release-confidence check. It is not another
memory source and it does not replace tests. It asks whether the repo and Vault
are still healthy under pressure:

- repository survey stays bounded
- SQLite memory cache can be rebuilt from Vault Markdown
- shared-Vault memory firewall keeps the current project isolated
- compact memory context stays small
- Baron project automation is configured
- release readiness has no open certification blocker

The report is written under `docs/baron/certification/` and mirrored into the
project Vault capsule. `baron certify status` reads the latest report so future
agents can see whether the last certification passed.

## Large And Meaning-Aware Memory

Each project receives a stored identity and a unique Vault capsule such as
`tomoty--<project-id>`. Two folders named `tomoty` can no longer share memory by
accident, and moving an initialized repo keeps the same identity.

The SQLite index is incremental and rebuildable:

- unchanged Markdown is reused
- changed Markdown is refreshed
- deleted Markdown is removed from the index
- repository and memory scans no longer stop at hidden fixed file counts

Recall combines exact words, engineering concepts, evidence quality, recency,
memory kind, project identity, and the memory firewall. Common Vietnamese and
English meanings are bridged, for example a request about `bảo mật dữ liệu
khách hàng` can retrieve a verified note about `Supabase RLS tenant isolation`.
`baron context --task "<task>"` uses the same ranking to load only the most
useful memory.

## Automatic Session Memory

For initialized projects, normal context startup checks a bounded number of
recent Codex and Claude session files. Baron imports only sessions that contain
an exact match to the current repository path. It keeps useful user messages
and assistant decisions, removes tool/system noise, redacts obvious secrets,
deduplicates imports, and writes clean Markdown under
`Sessions/Imported/` in the project Vault capsule.

Users normally do not run import manually. `baron memory import-sessions`
exists for inspection or recovery. Source overrides are available through
`BARON_CODEX_SESSIONS_ROOT` and `BARON_CLAUDE_SESSIONS_ROOT`.

## Execution And Completion Gates

Baron keeps current execution state under `docs/baron/` and mirrors it into the
project capsule in the Vault.

- Plan answers what is active and where work stopped.
- Product Harness answers what the feature must achieve and how risky it is.
- Validation Matrix connects each Product Harness story to its latest proof and
  labels weak evidence as insufficient instead of verified.
- Proof records verification that actually ran.
- Trace records what happened, files changed, and evidence quality.

Low-risk work requires minimal trace quality. Medium-risk work requires
standard quality. Auth, permissions, tenant/RLS, payment, migration, security,
secrets, uploads, external providers, and destructive-data work require
detailed traces plus explicit security/data-impact proof. `baron plan complete`
refuses completion when these gates are missing. `baron trace score` also
returns a failing process status when the required tier is not met, so an agent
cannot silently continue past a weak trace.
Baron's own generated state and adapter files do not count as product-file
changes for detailed traces.

## Native Legacy Migration

Baron can take over a project previously managed by Agent Bootstrap without
keeping the old runtime alive.

Start with:

```bash
baron migrate agent-bootstrap <repo-path> --dry-run
```

Dry-run only shows what Baron would import, preserve, quarantine, replace, or
remove. It writes nothing.

The apply command performs the same inventory again, creates a rollback backup
inside `Vault/Artifacts/Baron/Migrations/`, imports useful memory and execution
history, validates custom skills and agents, installs fresh Baron assets, and
only then retires verified legacy files.

```bash
baron migrate agent-bootstrap <repo-path>
```

Custom assets that are weak, conflicting, or still depend on Agent Bootstrap
are preserved under `.baron/quarantine/<migration-id>/`; they are not silently
activated or deleted. If installation or verification fails, Baron rolls the
transaction back automatically.

Use `baron migrate status` to inspect the latest result. A manual rollback is
also available through the migration id printed by the apply command. The old
source Vault is never deleted.

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
- `baron certify run` must pass before a release claim is trusted.

## Temporary Build Notes

`notes/build-log/` is a temporary working memory folder for building Baron. It is
safe to delete after Baron reaches a mature release, because the durable product
spec, roadmap, and architecture docs live under `docs/`.

For status, read `docs/BARON_STATUS.md` first. For interrupted work, read
`notes/build-log/CURRENT.md` next.
