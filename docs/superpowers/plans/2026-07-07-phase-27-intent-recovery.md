# Phase 27 Intent Clarity And Actionable Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist confirmed product intent before risky work and preserve actionable recovery evidence after failed, blocked, or interrupted work.

**Architecture:** Add a focused `intent` module for durable Product Harness intent briefs and confirmation gates. Extend `continuity` with append-only recovery packets and bounded current pointers, then connect both to hidden CLI commands, compact context, generated adapters, status, and Vault mirrors.

**Tech Stack:** Rust, clap, Markdown source-of-truth files, existing Baron Vault context, existing Rust integration-test harness.

---

### Task 1: Intent model and confirmation gate

**Files:**
- Create: `crates/baron-core/src/intent.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/harness.rs`
- Test: `crates/baron-core/tests/intent.rs`

- [x] Write failing tests for repo/Vault mirroring, medium/high confirmation refusal, matching confirmed intent acceptance, low-risk lightweight intake, and duplicate intent history prevention.
- [x] Run `cargo test -p baron-core --test intent` and confirm failure because the intent API does not exist.
- [x] Implement `IntentBriefInput`, `IntentBrief`, `record_intent`, `intent_status`, and `require_confirmed_intent`.
- [x] Gate medium/high `start_or_resume_intake` through a confirmed matching intent while preserving low-risk behavior.
- [x] Run the focused intent and Harness tests until green.

### Task 2: Intent CLI and compact context

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-core/src/context.rs`
- Test: `crates/baron-cli/tests/execution_cli.rs`
- Test: `crates/baron-core/tests/context_compiler.rs`

- [x] Write failing CLI tests for `baron harness intent` and `baron harness intent-status`.
- [x] Write failing context tests for bounded Intent Clarity output and `--why` explanation.
- [x] Implement hidden CLI parsing and dispatch with repeatable non-goal, constraint, decision, and unknown flags.
- [x] Load only the bounded current intent brief in compact context.
- [x] Run focused CLI/context tests until green.

### Task 3: Actionable recovery packets

**Files:**
- Modify: `crates/baron-core/src/continuity.rs`
- Test: `crates/baron-core/tests/continuity.rs`

- [x] Write failing tests for failed/blocked/interrupted packets, append-only history, Vault mirrors, deduplication, linked evidence, and current recovery status.
- [x] Implement `RecoveryInput`, `RecoveryPacket`, `record_recovery`, and bounded recovery status rendering.
- [x] Include latest recovery in continuity status and resume packet without deleting older attempts.
- [x] Run focused continuity tests until green.

### Task 4: Recovery CLI and context integration

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-core/src/context.rs`
- Test: `crates/baron-cli/tests/automation_cli.rs`
- Test: `crates/baron-core/tests/context_compiler.rs`

- [x] Write failing CLI tests for `baron continuity recover` and recovery status output.
- [x] Implement required root-cause, last-success, next-action, and outcome arguments plus repeatable evidence, affected-file, and retry-condition values.
- [x] Add bounded current recovery to compact context and explain it in context `--why`.
- [x] Run focused automation/context tests until green.

### Task 5: Adapter automation and durable project status

**Files:**
- Modify: `crates/baron-adapters/src/install.rs`
- Test: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Modify: `AGENTS.md`
- Modify: `README.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Create: `notes/build-log/2026-07-07-phase-27-intent-recovery.md`

- [x] Write failing adapter contract tests for read-before-ask, one-question-at-a-time, confirmed intent before risky intake, and recovery-before-unfinished-final behavior.
- [x] Update all generated adapters without adding user-facing command clutter.
- [x] Synchronize source docs, JSON status, current checkpoint, and Phase 27 build log.
- [x] Run adapter contract tests until green.

### Task 6: Phase verification and completion

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `notes/build-log/2026-07-07-phase-27-intent-recovery.md`

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Smoke a temp project/Vault through confirmed intent, high-risk intake, interrupted recovery, next-session context, and low-risk intake.
- [x] Parse `docs/BARON_STATUS.json` and run `git diff --check`.
- [x] Mark only Phase 27 complete, keep Phases 28-31 planned, commit, and push `main`.
