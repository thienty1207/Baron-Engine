# Phase 9-10 Automation And Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver IDE-compatible lifecycle automation, collision-resistant project identity, lossless large-memory indexing, multilingual semantic recall, and automatic Codex/Claude session ingestion.

**Architecture:** Phase 9 adds a stored project ID, unique Vault capsule key, lifecycle journal, reconciliation gate, and native Codex/Claude hooks while preserving generic fallback instructions. Phase 10 replaces rebuild-and-truncate indexing with a deterministic incremental SQLite cache, adds concept-aware ranking and task-focused context, and imports matched session logs into redacted, deduplicated Vault Markdown.

**Tech Stack:** Rust 2021, Clap, Serde, SQLite through rusqlite, SHA-256, Regex, Chrono, ignore walker.

---

## File Structure

- Create `crates/baron-core/src/identity.rs`: project ID generation, capsule metadata, legacy capsule migration, and registry helpers.
- Create `crates/baron-core/src/automation.rs`: lifecycle event journal, hook payload parsing, reconciliation, and hook responses.
- Create `crates/baron-core/src/session.rs`: session-root discovery, defensive JSONL parsing, repo matching, redaction, deduplication, and imported Markdown.
- Modify `crates/baron-core/src/config.rs`: schema v2 stored project identity and migration.
- Modify `crates/baron-core/src/vault.rs`: unique capsule resolution and project metadata.
- Modify `crates/baron-core/src/memory.rs`: deterministic incremental source cache and complete Markdown indexing.
- Modify `crates/baron-core/src/firewall.rs`: multilingual concept scoring, recency/evidence weighting, and task-focused compact memory.
- Modify `crates/baron-core/src/context.rs`: automatic session import, task-focused memory, and lifecycle evidence.
- Modify `crates/baron-core/src/survey.rs`: remove silent 5,000-entry truncation while keeping rendered output bounded.
- Modify `crates/baron-adapters/src/install.rs`: install native Codex/Claude hooks and preserve custom routing blocks.
- Modify `crates/baron-adapters/src/managed.rs`: managed-section merge helpers for indexes and structured hook JSON.
- Modify `crates/baron-cli/src/main.rs`: automation and session-import commands plus lifecycle recording.
- Add focused core/CLI/adapter regression tests.
- Update README, architecture docs, status JSON/Markdown, roadmap, and build logs.

### Task 1: Phase 9 Project Identity

**Files:**
- Create: `crates/baron-core/src/identity.rs`
- Modify: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/vault.rs`
- Test: `crates/baron-core/tests/config.rs`
- Test: `crates/baron-core/tests/vault_memory.rs`

- [x] Write failing tests proving two repositories named `same-app` receive different project IDs and Vault capsules.
- [x] Write a failing test proving a configured repository keeps its project ID after moving.
- [x] Write a failing test proving a legacy `<slug>` capsule migrates into the unique capsule without losing Markdown.
- [x] Run the focused tests and confirm failures come from basename-only identity.
- [x] Implement schema v2 project identity, unique capsule keys, metadata, and safe legacy migration.
- [x] Run focused tests until green.
- [x] Commit `feat: add collision resistant project identity`.

### Task 2: Phase 9 Native Automation Runtime

**Files:**
- Create: `crates/baron-core/src/automation.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Test: `crates/baron-core/tests/automation.rs`
- Test: `crates/baron-cli/tests/automation_cli.rs`
- Test: `crates/baron-adapters/tests/adapter_lifecycle.rs`

- [x] Write failing tests for SessionStart journaling, context injection, checkpoint throttling, and Stop reconciliation.
- [x] Write failing adapter tests for `.codex/hooks.json` and `.claude/settings.json`.
- [x] Verify custom hook entries survive repeated Baron updates.
- [x] Implement lifecycle events `session_start`, `prompt`, `checkpoint`, `context_compiled`, `proof_recorded`, `trace_scored`, and `stop`.
- [x] Implement `baron automation status|reconcile|hook`.
- [x] Make Stop request continuation once when an active plan lacks required proof or a passing trace.
- [x] Run focused automation tests until green.
- [x] Commit `feat: add observable native automation runtime`.

### Task 3: Phase 9 Routing Preservation

**Files:**
- Modify: `crates/baron-adapters/src/managed.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Test: `crates/baron-adapters/tests/adapter_lifecycle.rs`

- [x] Write failing tests proving custom skill and agent index entries survive update.
- [x] Verify managed core routing refreshes while custom blocks remain byte-for-byte.
- [x] Implement explicit managed markers for core skill/agent routing.
- [x] Run adapter tests until green.
- [x] Commit `fix: preserve custom routing during adapter updates`.

### Task 4: Phase 10 Incremental Massive Memory Index

**Files:**
- Modify: `crates/baron-core/src/memory.rs`
- Modify: `crates/baron-core/src/survey.rs`
- Test: `crates/baron-core/tests/vault_memory.rs`
- Test: `crates/baron-core/tests/survey.rs`
- Test: `crates/baron-cli/tests/release_smoke.rs`

- [x] Write a failing test indexing more than 300 Markdown sources and retrieving the final source.
- [x] Write failing tests for unchanged-source reuse, changed-source refresh, and deleted-source removal.
- [x] Write a failing survey test where a risky path appears after 6,000 entries.
- [x] Implement sorted complete source discovery, SQLite source metadata, transactional upsert, stale deletion, and cache reuse.
- [x] Store project ID, slug, modified time, and source metadata in the rebuildable index.
- [x] Remove silent repository traversal truncation.
- [x] Run focused index and survey tests until green.
- [x] Commit `feat: add incremental large memory index`.

### Task 5: Phase 10 Multilingual Semantic Recall

**Files:**
- Modify: `crates/baron-core/src/firewall.rs`
- Modify: `crates/baron-core/src/context.rs`
- Test: `crates/baron-core/tests/vault_memory.rs`
- Test: `crates/baron-core/tests/context_compiler.rs`

- [x] Write a failing test where `bảo mật dữ liệu khách hàng` retrieves `Supabase RLS tenant isolation`.
- [x] Write failing tests for current-project priority, global approval, stale demotion, recency, and explicit cross-project access.
- [x] Write a failing context test proving `--task` loads relevant memory rather than the first indexed records.
- [x] Implement Unicode tokenization, concept aliases, title/path/kind/evidence/recency scoring, and score explanations.
- [x] Add task-focused compact recall while retaining bounded output.
- [x] Run focused recall/context tests until green.
- [x] Commit `feat: add task aware multilingual recall`.

### Task 6: Phase 10 Automatic Session Ingestion

**Files:**
- Create: `crates/baron-core/src/session.rs`
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-core/tests/session.rs`
- Test: `crates/baron-cli/tests/memory_cli.rs`

- [x] Write failing fixtures for matched Codex and Claude JSONL, unmatched projects, tool/system noise, duplicate imports, and secret values.
- [x] Verify `context` imports matched sessions automatically.
- [x] Verify imported Markdown excludes tool/system noise and redacts secrets.
- [x] Verify repeated runs do not duplicate notes.
- [x] Implement bounded root discovery, defensive parsing, confident repo matching, redaction, state tracking, and clean Markdown output.
- [x] Add `baron memory import-sessions`.
- [x] Run focused session tests until green.
- [x] Commit `feat: add automatic session memory ingestion`.

### Task 7: Integration, Documentation, And Phase Closure

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/architecture/MEMORY_MODEL.md`
- Modify: `docs/architecture/ADAPTERS.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `notes/build-log/CURRENT.md`
- Create: `notes/build-log/2026-06-15-phase-9-10-automation-memory.md`

- [x] Document hook trust, generic fallback, project identity, incremental index, semantic recall, and session privacy behavior.
- [x] Mark Phase 9 and Phase 10 complete only after their acceptance tests pass.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run duplicate-name, 6,000-file, 1,000-memory, semantic-query, session-import, and adapter-update smoke tests.
- [x] Run `git diff --check`.
- [x] Commit `docs: complete Baron phases 9 and 10`.

## Self-Review

- Every Phase 9 and Phase 10 checklist item in `docs/BARON_STATUS.md` maps to a task above.
- Project identity migration occurs before memory schema migration.
- Native hooks remain optional at runtime and report trust/availability instead of claiming execution.
- Markdown remains the source of truth; SQLite and automation state remain rebuildable or diagnostic.
- Generic agents remain supported without inventing a non-existent universal hook standard.
- No local embedding model, API key, server, Python runtime, Zig runtime, or cloud dependency is mandatory.
