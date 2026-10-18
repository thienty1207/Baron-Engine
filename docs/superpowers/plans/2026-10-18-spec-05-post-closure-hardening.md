# SPEC-05 Post-Closure Hardening and Main Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify H-01/H-02/H-03 on the accepted SPEC-05 source, repair only demonstrated contract violations, and prepare reviewed PR integration into `main` without changing public version `5.0.0` or fabricating missing project/Vault identity.

**Architecture:** Reuse the existing hash- and owner-checked plan transition journal for ordinary progress publication; keep checkout ownership path-bound and fail closed across relocation; keep hook dedup memory bounded while preserving in-flight claims and rejecting malformed journal evidence in the inspected tail. Record the finite replay window explicitly.

**Tech Stack:** Rust workspace, `safe_io` atomic replacement and project locks, serde/JSONL, Cargo, GitHub Actions.

**Spec:** User-supplied `D:\Working\Baron-Engine\docs\specs\SPEC-05-POST-CLOSURE-HARDENING-AND-MAIN-INTEGRATION.md` (preserved in the primary checkout; currently untracked there).

## Global Constraints

- Public version remains `5.0.0`; no release, tag, version bump, force-push, or history rewrite.
- Do not run `baron init`, generate a Project ID, or create/replace the original Vault or capsule during recovery.
- Use only temporary repository/Vault fixtures with unrelated test identities for tests.
- Preserve project identity, identity binding, checkout owner fencing, lock order (checkout then shared Vault), and fail-closed behavior.
- Do not modify the user-provided `docs/current/current.md`; keep it in the primary checkout and out of the PR.
- Do not add SPEC-06 transaction scope or modify Hotel Staff, manifests, or `Cargo.lock` without evidence that the spec requires it.
- PR source is the full SPEC-05 lineage plus scoped hardening; do not merge until required checks pass and the user approves the reviewed integration scope.

## Review Focus

- Local plan progress may be published before its Vault mirror; test every journal recovery boundary and verify two-operation isolation.
- Retrying the same interrupted progress note may append it twice; assert one persisted progress effect after restart and repeated retry.
- Moving a checkout changes its path-derived owner; test that the move fails closed and leaves shared Vault bytes unchanged, while canonical aliases retain the same owner.
- Hook cache trimming may evict a live claim; test that the claim remains fenced and that a full cache of live claims rejects another claim without exceeding the bound.
- Malformed JSONL in the bounded lookup tail may be skipped; test that hook processing fails before continuity or journal publication.

---

### Task 1: Journal ordinary plan progress updates

**Files:**
- Modify: `crates/baron-core/src/plan.rs`
- Modify: `crates/baron-core/src/safe_io.rs` (test-only path-scoped interruption helper, if needed)
- Test: `crates/baron-core/src/plan.rs` unit tests

**Interfaces:**
- Consumes: existing `PlanTransitionJournal`, `recover_pending_plan_transition`, `plan_content_hash`, `ensure_active_plan_vault_index_consistent`, and checkout-then-Vault lock order.
- Produces: ordinary progress publication that persists a recoverable intent before touching local plan bytes; recovery validates source/mirror hashes, owner, binding, generation, and target hash before projecting; an immediate retry of the same note is idempotent.

- [x] **Step 1: Write the deterministic failure test first.** Add a test-only, one-shot fault point scoped to a temporary repo. Run the interrupted update in a child test process for each boundary: after durable intent but before local publication, after local plan publication, after Vault mirror publication, and after ACTIVE publication but before CURRENT. Start identified operations A and B, leave CURRENT pointing at B, then let the parent process reopen the Vault context and retry A twice. Assert CURRENT still points to B at each injected failure, one persisted progress note after recovery/retry, byte-identical local/Vault plan mirrors, B plan and ACTIVE entry unchanged, and no false completion.
- [x] **Step 2: Run the new test and verify RED.** Run `cargo test -p baron-core --lib plan::tests::identified_progress_update_recovers_idempotently_at_all_publication_boundaries -- --exact --nocapture`. Confirm the output says exactly one test ran. Expected: existing progress writes have no pending journal; local/Vault divergence blocks recovery or the repeated note is appended twice.
- [x] **Step 3: Implement the smallest safe recovery protocol.** Publish ordinary progress through the existing `.baron/plan-transition.json` path; reuse the existing hash/owner validation and replay code. Make the last identical progress note idempotent so a resumed retry does not append it again. Keep the fault point test-only.
- [x] **Step 4: Run focused plan and concurrency tests.** Run `cargo test -p baron-core --lib --test plan --test concurrency --no-fail-fast -j 1`. Expected: all library unit, authority, stale-mirror, foreign-owner, timeout, and new recovery tests pass.
- [x] **Step 5: Commit the task.** Commit only the plan recovery code and tests.

### Task 2: Pin checkout relocation as a fail-closed contract

**Files:**
- Modify: `crates/baron-core/tests/plan.rs`
- Modify: `docs/architecture/ADAPTERS.md` or the maintained plan/continuity contract section that best owns checkout rehoming.

**Interfaces:**
- Consumes: existing canonical checkout owner hash, project identity binding, plan frontmatter, local/Vault ACTIVE rows, and exact mirror comparison.
- Produces: a regression showing that moving the same checkout cannot silently claim the old owner; documented operator recovery restores the original canonical path or defers rehoming to an explicit audited migration.

- [x] **Step 1: Add `moving_checkout_fails_closed_without_changing_shared_plan_authority`.** Create an identified plan, move the fixture checkout, resolve the same persisted project identity and Vault, and attempt exact resume. Assert owner mismatch, no local plan mutation, and byte-identical shared Vault plan/ACTIVE. The existing Core unit test asserts canonical symlink aliases retain the same owner where supported.
- [x] **Step 2: Run the relocation test.** Run `cargo test -p baron-core --test plan moving_checkout_fails_closed_without_changing_shared_plan_authority -- --exact --nocapture`. Expected: moved path is rejected; canonical alias resolves to the same owner when supported.
- [x] **Step 3: Document the recovery boundary.** State that automatic owner reassignment is unsupported; restoring the checkout at its original canonical path preserves authority. A new-path rehome requires a future explicit operation that verifies project/capsule identity and both ACTIVE/frontmatter copies under both locks.
- [x] **Step 4: Commit the task.** Commit only the contract test and its documentation.

### Task 3: Preserve active hook claims and fail closed on malformed replay evidence

**Files:**
- Modify: `crates/baron-core/src/automation.rs`
- Modify: `crates/baron-core/tests/phase12_hooks.rs` or `crates/baron-core/src/automation.rs` unit tests
- Test: automation dedup and JSONL replay tests

**Interfaces:**
- Consumes: `MAX_DEDUP_ENTRIES = 256`, `MAX_JOURNAL_SCAN_LINES = 512`, `DEDUP_CLAIM_TTL_SECONDS = 300`, `journal_response`, `journal_contains_key`, shared Vault lock order, and existing claim-token fencing.
- Produces: response entries may be evicted to make room, but live claims are never evicted; a new claim fails closed if 256 entries are all live claims; malformed non-empty rows in the inspected JSONL tail return an error before hook side effects.

- [x] **Step 1: Write failing tests.** Added `live_hook_claim_survives_response_cache_trimming`, `dedup_capacity_of_live_claims_fails_closed`, and `malformed_journal_tail_blocks_hook_before_publication`. Extended the real 280-delivery retention stress test to replay E after 279 later hook events, and added child-process replay after 513 later journal rows with an evicted response cache; both assert continuity checkpoint immutability and authoritative journal counts.
- [x] **Step 2: Run those tests and verify RED.** The live-claim test failed because trimming removed the live claim; the capacity test failed because a full set of live claims accepted another claim; malformed-tail first failed because the hook continued, then a stronger correlated-prompt assertion failed because mapping publication preceded strict parsing. The pre-fix finite-window characterization passed and showed journal recovery within the window and a possible second journal row beyond both windows.
- [x] **Step 3: Implement bounded safe retention.** Trim completed responses first, then expired claims; keep non-expired and malformed/future claims; stage candidate cache state and fail without mutation if it cannot fit. Parse every non-empty row in the latest 512 journal lines strictly, and validate the tail before hook correlation or publication.
- [x] **Step 4: Define and test replay semantics.** Documented that response-cache and journal replay evidence are bounded to 256 entries and 512 lines; after both windows expire, replay may re-run and append another event row. Tests verify no duplicate publication in-window, one immutable operation checkpoint after out-of-window replay, and Stop replay reconciliation.
- [x] **Step 5: Run focused hook and concurrency tests.** `cargo test -p baron-core --lib --test automation --test phase12_hooks --test hook_identity --test concurrency --no-fail-fast -j 1` completed with 140 passed, 0 failed across 5 suite summaries, including Stop replay and shared-Vault concurrency.
- [x] **Step 6: Commit the task.** Commit the dedup implementation, tests, and replay contract documentation.

### Task 4: Record post-closure status and verification evidence

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Create: `docs/superpowers/plans/2026-10-18-spec-05-post-closure-hardening-verification.md`

- [x] **Step 1: Update maintained status.** Preserved the SPEC-05 historical CLOSED range; recorded H-01/H-02/H-03 outcomes, branch/source checkpoint, proof/trace status, persisted-state boundary, and next safe action using verified facts.
- [x] **Step 2: Run doc gates.** Status JSON parses; relative links in five changed Markdown files have 0 broken targets; `git diff --cached --check` passes; CLI smoke prints `baron 5.0.0`; retired-adapter content and filename gates have no tracked matches.
- [x] **Step 3: Commit the evidence.** Committed the maintained status, plan/log updates, and follow-up verification ledger as `3fe6354`.

### Task 5: Verify source and review the complete integration scope

**Files:**
- No source changes unless a test or review finding requires a separately tested fix.

- [x] **Step 1: Run required local gates.** Core, CLI, adapters, and workspace all-target tests pass; fmt, warnings-denied Clippy, locked release build, version/help smoke, and spec-specific focused checks pass. Lifecycle scripts used bundled PowerShell 7; the final Core all-target rerun used a temporary BARON_HOME.
- [ ] **Step 2: Verify product constraints.** Confirm `baron --version` is `5.0.0`; no secrets, machine config, `.env`, Vault data, generated artifacts, or `docs/current/current.md` are staged; compare the entire hardening head to `origin/main` and inspect all 82 pre-existing commits / 98 paths and any added files.
- [ ] **Step 3: Obtain an independent whole-branch review.** Review the exact source range and this plan/spec; resolve Critical/Important findings with RED→GREEN tests, record deferred Minor findings, and rerun the affected suites.

### Task 6: Prepare PR integration into `main`

**Files:**
- No additional source changes after final review unless a PR-head failure requires a new verified fix.

- [ ] **Step 1: Commit and push the reviewed hardening branch** after checking exact diff, clean staging, credentials, and remote refs.
- [ ] **Step 2: Open a PR** from `fix/spec-05-post-closure-hardening` to `main`, including the accepted SPEC-05 source boundary, hardening classifications, full integration scope, local evidence, identity recovery status, and hosted checks.
- [ ] **Step 3: Verify PR-head CI and mergeability.** Wait for Format/Clippy, Windows, Linux, release-build, and version-smoke checks on the actual candidate SHA. Stop before merge until the user approves the reviewed 82-commit / 98-path integration scope.
- [ ] **Step 4: After approval, merge through the supported PR mechanism and verify push-to-main CI on the exact merge SHA.** Update final docs only with observed PR, review, merge, and CI evidence.
