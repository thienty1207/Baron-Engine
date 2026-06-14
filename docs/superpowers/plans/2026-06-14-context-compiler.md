# Phase 3 - Context Compiler

Date: 2026-06-14
Status: completed
Feature commit: `c192baf feat: add Baron context compiler`

## Goal

Make Baron compile the right bounded context for Codex, Claude, and generic
agents without writing adapter files into the target repository.

## Delivered

- `baron context [repo-path] --codex --vault <vault-path>`
- `baron context [repo-path] --claude --vault <vault-path>`
- `baron context [repo-path] --agent --vault <vault-path>`
- `baron context [repo-path] --why --vault <vault-path>`
- optional `--task "<task>"` risk guidance
- Project Atlas, execution state, and Memory Firewall Brief composition
- 20,000-character output boundary
- explicit loaded/skipped context reasoning
- core and CLI integration tests

## Decisions

- Context remains stdout-only in Phase 3.
- Adapter file generation stays in Phase 4.
- `--why` may default to generic-agent selection reasoning.
- Missing Vault configuration is an error; Baron does not guess a Vault path.
- Unknown facts remain visible as unknown.

## Verification

- `cargo fmt --all`
- `cargo test`
- Codex, Claude, generic-agent, task-risk, and `--why` smoke commands
- `docs/BARON_STATUS.json` parse
- `git diff --check`

## Next

Phase 4 - Agent Adapters.
