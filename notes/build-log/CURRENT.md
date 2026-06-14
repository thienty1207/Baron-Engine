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

Implemented and verified. Final documentation commit and branch integration are
the only remaining actions.

## Verification

- Phase 3 baseline `cargo build --quiet`: passed
- Phase 3 baseline `cargo test --quiet`: passed
- configuration tests: passed
- adapter lifecycle and preservation tests: passed
- plan and harness tests: passed
- proof and trace tests: passed
- weak-proof and managed-state false-positive regressions: passed
- execution CLI tests: passed
- full `cargo test --quiet`: passed
- three-adapter/custom-preservation/high-risk lifecycle smoke: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- fresh three-adapter/custom-preservation/high-risk lifecycle smoke: passed
- `docs/BARON_STATUS.json` parse: passed
- `git diff --check`: passed after normalizing vendored trailing whitespace

## Next Action

Commit completion evidence, merge to `main`, and push.

## Phase 4-5 Feature Commits

- `07c09db feat: add Baron project configuration`
- `737a4e8 feat: add managed multi-agent adapters`
- `68df801 feat: add adapter init and update lifecycle`
- `0f555e9 feat: add risk-aware product harness`
- `055b0b8 feat: add proof and trace quality gates`
- `bd058e2 feat: add gated active plan state`
- `217233f feat: expose Baron execution state commands`
- `3ee93c5 feat: connect execution state to context and memory`
- `094891e feat: tighten Baron execution quality gates`

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
