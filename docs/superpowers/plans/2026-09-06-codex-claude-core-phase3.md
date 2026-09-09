# Codex + Claude Core Managed Ownership Phase 3 Implementation Plan

> **For agentic workers:** Execute this plan inline with the Phase 3 scope guard. Do not begin Phase 4 canonical Core installation, thin bridges, prepare, memory/context, routing, adapter retirement, or retired-adapter cleanup.

**Goal:** Replace adapter/path-only managed-state ownership with an explicit Core/adapter owner model and a transactional, restartable legacy ownership migration while preserving user content.

**Architecture:** Managed state schema v2 stores an explicit `owner` and provenance metadata. A tolerant v1 reader maps historical adapter strings into either supported adapter owners or a generic legacy owner; canonical `.baron/core/**` records are planned for transfer to `Core`. Live target uniqueness is validated before any mutation. Legacy adapter-local skills are classified from baseline/hash evidence and only verified unchanged Baron-owned files receive a migration plan; modified, custom, and ambiguous files block or remain untouched.

**Tech Stack:** Rust workspace, `serde`/`serde_json`, `anyhow`, Baron Phase 2 Safe I/O and project mutation lock, deterministic `tempfile` fixtures, existing adapter/update transaction tests.

**Spec:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md` and the user-provided Phase 3 requirements.

## Global Constraints

- Phase 3 only: no Phase 4 Core installation or thin bridge redesign.
- Generic and retired adapters remain readable only as legacy input; no new ownership type may make them active supported adapters.
- One live managed target path has exactly one owner; reject duplicates before publication.
- Preserve project identity, Vault routing, live bytes, baselines, user files, custom skills, commands, hooks, and settings.
- Modified former Baron-managed skills are blocking migration conflicts; never silently route around them.
- Use the Phase 2 Safe I/O and bounded project lock for every migration mutation.
- Do not bump project, journal, transaction, or unrelated schemas.
- If managed-state changes, use an explicit version and test v1 upgrade, restart, rollback, and older-writer refusal.
- Keep Phase 4–11 target tests ignored and intentionally red.

---

### Task 1: Promote Phase 2 safety regressions and add Phase 3 ownership fixtures

**Files:**
- Modify: `crates/baron-adapters/tests/phase1_current.rs`
- Modify: `crates/baron-adapters/tests/phase1_target_red.rs`
- Create: `crates/baron-adapters/tests/phase3_ownership.rs`
- Create: `crates/baron-adapters/tests/fixtures/phase3/managed-state-v1/manifest.json`
- Create: `crates/baron-adapters/tests/fixtures/phase3/managed-state-v1/base/agent/.baron/core/skills/legacy/SKILL.md`
- Create: `crates/baron-adapters/tests/fixtures/phase3/managed-state-v1/base/codex/.codex/skills/legacy-owned/SKILL.md`

**Interfaces:**
- Consumes existing `ManagedAssetPayload`, `ManagedBaseline`, `install_adapter`, `load_managed_baseline`, and Phase 2 Safe I/O.
- Produces executable tests for `ManagedOwner`, v1-to-v2 migration, duplicate live-path rejection, Core transfer, A/B/C/D classification, rollback/retry, and installation-order invariants.

- [x] **Step 1: Remove `#[ignore]` from the five Phase 2 safety assertions and run them.**

  Promote invalid UTF-8 preservation, malformed Codex hooks, malformed Claude settings, directory-target detection, and temp collision preservation. Run:

  ```text
  cargo test -p baron-adapters --test phase1_current -- --nocapture
  ```

  Expected: the five promoted tests pass; no production change is made in this step.

- [x] **Step 2: Write Phase 3 ownership tests first.**

  Cover:

  ```rust
  #[test] fn v1_generic_core_records_plan_core_transfer_without_live_rewrite() { /* ... */ }
  #[test] fn duplicate_live_paths_are_rejected_before_manifest_publication() { /* ... */ }
  #[test] fn unchanged_legacy_skill_is_classified_for_core_transfer() { /* ... */ }
  #[test] fn modified_legacy_skill_is_a_blocking_conflict_and_bytes_survive() { /* ... */ }
  #[test] fn custom_and_ambiguous_skills_are_preserved_without_claiming_ownership() { /* ... */ }
  #[test] fn ownership_migration_retry_is_idempotent_after_interruption() { /* ... */ }
  #[test] fn installation_order_keeps_core_owner_stable() { /* ... */ }
  ```

  Use checked-in v1 JSON and baseline bytes for old state. Do not generate old fixtures with the new serializer.

- [x] **Step 3: Run the new tests and the six Phase 1 ownership target tests. They were initially RED and are now promoted GREEN after the ownership implementation.**

  Run:

  ```text
  cargo test -p baron-adapters --test phase3_ownership -- --nocapture
  cargo test -p baron-adapters --test phase1_target_red -- --ignored --nocapture
  ```

  Result: the safety tests and ownership tests are green after the implementation; the three Phase 4 physical Core-installation assertions remain ignored.

### Task 2: Add explicit managed owners and a tolerant managed-state v2 representation

**Files:**
- Modify: `crates/baron-adapters/src/update.rs`
- Modify: `crates/baron-adapters/src/lib.rs`
- Modify: `crates/baron-adapters/tests/update_planner.rs`
- Modify: `crates/baron-adapters/tests/phase1_current.rs`

**Interfaces:**
- Produces `SupportedManagedAdapter`, `ManagedOwner`, `ManagedProvenance`, schema-v2 `ManagedAssetRecord`, and v1 compatibility parsing.
- Keeps runtime payload adapter strings for existing update callers while deriving ownership from the canonical path and supported adapter name.

- [x] **Step 1: Define the failing serialized contract test.**

  Assert v2 output contains `schema_version = 2`, `owner = "core"` for `.baron/core/**`, adapter owners for Codex/Claude, merge policy, baseline hash, installed version, and provenance. Assert a v1 fixture loads without a retired-specific enum and maps historical names to a generic legacy value.

- [x] **Step 2: Run the contract test and confirm it fails on schema version/owner absence.**

- [x] **Step 3: Implement the minimal owner model and parser.**

  Use a generic `ManagedOwner::UnsupportedLegacy(String)` representation for unknown historical values. New payloads accept only `core`, `codex`, or `claude` ownership. Preserve a runtime compatibility accessor for existing `record.adapter` callers without serializing a duplicate v2 adapter field.

- [x] **Step 4: Add exact v1 upgrade and older-writer refusal behavior.**

  A v1 manifest remains readable in memory, but any mutating path must publish v2 only after validation. A manifest with a newer schema or incompatible minimum writer is rejected without rewriting it. Keep project config, transaction, journal, and unrelated schemas unchanged.

- [x] **Step 5: Run focused serialization, planner, and legacy fixture tests.**

### Task 3: Enforce effective live-path uniqueness and provenance validation

**Files:**
- Modify: `crates/baron-adapters/src/update.rs`
- Modify: `crates/baron-cli/src/update_transaction.rs`
- Modify: `crates/baron-adapters/tests/phase3_ownership.rs`
- Modify: `crates/baron-cli/src/update_transaction.rs` unit tests where payload records are constructed.

**Interfaces:**
- Produces one validation routine used by baseline load, baseline publication, update planning, and transaction candidate validation.
- Uniqueness key is the normalized effective live relative path, independent of owner or adapter.

- [x] **Step 1: Add RED duplicate tests for Core/Codex and Codex/Claude collisions.**
- [x] **Step 2: Implement validation before any baseline, live-file, or transaction publication.**
- [x] **Step 3: Add Core path ownership derivation and provenance checks.**

  `.baron/core/**` always derives `ManagedOwner::Core`; adapter-local paths derive Codex/Claude only when supported; legacy values remain generic migration input.

- [x] **Step 4: Run adapter and CLI duplicate/transaction tests.**

### Task 4: Implement transactional Core ownership transfer and skill classification

**Files:**
- Modify: `crates/baron-adapters/src/lib.rs`
- Modify: `crates/baron-adapters/src/update.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/tests/phase3_ownership.rs`

**Interfaces:**
- Produces `OwnershipMigrationReport` and `migrate_managed_ownership`; classification is encoded by baseline/hash evidence and surfaced through conflict and preservation reports.
- Migration acquires the Phase 2 project lock, validates the old baseline, classifies records, stages metadata/baseline changes, verifies live bytes, publishes atomically, and keeps the old manifest and source bytes recoverable until publication succeeds. No separate receipt file is needed because the v1 manifest and preserved baseline copies are the retry record.

- [x] **Step 1: Implement classification RED/GREEN cycles.**

  - A: baseline hash equals live hash → verified unchanged Baron-managed copy.
  - B: no Baron baseline/provenance → user-owned/custom; preserve.
  - C: baseline exists but live hash differs → blocking conflict with path/hash evidence; preserve.
  - D: insufficient or contradictory evidence → ambiguous; preserve and block ownership transfer.

- [x] **Step 2: Implement v1 Generic-owned `.baron/core/**` transfer.**

  Transfer metadata to `Core`, retain project/Vault identity and hash, stage any baseline storage move, and avoid rewriting identical live bytes.

- [x] **Step 3: Implement verified unchanged adapter-local skill migration planning and safe publication.**

  Map `.codex/skills/<name>/...` and `.claude/skills/<name>/...` to `.baron/core/skills/<name>/...` only when the baseline proves unchanged. Do not delete or overwrite a legacy file until the Core copy and new metadata verify successfully. Modified or ambiguous files return a blocking conflict.

- [x] **Step 4: Add deterministic publication-failure, rollback, retry, and idempotence evidence.**

  The deterministic publication-temp collision is the interruption/failure
  fixture: it proves the old v1 manifest and legacy bytes remain recoverable,
  staged Core bytes are rolled back, and a retry succeeds after the collision
  is removed.

- [x] **Step 5: Run focused ownership migration tests.**

### Task 5: Integrate ownership-aware install preservation without starting Phase 4

**Files:**
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/tests/phase1_target_red.rs`
- Modify: `crates/baron-adapters/tests/phase3_ownership.rs`

**Interfaces:**
- New installs preserve unknown existing adapter-local files and report conflicts rather than overwriting them.
- Existing adapter payload layout remains unchanged; Phase 4 canonical Core installation is not introduced here.

- [x] **Step 1: Run the six Phase 3 target tests after Tasks 2–4; only physical Core-installation assertions remain red.**
- [x] **Step 2: Make Codex/Claude embedded asset writes ownership-aware.**

  Missing files are created; identical files are no-ops; baseline-proven managed files may be updated through the migration/update path; unknown or modified files are preserved and reported.

- [x] **Step 3: Make `install_adapter` run migration/preflight before managed writes and publish the new baseline only after conflict-free validation.**
- [x] **Step 4: Promote the six Phase 3 ownership target tests that are now truly green.**
- [x] **Step 5: Run adapter lifecycle, planner, transaction, and migration tests.**

### Task 6: Update Phase 3 evidence and verify the scope boundary

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-06-codex-claude-core-phase3.md`

- [x] **Step 1: Record the old/new managed-state representation, migration rule, downgrade behavior, tests, and exact remaining red tests.**
- [x] **Step 2: Run focused Phase 3 tests and the existing relevant regressions.**
- [x] **Step 3: Run the required formatter, workspace tests, Clippy, and diff checks.**
- [x] **Step 4: Confirm no Phase 4–11 target was pulled forward and stop after the Phase 3 report.**


## Phase 3 execution result

Implemented in `crates/baron-adapters/src/update.rs` and `install.rs` rather than a separate ownership module so the existing managed-state validation and Safe I/O transaction path remain the single source of truth. The schema-2 manifest writes an explicit `owner`, optional migration provenance, and `minimum_writer_schema: 2`; the old adapter field remains an in-memory compatibility alias and is omitted from new serialized manifests. Schema 1 is read and normalized, then migrated transactionally. Verified unchanged `.codex/skills/**` and `.claude/skills/**` files move to `.baron/core/skills/**`; modified former managed files block; unregistered or ambiguous files are preserved and excluded from new ownership claims. Publication collisions restore both legacy bytes and staged Core files, after which retry succeeds.

Focused evidence:

- `cargo test -p baron-adapters --test phase3_ownership -- --nocapture`: 12/12.
- `cargo test -p baron-adapters --test phase1_target_red -- --nocapture`: 11 pass, 3 ignored Phase 4 tests.
- `cargo test -p baron-adapters --test update_planner -- --nocapture`: 15/15.
- `cargo test -p baron-adapters --all-targets --no-fail-fast`: all adapter targets pass.
- `cargo test -p baron-cli --test update_recovery_cli -- --nocapture`: 3/3;
  the verified candidate transaction now migrates legacy managed ownership
  before planning its merge ancestor.
- `cargo test --workspace --all-targets --no-fail-fast`: all product targets
  pass; two lifecycle installer tests remain environment-limited by the
  unavailable `Microsoft.PowerShell.Archive` module on this Windows host.

Remaining ignored tests are the three physical canonical-Core/projection assertions in `phase1_target_red.rs`; they are intentionally Phase 4 red tests.
