# Current Baron Build Plan

Last updated: 2026-06-15

## Current Focus

- Completed phase: Phase 8 - Release Hardening
- Status: `completed`
- Verification: local release gates, four-platform native CI, tagged release workflow, and the published installer lifecycle pass
- Next action: maintain `v1.0.0` and start a new plan only for verified follow-up work

## Phase 8 Contract

- Workspace version is the release version.
- Four native target archives are required.
- Checksum verification happens before executable replacement.
- Update keeps rollback binaries.
- Uninstall never owns project or Vault data.
- Hosted Windows, Linux, Intel macOS, and Apple Silicon macOS proof is recorded.

## Active Documents

- Design: `docs/superpowers/specs/2026-06-15-release-hardening-design.md`
- Plan: `docs/superpowers/plans/2026-06-15-phase-8-release-hardening.md`
- Build log: `notes/build-log/2026-06-15-phase-8-release-hardening.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Baron is 100% complete against the Phase 0-8 roadmap for `v1.0.0`.
- Do not treat installed or detected tools as executed checks.
- Keep Vault Markdown as memory source of truth.
- Keep Superpowers and the three core quality agents unchanged.
