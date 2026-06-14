# Current Build Note

Date: 2026-06-14

## Current Phase

Phase 4-5 - Agent Adapters And Execution Engine.

## What Is Being Built

- shared `.baron/project.toml` configuration
- machine-local `.baron/local.toml` Vault routing
- Codex, Claude, and generic managed adapters
- user-content and custom-asset preservation
- Active Plan State
- Product Harness risk intake
- proof recording and completion gates
- trace recording and quality scoring

## Current Status

Design and implementation plan approved. Implementation starting in an isolated
worktree.

## Verification

- Phase 3 baseline `cargo build --quiet`: passed
- Phase 3 baseline `cargo test --quiet`: passed
- Phase 4-5 verification: pending

## Next Action

Implement Task 1 - project/local configuration with TDD.

## Phase 4-5 Worktree

- Branch: `codex/baron-phase-4-5`
- Path: `C:\Users\Ho Thien Ty\.config\superpowers\worktrees\Baron-Engine\codex-baron-phase-4-5`
- Design: `docs/superpowers/specs/2026-06-14-agent-adapters-execution-engine-design.md`
- Plan: `docs/superpowers/plans/2026-06-14-phase-4-5-adapters-execution-engine.md`

## Phase 3 Commit

- Commit: `c192baf feat: add Baron context compiler`

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
