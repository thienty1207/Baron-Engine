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

## Phase

Current phase: `6 - Native Migration And Legacy Retirement design pending`.

This repository intentionally starts with a product spec, roadmap, architecture
skeleton, and a Rust workspace. Survey, Vault Memory Firewall, Context Compiler,
multi-agent adapters, Active Plan State, Product Harness, proof gates, and trace
quality are implemented. Next, Baron will convert useful legacy project data
into Baron-native structures and retire Agent Bootstrap managed assets. Phase 7
adds the Baron Capability Registry. Phase 8 is release hardening.

Progress is tracked in `docs/BARON_STATUS.md`. Machine-readable progress is in
`docs/BARON_STATUS.json`.

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

## Initial Commands

Target command surface:

```bash
baron survey [repo-path]
baron survey [repo-path] --json
baron init [repo-path] --codex --shadow
baron init [repo-path] --claude --shadow
baron init [repo-path] --agent --shadow
baron memory status [repo-path] --vault <vault-path>
baron memory index [repo-path] --vault <vault-path>
baron memory compact [repo-path] --vault <vault-path>
baron recall "<query>" [repo-path] --vault <vault-path>
baron context [repo-path] --codex --vault <vault-path>
baron context [repo-path] --claude --vault <vault-path>
baron context [repo-path] --agent --vault <vault-path>
baron context [repo-path] --codex --task "<task>" --vault <vault-path>
baron context [repo-path] --why --vault <vault-path>
baron update [repo-path]
baron plan status
baron plan start "<title>"
baron plan update "<note>"
baron plan interrupt "<state>"
baron plan complete "<verification>"
baron harness status
baron harness intake "<title>"
baron harness decision "<summary>"
baron harness friction "<summary>"
baron proof status
baron proof record "<verification>"
baron trace record "<summary>" --outcome completed
baron trace score
```

`survey`, `init --shadow`, `memory status`, `memory index`, `memory compact`,
`recall`, `context`, adapter `init/update`, `plan`, `harness`, `proof`, and
`trace` are implemented. Migration and release commands remain roadmap
contracts until their phases are completed.

Memory and context commands require `--vault <path>` or `BARON_VAULT`. Baron
does not guess where memory should live.

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
custom skills, and custom agents survive updates.

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

## Source Of Truth

- Vault Markdown is the durable source of truth.
- SQLite/cache/index files are rebuildable accelerators only.
- Rust is the main engine runtime.
- Agent-specific files are adapters, not separate brains.

## Temporary Build Notes

`notes/build-log/` is a temporary working memory folder for building Baron. It is
safe to delete after Baron reaches a mature release, because the durable product
spec, roadmap, and architecture docs live under `docs/`.

For status, read `docs/BARON_STATUS.md` first. For interrupted work, read
`notes/build-log/CURRENT.md` next.
