# SPEC-05 Maintenance 01 — Logic Integrity Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Reproduce and remediate LOG-01 through LOG-06 and close G-01 with regression evidence while preserving Baron 5.0.0 and all pre-existing project, Vault, and machine state.

**Architecture:** Keep product semantics in Baron Core and make the CLI a typed projection of Core hook failure disposition. Reuse Core's safe filesystem and lock primitives; for shared journals, hold shared locks only to open a bounded snapshot, then validate and fold rows with bounded memory outside the lock.

**Tech Stack:** Rust workspace, Cargo integration tests, PowerShell 7 fixtures on Windows, GitHub Actions.

**Spec:** `docs/specs/SPEC-05-MAINTENANCE-01-LOGIC-INTEGRITY-HARDENING.md` (user-provided source is in the primary checkout and remains untracked); findings: `docs/specs/BARON-MAIN-LOGIC-AUDIT-2026-10-21.md`.

## Global Constraints

- Keep public/package version at `5.0.0`; do not release, tag, or start SPEC-06.
- Work only on branch `codex/spec-05-maintenance-01-logic-integrity`, based on `origin/main` SHA `9038afa9f840e3a62acc0f97c6aaa31849247525`.
- Use temporary repositories, Vaults, session roots, and `BARON_HOME`; do not initialize or mutate the user's original project/Vault/machine state.
- For every owned defect, record a test observed failing before the minimal fix and passing after it; distinguish bugs from coverage gaps and contract limits.
- Preserve historical SPEC-05 `CLOSED` evidence and existing untracked user files. Update only the new Maintenance 01 status and current-task summaries.
- Do not merge. Push/open a scoped PR only after local verification; report exact-head hosted CI separately.

## Review Focus

- Stop-hook parse, journal, correlation, ownership, and filesystem failures must block; only the documented malformed `UserPromptSubmit` JSON contract may soft-continue.
- Existing managed files with invalid UTF-8, permission/read failures, or unsafe filesystem entries must remain byte-identical and must not be replaced by a header/default.
- An unreadable source or malformed persisted session state must not be represented as empty/deleted truth or overwrite caches/import history.
- Review-gate diagnostics must report unknown/error when enumeration or a finding's status cannot be validated.
- Journal reads must validate all rows, bound memory and shared-lock time, and preserve exact total/latest semantics without treating missing, empty, or corrupt journals as equivalent.

---

## Task 1: LOG-01 — typed fail-closed hook errors

**Files:** `crates/baron-core/src/automation.rs`; `crates/baron-cli/src/main.rs`; `crates/baron-cli/tests/phase12_hooks_cli.rs`.

**Interfaces:** Core exposes a typed `HookFailureKind` (`Advisory` or `FailClosed`) in the error chain. The CLI renders only explicitly typed `Advisory` failures as `continue=true`; absent/unknown kinds block.

- [x] Add `cli_stop_malformed_json_is_blocking` and `cli_stop_corrupt_journal_is_blocking_without_publication`; assert `decision=block`, `completed=false`, no `continue=true`, and no authoritative writes.
- [x] Run the new CLI tests and confirm they fail for the soft response produced by `render_hook_failure`.
- [x] Add typed Core failure classification. Mark only malformed `UserPromptSubmit` JSON advisory to preserve the existing documented prompt fallback; default every other error and all untyped CLI setup errors to fail-closed.
- [x] Run the two new tests plus `cli_hook_failure_is_soft_and_fallback_command_remains_available` and `cli_corrupt_dedup_state_is_a_hard_hook_failure_with_blocking_response`.
- [x] Commit the Core/CLI fix and regressions.

## Task 2: LOG-02 — preserve managed index and document bytes

**Files:** `crates/baron-core/src/plan.rs`; `trace.rs`; `architecture.rs`; `platform.rs`; `harness.rs`; their Core integration tests.

**Interfaces:** Every selected writer uses `safe_io::read_text(path)?`; only `Ok(None)` supplies the existing new-file header. Existing locks and replace-first publication remain in force.

- [x] Add real-mutator regressions `plan_start_preserves_unreadable_index_bytes`, `plan_progress_preserves_unreadable_index_and_leaves_recovery_journal`, `trace_score_preserves_unreadable_indexes_and_trace_mirrors`, `architecture_update_preserves_invalid_utf8_managed_file`, `platform_update_preserves_invalid_utf8_managed_file`, and `intake_preserves_unreadable_story_index_bytes`.
- [x] Run the regressions and confirm current readers overwrite the plan, trace, architecture, and platform fixtures. The harness test's first draft stopped at a missing-intent precondition; it was corrected to use low-risk intake and then observed RED on the corrupt index.
- [x] Replace lossy reads with explicit missing-file handling. Trace scoring validates both indexes before writing either trace mirror. The sibling harness writer had the same bug and is fixed; the intent writer already used `safe_io::read_text`.
- [x] Run all six focused regressions; each passes with byte-identical corrupt files and, for trace scoring, byte-identical repo/Vault trace mirrors.

## Task 3: LOG-03 — fail CodeGraph builds on unreadable sources

**Files:** `crates/baron-core/src/knowledge.rs`; `crates/baron-core/tests/knowledge.rs` or the existing graph integration suite.

- [x] Add `unreadable_source_does_not_change_code_graph_cache`; seed a valid cache, replace a discovered Rust source with invalid UTF-8, and assert the build errors with cache bytes unchanged. Keep intentional source deletion coverage green.
- [x] Run the new test and confirm the current empty-content fallback returned success and changed the graph/cache.
- [x] Propagate source read errors; graph/cache publication occurs only after all selected source files are read and parsed successfully.
- [x] Run the regression and `knowledge::tests::wiki_and_local_graph_are_project_bound_and_incremental`; both pass, including intentional-deletion tombstone coverage.

## Task 4: LOG-04 — strict, serialized session imports

**Files:** `crates/baron-core/src/session.rs`; `crates/baron-core/tests/session.rs`.

**Interfaces:** `load_state(path)` returns `Result<ImportState>`; missing state alone maps to `ImportState::default()`. Import publication acquires checkout then shared-Vault mutation locks and reloads current state while locked. External session discovery/parsing occurs before that lock; disjoint concurrent sources merge against the latest state.

- [x] Add `corrupt_session_import_state_is_preserved_and_reported` and `session::tests::concurrent_imports_preserve_disjoint_source_union`; use isolated session-root environment overrides and a temp Vault.
- [x] Run both tests and confirm corrupt state was hidden by the summary and the barrier-synchronized source union was RED at 1/2 entries.
- [x] Implement strict missing-versus-error state loading and checkout-then-Vault locked revalidation/publication. External session discovery and parsing happen outside mutation locks; repeat import remains idempotent.
- [x] Run the full session integration target and the barrier/retry unit test; all pass.

## Task 5: LOG-05 — make review status incomplete state explicit

**Files:** `crates/baron-core/src/review_gate.rs`; `crates/baron-core/tests/review_gate.rs`; `context.rs` only if caller behavior needs adjustment.

- [x] Add `review_status_rejects_corrupt_finding_status`, missing/unknown/duplicate status checks, invalid findings-root and nested-entry checks, Unix symlink coverage, and context unavailable assertions.
- [x] Run the regressions RED: corrupt UTF-8 returned `Open findings: 1` despite the second finding being unreadable; malformed statuses were counted closed; a findings path that was a regular file returned zero findings; context omitted `unavailable`.
- [x] Distinguish a missing findings directory from unsafe path components, enumeration/read failures, non-regular entries, invalid UTF-8, and missing/unknown/duplicate status; propagate actionable errors.
- [x] Run the review-gate target (9/9) and both context unavailable regressions; commit the scoped implementation and evidence.

## Task 6: LOG-06 — stream validated journal snapshots outside locks

**Files:** `crates/baron-core/src/automation.rs`; `autopilot.rs`; `crates/baron-core/tests/automation.rs`; `tests/autopilot.rs`.

**Interfaces:** Add a crate-visible journal snapshot scanner. It opens the regular journal and captures its length while holding checkout then Vault locks, then releases both locks before streaming rows. It validates every non-empty JSONL row, enforces a maximum row size, distinguishes missing from present-empty, and folds into caller-owned bounded state.

- [x] Add tests for valid total/latest results, malformed prefix/tail and invalid UTF-8, missing versus empty, oversized row, latest-event correctness during concurrent append, scan under held mutation locks, and bounded retained event kinds.
- [x] Run tests RED: malformed automation prefix/tail, oversized rows, missing/empty labeling and malformed Autopilot input failed assertions because current readers reported counts or healthy partial/empty results. Existing lock tests confirmed readers waited on the shared Vault lock during reads.
- [x] Implement a regular-file snapshot opened under checkout then Vault locks; capture its length, release both locks, then stream 8 KiB buffers with a 1 MiB row cap, strict JSON/UTF-8 validation, exact count/latest semantics, and capped event-kind retention. Both automation status and Autopilot consume this scanner; missing, empty, present, and corrupt states are distinct.
- [x] Run automation (14/14), Autopilot (6/6), journal scanner unit (4/4), and phase12_hooks (16/16, including shared-Vault concurrency, replay/dedup, Stop); commit the scoped changes and evidence.

## Task 7: G-01 — recovery after CURRENT publication

**Files:** `crates/baron-core/src/plan.rs`; `crates/baron-core/tests/plan.rs`.

- [x] Add child-process fault stage `after_current_before_journal_cleanup` and an A/B fixture where A publishes progress while B's authority is snapshotted byte-for-byte.
- [x] Run the test RED: current code had no failpoint at this boundary, so the child completed publication instead of interrupting with the transition journal still present.
- [x] Require two fresh recovery subprocesses and repeated retries to apply A exactly once, keep Repo/Vault plan/index/CURRENT coherent, safely clear the journal, and leave B's plan files and ACTIVE rows byte-identical. CURRENT remains a projection, not the authority.
- [x] Run plan unit tests (5/5) and the complete plan integration target (67/67); commit the test-only boundary and classify `TEST GAP CLOSED — NO SOURCE BUG`.

## Task 8: sibling audit, status, and release gates

**Files:** `docs/BARON_STATUS.md`; `docs/BARON_STATUS.json`; `docs/superpowers/plans/CURRENT.md`; `notes/build-log/CURRENT.md`; new maintenance verification record.

- [x] Search and disposition production matches for the spec's error/default, ignored read, read-modify-write, completion/error, CURRENT authority, journal-lock, and multi-file publication patterns. The lexical sweep plus targeted semantic review is documented with its limits; it is not represented as a line-by-line review of every match.
- [x] Record actual source baseline/source snapshot SHAs, RED/GREEN outcomes, sibling classifications, remaining limitations, and local verification in the evidence ledger. Historical SPEC-05 CLOSED remains unchanged; Maintenance 01 reached review acceptance after local gates.
- [x] Run fmt; separate Core, CLI, adapter, and workspace tests; Clippy with warnings denied; locked release build; `baron 5.0.0` smoke; status JSON, relative links, diff, and scope checks. No retired-adapter identifier is specified by this maintenance scope. All tests used temporary BARON_HOME/session roots.
- [x] Address the initial read-only review's two Important findings: malformed-readable Harness CURRENT risk and LOG-04 note/state interruption retry overwriting user edits. Both source fixes have RED/GREEN regressions and current workspace, Clippy, and release gates pass; see the updated evidence ledger.
- [x] Obtain updated independent read-only review of the complete `9038afa9f840e3a62acc0f97c6aaa31849247525..3005f5f67a352d62bf60aa1ddb166eead90b5222` range; reviewer ACCEPTED with no Critical or Important findings and did not run tests.
- [ ] Push the reviewed branch, open a scoped PR, and verify hosted checks on the exact PR head. Stop before merge.
