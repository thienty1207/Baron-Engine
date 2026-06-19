# Phase 13-14 Certification And Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish Baron 2.0 by adding a self-contained certification gate for extreme scale and hardening the release contract to `2.0.0`.

**Architecture:** Add a Baron-native certification module that runs bounded stress checks across repository scale, shared Vault isolation, cache corruption recovery, context budgets, automation evidence, control-plane routing, and release readiness. Then wire the gate into the CLI, update release metadata/version/docs/status, and prove the final program through tests and smoke commands.

**Tech Stack:** Rust 2021, Clap, Serde/Markdown, existing Vault Markdown source of truth, existing SQLite cache, existing release metadata and adapter contracts.

---

## File Structure

- Create `crates/baron-core/src/certification.rs`: certification profiles, checks, Markdown/JSON report writer, latest-status reader, and release-readiness summary.
- Modify `crates/baron-core/src/lib.rs`: expose certification module and update phase identity.
- Modify `crates/baron-cli/src/main.rs`: add `baron certify run|status`.
- Create `crates/baron-core/tests/certification.rs`: stress/isolation/corruption/release-readiness tests.
- Create `crates/baron-cli/tests/certification_cli.rs`: CLI smoke tests for certification run/status.
- Modify release/version tests and docs for `2.0.0`.
- Modify `Cargo.toml`, `Cargo.lock`, README, AGENTS, architecture/roadmap/status/build-log docs.

### Task 1: Phase 13 Certification Core

**Files:**
- Create: `crates/baron-core/src/certification.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Test: `crates/baron-core/tests/certification.rs`

- [x] Write failing tests proving certification creates Markdown and JSON reports.
- [x] Write failing tests proving the report checks large repo survey, shared Vault isolation, memory index rebuild, cache corruption recovery, automation evidence, control-plane status, and release readiness.
- [x] Implement certification report types and `run_certification`.
- [x] Run focused certification tests until green.
- [x] Commit `feat: add Baron certification gate`.

### Task 2: Phase 13 Certification CLI

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Create: `crates/baron-cli/tests/certification_cli.rs`
- Modify: `crates/baron-cli/tests/cli.rs`

- [x] Write failing tests proving `baron certify run` prints a Baron Certification report and writes report files.
- [x] Write failing tests proving `baron certify status` reads the latest certification without creating new project files.
- [x] Add `certify run|status` command handling.
- [x] Run focused CLI tests until green.
- [x] Commit `feat: expose certification CLI`.

### Task 3: Phase 14 Version And Release Gate

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/baron-cli/tests/cli.rs`
- Modify: `crates/baron-core/tests/release.rs`
- Modify: `crates/baron-cli/tests/lifecycle_scripts.rs`
- Modify: `crates/baron-cli/tests/release_cli.rs`
- Modify: `crates/baron-cli/tests/release_smoke.rs`

- [x] Write failing tests expecting CLI version and release metadata to be `2.0.0`.
- [x] Bump workspace version to `2.0.0`.
- [x] Update installer/release tests to use the `2.0.0` contract.
- [x] Run focused release/version tests until green.
- [x] Commit `chore: bump Baron to 2.0.0`.

### Task 4: Docs, Status, And Final Audit

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `docs/architecture/ARCHITECTURE.md`
- Modify: `docs/architecture/COMMAND_SURFACE.md`
- Modify: `docs/RELEASE.md`
- Create: `docs/assessment/2026-06-16-baron-2-final-audit.md`
- Modify: `notes/build-log/CURRENT.md`
- Create: `notes/build-log/2026-06-16-phase-13-14-certification-release.md`

- [x] Update docs to describe certification, release gate, and Baron 2.0 status.
- [x] Add final audit explaining Baron's independent engine boundaries and proof surface.
- [x] Mark Phase 13 and Phase 14 complete only after verification passes.
- [x] Run docs/static scans.
- [ ] Commit `docs: complete Baron 2.0 certification and release`.

### Task 5: Final Verification, Merge, And Push

**Files:**
- All touched files

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Smoke a temp repo and Vault: init, memory index, context, control-plane route, harness audit, certify run, certify status, release metadata verify.
- [x] Run `git diff --check`.
- [ ] Merge `codex/phase-13-14` into `main`.
- [ ] Push `origin/main`.

## Self-Review

- Phase 13 maps to explicit certification and stress/regression proof, not vague confidence.
- Phase 14 maps to `2.0.0` version/docs/release readiness, not npm-style publishing.
- Baron remains its own harness: no Agent Bootstrap runtime and no borrowed external harness identity.
- Superpowers, the 3 core quality agents, optional frontend/security skills, Vault Markdown, memory firewall, and adapter contracts remain intact.
