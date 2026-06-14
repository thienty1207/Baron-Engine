# Baron Phase 4-5 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship durable Codex, Claude, and generic adapters plus risk-aware plan, harness, proof, and trace state.

**Architecture:** `baron-core` owns config and execution-state engines, `baron-adapters` owns managed adapter assets, and `baron-cli` exposes commands. Repo Markdown is active state, Vault Markdown is the durable mirror, and adapter instructions drive automatic agent behavior.

**Tech Stack:** Rust 2021, clap, serde, toml, chrono, rusqlite, Markdown, JSON.

---

### Task 1: Project And Local Configuration

**Files:**
- Create: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/vault.rs`
- Test: `crates/baron-core/tests/config.rs`

- [ ] Write failing tests for config creation, nested repo discovery, adapter
  registration, `--vault`/env/local precedence, and malformed config safety.
- [ ] Run `cargo test -p baron-core --test config` and confirm missing APIs fail.
- [ ] Add `ProjectConfig`, `LocalConfig`, config discovery, atomic writes, and
  repo-aware Vault resolution.
- [ ] Run the config test and full `baron-core` tests.
- [ ] Commit the configuration foundation.

### Task 2: Managed Adapter Engine

**Files:**
- Create: `crates/baron-adapters/src/managed.rs`
- Create: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/src/lib.rs`
- Test: `crates/baron-adapters/tests/adapter_install.rs`

- [ ] Write failing tests for Codex, Claude, and generic output shapes.
- [ ] Add preservation tests for text outside managed markers and custom
  skill/agent files.
- [ ] Implement managed-block replacement and known-asset refresh.
- [ ] Generate adapter startup contracts that require automatic Baron context,
  plan, harness, proof, and trace behavior.
- [ ] Run adapter tests and commit.

### Task 3: Adapter CLI Lifecycle

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-cli/tests/cli.rs`
- Create: `crates/baron-cli/tests/adapter_cli.rs`

- [ ] Write failing tests for non-shadow `init`, repeated multi-adapter init,
  `update`, local Vault reuse, and nested working-directory execution.
- [ ] Implement `init` and `update` dispatch through the adapter engine.
- [ ] Keep `init --shadow` read-only.
- [ ] Run CLI adapter tests and commit.

### Task 4: Active Plan State

**Files:**
- Create: `crates/baron-core/src/plan.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/context.rs`
- Test: `crates/baron-core/tests/plan.rs`

- [ ] Write failing tests for status, start/resume, update, interrupt, repo/Vault
  mirror, dated paths, and completion prerequisites.
- [ ] Implement plan Markdown and indexes under `docs/baron/plans`.
- [ ] Prefer `docs/baron/plans/CURRENT.md` in compiled context.
- [ ] Run plan and context tests and commit.

### Task 5: Product Harness And Risk

**Files:**
- Create: `crates/baron-core/src/harness.rs`
- Create: `crates/baron-core/src/risk.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Test: `crates/baron-core/tests/harness.rs`

- [ ] Write failing tests for low/medium/high intake, duplicate resume, decisions,
  friction, and Vault mirrors.
- [ ] Implement stories and current harness state under `docs/baron/harness`.
- [ ] Implement hard high-risk gates for auth, permissions, tenant/RLS, payment,
  migration, security, secrets, upload, provider, and destructive data.
- [ ] Run harness tests and commit.

### Task 6: Proof And Trace Quality

**Files:**
- Create: `crates/baron-core/src/proof.rs`
- Create: `crates/baron-core/src/trace.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Test: `crates/baron-core/tests/proof_trace.rs`

- [ ] Write failing tests for proof recording, risk-based requirements, trace
  recording, stable IDs, tier scoring, and Vault mirrors.
- [ ] Implement proof index and dated trace files.
- [ ] Block high-risk completion below detailed trace or without security/data
  proof.
- [ ] Run proof/trace tests and commit.

### Task 7: Execution CLI And Automatic Contract

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Create: `crates/baron-cli/tests/execution_cli.rs`
- Modify: adapter templates in `crates/baron-adapters/src/install.rs`

- [ ] Write failing CLI tests for all plan, harness, proof, and trace commands.
- [ ] Implement:
  - `baron plan status|start|update|interrupt|complete`
  - `baron harness status|intake|decision|friction`
  - `baron proof status|record`
  - `baron trace record|score`
- [ ] Verify commands work from nested directories without repeated `--vault`.
- [ ] Verify adapter instructions contain automatic startup and completion gates.
- [ ] Run CLI tests and commit.

### Task 8: Documentation, Status, And Release Evidence

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/architecture/ADAPTERS.md`
- Modify: `docs/architecture/COMMAND_SURFACE.md`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/CURRENT.md`

- [ ] Document automatic config, adapters, execution state, and proof gates.
- [ ] Mark Phase 4 and Phase 5 complete only after all verification passes.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Smoke init/update for all adapters and a high-risk plan lifecycle.
- [ ] Parse `docs/BARON_STATUS.json` and run `git diff --check`.
- [ ] Commit, merge to `main`, and push GitHub.

## Self-Review

- Every Phase 4 and Phase 5 checklist item maps to a task above.
- Config is navigation only; no memory is stored in TOML.
- Update behavior preserves user content and custom assets.
- High-risk completion has explicit proof and trace gates.
- Migration and release packaging stay outside this plan.
