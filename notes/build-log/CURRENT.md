# Current Build Note

Date: 2026-07-23
Target: Baron 3.4.0

## Current Phase

Phase 35 - Managed Baseline And Update Planner (`planned`, not started).

## What Is Being Built

- Phase 35 records the managed baseline and plans safe three-way updates without writing live targets.
- Phase 36 resolves and verifies the exact native release candidate before project activation.
- Phase 37 applies project/runtime changes transactionally, stages conflicts, and recovers interrupted updates.
- Phase 38 separates human update authority from AI local repair and certifies Baron 3.4.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/specs/2026-07-23-baron-3-4-safe-self-update-design.md`.
3. Execute `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`.
4. Begin with Phase 35 RED tests. Do not implement release download or binary replacement first.
5. Update the phase log after every verified checkpoint.

## Verified Baseline

- Isolated branch: `codex/baron-3-4-safe-update-plan`.
- Baseline commit: `d4ab320`.
- Baseline `cargo test --workspace --all-targets`: passed on 2026-07-23.
- Current source version remains `3.3.0`.
- Baron 3.4 currently has planning evidence only.

## Non-Negotiables

- Superpowers remains workflow core.
- Core agents remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains source of truth.
- Normal users keep the simple install/setup/init/update flow.
- AI never silently downloads or activates a release.
- Conflicting managed edits never overwrite live project content.
- No release tag or GitHub Release exists before its source and artifacts pass proof.
