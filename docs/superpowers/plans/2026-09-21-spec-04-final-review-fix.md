# SPEC-04 Final Review Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove the two reviewed SPEC-04 trust-boundary defects without broadening the feature: global newest proof/trace leakage through `automation::reconcile`, and persisted trace-score text being treated as completion authority.

**Architecture:** Reuse the existing operation-scoped completion evaluator from `plan.rs` for reconciliation. Extract the current trace scoring calculation into one fresh evaluator in `trace.rs`; `score_trace` persists its result for display, while correctness selectors recompute from the current trace, proof, receipt, gate, and capability state.

**Tech Stack:** Rust workspace, `baron-core` integration tests, `baron-cli` hook tests, Cargo fmt/test/clippy/release build, Markdown/JSON maintained status docs.

**Spec:** `D:/Works/Baron-Engine/fix-bug/prompt/README-PROMPT-SPEC-04-FINAL-REVIEW-FIX.md`

## Global Constraints

- Stay on `codex/spec-04-proof-trace-gate-completion-integrity` at reviewed HEAD `5e5a23dc17608adcc27d2f039f8a439bd4129013`.
- Implement only FIX-01 and FIX-02; do not start SPEC-05/SPEC-06, modify Hotel Staff, bump `5.0.0`, tag, release, or declare SPEC-04 closed.
- Preserve legacy global selectors only where their callers are explicitly diagnostic/status/history surfaces.
- A correctness-sensitive path must use the active operation binding and exact `trace.proof_id == proof.id` chain.
- Persisted `Passed`, `Achieved`, `Required`, `Missing`, and `Warnings` fields are cache/display evidence, never authority.

## Review Focus

- Active operation A must not reconcile from newer proof/trace/gates belonging to operation B.
- Stop-hook blocking must follow the same scoped evaluator as plan completion, including legacy unbound plans.
- A forged `Passed: yes` or elevated `Achieved` score block must not authorize completion.
- Rewritten trace headers and stale score blocks must be rejected by fresh current-state evaluation.
- The exact same-operation positive chain must remain green, including receipt freshness and strict quality gates.

### Task 1: Establish final-review regressions

**Files:**
- Modify: `crates/baron-core/tests/automation.rs`
- Modify: `crates/baron-core/tests/plan.rs`
- Modify: `crates/baron-core/tests/proof_trace.rs`
- Modify: `crates/baron-cli/tests/phase12_hooks_cli.rs` only if the existing hook harness is needed for an end-to-end Stop assertion.

**Interfaces:**
- Consume existing identity, proof, trace, gate-receipt, `reconcile`, `handle_hook`, `score_trace`, and `latest_trace_score_for_operation` APIs.
- Produce failing tests for operation-B leakage, cross-operation gates, tampered score fields, rewritten binding headers, and the unchanged positive chain.

- [x] **Step 1: Record the review findings and inspect current paths.**

  Confirm `automation::reconcile` calls global selectors and confirm `latest_trace_score_for_operation` parses the persisted score block.

- [x] **Step 2: Write the failing regressions.**

  Add deterministic fixtures that create an identified active plan A, newer valid artifacts for B, and tampered/re-written trace Markdown. Assert reconciliation/completion remains blocked and fresh score results reflect current state.

- [x] **Step 3: Run the focused regression targets and verify RED.**

  Run `cargo test -p baron-core --test automation --no-fail-fast`, `cargo test -p baron-core --test plan --no-fail-fast`, and `cargo test -p baron-core --test proof_trace --no-fail-fast`. The new tests must fail because production code still uses the global reconcile selector and persisted score block.

### Task 2: Share scoped completion evidence with reconciliation

**Files:**
- Modify: `crates/baron-core/src/plan.rs`
- Modify: `crates/baron-core/src/automation.rs`

**Interfaces:**
- Produce a public `CompletionEvidenceStatus` (or equivalent) and `active_plan_completion_evidence_status` API returning the same scoped issues used by `complete_plan` and completion-integrity diagnostics.
- `automation::reconcile` consumes that API and returns `active_plan: false` with `passed: true` when no active plan exists.

- [x] **Step 1: Expose the existing evaluator without duplicating it.**

  Wrap `completion_evidence_issues(repo_root, active_plan)` into a status value with `passed = issues.is_empty()`. Keep the active-plan status filter in the shared entry point so completed/no-plan state is not treated as an active operation.

- [x] **Step 2: Route `reconcile` through the shared entry point.**

  Remove `latest_proof` and `latest_trace_score` imports/calls from `automation.rs`; map shared issues into `ReconciliationReport.gaps` and preserve the current Stop-hook block/retry behavior.

- [x] **Step 3: Run the new reconciliation tests GREEN.**

  Re-run the automation, plan, and hook-focused tests and confirm operation A cannot use operation B proof, trace, or quality-gate receipts.

### Task 3: Freshly evaluate trace score authority

**Files:**
- Modify: `crates/baron-core/src/trace.rs`
- Modify: `crates/baron-core/src/plan.rs` only if visibility or evaluator reuse requires it.

**Interfaces:**
- Produce one internal `evaluate_trace_score(repo_root, content) -> Result<TraceScore>` authority evaluator.
- `score_trace` finds/writes the trace and persists the fresh result for display/history.
- `latest_trace_score_for_operation` reads the exact scoped trace and calls the fresh evaluator, never `parse_score` for authority.

- [x] **Step 1: Extract the current score calculation.**

  Move the existing structure/binding/proof/risk/receipt/gate/capability/tier computation out of `score_trace` into the evaluator, preserving the existing Low Minimal, Medium Standard, and High Detailed semantics.

- [x] **Step 2: Make the operation selector ignore cached score fields.**

  Remove the required score-block presence check and `parse_score` authority path from `latest_trace_score_for_operation`; evaluate current trace content and current proof/receipt/gate state instead.

- [x] **Step 3: Run score and completion tests GREEN.**

  Verify a missing, forged, stale, or contradictory score block cannot grant authority, while `score_trace` still rewrites the display block and returns the fresh score.

### Task 4: Static trust-boundary scan and documentation

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-21-spec-04-final-review-fix.md`

- [x] **Step 1: Classify every remaining global selector.**

  Scan `automation.rs`, `plan.rs`, `prepare.rs`, `continuity.rs`, `task_state.rs`, `harness_improvement.rs`, `trace.rs`, and `proof.rs`; label remaining occurrences diagnostic/status/history or out-of-scope, with no correctness-sensitive occurrence left.

- [x] **Step 2: Run the full required verification matrix.**

  Run focused tests, Core all-targets, workspace all-targets, formatter, warnings-denied Clippy, locked release build, binary version, diff check, and status JSON parsing. Record only observed unrelated blockers.

- [x] **Step 3: Update maintained evidence without closure.**

  Record FIX-01, FIX-02, regression counts, full verification, known blockers, and status `ready for final adversarial review`.

### Task 5: Review, commit, and push

- [x] **Step 1: Review `git status`, full diff, and `git diff --check`.**
- [x] **Step 2: Stage only the final-review source/tests/docs/plan files.**
- [x] **Step 3: Commit the review fix on `codex/spec-04-proof-trace-gate-completion-integrity`.**
- [x] **Step 4: Push the branch and verify local/remote SHAs match.**
