# Current Build Note

Date: 2026-07-23
Target: Baron 3.3.0

## Current Phase

Phase 34 - Immutable Release Promotion And Baron 3.3 Certification (`completed`, source pushed).

## What Is Being Built

- Phase 32 separates read-only requests from authorized repository changes.
- Phase 33 blocks durable state writes when project/Vault identity is incoherent, keeps SQLite query paths read-only, and detects completion claims without evidence.
- Phase 34 promotes releases only after exact-source, native-artifact, checksum, installer, and upgrade proof.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/specs/2026-07-14-baron-3-3-trust-state-safety-design.md`.
3. Treat the Baron 3.3 program as complete; do not reopen it without a new approved task.
4. Publish a binary GitHub Release only when explicitly requested.

## Verified Baseline

- Isolated branch: `codex/baron-3-3-trust-safety`.
- Baseline commit: `b10bf3efc0064d34e44f7631eb0d932c870097cd`.
- Baseline `cargo test --workspace --all-targets`: passed.
- Phase 32 core, CLI, adapter, normal-help, formatting, and full adapter lifecycle tests pass.
- Phase 33 state coherence, no-repair, read-only query, completion integrity, CLI, adapter, and formatting tests pass.
- Phase 34 full no-skip tests, Clippy, release build, installer lifecycle, real project/Vault smoke, JSON, YAML, and static checks pass.
- Full verification passed again on merged `main`; source commit `34a6cf4` is pushed to `origin/main`.

## Non-Negotiables

- Superpowers remains workflow core.
- Core agents remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains source of truth.
- Normal users keep the simple install/setup/init/update flow.
- No release tag or GitHub Release exists before its source and artifacts pass proof.
