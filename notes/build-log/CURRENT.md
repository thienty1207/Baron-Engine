# Current Build Note

## Baron 4.2 Implementation Checkpoint (2026-08-14)

- Owner approved the thirteen-phase Baron 4.2 program (Phases 88-100) and
  authorized implementation, private local evaluation, testing, GitHub
  publication, README synchronization, and final reinstall/rollback proof.
- Active design:
  `docs/superpowers/specs/2026-08-14-baron-4-2-practical-perfection-design.md`.
- Active plan:
  `docs/superpowers/plans/2026-08-14-baron-4-2-program.md`.
- Current phase: Phase 100 public release boundary. Local source/binary is
  `4.2.0`; public `releases/latest` remains `4.1.0` until the immutable tag,
  native assets, and reinstall proof finish.
- Continuity checkpoint: `notes/build-log/2026-08-14-baron-4-2-program.md`.
- Proof status: Phases 88-99 are checked in the status/plan and accepted by the
  bounded contract. Raw development runs are `100,100,100`; private holdout is
  `100/100` over 8 cases; promotion_ready is `true`.
- Acceptance artifacts: `docs/assessment/baron-4.2-acceptance.{json,md}`,
  contract `432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a`,
  source revision `545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d`.
- Verification status: `cargo fmt --check`, workspace check, warnings-denied
  Clippy, full CLI/core tests, and release build all passed. Trusted gate
  receipts for code-reviewer, security-auditor, and test-engineer passed in the
  disposable gate fixture outside the repository.
- First Phase 100 native workflow attempt was correctly rejected by the public
  trust-doc contract because `remainingPhaseCount` still described the old
  release-state convention. The status JSON is now aligned with the current
  4.2 target, and the complete local core (including Windows Graphify) and CLI
  suites pass again before the tag is recreated.
- The next native diagnostic exposed two Unix-only update-recovery fixture
  failures: the shell delegate candidate was created without executable mode.
  The fixture now sets mode `0755`, matching a real Unix raw candidate; this is
  a cross-platform test correction, not a gate relaxation.
- The first executable-mode patch kept a moved `PathBuf` behind the Unix-only
  cfg block, so Linux/macOS compilation caught it while Windows could not. The
  fixture now borrows for the write and moves only after permissions are set.
- The corrected candidate still reports the two Unix recovery tests as failed
  in the hosted verifier; CI now publishes the final test-log tail as
  annotations so the next run records the exact child-process error instead of
  only the test names.
- Safe resume point: run Phase 100 exact-source native CI/release, verify
  `releases/latest`, perform README-only Windows reinstall/4.1 rollback/4.0
  fallback smoke, then update this checkpoint and push the final docs.
- Fallback: keep the verified `v4.1.0` public baseline and explicit
  `BARON_ENGINE_GENERATION=4.0` per-query/whole-engine recovery path.

## Baron 4.1 Release Checkpoint (2026-08-14)

- Owner approved implementation of the eleven-phase Baron 4.1 program after
  removing automatic Skill creation/distillation from scope.
- Owner authorized the Baron-only 4.1 release: Tencent comparison is optional
  reference material and no longer blocks promotion. Baron 4.0 remains the
  explicit fallback.
- Active design:
  `docs/superpowers/specs/2026-08-13-baron-4-1-intelligence-evaluation-design.md`.
- Active plan:
  `docs/superpowers/plans/2026-08-13-baron-4-1-program.md`.
- Last successful checkpoint: Baron 4.1 source `6bea181` is public as tag
  `v4.1.0`; Baron CI #56 and Baron Release #24 passed exact-source,
  cross-platform, checksum, manifest, installer, and immutable-promotion gates.
- Current work: reconcile final public-release evidence in status files, retain
  the 4.0 fallback, and perform the requested disposable cache cleanup.
- Latest proof batch: `cargo check --workspace`, workspace Clippy with
  warnings denied, Baron core library tests, CLI integration tests, and the
  focused Vault/CodeGraph/context suites passed. The full workspace/all-target
  test command passed locally after the semantic candidate filter fix and also
  passed in the hosted Ubuntu release verifier.
- The latest complete `cargo test -p baron-cli --tests --no-fail-fast` run also
  passed all CLI integration groups (23 test binaries/groups, including
  lifecycle, memory, context, CodeGraph, release, and recovery coverage).
- The complete CLI suite and 28 Baron core tests also passed; the hosted native
  matrix covered Windows x64, Linux x64, Intel macOS, and Apple Silicon.
- The isolated seeded development-fixture test passes all five local surfaces
  at `100/100` without mutating a user's Vault; external comparison and
  confidence artifacts are non-blocking diagnostics.
- The frozen-contract Phase 86 runner now repeats the release binary three
  times, verifies the contract/source hashes, records raw failures, and accepts
  the Baron-only local gate without inventing Tencent data. Artifact:
  `docs/assessment/baron-4.1-phase86-runner.*`.
- Implemented candidate surfaces include deterministic lexical/vector/RRF
  retrieval, bounded session-learning candidates with no Skill creation,
  temporal ledger refresh/rollback, grounded handoff citations, semantic Wiki
  and CodeGraph ranking, poisoning quarantine, token/cost measurements, stale
  cache rebuild, and a default `BARON_ENGINE_GENERATION=4.1` path with
  explicit 4.0 and 3.8/baseline fallbacks.
- Latest clean benchmark report: `docs/assessment/baron-4.1-benchmark.json`
  (report `bbc04a89ef0102aa584f2e77710f5ea1029032a77873a578109ed77f7263ee9b`,
  contract `86054c9a45c7d61df91b8b1468ed13347ef96a66091f69a7c404c646dab62af2`).
  The seeded development fixture scored Memory `100/100`, Semantic/grounded
  synthesis `100/100`, Session learning `100/100`, Wiki `100/100`, and
  CodeGraph `100/100` on the release binary. Tencent and external confidence
  are optional and were not used as release gates. Total runtime was `4533 ms`
  (`2189 ms` indexing, `2343 ms` query), estimated 367 handoff tokens,
  `44,155,262` cache/artifact bytes, and `168,239,104` peak working-set bytes;
  the 10,000 ms query and 512 MiB memory budgets passed.
- Proof status: Baron-only Phase 86 and public Phase 87 are complete; a fresh
  public `install.ps1` smoke returned `baron 4.1.0` in an isolated directory.
  Broader real-corpus/scale hardening remains a non-blocking follow-up.
- Safe resume point: use the published `v4.1.0` baseline, retain the explicit
  `BARON_ENGINE_GENERATION=4.0` fallback, and clean only rebuildable/disposable
  caches. Optional Tencent diagnostics are not a release claim.
- Tencent inspection evidence is recorded in
  `docs/assessment/baron-4.1-tencent-v2.0.0-inspection.*`: tag `v2.0.0`
  resolves to commit `0aff21a2d9f2b8a0354aaa80a2e586aab4054562`, but the public
  README only exposes PersonaMem `48% -> 76%`, not the five per-surface scores
  needed for a same-corpus gate.

Date: 2026-08-14
Target: Baron 4.1.0 public release
Program target: Baron 4.1.0

Public release evidence:

- Source/tag: `6bea181044fa0d6f4a74195b8c7455eaa09fdf62` / `v4.1.0`
- Release: https://github.com/thienty1207/Baron-Engine/releases/tag/v4.1.0
- CI: https://github.com/thienty1207/Baron-Engine/actions/runs/31723285579
- Release workflow: https://github.com/thienty1207/Baron-Engine/actions/runs/31723297751

## Final Public Release Checkpoint (2026-08-13)

- Baron `v4.0.0` is public at
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v4.0.0`.
- Release source is exact commit
  `041564d7387e88f5894cb0f9cb3eecb1d328c5b7`, tagged `v4.0.0`; the release
  workflow is
  `https://github.com/thienty1207/Baron-Engine/actions/runs/31680499831`.
  The preceding native CI matrix is
  `https://github.com/thienty1207/Baron-Engine/actions/runs/31679762576`.
- Native CI passed Format/Clippy, full tests, release build, and smoke on
  Windows x64, Linux x64, Intel macOS, and Apple Silicon. The release job then
  passed exact-source verification, archive builds, SHA-256 checksums,
  `release-manifest.json`, installer lifecycle smoke, and immutable promotion.
- Public assets include four native archives and raw binaries, `SHA256SUMS`,
  `release-manifest.json`, `install.ps1`, and `install.sh`. The public
  `releases/latest` endpoint resolves to `v4.0.0`; the downloaded manifest
  reports version `4.0.0` and source `041564d…`, and the public Windows archive
  SHA-256 matches `SHA256SUMS`.
- Fresh Windows public install downloaded `install.ps1` from `releases/latest`,
  verified the archive checksum, installed `baron 4.0.0`, and passed `setup`,
  real `init --codex --fullstack`, `context`, memory index/recall, Wiki
  index/search, and CodeGraph index/query. A reinstall into the same temporary
  directory preserved the project marker and all Vault files (`14` before and
  `14` after). The installed runtime's same-version `baron update --codex`
  attempt refused the redundant `4.0.0` candidate without changing project or
  Vault state.
- Release scope is complete. Real-repository/large-corpus scale, durable
  temporal compaction, full adapter parity, and dynamic lab execution remain
  explicit follow-up limits; no release claim relies on those unimplemented
  paths.
- Final documentation synchronization commit `764baaced3275ac57d6b72a6c85c9e627620de84`
  is pushed to `origin/main`. Its follow-up CI run
  `https://github.com/thienty1207/Baron-Engine/actions/runs/31682195497` passed
  Format/Clippy and all four native runners. Documentation trust tests are 9/9,
  status JSON parses, and the working tree is clean.
- Next action: normal maintenance from the published `v4.0.0` baseline.

## Baron 4.0 Release Checkpoint

- Owner approval: explicit approval to implement Phases 65-76, run the full
  benchmark/test/certification program, publish `v4.0.0` to GitHub, and update
  the README with the newest install path.
- Baseline: Baron `3.8.0` remains the explicit recovery fallback; public stable
  and source release are now `4.0.0`.
- Release source `041564d7387e88f5894cb0f9cb3eecb1d328c5b7` is pushed to
  `origin/main`; the working branch retains the same source while final docs
  synchronization is prepared.
- Active design: `docs/superpowers/specs/2026-08-13-baron-4-0-intelligence-security-design.md`.
- Active plan: `docs/superpowers/plans/2026-08-13-baron-4-0-program.md`.
- Phase 65 starts with a frozen score contract and baseline; no 4.0 result may
  replace the 3.8 path without per-query comparison and hard-gate evidence.
- Current proof: `cargo fmt --all -- --check`, workspace check, workspace
  Clippy with warnings denied, full workspace/all-target tests (exit 0), all
  Baron core/CLI focused tests, guarded context smoke, the independent
  sixteen-case A/B benchmark (all four surfaces 100/100, zero leakage), the
  nine-case security routing regression (100/100), the bounded static scan
  (100/100, 0 findings), and integrated acceptance (100/100) pass. Reports are in
  `docs/assessment/baron-4.0-benchmark.*` and
  `docs/assessment/baron-4.0-security-regression.*` and
  `docs/assessment/baron-4.0-certification.*`. The expanded clean benchmark
  report id is `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`
  with source fingerprint
  `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`.
- Candidate behavior: released 4.0 memory/Wiki/CodeGraph paths are default;
  `BARON_ENGINE_GENERATION=3.8` or `baseline` forces the active 3.8 recovery
  path, and unknown values fail closed.
  Wiki links/citations, CodeGraph imports/spans/calls/references, and security
  fail-closed/allowlist checks are implemented as local bounded candidates.
- Phase 67 now has a read-only `baron memory consolidate` analyzer that stages
  duplicate/conflict/supersession candidates without writing or promoting any
  Vault record; durable compaction, temporal authority, and rollback are still
  open.
- All public release gates are now closed: exact-source push, native matrix,
  immutable tag/Release, checksums/manifest, and fresh public Windows recovery
  install passed. Broader real-repository/large-scale corpus and durable
  temporal compaction remain explicit follow-up limits and are not silently
  claimed as completed.
- Safe next action: commit/push the final truthful README, status, and build-log
  synchronization, run the documentation trust tests, and leave the tree clean.

## Pre-Publication Evidence (All Gates Passed)

- Source version is `4.0.0`; `cargo fmt --all -- --check`, `cargo check
  --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo build --release --locked -p baron-cli` pass. The optimized binary
  reports `baron 4.0.0`.
- `cargo test --workspace --all-targets --no-fail-fast` passes after the README,
  status, and fingerprint-boundary updates. The final `public_trust_docs` suite
  is 9/9, and the full workspace run exits 0.
- Final clean acceptance report: `docs/assessment/baron-4.0-certification.*`,
  score `100/100`, benchmark id
  `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`, source
  fingerprint
  `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`.
  Memory/Wiki/CodeGraph/Security each score 100/100; leakage is 0; security
  routing is 9/9; static scan is 100/100 with 0 findings; bounded handoff is
  902 characters and project-grounded.
- CodeGraph source fingerprints now exclude generated assessment/status/build
  metadata and release README prose, so evidence references cannot invalidate
  their own code identity; Wiki remains the documentation index/source path.
- Optimized binary smoke passes setup, Codex fullstack init, default 4.0
  context/Resume Brief, Wiki index/search, CodeGraph index/query, and explicit
  `BARON_ENGINE_GENERATION=3.8` recovery context.
- Public release gate complete; only final documentation synchronization remains
  in this build note. No source or release artifact change is required.

## Current Phase

Baron 4.0 Phase 76 exact-source publication and recovery install (`completed`).

## Baron 3.8 Implementation Checkpoint

- Owner approval: explicit approval to implement all planned Baron 3.8 phases,
  update README, publish the newest Windows-downloadable release, and mark each
  completed phase/task in `docs/BARON_STATUS.md`.
- Phase 53-56 implementation evidence now exists in the `knowledge` module:
  bounded Resume Brief, deterministic local hybrid recall, layered memory
  labels, redaction, benchmark output, and Continuity/context integration.
- Phase 57-59 implementation evidence now exists for project-bound incremental
  Wiki citations and a disposable local CodeGraph fallback with symbol/edge
  queries.
- Phase 60-63 implementation evidence now includes project identity checks,
  secret redaction, local cache boundaries, three Baron-owned optional reverse
  skills, narrow reverse routing, and adapter asset lifecycle integration.
- Current proof: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, knowledge tests, control-plane tests, adapter
  lifecycle tests, locked release build, exact `baron 3.8.0`, status JSON parse,
  and new CLI Wiki/CodeGraph/Resume smoke all pass. GitHub Actions run
  `31603251123` also passed the full workspace suite, Clippy, all four native
  targets, release metadata/checksums, installer lifecycle, and immutable
  `v3.8.0` promotion. Fresh `releases/latest` Windows setup/init/context smoke
  returned `baron 3.8.0`.
- Follow-up CI run `31604514726` exposed an OS-dependent assertion in the
  bounded preservation-preview test: macOS directory enumeration can stop
  before one particular unlisted file while still satisfying the contract.
  The test now checks only its actual guarantees (bounded diagnostic and
  dependency-artifact exclusion); the preceding planner test retains the
  explicit user-file preservation assertion. Local `update_planner` coverage
  is 15/15 after this contract-alignment fix, with no release artifact change.
- GitHub Actions CI run `31605317869` (`#48`) then passed format/clippy and
  all four native targets, including the formerly failing Apple Silicon job;
  `main` is green at commit `01c1051991ea772da59bae4a622d98cdeff90dad`.
- Final status-audit commit `97015aef29f60bb2a7a5c170130ece82913944cf`
  leaves every checklist item in `BARON_STATUS.md` checked (658 checked,
  0 unchecked). GitHub Actions CI run `31607104564` (`#50`) passed format,
  Clippy, Windows, Linux, macOS Intel, and Apple Silicon for that exact
  revision.
- Safe next action: normal maintenance from the published `v3.8.0` baseline;
  re-run the native release workflow only for a future version bump.

## Final Baron 3.7 Public Release Checkpoint

- Baron 3.7 is complete and public. Source revision
  `cc14c222130ac2047d36b3b752d9140521d3538e` is pushed to `origin/main` and
  was accepted by GitHub Actions release run `31582187832`.
- Exact-source verification, Ubuntu full workspace tests, Clippy, Windows
  x64, Linux x64, macOS Intel, Apple Silicon, checksum/manifest generation,
  installer lifecycle smoke, annotated tag, and immutable Release promotion
  all passed.
- Public Release: `v3.7.0` at
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v3.7.0`. Assets
  include all four native targets, raw update candidates, `SHA256SUMS`,
  `release-manifest.json`, `install.ps1`, and `install.sh`.
- Fresh Windows public smoke downloaded `install.ps1` from
  `releases/latest`, installed `baron 3.7.0`, passed `setup`,
  `init --codex --fullstack`, and `context`; the same-version update guard
  rejected a redundant update without changing the user-owned marker.
- README, `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, the certification
  report, the executable 3.7 plan, and the design completion record were
  synchronized in final documentation commit `2e637495d3d8c639e5920633ab21e9e4c3b7e9ee`.
- A follow-up README-only commit `1fb041768c7a8937dbe6632f2d66df85e9f947b5`
  makes the Windows installer command work under the host's default execution
  policy by invoking the downloaded script with `-ExecutionPolicy Bypass`.
- Proof status is complete; `origin/main` equals `1fb0417` and the local
  working tree is clean and synchronized.

## Status Checklist Repair

- GitHub review caught that the Phase 46-51 detailed checklists in
  `docs/BARON_STATUS.md` still displayed `[ ]` even though the phase summary,
  status JSON, tests, and release evidence were complete.
- Every actual completion item and exit-gate item under Phases 46-51 is now
  checked `[x]`. Phase 52's remaining unchecked bullets are explicitly labeled
  permanent hard-stop policy rules, not unfinished work.
- The Phase 51 status text now describes the certified 3.7 release source;
  historical 3.6 baseline references remain only where they document the
  pre-bump certification boundary.

## Checkpoint After Phase 46-50 And Phase 48 Hardening

- Phase 46 now has a write-free work-shape decision and hidden CLI surface;
  Phase 47 has source/project-bound receipts from the Baron-owned runner;
  Phase 48 now requires current receipts for medium/high plan proof and all
  three quality gates; Phase 49 has approval plus comparable-rerun experiment
  records; Phase 50 routes a bounded project runbook only to runtime tasks.
- Focused proof passed: core receipt (including large-output/no-deadlock,
  source-change, and tamper invalidation), trusted proof, high-risk plan with
  receipt-backed gates, proof/trace, control plane, operations, harness
  experiments, and CLI command suites.
- A first large-output test exposed that the bounded reader stopped draining a
  pipe; the runner now drains concurrently and retains only bounded excerpts.
  A second issue exposed Baron-managed Markdown changing source fingerprints;
  `docs/baron`, `.baron`, and adapter-managed runtime trees are excluded from
  the product source fingerprint while project source contents are hashed.
- Proof status: focused implementation evidence is passing; integrated Phase
  51 certification is not yet complete. Source/version remains `3.6.0`.
- Trace status: this implementation batch has no public release trace yet;
  release trace must include exact source SHA, native matrix, assets, and
  fresh public installer smoke.
- Safe next action: run the full workspace suite and Clippy, then close any
  certification gaps before changing release metadata.

## Phase 51 Certification Checkpoint Before Version Bump

- Phase 51 local certification is complete on the source version `3.6.0`.
- Passing evidence: `cargo fmt --all -- --check`; all `baron-core` and
  `baron-cli` targets with single-threaded integration execution; Graphify
  local fixture with a bounded 3-second Windows startup budget; Clippy with
  `-D warnings`; locked release build; release binary `baron 3.6.0`; release
  profile certification; adapter/preservation/scale/migration/interruption/
  redaction/public-doc regression gates.
- The combined workspace command was attempted and timed out on this Windows
  host after emitting passing results; the independent package all-target
  suites completed without assertion failures. The timeout is recorded as a
  test-runner scheduling limitation, not a release pass.
- Proof status: Phases 46-51 are certified locally; public 3.7 proof does not
  exist yet. Trace status: release trace is pending exact candidate SHA.
- Safe next action: bump workspace/package/lock/docs metadata to `3.7.0`, keep
  README claims aligned with the public release only after promotion, then run
  the release manifest and native GitHub workflow gates.

## Phase 52 Candidate Checkpoint After Version Bump

- Workspace/package/lockfile, certification target, dynamic version assertions,
  release guide, and source-version README line now agree on `3.7.0`.
- The README deliberately still identifies public `releases/latest` as
  `v3.6.0` until the immutable 3.7 Release exists; it contains the candidate
  version and the exact post-promotion check without making a false download
  claim.
- Candidate proof passed: formatting check, Clippy, focused core/CLI release
  tests, locked optimized build, and release binary `baron 3.7.0`.
- Public proof status: no 3.7 GitHub tag, Release, native matrix, assets, or
  latest-install smoke exists yet. Trace status is pending the exact pushed
  candidate SHA and workflow run.
- Safe next action: inspect and commit this candidate, push the exact source to
  `origin/main`, dispatch the immutable release workflow, and wait for all
  native/public checks before changing the README public-release block.

## Phase 52 Workflow Repair Checkpoint

- Release workflow `31581443056` verified the exact candidate SHA, formatting,
  and the beginning of the full workspace suite, then failed only in the
  Linux self-update fixture because its hard-coded `3.6.1` candidate is older
  than the now-running `3.7.0` binary.
- The fixture now derives its candidate from `CARGO_PKG_VERSION` and increments
  the patch number, preserving the intended staged-candidate assertion without
  weakening the production older-version rejection.
- Local proof passed after the repair: formatting check, the Linux-sensitive
  self-update integration test, and Baron CLI Clippy with `-D warnings`.
- Current proof status: release workflow must be re-dispatched from the repair
  commit; no tag or public 3.7 Release is claimed until that run passes.
- Safe next action: commit this focused test repair, push `origin/main`,
  dispatch with the exact new SHA, and inspect all native/public release jobs.

## Phase 52 Clippy Repair Checkpoint

- Release workflow `31581885211` passed exact-source verification, formatting,
  and the full Ubuntu workspace test, then exposed four Unix-only unused imports
  in Windows-gated receipt tests during workspace Clippy.
- Those imports are now gated with `cfg(windows)`, preserving the Windows test
  coverage while making the all-target lint clean on Ubuntu.
- Local proof passed after the repair: `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`.
- Current proof status: no 3.7 tag or public Release exists; the workflow must
  be re-dispatched from this lint-clean commit.
- Safe next action: commit, push, dispatch the exact SHA, and monitor the full
  native matrix plus immutable publication.

## Continuity Checkpoint Before Source Edits

- User approval: explicit approval to implement and publish the full Baron 3.7
  program, including README, GitHub push, tag, Release, and public installer
  smoke.
- Last successful step: baseline audit confirmed `v3.6.0` is the current public
  release; only the pre-existing Baron 3.7 plan edit was dirty.
- Proof status: no Baron 3.7 implementation proof yet; source remains `3.6.0`.
- Trace status: no active 3.7 execution trace yet.
- Affected files so far: new 3.7 design/plan and this build note, plus the
  approved 3.7 status plan in `docs/BARON_STATUS.md`.
- Safe next action: implement Phase 46 work-shape and Phase 47 trusted receipt
  primitives, keeping all release metadata at `3.6.0` until certification.
- Retry condition: if any batch fails, preserve the failing test/output and
  record the exact source files and next safe repair in this file before
  continuing.

## Final Release Checkpoint

- The user explicitly authorized publishing the immutable `v3.6.0` GitHub
  Release after asking for a reinstall-ready README.
- README now provides a one-block Windows installer, a strict `baron 3.6.0`
  version check, Vault restore guidance, and project refresh guidance.
- `docs/BARON_STATUS.md` has a visible final-release checklist. It is checked
  only after GitHub native builds, immutable tag/assets, and `releases/latest`
  smoke have completed.
- Fresh pre-publish verification passed: `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo build --release --locked -p baron-cli`, YAML lint for
  `.github/workflows/release.yml`, and a release-profile Vault/project
  certification smoke using `baron 3.6.0`.
- The initial immutable promotion run `30243740873` stopped in its Ubuntu
  verification job before any native build, tag, or GitHub Release was made.
  The error was a missing `CandidateBinaryInspector` trait import in an
  Unix-only runtime-activation branch. Windows local verification did not
  compile that branch, which is why the failure first appeared in GitHub.
- A RED/GREEN regression test now covers exact activated-runtime version
  matching. The repair moves that check into one shared helper so both the
  trait contract and the mismatch rejection are compiled on every platform.
- Fresh repair evidence: the exact-version regression, full workspace suite,
  Clippy, and `cargo build --release --locked -p baron-cli` passed. A local
  `x86_64-unknown-linux-gnu` cross-check is blocked only because this Windows
  machine lacks `x86_64-linux-gnu-gcc` for `ring` and bundled SQLite; it is not
  accepted as Linux proof. GitHub Ubuntu must compile the repaired exact source
  before Baron can create a tag or release.
- The repaired release binary completed a fresh `setup --vault`,
  `init --codex --fullstack`, and `certify run --profile release` smoke. It
  produced the expected Vault, Codex adapter, project configuration, and all
  release certification checks passed.
- The second immutable promotion run `30244634765` also stopped before any
  tag, native artifact, or GitHub Release. Its Linux-only self-update fixture
  copied a debug binary larger than the real 128 MB candidate ceiling, so the
  fixture failed before its intended version-mismatch assertion. The fix uses
  a tiny executable Unix wrapper that delegates to the running test binary;
  production candidate limits remain enforced. The fresh full suite, Clippy,
  locked release build, and workflow YAML check passed after this fixture fix.
- An incorrect-SHA dispatch `30245161318` was cancelled before any build or
  promotion after its mismatch was detected. The next dispatch always derives
  `source_revision` directly from `origin/main`, not from copied text.
- The third immutable promotion run `30245233369` also stopped before any tag,
  native artifact, or GitHub Release. The remaining Unix update-transaction
  fixture staged an oversized patched debug binary. It now stages a bounded
  wrapper that delegates to the patched backing binary, preserving the expected
  candidate protocol and the real production size ceiling. The complete local
  test suite, Clippy, locked release build, and workflow YAML gate passed after
  the complete fixture audit.
- The fourth immutable promotion run `30245849879` passed its complete Ubuntu
  test suite, then stopped before tag creation during Clippy. The error set was
  limited to an empty Unix-only placeholder test plus two Windows-only runtime
  owners compiled on Linux. The placeholder test is removed and the runtime
  state path/helper are now explicitly Windows-scoped; the release workflow and
  every production safety ceiling remain unchanged.
- Fresh fourth-repair local evidence passed: `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast --quiet`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo build --release --locked -p baron-cli`, workflow YAML lint, status JSON
  parsing, and a release binary `setup --vault`, `init --codex --fullstack`,
  `certify run --profile release`, and `--version` smoke. The smoke returned
  `baron 3.6.0` and passed every release certification check.
- Repaired source commit `36413199d39c547664d1a2500e8a4444219d858e` is now
  pushed to `origin/main`. The next immutable promotion derives its exact source
  SHA from the current remote branch instead of using copied text.
- Final public proof: GitHub Actions run `30246729740` passed exact-source
  verification, Windows, Linux, macOS Intel, macOS Apple Silicon, and immutable
  promotion. Tag `v3.6.0` resolves to
  `c89486694d9a4431e04106274d0c9f997db42683`; the release contains native
  archives, `SHA256SUMS`, `release-manifest.json`, `install.ps1`, and
  `install.sh`.
- Public Windows smoke: downloaded `install.ps1` from `releases/latest` into a
  new temporary directory, installed with `-Version 3.6.0 -NoPathUpdate`, then
  verified `baron 3.6.0`, `setup --vault`, `init --codex --fullstack`, and
  `context` in a fresh project.
- No release action remains. For a Windows reinstall, follow the short README
  flow: restore the Vault and project folders, install from `releases/latest`,
  run `baron setup --vault`, then run `baron update` in each project.

## Verified Checkpoint Before Phase 35

The bundled Superpowers workflow core has been refreshed to the pinned upstream
`v6.2.0` skill tree without changing Baron's core ownership:

- exact upstream commit:
  `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`
- upstream subtree: 50 files with no missing or extra file; one recorded Baron
  patch removes the visual companion's remote branding/telemetry request
- Baron-owned root routing and adapter integration remain local
- Codex, Claude, and generic adapter contract test passed
- provenance, MIT license, and offline runtime ownership are recorded
- Unix installs preserve executable mode for embedded shebang scripts
- a short Baron-owned entry contract protects the SDD review breaker and
  behavior-test rules from long-context summarization drift
- extension boundaries for code graphs and optional skill sources are accepted
  in `docs/decisions/0002-extension-ownership-and-code-graph.md`

This checkpoint starts Phase 35 only. It does not bump the Baron version, add
Graphify, or change the 3.4 phase order.

## Blueprint Audit

The two apparent core trees are not equal maintained sources:

- `assets/core/` has 145 runtime asset files and is embedded by
  `baron-adapters`.
- `blueprints/core/` has 7 stale historical files.
- Six paths overlap and all six have different content.
- No Rust crate, installer, manifest, test, or workflow reads
  `blueprints/core/`.

Phase 35 completed the source-of-truth contract, deletion of
`blueprints/core/`, adapter parity proof, managed baseline, conservative
three-way planner, malformed-marker refusal, and no-write CLI preview. The
RED test failed on the stale directory; the same focused test passed after
removal. A fresh install now records only repository-relative Baron-owned
content under `.baron/managed-state/`; custom assets and project state remain
outside its write set.

## What Is Being Built

- Phase 35 has removed stale blueprints, proved `assets/core/` is the only
  runtime source, and recorded the managed baseline plus safe read-only
  three-way preview.
- Phase 36 resolved and verified the exact native release candidate before project activation.
- Phase 37 applies project/runtime changes transactionally, stages conflicts, and recovers interrupted updates; its focused transaction/recovery suites now pass.
- Phase 38 separates human update authority from AI local repair and certifies Baron 3.4; the full certification gate has passed and `3.4.0` is the current stable source baseline.
- Phases 39-41 produced certified Baron 3.5 by distilling selected frontend,
  deep-module, and domain-language techniques into existing Baron owners, with
  no second workflow or frontend skill. Reconcile now restores missing managed
  Domain Language documents without overwriting user content.
- Phases 42-45 produced Baron 3.6 with an optional project-scoped Graphify
  code-only provider, bounded context, source verification, strict isolation,
  and Survey fallback. The source baseline is now `3.6.0`.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`.
3. Read
   `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`.
4. Execute
   `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`.
5. Phase 39 completed: local frontend guidance now has a brief fingerprint,
   anti-template gates, responsive/state proof, three-adapter installation
   proof, narrow routing proof, and no operational live dependency.
6. Phase 40 completed: local deep-module guidance and project-scoped domain
   language now preserve user content, avoid generic memory leakage, and
   withhold divergent repo/Vault terms from trusted context.
7. Phase 41 completed: local assets, narrow routing, adapter parity, user
   preservation, public flow, and the Baron 3.5 source gate passed.
8. Phase 42 completed: provider-neutral graph state, source fingerprint,
   project identity, cache path, junction, query-bound, and optional-capability
   contracts passed without provider invocation.
9. Phase 43 completed: the deterministic fake Graphify adapter accepts only
   `0.9.25`, permits only local probe/code-only extraction/bounded query calls,
   disables provider query logging, uses bounded subprocess files and atomic
   cache promotion, and keeps the last good graph on every tested failure.
10. Phase 44 completed: task-matched context never invokes Graphify, caches at
   most eight local hints, and routes AI to hidden refresh/query only when
   needed. Every graph result is checked against current repo source; inferred
   results remain advisory.
11. Phase 45 completed: same-name graph isolation, Vault exclusion, stale and
   corrupt fallback, 6,100+ source fingerprint coverage, and hook/instruction
   preservation passed.
12. Baron 3.6 source/docs/lock/certification are synchronized. Read
   `docs/assessment/baron-3.6.0-code-graph-certification.md` before any later
   code-map change.
13. `v3.6.0` is publicly released after immutable GitHub proof. No active
   implementation or release action remains.

## Verified Baseline

- Isolated branch: `codex/baron-3-4-to-3-6`.
- Planning baseline commit: `b6e619b`.
- Baseline `cargo test --workspace --all-targets`: passed on 2026-07-23.
- Source and stable-source version are now `3.6.0` after Phase 45's full gate.
- Baron 3.4 and 3.5 have source certification; no GitHub Release/tag was made.
- Baron 3.6 Phases 42-45 have complete source/test evidence.
- Baron `v3.6.0` is the latest downloadable release, built and promoted by
  GitHub Actions run `30246729740`.
- The program has 0 planned phases remaining.
- Superpowers `v6.2.0` adapter contract: passed on 2026-07-24.
- Full post-refresh workspace tests, Clippy, release build, adapter smoke, SDD
  semantic smoke, and visual-server behavior tests: passed on 2026-07-24.

## Non-Negotiables

- Superpowers remains workflow core.
- Core agents remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains source of truth.
- Normal users keep the simple install/setup/init/update flow.
- AI never silently downloads or activates a release.
- Conflicting managed edits never overwrite live project content.
- No release tag or GitHub Release exists before its source and artifacts pass proof.
- `assets/core/` is the only runtime source for bundled skills and agents.
- `blueprints/core/` must be removed before any managed baseline is trusted.
- Optional providers and skills cannot become a second workflow, memory, or
  instruction owner.
- Baron 3.5 may deepen existing owners but must not create Hallmark or Matt
  workflow skills.
- Baron 3.6 may use Graphify only as a local optional code map; it cannot own
  hooks, instructions, Vault memory, global context, or workflow state.
