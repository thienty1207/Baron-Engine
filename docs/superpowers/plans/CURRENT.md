# Current Baron Build Plan

Last updated: 2026-06-15

## Current Focus

- Current phase: Phase 8 - Release Hardening
- Status: `implementation_in_progress`
- Verification: Phase 7 full suite plus Phase 8 release contract, Windows installer lifecycle, large-repo, shared-Vault, multi-adapter, and degradation smoke pass locally
- Next action: finish docs, run the complete local release gate, then push for native hosted-runner proof

## Phase 8 Contract

- Workspace version is the release version.
- Four native target archives are required.
- Checksum verification happens before executable replacement.
- Update keeps rollback binaries.
- Uninstall never owns project or Vault data.
- Local proof cannot substitute for hosted macOS and Linux proof.

## Active Documents

- Design: `docs/superpowers/specs/2026-06-15-release-hardening-design.md`
- Plan: `docs/superpowers/plans/2026-06-15-phase-8-release-hardening.md`
- Build log: `notes/build-log/2026-06-15-phase-8-release-hardening.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Do not call Baron 100% complete before Phase 8 release hardening passes.
- Do not treat installed or detected tools as executed checks.
- Keep Vault Markdown as memory source of truth.
- Keep Superpowers and the three core quality agents unchanged.
