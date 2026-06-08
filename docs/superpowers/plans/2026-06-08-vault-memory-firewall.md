# Vault Memory Firewall Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Baron Phase 2 so a shared Vault can safely hold memory for many projects without leaking unrelated project context into the current repo.

**Architecture:** Vault Markdown remains the durable source of truth. SQLite is a rebuildable cache under `Vault/Artifacts/Baron/`, and every recall/compact result goes through a firewall that prefers current-project memory, allows verified global memory only when relevant, and blocks cross-project memory unless the query strongly asks for it.

**Tech Stack:** Rust workspace, `baron-core`, `baron-cli`, `rusqlite` for disposable cache, Markdown/plain JSON for durable vault artifacts, `tempfile` + CLI integration tests.

---

## Scope

Phase 2 implements:

- `baron memory status [repo-path] --vault <vault-path>`
- `baron memory index [repo-path] --vault <vault-path>`
- `baron memory compact [repo-path] --vault <vault-path>`
- `baron recall "<query>" [repo-path] --vault <vault-path>`
- fallback vault lookup from `BARON_VAULT`
- vault scaffold and project capsule creation
- SQLite cache rebuild from Markdown
- memory firewall ranking and blocking
- multi-project isolation tests

Phase 2 does not implement:

- Codex/Claude adapter file generation
- `baron context`
- active plan commands
- Product Harness commands
- migration from `agent-bootstrap`

## File Structure

- Modify `crates/baron-core/Cargo.toml`: add `rusqlite`, and add any small parsing/hash dependency only if tests prove stdlib is not enough.
- Create `crates/baron-core/src/vault.rs`: vault path resolution, scaffold creation, project slug/path mapping, durable Markdown artifact paths.
- Create `crates/baron-core/src/memory.rs`: memory record model, Markdown scanning, confidence/status classification, SQLite cache build/read.
- Create `crates/baron-core/src/firewall.rs`: ranking rules, cross-project gating, compact brief generation, recall output selection.
- Modify `crates/baron-core/src/lib.rs`: export `vault`, `memory`, and `firewall`.
- Modify `crates/baron-cli/src/main.rs`: add `memory` and `recall` commands with `--vault`.
- Add `crates/baron-core/tests/vault_memory.rs`: core tests for scaffold, indexing, firewall, stale/unknown handling.
- Add `crates/baron-cli/tests/memory_cli.rs`: CLI smoke tests for status/index/compact/recall and no accidental project writes.
- Modify `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `notes/build-log/CURRENT.md`, `docs/architecture/MEMORY_MODEL.md`, `docs/architecture/COMMAND_SURFACE.md`, and `README.md`: update Phase 2 behavior and proof.

## Data Contract

Vault shape created by Phase 2:

```text
Vault/
  AGENTS.md
  Init.md
  Projects/
    <project-slug>/
      README.md
      Facts.md
      Decisions.md
      Tasks.md
      Plans/
      ProductHarness/
      Sessions/
      Artifacts/
        project-memory.json
  Artifacts/
    Baron/
      memory-index.sqlite
      memory-engine-state.json
      APPROVED_GLOBAL.md
      GLOBAL_CANDIDATES.md
```

Memory record fields:

```rust
pub struct MemoryRecord {
    pub id: String,
    pub scope: MemoryScope,
    pub project_slug: Option<String>,
    pub kind: MemoryKind,
    pub path: String,
    pub title: String,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub confidence: MemoryConfidence,
    pub status: MemoryStatus,
    pub updated_at: Option<String>,
    pub content_hash: String,
}
```

Firewall order:

```text
project_verified
project_likely
global_verified
project_stale as warning
cross_project only on strong explicit match
global_candidate never as fact
unknown remains unknown
```

## Task 1: Add Failing Vault Scaffold Tests

**Files:**
- Create: `crates/baron-core/tests/vault_memory.rs`
- Later modify: `crates/baron-core/src/vault.rs`

- [ ] Write tests that call `ensure_vault(vault_path, repo_path)`.
- [ ] Assert `Vault/Init.md`, `Vault/AGENTS.md`, `Vault/Projects/<slug>/README.md`, `Facts.md`, `Decisions.md`, `Tasks.md`, and `Artifacts/Baron/APPROVED_GLOBAL.md` exist.
- [ ] Assert the project slug is stable, lowercase, filesystem-safe, and based on the repo folder name.
- [ ] Run `cargo test -p baron-core vault_scaffold` and confirm failure because the module does not exist.

## Task 2: Implement Vault Path Resolution And Scaffold

**Files:**
- Create: `crates/baron-core/src/vault.rs`
- Modify: `crates/baron-core/src/lib.rs`

- [ ] Implement `resolve_vault_path(cli_vault: Option<PathBuf>) -> Result<PathBuf>` with this order: CLI `--vault`, `BARON_VAULT`, error with clear message.
- [ ] Implement `project_slug(repo_path) -> String`.
- [ ] Implement `ensure_vault(vault_path, repo_path) -> Result<VaultContext>`.
- [ ] Use create-if-missing only; never overwrite existing Markdown files.
- [ ] Run vault scaffold tests and confirm they pass.

## Task 3: Add Failing Memory Index Tests

**Files:**
- Modify: `crates/baron-core/tests/vault_memory.rs`
- Later create: `crates/baron-core/src/memory.rs`

- [ ] Seed two project capsules in one temp Vault: `tomoty` and `legacy-crm`.
- [ ] Write `Facts.md` and `Decisions.md` for both projects with overlapping words.
- [ ] Write `APPROVED_GLOBAL.md` and `GLOBAL_CANDIDATES.md`.
- [ ] Assert `build_memory_index(vault_context)` creates `Artifacts/Baron/memory-index.sqlite`.
- [ ] Assert index records include current project, other project, approved global, and global candidate with correct scopes.
- [ ] Assert missing or empty memory files become status diagnostics, not fake records.

## Task 4: Implement Markdown Scanner And SQLite Cache

**Files:**
- Modify: `crates/baron-core/Cargo.toml`
- Create: `crates/baron-core/src/memory.rs`
- Modify: `crates/baron-core/src/lib.rs`

- [ ] Add `rusqlite = { version = "0.32", features = ["bundled"] }`.
- [ ] Implement a small Markdown scanner that reads headings and bullets from `Facts.md`, `Decisions.md`, `Tasks.md`, `Plans/**/*.md`, `ProductHarness/**/*.md`, and `Sessions/**/*.md`.
- [ ] Classify source files into `fact`, `decision`, `task`, `plan`, `harness`, `session`, `global`.
- [ ] Store records in SQLite with fields from the Data Contract.
- [ ] Make the cache rebuildable by deleting and recreating Baron-owned tables only.
- [ ] Run core memory index tests and confirm they pass.

## Task 5: Add Failing Firewall Ranking Tests

**Files:**
- Modify: `crates/baron-core/tests/vault_memory.rs`
- Later create: `crates/baron-core/src/firewall.rs`

- [ ] Query `"auth login"` from project `tomoty`.
- [ ] Assert `tomoty` memory ranks before `legacy-crm` memory even if both contain `auth`.
- [ ] Assert `APPROVED_GLOBAL.md` can appear when relevant.
- [ ] Assert `GLOBAL_CANDIDATES.md` does not appear as trusted fact.
- [ ] Assert cross-project memory appears only when query explicitly includes the other project slug or a strong exact phrase.
- [ ] Assert stale/interrupted/draft/session-only records are marked as warnings or lower confidence.

## Task 6: Implement Memory Firewall And Recall

**Files:**
- Create: `crates/baron-core/src/firewall.rs`
- Modify: `crates/baron-core/src/lib.rs`

- [ ] Implement lexical token scoring with small normalization: lowercase, split punctuation, ignore short common words.
- [ ] Add project-scope boost for current project records.
- [ ] Add verified global boost only for records from `APPROVED_GLOBAL.md`.
- [ ] Add cross-project gate: require explicit project slug/path token or high exact phrase overlap.
- [ ] Add candidate gate: global candidates can be listed as “candidate” only in status/diagnostics, never as fact.
- [ ] Implement `recall(query, vault_context, limit)` returning ranked records plus blocked/skipped counts.
- [ ] Implement `compact_memory_brief(vault_context)` with bounded current-project facts, verified global hits, stale warnings, and unknowns.

## Task 7: Add CLI Tests For Memory And Recall

**Files:**
- Create: `crates/baron-cli/tests/memory_cli.rs`

- [ ] Test `baron memory status <repo> --vault <vault>` prints vault health, project capsule, index path, record counts, and firewall state.
- [ ] Test `baron memory index <repo> --vault <vault>` creates/rebuilds SQLite and reports counts.
- [ ] Test `baron memory compact <repo> --vault <vault>` prints a bounded Memory Firewall Brief.
- [ ] Test `baron recall "auth login" <repo> --vault <vault>` returns current-project memory first.
- [ ] Test commands fail clearly when neither `--vault` nor `BARON_VAULT` exists.
- [ ] Test command runs do not write into target repo paths.

## Task 8: Implement CLI Dispatch

**Files:**
- Modify: `crates/baron-cli/src/main.rs`

- [ ] Add `Memory` subcommand with `status`, `index`, and `compact`.
- [ ] Add top-level `Recall { query, repo_path, vault }` subcommand.
- [ ] Keep `repo_path` optional and default to current directory.
- [ ] Keep `--vault` optional but require either `--vault` or `BARON_VAULT`.
- [ ] Print human-readable Markdown by default.
- [ ] Ensure all output states when it wrote to the Vault and when it only read.

## Task 9: Update Docs And Build Status

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/architecture/MEMORY_MODEL.md`
- Modify: `docs/architecture/COMMAND_SURFACE.md`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `notes/build-log/CURRENT.md`

- [ ] Document Phase 2 commands and `--vault`/`BARON_VAULT`.
- [ ] Explain in natural language: Vault is the real memory, SQLite is only the fast card catalog.
- [ ] Explain Memory Firewall: current project first, approved global second, cross-project blocked unless explicit.
- [ ] Mark Phase 2 completed only after all tests and smoke checks pass.
- [ ] Keep Phase 3 listed as next: Context Compiler.

## Task 10: Final Verification And Commit

- [ ] Run `cargo fmt --all`.
- [ ] Run `cargo test`.
- [ ] Smoke:

```bash
cargo run -p baron-cli -- memory status . --vault .tmp/baron-vault
cargo run -p baron-cli -- memory index . --vault .tmp/baron-vault
cargo run -p baron-cli -- memory compact . --vault .tmp/baron-vault
cargo run -p baron-cli -- recall "survey engine proof" . --vault .tmp/baron-vault
```

- [ ] Multi-project smoke with one temp Vault and two temp repos; confirm current project memory wins.
- [ ] Run `git diff --check`.
- [ ] Commit:

```bash
git add .
git commit -m "feat: add vault memory firewall"
```

## Completion Criteria

Phase 2 is complete only when:

- a new Vault scaffold is created safely
- a project capsule is created safely
- SQLite index can be rebuilt from Markdown
- current-project memory ranks first
- verified global memory is allowed only when relevant
- global candidates are not treated as facts
- cross-project memory is blocked unless explicitly matched
- stale/unknown memory is labeled honestly
- CLI commands work with `--vault` and `BARON_VAULT`
- no target repo files are written by memory status/index/compact/recall
- docs/status/build log agree with actual behavior

## Self-Review

- Spec coverage: every Phase 2 checklist item from `docs/BARON_STATUS.md` maps to a task above.
- Placeholder scan: no task depends on undefined “later” behavior; Phase 3 context compiler is explicitly out of scope.
- Type consistency: `VaultContext`, `MemoryRecord`, `MemoryScope`, `MemoryKind`, `MemoryConfidence`, and `MemoryStatus` are introduced before use.
- Main tradeoff: Phase 2 adds `--vault`/`BARON_VAULT` even though the roadmap command list was shorter. This prevents Baron from guessing where memory should live.
