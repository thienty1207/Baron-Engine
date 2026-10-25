# Current Baron Engine Handoff

> This is the single canonical project handoff. The section below is current;
> all checkpoints and phase snapshots after it are historical records and must
> not override these facts or the checked-out source.

## Current source and release boundary — 2026-10-24

- Repository: `thienty1207/Baron-Engine`.
- Candidate branch: `codex/version-5-0-1`, based on verified `main` merge
  commit `cf319a12cfc19e2739303b1b26ee71357ae6c0cd`.
- Source version: `5.0.1` in the workspace crates and certification target.
  The source candidate is unpublished. Stable and downloadable release remain
  `5.0.0`; do not describe 5.0.1 as released or downloadable.
- The owner authorized committing and pushing this candidate branch after the
  handoff documentation update. That authorization does not authorize a tag,
  release publication, merge, or PR creation. Verify the branch's actual remote
  state before saying it was pushed; no PR or hosted CI result exists unless
  separately confirmed.
- The 5.0.1 release-candidate change fixes the Control Plane false positive for
  negated subagent mentions from merged PR #4. Read the version-bump plan for
  its version and verification details.
- The documentation duplicate audit found no byte-identical files under
  `docs/`; no documentation file was deleted. Similar current summaries now
  link to this handoff while preserving their separate dashboard/log roles.

## Product implementation state

- Baron is Rust-first. Baron Core owns product semantics, identity, Vault
  memory, trusted recall, Task State, routing, plans, continuity, recovery,
  proof, trace, quality gates, and Autopilot policy.
- `assets/core/**` is the canonical managed runtime source. Codex and Claude
  are thin native projections. Generic `AGENTS.md`/`CLAUDE.md` guidance is the
  fallback when host hooks do not run.
- The Codex + Claude Core consolidation through Phase 15 is complete. Phase 16
  adversarial verification has not started. SPEC-01 through SPEC-05 are
  recorded complete and merged. SPEC-06 is not started; no canonical SPEC-06
  or approved acceptance criteria are present in this checkout.
- Do not infer that the product has no remaining bugs. Future hardening should
  start from a reproducible defect, regression, security issue, or concrete
  integrity risk. Historical SPEC-05 residuals are documented in the status
  dashboard and JSON.

## Verification and known limits

- On this candidate, local format, full workspace tests, locked workspace
  check, warnings-denied Clippy, focused Core/adapter suites, CLI help, locked
  release build, JSON parsing, links, and diff checks passed. The release binary
  reports `baron 5.0.1`.
- The Windows lifecycle tests require the test-supported
  `BARON_TEST_POWERSHELL` override to bundled PowerShell 7 in this environment.
  Default Windows PowerShell 5.1 failed to auto-load `Microsoft.PowerShell.Archive`
  in three installer cases; the complete workspace run passed with the
  override. Hosted Windows/Linux CI on this candidate branch remains required
  before a release promotion.
- End-to-end host limits from the merged-source check: live Claude Code CLI was
  unavailable, so its projection and hook contract were tested without a live
  Claude run. Codex's implicit no-path context failed Windows path inspection
  involving the host shell's extended path prefix; the explicit project-path
  call passed. Follow up on implicit host cwd/path handling before treating
  that path as verified.
- No original project/Vault/machine configuration was changed. Scratch
  project/Vault testing used temporary paths only.

## Continue safely

1. Read [`AGENTS.md`](../../../AGENTS.md), then
   [`SOURCE_READING_GUIDE.md`](../../architecture/SOURCE_READING_GUIDE.md)
   before claiming a complete source audit.
2. Review the candidate branch and verify its remote commit. The owner has not
   requested a PR in this task; do not create one without that request.
3. Before release promotion, open an approved PR through the owner's workflow,
   require hosted Windows and Linux CI on the exact head, then follow
   `docs/RELEASE.md`. Keep stable/downloadable at 5.0.0 until release
   publication is separately authorized and complete.

Current records: [release/phase dashboard](../../BARON_STATUS.md),
[machine-readable status](../../BARON_STATUS.json),
[current build evidence](../../../notes/build-log/CURRENT.md),
[5.0.1 version-bump plan](2026-10-24-v5-0-1-version-bump.md), and
[AI handoff/source-guide plan](2026-10-24-ai-handoff-and-source-reading-guide.md).

## Historical checkpoints — context only

## Historical checkpoint — Post-merge status reconciliation and end-to-end validation (COMPLETE WITH HOST LIMITS — 2026-10-24)

- Objective completed: reconcile the four maintained status documents, sync local `main`, and validate the merged source through scratch installation, project init, hooks/logging, search, and continuity. This was verification/maintenance work, not SPEC-06 or a release.
- Verified GitHub and local `main`: `cf319a12cfc19e2739303b1b26ee71357ae6c0cd`. Merge workflow 39982258262 passed Format/Clippy plus Linux and Windows native tests. PR-head run 39977758980 also passed.
- Local checks passed: fmt; workspace all-target tests (1,029 passed, 0 failed, 4 ignored in 112 test binaries); warnings-denied Clippy. The v5.0.0 release installer passed in a disposable root but predates PR #4; a separate locked install from the merge checkout also succeeded and reports v5.0.0.
- Source E2E: setup and sequential Codex/Claude init passed in the same scratch Fiber project/Vault, with no conflicts and all three mandatory agents plus generic `AGENTS.md`. Go test passed; live `/health` returned `{"status":"ok"}`. Control Plane status passed; prepare had no false-positive `control_plane_invalid` blocker. Wiki exact search hit the seeded document; a separate project did not retrieve it.
- Hooks/logging: Codex and Claude `SessionStart`/`UserPromptSubmit` contracts passed and retries were idempotent. A real ephemeral Codex CLI run completed and its events reached the scratch Vault journal. Automation reconciliation correctly reports `attention_required` because this read-only smoke test produced no proof, required-gate receipts, or passing trace. Stop after an intentionally interrupted plan correctly failed closed.
- Codex host caveat: its no-path `baron context --codex` session-start command failed Windows path inspection at `\\?\C:\Users\tytyb` in the host shell. Explicit scratch-project context and context from a shell rooted at that project passed. Its `git status` command also failed because the disposable project had no `.git`; investigate host cwd/path handling before treating implicit-path context as verified.
- Continuity: the checkpoint was written to project and Vault; `plan status` preserved `interrupted` and its next action; `plan start` with the exact binding returned `Action: resumed`. A separate unmatched Codex session remained `unknown`. Session-replay indexed 0 sources/0 messages for the ephemeral host session.
- Host limits: Claude Code CLI is absent, so live Claude was unavailable; its generated projection and structured hook contract passed. Other hosts are covered by generic `AGENTS.md`, not native adapter claims. No product proof or trace was created.
- Persisted-state boundary: only these four maintained docs, this task plan, and the disposable temp install/runtime changed. User untracked files and real project/Vault/machine state were preserved. No source edits, commit, push, release, or publication.
- Safe next action: review the local documentation diff and follow up on implicit-path `baron context --codex` under the Codex Windows sandbox. Rerun live Claude/session-replay checks if required when the CLI and non-ephemeral host session are available.

Plan: [Post-merge status reconciliation and end-to-end verification](2026-10-23-post-merge-e2e-verification.md).


## Historical checkpoint — post-SPEC-05 status and roadmap reconciliation (DOCS-ONLY PR #3 merged — 2026-10-23)

- Objective: align the maintained status documents with the verified GitHub main state after SPEC-05; this phase changes documentation only.
- Verified main: 11a731c82d999366f287479e7b2afda46898570f. [PR #1](https://github.com/thienty1207/Baron-Engine/pull/1) merged on 2026-10-21 at 9038afa9f840e3a62acc0f97c6aaa31849247525; [PR #2](https://github.com/thienty1207/Baron-Engine/pull/2) merged on 2026-10-22 at 11a731c82d999366f287479e7b2afda46898570f.
- Integrated CI: [GitHub Actions run 39740216250](https://github.com/thienty1207/Baron-Engine/actions/runs/39740216250) succeeded on the exact PR #2 merge commit. Format and Clippy, Linux native tests, and Windows native tests all passed. This is existing hosted CI evidence; this documentation-only task did not rerun tests or builds.
- SPEC-01 through SPEC-05 are recorded complete in the repository status history. SPEC-05 post-closure H-01/H-02/H-03 hardening and Maintenance 01 are present in merged main. This does not claim the product is bug-free.
- The current proactive bug-fixing and hardening pass is closed for now. Future bug-fix work should start from a reproducible defect, regression, security issue, or concrete integrity risk.
- Preserved residuals: pre-fix plans created below case-sensitive Windows ancestors can retain an ambiguous folded v1 owner ID and fail strict validation; recovery needs an explicit audited migration, and checkout relocation remains fail-closed. The diagnostic automation_status path still reads its full journal; the final review recorded this as a Minor note.
- SPEC-06 is not started and its scope and acceptance criteria are not confirmed. The tracked repository contains no canonical SPEC-06 spec. The total number of specs in the current roadmap is Unknown; historical references do not establish it.
- The three existing 4.2 dashboard follow-ups remain separate from this closed SPEC-05 pass.
- Proof status: only repository and hosted-CI evidence is carried forward; this docs-only task creates no new Baron proof artifact. Trace status: no project-scoped Baron trace was produced for the unrecovered original project identity.
- Persisted-state boundary: only the four maintained status documents are in scope in this isolated worktree. No source, tests, runtime configuration, dependencies, generated binaries, project/Vault data, or user untracked files are changed.
- Safe next action: review and merge the docs-only status PR. Keep SPEC-06 unstarted until a canonical scope and acceptance criteria are approved.

## Historical pre-merge checkpoint — PR #2 review follow-up (state recorded 2026-10-22; superseded by merge)

This section preserves the pre-merge checkpoint as it was recorded. PR #2 later merged on 2026-10-22 at 11a731c82d999366f287479e7b2afda46898570f; integrated merge-commit CI passed as stated in the current phase above.

- Baseline under review: PR #2 head `21eaa03b314d479d6524dba34d0c6bc34be8368a`;
  branch `codex/spec-05-maintenance-01-logic-integrity` in the managed worktree.
- F-01: confirmed and fixed in source commit
  `61d111dff35a5586a490d496868309ca68557097`. RED/GREEN tests cover an actual
  nested Git repo, a linked worktree `.git` file, an ordinary subdirectory,
  unresolved cwd, and symlink escape. Windows denied symlink creation locally;
  that regression is expected to run on Linux CI.
- F-02: confirmed and fixed in the same source commit. Parsing requires the
  `## Summary` boundary and considers only header status; evidence and affected
  file bullets remain unchanged. Duplicate header status remains an error.
- F-03: test gap closed without production changes. Added successful replay and
  external-edit rejection at all three missing journal boundaries; existing
  transaction code passes the matrix.
- Adjacent trace-score idempotency defect: repeated public scoring appended a
  newline. A RED/GREEN byte-stability regression and fix are committed as
  `195bdf7a1ffae166e67e9fdb5436098598678da1`; both trace copies and indexes
  remain byte-identical after the second score.
- Proof status: focused RED/GREEN regressions pass; the full workspace suite
  passes. No project-scoped Baron proof/trace was created. Fixtures use
  temporary repos, Vaults, sessions, and `BARON_HOME`.
- Local gates pass: fmt; full workspace all-target tests; warnings-denied
  Clippy; locked release build and CLI version; CLI help, JSON, 32
  repository-relative links; retired-adapter and diff checks. Exact-head CI
  and fresh independent review are pending after push.
- Safe next action: push the source and evidence checkpoint to the existing
  PR #2, verify CI on that exact head, and request fresh review. Stop before
  merge or release.

Plan: [PR #2 follow-up plan](2026-10-22-pr2-review-followup-21eaa03b.md); evidence: [follow-up verification ledger](2026-10-22-pr2-review-followup-21eaa03b-verification.md).

## Historical checkpoint — SPEC-05 Maintenance 01 PR #2 R01-R03 review fixes (HEAD 21eaa03b — SUPERSEDED — 2026-10-22)

- Baseline: `main` / `origin/main` at `9038afa9f840e3a62acc0f97c6aaa31849247525`.
- Branch: `codex/spec-05-maintenance-01-logic-integrity`; isolated worktree.
- Scope completed: Maintenance 01 LOG-01..LOG-06, G-01 recovery coverage, and
  the sibling audit are recorded in the implementation plan and evidence
  ledger. This checkpoint addresses the new R01/R02/R03 findings in the
  independent PR #2 review.
- R01: session imports use exact context metadata fields, canonical path
  identity with case preserved, and safe migration of a unique legacy folded
  state key. RED/GREEN tests cover message-text false binding, case-distinct
  sources, and an edited note during state-key migration.
- R02: closure reads one valid `Status` field in the finding header before
  `## Summary`, then replaces only that field. Interrupted closure publication
  with the Vault closed and repository open remains repairable. RED/GREEN tests
  cover summary spoofing and the interrupted projection.
- R03: a bounded pending journal stores prior and target bytes for both trace
  mirrors and both indexes. Recovery validates all targets before completing
  missing writes, preserves unrelated rows, and runs before mirror preflight.
  A private writer failpoint and a Windows file-lock injection reproduce the
  interrupted publication; retry recovery passes.
- Source commit: `8ea372bfc5e7ecef961fb9eefce92e2219445d1b`. Local checks passed:
  fmt, all workspace all-target tests, warnings-denied Clippy, locked release
  build, release CLI `baron 5.0.0`, and the exact case-sensitive session test
  after the final helper cleanup. The workspace suite ran before the
  test-helper-only lint cleanup; the affected test, fmt, Clippy, and release
  build passed after it. Fixtures use temporary BARON_HOME and session roots.
- The earlier ACCEPT review predates this source commit. The updated PR #2
  head needs hosted CI verification and independent review. Source/evidence
  head `b9cbbecb0ede08516590ff0d784693936e5cb79d` was pushed; its Baron CI run
  was queued before this status-document update. The production-tree sweep
  remains lexical plus targeted semantic tracing, not a
  line-by-line review of every match. No project-scoped Baron proof or trace
  was created for the user's unrecovered identity.
- Persisted-state boundary: use temp repositories, Vaults, session roots, and
  BARON_HOME only. Preserve the user's original checkout and machine state.
- Safe next action: verify hosted checks on the latest PR #2 head and await
  independent review of R01-R03; stop before merge pending owner approval.

Plan: [SPEC-05 Maintenance 01 logic integrity](2026-10-21-spec-05-maintenance-01-logic-integrity-plan.md); verification: [evidence ledger](2026-10-21-spec-05-maintenance-01-logic-integrity-verification.md).

## Historical pre-integration checkpoint — SPEC-05 post-closure hardening (IN PROGRESS at 2026-10-18; later merged through PR #1)

This section preserves the pre-integration state. PR #1 later merged on 2026-10-21 at 9038afa9f840e3a62acc0f97c6aaa31849247525; the integrated PR #2 merge commit 11a731c82d999366f287479e7b2afda46898570f passed hosted CI. Residual limitations remain listed here and in the current phase above.

This follow-up is separate from the original SPEC-05 closure. The accepted
historical range and CLOSED status remain intact. Branch
fix/spec-05-post-closure-hardening source checkpoint is
e452b62a4aac08def36e51af1d559679d73bd1b3.

- H-01 is a confirmed progress-publication bug, fixed with recoverable intent
  and four subprocess interruption regressions. A follow-up test exposed
  start/resume rejecting the local/Vault divergence before journal replay
  after an `after_local` interruption; recovery now runs before mirror
  validation, and that regression passed RED/GREEN.
- H-02 is a confirmed Windows case-sensitive path-collision bug, fixed with a
  case-preserving, domain-separated owner hash only when an ancestor enables
  case sensitivity. Case-insensitive paths retain legacy owner IDs. RED/GREEN
  tests cover `Repo` versus `repo`, aliases, and legacy hash stability.
  Relocation remains fail-closed; restore the original canonical path to resume
  plans with existing owner IDs. Pre-fix plans under a case-sensitive Windows
  ancestor keep the ambiguous folded v1 ID and fail strict validation after
  the hash change; recovery needs a future explicit audited migration.
- H-03 live hook claims are retained, all-live capacity fails closed, and
  malformed journal rows in the inspected tail fail before hook publication.
  Replay evidence remains bounded to 256 cache entries and 512 journal lines.
  Stop replay after expiry and concurrent shared-Vault publication have
  additional integration coverage. A review found unbounded reads of the
  growing journal; reverse reads are now limited to 4 MiB and the selected
  512-line window, while append boundary detection reads one byte. Oversized
  unresolved tails fail closed. Malformed-prefix and bounded-I/O regressions
  were observed RED then GREEN.
- Independent final-range review verified HEAD
  `74c231269d2042e5bf0dc3d3a97f2ff9af04b972`, the 95-commit/102-path range,
  the plan/spec, and `AGENTS.md`. It found no Critical or Important findings.
  Two Minor notes remain: the diagnostic `automation_status` path reads its
  full journal, and pre-fix plans under case-sensitive Windows ancestors need
  a future explicit audited migration. The review was scoped, not line-by-line.
- `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast -j 1`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo build --workspace --release --locked` pass at source checkpoint
  `e452b62`; release CLI reports `baron 5.0.0`. Workspace tests used a
  temporary BARON_HOME and bundled PowerShell 7. Hosted CI remains pending.
- The original project identity, project/local TOML, machine config, Vault and
  capsule were not recovered; no replacement was created. An earlier
  workspace test invocation used the host default machine home; possible
  per-user test state is recorded in the verification ledger. The latest full
  workspace run used a temporary BARON_HOME. No original-project proof or
  trace was produced.
- Tested source checkpoint `e452b62a4aac08def36e51af1d559679d73bd1b3` is 94
  commits ahead, 0 behind, and 102 paths from origin/main. Final reviewed HEAD
  `74c231269d2042e5bf0dc3d3a97f2ff9af04b972` includes the documentation evidence
  commit and is 95 commits ahead with the same 102 paths. The reviewed branch
  push completed. PR creation is blocked by GitHub connector 403 and a
  signed-out in-app browser; no hosted PR checks have run.

Plan: [SPEC-05 post-closure hardening](2026-10-18-spec-05-post-closure-hardening.md); verification: [evidence ledger](2026-10-18-spec-05-post-closure-hardening-verification.md).

Checkpoint: H-01/H-02/H-03 fixes and local verification are complete at
`e452b62`. The first review's two Important findings have RED/GREEN regressions
and fixes; final review found no Critical or Important findings and documented
two Minor notes. PR creation and hosted PR checks remain outstanding; the GitHub
connector lacks write access and the in-app browser is signed out.
Trace/proof against the unrecovered original project identity remains absent.
Changes are limited to the managed worktree and temporary fixtures; original
checkout, Vault, and per-user authority seed remain untouched. Safe next
action: after GitHub PR write access is available, open the PR and verify
required checks on its exact head. Wait for user approval before merge.

## SPEC-05 final closure (CLOSED — 2026-10-10)

The shared-Vault source repairs are implemented and locally verified. Identified plan and ACTIVE records carry a hashed canonical checkout owner; stale mirror divergence and foreign pending-journal recovery fail closed; orphaned plans prevent duplicate operation identity; exact-owner recovery works when a shared ACTIVE row is missing. Generic reconcile now recognizes only a matching validated indexed plan.

Accepted source range: `ecaf362c275431edfc5e590fd6a87d926b9bde9d..af2e1db020355af4fe0f621e8de3f8f780fbd2ad`. Fresh all-target gates: Core 641/0/0 across 60 suites; CLI 190/0/1 ignored across 38; adapters 124/0/3 ignored; full workspace command exit 0. Focused matrix: Core 327/0 across 23 targets plus repeated harness-improvement (340 executions); required CLI selector/evidence/hooks 38/0 plus migration CLI 3/3. fmt, Clippy, locked release, smoke 1/1, public docs 12/12 and 48-path authority audit (BUG=0) pass. Default PowerShell lifecycle is 4/7 due to the unavailable archive module; bundled PowerShell 7 passes 7/7.

- Execution plan: [SPEC-05 final closure verification](2026-10-06-spec-05-final-closure-verification-plan.md); evidence ledger: [2026-10-06 final verification](2026-10-06-spec-05-final-verification.md); authority audit: [current caller inventory](2026-10-01-spec-05-authority-audit.md).
- Authority boundary: exact lifecycle identity and persisted binding → indexed ACTIVE → canonical managed frontmatter. CURRENT stays a presentation projection for identified work; ambiguous legacy selectors fail closed. Shared Vault journal operations follow checkout → Vault locking.
- Lifecycle baseline: default host PowerShell is 4/7 because Microsoft.PowerShell.Archive cannot load; bundled PowerShell 7 is 7/7. This environment-specific failure is disclosed, not counted as a product pass.
- Scope: no release, tag, version bump, SPEC-06, Hotel Staff, or manifest/lockfile edits. Public version remains `5.0.0`.
- Exact-SHA GitHub Actions [run 38014117177](https://github.com/thienty1207/Baron-Engine/actions/runs/38014117177) succeeded on `af2e1db020355af4fe0f621e8de3f8f780fbd2ad`, including Format/Clippy and Linux/Windows native tests.
- Fresh read-only reviewers Nietzsche and Dewey independently returned ACCEPT for that same complete range; both assessed all 23 checklist items and reported zero Critical/Important findings. Reviewers did not run tests.
- Status is CLOSED. Closure evidence is documentation-only after the accepted source head. No release/tag/version bump, SPEC-06, Hotel Staff, manifest/lockfile, or history rewrite is in scope.
- Closure evidence commit `32a92331716d1c26f69bed1e32312a7b4981d0f6` was pushed and verified against the remote branch SHA; executable source remains the reviewed `af2e1db` candidate.
- Safe next action: normal Baron 5.0.0 maintenance; no SPEC-05 closure gate remains.

## Baron 5.0.1 Stabilization - SPEC-05 Multi-Agent Concurrency and Durable State (ready for adversarial review, 2026-09-29)

- Active spec: Multi-Agent Concurrency and Durable State.
- Status: `SPEC-05: READY FOR ADVERSARIAL REVIEW`; this is not a closure
  claim. SPEC-04 is closed at the `ecaf362` closure base.
- Scope: serialize the owned shared mutation surfaces for B-09, B-10, B-11,
  B-18, B-25, B-26, B-27, W-05, and W-08 using the existing project-scoped
  `safe_io` lock. Keep child processes, network calls, probes, and slow
  context/prepare work outside the lock; preserve legacy readability and
  SPEC-03/SPEC-04 authority boundaries.
- Implementation checkpoint: proof/trace/plan publication, config and
  first-init, harness and improvement, intent/recovery, control-plane,
  review-gate/experiment, automation dedup publication, capability registry,
  runtime JSONL, and cache semantics are implemented and covered by the
  deterministic multiprocess matrix. Provider fingerprints prevent stale
  capability observations from attaching to a re-registered definition.
  Identified active plans use a managed per-operation `ACTIVE.md` lookup while
  `CURRENT.md` remains a presentation projection. Plan lifecycle, trace, and
  identified Stop/reconcile mutations now resolve the exact full operation
  identity; legacy ambiguous mutations fail closed.
- Proof checkpoint: focused concurrency `16/16`, config `16/16`, plan `51/51`,
  proof/trace `29/29`, capability `9/9`, automation `6/6`, harness `5/5`,
  harness improvement `5/5`, intent `7/7`, continuity `6/6`, control-plane
  `9/9`, migration `10/10`; retired-adapter gate `8/8`.
- Adversarial checkpoint: the first fresh review identified Important issues
  in lock scope, plan-index history, config preservation, capability cache
  identity, and legacy artifact selection; RED-to-GREEN fixes are in
  `34de705` and `0c12c0c`. Follow-up commits `cfaec117`, `318594c`,
  `674a059`, and `7a6b473` close publication-authority, concurrent plan/config,
  continuity, hook-claim, and receipt-freshness boundaries. Commit `eeae7d8`
  restores public config `Eq` compatibility, preserves the public observation
  shape while fingerprinting capability cache evidence, narrows migration lock
  scope, and makes post-handoff rollback fail closed on changed paths.
  `cc6fbfc` reserves migration backup roots under the project lock after a
  self-audit found an `exists-check → create` race. Fix commit `d161d05`
  closes the interleaved-plan path bug, ignores foreign/invalid proof/trace
  Markdown, preserves create-new success after staging cleanup failure,
  recovers hook responses from a journal publication crash, and snapshots the
  migration handoff baseline under one bounded lock. Repair commit `ac11117`
  closes the remaining operation-authority gap by removing singleton
  `CURRENT.md` selection from exact lifecycle, operation-bound trace, and
  identified Stop/reconcile paths; legacy ambiguous mutations fail closed.
  The updated review range is `ecaf362..ac11117`; independent acceptance
  remains unverified.
- Residual classification: the hook journal/dedup pair remains a bounded
  cross-file crash-recovery seam for SPEC-06; published responses are
  recoverable and journal data is diagnostic-only. External migration
  installation remains unlocked by design; backup reservation and
  post-handoff hashes fail closed, while full multi-migration handoff
  serialization belongs to SPEC-06.
- Scope boundary: context/session/replay/cache writers and diagnostic
  SQLite/index accelerators are out of SPEC-05 scope and mapped to B-28..B-31;
  automation hook claim/journal publication remains a locked SPEC-05 mutation.
- CLI identity evidence: exact update/interrupt/complete and legacy ambiguity
  fail-closed tests pass `4/4` in `baron-cli --test plan_identity_cli`.
- Workspace evidence: `cargo test --workspace --all-targets --no-fail-fast
  -j 1` exits 0 with isolated session roots and the supported PowerShell 7
  test-host override. Current lifecycle scripts pass `7/7`; current
  `prepare_cli` passes `6/6`; the UTF-8 replay regression passes `5/5`.
  Formatter, workspace warnings-denied Clippy, locked release build, explicit
  release smoke, release binary `baron 5.0.0`, status JSON parsing, maintained
  docs tests, and diff checks pass.
- Persisted-state boundary: only SPEC-05 source/tests, this active plan, and
  maintained status/build-log evidence are in scope. Project/Vault data,
  unrelated user files, release metadata, and public version remain untouched.
- Safe next action: commit the verified repair/evidence and obtain two fresh
  independent reviews over the final `ecaf362..HEAD` range. Keep the public
  version at `5.0.0`; do not release or tag. Do not close SPEC-05 until both
  reviewers accept.

## Baron 5.0.1 Stabilization - SPEC-04 Managed Plan-Path Authority Fix (closed, 2026-09-22)

- Active spec: Proof, Trace, Gate, and Completion Integrity.
- Status: `SPEC-04: CLOSED`; the source/public version remains `5.0.0`, with
  no tag, release, or version bump.
- Closure source/evidence HEAD: `250989011f5b5ecda8741582a36c031c248c1d3e` on
  `codex/spec-04-proof-trace-gate-completion-integrity`.
- Closure basis: B-08, B-17, and B-32 are closed; the managed plan-path
  authority final fix was accepted by independent adversarial review, and no
  SPEC-04-owned issue remains.
- Delivered: CURRENT `- Plan:` pointers are confined to the exact
  `docs/baron/plans/` root using canonical containment. Symlink/reparse,
  non-regular, missing, outside-root, and nested-prefix targets fail closed;
  `vault_plan_path()` has no outside-root fallback.
- Plan document contract: linked plans require leading frontmatter with
  `type: baron-plan`; authority fields are read only from frontmatter and
  duplicate known fields fail closed while benign unknown fields remain
  tolerated.
- Mutation evidence: hostile pointers cannot authorize or mutate README,
  `src/fake.md`, unrelated docs, `docs/baron/plans-evil/`, or their Vault
  mirror. Missing type, body-only metadata, duplicate risk, duplicate
  operation ID, and link/reparse regressions are covered. The valid high-risk
  proof/trace completion path remains green.
- Focused verification: plan 44/44, automation 5/5, proof_trace 16/16,
  operation_identity 10/10, phase12_hooks_cli 4/4, and execution_cli 15/15;
  Core all-targets, formatter, warnings-denied Clippy, locked release build,
  `baron 5.0.0`, status JSON parsing, and diff check pass.
- Full workspace verification with serial Cargo jobs retains only the two known
  unrelated blockers: missing `Microsoft.PowerShell.Archive` in
  `lifecycle_scripts` and the existing `session_replay` UTF-8 boundary panic in
  `prepare_cli` at `crates/baron-core/src/session_replay.rs:383`.
- Active plan:
  `docs/superpowers/plans/2026-09-22-spec-04-managed-plan-path-authority-final-fix.md`.
- Residual limitation: the current Markdown plan schema has no independent
  persisted origin record for a fully recomputed edit of both files; canonical
  derivation still rejects the reviewed A-to-B bypass. Durable origin binding is
  deferred for a later scope.
- Next action: create `codex/spec-05-multi-agent-concurrency-durable-state`
  from this closure commit. Do not release, tag, bump the version, or modify
  Hotel Staff.

## Baron 5.0.1 Stabilization - SPEC-02 (2026-09-20)

- Active spec: Operation Identity & Lifecycle Wiring.
- Status: `closed; final review fixes implemented and verified; unrelated host
  blockers remain`; the source/public version remains `5.0.0` and no release,
  tag, or version bump is authorized by this work.
- Baseline: completed SPEC-01 branch HEAD `adccb6e`; work is isolated on
  `codex/spec-02-operation-identity`.
- Scope: introduce one checked lifecycle identity, resolve missing Prepare IDs
  once at ingress, unify task identity with Task State and Plan writes, preserve
  Codex and Claude native-hook parity, make request operation derivation fail
  closed, and require complete non-blank identity for supported CLI plan
  creation.
- Known unrelated verification blockers carried forward: the host cannot load
  `Microsoft.PowerShell.Archive`, and `session_replay` has an existing UTF-8
  boundary panic. Neither is part of SPEC-02.
- Active plan:
  `docs/superpowers/plans/2026-09-20-spec-02-final-review-fix.md`.
- Verification: final review suites cover forged operation/context identity,
  task mismatch before Plan writes, incomplete request identity, identified
  hook retries, anonymous delivery isolation, and empty CLI IDs. Core
  all-targets, identity/Prepare/plan/automation/hooks/control-plane focused
  suites, CLI plan/execution/control-plane suites, formatter, warnings-denied
  Clippy, locked release build, binary version smoke, and diff checks pass. The
  full workspace retains only the two known blockers: three installer tests
  cannot load `Microsoft.PowerShell.Archive`, and one Prepare CLI test reaches
  the existing `session_replay` UTF-8 boundary panic. The adversarial closure
  scan found no remaining SPEC-02-owned issue.
- Implementation commits: `b58c701`, `4ee56e4`, `3771df8`, `95a98a7`,
  `1df57ef`, and `9b589ec`.
- Safe next action: push the completed SPEC-02 final review-fix branch once; do
  not release, tag, bump the version, or begin SPEC-03/SPEC-04.

## Baron 5.0.1 Stabilization - SPEC-01 (closed, 2026-09-20)

- Active spec: Graphify Provider Contract Stabilization.
- Status: `closed; final review fix implemented; unrelated verification blockers remain`;
  the source/public version remains `5.0.0` and no release, tag, or version
  bump is authorized by this work. The user has explicitly authorized the
  SPEC-01 branch commit and push after verification.
- Plan: `docs/superpowers/plans/2026-09-20-graphify-provider-contract.md`.
- Evidence: real Graphify `0.9.25` writes
  `<out>/graphify-out/graph.json`; Baron now has RED tests for that boundary
  and the Option B local-artifact query design is selected. Final review fixes
  harden traversal confidence, isolate the missing-provider CLI test from host
  PATH, and add a mandatory `graphifyy==0.9.25` protected release gate.
- Focused evidence: Graphify provider `6/6`, CLI code-map `1/1`, real local
  Graphify `1/1`, workflow `actionlint` clean. The two known full-workspace
  blockers remain outside SPEC-01: `Microsoft.PowerShell.Archive` and the
  existing `session_replay` UTF-8 boundary panic.
- Git checkpoint: review-fix commit `bac54f3` is pushed on
  `codex/spec-01-graphify-provider-contract`.
- Safe next action: observe the protected certification gate on any release
  attempt and defer the two unrelated full-workspace blockers.

## Historical checkpoint — Phase 16 v5.0.0 release candidate (superseded by local v5.0.1 candidate on 2026-10-24)

> The commit, push, tag, and publication instructions in this checkpoint are historical 5.0.0 release-candidate steps. The active state is the unpublished 5.0.1 source candidate at the top of this file.

- Active phase: Baron Codex + Claude Core Consolidation - Phase 16 release
- Status: `release candidate ready for hosted verification`
- Plan: `docs/superpowers/plans/2026-09-09-final-adversarial-verification-phase16.md`
- Scope: complete the verified 5.0.0 candidate through commit, hosted CI,
  protected signing preflight, tag, and GitHub Release.
- Current work: the authenticated manifest/updater/installers, typed receipt
  authority, exact production identity, and all Phase 16 regression suites are
  green after the 4.2.2 to 5.0.0 version bump. The clean release binary is
  built and the source remains uncommitted.
- Next action: inspect and commit the candidate, fetch and compare remote main,
  push without force, then observe the actual hosted workflows before tagging
  or publishing.

Last updated: 2026-09-20

## Historical Phase 16 release-track snapshot

- Active phase: Baron Codex + Claude Core Consolidation - Phase 16
- Status: `release candidate ready for hosted verification`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-09-final-adversarial-verification-phase16.md`
- Scope: independent final adversarial verification of the completed Phase
  1–15 source, followed by a release only if every local and hosted gate passes.
- Constraints: preserve user state and the zero retired-adapter guards; do not
  start a later phase; commit, push, tag, and publish only after the complete
  hosted release gate remains green.
- Initial evidence: the clean 5.0.0 candidate, formatter, compatible workspace
  tests, Clippy, release build, final-binary smoke, and fresh native security
  review pass; three ignored Phase 1 fixtures remain historical behavior.
- Next action: inspect and commit the candidate, compare and push `main`, then
  observe the actual hosted workflows before tagging or publishing.

### Phase 16 security remediation checkpoint (completed)

- SEC-03 and SEC-04 are closed locally at Baron 5.0.0. TAC is unavailable on
  this host. The remotely fetched bootstrap-script self-authentication limit is
  documented as an explicit trust boundary.
- The version bump is complete; the candidate remains uncommitted until the
  final remote comparison and hosted release workflow checks are complete.

### Phase 16 repair checkpoint

- A Windows junction fixture caught initialization writing through a linked
  `.baron` directory before rejection. The initialization path now validates
  the directory chain before mutation, and durable Core/config/Vault writers
  use safe replacement and explicit read errors.
- The new linked control-plane fixture remained red until its writer was
  hardened; both current Windows adversarial tests pass. Release metadata and
  Git history remain untouched pending all gates.
- Final decision: local adversarial and release checks pass; SEC-01/02/05,
  SEC-03, and SEC-04 are repaired and tested. The next boundary is clean target
  deletion, final release build, and hosted CI; no Phase 17 work has started.

Last updated: 2026-09-09

The remaining entries are historical phase plans and are retained for audit.

## Historical refactor snapshot — Phase 15

- Active phase: Baron Codex + Claude Core Consolidation - Phase 15
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-08-final-documentation-product-model-phase15.md`
- Scope delivered: final public product model, README, maintained architecture
  and compatibility docs, generated Codex/Claude contract audits,
  command-surface positioning, changelog, historical framing, and docs tests.
- Constraints: do not begin Phase 16 adversarial verification, release, tag,
  push, publication, version bump, or history rewrite. Preserve current source
  behavior and zero retired-adapter gates.
- Verification: focused documentation/generated-contract suites, relevant
  adapter and bridge regressions, CLI help, links, status JSON, formatter,
  workspace tests, Clippy, diff checks, and zero retired-adapter gates pass.
- Next action: prepare Phase 16 adversarial verification only; do not release,
  tag, push, publish, bump the version, or rewrite history.

The remaining entries are historical phase plans and are retained for audit.

Last updated: 2026-09-08

## Historical refactor snapshot — Phase 14

- Active phase: Baron Codex + Claude Core Consolidation - Phase 14
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-08-update-downgrade-migration-release-phase14.md`
- Scope: persisted-state compatibility, safe update/downgrade/migration,
  crash recovery, rollback, dry-run non-mutation, Core/projection coherence,
  and local release-package proof.
- Constraints: do not begin Phase 15 or Phase 16, bump version, publish,
  tag, push, or rewrite history. Preserve user-owned project, Vault, managed,
  task, continuity, journal, dedup, and Autopilot state.
- Dependency findings: managed-state v1/v2 and safe-I/O transaction rollback
  already had strong foundations; direct project writers now fail closed on
  future schemas, and completion receipts now carry recovery evidence after a
  committed runtime checkpoint.
- Verification: Phase 14 focused suites, the complete workspace matrix under
  system Windows PowerShell, release build/binary smoke, formatter, Clippy,
  diff checks, and retired-adapter gates pass. The default Codex-bundled
  PowerShell runtime still blocks two installer tests at archive creation.
- Next action: prepare the Phase 15 documentation rewrite only; do not begin
  Phase 16 or release work.

Last updated: 2026-09-08

## Historical refactor snapshot — Phase 13

- Active phase: Baron Codex + Claude Core Consolidation - Phase 13
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-08-autopilot-ux-phase13.md`
- Scope: safe untrusted Autopilot candidates, bounded housekeeping, and
  project-scoped conversational approval over existing Core authorities.
- Constraints: do not change Project, managed-state v2, memory, or
  PreparePacket v1 schemas unless a minimal additive Autopilot field is proven
  necessary. Do not mutate Core assets, auto-promote candidates, begin Phase
  14+, bump version, release, tag, push, or rewrite history.
- Verification: Phase 13 Core `14/14`, CLI `2/2`, all prior regression suites,
  workspace formatter, tests, Clippy, diff checks, and retired-adapter gates
  pass. The workspace test sweep under the compatible system Windows
  PowerShell module path has zero failures and three intentionally ignored
  historical fixtures.
- Next action: prepare the Phase 14 plan only; do not begin Phase 14 in this
  checkpoint.

Last updated: 2026-09-08

## Historical refactor snapshot — Phase 12

- Active phase: Baron Codex + Claude Core Consolidation - Phase 12
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-08-native-hook-idempotency-phase12.md`
- Scope: native Codex/Claude hook acceleration, normalized lifecycle events,
  durable idempotency, parent/child boundaries, and reliable checkpoints;
  AGENTS.md/CLAUDE.md remain the correctness fallback.
- Implementation: SessionStart, UserPromptSubmit, PreCompact, and Stop now
  use one bounded Core lifecycle path with structured stdin, event-key dedup,
  durable append, parent-owned child evidence, recursion protection, and
  additive event-keyed continuity checkpoints. Codex and Claude projections
  preserve third-party hook/settings content and expose normalized accelerators.
- Verification: Phase 12 Core `9/9`, adapter `3/3`, and CLI `4/4` focused suites
  pass; relevant prior-phase regressions pass. The default Codex PowerShell
  runtime still blocks two installer tests before assertions, while the same
  lifecycle suite passes `5/5` with system Windows PowerShell. Formatter,
  workspace Clippy, diff, status JSON, and retired-adapter grep gates pass.
- Constraints: no Phase 13 Autopilot UX, Phase 14 release hardening, Phase 15
  broad documentation rewrite, Phase 16 final verification/release work,
  version bump, release, tag, push, or history rewrite.
- Evidence before Phase 12: existing hooks preserve third-party JSON but
  automation journal append rewrites the full file and has no event-key dedup;
  continuity checkpoints have no event-key byte-stability guard.
- Next action: prepare the Phase 13 plan only; do not begin Phase 13 in this
  checkpoint.

Last updated: 2026-09-08

## Historical refactor snapshot — Phase 11

- Active phase: Baron Codex + Claude Core Consolidation - Phase 11
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-08-final-supported-adapter-cleanup-phase11.md`
- Scope: remove unsupported adapter product surfaces and preserve only Codex
  and Claude as active integrations; retain generic opaque legacy parsing.
- Evidence before cleanup: 476 tracked content match lines and 3 tracked
  filenames contain the retired adapter token.
- Verification: Phase 11 focused target, Core, adapter, and CLI regression
  suites pass; the workspace has no product failures. Two Windows installer
  tests remain host-limited by unavailable `Microsoft.PowerShell.Archive`.
  Formatter, Clippy, diff, status JSON, content, and filename gates pass.
- Next action: prepare Phase 12 only; do not begin Phase 12 in this
  checkpoint.

Last updated: 2026-09-08

## Historical refactor snapshot — Phase 10

- Active phase: Baron Codex + Claude Core Consolidation - Phase 10
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-08-remove-global-adapter-authority-phase10.md`
- Verification: Phase 10 Core `14/14`, CLI `8/8` including cross-adapter
  resume, all Core tests, formatter,
  workspace Clippy, diff, and status JSON checks pass. The workspace sweep has
  only the two known Windows installer failures caused by unavailable
  `Microsoft.PowerShell.Archive` support.
- Next action: prepare the Phase 11 plan; do not begin Phase 11 in this
  checkpoint.

Last updated: 2026-09-08

## Historical refactor snapshot — Phase 8

- Active phase: Baron Codex + Claude Core Consolidation - Phase 8
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-07-codex-native-thin-bridge-phase8.md`
- Next action: prepare the Phase 9 plan; do not begin Phase 9 in this
  checkpoint.

Last updated: 2026-09-07

## Historical focus snapshot

- Active phase: normal Baron 4.2.2 maintenance after public patch release
- Status: `completed`
- Design: `docs/superpowers/specs/2026-08-20-multi-agent-core-parity-design.md`
- Plan: `docs/superpowers/plans/2026-08-20-multi-agent-core-parity.md`
- Next action: normal `4.2.2` maintenance; preserve the shared core contract
  and rerun the cross-adapter parity/preservation suite for future changes.

## Historical release snapshot — Baron 4.2.2

- Active phase: Baron 4.2.2 public patch release
- Status: `completed`
- Design: `docs/superpowers/specs/2026-08-20-baron-4-2-2-release-design.md`
- Plan: `docs/superpowers/plans/2026-08-20-baron-4-2-2-release.md`
- Next action: normal `4.2.2` maintenance; no release phase remains open.

- Final commit: `80f4daa13e15b2aafb53bbefe0f54454130fcaba`.
- Final tag: `v4.2.2`; CI run `32347122499` and Release run `32347138548`
  passed; `releases/latest` resolves to `v4.2.2` with 12 assets.

- Previous completed phase: Baron 3.6 final public release
- Previous status: `completed`
- Verification: same-name project isolation, Vault memory exclusion, 6,100+
  mixed-language legacy repository bounds, failure fallback, hook/instruction
  preservation, full workspace tests, Clippy, locked release build, release
  binary smoke, and public installer smoke passed. GitHub Actions run
  `30246729740` passed exact-source verification and all four native targets
  before immutable promotion to `v3.6.0`.
- Next action: no active release work. Follow the README install/reinstall flow
  for new or restored Windows machines.

## Release Repair Checkpoint

- Root cause: the Unix-only runtime activation call used
  `CandidateBinaryInspector::reported_version` without putting the trait in
  scope. GitHub Ubuntu caught this before promotion; Windows did not compile
  the Unix-only branch.
- Repair: one shared exact-version helper is now called by the Unix activation
  branch, with a RED/GREEN test for accepted and rejected runtime versions.
- Follow-up root cause: the Linux self-update integration test copied its debug
  binary as a fake candidate. That binary exceeded the real 128 MB safety cap
  before the intended version-mismatch assertion could run. The test now uses
  a bounded executable wrapper on Unix; production candidate-size limits are
  unchanged.
- Final fixture audit: the Unix update-transaction fixture had the same issue
  with a patched debug binary. It now stages a bounded wrapper which delegates
  to the patched backing binary, preserving the verified-candidate protocol
  without weakening the production size boundary.
- Fourth promotion result: GitHub Ubuntu passed the complete test suite, then
  Clippy stopped before tag creation because an empty Unix-only test and two
  Windows-only runtime owners were compiled on Linux. The placeholder test is
  removed and the runtime-only state path/helper are now explicitly scoped to
  Windows; no production release safety boundary was relaxed.
- Local evidence: the targeted regression, full workspace suite, Clippy, and
  locked release build passed again after the fourth repair. Workflow YAML and
  status JSON validation also passed. A release-binary Vault/Codex/fullstack
  certification smoke also passed. A Windows-to-Linux cross-check cannot
  complete on this machine because `x86_64-linux-gnu-gcc` is not installed; the
  required GitHub Ubuntu build remains the authoritative cross-platform proof.
- Final release evidence: GitHub run `30246729740` passed Ubuntu verification,
  Windows, Linux, macOS Intel, macOS Apple Silicon, and immutable promotion.
  Tag `v3.6.0` resolves to `c89486694d9a4431e04106274d0c9f997db42683`; a fresh
  Windows install from `releases/latest` returned `baron 3.6.0` and passed
  `setup`, `init --codex --fullstack`, and `context`.

## Baron 3.4 Contract

- Public `baron update` is the human-authorized complete update flow.
- `assets/core/` is the sole managed runtime source; stale
  `blueprints/core/` is removed before the first managed baseline.
- AI agents use local-only `baron automation reconcile` and cannot silently install releases.
- Baron records the last installed managed baseline before making update decisions.
- Ambiguous local/upstream edits become staged conflicts, not live overwrites.
- Custom skills, custom agents, project source, and Vault memory remain user-owned.
- A verified candidate renders new managed assets.
- Project and runtime activation are one recoverable transaction.
- Immutable exact-source release promotion remains unchanged.
- Vault Markdown remains memory source of truth.
- Superpowers and the three core quality agents remain unchanged.

## Program Queue

- Baron 3.4, Phases 35-38: one runtime source and safe recoverable update.
- Baron 3.5, Phases 39-41: certified local skill intelligence with no duplicate
  workflow.
- Baron 3.6, Phases 42-43: provider-neutral project-scoped cache/identity
  contract plus exact local-only adapter completed with deterministic fallback
  evidence.
- Baron 3.6, Phase 44: bounded task routing, hidden AI automation, query
  cache, and source verification completed without startup blocking.
- Baron 3.6, Phase 45: strict isolation and certification completed.
- Total remaining planned phases: 0.

## Historical document index

- Program design:
  `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`
- Master program:
  `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`
- Design: `docs/superpowers/specs/2026-07-23-baron-3-4-safe-self-update-design.md`
- Baron 3.4 plan:
  `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
- Baron 3.5 plan:
  `docs/superpowers/plans/2026-07-24-phase-39-41-baron-3-5-skill-intelligence.md`
- Baron 3.6 plan:
  `docs/superpowers/plans/2026-07-24-phase-42-45-baron-3-6-code-graph.md`
- Current build log: `notes/build-log/CURRENT.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Follow RED/GREEN TDD for every production behavior.
- Keep Phase 35-42 evidence intact before provider invocation.
- Update the build log and status Markdown/JSON after every phase checkpoint.
- Do not mark a phase complete from test intent; record fresh command evidence.
- Keep the normal user command surface small.
- Do not invoke a graph provider outside the exact Phase 43 command allowlist.
- Do not start Phase 44 before Phase 43 provider safety evidence is recorded.
