# Current Build Note

Date: 2026-06-14

## Current Phase

Phase 3 - Context Compiler.

## What Is Being Built

- bounded context compilation from repo survey and Vault memory
- Codex, Claude, and generic-agent context targets
- task-aware low, medium, high, and unknown risk guidance
- current execution-state excerpt
- explicit skipped-context explanation
- `context --why` selection diagnostics

## Current Status

Implemented and verified. Commit pending.

## Verification

- `cargo fmt --all`: passed
- `cargo test`: passed
- context compiler core tests: passed
- context compiler CLI tests: passed
- `baron context . --codex --task "implement auth login" --vault <temp-vault>`: passed
- `baron context . --claude --vault <temp-vault>`: passed
- `baron context . --agent --vault <temp-vault>`: passed
- `baron context . --why --vault <temp-vault>`: passed
- `docs/BARON_STATUS.json` parses as JSON: passed
- `git diff --check`: passed

## Next Action

Begin Phase 4 - Agent Adapters.

## Phase 3 Commit

- Commit: pending

## Phase 2 Commit

- Commit: `c6fc469 feat: add vault memory firewall`

## Phase 1 Commit

- Commit message: `feat: add Baron survey engine`
- Use `git log -1 --oneline` after commit for the exact hash.

## Phase 0 Commit

- Commit message: `chore: create Baron phase 0 foundation`
- Use `git log -1 --oneline` for the exact hash.

## Do Not Forget

- Superpowers remains the workflow core.
- Core agents are `code-reviewer`, `security-auditor`, `test-engineer`.
- Optional bundled domain skills are `frontend-design` and `vibe-security-scan`.
- Baron must support Codex, Claude, and generic agents through adapters.
- Shadow-first onboarding is mandatory for old repos.
