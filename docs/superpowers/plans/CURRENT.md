# Current Baron Build Plan

Last updated: 2026-08-20

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
