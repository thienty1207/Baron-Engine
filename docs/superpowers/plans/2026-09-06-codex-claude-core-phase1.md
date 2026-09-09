# Codex + Claude Core Consolidation — Phase 1 Plan

Last updated: 2026-09-06

## Objective

Freeze the current Baron behavior and migration inputs as deterministic
regression fixtures, while recording the revised Core architecture as explicit
expected-red tests. Phase 1 does not change production behavior.

## Evidence boundary

- Read the deep audit as evidence and the Codex + Claude Core optimization
  specification as the target authority.
- Keep current regression tests and target-red tests in separate files.
- Construct the exact historical unsupported adapter value at runtime so the
  compatibility fixture proves real deserialization without adding a searchable
  retired product literal.
- Treat the generic adapter fixture as migration input only.
- Keep platform-specific path and permission mechanics guarded; use deterministic
  malformed, missing, directory, and barrier-ordered mutation fixtures for the
  portable cases.
- Do not alter Rust production modules, schema versions, managed ownership, or
  command behavior in this phase.

## Fixture/test work

1. Add current adapter fixtures for Codex-only, Claude-only, generic-only,
   Codex + Claude, and both installation orders. Capture user text, hooks,
   settings, commands, and custom skills.
2. Add deterministic managed-state fixtures for the existing manifest schema,
   corrupt/truncated state, missing baselines, modified/unchanged files,
   Core-owned legacy paths, duplicate owners, and interrupted state.
3. Encode legacy skill ownership cases A (unchanged managed), B (custom), C
   (modified former managed), and D (ambiguous) without implementing migration.
4. Add cross-platform safe-I/O and path-boundary coverage, with Unix/Windows
   guards for symlink/reparse behavior and a barrier-ordered mutation case.
5. Freeze memory trust/firewall, interrupted continuity, profile generation,
   flat context truncation, and adversarial structured task transport.
6. Add ignored target tests for canonical Core ownership/projections, safe
   replacement, unified trusted context, protected Tier 0, Database routing,
   structured prepare, thin adapter bridges, generic retirement, and legacy
   ownership migration.

## Verification sequence

1. Run focused current fixture tests.
2. Run ignored target tests and record every intentional failure by test name,
   current behavior, target behavior, and the later implementation phase.
3. Run all relevant adapter, core, and CLI regression test targets.
4. Run formatter and hygiene checks, validate status JSON, and record any
   environment-only installer limitation without relabeling it as target red.
5. Publish the Phase 1 report and stop. Phase 2 begins only in a later task.

## Later phase map

- Phase 2: safe I/O and locking primitives.
- Phase 3: Core managed ownership and legacy ownership migration.
- Phase 4: canonical Core installation and projections.
- Phase 5: control-plane prepare and structured input.
- Phase 6: trusted memory/context unification and protected context tiers.
- Phase 7: profile-aware routing, including a separate Database profile.
- Phases 8–9: Codex and Claude thin adapter bridges.
- Phase 11: unsupported adapter and legacy command/document cleanup.
