# Phase 6 Native Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a transactional Agent Bootstrap importer that converts useful data into Baron-native structures, validates custom assets, retires legacy runtime, and supports rollback.

**Architecture:** `baron-core::migration` owns inventory, backup, conversion, verification, cleanup, receipts, status, and rollback. `baron-cli` owns command parsing and calls the existing Baron configuration and adapter installers during the transaction. The migration writes only after an immutable Vault backup exists and removes legacy assets only after verification passes.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `toml`, `chrono`, existing Baron Vault/adapter modules, filesystem content hashes.

---

### Task 1: Lock The Native Migration Contract

**Files:**
- Create: `docs/superpowers/specs/2026-06-15-native-migration-legacy-retirement-design.md`
- Create: `docs/superpowers/plans/2026-06-15-phase-6-native-migration.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`

- [x] Write the approved design with import, preserve, quarantine, cleanup, and rollback boundaries.
- [x] Record the exact resume point as migration tests first.
- [ ] Commit the design and plan.

### Task 2: Add Failing Inventory And Dry-Run Tests

**Files:**
- Create: `crates/baron-core/tests/migration.rs`
- Create: `crates/baron-cli/tests/migration_cli.rs`
- Create: `crates/baron-core/src/migration.rs`
- Modify: `crates/baron-core/src/lib.rs`

- [ ] Test that dry-run detects `vault.config.json`, the scaffold manifest,
  runtime script, managed hook, legacy Vault capsule, plan/harness data, and
  custom assets.
- [ ] Test that dry-run classifies bundled assets separately from custom assets.
- [ ] Test that dry-run writes nothing to repo or Vault.
- [ ] Run `cargo test -p baron-core --test migration` and confirm failure because
  the migration API does not exist.
- [ ] Implement only the inventory model and read-only scanner.
- [ ] Run the core and CLI dry-run tests until they pass.

### Task 3: Add Transactional Backup And Import

**Files:**
- Modify: `crates/baron-core/src/migration.rs`
- Modify: `crates/baron-core/tests/migration.rs`

- [ ] Test that backup copies every candidate repo file and touched Vault path
  under `Artifacts/Baron/Migrations/<id>/`.
- [ ] Test that existing backup files are immutable.
- [ ] Test that memory files and nested folders import without overwriting
  existing Baron Markdown.
- [ ] Test normalized conversions from legacy plan/harness paths into
  `docs/baron/`.
- [ ] Run tests and confirm the new behaviors fail.
- [ ] Implement backup manifest, copy helpers, collision-safe import, and hash
  verification.
- [ ] Run the migration tests until they pass.

### Task 4: Validate And Quarantine Custom Assets

**Files:**
- Modify: `crates/baron-core/src/migration.rs`
- Modify: `crates/baron-core/tests/migration.rs`

- [ ] Test a valid custom Rust skill and backend agent are activated.
- [ ] Test missing metadata, workflow ownership conflicts, recursive agent
  orchestration, and Agent Bootstrap commands are quarantined.
- [ ] Test quarantine reports exact reasons and preserves original bytes.
- [ ] Run tests and confirm failure.
- [ ] Implement skill frontmatter checks, TOML agent checks, conflict checks,
  and quarantine copying.
- [ ] Run migration tests until they pass.

### Task 5: Install Baron And Retire Legacy Runtime

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-cli/tests/migration_cli.rs`
- Modify: `crates/baron-core/src/migration.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/src/managed.rs`

- [ ] Test `baron migrate agent-bootstrap --dry-run`.
- [ ] Test `baron migrate agent-bootstrap` creates Baron config and Codex
  adapter assets, preserves user text, and removes only verified legacy files.
- [ ] Test modified or unknown legacy-looking files survive cleanup.
- [ ] Test migration receipt records imported, quarantined, removed, and
  preserved counts.
- [ ] Run CLI tests and confirm failure.
- [ ] Add CLI parsing and orchestration.
- [ ] Add managed-block removal support and allowlisted cleanup.
- [ ] Run core, adapter, and CLI migration tests until they pass.

### Task 6: Add Status And Rollback

**Files:**
- Modify: `crates/baron-core/src/migration.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-core/tests/migration.rs`
- Modify: `crates/baron-cli/tests/migration_cli.rs`

- [ ] Test `baron migrate status` for never-run, completed, and rolled-back
  migrations.
- [ ] Test `baron migrate rollback --id <id>` restores recorded paths and does
  not touch unrelated post-migration files.
- [ ] Test an injected verification failure triggers automatic rollback.
- [ ] Run tests and confirm failure.
- [ ] Implement state lookup, rollback manifest replay, and failure receipts.
- [ ] Run migration tests until they pass.

### Task 7: Rewrite Core Assets As Baron-Native

**Files:**
- Modify: `assets/core/agents/code-reviewer.toml`
- Modify: `assets/core/agents/security-auditor.toml`
- Modify: `assets/core/agents/test-engineer.toml`
- Modify: `assets/core/skills/vibe-security-scan/SKILL.md`
- Modify: `assets/core/skills/vibe-security-scan/references/chunking-strategy.md`
- Modify: `assets/core/skills/vibe-security-scan/references/language-detection.md`
- Modify: `assets/core/skills/vibe-security-scan/rules/languages/README.md`
- Modify: `assets/core/skills/vibe-security-scan/workflows/large-review-sequential.md`
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`

- [ ] Add contract tests proving all core agents name Baron, Superpowers,
  evidence, Vault firewall, proof/trace gates, and no recursive orchestration.
- [ ] Add static tests proving active core assets contain no Agent Bootstrap
  runtime wording.
- [ ] Run adapter tests and confirm failure.
- [ ] Rewrite the assets without changing third-party attribution.
- [ ] Run adapter tests until they pass.

### Task 8: Documentation, Status, And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/architecture/ADAPTERS.md`
- Modify: `docs/architecture/MEMORY_MODEL.md`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Create: `notes/build-log/2026-06-15-phase-6-native-migration.md`

- [ ] Document migration, dry-run, automatic backup, quarantine, rollback, and
  legacy retirement in natural language.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test --workspace --all-targets`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run a fresh temp migration smoke with memory, custom assets, cleanup, and
  rollback.
- [ ] Run a static scan proving active Baron runtime/assets do not depend on
  Agent Bootstrap.
- [ ] Run `git diff --check`.
- [ ] Mark Phase 6 complete only after every command passes.
- [ ] Commit, merge to `main`, and push `origin/main`.

