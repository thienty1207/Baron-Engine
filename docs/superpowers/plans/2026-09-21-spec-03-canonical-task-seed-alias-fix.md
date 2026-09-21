# SPEC-03 Canonical Task Authority and Seed Alias Fix Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close only BUG-01 and WARN-01 from the supplied SPEC-03 follow-up prompt without changing SPEC-04, the public version, release metadata, or user-owned files.

**Architecture:** Preserve `LifecycleIdentity` as a checked operation-tuple value for compatibility and comparison, but introduce a stronger authority identity constructible only from canonical task text. Schema-v2 issuance accepts only that stronger value, so parts-only reconstruction cannot load/create a seed, start a child, or append a receipt. Keep operation-ID recomputation in verification and document that task canonicality is guaranteed at trusted issuance while operation canonicality is revalidated at verification. After the validated external authority seed exists, scan only the exact Baron staging-name contract in that authority directory and remove only regular, non-link, exact-32-byte completed aliases; never promote staging material or touch the final seed when the final seed is missing or malformed.

**Tech Stack:** Rust, existing `LifecycleIdentity`/operation hashing, Ed25519 receipt authority, `safe_io`, Cargo unit/integration tests, and the existing cross-process receipt fixtures.

**Source prompt:** `fix-bug/prompt/PROMPT-SPEC-03-CANONICAL-TASK-SEED-ALIAS-FIX.md`

## Global Constraints

- Preserve schema-v1 diagnostic-only execution and signed schema-v2 receipt behavior.
- Preserve the current external authority-root, repo/Vault containment, link/reparse, no-replace activation, strict-loader, cross-process, and receipt-binding guarantees.
- Do not make `LifecycleIdentity::from_parts_checked` impossible for historical reconstruction; it remains comparison-only.
- Do not claim schema-v2 independently proves task canonicality while task text is absent from the signed schema.
- Do not recursively delete, follow links/reparse points, remove arbitrary `.stage` files, remove the final seed, or promote a stale stage when the final seed is absent/malformed.
- Keep `BARON_V5_0_0_DEEP_AGENT_CONTEXT.md`, `fix-bug/`, and other unrelated untracked/user-owned files untouched and unstaged.
- Do not modify Hotel Staff, SPEC-04, public version metadata, tags, releases, or history.
- Update maintained status, build-log, and this plan only after evidence supports each state; final `SPEC-03: CLOSED` is allowed only when all owned closure checks pass.

## Task 1: Establish RED coverage for both trust-boundary defects

**Files:** `crates/baron-core/src/operation.rs`, `crates/baron-core/src/execution_receipt.rs`, `crates/baron-core/src/receipt_authority.rs`, `crates/baron-core/tests/operation_identity.rs`, `crates/baron-core/tests/receipt_authority.rs`, and relevant receipt tests.

- [x] Add an operation-identity contract test showing the exact `task-fake` tuple from the prompt is valid for reconstruction but cannot be converted into an authority-capable identity.
- [x] Add a sentinel execution test proving a reconstructed identity is rejected before authority seed creation/loading, child start, receipt append, or proof/gate mutation.
- [x] Add a positive canonical-path test proving schema-v2 callers receive canonical task/operation IDs; the separate-process proof remains covered by the existing multiprocess verifier fixture.
- [x] Add crash-window seed tests for a valid exact Baron staging alias hard-linked to a valid final seed, unrelated stage-like files, wrong-size exact-pattern stages, a staging-shaped symlink/reparse path, and a missing-final-seed stale stage.
- [x] Run the smallest focused targets; record that host Application Control blocked build-script execution before runnable RED output in `.superpowers/sdd/2026-09-21-spec-03-canonical-task-seed-alias-fix/progress.md`.

## Task 2: Encode canonical task proof in the authority API

**Files:** `crates/baron-core/src/operation.rs`, `crates/baron-core/src/execution_receipt.rs`, `crates/baron-cli/src/main.rs`, and every production/test caller of `execute_command_for_identity`.

- [x] Add `AuthoritativeLifecycleIdentity` whose constructors receive canonical task text and delegate to `LifecycleIdentity::resolve` or validate the supplied task against an existing identity.
- [x] Expose only read-only identity access needed by receipt context and comparison; no conversion from parts-only data can create the stronger type without canonical task text.
- [x] Change schema-v2 `execute_command_for_identity` to accept the stronger type; an unproven identity fails at the wrapper boundary before authority resolution, key load/create, child spawn, and receipt append.
- [x] Migrate the CLI proof path and all authority-bearing tests/callers; diagnostic-only `LifecycleIdentity` and `ReceiptContext` paths remain compatible.
- [x] Retain `operation_id_for_parts` verification and document the issuance/verification split in code comments and maintained records.
- [ ] Run focused operation, execution, receipt-authority, proof, gate, capability/runtime, plan, CLI, and multiprocess tests; record GREEN evidence.

## Task 3: Safely clean completed staging aliases

**Files:** `crates/baron-core/src/receipt_authority.rs`, `crates/baron-core/tests/receipt_authority.rs`, and any focused unit-test module needed for the naming predicate.

- [x] Define the exact generated staging filename contract and inspect only entries in the already validated authority directory.
- [x] After a valid final seed is loaded, remove only matching regular non-link files whose metadata size is exactly 32 bytes and whose bytes match the loaded final seed; fail closed on staging links/reparse points and preserve unrelated names, independently written stages, wrong-size files, directories, and missing races.
- [x] Never run cleanup when the final seed is absent or malformed; stale staging material cannot become a trust root.
- [x] Preserve current first-start cleanup, no-replace hard-link activation, permissions, malformed-final rejection, and concurrent convergence behavior.
- [ ] Run the seed/authority and multiprocess focused suites and record runnable alias-protection evidence.

## Task 4: Fresh adversarial review and maintained records

**Files:** `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `notes/build-log/CURRENT.md`, this plan, and the SDD ledger.

- [x] Search every `LifecycleIdentity::from_parts_checked`, `LifecycleIdentity::new`, `LifecycleIdentity::resolve`, `validate_task`, `execute_command_for_identity`, `ReceiptContext::for_identity`, `operation_id_for_parts`, `task_id_for_task`, `hard_link`, `.stage`, `STAGING_SEQUENCE`, `next_staging_path`, `create_seed_without_replacement`, `load_or_create_for_project`, and `load_existing_for_project` use.
- [x] Classify all authority issuance paths and confirm no reconstructed-only path remains; confirm cleanup has no broad delete, recursion, link following, final-seed removal, or stale-stage promotion.
- [x] Confirm all previously correct SPEC-03 behaviors remain covered and separate unrelated PowerShell Archive, session-replay UTF-8, and Windows Application Control blockers.
- [x] Perform a separate self-review because no reviewer/subagent tool is available; record PASS/WARNING/BUG/OUT-OF-SCOPE rulings in the SDD ledger.
- [x] Update maintained records with the current evidence and keep status `NOT CLOSED` while the new required tests cannot execute.

## Task 5: Full verification, commit, push, and remote proof

- [x] Run the prompt-listed focused tests and mandatory repository commands where the host permits them:
  `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  release build/version smoke where executable, JSON/status parsing, and `git diff --check`.
- [x] Report exact pass/fail/blocker evidence; do not relabel unexpected failures as expected.
- [x] Stage only intended tracked implementation/tests/docs/plan files, commit the follow-up, push `codex/spec-03-trusted-receipt-architecture`, and verify local/remote SHA equality at `994ce18a92eff57be497d3b99420c8bde6078c99`.
- [ ] Mark the plan complete and use final verdict `SPEC-03: CLOSED` only if BUG-01, WARN-01, regression, and verification closure requirements all pass; otherwise use `SPEC-03: NOT CLOSED` with the blocked closure evidence.
