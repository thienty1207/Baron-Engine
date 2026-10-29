# SPEC-08-05 Safety, Recovery, and Failure Handling — Implementation Plan

> **For agentic workers:** implement this plan task by task with red/green verification. Preserve the in-flight SPEC-08-02/03/04 changes and do not publish this work.

**Goal:** Make security-audit state, capability gates, checkpoints, and reports resumable and truthful under interruption, source changes, malformed evidence, missing controls, and publication failure.

**Architecture:** Extend the Core audit workflow with explicit `in_progress`, `complete`, `incomplete`, `blocked`, and `failed` outcomes. Add a Core safety/recovery module that evaluates registered/present/executed capability facts, refuses target execution without every OS-enforced sandbox control, rechecks exact project/revision/source content on resume, and publishes explicit-output artifacts atomically without overwriting edits. Keep static source analysis available when execution is blocked.

**Tech Stack:** Rust, existing Baron Core Capability Registry, verified execution receipts, continuity, `ignore`, `sha2`, `serde`, and `safe_io`; CLI integration in `baron`; no automatically installed providers or new runtime dependency.

**Spec:** `docs/specs/Spec8 - Replace VibeSecurity/SPEC-08-05-SAFETY-RECOVERY-AND-FAILURE-HANDLING.md`

## Global Constraints

- Preserve all local SPEC-08-02/03/04 changes; do not reset, commit, push, merge, release, tag, bump the version, or rewrite history.
- Never run target-controlled builds, tests, fixtures, fuzzers, browsers, or processes without verified OS enforcement for network denial, sanitized allowlisted environment, read-only source/toolchain, assigned scratch writes, and CPU, memory, process, disk, file-size, and time limits.
- If a required sandbox control/provider is missing, continue source-only reasoning and return a blocker or `needs_validation`; do not execute target code, install providers, or claim a clean audit.
- Resume may reuse prior evidence only after checking project ID, Git revision, dirty-state/source-content fingerprint, mode, scope, languages, phase, and pending work against the current checkout; changed content invalidates prior evidence for review.
- Checkpoints and reports are bounded, validated, atomic/create-only or compare-before-replace, recoverable, secret-redacted where user-visible, and written only to an explicit existing output location or the existing Baron capsule contract.
- Tests use disposable synthetic repositories and temporary files; they do not run application code or touch user projects, Vaults, machine settings, or ambient secrets.

## Review Focus

- A missing or incomplete OS sandbox must block target-controlled execution while retaining source-only results; test each required control independently.
- A capability that is only registered or present must not count as executed; test current-operation verified receipt binding and stale evidence.
- A changed project ID, revision, dirty fingerprint, or source file must prevent stale checkpoint evidence from being trusted; test changed and unchanged resume paths.
- Missing, malformed, oversized, or schema-mismatched checkpoints and interrupted publication must not become complete or replace a valid prior artifact; test bounded parsing and atomic/create-only publication.
- Failed commands, timeouts, verifier disagreement, resource limits, and secrets in evidence must remain distinct from negative findings and be reported without reflecting raw secrets; test every failure class and redaction.

## Tasks

### Task 1: Truthful run states and failure matrix

**Files:** Modify `crates/baron-core/src/security_audit_workflow.rs`; create or extend `crates/baron-core/tests/security_audit_safety.rs`.

**Interfaces:** Add `AuditRunStatusV1::{Incomplete, Failed}`, an `AuditFailureClassV1` enum, `AuditFailureV1 { class, code }`, and bounded `AuditRunV1::mark_incomplete` / `AuditRunV1::mark_failed` transitions. The report adds `clean` and `application_task_blocked`; `clean` can be true only when current trusted execution evidence and all required workflow/coverage gates allow it, which this schema-only stage does not establish.

- [x] Write tests `run_states_distinguish_in_progress_complete_incomplete_blocked_and_failed`, `failed_execution_and_timeout_never_become_negative_findings`, `deferred_coverage_cannot_produce_clean_report`, `confirmed_finding_does_not_prevent_audit_completion`, and `application_task_remains_blocked_until_trusted_closure`.
- [x] Run `cargo test -p baron-core --test security_audit_safety` and observe the intended failures.
- [x] Implement bounded failure classes and state transitions; retain failure class and safe code, never raw command output or input JSON.
- [x] Add explicit report fields for clean/audit status/application-task blocker; a complete audit may still contain findings, and schema-only evidence must never produce `clean: true`.
- [x] Run `cargo test -p baron-core --test security_audit_safety` (7/7), `cargo test -p baron-core --test security_audit_workflow` (20/20), and `cargo fmt --all -- --check` (passed).

### Task 2: Capability and sandbox execution gate

**Files:** Create `crates/baron-core/src/security_audit_safety.rs`; export it from `crates/baron-core/src/lib.rs`; extend Core safety tests and capability tests as needed.

**Interfaces:** Add `AuditCapabilityUsageV1 { capability, provider, registered, present, executed, blocker }`, `AuditSandboxReadinessV1 { controls, execution_allowed, blockers }`, `inspect_audit_capabilities(repo_root, operation, evidence)`, and `assess_audit_sandbox(repo_root, operation)`. The sandbox result must remain denied until a trusted OS-enforcement backend can prove every named control; capability registration or presence alone is not that proof.

- [x] Write tests `registered_capability_is_not_present_or_executed`, `present_capability_is_not_executed_without_current_verified_receipt`, `stale_receipt_does_not_satisfy_current_operation`, and `missing_sandbox_controls_block_without_running_target`.
- [x] Run the focused Core safety test and observe the intended failures.
- [x] Read registry/cache without auto-installing or probing providers; validate execution only through Core verified receipts bound to the current task/operation/adapter/session/request.
- [x] Return stable missing-control codes for network denial, sanitized environment, read-only source/toolchain, assigned scratch, CPU, memory, process, disk, file-size, and time controls. Keep source-only analysis available.
- [x] Run `cargo test -p baron-core --test security_audit_safety` (11/11).

### Task 3: Checkpoint binding, source revalidation, and atomic persistence

**Files:** Extend `crates/baron-core/src/security_audit_safety.rs`; add `crates/baron-core/tests/security_audit_safety.rs`; extend `crates/baron-cli/src/main.rs` and `crates/baron-cli/tests/security_audit_cli.rs`.

**Interfaces:** Add `AuditRecoveryCheckpointV1` containing the validated run, exact binding, deterministic source-file fingerprints, pending work, and safe next action; `create_audit_checkpoint(repo_root, run, pending_work, next_action)`, `validate_audit_checkpoint_json(bytes)`, `assess_audit_resume(repo_root, checkpoint)`, and create-only `publish_audit_checkpoint(output_path, checkpoint)`. CLI adds `security-audit save-checkpoint --repo-path <repo> --output <file>` and `security-audit resume-checkpoint --repo-path <repo> --input <file>`.

- [x] Write tests for checkpoint identity/revision/source changes, unchanged resume hints, malformed/schema/oversized JSON, symlink/size limits, create-only idempotency, and preserving external edits. A stale dirty-state binding regression was also observed RED then GREEN.
- [x] Run focused Core and CLI tests; the new stale dirty-state regression failed before the binding check and passed after it.
- [x] Snapshot only validated scope files with no-follow regular-file reads, bounded file/count/aggregate sizes, sorted content hashes, and an exact current Git revision; unsafe/unreadable/truncated scope fails closed.
- [x] Validate checkpoint project ID, source revision, scoped dirty state, content hash, full run binding, mode, scope, languages, phase, coverage, candidate fingerprints, pending work, and next action before resume.
- [x] On changed content, retain the old checkpoint and return changed paths plus `fresh_run_required`; never carry old findings/receipts forward as current evidence.
- [x] Require explicit output, existing parent directory, bounded JSON, and create-only atomic publication. Identical retry is idempotent; conflicting existing bytes fail without overwrite.
- [x] Add `save-checkpoint` and `resume-checkpoint` CLI commands; malformed checkpoint input is rejected without reflecting its body. Atomic publication delegates to Core's tested create-new writer.

### Task 4: Safe reporting, runtime contract, status, and full verification

**Files:** Extend report/CLI integration and tests; update `assets/core/skills/security-audit/SKILL.md`, relevant adapter/Core contract tests, SPEC-08-05 and parent SPEC-08-00 statuses, `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, and `notes/build-log/CURRENT.md`.

**Interfaces:** Add `redact_audit_report(report)`, create-only `publish_audit_report(output_path, report)`, and `security-audit publish-report --output <file>`; keep output free of proof/trace claims unless separately verified by existing Baron authorities.

- [x] Write redaction and conflicting-publication tests and observe the expected failure before implementation.
- [x] Ensure report and CLI output redact common credential formats and never echo raw malformed input, stderr/stdout, or environment values.
- [x] Update managed skill instructions to distinguish capability registered/present/executed, forbid unsandboxed target execution, retain partial results, and resume from exact state with the next safe action.
- [x] Update SPEC-08-05/parent and status docs truthfully; record that OS sandbox execution remains unavailable and keep SPEC-08 open.
- [x] Run focused Core/CLI/adapter suites, `cargo fmt --all -- --check`, `cargo test --workspace --all-targets --no-fail-fast -j 2 --quiet`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Parse status JSON, check changed relative Markdown links and `git diff --check`, review the SPEC-08-05 implementation/documentation changes, and preserve the existing uncommitted work without publishing.

### Task 5: Phase-boundary checkpoint persistence assessment

**Files:** Extend `crates/baron-cli/tests/security_audit_cli.rs`; optimize the bounded secret-redaction path in `crates/baron-core/src/security_audit_safety.rs`; update SPEC-08-05, parent SPEC-08-00, status, handoff, and build log.

- [x] Build synthetic runs at boundaries 0 through 6, including one candidate record and a complete coverage ledger from the hunting boundary onward; mark boundaries before reporting as interrupted.
- [x] Save each checkpoint through one CLI process and assess it twice through fresh CLI processes; assert exact phase/status, retained candidate fingerprints, coverage unit count, resume eligibility, terminal-state rejection, deterministic assessments, and byte-preserving checkpoint state.
- [x] The first candidate-bearing save exceeded 60 seconds because redaction patterns were recompiled for each string. Cache the static patterns once with `OnceLock`; the complete phase-boundary integration then passed in 12.41 seconds without changing redaction behavior.
- [x] Run formatter, focused Core safety unit/integration tests, all security-audit CLI tests, full workspace tests, and warnings-denied Clippy.
- [x] Update status and handoff to report the exact Windows acceptance scope and remaining blockers.
- [ ] Linux acceptance remains unavailable in this Windows environment because WSL is not installed; run the Linux job in CI before closing acceptance.

**Acceptance limit:** `resume-checkpoint` returns a source-revalidation assessment only. It does not mutate or automatically continue an audit run, so this test proves saved-state integrity and resume eligibility, not automatic per-phase execution or finding deduplication after a continued run. No OS-enforced sandbox backend is verified; target execution remains blocked.

## Initial checkpoint — 2026-10-29

- Active task: implement SPEC-08-05 in the existing linked worktree on `codex/spec-08-02-seven-language-coverage`, preserving local SPEC-08-02/03/04 work.
- Starting HEAD: `0933e2a11b5cd043a4533c43879ad3d842bd38a7` (pushed SPEC-08-01 base); worktree was already dirty before SPEC-08-05 changes.
- Source review so far: SPEC-08-03 has schema-only receipts and operation-bound continuity summaries; SPEC-08-04 remains explicitly blocked from target execution by SPEC-08-05. Generic Baron execution receipts are signed but the generic runner is not a sandbox and must not be reused for target code.
- Proof/trace: none for a user project; no target audit, remediation, or target-controlled process has run.
- Persisted-state boundary: this linked worktree, its ignored SDD ledger, Cargo outputs, and test-only temporary directories. Other worktrees, user projects, Vaults, machine configuration, and GitHub are out of scope.
- Safe next action: implement Task 2 capability/sandbox readiness test-first; no target execution path exists or is authorized.

## Final local checkpoint — 2026-10-29

- Task 4 local implementation and Windows verification are complete. The checkpoint file reader and create-only artifact comparison now reject oversized files before unbounded allocation; two Core regressions and one CLI regression were added. Both Core regressions were observed failing against the previous implementation and passing after the fix.
- Verification: format passed; workspace tests passed on 117 targets (1,150 passed, 0 failed, 4 ignored); warnings-denied Clippy passed after a behavior-neutral lexical-scope cleanup; final focused safety tests pass (Core 13/13, integration 11/11, CLI 11/11). Status JSON parses, 74 links across 29 changed Markdown files resolve, and `git diff --check` passes.
- No target project code or target-controlled process ran. All acceptance repositories and source files were synthetic temporary fixtures. A phase-boundary checkpoint integration now preserves state and confirms resume eligibility in fresh CLI processes, but it does not implement automatic phase execution/continuation or test finding deduplication after continuation. No OS-enforced sandbox backend is verified. Linux acceptance is unavailable on this host because WSL is not installed. SPEC-08 and executable audit/remediation remain incomplete.
- Worktree remains on `codex/spec-08-02-seven-language-coverage` at `0933e2a11b5cd043a4533c43879ad3d842bd38a7`, preserving existing uncommitted SPEC-08-02/03/04/05 changes; no commit, push, merge, tag, release, or version change occurred.
- Latest Windows verification: `cargo fmt --all -- --check` passed; `cargo test --workspace --all-targets --no-fail-fast -j 2 --quiet` passed on all 117 targets (exit 0, no failures, 4 ignored); `cargo clippy --workspace --all-targets -- -D warnings` passed. Focused safety tests pass Core 13/13, integration 11/11, and CLI 12/12. The PreparePacket CLI target passed 9/9 in 122.76 seconds; `hook_identity` passed 20/20 in 153.24 seconds. These were slow Windows targets, not failures.
- Safe next action: run Linux acceptance in CI, decide whether to add a phase coordinator for automatic persistence/continuation, and evaluate an OS-enforced sandbox backend before authorizing target execution.
