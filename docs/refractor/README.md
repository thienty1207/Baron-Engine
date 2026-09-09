# Refactor evidence and status

This directory contains engineering provenance for the Codex + Claude Core
consolidation. It is intentionally separate from the normal user entry point
in the repository root.

## Current contract

The maintained product documentation is:

- [README](../../README.md) for installation, initialization, profiles, update,
  recovery, and the normal ask-first flow;
- [architecture](../architecture/ARCHITECTURE.md) for Core ownership and
  lifecycle contracts;
- [command surface](../architecture/COMMAND_SURFACE.md) for advanced and
  diagnostic commands;
- [Codex compatibility](../compatibility/CODEX.md) and
  [Claude compatibility](../compatibility/CLAUDE.md) for native projections.

These pages describe exactly two active integrations: Codex and Claude. Core
owns the semantic runtime under `.baron/core/**`; legacy values are migration
inputs and are not active products.

## Historical evidence

`BARON_ENGINE_DEEP_AUDIT_2026-09-06.md` is a dated audit of the pre-refactor
source. The phase plans and older design/spec documents record decisions,
fixtures, and verification from earlier phases. They are retained for audit,
not as current user instructions.

`BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md` is the authoritative engineering
target for this refactor. Its status line records implementation through Phase
15 and the remaining Phase 16 adversarial verification boundary.

When a historical plan conflicts with maintained docs or source, use the
current source and the status dashboard, then record the discovery in the
active plan. Do not revive an old adapter architecture from a historical file.
