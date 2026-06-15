# Baron Phase 7 Capability Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Baron-native capability registry that detects available providers, understands adapter compatibility, degrades safely, and requires execution evidence for tool-backed completion claims.

**Architecture:** Store the committed registry in `.baron/capabilities.toml`, machine observations in `.baron/cache/capability-state.json`, and durable execution evidence in existing repo/Vault proof and trace Markdown. A focused `capability` core module owns validation, probing, selection, rendering, and cache persistence; CLI, context, proof, trace, and adapter startup contracts consume that API.

**Tech Stack:** Rust 2021, Clap, Serde/TOML/JSON, Chrono, existing Baron config/context/proof/trace modules, std networking and filesystem APIs.

---

### Task 1: Lock Phase 7 Recovery State

**Files:**
- Create: `docs/superpowers/specs/2026-06-15-baron-capability-registry-design.md`
- Create: `docs/superpowers/plans/2026-06-15-phase-7-capability-registry.md`
- Create: `notes/build-log/2026-06-15-phase-7-capability-registry.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`

- [x] **Step 1: Record the approved architecture and exact command/data contracts**
- [ ] **Step 2: Mark Phase 7 `implementation_in_progress` and set the next TDD action**
- [ ] **Step 3: Validate JSON and commit the recovery checkpoint**

Run:

```powershell
Get-Content docs/BARON_STATUS.json | ConvertFrom-Json | Out-Null
git diff --check
```

Expected: both commands exit successfully.

### Task 2: Add Registry Types, Validation, And Atomic Persistence

**Files:**
- Create: `crates/baron-core/src/capability.rs`
- Create: `crates/baron-core/tests/capability.rs`
- Modify: `crates/baron-core/src/lib.rs`

- [ ] **Step 1: Write failing tests for normalization, all provider kinds, duplicate rejection, and registry round-trip**
- [ ] **Step 2: Run `cargo test -p baron-core --test capability` and confirm missing API failures**
- [ ] **Step 3: Implement minimal registry types, validation, atomic load/save, register, list, and remove**
- [ ] **Step 4: Re-run the focused tests to green**
- [ ] **Step 5: Commit `feat: add Baron capability registry contract`**

### Task 3: Add Presence And Adapter Compatibility Checks

**Files:**
- Modify: `crates/baron-core/src/capability.rs`
- Modify: `crates/baron-core/tests/capability.rs`

- [ ] **Step 1: Add failing tests for CLI/binary path resolution, skill/MCP targets, HTTP unknown/bounded checks, agent adapters, and adapter restrictions**
- [ ] **Step 2: Run the focused test and confirm expected failures**
- [ ] **Step 3: Implement non-executing local probes, explicit bounded HTTP reachability, compatibility evaluation, and cache persistence**
- [ ] **Step 4: Add fallback selection tests for inactive, degraded, and full states**
- [ ] **Step 5: Re-run focused tests and commit `feat: detect capability providers safely`**

### Task 4: Expose Capability CLI

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Create: `crates/baron-cli/tests/capability_cli.rs`

- [ ] **Step 1: Write failing CLI tests for register, check, list, remove, JSON output, nested paths, and malformed registrations**
- [ ] **Step 2: Run `cargo test -p baron-cli --test capability_cli` and confirm command parsing failures**
- [ ] **Step 3: Implement `baron capability register|check|list|remove`**
- [ ] **Step 4: Verify human and JSON outputs expose presence, evidence, adapter compatibility, and requirement**
- [ ] **Step 5: Commit `feat: expose Baron capability commands`**

### Task 5: Connect Capability State To Bounded Context

**Files:**
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-core/tests/context_compiler.rs`
- Modify: `crates/baron-cli/tests/context_cli.rs`

- [ ] **Step 1: Write failing tests for bounded summaries, adapter filtering, stale/unknown state, and no product-file writes**
- [ ] **Step 2: Run focused context tests and confirm missing Capability Summary failures**
- [ ] **Step 3: Render a bounded cached capability summary without network probing or recursive scans**
- [ ] **Step 4: Update `context --why` with loaded/skipped capability rationale**
- [ ] **Step 5: Commit `feat: compile bounded capability context`**

### Task 6: Enforce Execution Evidence In Proof And Trace

**Files:**
- Modify: `crates/baron-core/src/proof.rs`
- Modify: `crates/baron-core/src/trace.rs`
- Modify: `crates/baron-core/tests/proof_trace.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-cli/tests/execution_cli.rs`

- [ ] **Step 1: Write failing tests proving provider presence alone cannot satisfy proof**
- [ ] **Step 2: Write failing tests for required missing/incompatible providers and valid structured execution evidence**
- [ ] **Step 3: Add structured capability evidence to proof records while preserving existing proof APIs**
- [ ] **Step 4: Make trace scoring inherit required capability gate failures and surface optional degradation as warnings**
- [ ] **Step 5: Run focused proof/trace and CLI tests to green**
- [ ] **Step 6: Commit `feat: gate tool-backed claims on execution evidence`**

### Task 7: Make Capability Checks Automatic For Agents

**Files:**
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`

- [ ] **Step 1: Write failing adapter contract tests for automatic capability checks and no-presence-equals-execution wording**
- [ ] **Step 2: Run `cargo test -p baron-adapters` and confirm failures**
- [ ] **Step 3: Update Codex, Claude, and generic startup contracts and helper surfaces**
- [ ] **Step 4: Verify custom skills/agents and user root instructions remain preserved**
- [ ] **Step 5: Commit `feat: automate capability awareness across adapters`**

### Task 8: Documentation, Status, Full Verification, And Integration

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/COMMAND_SURFACE.md`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `notes/build-log/2026-06-15-phase-7-capability-registry.md`
- Modify: `crates/baron-core/src/lib.rs`

- [ ] **Step 1: Document the natural-language capability model, automatic behavior, and execution-evidence rule**
- [ ] **Step 2: Mark Phase 7 complete, overall completion 95%, and Phase 8 next**
- [ ] **Step 3: Run formatting, focused tests, full tests, Clippy, JSON parse, and `git diff --check`**
- [ ] **Step 4: Smoke register/check/list/context/proof/trace/remove against a temp project**
- [ ] **Step 5: Commit final Phase 7 docs and verification evidence**
- [ ] **Step 6: Merge to `main`, rerun verification, and push `origin/main`**

Final commands:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
Get-Content docs/BARON_STATUS.json | ConvertFrom-Json | Out-Null
git diff --check
```

Expected: all commands pass with no warnings or uncommitted generated drift.

