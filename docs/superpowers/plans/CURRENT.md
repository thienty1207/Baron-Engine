# Current Baron Build Plan

## Phase 16 v5.0.0 release candidate checkpoint

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

Last updated: 2026-09-09

## Current Refactor Track

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

## Current Refactor Track

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

## Current Refactor Track

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

## Current Refactor Track

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

## Current Refactor Track

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

## Current Refactor Track

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

## Current Refactor Track

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

## Current Refactor Track

- Active phase: Baron Codex + Claude Core Consolidation - Phase 8
- Status: `completed`
- Design: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
- Plan: `docs/superpowers/plans/2026-09-07-codex-native-thin-bridge-phase8.md`
- Next action: prepare the Phase 9 plan; do not begin Phase 9 in this
  checkpoint.

Last updated: 2026-09-07

## Current Focus

- Active phase: normal Baron 4.2.2 maintenance after public patch release
- Status: `completed`
- Design: `docs/superpowers/specs/2026-08-20-multi-agent-core-parity-design.md`
- Plan: `docs/superpowers/plans/2026-08-20-multi-agent-core-parity.md`
- Next action: normal `4.2.2` maintenance; preserve the shared core contract
  and rerun the cross-adapter parity/preservation suite for future changes.

## Current Release Track

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

## Active Documents

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
