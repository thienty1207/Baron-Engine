# Current Baron Build Plan

Last updated: 2026-06-15

## Current Focus

- Completed phase: Phase 7 - Baron Capability Registry
- Current phase: Phase 8 - Release Hardening
- Status: `not_started`
- Verification: Phase 7 full suite, Clippy, format, positive lifecycle smoke, and negative capability-gate smoke pass
- Next action: design the cross-platform release and installer contract

## Phase 7 Contract

- Registry source: `.baron/capabilities.toml`
- Machine cache: `.baron/cache/capability-state.json`
- Presence is not execution evidence.
- Missing optional providers degrade with warnings.
- Missing required providers or evidence block Proof and Trace.
- Codex, Claude, and generic adapters run capability checks automatically.

## Active Documents

- Design: `docs/superpowers/specs/2026-06-15-baron-capability-registry-design.md`
- Plan: `docs/superpowers/plans/2026-06-15-phase-7-capability-registry.md`
- Build log: `notes/build-log/2026-06-15-phase-7-capability-registry.md`
- Status: `docs/BARON_STATUS.md`

## Rules

- Do not call Baron 100% complete before Phase 8 release hardening passes.
- Do not treat installed or detected tools as executed checks.
- Keep Vault Markdown as memory source of truth.
- Keep Superpowers and the three core quality agents unchanged.
