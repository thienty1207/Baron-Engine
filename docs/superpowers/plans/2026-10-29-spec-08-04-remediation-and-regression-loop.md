# SPEC-08-04 Automatic Remediation and Regression Loop — Implementation Plan

> **For agentic workers:** execute this plan task by task with red/green verification. Do not treat historical checkpoints below as current authority.

**Goal:** Implement a Baron-owned, bounded remediation workflow that only considers independently confirmed, in-scope findings, records regression and re-audit claims in order, and cannot report a trusted closure while SPEC-08-05 evidence and sandbox controls are unavailable.

**Architecture:** Keep source auditing read-only and let the existing Superpowers/TDD flow own fixes. Add deterministic Core remediation state contracts over SPEC-08-03 audit records, project the policy through PreparePacket, and expose a bounded schema-only validator for host instructions. The current contract must keep automatic edit authorization and finding closure disabled until SPEC-08-05 supplies trusted evidence and enforced execution safety.

**Tech Stack:** Rust, serde/serde_json, sha2, existing Baron audit workflow, PreparePacket, capability, proof, trace, and continuity contracts. No Node runtime or external scanner.

**Spec:** `docs/specs/Spec8 - Replace VibeSecurity/SPEC-08-04-AUTOMATIC-REMEDIATION-AND-REGRESSION-LOOP.md`

## Approaches considered

1. Let the audit skill edit target source directly. Rejected because SPEC-08-04 assigns evidence gathering and editing to separate roles and Superpowers already owns implementation/TDD.
2. Add a Core-owned bounded remediation contract and keep edits in Superpowers. Selected because it follows the spec, reuses Baron identities and proof/gate authorities, and can safely represent blockers without running target code.
3. Add an external scanner/fix runner. Rejected because it introduces a new execution/runtime boundary and SPEC-08-05 has not established an enforceable sandbox.

## Global Constraints

- Preserve the local uncommitted SPEC-08-02/08-03 work. Do not reset, checkout, commit, push, merge, release, or bump the version.
- SPEC-08-04 depends on SPEC-08-03 and SPEC-08-05. The owner requested 08-04 now; implement deterministic policy/state contracts only and record the sequencing cost. Do not claim the full child is complete until 08-05 trusted evidence and safety integration is available.
- Only schema-confirmed findings can be considered for a remediation plan; `needs_validation` and `rejected` findings are never auto-fixed. Current serialized reviewer/artifact claims are not authenticated and cannot authorize source edits or closure.
- Enforce a maximum of three remediation/re-audit cycles per finding and exact project, operation, source, finding, and authorized-scope binding.
- A regression failure, still-confirmed re-audit, missing capability, disagreement, or product decision stops closure and preserves the finding as open/blocked/incomplete.
- Target-controlled builds, tests, fixtures, scanners, or processes remain `not_run` until SPEC-08-05 establishes an OS-enforced sandbox. Do not mutate any separate target application, project, Vault, or machine configuration.
- Do not let a serialized pass, evidence reference, gate name, proof ID, or trace claim count as trusted execution evidence.

## Review Focus

- A finding with missing, stale, ambiguous, needs-validation, rejected, or unaccepted independent review cannot enter remediation.
- Finding and each cycle bind to the exact original audit run, project, task operation, source/dirty state, finding fingerprint, and authorized scope.
- Changed paths cannot escape the audit's authorized repository scope; empty, oversized, malformed, duplicate, or traversal paths fail closed.
- Cycles are sequential, unique, and capped at three; retry cannot reuse stale source, regression, re-audit, gate, proof, or trace claims.
- Regression cannot pass from command presence or an exit-code string alone; failure/timeout/blocked stays distinct from a negative vulnerability result.
- Re-audit must use the post-fix source and same authorized boundary. “Not confirmed” is only a claim until 08-05 authenticates the evidence.
- Every applied source change records a bounded diff digest; regression and gate results bind to the exact post-fix source. Fix implementers, audit authors/validators, and the independent security reviewer must have distinct identities and fresh review context.
- The independent `security-auditor` gate remains separate from the audit author and fix implementer; it cannot be satisfied by the schema validator.
- Proof and trace cannot be created or claimed by this contract. Current validator output must always state schema-only scope, no execution, no auto-fix authorization, and no trusted closure.
- Bounded CLI input/output rejects unknown fields and must not echo raw run IDs, source, secrets, or finding text.

## Tasks

### Task 1: Core remediation state and eligibility contract

**Files:** Create `crates/baron-core/src/security_audit_remediation.rs`; export it from `crates/baron-core/src/lib.rs`; create `crates/baron-core/tests/security_audit_remediation.rs`.

- [x] Write failing tests for confirmed-only eligibility, exact run/source/task binding, scope-safe changed paths, ordered cycle state, three-cycle cap, and blocker handling.
- [x] Run the focused suite and observe the intended failures.
- [x] Implement bounded versioned request, cycle, status, and validation-result types. Require an accepted independent record review and a schema-confirmed finding, but report the result as an untrusted schema claim.
- [x] Prove a regression failure, unresolved re-audit, or missing capability cannot advance to trusted closure; current `auto_fix_authorized` and `closure_authorized` remain false.

### Task 2: PreparePacket and hidden validator integration

**Files:** Modify `crates/baron-core/src/prepare.rs`; modify `crates/baron-cli/src/main.rs`; extend CLI PreparePacket tests and create/extend security-audit CLI tests.

- [x] Add a remediation policy projection with the three-cycle limit, ordered required steps, and explicit SPEC-08-05 blocker; it must not imply target execution or authorization.
- [x] Add hidden `baron security-audit validate-remediation` for bounded JSON input. It performs schema/workflow checks only, writes no project/Vault files, and never claims an audit, test, gate, proof, trace, or closure passed.
- [x] Test malformed/oversized input, unknown fields, stale bindings, non-confirmed findings, secret-like input non-reflection, and schema-only output.

### Task 3: Runtime Superpowers remediation instructions

**Files:** Modify `assets/core/skills/security-audit/SKILL.md`; extend relevant adapter/Core contract tests.

- [x] Keep the audit read-only and delegate source edits to the existing Superpowers/TDD flow.
- [x] Specify confirmed-only entry, source/data-flow inspection, a regression test that fails before the fix where feasible, minimal safe fix, green regression, targeted re-audit, independent `security-auditor`, and current-operation proof/trace.
- [x] Specify product-decision stop conditions, maximum three cycles, and exact handling of failed, blocked, or unresolved results.
- [x] While SPEC-08-05 is pending, the runtime must preserve `not_run`, leave findings open, and prohibit target-controlled execution and trusted closure.

### Task 4: Status, final review, and verification

**Files:** Update `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, `notes/build-log/CURRENT.md`, the parent roadmap and SPEC-08-04 status, this plan, and the SDD ledger.

- [x] Record the implementation boundary and 08-05 dependency without marking SPEC-08-04 or SPEC-08 overall complete.
- [x] Run focused Core/CLI/adapter suites, `cargo fmt --all -- --check`, `cargo test --workspace --all-targets --no-fail-fast`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Parse status JSON, check relative Markdown links and `git diff --check`, and review the touched diff. Preserve all pre-existing 08-02/08-03 changes; no commit or publication.

## Initial implementation checkpoint (historical)

- Active task: begin SPEC-08-04 in this linked worktree, preserving uncommitted SPEC-08-02/08-03 changes.
- Current checkpoint (2026-10-29): source reconnaissance is targeted to audit workflow, PreparePacket, review gate, proof/trace interfaces, CLI validator, and runtime security-audit skill. Implementation has not started before this checkpoint.
- Proof/trace: none for a user project; no security audit or target-controlled check has run.
- Persisted-state boundary: this linked worktree, plan/ledger, source changes, and Cargo outputs only. Primary checkout, user projects, Vaults, machine configuration, and GitHub are out of scope.
- Sequencing ruling: owner requested 08-04 before its 08-05 dependency. Implement only deterministic fail-closed remediation contracts now; 08-05 remains mandatory before target checks, auto-fix authorization, or closure. Cost if wrong: additive evidence/sandbox integration may be required later.
- Safe next action: execute Task 1 test-first; do not enable target execution or claim SPEC-08-04 complete.

## Current implementation checkpoint — 2026-10-29

- Task 1 Core schema contract, Task 2 PreparePacket/CLI integration, and Task 3 managed skill instructions are implemented locally. Focused Core remediation tests pass 20/20; hidden security-audit CLI tests pass 7/7; the focused adapter managed-runtime contract test passes 1/1. The PreparePacket CLI suite passes 9/9.
- Additional RED/GREEN checks found and closed a cycle-bypass bug: a failed regression or a `not_confirmed` re-audit with a pending independent gate could previously be followed by a new fix cycle. A retry now requires the prior cycle to pass regression and remain `still_confirmed`; stopped/exhausted cycles retain their own blocker.
- Fresh read-only final review reproduced three Important schema gaps: a post-fix digest could refer to another project/task/scope; regression evidence and re-audit run IDs could be replayed across cycles; and the final security reviewer could be an earlier-cycle fixer. Four new tests were observed failing before fixes and pass after them. Cycles now carry a validated structured post-fix binding whose fingerprint is recomputed, evidence/run IDs cannot be reused, and the final reviewer must differ from every fixer in the timeline.
- Windows verification passes: formatter, full workspace tests (116 targets; 1,122 passed, 0 failed, 4 ignored), and warnings-denied Clippy. The full run includes all adapter lifecycle tests. Focused Core/CLI/PreparePacket tests pass 20/20, 7/7, and 9/9; the final public-doc contract suite passes 12/12. Status JSON parses, 69 relative links across 27 changed Markdown files resolve, `git diff --check` reports no whitespace errors, and 47 changed text files have no trailing whitespace.
- SPEC-08-04 remains incomplete as a product workflow: SPEC-08-05 has not provided trusted evidence or an enforced target sandbox. No target project code, fixtures, or build/test commands were run. The schema validator cannot authorize edits, gate execution, proof, trace, or closure.
- Safe next action: implement SPEC-08-05 before audit or remediation execution integration. Preserve the local 08-02/08-03/08-04 worktree; no commits, pushes, merges, or releases were performed.
