# Current Baron Build Plan

Last updated: 2026-07-24

## Current Focus

- Active phase: Phase 44 - Automatic Bounded Context And Source Verification
- Status: `in_progress`
- Verification: Baron 3.5 source certification passed: full workspace tests,
  Clippy, locked release build, adapter parity, local-only reconcile recovery,
  custom-asset preservation, bounded context, and public-flow contracts.
  `v3.5.0` is the stable source baseline; no tag or GitHub Release was created.
- Next action: add bounded task-routing, hidden AI automation commands, and
  direct source verification without making normal context wait for a graph
  refresh.

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
- Baron 3.6, Phases 44-45: bounded source verification, strict isolation, and
  certification.
- Total remaining planned phases: 2.

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
