# SPEC-04 CURRENT Plan Integrity Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the linked plan file the canonical authority for active-plan metadata and fail closed whenever `CURRENT.md` disagrees with it.

**Architecture:** Parse the linked plan frontmatter into typed metadata, compare it with the authority-bearing fields in `CURRENT.md`, and carry mismatch issues through the existing shared active-plan evaluator. Completion, reconciliation, Stop, and completion-integrity status will consume this one validation path; lifecycle writers will refuse to write through a mismatched active pointer.

**Tech Stack:** Rust workspace, `baron-core` plan/automation/Prepare paths, integration tests, Cargo formatter/tests/Clippy/release build, maintained Markdown/JSON status evidence.

**Spec:** `D:/Works/Baron-Engine/fix-bug/prompt/README-PROMPT-SPEC-04-CURRENT-PLAN-INTEGRITY-FIX.md`

## Global Constraints

- Stay on `codex/spec-04-proof-trace-gate-completion-integrity` at starting HEAD `459b45602061e5bfd294610e00f8ac3ec04f8887`.
- Implement only the remaining SPEC-04 CURRENT-to-linked-plan authority fix; do not rework FIX-01/FIX-02 except where the shared validation path requires integration.
- Keep source version `5.0.0`; do not start SPEC-05/SPEC-06, modify Hotel Staff, create a tag/release, or declare SPEC-04 closed.
- Preserve safe plan-path checks, operation-scoped proof/trace/gate authority, fresh trace score evaluation, legacy diagnostic-only behavior, and unrelated user files.
- Keep mismatch behavior fail-closed: no guessing, auto-repair, plan completion, or silent selection of either metadata source.

## Review Focus

- A high-risk linked plan cannot be downgraded by changing only `CURRENT.md` risk; the completion gate must remain high-risk.
- A partial or rewritten operation identity in `CURRENT.md` cannot become an identified authority against a linked legacy or different-operation plan.
- A title-only edit, task/operation/session/request/adapter rewrite, or pointer switch to another valid plan must fail before evidence evaluation.
- A completed plan must detect later CURRENT metadata tampering through `plan_status` completion-integrity output.
- An exact CURRENT/linked-plan match must preserve the positive proof → trace → receipt/gate completion path.

### Task 1: Add deterministic authority-root regressions

**Files:**
- Modify: `crates/baron-core/tests/plan.rs`
- Modify: `crates/baron-core/tests/automation.rs` only if a small shared fixture or Stop assertion is required

**Interfaces:**
- Consume existing `start_or_resume_plan_for_operation`, `complete_plan`, `plan_status`, `reconcile`, `handle_hook`, proof/trace/gate helpers.
- Produce tests for risk downgrade, each identity-field rewrite, title/path mismatch, identified-versus-legacy mismatch, post-completion tamper, the exact reviewed bypass, and the unchanged positive chain.

- [x] **Step 1: Complete preflight and trace the current authority flow.**

  Confirm the branch, reviewed HEAD, clean worktree, current `active_plan` parsing, and every correctness-sensitive consumer before editing.

- [x] **Step 2: Write the failing metadata-mismatch regressions.**

  Add deterministic tests that mutate only `docs/baron/plans/CURRENT.md`, assert `complete_plan` fails, `reconcile` reports `passed == false`, Stop returns a block decision, and the linked plan remains uncompleted. Cover risk, task ID, operation ID, adapter where practical, session ID, request ID, title, path switch, and legacy/identified binding state.

- [x] **Step 3: Run the plan and automation targets to verify RED.**

  Run `cargo test -p baron-core --test plan --no-fail-fast` and `cargo test -p baron-core --test automation --no-fail-fast`. The new mismatch tests must fail against the current trust model for the expected reason.

### Task 2: Parse and validate linked-plan metadata centrally

**Files:**
- Modify: `crates/baron-core/src/plan.rs`

**Interfaces:**
- Add a private typed `PlanFileMetadata` containing title, status, risk, task ID, and optional operation ID, adapter, session ID, and request ID.
- Extend the active-plan load path with authority mismatch issues and a single CURRENT↔linked-plan validation routine.
- Keep `active_plan_completion_evidence_status`, `completion_evidence_status`, `complete_plan`, `completion_integrity_issues`, `plan_status`, `reconcile`, and Stop on the shared path.

- [x] **Step 1: Parse linked plan frontmatter without reconstructing missing identity.**

  Read the linked file selected by the safe CURRENT path, parse exact `title:`, `status:`, `risk:`, `task_id:`, `operation_id:`, `adapter:`, `session_id:`, and `request_id:` fields, and represent absent operation fields as unbound. Invalid or incomplete linked metadata becomes a fail-closed authority issue.

- [x] **Step 2: Compare all authority-bearing fields exactly.**

  Compare title, risk, task ID, operation ID, adapter, session ID, and request ID. Require both sides to be fully identified or both sides to remain legacy-unbound. Preserve status as a controlled transition field, but never use a mismatch to guess authority.

- [x] **Step 3: Feed mismatch issues to all correctness-sensitive consumers.**

  Make completion and lifecycle writers reject the validated active plan when authority issues exist; make reconciliation/Stop return a failing report; and make completed `plan_status` append `Completion integrity: failed` with the mismatch evidence. Do not rewrite CURRENT or the linked plan.

### Task 3: Verify the full SPEC-04 chain

**Files:**
- Modify: `crates/baron-core/tests/plan.rs` if additional exact-match coverage is needed
- Modify: `crates/baron-core/tests/automation.rs` if Stop coverage is needed

**Interfaces:**
- Reuse existing operation-scoped proof, fresh trace score, receipt, and strict quality-gate paths without changing their authority model.

- [x] **Step 1: Run focused plan, automation, proof/trace, and CLI regressions.**

  Run `cargo test -p baron-core --test plan --no-fail-fast`, `cargo test -p baron-core --test automation --no-fail-fast`, `cargo test -p baron-core --test proof_trace --no-fail-fast`, `cargo test -p baron-cli --test phase12_hooks_cli --no-fail-fast`, and `cargo test -p baron-cli --test execution_cli --no-fail-fast`.

- [x] **Step 2: Run the mandatory adversarial checklist.**

  Confirm risk downgrade, task/operation/adapter/session/request rewrites, title mismatch, pointer mismatch, identified/legacy mismatch, post-completion CURRENT tamper, operation-B evidence isolation, reconcile failure, Stop block, uncompleted linked plan, and exact-match positive completion.

- [x] **Step 3: Scan CURRENT readers and classify authority.**

  Inspect production readers of `docs/baron/plans/CURRENT.md`, `active_plan`, `Task ID:`, `Operation ID:`, `Risk:`, and `Plan:`. Confirm correctness-sensitive readers use the validated path and diagnostic readers do not claim authority.

### Task 4: Update evidence and perform final verification

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-21-spec-04-current-plan-integrity-fix.md`

- [x] **Step 1: Run Core all-targets and the full workspace suite.**

  Run `cargo test -p baron-core --all-targets --no-fail-fast` and `cargo test --workspace --all-targets --no-fail-fast`; investigate every new failure and record only the known PowerShell Archive and session-replay blockers if unchanged.

- [x] **Step 2: Run formatter, Clippy, release, version, diff, and JSON checks.**

  Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release --locked -p baron-cli`, `target/release/baron --version`, `git diff --check`, and a direct `BARON_STATUS.json` parse.

- [x] **Step 3: Record current-state evidence without closure.**

  Document CURRENT↔linked-plan validation, all regression counts, verification results, and unchanged unrelated blockers. Keep the status exactly `ready for final adversarial review`.

### Task 5: Review, commit, and push

- [x] **Step 1: Review status, full diff, staged diff, and diff check.**
- [x] **Step 2: Stage only the authority-fix source/tests/docs/plan files.**
- [x] **Step 3: Commit the narrow SPEC-04 review fix.**
- [x] **Step 4: Push `codex/spec-04-proof-trace-gate-completion-integrity` and verify local/remote SHAs match.**
