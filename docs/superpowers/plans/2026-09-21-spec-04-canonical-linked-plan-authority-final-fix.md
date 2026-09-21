# SPEC-04 Canonical Linked-Plan Authority Final Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close FIX-A and FIX-B by making linked-plan lifecycle status and canonical task/operation/risk derivation authoritative for every SPEC-04 correctness path.

**Architecture:** Load the safe CURRENT pointer, parse linked-plan metadata once, validate its lifecycle state and canonical identity against the existing project/operation contracts, then compare CURRENT as a projection. Feed the resulting authority issues and linked status through completion, reconciliation, Stop, plan status, and operation-bound trace paths without repairing either Markdown file.

**Tech Stack:** Rust workspace, `baron-core` plan/operation/config/vault paths, integration tests, Cargo formatter/tests/Clippy/release build, maintained Markdown/JSON status evidence.

**Spec:** `D:/Works/Baron-Engine/fix-bug/prompt/README-PROMPT-SPEC-04-CANONICAL-LINKED-PLAN-AUTHORITY-FINAL-FIX.md`

## Global Constraints

- Stay on `codex/spec-04-proof-trace-gate-completion-integrity` at reviewed HEAD `39203407e3faa3321bd00181ee65d5b89647d03a` before this fix.
- Implement only FIX-A lifecycle-status authority and FIX-B canonical linked-plan derivation; preserve all previously passing SPEC-04 fixes.
- Use the existing canonical project identity resolution, `classify_risk`, `task_id_for_task`, `LifecycleIdentity::from_parts_checked`, and `LifecycleIdentity::validate_task`; do not duplicate hash algorithms.
- Keep legacy plans readable but unbound; partial identity metadata fails closed and no identity is synthesized.
- Keep source version `5.0.0`; do not start SPEC-05/SPEC-06, modify Hotel Staff, create a tag/release, bump the version, or declare SPEC-04 closed.
- Do not auto-repair CURRENT or linked plans, migrate operation identity, or create a broad transaction system.
- Preserve unrelated user-owned/untracked files and stage only this final trust fix, its tests, plan, and maintained status evidence.

## Review Focus

- A linked `in_progress` plan cannot be hidden by `CURRENT.md` claiming `completed`, malformed, missing, or otherwise inactive status; reconciliation and Stop must fail closed.
- A linked `completed` plan cannot be silently treated as active when CURRENT says `in_progress`; status divergence must remain an integrity failure.
- A matching CURRENT/linked pair with a forged risk, task ID, operation ID, adapter, session, or request must fail canonical derivation rather than become authority through agreement.
- A recomputed forged operation tuple must not bypass task/risk derivation; the residual limitation that Markdown alone cannot prove historical origin must be documented if no independent origin source exists.
- Exact high-risk plan A to low-risk operation B evidence must be rejected before proof/trace/gate authorization, while the untouched exact-operation completion chain remains green.

### Task 1: Add FIX-A/FIX-B RED regressions

**Files:**
- Modify: `crates/baron-core/tests/plan.rs`

**Interfaces:**
- Consume `start_or_resume_plan_for_identity`, `complete_plan`, `plan_status`, `reconcile`, `handle_hook`, operation identity helpers, proof/trace/gate helpers, and the existing test fixtures.
- Produce observable failures for lifecycle status hiding, malformed CURRENT status, canonical linked metadata tampering, the reviewed operation-B bypass, and the unchanged positive completion chain.

- [x] **Step 1: Add lifecycle-status mismatch tests before production changes.**

  Add tests that mutate only CURRENT status to `completed`, `in_progress`, an unsupported value, and a missing value while the linked plan has the opposite or active state. Assert `reconcile().passed == false`, Stop returns `decision:block`, `plan_status` reports failed integrity where applicable, and the linked plan file is not rewritten.

- [x] **Step 2: Add canonical linked metadata tamper tests before production changes.**

  Add tests that mutate both CURRENT and the linked plan for risk, task ID, operation ID, adapter, session ID, request ID, partial identity, and a recomputed forged tuple. Assert completion/reconcile/Stop fail and the linked plan remains uncompleted. Add the exact high-risk A/low-risk B evidence bypass with both files rewritten to B metadata.

- [x] **Step 3: Run the focused plan target and record the expected RED results.**

  Run `cargo test -p baron-core --test plan --no-fail-fast`. The new FIX-A/FIX-B tests must fail because raw CURRENT status can hide the linked plan and linked metadata is not yet derived canonically; existing tests must remain distinguishable from the new failures.

### Task 2: Centralize canonical linked-plan and lifecycle validation

**Files:**
- Modify: `crates/baron-core/src/plan.rs`
- Modify: `crates/baron-core/src/vault.rs` only to expose the existing canonical project-identity resolution to the plan validator, if required by the implementation.

**Interfaces:**
- Consume `load_plan_file_metadata`, `ActivePlan`, `active_plan_completion_evidence_status`, `load_project_config`/Vault identity resolution, `classify_risk`, `task_id_for_task`, `SupportedAdapter`, and `LifecycleIdentity`.
- Produce one validated linked-plan authority result that carries canonical linked status, canonical title/risk, and an optional validated `PlanOperationBinding`, plus fail-closed authority issues.

- [x] **Step 1: Validate linked lifecycle status and canonical metadata.**

  Parse only supported lifecycle statuses (`in_progress`, `interrupted`, `needs_correction`, `blocked`, `completed`) for both linked metadata and CURRENT. Resolve the project ID through the existing config/Vault identity source. Require linked risk to equal `classify_risk(title)` and new/identified linked task IDs to equal `task_id_for_task(project_id, title)`. Preserve exact historical `task-<slug>` IDs for legacy-unbound plans without treating them as operation identity; arbitrary legacy task IDs fail closed.

- [x] **Step 2: Validate identified operation tuples through existing identity contracts.**

  Keep all four operation fields absent for legacy plans. For identified plans parse the supported adapter, call `LifecycleIdentity::from_parts_checked(project_id, task_id, operation_id, adapter, session_id, request_id)`, then call `validate_task(title)`. Convert validation errors into authority issues; never reimplement the operation hash and never synthesize missing identity.

- [x] **Step 3: Make linked status drive every correctness-sensitive decision.**

  Load and validate linked metadata before deciding active/inactive. Treat any CURRENT/linked status mismatch or malformed CURRENT status as fail-closed. Use linked status for active/completed decisions; make `complete_plan`, `reconcile`, Stop, `plan_status`, and operation-bound trace creation consume the same validated authority path without rewriting either file.

### Task 3: Verify preservation and adversarial boundaries

**Files:**
- Modify: `crates/baron-core/tests/plan.rs` only if a missing matrix assertion is discovered.

**Interfaces:**
- Reuse the existing FIX-01/FIX-02 completion evaluator, operation-scoped proof/trace/gate selectors, fresh trace-score evaluation, and legacy diagnostic-only APIs.

- [x] **Step 1: Run all focused plan, automation, proof/trace, identity, and CLI suites.**

  Run `cargo test -p baron-core --test plan --no-fail-fast`, `automation`, `proof_trace`, `operation_identity`, `cargo test -p baron-cli --test phase12_hooks_cli --no-fail-fast`, and `cargo test -p baron-cli --test execution_cli --no-fail-fast`; record exact counts and failures.

- [x] **Step 2: Execute the mandatory regression checklist.**

  Confirm status hiding/mismatch, malformed status, linked risk/task/operation/adapter/session/request validation, partial identity, recomputed tuple behavior, identified/legacy separation, exact A→B bypass rejection, reconcile failure, Stop block, uncompleted plan preservation, post-completion status tamper, and legitimate exact-chain completion.

- [x] **Step 3: Scan production authority readers and document any residual origin limitation.**

  Search `active_plan`, `active_plan_authority`, `active_plan_completion_evidence_status`, `current_plan_risk`, `current_plan_title`, `CURRENT.md`, `load_plan_file_metadata`, `classify_risk`, `task_id_for_task`, and `LifecycleIdentity::from_parts_checked`. Confirm no correctness-sensitive path trusts raw CURRENT lifecycle state or a linked risk/task/operation value without canonical validation. If no independent persisted origin can distinguish a fully recomputed edit of both Markdown files, record that residual limitation explicitly while confirming the required A→B bypass is impossible.

### Task 4: Update evidence and run final verification

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-21-spec-04-canonical-linked-plan-authority-final-fix.md`

- [x] **Step 1: Run Core and workspace tests, then required quality gates.**

  Run `cargo test -p baron-core --all-targets --no-fail-fast`, `cargo test --workspace --all-targets --no-fail-fast`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release --locked -p baron-cli`, `git diff --check`, `target/release/baron --version`, and parse `docs/BARON_STATUS.json`. Record only the known PowerShell Archive and session-replay blockers if they remain unchanged.

- [x] **Step 2: Record current-state evidence without closure.**

  Document linked-plan lifecycle status authority, canonical task/operation/risk checks, status/tamper regressions, the exact reviewed bypass, focused counts, full verification, and any residual origin limitation. Keep the maintained status exactly `SPEC-04: READY FOR FINAL ADVERSARIAL REVIEW` and never write `SPEC-04: CLOSED`.

### Task 5: Review, commit, and push

- [x] **Step 1: Review pre-commit status, full diff, staged diff, and diff check.**
- [x] **Step 2: Stage only the final trust-fix source/tests/docs/plan files.**
- [x] **Step 3: Commit the narrow FIX-A/FIX-B final trust fix.**
- [x] **Step 4: Push `codex/spec-04-proof-trace-gate-completion-integrity`, verify local/remote SHA equality, and confirm a clean worktree.**
