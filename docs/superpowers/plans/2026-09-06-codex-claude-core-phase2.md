# Codex + Claude Core Consolidation — Phase 2 Plan

Last updated: 2026-09-06

## Objective

Harden the filesystem paths needed by managed adapter installation, managed
state, update transactions, and migration-state publication. Phase 2 keeps the
persisted schemas and all Core ownership, routing, bridge, memory, and adapter
product behavior unchanged.

## Current dependency graph

- `baron-adapters::install_adapter` calls the adapter installers, which call
  `managed::{upsert_managed_block, upsert_routing_block, write_managed_file}`
  and embedded-tree writers, then call managed-baseline publication.
- `baron-adapters::update` owns managed-state reads, three-way planning,
  reconciliation, baseline copies, and the current deterministic temporary
  writer.
- `baron-cli::update_transaction` owns staged update publication and has a
  second temporary writer; it calls the adapter baseline APIs during project
  activation and rollback.
- `baron-core::migration` owns migration-state JSON and migration publication
  and has a third temporary writer.
- These paths can share a `baron-core::safe_io` module. The project lock lives
  at `.baron/.baron-mutation.lock`; its OS-level advisory lock is
  process-crash releasing, while a process-local registry makes nested calls
  reentrant. The lock is kept beside `.baron` rather than inside
  `.baron/managed-state` so managed-state snapshots never try to copy an open
  lock handle.

## Implementation steps

1. Add focused Safe I/O tests for tri-state reads, malformed/invalid data,
   directory type errors, path-boundary checks, unique temporary names,
   interrupted replacement, metadata preservation, no-op writes, and safe
   cleanup. Add deterministic lock tests for reentrancy, bounded contention,
   stale marker recovery, and two-writer serialization.
2. Implement `baron-core::safe_io` with explicit missing/readable/error text and
   byte reads, same-directory staged replacement, flush/sync, platform-aware
   activation, safe parent validation, and a bounded project-scoped mutation
   lock. Keep schema versions unchanged.
3. Replace adapter managed/install/update reads and writes with the shared
   primitive. Preflight Codex/Claude native JSON before any install mutation;
   preserve malformed bytes and return an actionable error. Lock complete
   install, baseline, and reconcile mutations, retaining existing rollback
   machinery.
4. Replace update-transaction and migration-state temporary writers with the
   shared replacement primitive and guard their mutating entry points with the
   project lock. Do not redesign their transaction/state schemas.
5. Run focused Phase 2 tests, the existing adapter/core/CLI regressions,
   formatter, workspace tests, Clippy, and diff hygiene. Keep the two known
   PowerShell Archive installer failures separated as environment evidence.

## Scope guard

Phase 2 does not implement `ManagedOwner::Core`, Generic migration, canonical
Core installation, thin adapter bridges, prepare protocol, memory/context
changes, profile routing, Generic/retired-adapter removal, or documentation
rewrites beyond status/build-log/plan updates.

## Verification contract

- Existing schemas remain at their current versions; no downgrade work is
  required.
- A failed read or replacement leaves the original bytes available and never
  treats an arbitrary error as an empty file.
- A live lock is never deleted by a contender; stale marker text is recoverable
  only after the OS lock is demonstrably available.
- Any target-red test left for ownership/Core work remains ignored and is not
  weakened.

## Execution result

- Status: complete on 2026-09-06; no Phase 3 work was started.
- Shared Safe I/O is now used by adapter managed/install/update paths, CLI
  update transactions, and migration publication. The project lock is
  `.baron/.baron-mutation.lock`; its diagnostic marker is excluded only from
  operational snapshot fixtures.
- Focused proof passed: `baron-core` `phase2_safe_io` `4/4`, adapter
  `phase2_current` `8/8`, update planner `15/15`, migration `8/8`, update
  transaction unit tests `8/8`, update recovery CLI `3/3`, and self-update CLI
  `1/1`. The five Phase 1 Safe I/O target tests are green when run as ignored
  tests.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
  -D warnings`, and `git diff --check` passed. The workspace test sweep has no
  Baron source regression; two lifecycle tests remain environment-only because
  this Windows host cannot load `Microsoft.PowerShell.Archive`.
- Persisted schemas remain unchanged. The remaining ignored reds are ownership,
  Core consolidation, prepare, trusted context, routing, and retired-adapter
  requirements for later phases.
