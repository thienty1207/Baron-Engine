# SPEC-03 Final Review Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the five SPEC-03 final-review findings by isolating the machine receipt authority from repository/Vault paths, requiring typed lifecycle identity for signing, hardening first-key activation and race evidence, and making authority-loading semantics explicit.

**Architecture:** Keep raw persisted receipts diagnostic and make `LifecycleIdentity` the only input to authoritative execution. Resolve the machine authority under a canonical external machine home, reject repo/Vault containment and link traversal before filesystem creation, stage a complete seed and activate it with no-replace hard-link semantics, and verify the canonical operation ID before returning `VerifiedExecutionReceipt`. Preserve a separate diagnostic loader while correctness consumers use strict verification.

**Tech Stack:** Rust, `ed25519-dalek`, `getrandom`, existing `safe_io`, serde JSONL, Cargo integration/unit tests, and Clap subprocess tests.

**Spec:** `fix-bug/prompt/PROMPT-SPEC-03-FINAL-REVIEW-FIX.md` and `fix-bug/spec/SPEC-03-trusted-execution-receipt-architecture.md`

## Global Constraints

- Keep source/public version `5.0.0`; do not bump, tag, release, or modify Hotel Staff.
- Implement only FIX-01, FIX-02, WARN-01, WARN-02, and WARN-03 plus the strict bulk-loader classification required by the prompt.
- Stay on `codex/spec-03-trusted-receipt-architecture` from current HEAD `1e81c92ddb8f738ad2894d1d551dab5b7d6ad292`.
- Preserve user-owned untracked files: `BARON_V5_0_0_DEEP_AGENT_CONTEXT.md`, `docs/specs/BARON_TRANSPARENT_UPDATE_COMPATIBILITY_SPEC.md`, and `fix-bug/`.
- Do not fix SPEC-04, session replay UTF-8 behavior, PowerShell archive availability, Hotel Staff, or unrelated B-08/B-09/B-10/B-11/B-15/B-16/B-17/B-18/B-19/B-22/B-23/B-24 findings.
- Push the completed review-fix commits once to the existing GitHub branch after fresh verification.

## Review Focus

- A repo-local, Vault-local, relative, or aliased `BARON_HOME` must fail before the child process and leave no active seed.
- A first-start crash or staging failure must never leave a partial final seed that blocks later recovery; a valid existing seed must never be replaced.
- An arbitrary `ReceiptContext` must not issue schema-v2 authority; an identity with a forged operation tuple must fail verification even with a valid signature.
- Concurrent empty-home workers must cross a real ready/release barrier before first authority creation, then converge on one key and complete records.
- A valid receipt plus an invalid schema-v2 record must not be silently reduced to a valid-only authority set by correctness-sensitive consumers.

### Task 1: Add RED coverage for final-review findings

**Files:**
- Modify: `crates/baron-core/tests/receipt_authority.rs`
- Modify: `crates/baron-core/tests/execution_receipt.rs`
- Modify: `crates/baron-core/tests/receipt_multiprocess.rs`
- Modify: `crates/baron-cli/tests/execution_cli.rs`
- Modify: `crates/baron-core/src/execution_receipt.rs` (unit test for valid-signature operation mismatch)

**Interfaces:**
- Consumes current `ReceiptAuthority`, `ReceiptContext`, `execute_command_with_context`, `LifecycleIdentity`, raw/verified receipt loaders, and the existing child-process worker harness.
- Produces failing tests for external authority-root enforcement, typed signing, canonical operation verification, strict loading, staged seed activation, and deterministic first-start race.

- [x] **Step 1: Add scope and fail-before-child tests.** Set `BARON_HOME` to repo root, repo descendants, `.baron`, a relative repo alias, known Vault root, and Vault descendants; use a child sentinel and assert the sentinel is absent. Add an external machine-home success case and normal `HOME`/`USERPROFILE` success case.
- [x] **Step 2: Add seed lifecycle tests.** Cover zero/short/oversized final seed rejection, seed and authority-parent links/reparse rejection, a blocked staging path leaving no final seed, and owner-only Unix permissions where supported.
- [x] **Step 3: Add typed-boundary and operation tests.** Assert the legacy arbitrary-context path cannot produce verified schema-v2 authority, add a valid-signature forged-operation fixture in the execution-receipt unit tests, and retain a valid `LifecycleIdentity::resolve` cross-process path.
- [x] **Step 4: Add strict loader and barrier tests.** Add a mixed valid/invalid schema-v2 log test that requires strict failure, retain a diagnostic-only filtering assertion, and change the multiprocess harness to ready/release barrier coordination before any worker loads or creates authority.
- [x] **Step 5: Run RED targets.** Run the affected focused targets and record failures caused by the missing new APIs/behavior, not test syntax or unrelated blockers.

### Task 2: Enforce external machine authority scope and crash-safe seed activation

**Files:**
- Modify: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/receipt_authority.rs`
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Test: `crates/baron-core/tests/receipt_authority.rs`

**Interfaces:**
- Consumes `machine_config_path`, `safe_io::{ensure_directory_chain, read_bytes}`, configured/local/default Vault resolution, and canonical repo paths.
- Produces `ReceiptAuthority::{load_or_create_for_project, load_existing_for_project, verify_for_project}` with canonical scope validation and a complete-seed no-replace activation path.

- [x] **Step 1: Add an optional configured-Vault resolver.** Return a Vault only when `BARON_VAULT`, repo-local `.baron/local.toml`, or an existing machine config supplies one; return `None` when no configured/default Vault is available and never invent a path.
- [x] **Step 2: Implement canonical scope validation.** Resolve relative machine-home paths against the process working directory, canonicalize existing ancestors for alias comparison, reject equality/containment with canonical repo or known Vault, and validate every authority parent component with safe link/reparse checks before creation.
- [x] **Step 3: Replace direct final-seed creation.** Generate 32 bytes with OS CSPRNG, write/flush/sync/permission the complete seed to a unique same-directory staging file, create the final path with atomic no-replace hard-link activation, remove staging only after activation, and converge on an existing creator's seed without overwriting or deleting malformed final data.
- [x] **Step 4: Route signing and verification through project scope.** Canonicalize the execution repo before authority load, pass the known Vault to scoped authority methods, and make missing/malformed/foreign/link targets fail closed with bounded diagnostics.
- [x] **Step 5: Run seed/scope tests GREEN.** Run `cargo test -p baron-core --test receipt_authority --no-fail-fast` and the relevant unit test; all new scope, seed, staging, and link cases must pass.

### Task 3: Make canonical `LifecycleIdentity` the signing boundary

**Files:**
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-core/src/proof.rs`
- Modify: `crates/baron-core/src/control_plane.rs`
- Modify: `crates/baron-core/src/capability.rs`
- Modify: authority-bearing test files that currently call `execute_command_with_context`

**Interfaces:**
- Consumes `LifecycleIdentity::resolve`, `ReceiptContext::for_identity`, `SupportedAdapter::parse`, and the scoped authority API from Task 2.
- Produces `execute_command_for_identity(request, &identity, gate_kind)` as the only schema-v2 authority emitter; the old context API becomes explicitly diagnostic or is removed from authority call sites.

- [x] **Step 1: Add the typed execution API and diagnostic split.** Build `ReceiptContext` internally from `LifecycleIdentity`, verify identity project matches the canonical request repo, and make arbitrary `ReceiptContext` execution schema-v1 diagnostic-only or reject it without issuing authority.
- [x] **Step 2: Migrate production callers.** Change CLI proof execution and every correctness-sensitive production caller to typed identity; classify remaining context uses as expected comparison/diagnostic compatibility only.
- [x] **Step 3: Recompute canonical operation IDs during verification.** Parse the signed adapter, call `operation_id_for_parts(receipt.project_id, receipt.task_id, adapter, receipt.session_id, receipt.request_id)`, reject mismatch before authority construction, and do not attempt task-text recomputation because schema-v2 has no canonical task text.
- [x] **Step 4: Migrate authority tests and add forged-operation coverage.** Use `LifecycleIdentity::resolve` for issuing receipts, preserve comparison contexts for negative cases, and ensure the valid-signature forged-operation unit test fails specifically on canonical operation mismatch.
- [x] **Step 5: Run typed execution/proof consumers GREEN.** Run execution receipt, trusted proof, proof trace, runtime policy, plan, and CLI execution targets and inspect all output.

### Task 4: Make strict authority loading and deterministic multiprocess race explicit

**Files:**
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Modify: `crates/baron-core/src/capability.rs`
- Modify: `crates/baron-core/src/control_plane.rs`
- Modify: `crates/baron-core/tests/receipt_authority.rs`
- Modify: `crates/baron-core/tests/receipt_multiprocess.rs`

**Interfaces:**
- Consumes verified receipt construction and project-scoped verification from Tasks 2–3.
- Produces `load_verified_receipts_strict` for correctness paths and a separately named diagnostic-valid-record loader; the ready/release worker harness proves true first-start contention.

- [x] **Step 1: Split bulk-loading semantics.** Strict loading skips only known schema-v1 diagnostic records and returns an error for malformed/unknown schema, wrong-key, invalid-signature, stale, or binding-invalid schema-v2 records; filtering remains only under explicitly diagnostic APIs and the distinction is documented.
- [x] **Step 2: Migrate strict consumers.** Gate, capability, runtime, and other correctness-sensitive consumers must call strict loading; diagnostic/reporting paths may use the filtering API but never return authority from raw data.
- [x] **Step 3: Add the process barrier.** Each worker creates a unique READY marker, blocks until a parent-created RELEASE marker, then performs first authority initialization. The parent waits for all N READY markers before releasing and asserts one final 32-byte seed/key ID and N complete signatures.
- [x] **Step 4: Run strict/race tests GREEN.** Run receipt authority and multiprocess targets, including the mixed-log strict rejection and separate-process verifier.

### Task 5: Correct semantics, update maintained records, review, commit, and push

**Files:**
- Modify: `crates/baron-core/src/receipt_authority.rs`
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Modify: this plan

**Interfaces:**
- Consumes all verified APIs and test evidence from Tasks 1–4.
- Produces accurate machine-local cross-process comments, explicit raw/strict/diagnostic semantics, final adversarial classifications, and a pushed review-fix branch only if all owned closure requirements pass.

- [x] **Step 1: Correct comments and provenance documentation.** State that signed machine-local receipts survive process boundaries while signature/key/freshness/binding remain valid; document `ReceiptProvenance` as diagnostic metadata only. The final pass also canonicalizes existing Vault ancestors so symlink aliases cannot bypass the scope boundary.
- [x] **Step 2: Run repository-wide trust scans.** Classify every raw/verified receipt use, authority-root path, operation-ID check, seed activation, and invalid-record behavior as PASS/WARNING/BUG/OUT-OF-SCOPE; no SPEC-03-owned BUG or WARNING remains.
- [x] **Step 3: Run full verification.** Run all prompt-listed focused targets, `cargo fmt --all -- --check`, Core all-targets, workspace all-targets, Clippy with warnings denied, release build/version, JSON parsing, and `git diff --check`; report the baseline PowerShell/UTF-8 blockers and host Application Control execution blocks separately.
- [x] **Step 4: Perform separate self-review if no reviewer tool is available.** Read the final diff against the prompt checklist, record the self-review rulings in the ledger, and confirm that no owned BUG/WARNING remains. No reviewer/subagent tool was available in this session.
- [x] **Step 5: Stage exact files, commit, push once, and verify remote.** Preserve unrelated untracked files; update maintained status only after evidence; use final verdict `SPEC-03: CLOSED` only when every closure item is proven, otherwise `SPEC-03: NOT CLOSED` with exact owned issues. Implementation/review-fix commit: `2079379`.
