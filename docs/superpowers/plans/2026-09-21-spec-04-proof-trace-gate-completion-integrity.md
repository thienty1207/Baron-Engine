# Plan: SPEC-04 Proof, Trace, Gate, and Completion Integrity

## Purpose

Implement only SPEC-04 on top of baseline `fa1bd91c162f38800070d1cfded69a3a68d474e0`.
The implementation must make proof publication, trace scoring, quality gates, and
plan completion use one exact operation-scoped chain. It must not begin SPEC-05,
change release metadata, or claim SPEC-04 closure.

## Constraints and review focus

- Preserve the existing Rust-first Baron Core model and legacy artifacts as
  diagnostic-only compatibility data.
- Correctness-sensitive callers must never use repository-global newest proof or
  trace selection when an identified active plan exists.
- A receipt-bound proof must be complete in memory before its first authoritative
  publication. A failure in the narrow publication path must not leave a repo
  proof or validation evidence that can authorize completion.
- Operation identity is the exact tuple: task ID, operation ID, adapter, session
  ID, and request ID. Proof ID is additionally exact for trace binding.
- Keep Low Minimal, Medium Standard, and High Detailed trace tiers.
- Do not broaden this into a repository/Vault transaction or start SPEC-05.
- At completion, report `SPEC-04: READY FOR ADVERSARIAL REVIEW`; never mark it
  `CLOSED` in this implementation pass.

## Current checkpoint

- Worktree: `C:/Users/Ty/.codex/worktrees/spec-04-proof-trace-gate-completion-integrity/Baron-Engine`
- Branch: `codex/spec-04-proof-trace-gate-completion-integrity`
- Baseline: `fa1bd91c162f38800070d1cfded69a3a68d474e0`
- Baseline focused tests passed:
  - `cargo test -p baron-core --test proof_trace --no-fail-fast`
  - `cargo test -p baron-core --test plan --no-fail-fast`
- Implementation checkpoint: the bound-proof publication path, operation-bound
  trace path, shared completion evaluator, scoped Prepare selectors, CLI wiring,
  and adversarial regressions are implemented and verified. Correctness paths
  no longer select global newest proof/trace artifacts.

## Implementation tasks

### 1. Establish failing regressions first

- Add focused tests for complete bound-proof rendering and publication failure.
- Add tests proving newer unrelated proofs/traces, cross-operation proof
  references, and cross-operation gates cannot authorize completion.
- Add a legacy unbound proof/trace diagnostic-only regression.
- Add a positive exact proof-to-trace-to-gate-to-plan completion regression.
- Run the focused tests and record the expected failures before implementing.

### 2. Make proof publication operation- and receipt-bound

- Extend proof parsing/records with typed receipt and operation binding data,
  while treating legacy unbound records as diagnostic-only.
- Build the complete final bound proof body, including proof ID, receipt ID,
  task ID, operation ID, adapter, session ID, request ID, gate kind, source
  fingerprint, capability gate, and evidence, before the first authoritative
  write.
- Validate the receipt, gate kind, exact source fingerprint, and exact identity
  before publication. Use the narrow existing safe-write paths and ensure a
  publication failure cannot create repo proof authority or promoted validation
  evidence.
- Add exact proof selectors for IDs and operation bindings without changing the
  diagnostic global `latest_proof` API.

### 3. Add operation-bound trace recording and scoring

- Add a typed trace binding with the five identity fields plus exact proof ID.
- Add an operation-aware trace recorder that references only an exact bound
  proof and never consults global newest proof selection.
- Parse and verify proof/trace binding equality and current receipts for
  Medium/High evidence; preserve the existing tier semantics.
- Add exact trace selectors and keep global newest trace APIs diagnostic-only.

### 4. Centralize completion-integrity evaluation

- Create one scoped completion evaluator used by both `complete_plan` and
  `plan_status`/completion-integrity diagnostics.
- For identified active plans, select exact proof and trace artifacts, require
  exact proof ID linkage, exact operation identity, strict operation-scoped gate
  evidence, and current receipts for Medium/High risk.
- Ensure the authoritative chain is active plan == proof == trace == gate
  binding, with `trace.proof_id == proof.id`.
- Leave legacy unbound artifacts available for status diagnostics but unable to
  authorize an identified or higher-risk completion.

### 5. Wire CLI and compatibility surfaces

- Route operation-aware proof/trace flows through the new bound APIs.
- Keep explicit legacy commands usable only as diagnostic compatibility paths;
  do not let them satisfy correctness-sensitive completion.
- Update existing CLI and Core tests to exercise the operation-scoped flow.

### 6. Documentation and evidence

- After code and tests are verified, update the status dashboard, machine
  status, and current build log with the actual SPEC-04 implementation state.
- Record known unrelated blockers only when reproduced by the required matrix.
- Do not mark SPEC-04 closed and do not touch release/version/tag state.

### 7. Verification and delivery

- Run focused Core/CLI contract tests, static scans for unsafe global artifact
  selection, formatting, all workspace tests, clippy, locked release build,
  diff check, and `baron --version`.
- Review the final diff and stage only SPEC-04 implementation/evidence files.
- Commit once on `codex/spec-04-proof-trace-gate-completion-integrity`, push it,
  and verify local and remote SHAs. Do not open a PR unless requested.

## Completion ledger

- [x] Regressions added and observed failing
- [x] Bound proof publication implemented and verified
- [x] Operation-bound trace implemented and verified
- [x] Scoped completion evaluator implemented and verified
- [x] CLI/legacy compatibility behavior verified
- [x] Full verification matrix recorded
- [x] Status/build-log evidence updated without closure claim
- [x] Commit pushed and remote SHA verified
