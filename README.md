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

Current phase: `0 - foundation skeleton`.

This repository intentionally starts with a product spec, roadmap, architecture
skeleton, and tiny compiling Rust workspace. The full engine is not implemented
yet. The goal of phase 0 is to make the direction impossible to lose.

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
baron survey
baron init --codex
baron init --claude
baron init --agent
baron context --codex
baron context --claude
baron context --agent
baron recall "<query>"
baron memory status
baron plan status
baron harness status
baron trace score
```

These commands are a contract for the roadmap, not all implemented behavior in
phase 0.

## Source Of Truth

- Vault Markdown is the durable source of truth.
- SQLite/cache/index files are accelerators only.
- Rust is the main engine runtime.
- Agent-specific files are adapters, not separate brains.

## Temporary Build Notes

`notes/build-log/` is a temporary working memory folder for building Baron. It is
safe to delete after Baron reaches a mature release, because the durable product
spec, roadmap, and architecture docs live under `docs/`.

For status, read `docs/BARON_STATUS.md` first. For interrupted work, read
`notes/build-log/CURRENT.md` next.
