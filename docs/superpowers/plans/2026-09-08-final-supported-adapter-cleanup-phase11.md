# Phase 11: Final Supported-Adapter Cleanup

## Scope

This plan removes unsupported adapter product surfaces and leaves Codex and
Claude as the only active Baron integrations. Historical serialized values are
handled through a generic opaque compatibility boundary and can never become a
runtime operation identity. Core ownership, PreparePacket v1, managed-state v2,
memory schemas, and the Phase 8/9 thin bridges remain intact.

Phase 12 hook/idempotency work, later Autopilot UX, release hardening, and the
final documentation rewrite are explicitly deferred.

## Evidence before cleanup

- Tracked content currently contains 476 lines matching the retired adapter
  token.
- Three tracked filenames contain that token.
- Active unsupported surfaces are present in adapter enums, context and hook
  targets, CLI flags, installer/update branches, tests, packaged projections,
  and current documentation.
- Existing managed-state ownership already has generic unsupported provenance;
  verify it remains readable without restoring old runtime variants.

## Execution order

- [x] Record the Phase 11 start checkpoint and inventory.
- [x] Add or convert focused tests for the final supported surface, tolerant
      legacy config/history parsing, no replacement selection, Core preservation,
      package/layout cleanup, and content/filename zero gates.
- [x] Observe target failures before production edits and classify them.
- [x] Remove unsupported runtime variants and CLI/context/hook/install/update
      branches while preserving Codex and Claude behavior.
- [x] Add generic compatibility parsing at persistence/history boundaries and
      prove it cannot activate an operation.
- [x] Remove or generically rename obsolete tests, assets, docs, plans, and
      filenames. Preserve useful architecture guidance without retired names.
- [x] Run focused tests, relevant regressions, formatter, workspace tests,
      Clippy, diff checks, and both zero gates.

## Schema and safety constraints

- PreparePacket v1: unchanged.
- Managed-state v2: unchanged.
- Memory schemas: unchanged.
- Project schema: unchanged unless tolerant parsing is demonstrably unsafe.
- Unsupported-only projects are preserved and fail closed until an explicit
  Codex or Claude initialization.
- No automatic adapter replacement, release, tag, push, or history rewrite.

## Completion evidence

Record active adapter types and variants, the opaque compatibility model,
removed surfaces/files, final generated layout, Core/Codex/Claude regression
results, exact zero-gate results, known host-only installer limitations, and
the Phase 12 readiness decision in the final report.

## Completion evidence (2026-09-08)

- Active runtime adapters are Codex and Claude only. Historical serialized
  values are retained in opaque compatibility fields and fail closed for
  operation, context, capability, proof, and update selection.
- Core, bridge, ownership, migration, memory/context, routing, and CLI suites
  pass. With the Windows PowerShell system module path, the final workspace
  sweep reports 568 passed, 0 failed, and 3 intentionally ignored historical
  fixtures. The default Codex module path causes two installer tests to fail
  before their assertions because it resolves an incompatible
  `Microsoft.PowerShell.Archive`; the focused installer suite is 5/5 with the
  system path.
- `cargo fmt --all -- --check`, workspace Clippy, `git diff --check`, status
  JSON validation, and tracked content/filename zero gates pass. PreparePacket
  v1, managed-state v2, memory schemas, and project schema remain unchanged.
