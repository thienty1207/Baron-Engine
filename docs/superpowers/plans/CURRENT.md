# Current Baron Build Plan

Last updated: 2026-06-16

## Current Focus

- Completed phase: Phase 13-14 - Certification And Release
- Status: `completed`
- Verification: certification core/CLI tests, full workspace tests, Clippy, local smoke, and release metadata verification pass
- Next action: publish native GitHub release assets only when the operator is ready

## Baron 2.0 Contract

- Workspace version is the release version.
- Four native target archives are required.
- Checksum verification happens before executable replacement.
- Update keeps rollback binaries.
- Uninstall never owns project or Vault data.
- `baron certify run` must pass before release confidence is trusted.
- Vault Markdown remains memory source of truth.
- Cross-project memory stays blocked unless explicitly matched.
- Skill/agent routing must remain control-plane validated.
- Medium/high-risk completion still requires proof and trace quality.

## Active Documents

- Program design: `docs/superpowers/specs/2026-06-15-baron-2-program-design.md`
- Plan: `docs/superpowers/plans/2026-06-16-phase-13-14-certification-release.md`
- Build log: `notes/build-log/2026-06-16-phase-13-14-certification-release.md`
- Final audit: `docs/assessment/2026-06-16-baron-2-final-audit.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Baron is 100% complete against the Phase 9-14 roadmap for `v2.0.0`.
- Do not treat installed or detected tools as executed checks.
- Keep Vault Markdown as memory source of truth.
- Keep Superpowers and the three core quality agents unchanged.
