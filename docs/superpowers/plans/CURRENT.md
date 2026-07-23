# Current Baron Build Plan

Last updated: 2026-07-23

## Current Focus

- Active phase: Phase 35 - Managed Baseline And Update Planner
- Status: `planned`
- Verification: Baron 3.3 baseline `cargo test --workspace --all-targets` passed; no Baron 3.4 implementation evidence exists yet
- Next action: write Phase 35 RED tests before changing managed writers

## Baron 3.4 Contract

- Public `baron update` is the human-authorized complete update flow.
- AI agents use local-only `baron automation reconcile` and cannot silently install releases.
- Baron records the last installed managed baseline before making update decisions.
- Ambiguous local/upstream edits become staged conflicts, not live overwrites.
- Custom skills, custom agents, project source, and Vault memory remain user-owned.
- A verified candidate renders new managed assets.
- Project and runtime activation are one recoverable transaction.
- Immutable exact-source release promotion remains unchanged.
- Vault Markdown remains memory source of truth.
- Superpowers and the three core quality agents remain unchanged.

## Active Documents

- Design: `docs/superpowers/specs/2026-07-23-baron-3-4-safe-self-update-design.md`
- Plan: `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
- Build log: `notes/build-log/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Follow RED/GREEN TDD for every production behavior.
- Complete and record Phase 35 before beginning release networking.
- Update the build log and status Markdown/JSON after every phase checkpoint.
- Do not mark a phase complete from test intent; record fresh command evidence.
- Keep the normal user command surface small.
- Do not bump the source version before Phases 35-37 are green.
