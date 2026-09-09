# Codex Native Thin Bridge Phase 8 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Make `baron init --codex` install a compact Codex-native projection that routes normal work to the canonical `.baron/core/**` runtime without recreating Baron skill semantics under `.codex/skills/**`.

**Architecture:** Baron Core remains the sole semantic owner. The Codex adapter publishes one managed AGENTS contract, one thin `.agents/skills/baron-engine` bridge with explicit metadata, three bounded `.codex/agents` wrappers, a diagnostic index, and merged native hooks. Existing user files are classified through the Phase 2/3 managed-state and safe-I/O primitives; ambiguous full-text projection collisions are preserved and reported.

**Tech Stack:** Rust workspace, `baron-adapters` managed payload/update planner, `baron-cli` lifecycle tests, `include_dir` embedded Core assets, JSON/TOML/Markdown native projections, `cargo test`/Clippy.

**Spec:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md` and the accepted Phase 8 request in `ae7e64ee-c8ab-4d82-8ec9-ba464283c7ac/pasted-text.txt`.

## Global Constraints

- Phase 8 only: do not implement Claude's final bridge, global adapter-authority removal, retired adapter/Generic purge, final hooks/idempotency, Autopilot UX, release work, or docs rewrite.
- Persisted schema changes: `None.`; keep managed-state v2 and `PreparePacketV1` unchanged.
- Baron Core owns canonical skills and semantic agents under `.baron/core/**`; Codex files are projections/wrappers only.
- New Codex installs must not materialize a full Baron skill tree under `.codex/skills/**`; preserve existing user/custom content and ambiguous legacy files.
- `AGENTS.md` remains the automatic correctness contract; the bridge metadata must set `allow_implicit_invocation: false` and restrict products to Codex.
- Preserve user text, unrelated `.agents/skills`, unrelated `.codex/agents`, hooks, settings, and Claude-owned files.
- All task input reaches `control-plane prepare` through structured data; never shell-interpolate arbitrary user text.
- Write tests first, observe the expected failure, keep focused tests green, and separate the known Windows PowerShell Archive host failures.

---

### Task 1: Freeze Phase 8 Codex fixtures and target tests

**Files:**
- Create: `crates/baron-adapters/tests/phase8_codex_bridge.rs`
- Create: `crates/baron-cli/tests/phase8_codex_cli.rs`
- Modify: `crates/baron-cli/tests/phase1_target_red.rs` (promote the accepted Codex target only)

**Interfaces:**
- Consumes: `install_adapter`, `managed_payloads_for_adapter`, Core baseline records, and the existing CLI `init` command.
- Produces: executable assertions for native paths, metadata, wrapper provenance, AGENTS size/markers, preservation, idempotency, Core sharing, and structured prepare guidance.

- [x] **Step 1: Write the failing adapter-level tests**

  Add tests that install Codex into a clean temp repository and assert:
  `AGENTS.md`, `.agents/skills/baron-engine/SKILL.md`,
  `.agents/skills/baron-engine/agents/openai.yaml`, `.codex/INDEX.md`,
  `.codex/agents/{code-reviewer,security-auditor,test-engineer}.toml`, and
  `.codex/hooks.json` exist; `.baron/core/skills/superpowers/SKILL.md` and
  `.baron/core/agents/code-reviewer.toml` remain the only semantic sources;
  no `.codex/skills/<core-skill>/SKILL.md` exists; metadata parses and disables
  implicit invocation; wrappers reference their matching Core path; the
  managed AGENTS block is bounded and contains the automatic prepare/resume,
  selected-resource, preservation, checkpoint, verification, and blocker rules.
  Add preservation fixtures for user AGENTS text, unrelated bridge skill,
  user-named agent wrapper, third-party hook entries, and a Claude file.
  Add repeated-install and Codex-then-Claude assertions proving only Codex
  projections change and Core bytes stay identical.

- [x] **Step 2: Run the focused tests and confirm the expected RED**

  Run:

  ```text
  cargo test -p baron-adapters --test phase8_codex_bridge -- --nocapture
  cargo test -p baron-cli --test phase8_codex_cli -- --nocapture
  ```

  Expected: fail because the current installer has no `.agents/skills/baron-engine` bridge, no `openai.yaml`, and no native Codex wrapper payloads. Fix test/setup errors until the failures are feature failures, then keep the failure evidence in the Phase 8 build note.

- [x] **Step 3: Promote the existing Codex target test**

  Remove only the ignore marker from `target_codex_cli_install_is_a_thin_core_bridge`; retain its assertions unchanged. Leave Claude and future-phase target tests ignored.

### Task 2: Implement thin Codex projections over Core

**Files:**
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/src/managed.rs` only if a behavior-neutral marker/merge helper is needed
- Modify: `crates/baron-adapters/src/update.rs` only if new projection records need the existing owner/baseline path
- Test: `crates/baron-adapters/tests/phase8_codex_bridge.rs`

**Interfaces:**
- Consumes: `core_managed_payloads`, `ManagedAssetPayload`, `ManagedMergeKind`, `read_bytes`, `replace_file`, `ensure_managed_baseline`, and existing native-hook merge behavior.
- Produces: deterministic Codex payloads for the bridge, metadata, three wrappers, index, AGENTS contract, and hooks; safe collision reporting through `InstallReport`.

- [x] **Step 1: Add minimal payload/rendering tests for each new native file**

  Assert exact relative paths, `adapter == "codex"`, `ManagedMergeKind::FullText` for bridge/metadata/index/wrappers, and `ManagedMergeKind::JsonOwnedEntries` for hooks. Assert each wrapper contains its canonical `.baron/core/agents/<name>.toml` identity and the bridge contains no copied Core skill body.

- [x] **Step 2: Implement deterministic bridge and metadata bodies**

  Generate `.agents/skills/baron-engine/SKILL.md` as a short native bridge that names Baron Core, invokes structured `control-plane prepare --adapter codex --json`, loads only selected canonical skill roots and relative resources, preserves explicit user intent, and fails clearly when Core/runtime is unavailable. Generate `agents/openai.yaml` with `policy.allow_implicit_invocation: false` and `products: [codex]`.

- [x] **Step 3: Implement Core-provenance Codex agent wrappers**

  Generate exactly the three mandatory wrapper files. Each wrapper must contain only native identity/role/evidence instructions, the matching canonical Core source path, a bounded-child-agent rule, and parent-owned completion language. Do not copy the Core `developer_instructions` body into the wrapper.

- [x] **Step 4: Replace the Codex startup contract with the compact automatic contract**

  Keep the existing managed markers and user text outside them. Include the required lifecycle phrases used by current regression tests, then add the Phase 8 rules: automatic prepare, proportional work shape, resume before re-investigation, selected skills only, explicit user precedence, Core-relative resource resolution, checkpoint/recovery, proof/verification before completion, bounded child delegation, hook fallback, and no manual hidden CLI operation. Keep it below the tested size limit and never enumerate the complete Core tree.

- [x] **Step 5: Install projections through the existing safe ownership boundary**

  Extend `install_codex` and `managed_payloads_for_adapter` to publish the bridge, metadata, wrappers, and index. Reuse native JSON hook merging and existing marker/routing merges. For new full-text projection paths, preserve a pre-existing differing file and report a conflict unless the managed baseline proves it is an unchanged Baron-owned copy. Do not touch Claude files or global user configuration. Preserve an existing `.codex/skills/**` tree and do not add new Core skill bodies.

- [x] **Step 6: Run adapter tests and repair only production/test defects**

  Run the new adapter test plus `phase4_core`, `phase2_current`, `phase3_ownership`, `adapter_lifecycle`, and `update_planner`. Keep the Core and existing preservation assertions green; do not weaken Phase 8 target assertions.

### Task 3: Wire CLI lifecycle/update coverage and compatibility notes

**Files:**
- Create: `crates/baron-cli/tests/phase8_codex_cli.rs` (if Task 1 did not create it)
- Modify: `crates/baron-cli/src/main.rs` only if CLI reporting or update payload selection omits new Codex projections
- Modify: `crates/baron-cli/tests/adapter_cli.rs` only to assert the new native surface while preserving unrelated custom fixtures
- Create: `docs/compatibility/CODEX.md`

**Interfaces:**
- Consumes: the adapter payload set and existing `update`/`init` transaction paths.
- Produces: CLI evidence that fresh init, repeated init, update, multi-adapter coexistence, and structured prepare guidance use the same Core without manual Baron commands.

- [x] **Step 1: Add CLI red/green tests**

  Exercise `baron init <repo> --codex --vault <vault>`, repeated init, `baron update --dry-run --installed`, and `control-plane prepare --adapter codex --json` with structured stdin. Assert the native files, bounded AGENTS markers, no Core copy in `.codex/skills`, unchanged Claude files, and idempotent snapshots.

- [x] **Step 2: Update the CLI payload aggregation only if required**

  Ensure registered Codex update plans include the new payloads exactly once and preserve existing custom files. Do not change packet schema, adapter enum, or global active-adapter policy.

- [x] **Step 3: Record Codex compatibility evidence**

  Add `docs/compatibility/CODEX.md` with the verification date, current source-level native paths, metadata policy, hook trust/fallback caveat, Core-relative resource rule, and the fact that no external Codex executable was claimed unless detected and run.

### Task 4: Update Phase 8 durable records and verify the workspace

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-07-codex-native-thin-bridge-phase8.md`
- Modify: `notes/build-log/CURRENT.md`

**Interfaces:**
- Consumes: focused test results and the final native layout.
- Produces: phase-complete status and an exact verification record with Phase 9 as the next action.

- [x] **Step 1: Run focused verification**

  Run:

  ```text
  cargo test -p baron-adapters --test phase8_codex_bridge -- --nocapture
  cargo test -p baron-cli --test phase8_codex_cli -- --nocapture
  cargo test -p baron-adapters --test phase4_core -- --nocapture
  cargo test -p baron-core --test prepare -- --nocapture
  cargo test -p baron-core --test phase6_trusted_memory -- --nocapture
  cargo test -p baron-core --test phase7_routing -- --nocapture
  cargo test -p baron-adapters --test adapter_lifecycle -- --nocapture
  cargo test -p baron-adapters --test update_planner -- --nocapture
  cargo fmt --all -- --check
  cargo test --workspace --all-targets --no-fail-fast
  cargo clippy --workspace --all-targets -- -D warnings
  git diff --check
  ```

  Record the two known Windows `Microsoft.PowerShell.Archive` lifecycle failures separately if they remain the only workspace failures.

- [x] **Step 2: Complete the Phase 8 report**

  Record changed files, native tree, AGENTS size/boundary, bridge metadata, Core authority, prepare behavior, preservation/idempotency evidence, optional runtime result, remaining future-phase RED tests, schema/packet compatibility, unexpected failures, discoveries, spec adjustment, and `PHASE 9 READINESS`.

- [x] **Step 3: Stop after the report**

  Do not begin Claude bridge or any later phase, and do not create a release or tag.
