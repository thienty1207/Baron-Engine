# Baron 2.2 Agent Skills And Continuity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Baron 2.2.0 with tighter skill/agent intelligence and an explicit continuity ledger for interrupted implementation work.

**Architecture:** Phase 16 refines Baron-managed core assets and control-plane routing without changing Superpowers ownership or making optional assets core. Phase 17 adds a focused `continuity` module that writes a bounded resume packet from existing plan, harness, proof, trace, and automation state, then exposes it through context and hidden automation commands.

**Tech Stack:** Rust 2021, Clap, Serde/TOML/JSON, Chrono, Markdown assets, existing Baron adapter/control-plane/context/automation modules.

---

### Task 1: Red Tests For Skills, Agents, And Routing

**Files:**
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Modify: `crates/baron-core/tests/control_plane.rs`

- [x] Add tests that require Addy-inspired rubrics in the three core agents while keeping them Baron-native.
- [x] Add tests that require optional `web-performance-auditor` to be installed but not counted as a core gate.
- [x] Add tests that route API, observability, performance, migration, and web performance tasks narrowly.
- [x] Run targeted tests and confirm they fail before implementation.

### Task 2: Red Tests For Continuity Ledger

**Files:**
- Create: `crates/baron-core/tests/continuity.rs`
- Modify: `crates/baron-core/tests/context_compiler.rs`
- Modify: `crates/baron-cli/tests/automation_cli.rs`

- [x] Add tests that a continuity checkpoint writes repo and Vault `CURRENT.md`.
- [x] Add tests that SessionStart/Checkpoint hooks update the resume packet.
- [x] Add tests that compact context includes the bounded resume point.
- [x] Add CLI tests for hidden continuity status/checkpoint commands.
- [x] Run targeted tests and confirm they fail before implementation.

### Task 3: Implement Phase 16 Assets And Control Plane

**Files:**
- Modify: `assets/core/agents/code-reviewer.toml`
- Modify: `assets/core/agents/security-auditor.toml`
- Modify: `assets/core/agents/test-engineer.toml`
- Create: `assets/core/agents/web-performance-auditor.toml`
- Modify: `assets/core/skills/frontend-design/SKILL.md`
- Modify: `assets/core/skills/vibe-security-scan/SKILL.md`
- Create: `assets/core/skills/api-and-interface-design/SKILL.md`
- Create: `assets/core/skills/observability-and-instrumentation/SKILL.md`
- Create: `assets/core/skills/performance-optimization/SKILL.md`
- Create: `assets/core/skills/deprecation-and-migration/SKILL.md`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-core/src/control_plane.rs`
- Modify: `crates/baron-cli/src/main.rs`

- [x] Add narrow optional skill routing and optional agent routing.
- [x] Keep mandatory core gates exactly `code-reviewer`, `security-auditor`, `test-engineer`.
- [x] Keep all optional domains lazy, explainable, and non-core.

### Task 4: Implement Phase 17 Continuity Ledger

**Files:**
- Create: `crates/baron-core/src/continuity.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/automation.rs`
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-adapters/src/install.rs`

- [x] Add repo/Vault continuity packet writing.
- [x] Update hooks to checkpoint continuity automatically.
- [x] Add hidden `baron continuity status|checkpoint` for AI/runtime use.
- [x] Include continuity resume in compact context.
- [x] Update adapter startup contracts to force resume-first behavior.

### Task 5: Docs, Status, Version, And Verification

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `notes/build-log/2026-06-18-baron-2-2-agent-skills-roadmap.md`

- [x] Bump version to `2.2.0`.
- [x] Mark Phase 16 and Phase 17 complete only after tests pass.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Smoke init/context/control-plane/continuity on a temp project.
- [ ] Commit and push.
