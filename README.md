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
- adapter-specific output for Codex, Claude, Cursor, and generic agents

## Phase

Current phase: `3 - Context Compiler completed`.

This repository intentionally starts with a product spec, roadmap, architecture
skeleton, and a Rust workspace. The read-only Survey Engine, Vault Memory
Firewall, and bounded Context Compiler are implemented. Agent Adapters are the
next major phase.

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
baron plan status
baron harness status
baron trace score
```

`survey`, `init --shadow`, `memory status`, `memory index`, `memory compact`,
`recall`, and `context` are implemented. Later commands remain roadmap
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
and managed-file updates remain Phase 4 work.

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
