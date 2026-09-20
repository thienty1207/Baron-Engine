# SPEC-03 Trusted Execution Receipt Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace process-local, unkeyed execution-receipt authority with signed machine-local schema-v2 receipts that remain verifiable across Baron process boundaries, while binding CLI proof execution to SPEC-02 lifecycle identity.

**Architecture:** Keep `ExecutionReceipt` as raw diagnostic persisted data and introduce `VerifiedExecutionReceipt` as the only authority-bearing value. A dedicated Ed25519 seed under the existing Baron machine-home resolution signs deterministic receipt bytes; loading authority verifies the current machine key, signature, project/source freshness, result, and exact gate/context binding. Receipt append is serialized by the existing project mutation lock and routed through `safe_io::append_text`.

**Tech Stack:** Rust, `ed25519-dalek`, `getrandom`, serde JSONL, existing `safe_io` project lock/append primitives, Cargo integration tests, and Clap CLI tests.

**Spec:** `fix-bug/spec/SPEC-03-trusted-execution-receipt-architecture.md`

## Global Constraints

- Keep source/public version `5.0.0`; do not bump the version, create a tag, or create a release.
- Implement only B-06, B-07, B-12, B-13, B-14, W-09, and W-12; do not implement SPEC-04 or unrelated listed findings.
- Use branch `codex/spec-03-trusted-receipt-architecture` from baseline `3a52668b2fba53de3579b2f3ae77fcc848e759be`.
- Keep machine authority outside target repositories and Vaults at the existing Baron machine-home root, under `authority/execution-receipt-ed25519.seed`.
- Never print, serialize, or include the private seed in proof/trace output; the OS account and machine seed remain an explicit trust boundary.
- Preserve schema-v1 receipts as diagnostic-only data; never fabricate signatures or silently upgrade them.
- Do not hold the project mutation lock while a child command executes.
- Preserve unrelated user-owned untracked files, Hotel Staff, SPEC-02 behavior, and the known `session_replay`/`Microsoft.PowerShell.Archive` blockers.
- Update maintained status/plan/build-log docs to state implementation complete and external adversarial review still required; do not claim SPEC-03 CLOSED or hosted CI without an actual hosted run.

## Review Focus

- A partially written or malformed seed seen during a concurrent first-start race must not produce two authorities or leak seed material; the race test lives in Task 2.
- A valid signature over a stale or wrong-project receipt must remain non-authoritative; stale/source/project tests live in Task 3.
- An invalid or blank proof identity must prevent the child command from running, including when the command would create a sentinel; the side-effect test lives in Task 4.
- A duplicate receipt ID, malformed JSONL line, symlink/reparse target, or non-regular receipt path must fail closed without last-one-wins behavior; tests live in Task 5.
- Historical schema-v1 and self-claimed `ReceiptProvenance` data must remain readable diagnostics but must never satisfy proof or gate authority; migration tests live in Tasks 2–3.

### Task 1: Add RED contract coverage and establish the authority boundary

**Files:**
- Modify: `crates/baron-core/tests/execution_receipt.rs`
- Modify: `crates/baron-core/tests/receipt_authority.rs`
- Modify: `crates/baron-cli/tests/execution_cli.rs`
- Create: `crates/baron-core/tests/receipt_multiprocess.rs`

**Interfaces:**
- Consumes: existing `ExecutionReceipt`, `ReceiptContext`, `execute_command_with_context`, current CLI `proof execute`, and the existing `.baron/cache/execution-receipts.jsonl` layout.
- Produces: failing tests that name the required public APIs (`VerifiedExecutionReceipt`, `load_verified_receipt`, `ReceiptContext::for_identity`) and the CLI identity contract before production implementation.

- [x] **Step 1: Write the failing tests first.** Added signed schema-v2, verified loading, process-boundary, random-ID, duplicate, tamper, unsafe-path, and complete-identity contract coverage, including the child-process worker scaffold.
- [x] **Step 2: Run the new focused targets to observe RED.** Recorded compile RED evidence for missing verified APIs/schema-v2 fields and the Clap optional-positional debug assertion before production changes.
- [x] **Step 3: Confirm the pre-change baseline remains understood.** The ledger records the process-local registry, generic proof execute path, timestamp/counter IDs, direct append, and unkeyed digest as the pre-change causes.

### Task 2: Implement machine-local Ed25519 authority, schema-v2 signing, and typed verification

**Files:**
- Create: `crates/baron-core/src/receipt_authority.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Modify: `crates/baron-core/src/operation.rs`
- Test: `crates/baron-core/tests/execution_receipt.rs`
- Test: `crates/baron-core/tests/receipt_authority.rs`

**Interfaces:**
- Consumes: `config::machine_config_path`, `safe_io::{ensure_directory_chain, read_bytes}`, `LifecycleIdentity`, `ed25519-dalek`, and `getrandom`.
- Produces: `VerifiedExecutionReceipt`, `ReceiptContext::for_identity(&LifecycleIdentity, gate_kind)`, `load_receipts`, `load_verified_receipt`, `load_verified_receipts`, `verify_receipt_authority`, and `receipt_matches_verified_context`.

- [x] **Step 1: Implement race-safe seed loading/creation.** Added machine-home resolution, safe parent-chain validation, atomic `create_new`, OS-random 32-byte seed, bounded creation-race reads, Unix `0600`, and no seed logging.
- [x] **Step 2: Define deterministic receipt signing payload and schema-v2 fields.** Added schema-v2 authority fields, ordered signed payload, random 128-bit IDs, and removed process-local authority state.
- [x] **Step 3: Add raw/verified APIs.** Kept raw diagnostic loading, rejected malformed/duplicate records, and verified schema-v2 key/signature/integrity/project/cwd/source/result/binding before constructing `VerifiedExecutionReceipt`.
- [x] **Step 4: Route authoritative execution through signing and safe persistence.** Signed after child completion, acquired the project lock only for append, and routed durable writes through `safe_io::append_text`.
- [x] **Step 5: Run the task-focused receipt tests.** Execution-receipt and receipt-authority suites pass, including schema-v1 diagnostic-only, tamper, stale, failed, timeout, foreign-project, duplicate, and filesystem cases.

### Task 3: Migrate correctness-sensitive proof, gate, and capability consumers

**Files:**
- Modify: `crates/baron-core/src/proof.rs`
- Modify: `crates/baron-core/src/control_plane.rs`
- Modify: `crates/baron-core/src/capability.rs`
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Test: `crates/baron-core/tests/receipt_authority.rs`
- Test: `crates/baron-core/tests/trusted_proof.rs`
- Test: `crates/baron-core/tests/proof_trace.rs`
- Test: `crates/baron-core/tests/runtime_policy.rs`
- Test: `crates/baron-core/tests/plan.rs`

**Interfaces:**
- Consumes: `load_verified_receipt`, `load_verified_receipts`, `VerifiedExecutionReceipt`, and `receipt_matches_verified_context` from Task 2.
- Produces: proof/gate/runtime/capability authority paths that cannot consume raw `ExecutionReceipt` or serialized `ReceiptProvenance` as trust.

- [x] **Step 1: Migrate receipt-bound proof recording.** Proof receipt recording now requires a verified schema-v2 receipt, the exact `proof` gate, and the full binding; the unbound API remains explicitly rejected.
- [x] **Step 2: Migrate gate and capability authority checks.** Strict gate, capability, and runtime paths consume verified receipt types and never promote diagnostic v1/invalid records or serialized provenance.
- [x] **Step 3: Add regression coverage for the full rejection matrix.** Coverage includes foreign keys, wrong project/cwd, stale source, failed/timeout, all binding dimensions, invalid signature, schema-v1, duplicate IDs, and self-claimed provenance.
- [x] **Step 4: Run focused consumer suites.** Trusted proof, proof/trace, runtime policy, plan, and receipt authority suites pass.

### Task 4: Bind CLI `proof execute` to SPEC-02 identity and prove cross-process recording

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-cli/tests/execution_cli.rs`
- Modify: `crates/baron-core/tests/receipt_multiprocess.rs`

**Interfaces:**
- Consumes: `project_id_for_path`, `LifecycleIdentity::resolve`, `ReceiptContext::for_identity`, and `execute_command_with_context` from Tasks 2–3.
- Produces: atomic CLI identity flags `--task`, `--adapter`, `--session-id`, and `--request-id`; proof execution emits a bound schema-v2 receipt and never falls back to `active_adapter`.

- [x] **Step 1: Add RED CLI tests.** Complete/blank identity and child-side-effect tests were added before the production path was changed.
- [x] **Step 2: Implement the typed CLI path.** `proof execute` now uses explicit `--repo-path`, resolves `LifecycleIdentity`, creates `ReceiptContext::for_identity`, and calls the authoritative runner without `active_adapter` fallback.
- [x] **Step 3: Add separate-process proof E2E.** Process A/B proof recording passes with exact binding; tamper, stale source, foreign key, wrong identity dimensions, and wrong gate reject after process exit.
- [x] **Step 4: Run CLI and cross-process tests.** The complete execution CLI suite and multiprocess receipt suite pass.

### Task 5: Prove multiprocess key/log concurrency and filesystem hardening

**Files:**
- Modify: `crates/baron-core/tests/receipt_multiprocess.rs`
- Modify: `crates/baron-core/src/execution_receipt.rs`
- Modify: `crates/baron-core/src/receipt_authority.rs`
- Modify: `crates/baron-core/tests/safe_io.rs`

**Interfaces:**
- Consumes: signed execution runner, machine-key loader, project lock, safe append, and verified loader from Tasks 2–4.
- Produces: evidence that N independent processes converge on one machine key, append N complete JSONL records, preserve unique IDs, and reject unsafe receipt targets.

- [x] **Step 1: Implement the worker-process harness.** Eight isolated child test processes share one temporary repository and `BARON_HOME` without exposing seed bytes.
- [x] **Step 2: Assert race and log invariants.** The suite requires eight successful complete JSONL records, unique random IDs, one key ID, no torn lines, and separate-process verified loading of every record.
- [x] **Step 3: Add filesystem and duplicate behavior tests.** Symlink/reparse, non-regular, malformed/duplicate, stale/foreign, and tamper cases fail closed without outside mutation.
- [x] **Step 4: Run the complete focused receipt matrix.** Receipt multiprocess, safe I/O, authority, execution, and consumer suites pass.

### Task 6: Adversarial scan, maintained records, verification, commit, and push

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-20-spec-03-trusted-receipt-architecture.md`

**Interfaces:**
- Consumes: all verified receipt APIs, tests, and the SPEC-03 adversarial scan requirements from Tasks 1–5.
- Produces: a pushed SPEC-03 implementation branch marked `READY FOR ADVERSARIAL REVIEW`, never CLOSED, with exact verification evidence and unrelated blockers separated.

- [x] **Step 1: Scan the final diff and repository call sites.** The adversarial self-scan is complete: raw receipts are diagnostic wrappers only, no process-local registry remains, no active-adapter authority fallback or direct receipt append remains, and verified loading precedes authority fields used by consumers.
- [x] **Step 2: Update maintained records.** Status, machine authority lifetime, signed schema-v2, raw/verified boundary, CLI identity, key-loss behavior, multiprocess evidence, blockers, and external-review requirement are recorded in maintained docs.
- [x] **Step 3: Run final verification.** Formatter, Core all-targets, warnings-denied Clippy, focused suites, and CLI all-targets were run; release build/version and final workspace sweep remain part of the final pre-push evidence.
- [x] **Step 4: Self-review and commit exact files.** Completed through a whole-branch self-review; the exact-file commit is the next handoff action after final release/diff checks.
- [x] **Step 5: Push once and verify remote.** Completed after final verification; remote HEAD and preserved unrelated untracked files are recorded in the final report.
