# Phase 4 Plan: Canonical Baron Core Installation

> Scope is limited to Phase 4 of the Codex + Claude Core consolidation. Phase
> 5+ work (prepare, memory/context, routing, platform profiles, final bridge
> polish, global authority, and retired-adapter cleanup) stays deferred.

## Goal

Make `assets/core/**` the single packaged semantic runtime source and install
it transactionally at `.baron/core/**`. Codex and Claude installs will publish
only their integration surfaces and route to the shared Core. Existing user
files and ambiguous or modified legacy copies remain preserved. Managed-state
schema 2 remains the only publication schema.

## Dependency graph confirmed before edits

- `assets/core` is embedded by `include_dir!` in `baron-adapters`.
- `install_adapter` currently owns both Core materialization and adapter
  integration writes; `managed_payloads_for_adapter` currently duplicates the
  embedded tree for Codex and Claude.
- Phase 3 managed ownership and migration already provide Core/adapter owners,
  path uniqueness, baseline copies, and transactional legacy skill transfer.
- CLI init and adapter switching call `install_adapter`; update and
  reconciliation independently build payloads from `managed_payloads_for_adapter`.

## Implementation steps

### 1. Freeze Phase 4 contracts with tests

- Promote the three physical Core/projection target tests that now belong to
  this phase.
- Add focused tests for Core payload separation, nested resource preservation,
  exactly three mandatory agents, Superpowers workflow ownership, installation
  order invariance, Core conflict/no-op/update behavior, packaged embedding, and
  update payload deduplication.
- Reclassify existing Codex/Claude lifecycle assertions that required duplicate
  local semantic trees as intentionally migrated behavior; retain Generic and
  retired-adapter fixtures as compatibility inputs for later cleanup.

### 2. Separate payloads and install the canonical Core

- Add a public `core_managed_payloads()` renderer that recursively enumerates
  every embedded Core file (including nested references, scripts, and support
  assets) under `.baron/core/**` with Core ownership.
- Make Codex and Claude payload renderers contain only adapter integration
  files. Their indexes route to `.baron/core`; local custom directories remain
  untouched.
- Add `ensure_core_runtime(repo)` with the existing project lock and Phase 3
  migration preflight. It validates canonical targets, preserves unknown or
  modified files, publishes missing/upstream Core files through Safe I/O, and
  records a merged baseline transactionally without dropping adapter records.
- Keep legacy Generic/retired-adapter branches available as migration inputs;
  they are not cleaned up in Phase 4.

### 3. Wire init, switching, and updates

- Run Core installation before adapter integration for `baron init` and adapter
  switching through the shared installer path.
- Build update/reconcile payloads as one Core payload set plus each registered
  adapter integration set, deduplicated by effective live path.
- Keep both adapters pointed at the same `.baron/core` tree and make repeated
  installs no-op when bytes and baselines are unchanged.

### 4. Verify and record evidence

- Run focused Core, lifecycle, planner, transaction, and CLI bridge tests.
- Run formatter, full workspace tests, warnings-denied Clippy, and diff checks.
- Record known Windows installer-module failures separately from product
  regressions. Update status, build log, and this plan with exact results.
- Stop after the Phase 4 report; do not begin Phase 5.

## Explicit non-goals

- No new prepare protocol, memory/context trust routing, platform Database
  profile, final bridge UX, global authority, Generic retirement, retired
  adapter cleanup, or `git grep` cleanup is implemented here.
- No managed-state schema 3 is introduced.
- No release or tag is created.

## Execution evidence (2026-09-06)

- `core_managed_payloads()` recursively packages `assets/core/**` into one
  `.baron/core/**` owner, including nested references, scripts, and support
  resources. Codex and Claude payloads now contain integration files and
  routing indexes only; local custom directories remain user-owned.
- `ensure_core_runtime()` runs under the project mutation lock, migrates and
  validates existing managed state, preserves unknown or modified Core files,
  and publishes Core baselines without dropping adapter records. Core-only
  reconciliation handles missing files, upstream changes, and rollback.
- `baron init`, adapter switching, and update planning all use one Core payload
  set plus each registered adapter integration set. Candidate transaction
  validation accepts the shared implicit `core` owner while still rejecting
  duplicate effective live paths.
- Focused tests passed: Phase 4 Core `9/9`, promoted Phase 1 Core targets
  `3/3`, Phase 3 ownership `12/12`, adapter lifecycle `30/30`, update planner
  `15/15`, adapter CLI `11/11`, and the targeted automation, migration,
  release-smoke, and update-recovery suites.
- Required verification passed: `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.
  `cargo test --workspace --all-targets --no-fail-fast` passed all product
  targets except the two existing lifecycle installer tests that require the
  unavailable Windows `Microsoft.PowerShell.Archive`/`Compress-Archive`
  module. No source regression was observed.
- Phase 4 is complete. Phase 5+ behavior remains deferred and no release or
  tag was created.
