# Current Baron Build Plan

Last updated: 2026-07-23

## Current Focus

- Active phase: Phase 34 - Immutable Release Promotion And Baron 3.3 Certification
- Status: `completed`
- Verification: full no-skip workspace tests, Clippy, release build, installer lifecycle, real project/Vault smoke, JSON, YAML, and static checks passed
- Next action: publish a binary GitHub Release only when explicitly requested

## Baron 3.3 Contract

- Read-only requests do not authorize plan, Harness, proof, trace, review, or learning mutations.
- Ambiguous request authority remains read-only until change intent is explicit.
- Mutating commands require coherent project, Vault, capsule, and identity state.
- SQLite query paths are read-only and cannot fabricate an empty index.
- Completion text is not trusted without verification, proof, and passing trace evidence.
- Release source and every native artifact are proven before a tag or GitHub Release exists.
- Vault Markdown remains memory source of truth.
- Superpowers and the three core quality agents remain unchanged.

## Active Documents

- Design: `docs/superpowers/specs/2026-07-14-baron-3-3-trust-state-safety-design.md`
- Plan: `docs/superpowers/plans/2026-07-14-phase-32-34-baron-3-3-trust-state-safety.md`
- Build log: `notes/build-log/2026-07-14-phase-32-34-baron-3-3-trust-state-safety.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Follow RED/GREEN TDD for production behavior.
- Update the build log after every phase checkpoint.
- Do not mark a phase complete from test intent; record fresh command evidence.
- Keep the normal user command surface small.
