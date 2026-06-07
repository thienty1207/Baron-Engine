# Current Build Note

Date: 2026-06-08

## Current Phase

Phase 1 - Survey Engine.

## What Is Being Built

- read-only repo survey
- Project Atlas Markdown to stdout
- Project Atlas JSON to stdout
- no-write `init --codex --shadow`
- no-write `init --claude --shadow`
- no-write `init --agent --shadow`

## Current Status

Implemented, verified, and ready to commit.

## Verification

- `cargo fmt --all`: passed
- `cargo test`: passed
- `cargo run -p baron-cli -- --help`: passed
- `cargo run -p baron-cli -- survey .`: passed
- `cargo run -p baron-cli -- survey . --json`: passed
- `cargo run -p baron-cli -- init . --codex --shadow`: passed
- `docs/BARON_STATUS.json` parses as JSON: passed
- `git diff --check`: passed

## Next Action

Commit Phase 1, then begin Phase 2 - Vault + Memory Firewall.

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
