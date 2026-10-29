# SPEC-08-03 Audit Workflow and Finding Quality — Implementation Plan

> **For agentic workers:** execute this plan task by task with red/green verification. Do not treat historical checkpoints below as current authority.

**Goal:** Implement Baron-owned, versioned audit-run, coverage, finding, validation, and report contracts that enforce SPEC-08-03's six-phase workflow and honest findings.

**Architecture:** Add deterministic Rust Core models and validators for exact run binding, coverage-led phases, candidate verification, structured verdicts, independent record verification, and target-neutral reports. Expose a hidden read-only CLI validator for host projections; retain Core's existing PreparePacket as the route and capability authority. Use Baron continuity checkpoints for resumable progress. Do not execute target-controlled code, create a second security gate, write Vault truth from candidates, or enable audit execution before SPEC-08-05.

**Tech Stack:** Rust, serde/serde_json, sha2, existing Baron continuity and Task State APIs. No Node runtime or external scanner.

**Spec:** `docs/specs/Spec8 - Replace VibeSecurity/SPEC-08-03-AUDIT-WORKFLOW-AND-FINDING-QUALITY.md`

## Global Constraints

- Preserve the in-progress, uncommitted SPEC-08-02 work in this linked worktree. No reset, checkout, commit, push, merge, release, or version bump.
- Owner requested SPEC-08-03 before SPEC-08-05 although both the child spec and parent roadmap list SPEC-08-05 as a dependency/earlier step. Implement deterministic contracts, validation, phase bookkeeping, reports, and continuity projection only. Keep target audit execution `not_run` and blocked until 08-05 supplies enforced sandbox, recovery, and safe execution capabilities.
- Bind every run to project ID, exact source revision and dirty-state fingerprint, authorized scope, language set, profile, task ID, and Task State revision. Stale-source findings must not be confirmed without fresh evidence.
- A run may not skip one of six phases. The candidate author cannot validate their own finding; independent record verification must be distinct from both author and candidate verifier.
- `needs_validation` has an exact unresolved fact and blocker and no severity. `rejected` preserves the claim and disproof. Only validated records can enter a final report.
- Duplicate consolidation uses an explicit root-cause key and boundary evidence so different causes on the same line remain distinct.
- Report coverage includes covered, skipped, deferred, and out-of-scope units, plus exact residual checks and limitations. No scan/fixture presence counts as execution evidence.
- Use existing Task State/continuity and proof/gate/trace authorities. Audit progress is operational state, not trusted Vault memory. The required `security-auditor` gate remains independent.
- Do not change or execute target repositories, fixtures, scanners, or third-party code. Tests may exercise Baron-owned fixtures/contracts and temporary test data only.

## Review Focus

- Reject path traversal, malformed identities/revisions, duplicate/missing coverage units, unsupported schema versions, unbounded fields, and stale source bindings.
- Every phase requires a schema-bound exact-scope/revision receipt and phases cannot be skipped. In this dependency-limited implementation, receipts and reviewer IDs are unverified serialized claims; they cannot establish execution, proof, a gate, or a trace until SPEC-08-05 integrates the trusted evidence boundary.
- Verify independent author/verifier/final-reader identities and fresh context IDs, including Deep-mode review minimums.
- Candidate and independent record reviews must bind to the complete run digest, including dirty-state and Task State hashes; same-HEAD stale reviews must fail.
- Prove `needs_validation` cannot serialize or validate with severity, and that a rejected record includes evidence that refutes the claim.
- Confirmed source lines must remain within the submitted candidate range and match its evidence. A rejection must preserve its exact claim and cite relevant evidence within authorized scope.
- Run IDs must use a canonical UUID token; CLI and continuity output must never reflect the raw ID.
- Ensure stable deduplication merges only an identical root-cause/boundary fingerprint and preserves all author/evidence references.
- Ensure final report is derived from a validated run and preserves residual gaps; a completed audit can contain findings, while task gate state remains independent.
- Confirm PreparePacket remains `not_run` with the SPEC-08-05 safety blocker and Codex/Claude see the same Core contract.
- Ensure hidden CLI validation reads bounded JSON only, writes no target files, and never echoes supplied secrets or source contents.

## Sequencing Ruling

SPEC-08-03 explicitly depends on SPEC-08-05, and SPEC-08-00 orders 08-05 before 08-03. The owner selected 08-03 now. Proceed only with deterministic workflow contracts that do not make target execution possible. Keep safety/recovery blockers authoritative. Cost if this sequencing is wrong: once 08-05 is implemented, its persisted-state and sandbox requirements may require additive changes to this schema/continuity projection; no unsafe audit run is authorized by this plan.

## Tasks

### Task 1: Run binding, phase, and coverage ledger contracts

**Files:** Create `crates/baron-core/src/security_audit_workflow.rs`; export from `crates/baron-core/src/lib.rs`; add `crates/baron-core/tests/security_audit_workflow.rs`.

- [x] Write failing tests for schema version, complete run binding, safe relative scope, unique coverage IDs, required dimensions, and each coverage disposition.
- [x] Observe focused failures before adding implementation.
- [x] Implement bounded, deterministic run/coverage types and validators; enforce exact revision and Task State binding.
- [x] Test invalid/missing ledger fields and duplicate IDs fail closed.

### Task 2: Candidate, verdict, validation, and deduplication contracts

**Files:** `crates/baron-core/src/security_audit_workflow.rs`, focused integration tests.

- [x] Write failing tests for confirmed evidence requirements, author/verifier separation, `needs_validation` without severity, rejected claim/disproof, stale revisions, and deterministic duplicate handling.
- [x] Implement tagged verdict types, fresh verifier records, independent record-review records, bounded validation, and root-cause-aware stable fingerprints.
- [x] Prove distinct root causes remain distinct even when path, line, and attack class match.

### Task 3: Six-phase state machine and report derivation

**Files:** workflow Core module and integration tests.

- [x] Write failing transition tests for skipped/out-of-order phases, missing ledger, missing candidate review, missing independent record review, and incomplete report residuals.
- [x] Implement exact sequential transitions with phase receipts bound to run scope/revision/operation. Artifact existence and authenticity remain outside this schema-only contract.
- [x] Derive reports from schema-valid run state, retaining full coverage, unresolved candidate context, verdicts, residual checks, prior-run limits, and incomplete/blocked state.
- [x] Keep execution state distinct from audit result and Baron application-task gate/proof/trace; receipts remain explicitly unverified.

### Task 4: Core projections, resumability boundary, and hidden validator CLI

**Files:** `crates/baron-core/src/prepare.rs`, `crates/baron-cli/src/main.rs`, CLI integration tests, and `assets/core/skills/security-audit/SKILL.md`.

- [x] Write failing tests for PreparePacket workflow schema/phases, unchanged `not_run` and safety blocker, and CLI bounded read-only validation.
- [x] Add additive workflow metadata to PreparePacket; remove only the 08-03-pending blocker after the deterministic contracts are available, retaining 08-05 and inventory blockers.
- [x] Add a hidden host-facing validator command for versioned run JSON. It reads bounded stdin, validates deterministically, prints a compact result, and does not modify files or execute target code.
- [x] Specify continuity checkpoint summaries using exact operation identity; do not persist findings or promote candidate output as trusted memory.
- [x] Update the runtime entrypoint so Codex and Claude follow the same six phases and require the Core schema/workflow validator; keep target execution `not_run` until 08-05.

### Task 5: Status, review, and verification

**Files:** SPEC-08-03 status, `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, `notes/build-log/CURRENT.md`, this plan, and SDD progress ledger.

- [x] Record final implemented scope, dependency ruling, tests, proof/trace state, and uncommitted boundary.
- [x] Run focused Core/CLI suites, `cargo fmt --all -- --check`, `cargo test --workspace --all-targets --no-fail-fast --quiet`, and `cargo clippy --workspace --all-targets -- -D warnings` on the final source diff.
- [x] Parse status JSON, check repository-relative Markdown links, inspect the SPEC-08-03 changes through focused independent review, and ensure 08-02 work is still present.
- [x] Mark only verified tasks complete. No commit, push, merge, release, target audit, or clean-audit claim.

## Implementation Checkpoint

- Active task: the deterministic workflow/schema portion of SPEC-08-03 is locally verified after locally verified but uncommitted SPEC-08-02 in the same linked worktree.
- Current checkpoint (2026-10-29): deterministic 08-03 schema, workflow, PreparePacket, validator, report, continuity, and skill changes are implemented and locally verified. Fresh review fixes are covered by regressions. Full 08-03 remains incomplete pending 08-05 safety/recovery integration.
- Proof: Core workflow 20/20; CLI 4/4; full workspace 115 targets, 1,099 passed, 0 failed, 4 ignored; format and warnings-denied Clippy passed. No project-scoped proof/trace exists and no security audit was executed.
- Persisted-state boundary: this linked worktree, tracked/untracked 08-02 changes, new plan/ledger, and Cargo outputs. Primary checkout, user projects, Vault, machine configuration, and GitHub are out of scope.
- Safe next action: implement SPEC-08-05 before full 08-03 audit integration; do not enable target execution before its safety/recovery gates exist.
