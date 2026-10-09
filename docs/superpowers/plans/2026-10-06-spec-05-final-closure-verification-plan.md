# SPEC-05 Final Closure Verification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Verify the exact current SPEC-05 source, establish zero known owned authority bugs, obtain two fresh independent ACCEPT reviews of one pushed exact range, and close SPEC-05 only if every gate passes.

**Architecture:** Start from the user-specified branch and commit, audit production authority callers, run fresh focused and full gates, then publish an evidence-complete readiness snapshot. Two isolated read-only reviewers inspect the same exact `ecaf362..FINAL_REVIEW_HEAD` range; any Critical/Important finding resets readiness and requires a regression-backed repair plus a new full review range.

**Tech Stack:** Rust workspace (`cargo`), PowerShell 7 test host, Git/GitHub, Markdown and JSON evidence.

**Spec:** `D:/Works/Baron-Engine/fix-bug/prompt/SPEC-05-FINAL-CLOSURE-VERIFICATION-AND-REVIEW-PROMPT.md`

## Global Constraints

- Branch: `codex/spec-05-multi-agent-concurrency-durable-state`; starting HEAD: `3cd03c6793807f2a149b3ed6f56dc43027640955`.
- Review base: `ecaf362c275431edfc5e590fd6a87d926b9bde9d`; binary remains `baron 5.0.0`.
- `CURRENT.md` is presentation-only for identified operations; ambiguity without exact identity fails closed.
- Do not release, tag, bump version, implement SPEC-06, modify Hotel Staff, or rewrite pushed history.
- Only source/test/evidence changes required by the prompt are in scope; preserve unrelated changes.
- READY requires all local gates plus authority `BUG=0`; CLOSED requires two fresh exact-range ACCEPT verdicts.
- No code repair without a test first observed failing for the expected reason, then passing, followed by its owning suites and the prompt's fresh gates.
- Local tests are not GitHub CI. Report hosted checks only from current GitHub evidence.

## Review Focus

- Host-shaped Codex `turn_id` and Claude `prompt_id` correlation across restart, including Stop with no prompt text.
- Cross-operation A/B plan start/resume/update/interrupt/complete, proof/trace/gate provenance, and exact selectors while CURRENT points to B.
- Legacy multi-active ambiguity, malformed/duplicate/linked story authority, and fail-closed no-write behavior.
- Shared-Vault lock ordering, timeout/no-write semantics, cross-checkout deduplication, and migration handoff/rollback races.
- Backward compatibility: legacy single-active plans, public config literals, unknown TOML, opaque adapter values, and the exact 5.0.0 product boundary.

---

### Task 1: Establish the execution checkpoint

**Files:**
- Create: this plan and its SDD ledger under the matching ignored `.superpowers/sdd/` workspace.
- Modify: `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, `notes/build-log/CURRENT.md` only to record that Phase 16 verification has started, the exact checkpoint, and the next safe action.

**Interfaces:** consumes current branch/HEAD and the closure prompt; produces the immutable `ecaf362` base, starting `3cd03c6` head, isolated test environment, and ledger used by later tasks.

- [x] Verify worktree branch, exact starting SHA, remote SHA, and clean state before task files were added: starting HEAD and remote were both `3cd03c6793807f2a149b3ed6f56dc43027640955`.
- [x] Create a task-specific SDD ledger; record identity, persisted-state boundary, proof/trace state, and no release/tag/version changes.
- [x] Update maintained current status to `FIX REQUIRED — verification in progress` without replacing historical evidence.
- [x] Verify `docs/BARON_STATUS.json` parses and the scope diff is clean.

### Task 2: Re-audit production authority callers

**Files:**
- Inspect: `crates/baron-core/src/**`, `crates/baron-cli/src/**`, `crates/baron-adapters/src/**`.
- Update: `docs/superpowers/plans/2026-10-01-spec-05-authority-audit.md` and the new final verification ledger.

**Interfaces:** Task 1 fixes the source SHA; this task produces the caller inventory that Task 3/4 validate and reviewers can reproduce. Tests, comments, and imports are not production caller groups.

- [x] Re-run every search pattern in the prompt over the full named source trees.
- [x] Trace production caller paths in Core, CLI, and adapters to their selecting identity and persisted authority; conditional branches are separate paths where needed.
- [x] Publish the exact-one-category inventory in `docs/superpowers/plans/2026-10-01-spec-05-authority-audit.md` (45 caller-path groups; category counts sum to 45).
- [x] Current audit result is `BUG=0`; the concrete RED→GREEN repairs are recorded in the audit and regression tests. This is not yet READY because full verification remains pending.

### Task 3: Run fresh focused regression matrix

**Files:**
- Inspect/run: focused Core, CLI, and adapter integration targets enumerated in the prompt, including the duplicated `harness_improvement` entry once per listed invocation if needed.
- Update: the final verification ledger with exact command, counts, exit codes, and environment.

**Interfaces:** consumes the audited exact source; produces fresh focused evidence for Task 4 and both reviewers.

- [x] Use nonexistent isolated Codex/Claude session roots and repository-supported PowerShell 7 where required.
- [x] Run all named focused targets from current source; Core targets total 317 passed, plus the required repeated `harness_improvement` target 13/13; CLI identity/evidence/hook targets total 38 passed, all with zero failures.
- [x] Re-run tracked-source retirement gate; it exited 1 with no output (no tracked hits).
- [x] Baseline lifecycle behavior recorded: default host PowerShell 4/7 with three `Compress-Archive` environment failures; bundled PowerShell 7 7/7.
- [x] Every required focused target passed. The separately requested default PowerShell lifecycle attempt failed 3 tests only due to its observed missing `Microsoft.PowerShell.Archive` module; rerunning under the supported bundled PowerShell 7 passed 7/7.

### Task 4: Run fresh full verification and baseline/scope gates

**Files:**
- Run exact cargo commands in the prompt and release/docs gates from the current source.
- Inspect: `.superpowers/sdd/2026-10-01-spec-05-zero-current-closure-repair/verify-final.ps1` and existing test-host settings before use.
- Update: final verification ledger only with observed results.

**Interfaces:** consumes the focused-green tree; produces all-target, lint, release, binary, docs, environment, baseline, and scope evidence needed to qualify READY.

- [x] Re-run fmt, Core/CLI/adapters/workspace all-targets, Clippy, and locked release on the repaired tree: Core 631/0/0, CLI 190/0/1 ignored, adapters 124/0/3 ignored, workspace 945/0/4 ignored across 112 suites; fmt, Clippy `-D warnings`, and release build exit 0.
- [x] Verify the default-host lifecycle result: 4/7 because the archive module cannot load; supported bundled PowerShell 7 lifecycle target passes 7/7.
- [x] Verify `session_replay` UTF-8 regression on current code: 5/5.
- [x] Re-run binary version (`baron 5.0.0`), release smoke 1/1, trust docs 12/12, status JSON parse, 12 changed-Markdown links, tracked-source gate, five CLI help commands, and `git diff ecaf362 --check` after the evidence update.
- [x] Confirm no release/tag/version bump/SPEC-06/Hotel Staff or out-of-scope manifest/lockfile changes.
- [x] Query GitHub Actions for repaired code commit `5c2bb5bda74363be3532a9f5516d26848379bada`: run `37718795708` completed with Format/Clippy job `113121400969`, Windows native job `113121401034`, and Linux native job `113121401053` all successful; both native jobs passed the full suite, release build, and CLI version smoke.
- [x] All local gates for the repaired source pass. The default-host lifecycle limitation and supported bundled PowerShell 7 result remain separately disclosed. Local checks do not imply hosted CI.
- [x] Hosted Baron CI `37461605214` at `c2b91db` was checked: Format/Clippy and Windows native passed; Linux native failed. Its artifact identified Windows-specific test fixtures. This failure invalidates the earlier readiness snapshot.

### Task 5: Publish an evidence-complete readiness snapshot

**Files:**
- Update: `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, `notes/build-log/CURRENT.md`, final verification ledger, and authority-audit ledger.
- Include: this plan and only scoped source/tests/evidence.

**Interfaces:** consumes Tasks 2–4; produces one committed/pushed exact readiness SHA for both independent reviews.

- [x] Reconcile the current status as `FIX REQUIRED` after the hosted Linux failure; prior readiness claims remain historical only.
- [x] Set `READY FOR ADVERSARIAL REVIEW` for repaired-source commit `1d02d3272b196c96ae84354a3478117af97459b9` after exact-SHA Baron CI run `37732260773` passed all three jobs and the fresh 47-path audit remained `BUG=0`. The earlier readiness snapshot is historical and invalidated by the two review findings.
- [ ] Commit/push this synchronized readiness snapshot, verify local HEAD equals remote HEAD, and verify its exact-SHA hosted workflow before freezing a new `FINAL_REVIEW_HEAD`.

### Task 6: First exact-range adversarial reviews (completed — FIX REQUIRED)

**Files:**
- Read-only review range: `ecaf362c275431edfc5e590fd6a87d926b9bde9d..FINAL_REVIEW_HEAD`.
- Record each complete reviewer report and verdict in the closure evidence.

**Interfaces:** both reviewers consume identical committed source/evidence and the mandatory 23-point checklist in the prompt; their outputs must be independent.

- [x] Dispatch two separate fresh read-only reviewers; neither edited worktree/index/HEAD.
- [x] Both reviewed exact range `ecaf362c275431edfc5e590fd6a87d926b9bde9d..c50b3b9bb29e58878f7edf00bcea4f0adb1bb06f` and returned the required format.
- [x] Boyle: zero Critical; one Important (identity-less Harness intake can borrow another operation's confirmed intent); `FIX REQUIRED`.
- [x] Parfit: zero Critical; one Important (installer output capture can absorb a concurrent project-config update and later rollback over it); `FIX REQUIRED`.
- [x] Treat both verdicts as a failed closure gate; neither is acceptance for the repair range.

### Task 7: Close and deliver only after both ACCEPT

**Files:** maintained status/build evidence and final closure ledger only; no executable change after accepted source boundary.

**Interfaces:** consumes two ACCEPT verdicts for one exact identical range; produces the closure-evidence commit and final verified remote SHA.

- [ ] Set `SPEC-05: CLOSED` only when every prompt criterion is true and unresolved Critical/Important are both zero.
- [ ] Record exact accepted source range, both reviewer identities/verdicts, closure evidence commit, and no release/tag/version changes.
- [ ] Push closure documentation, verify local HEAD equals remote HEAD, and confirm source semantics did not change after acceptance.
- [ ] Otherwise retain `FIX REQUIRED` or `READY FOR ADVERSARIAL REVIEW` and report the exact remaining gate.

### Task 8: Repair both fresh-review findings

**Files:** `crates/baron-core/src/intent.rs`, `crates/baron-core/tests/intent.rs`, `crates/baron-core/src/migration.rs`, `crates/baron-core/tests/migration.rs`, the migration CLI output declaration, and maintained status/evidence.

- [x] Add the identity-less intent-ingress regression; observe the old code incorrectly creates a high-risk story using another operation's confirmation.
- [x] Reject operation-scoped `CURRENT_INTENT` as authority for identity-less intake; preserve legacy unbound-intent behavior; focused intent tests pass 8/8.
- [x] Add a migration rollback regression that observed the concurrent project-config file disappear on the old code after a later installer failure.
- [x] Prevent callback-time output capture from adopting user-owned configuration; project/local config is excluded from CLI installer outputs and rejected as rollback authority by Core. Core migration passes 23/23 and CLI migration passes 3/3.
- [x] Run focused migration/intent targets (intent 8/8, migration 23/23, CLI migration 3/3), the full prompt matrix (329 Core executions, including the required repeated target; CLI selector/evidence/hook 38/38), the fresh authority audit (47 groups, `BUG=0`), and all local gates on the repaired tree.
- [x] Full all-targets: Core 634/0/0 across 60 suites; CLI 190/0/1 ignored across 38; adapters 124/0/3 ignored across 14; workspace 948/0/4 ignored across 112. `fmt --check`, warnings-denied Clippy, locked release build, release smoke 1/1, trust docs 12/12, version/help/JSON checks pass. Default PowerShell lifecycle is 4/7 due to the host's unavailable `Microsoft.PowerShell.Archive`; supported bundled PowerShell 7 passes 7/7.
- [x] Post-edit gates pass: 13 changed-Markdown relative links, status JSON parse, tracked-source/filename retirement checks, diff check from `ecaf362`, and scoped file inventory (no manifests/lockfiles or out-of-scope paths).
- [x] Commit/push the two regression-backed repairs and evidence at `1d02d3272b196c96ae84354a3478117af97459b9`; verify local/remote equality and exact-SHA hosted CI run `37732260773` (Format/Clippy, Linux, Windows all success).
- [ ] Publish the synchronized readiness-doc snapshot, verify its exact-SHA workflow, freeze `FINAL_REVIEW_HEAD`, and obtain two fresh independent reviews of that identical range.

### Task 9: Fresh post-repair exact-range reviews

**Files:** read-only review package for the final pushed range; final verification and closure evidence only after both reports return.

**Interfaces:** consumes the pushed readiness snapshot and its exact-SHA hosted CI; produces two independent complete reports for the identical `ecaf362..<FINAL_REVIEW_HEAD>` range. This is the required post-repair continuation of Task 6; the earlier `FIX REQUIRED` verdicts do not count.

- [ ] Generate and verify the complete review package from `ecaf362c275431edfc5e590fd6a87d926b9bde9d` through the frozen final review SHA.
- [ ] Dispatch two separate read-only reviewers; neither may mutate the worktree, index, HEAD, or branch.
- [ ] Each reviewer explicitly assesses all 23 prompt checklist items and returns `Critical findings`, `Important findings`, and `Verdict: ACCEPT | FIX REQUIRED | INCONCLUSIVE`.
- [ ] Require both verdicts to be `ACCEPT` for the same exact range; quota/tool failure is `INCONCLUSIVE`, never acceptance.
- [ ] If either reviewer finds Critical/Important, keep SPEC-05 open and repeat the prompt's regression-backed repair, full verification, push, and fresh-review cycle.

### Task 10: Repair final reviewer finding I-1

**Files:** `crates/baron-core/src/plan.rs`, `proof.rs`, `trace.rs`, `control_plane.rs`; related Core/CLI tests; synchronized status and verification evidence.

**Interfaces:** consumes the fresh Important review of `ecaf362..a46ac3e`; produces strict indexed-ACTIVE publication authority, non-revivable evidence generations, and a new review candidate only after every local gate passes.

- [x] Reproduce the missing-ACTIVE frontmatter fallback: stale identified plan metadata could authorize evidence publication without a validated ACTIVE row.
- [x] Add the no-write regression across proof, trace, and receipt-bound gate ingress; recovery of the same identity must rotate generation and leave prior proof/trace unusable for completion.
- [x] Enforce strict indexed ACTIVE/frontmatter validation under the mutation lock for evidence writers; persist authority generation in the plan index/frontmatter and evidence; preserve existing public Rust struct shapes.
- [x] Make completed-plan integrity audit resolve exact risk/generation and mirrored Vault trace without requiring the completed plan to remain ACTIVE.
- [x] Focused Core tests pass: authority_ingress 6/6, control_plane 9/9, plan 57/57, proof_trace 36/36. Focused CLI tests pass: execution 15/15, operation_evidence 11/11, plan_identity 4/4. Receipt authority fixtures and trusted proof pass 29/29 after seeding exact active plans.
- [x] Finish serial workspace all-targets run: Core 634/0/0, CLI 190/0/1 ignored, adapters 124/0/3 ignored, workspace 948/0/4 ignored across 112 suites. The earlier parallel attempt hit Windows `os error 1450`; the five stale fixtures were corrected and the final serial run passed.
- [x] Run warnings-denied Clippy and locked release build; release binary is exactly `baron 5.0.0`, release smoke 1/1, public trust docs 12/12, and CLI help selectors pass.
- [x] Finish post-sync status JSON, changed-Markdown links, tracked retirement, diff, and scope gates: JSON parses; 16 relative links across 10 changed Markdown files, 0 broken; retirement gate 8/8; `git diff ecaf362 --check` exits 0; 70-file range inventory has no manifests/lockfiles, release workflow, Hotel Staff, or SPEC-06 changes.
- [x] Commit and push the synchronized repair at `752ea138d7092acda56edc832472e3bae78ec820`; verify local/remote branch SHA equality and exact-SHA Baron CI run `37889435584` (Format/Clippy, Linux and Windows native tests, release builds, and version smoke all succeeded).
- [ ] Synchronize run `37889435584` into maintained status/evidence, push the documentation snapshot, verify its local/remote SHA equality and exact-SHA CI, then freeze `FINAL_REVIEW_HEAD`.
- [ ] Obtain two fresh independent read-only reviews of the identical complete `ecaf362..<FINAL_REVIEW_HEAD>` range; both must explicitly ACCEPT with zero Critical/Important findings.

### Task 11: Repair final exact-range review findings

**Files:** `crates/baron-core/src/proof.rs`, `crates/baron-core/src/plan.rs`, `crates/baron-core/src/task_state.rs`, `crates/baron-core/src/prepare.rs`, `crates/baron-core/src/continuity.rs`, related Core tests, and maintained status/audit/verification evidence.

**Interfaces:** consumes the two `FIX REQUIRED` reviews of `ecaf362..02779815c87bf2df6373085125857302503f2e05`; produces fresh RED→GREEN regressions and a new exact review candidate only after full verification and authority re-audit.

- [x] Reproduce missing-ACTIVE proof visibility, Task State proof/current-plan visibility, and identity completion status before generation recovery; each failed before its repair.
- [x] Reproduce generic completion/reconciliation passing from frontmatter-only evidence without indexed ACTIVE; the audit-driven regression failed before its repair.
- [x] Reproduce the repeated identical lifecycle identity across two checkouts sharing one Vault; the RED test observed partial local plan/index writes before the preflight repair.
- [x] Add regressions for each observed behavior and run them RED before modifying the corresponding production paths.
- [x] Fix each confirmed root cause while preserving frontmatter fallback for legacy diagnostics and repo→Vault lock ordering. Indexed current-state readers now fail closed; shared identity ownership is checked before plan publication.
- [x] Rerun affected focused suites and the complete prompt matrix: 325 Core passes across 23 targets plus the required repeated `harness_improvement` 13/13 (338 executions total); required CLI selectors/evidence/hooks 38/38 plus migration CLI 3/3.
- [x] Run all Core/CLI/adapters/workspace targets: Core 635/0/0 across 60 suites; CLI 190/0/1 ignored across 38; adapters 124/0/3 ignored across 14; workspace 949/0/4 ignored across 112.
- [x] Run `cargo fmt --all -- --check`, warnings-denied Clippy, and `cargo build --workspace --release --locked`; all exit 0. Release binary is exactly `baron 5.0.0`; required ignored release smoke 1/1; public trust docs 12/12; five CLI help selectors pass.
- [x] Verify lifecycle baseline honestly: default host 4/7 due unavailable `Microsoft.PowerShell.Archive`; supported bundled PowerShell 7 7/7. Current `session_replay` is 5/5.
- [x] Refresh the complete production authority audit after the latest repairs: 48 groups, categories 11/8/17/9/3, `BUG=0`; no owned Critical/Important issue remains known from this source audit.
- [x] Verify JSON, changed-Markdown relative links, retired-adapter source/filename gate, `git diff ecaf362 --check`, and 70-path scope (no manifest/lockfile, release workflow, Hotel Staff, or SPEC-06 changes).
- [ ] Commit and push the readiness snapshot; verify local/remote SHA equality and exact-SHA hosted CI before freezing `FINAL_REVIEW_HEAD`.
- [ ] Obtain two fresh independent read-only ACCEPT reviews of the identical complete `ecaf362..<FINAL_REVIEW_HEAD>` range; if either finds Critical/Important, restart the repair/full-verification/review cycle.
