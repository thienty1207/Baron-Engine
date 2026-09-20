# SPEC-02 Final Review Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the three identified SPEC-02 review gaps without changing the canonical v2 identity algorithms or expanding into SPEC-03/04.

**Architecture:** Make every public `LifecycleIdentity` reconstruction recompute and verify the operation tuple, while task-aware lifecycle entry points additionally verify the task-text derivation. Make the request compatibility helper fail closed on absent identity components. Preserve anonymous hook deliveries as distinct operations and document/test that contract because current Codex/Claude bridge payloads provide no stable delivery identifier.

**Tech Stack:** Rust, Cargo tests, serde JSON, Markdown status/build records.

**Spec:** `fix-bug/prompt/PROMPT-SPEC-02-FINAL-REVIEW-FIX.md`

## Global Constraints

- Keep branch `codex/spec-02-operation-identity`.
- Preserve `baron-task-v2`, `baron-operation-v2`, and public version `5.0.0`.
- Do not modify Hotel Staff, create tags/releases, or implement SPEC-03/SPEC-04.
- Do not stage the user-owned untracked files already present in the worktree.

## Review Focus

- Forged operation IDs must fail before plan writes; regression tests cover reconstruction, context conversion, and plan no-write behavior.
- Task IDs must be checked against canonical task text wherever a lifecycle entry point has the task; regression tests cover plan title mismatch.
- Missing or blank request/session IDs must never produce an authority-bearing operation ID; regression tests cover all incomplete combinations.
- Identified hook retries must remain byte-stable and deduplicated; anonymous identical deliveries must remain distinct and explicitly non-idempotent.
- Codex and Claude payload handling must not claim an unsupported delivery key; compatibility documentation records the evidence and limitation.

### Task 1: Enforce canonical identity reconstruction

**Files:**
- Modify: `crates/baron-core/src/operation.rs`
- Modify: `crates/baron-core/src/plan.rs`
- Test: `crates/baron-core/tests/operation_identity.rs`
- Test: `crates/baron-core/tests/plan.rs`

- [x] Add checked parts reconstruction and task-aware context validation tests, observe RED.
- [x] Make `LifecycleIdentity::new` checked, add `from_parts_checked`, and reject forged operation tuples.
- [x] Validate task IDs against canonical title text before any plan write.
- [x] Migrate tests to canonical identities and run operation/plan suites.

### Task 2: Make request operation derivation fail closed

**Files:**
- Modify: `crates/baron-core/src/prepare.rs`
- Test: `crates/baron-core/tests/operation_identity.rs`

- [x] Add tests for present, missing, and blank request/session components, observe RED.
- [x] Return `Result<String, OperationIdentityError>` and validate both components before derivation.
- [x] Confirm repository callers are migrated or absent and run focused tests.

### Task 3: Specify anonymous hook delivery semantics

**Files:**
- Modify: `crates/baron-core/src/automation.rs`
- Modify: `crates/baron-core/tests/phase12_hooks.rs`
- Modify: `docs/compatibility/CODEX.md`
- Modify: `docs/compatibility/CLAUDE.md`

- [x] Rename/comment the anonymous hook regression to state intentional non-idempotence and verify the existing behavior.
- [x] Document the identified retry contract and the absence of a stable host delivery key in both adapter compatibility documents.
- [x] Run Codex/Claude hook parity and automation suites.

### Task 4: Adversarial closure and maintained records

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`

- [x] Run repository-wide SPEC-02 call-site scans and required adversarial scenarios; record PASS/WARNING/BUG/OUT-OF-SCOPE classifications.
- [x] Run focused verification, formatting, full workspace tests, clippy, release build, diff check, and version check.
- [x] Update maintained records only if no SPEC-02-owned issue remains, then commit exact files and push the branch.

## Completion evidence

- Canonical identity reconstruction now rejects forged operation IDs and checks
  task-text derivation before Plan writes. The empty CLI session/request guard
  was also found and fixed during the adversarial scan.
- `operation_id_for_request` fails closed for missing or blank identity fields;
  no production callers required migration.
- Identified hook retries remain idempotent. Anonymous deliveries remain
  intentionally distinct because the current Codex/Claude bridge payloads have
  no stable delivery key.
- Focused suites passed: Core operation identity `8/8`, Plan `12/12`, hooks
  `11/11`, automation `4/4`, Prepare `8/8`, context compiler `19/19`, trusted
  memory `4/4`, control plane `9/9`, CLI plan identity `3/3`, execution `9/9`,
  and control-plane CLI `2/2`.
- `cargo fmt --all -- --check`, workspace Clippy with `-D warnings`,
  `cargo build --release --locked -p baron-cli`, `baron 5.0.0`, and
  `git diff --check` passed. The full workspace run retained only the known
  `Microsoft.PowerShell.Archive` installer blocker and the existing
  `session_replay` UTF-8 boundary panic.
- Adversarial closure scan found no remaining SPEC-02-owned issue. Scope stays
  limited to SPEC-02; no version bump, tag, release, Hotel Staff, SPEC-03, or
  SPEC-04 work was performed.
- Final review: self-review (no subagent tool); no Critical or Important issue
  remained after the review pass.
