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

- [x] Fresh full gate script: `cargo fmt --all -- --check` exit 0; Core 631/0/0 across 60 suites; CLI 190/0/1 ignored across 38 suites; adapters 124/0/3 ignored across 14 suites; workspace 945/0/4 ignored across 112 suites; Clippy `-D warnings` and locked release build exit 0.
- [x] Verify supported PowerShell 7 `lifecycle_scripts` result: 7/7; default host failed 3 tests with the module-load error recorded in the ledger.
- [x] Verify `session_replay` UTF-8 regression on current code: 5/5.
- [x] Verify binary version (`baron 5.0.0`), release smoke 1/1, trust docs 12/12, status JSON parse, 11 Markdown links/0 broken, tracked-source retirement gate, all CLI help checks, and `git diff ecaf362 --check` exit 0. The staged wording was checked RED→GREEN and public trust docs rerun afterward.
- [x] Confirm no release/tag/version bump/SPEC-06/Hotel Staff or out-of-scope manifest/lockfile changes.
- [ ] Query GitHub Actions and commit statuses for the eventual readiness commit; if absent, record exactly the prompt's no-hosted-evidence sentence.
- [x] All required local gates pass. The default host lifecycle limitation (4/7 due to missing archive module) is disclosed; the supported bundled PowerShell 7 target passes 7/7. Local checks do not imply hosted CI.

### Task 5: Publish an evidence-complete readiness snapshot

**Files:**
- Update: `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, `notes/build-log/CURRENT.md`, final verification ledger, and authority-audit ledger.
- Include: this plan and only scoped source/tests/evidence.

**Interfaces:** consumes Tasks 2–4; produces one committed/pushed exact readiness SHA for both independent reviews.

- [x] Reconcile current counts and findings across maintained status artifacts; preserve historical runs and prior FIX REQUIRED verdicts as historical.
- [x] Set `READY FOR ADVERSARIAL REVIEW` only after all local gates pass and authority `BUG=0`.
- [ ] Commit scoped changes, push the branch, and verify local HEAD equals remote HEAD; record `FINAL_REVIEW_HEAD`.

### Task 6: Obtain two independent exact-range adversarial reviews

**Files:**
- Read-only review range: `ecaf362c275431edfc5e590fd6a87d926b9bde9d..FINAL_REVIEW_HEAD`.
- Record each complete reviewer report and verdict in the closure evidence.

**Interfaces:** both reviewers consume identical committed source/evidence and the mandatory 23-point checklist in the prompt; their outputs must be independent.

- [ ] Dispatch two separate fresh read-only reviewers; neither may edit worktree/index/HEAD.
- [ ] Require explicit `Critical findings`, `Important findings`, and `Verdict: ACCEPT | FIX REQUIRED` plus every checklist item.
- [ ] Quota/tool/time failure is `INCONCLUSIVE`; prior review ranges do not count.
- [ ] If either finds Critical/Important, keep FIX REQUIRED, create a failing regression, repair RED→GREEN, rerun full gates/audit, push a new range, and obtain two new reviews.

### Task 7: Close and deliver only after both ACCEPT

**Files:** maintained status/build evidence and final closure ledger only; no executable change after accepted source boundary.

**Interfaces:** consumes two ACCEPT verdicts for one exact identical range; produces the closure-evidence commit and final verified remote SHA.

- [ ] Set `SPEC-05: CLOSED` only when every prompt criterion is true and unresolved Critical/Important are both zero.
- [ ] Record exact accepted source range, both reviewer identities/verdicts, closure evidence commit, and no release/tag/version changes.
- [ ] Push closure documentation, verify local HEAD equals remote HEAD, and confirm source semantics did not change after acceptance.
- [ ] Otherwise retain `FIX REQUIRED` or `READY FOR ADVERSARIAL REVIEW` and report the exact remaining gate.
