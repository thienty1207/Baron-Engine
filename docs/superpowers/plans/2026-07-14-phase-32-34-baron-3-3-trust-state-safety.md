# Baron 3.3 Trust And State Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Baron 3.3.0 with explicit request authority, coherent state mutation, proof-backed completion integrity, read-only query storage, and proof-before-tag release promotion.

**Architecture:** Add small Baron-core modules for authority and state integrity, route all adapter automation and mutating CLI context through those contracts, then harden the existing release manifest/workflow instead of creating a parallel release system. Vault Markdown remains durable truth; SQLite remains a disposable accelerator.

**Tech Stack:** Rust 2021, Clap, Serde/TOML/JSON, rusqlite, GitHub Actions, PowerShell/Bash installers.

---

### Task 1: Phase 32 Request Authority Contract

**Files:**
- Create: `crates/baron-core/src/authority.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Test: `crates/baron-core/tests/authority.rs`
- Test: `crates/baron-cli/tests/authority_cli.rs`
- Test: `crates/baron-adapters/tests/adapter_lifecycle.rs`

- [ ] Write failing tests for read-only, change, mixed review-and-fix, Vietnamese, and ambiguous requests.
- [ ] Run the focused tests and confirm they fail because authority classification is absent.
- [ ] Implement `RequestAuthority` and evidence-backed classification with change intent taking precedence and ambiguity defaulting to no mutation.
- [ ] Add hidden `baron authority classify "<request>"` output for AI automation.
- [ ] Update every generated adapter to classify authority before durable Baron writes and to keep read-only work mutation-free.
- [ ] Run focused core, CLI, and adapter tests until green.
- [ ] Commit Phase 32 independently.

### Task 2: Phase 33 Coherent State And Completion Integrity

**Files:**
- Create: `crates/baron-core/src/state_guard.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/memory.rs`
- Modify: `crates/baron-core/src/session_replay.rs`
- Modify: `crates/baron-core/src/plan.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-core/tests/state_guard.rs`
- Test: `crates/baron-core/tests/vault_memory.rs`
- Test: `crates/baron-core/tests/session_replay.rs`
- Test: `crates/baron-core/tests/plan.rs`
- Test: `crates/baron-cli/tests/execution_cli.rs`

- [ ] Write failing tests for missing capsule state, identity mismatch, unsupported config schema, and no-write failure behavior.
- [ ] Write failing tests proving SQLite query paths do not create or mutate indexes.
- [ ] Write a failing tampered-completion test requiring verification, proof, and passing trace evidence.
- [ ] Implement a coherent execution-context guard with actionable `baron update` recovery guidance.
- [ ] Route mutating CLI commands through the guard while keeping init/update as explicit repair owners.
- [ ] Open memory/session query connections read-only and preserve rebuild diagnostics.
- [ ] Add completion-integrity output to plan status and context without trusting hand-edited status text.
- [ ] Run focused state, memory, replay, plan, execution, context, and preservation tests until green.
- [ ] Commit Phase 33 independently.

### Task 3: Phase 34 Immutable Release Promotion And 3.3.0 Certification

**Files:**
- Modify: `crates/baron-core/src/release.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-core/tests/release.rs`
- Modify: `crates/baron-cli/tests/release_cli.rs`
- Modify: `crates/baron-cli/tests/workflow_contract.rs`
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`
- Modify: `docs/RELEASE.md`
- Create: `docs/assessment/baron-3.3.0-certification.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `notes/build-log/2026-07-14-phase-32-34-baron-3-3-trust-state-safety.md`

- [ ] Write failing tests for exact expected version/source identity, proof-before-tag ordering, publish-only write permission, and no asset overwrite.
- [ ] Harden release verification so manifest identity must match the proven candidate commit and requested version.
- [ ] Change release automation to build from an exact source candidate, verify all native artifacts and installer lifecycle, then create the tag and release in the final job only.
- [ ] Refuse existing tags/releases and never use replacement upload behavior.
- [ ] Bump workspace and lockfile version to `3.3.0`; synchronize docs and phase identity.
- [ ] Run format, full workspace tests, Clippy, release build, installer lifecycle, JSON/status, static workflow, and real project smoke checks.
- [ ] Record exact evidence in certification/status/build log and mark Phases 32-34 complete only after it passes.
- [ ] Merge the verified branch to `main`, push `origin/main`, and confirm local/remote heads match. Do not create the binary release unless explicitly requested.

## Plan Self-Review

- Every requested safeguard maps to one phase and one focused test group.
- No raw SQL command is added; existing query paths are hardened read-only instead.
- No new core skill or agent is introduced.
- Normal users still see only install, setup, init/platform, and update.
- GitHub tag/release creation remains separate from source push and happens only after hosted proof.
