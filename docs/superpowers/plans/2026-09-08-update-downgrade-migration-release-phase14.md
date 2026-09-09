# Baron Codex + Claude Core Consolidation — Phase 14

**Status:** `completed`

**Goal:** Harden update, downgrade, migration, crash recovery, and local
release-package behavior while preserving the accepted Phase 1–13 contracts.

**Authority:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
The Deep Audit is evidence about the current implementation; this plan records
where the implementation requires a safer path rather than blindly copying a
target statement.

## Scope boundary

- Phase 14 only: persisted-state compatibility, writer floors where evidence
  requires them, transactional update/recovery, rollback, dry-run non-mutation,
  Core/projection coherence, active-task and hook/dedup/Autopilot preservation,
  and local release artifact proof.
- Phase 15 documentation rewrite, Phase 16 final verification/release work,
  version bumps, tags, GitHub publication, and history rewrites remain out of
  scope.
- Existing project, Vault, managed, task, continuity, journal, dedup, and
  Autopilot data remains user-owned and is never reset as a test shortcut.

## Actual dependency map

1. `baron-core::config` owns `.baron/project.toml` (schema 4), supported
   Codex/Claude enums, and opaque historical adapter values. `initialize`,
   adapter switching, and platform writes load this file and then serialize
   the current schema. A future schema is checked by the state-guard path but
   is not rejected by every direct writer yet; Phase 14 tests must close that
   writer gap without changing old-schema readability.
2. `baron-adapters::update` owns managed-state manifest v1/v2, baseline copies,
   Core ownership, and three-way merge planning. v1 is normalized in memory,
   v2 requires `minimum_writer_schema = 2`, missing baselines and duplicates
   fail closed, and migration publishes only after verified copies are staged.
3. `baron-cli::update_transaction` owns candidate identity, transaction state
   schema 1, packet staging, project activation, managed-baseline backup,
   runtime handoff, receipt, startup recovery, and rollback. It already uses
   the Phase 2 project lock and has five project-activation failure points.
   Completion currently writes a small receipt before the terminal state; the
   receipt lacks source/files/rollback/next-action evidence and has no explicit
   receipt-failure checkpoint.
4. `baron-core::automation` owns append-safe lifecycle journal entries and the
   schema-1 hook dedup cache. `baron-core::autopilot` owns the additive schema-1
   project/Vault ledger. Both reject future schema values before writing;
   Phase 14 must prove bytes remain unchanged on malformed/future input and
   that update does not disturb them.
5. `baron-adapters::install` renders one embedded `.baron/core/**` payload set
   followed by deduplicated Codex/Claude integration payloads. The transaction
   planner therefore receives one Core owner per live path and adapter
   projections in one packet set. Phase 14 will prove this remains true during
   update and that modified Core or projection conflicts preserve user bytes.
6. `baron-core::release` validates release manifests and checksums; the local
   release workflow packages the built binary and embedded assets. Phase 14
   will build locally, inventory embedded Core/Codex/Claude assets, and smoke
   the release binary from a source-independent temporary project.

## Test-first batches

### Batch A — persisted compatibility and downgrade refusal

- Add deterministic fixtures for old/current/future project config, managed
  state, transaction state, Autopilot, dedup, journal, continuity, and
  PreparePacket input.
- Assert old supported data remains readable, current serialization is
  deterministic, future/minimum-writer data is inspectable but never rewritten,
  and malformed/truncated input leaves bytes unchanged.
- Add the missing project writer guard test first; retain the actual historical
  adapter value only as runtime-constructed data so retired product text stays
  absent from tracked source.

### Batch B — update transaction and migration recovery

- Extend crash injection coverage through staging, publication, baseline/state
  commit, receipt, Core, and projection boundaries.
- Add receipt evidence assertions, deterministic retry/no-op checks, active
  task/continuity/recovery/journal/dedup/Autopilot byte preservation, and
  rollback evidence/refusal cases.
- Exercise v1→v2 migration permutations, duplicate ownership, modified files,
  missing baselines, and future managed state without inventing replacement
  ownership.

### Batch C — dry-run, release package, and binary smoke

- Prove `baron update --dry-run` does not mutate project, managed, Vault,
  transaction, journal, dedup, or Autopilot bytes.
- Build the local release profile, inspect the packaged inventory for canonical
  Core and both thin bridges, and smoke the release binary in fresh Codex,
  Claude, and Database-profile projects with no source-tree asset dependency.
- Inspect the PowerShell archive failure on this host and record whether it is
  Codex's bundled runtime module or a real system PowerShell defect.

## Implementation rules

- Keep schema versions unchanged unless a focused red test demonstrates that
  additive state cannot be protected by existing exact-version validation.
- Add only behavior-neutral test seams required for deterministic crash
  injection; each seam gets a no-injection regression assertion.
- Preserve one owner per live managed path and never overwrite modified or
  ambiguous user content.
- Run focused tests after each batch, then the required formatter, workspace
  tests, Clippy, release build, diff checks, retired-adapter gates, and local
  binary smoke. Hosted CI is not claimed from local execution.

## Expected-red evidence before implementation

- A future `.baron/project.toml` can currently be parsed by direct config
  loading and may be rewritten by direct initialization/platform/adapter
  writers; the target is fail-closed mutation with unchanged bytes.
- Completion receipts currently contain only identity, target, hash, runtime
  proof, and status; the target requires operation/source/version, staged and
  published files, baseline/state commit, rollback availability, and a safe
  next action.
- A receipt write failure has no explicit transaction checkpoint; the target is
  recoverable evidence that distinguishes a completed state from a missing
  receipt and never reports a false success.

## Exit evidence

Phase 14 ends with the exact report headings required by the accepted brief,
including PASS — CURRENT REGRESSION, EXPECTED RED — TARGET ARCHITECTURE,
UNEXPECTED FAILURE, persisted schema changes, release artifact/binary proof,
retired-adapter gates, and Phase 15 readiness. No Phase 15 work starts here.

## Completed implementation

- Added deterministic compatibility fixtures for old/additive/future project,
  managed, Autopilot, dedup, and journal state. Future project and managed
  writers now refuse mutation while preserving the original bytes and
  actionable schema/writer-floor causes.
- Hardened the update completion boundary with a committed runtime checkpoint,
  additive source/version/file/rollback/next-action receipt evidence, and an
  explicit recoverable receipt-pending state. The test-only failure seam is
  behavior-neutral and the no-injection path remains covered.
- Preserved Core ownership and thin Codex/Claude projections through update,
  Core conflict handling, v1 managed ownership migration, active task and
  continuity state, native hook/dedup journals, and Autopilot state.
- Added local release inventory and release-binary smoke fixtures for Codex,
  Claude, and the Database profile. The release-profile binary was built and
  exercised from a temporary source-independent project.

## Phase 14 verification

- Focused Phase 14 compatibility, adapter update, CLI update, and release
  inventory suites pass. The explicit release-binary smoke passes after the
  local release build.
- The complete workspace matrix passes with zero failures under the compatible
  system Windows PowerShell module path. The three historical Phase 1 fixtures
  remain intentionally ignored; the release-binary test is run explicitly
  after a release build.
- `cargo fmt --all -- --check`, workspace Clippy, release build, `git diff
  --check`, and `git diff --cached --check` pass. The default Codex-bundled
  PowerShell runtime still blocks two installer tests before their assertions
  because `Microsoft.PowerShell.Archive` cannot autoload; the system module
  path passes those lifecycle tests.
- No persisted schema version changed. Hosted CI, tags, publication, and
  version bumps were not run.
