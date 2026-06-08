# Current Build Note

Date: 2026-06-09

## Current Phase

Phase 2 - Vault + Memory Firewall.

## What Is Being Built

- Vault scaffold
- project capsule
- SQLite memory index rebuilt from Markdown
- Memory Firewall ranking
- cross-project blocking
- `memory status`, `memory index`, `memory compact`, and `recall`

## Current Status

Implemented, verified, and ready to commit.

## Verification

- `cargo fmt --all`: passed
- `cargo test`: passed
- `baron memory status . --vault <target-temp-vault>`: passed
- `baron memory index . --vault <target-temp-vault>`: passed
- `baron memory compact . --vault <target-temp-vault>`: passed
- `baron recall "survey engine proof" . --vault <target-temp-vault>`: passed
- multi-project firewall smoke: passed
- `docs/BARON_STATUS.json` parses as JSON: passed
- `git diff --check`: passed

## Next Action

Run final verification, commit Phase 2, then begin Phase 3 - Context Compiler.

## Phase 2 Commit

- Commit message: `feat: add vault memory firewall`
- Use `git log -1 --oneline` after commit for the exact hash.

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
