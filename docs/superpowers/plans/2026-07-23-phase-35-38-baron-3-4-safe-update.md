# Baron 3.4 Safe Self-Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `baron update` safely update both the Baron runtime and Baron-managed project assets while preserving user content, custom skills/agents, Vault memory, and a recoverable working installation.

**Architecture:** Record the last installed managed baseline, compute a conservative three-way update plan, verify an immutable platform-specific release candidate, let that candidate render the new assets, and activate project/runtime changes as one recoverable transaction. Human `baron update` owns remote update authority; AI `baron automation reconcile` remains local-only.

**Tech Stack:** Rust 2021, Clap, Serde JSON, SHA-256, existing Baron adapter/release modules, HTTPS client with rustls, filesystem transactions, PowerShell/Bash installer compatibility, GitHub Actions.

---

### Task 1: Phase 35 Managed Baseline And Update Planner

**Files:**
- Create: `crates/baron-adapters/src/update.rs`
- Modify: `crates/baron-adapters/src/lib.rs`
- Modify: `crates/baron-adapters/src/managed.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/Cargo.toml`
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-adapters/tests/update_planner.rs`
- Test: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Test: `crates/baron-cli/tests/update_cli.rs`

**Required model:**

```rust
pub enum ManagedMergeKind {
    MarkerBlock,
    RoutingBlock,
    JsonOwnedEntries,
    FullText,
}

pub struct ManagedAssetRecord {
    pub adapter: String,
    pub relative_path: PathBuf,
    pub base_sha256: String,
    pub installed_version: String,
    pub merge_kind: ManagedMergeKind,
}

pub enum UpdateDisposition {
    TakeUpstream,
    KeepLocal,
    Identical,
    AutoMerge,
    Conflict,
}
```

- [ ] Write RED tests proving first install records `.baron/managed-state/manifest.json` and exact baseline copies without placing absolute paths in the manifest.
- [ ] Write RED tests for the three-way matrix: unchanged local takes upstream, unchanged upstream keeps local, equal local/upstream deduplicates, marker content updates without touching surrounding text, and ambiguous dual edits become conflicts.
- [ ] Write RED tests proving custom `.codex/skills`, custom `.codex/agents`, custom routing sections, source files, plans, Harness records, and Vault paths never enter the managed update set.
- [ ] Write a RED repeated-update test proving a successful activation replaces the old baseline, so the next plan compares against the last installed release rather than the first release.
- [ ] Run `cargo test -p baron-adapters --test update_planner` and confirm failures are caused by the missing baseline/planner.
- [ ] Implement canonical repository-relative managed records and SHA-256 baseline snapshots.
- [ ] Refactor managed writers so install/update return the rendered managed payload and merge kind before any write. Keep existing first-install output byte-compatible.
- [ ] Implement a read-only `plan_managed_update(...)` that returns actions, conflicts, preserved paths, and diagnostics without changing target files.
- [ ] Add hidden `baron update --dry-run --installed` for inspecting the currently installed embedded asset update plan; keep it out of the normal README flow.
- [ ] Reject path traversal, symlink/junction escape, duplicate managed ownership, malformed marker pairs, unsupported manifest schema, and missing baseline with actionable diagnostics.
- [ ] Run focused adapter/CLI tests, then `cargo test -p baron-adapters --all-targets`.
- [ ] Update `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, and the Phase 35 build-log checkpoint with fresh evidence.
- [ ] Commit Phase 35 independently with message `feat: add managed update baseline and planner`.

### Task 2: Phase 36 Verified Release Candidate And Binary Handoff

**Files:**
- Create: `crates/baron-cli/src/self_update.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-cli/Cargo.toml`
- Modify: `crates/baron-core/src/release.rs`
- Modify: `crates/baron-core/tests/release.rs`
- Modify: `crates/baron-cli/tests/release_cli.rs`
- Create: `crates/baron-cli/tests/self_update_cli.rs`
- Modify: `.github/workflows/release.yml`
- Modify: `installers/install.ps1`
- Modify: `installers/install.sh`
- Modify: `Cargo.lock`

**Required model:**

```rust
pub struct UpdateCandidate {
    pub version: String,
    pub source_revision: String,
    pub target: String,
    pub executable_name: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub staged_path: PathBuf,
}

pub trait CandidateSource {
    fn latest_manifest(&self) -> Result<ReleaseManifest>;
    fn fetch_candidate(
        &self,
        artifact: &ReleaseUpdateArtifact,
        destination: &Path,
    ) -> Result<()>;
}
```

- [ ] Write RED manifest tests requiring exactly one raw update candidate for each supported native target while retaining existing install archives.
- [ ] Write RED resolver tests for target selection, same-version no-op, downgrade refusal, malformed exact source revision, missing target, size mismatch, SHA-256 mismatch, and candidate `--version` mismatch.
- [ ] Write RED CLI tests proving candidate discovery/download/verification writes only under `.baron/update/` and cannot modify managed project targets or the installed executable.
- [ ] Use an injected directory-backed source for deterministic tests; tests must not call GitHub or require internet access.
- [ ] Run focused release/self-update tests and confirm they fail at the missing candidate schema/service boundary.
- [ ] Extend `ReleaseManifest` with a backward-compatible update-candidate list and increment its schema with explicit schema-1 read compatibility for existing installers.
- [ ] Add raw native executables to the release workflow and include their target/version/source/checksum/size identity in generated release metadata.
- [ ] Add an HTTPS-only production candidate source using a rustls-backed client with bounded response size, connect/read timeout, and redacted diagnostics.
- [ ] Download to a transaction-specific temporary path, fsync/close before hashing, then verify manifest product, schema, version ordering, target, byte size, checksum, exact source revision, and executable `--version`.
- [ ] Reject redirects or final URLs that leave the approved GitHub release host unless an explicitly configured trusted mirror is in use.
- [ ] Implement platform handoff primitives: direct atomic replacement support for Unix and delayed verified finalizer metadata for Windows. Do not activate them yet.
- [ ] Keep the existing PowerShell/Bash installer lifecycle compatible with the extended manifest and prove explicit-version/latest install still work.
- [ ] Run focused tests, release metadata smoke, installer lifecycle tests, and workflow contract tests.
- [ ] Update status/JSON/build log with Phase 36 evidence.
- [ ] Commit Phase 36 independently with message `feat: verify native self-update candidates`.

### Task 3: Phase 37 Conflict-Safe Activation And Recovery

**Files:**
- Modify: `crates/baron-adapters/src/update.rs`
- Create: `crates/baron-cli/src/update_transaction.rs`
- Modify: `crates/baron-cli/src/self_update.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-adapters/tests/update_transaction.rs`
- Test: `crates/baron-cli/tests/update_recovery_cli.rs`
- Test: `crates/baron-cli/tests/self_update_cli.rs`
- Test: `crates/baron-cli/tests/execution_cli.rs`

**Transaction statuses:**

```text
discovered -> downloaded -> verified -> planned
planned -> conflict | project_activated
conflict -> planned | aborted
project_activated -> runtime_pending -> completed
any applied state -> rolled_back
```

- [ ] Write RED tests proving any conflict leaves every managed target, active binary, Vault file, source file, custom skill, and custom agent byte-identical.
- [ ] Write RED tests for staged `BASE`, `LOCAL`, `UPSTREAM`, and `RESOLVED` packets with stable transaction id, relative target, merge kind, and frozen hashes.
- [ ] Write RED continuation tests proving `--continue` refuses changed project identity, target file, staged input, resolved file without recorded hash, candidate binary, or transaction schema.
- [ ] Write RED abort tests proving `--abort` deletes only the staged transaction and never rewrites the project or Vault.
- [ ] Write RED failure-injection tests at every write checkpoint, including project backup, first/middle/last managed write, baseline update, runtime handoff, candidate validation, receipt write, and recovery startup.
- [ ] Write RED Windows tests for parent-exit finalization metadata and backup restoration; write Unix tests for atomic candidate activation.
- [ ] Run focused tests and confirm failures are caused by the absent transaction/continuation/recovery behavior.
- [ ] Implement the transaction directory, monotonic state machine, frozen hashes, per-target backups, atomic target writes, and activation receipt.
- [ ] Let the verified candidate render and apply the new managed assets through a hidden candidate protocol. Revalidate repository identity and transaction paths in the candidate process.
- [ ] Implement conservative structural/marker merges and conflict staging. Never place conflict markers directly in a live project file.
- [ ] Implement hidden `baron update --continue` and `baron update --abort`; keep normal user documentation centered on `baron update`.
- [ ] Implement rollback so project managed files, baseline records, and active runtime return to one compatible version if any post-write step fails.
- [ ] Implement startup recovery for `runtime_pending` and incomplete applied transactions. Recovery must complete a fully verified handoff or roll back; it cannot infer success.
- [ ] Prove repeated update, interrupted update, offline-after-download, locked file, same-name project, irregular old repo, and shared-Vault scenarios.
- [ ] Run focused transaction/recovery suites, all adapter tests, and CLI all-target tests.
- [ ] Update status/JSON/build log with Phase 37 evidence.
- [ ] Commit Phase 37 independently with message `feat: activate Baron updates transactionally`.

### Task 4: Phase 38 Automation Contract And Baron 3.4 Certification

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Modify: `crates/baron-cli/tests/automation_cli.rs`
- Modify: `crates/baron-cli/tests/update_cli.rs`
- Modify: `crates/baron-cli/tests/lifecycle_scripts.rs`
- Modify: `crates/baron-core/tests/public_trust_docs.rs`
- Modify: `crates/baron-cli/tests/workflow_contract.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`
- Modify: `docs/COMMAND_SURFACE.md`
- Modify: `docs/RELEASE.md`
- Create: `docs/assessment/baron-3.4.0-safe-update-certification.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `notes/build-log/2026-07-23-phase-35-38-baron-3-4-safe-update.md`

- [ ] Write RED adapter tests requiring state-mismatch guidance to use local-only `baron automation reconcile`, never silent public `baron update`.
- [ ] Write RED automation tests proving reconcile uses only installed embedded assets, makes no network calls, cannot replace the binary, and still obeys baseline/conflict preservation.
- [ ] Write RED public-flow tests proving top-level help remains simple and the README exposes only install, setup Vault, init adapter/platform, and update.
- [ ] Write RED lifecycle tests for `3.3.x` one-time installer bootstrap, `3.4.0` same-version refresh, future-version self-update, conflict/continue, abort, rollback, and post-crash recovery.
- [ ] Extend `baron automation reconcile` to repair only currently installed Baron-managed assets and record observed automation evidence.
- [ ] Update Codex, Claude, and generic startup contracts so AI local repair and human release authority cannot be confused.
- [ ] Bump workspace and lockfile version to `3.4.0` only after Phases 35-37 are green.
- [ ] Rewrite update/release documentation in natural language, including the honest one-time `3.3.x -> 3.4.0` installer boundary and the fact that AI never silently installs releases.
- [ ] Run the full certification batch:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
npx --yes yaml-lint .github/workflows/release.yml
git diff --check
```

- [ ] Run real lifecycle smoke with a local immutable release fixture: clean install, setup Vault, Codex/fullstack init, custom skill/agent edits, managed-file local edit, successful update, conflict stage/continue, abort, interrupted recovery, rollback, context, memory firewall, and `baron --version`.
- [ ] Run Windows delayed-finalizer smoke locally and require hosted Linux plus Intel/Apple Silicon macOS jobs to prove native activation behavior.
- [ ] Verify release archives, raw candidates, checksums, manifest identity, installers, source revision, and version all agree.
- [ ] Record exact command evidence in `docs/assessment/baron-3.4.0-safe-update-certification.md`; mark Phase 38 complete only after every required proof passes.
- [ ] Merge verified implementation to `main` and push `origin/main`. Do not create a tag or GitHub Release unless the user explicitly requests release promotion.
- [ ] Commit Phase 38 independently with message `release: certify Baron 3.4 safe update`.

## Plan Self-Review

- The public experience remains one command: `baron update`.
- The AI never receives silent authority to download or activate a release.
- The old binary never renders new release assets.
- Project source and Vault memory are outside the update write set.
- Custom skills, agents, and routing remain user-owned.
- Every ambiguous merge stops before live target writes.
- Candidate, project, and runtime activation share one recoverable transaction.
- Windows executable locking has an explicit delayed-finalizer and receipt path.
- The immutable exact-source release contract remains in force.
- No production implementation or version bump belongs in the planning commit.
