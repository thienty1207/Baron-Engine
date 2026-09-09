# Claude Native Thin Bridge Phase 9 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Make `baron init --claude` install a compact Claude-native projection over the canonical `.baron/core/**` runtime while preserving user-owned Claude and Codex files.

**Architecture:** Baron Core remains the semantic and lifecycle authority. Claude receives one managed `CLAUDE.md` contract, one explicit bridge skill with disabled model invocation, three small native subagent wrappers, the existing diagnostic command/index surfaces, and merged native settings hooks. Existing files are classified through the Phase 2/3 safe-I/O, lock, ownership, and managed-baseline primitives; collisions are preserved and reported.

**Tech Stack:** Rust workspace, `baron-adapters` managed payload/install/update planner, `baron-cli` lifecycle tests, embedded Core assets, Claude Markdown/YAML/JSON projections, `cargo test`/Clippy.

**Spec:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md` and the accepted Phase 9 request in `277c9c2f-e7ad-400b-bcfe-e222f79dac6a/pasted-text.txt`.

## Global Constraints

- Phase 9 only: do not implement Phase 10 active-adapter removal, Phase 11 Generic/retired adapter cleanup, Phase 12 final hook/idempotency architecture, Phase 13 Autopilot UX, Phase 14 release hardening, or final documentation cleanup.
- Persisted schema changes: `None.`; keep managed-state v2, `PreparePacketV1`, Task State, memory schemas, and project schema unchanged.
- Baron Core owns canonical skills and semantic agents under `.baron/core/**`; Claude files are projections and wrappers only.
- Fresh Claude installs must not materialize a complete semantic tree below `.claude/skills/**`; preserve existing user/custom/ambiguous content.
- `CLAUDE.md` owns automatic lifecycle correctness; the bridge skill uses `disable-model-invocation: true` and is explicit/diagnostic only.
- Preserve user CLAUDE text, custom skills, user subagents, commands, settings, hooks, Codex files, and unknown JSON keys.
- Claude host auto memory remains host-local non-authoritative context; Baron trusted project state remains authoritative and is never imported from auto memory.
- Task input reaches prepare through structured data; never shell-interpolate arbitrary user text.
- Write tests first, observe feature RED, implement the smallest behavior, keep focused and existing regression tests green, and classify the two known Windows Archive failures separately.

---

### Task 1: Freeze Claude Phase 9 target fixtures

**Files:**
- Create: `crates/baron-adapters/tests/phase9_claude_bridge.rs`
- Create: `crates/baron-cli/tests/phase9_claude_cli.rs`
- Modify: `crates/baron-cli/tests/phase1_target_red.rs` to promote only `target_claude_cli_install_is_a_thin_core_bridge`

**Interfaces:**
- Consumes: `install_adapter`, `managed_payloads_for_adapter`, managed baseline records, and the existing CLI `init`, `update`, and structured prepare commands.
- Produces: executable assertions for native Claude paths, frontmatter, Core provenance, compact CLAUDE contract, auto-memory policy, settings/hook preservation, collision safety, idempotency, and Codex coexistence.

- [x] **Step 1: Write adapter-level failing tests**

  Add tests that install Claude into a clean repository and assert the future
  surface: `CLAUDE.md`, `.claude/skills/baron-engine/SKILL.md`, three
  `.claude/agents/*.md` wrappers, `.claude/settings.json`, and the canonical
  `.baron/core/**` tree. Assert no semantic `.claude/skills/<core-skill>` body,
  bridge frontmatter with `disable-model-invocation: true`, wrapper provenance,
  structured prepare guidance, selected resources only, user intent precedence,
  bounded CLAUDE size, verification/recovery rules, and non-authoritative
  Claude auto memory wording. Add fixtures for user CLAUDE text, custom skill,
  user subagent, personal command, third-party settings/hooks, malformed
  settings, Codex files, and collisions at Baron-owned bridge/wrapper paths.

- [x] **Step 2: Run adapter tests and verify feature RED**

  Run:

  ```text
  cargo test -p baron-adapters --test phase9_claude_bridge -- --nocapture
  ```

  Expected failure: the current Claude payload set has no bridge, no native
  subagent wrappers, and uses the older full startup contract. Preserve the
  failure evidence in the Phase 9 build note; fix only fixture/setup errors.

- [x] **Step 3: Write CLI failing tests**

  Exercise fresh and repeated `baron init --claude`, `baron update --dry-run
  --installed`, and `control-plane prepare --adapter claude --json` with
  structured adversarial task input. Assert no semantic Claude skill copy,
  unchanged Codex projection, preserved user files, valid settings, and stable
  snapshots.

- [x] **Step 4: Run CLI tests and verify feature RED**

  Run:

  ```text
  cargo test -p baron-cli --test phase9_claude_cli -- --nocapture
  ```

  Expected failure: the current CLI install cannot produce the Claude bridge or
  wrappers. Do not weaken target assertions.

- [x] **Step 5: Promote the existing Claude target**

  Remove only the ignore marker from `target_claude_cli_install_is_a_thin_core_bridge` after the new target tests prove the expected failure. Leave Generic/retired-adapter targets ignored for their later phases.

### Task 2: Implement Claude-native thin projections

**Files:**
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/src/managed.rs` only if a behavior-neutral reusable projection helper is required
- Modify: `crates/baron-adapters/src/update.rs` only if the existing planner needs no new policy, only path/owner compatibility
- Test: `crates/baron-adapters/tests/phase9_claude_bridge.rs`

**Interfaces:**
- Consumes: Core payloads, `ManagedAssetPayload`, `ManagedMergeKind`, safe text/byte I/O, `upsert_managed_block`, routing merges, native settings hook merge, managed baseline ownership, and Claude adapter identity.
- Produces: deterministic Claude bridge, metadata/frontmatter, three wrappers, concise contract, preserved diagnostic commands/indexes, merged settings, and collision reports.

- [x] **Step 1: Add payload assertions for every Claude-native file**

  Assert exact paths, `adapter == "claude"`, `FullText` for the bridge/wrappers
  and any retained diagnostic command, `MarkerBlock` for `CLAUDE.md`, routing
  blocks for indexes, and `JsonOwnedEntries` for settings. Assert no payload
  path is a copied Core skill body.

- [x] **Step 2: Implement the bridge skill and invocation policy**

  Generate `.claude/skills/baron-engine/SKILL.md` with Claude-compatible
  frontmatter including `disable-model-invocation: true`. The body references
  structured `PrepareRequestV1`, `baron control-plane prepare --adapter claude
  --json`, existing `PreparePacketV1`, selected Core roots, relative resources,
  explicit user intent, bounded context, and completion evidence. It must not
  contain copied skill bodies or a second router.

- [x] **Step 3: Implement Core-provenance Claude subagent wrappers**

  Generate `.claude/agents/code-reviewer.md`, `security-auditor.md`, and
  `test-engineer.md` with small YAML frontmatter and explicit canonical source
  paths under `.baron/core/agents/`. Wrappers return bounded findings/evidence
  to the parent and cannot own plans, adapters, lifecycle, memory, or overall
  completion.

- [x] **Step 4: Implement the compact CLAUDE.md contract**

  Add a Claude-specific managed contract retaining the existing managed markers
  and user text. Include automatic structured prepare, `PreparePacketV1`, user
  precedence, selected Core resources only, resume/recovery, proportional
  lifecycle, proof/verification/checkpoint rules, one-question clarification,
  no hidden CLI requirement, no recursive Core load, hook fallback, route
  authority, and the explicit non-authoritative auto-memory policy. Keep it
  below the regression size bound and retain phrases required by current
  adapter contract tests.

- [x] **Step 5: Install through safe ownership and preserve commands/settings**

  Route bridge, wrappers, and retained commands through an ownership-aware
  full-text helper: identical bytes are no-ops; an unchanged Baron-owned
  baseline may be refreshed; a differing unknown or modified file is preserved
  and reported. Keep routing/index merges and native settings hook merging.
  Do not overwrite malformed JSON, user CLAUDE text, custom skills, user
  agents, commands, Codex files, or third-party settings/hooks.

- [x] **Step 6: Run adapter-focused green verification**

  Run the Phase 9 adapter test plus `phase4_core`, `phase5 prepare`, `phase6
  trusted memory`, `phase7_routing`, `phase8_codex_bridge`, `adapter_lifecycle`,
  and `update_planner`. Repair production or fixture defects only; do not alter
  target semantics to make tests pass.

### Task 3: Wire CLI/update evidence and auto-memory compatibility

**Files:**
- Modify: `crates/baron-cli/tests/phase1_target_red.rs`
- Create: `crates/baron-cli/tests/phase9_claude_cli.rs`
- Modify: `crates/baron-cli/tests/adapter_cli.rs` only for Claude native assertions
- Modify: `crates/baron-cli/src/main.rs` only if payload aggregation omits Claude projections
- Create: `docs/compatibility/CLAUDE.md`

**Interfaces:**
- Consumes: Claude payloads and existing CLI init/update/prepare paths.
- Produces: CLI proof that Claude init, repeated init, update planning, structured prepare, multi-adapter coexistence, and host auto-memory boundaries use the same Core.

- [x] **Step 1: Add CLI preservation and coexistence assertions**

  Preserve user CLAUDE text, custom Claude skill, user agent, personal command,
  settings keys, third-party hooks, and all Codex files across Claude init and
  repeated initialization. Assert the shared Core bytes remain unchanged.

- [x] **Step 2: Add explicit auto-memory policy tests**

  Assert generated CLAUDE guidance treats host auto memory as non-authoritative,
  does not import host notes into trusted Baron memory, does not write Baron
  state to host auto memory, and preserves an existing explicit auto-memory
  preference in settings.

- [x] **Step 3: Verify update/dry-run behavior**

  Assert registered Claude update plans include the new projection exactly once,
  fresh updates do not add semantic `.claude/skills` bodies, collisions remain
  staged/preserved, and dry-run leaves project bytes unchanged.

- [x] **Step 4: Record compatibility evidence**

  Add `docs/compatibility/CLAUDE.md` describing native paths, disabled implicit
  invocation, Core authority, prepare transport, hook fallback, auto-memory
  decision, preservation boundaries, and external runtime evidence limits.

### Task 4: Update durable Phase 9 records and verify

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-08-claude-native-thin-bridge-phase9.md`
- Modify: `notes/build-log/CURRENT.md`

**Interfaces:**
- Consumes: Phase 9 focused test results, native Claude layout, auto-memory decision, and full verification output.
- Produces: phase-complete evidence with Phase 10 as the next action and no later-phase implementation.

- [x] **Step 1: Run required verification**

  Run the focused Phase 9 tests first, then:

  ```text
  cargo fmt --all -- --check
  cargo test --workspace --all-targets --no-fail-fast
  cargo clippy --workspace --all-targets -- -D warnings
  git diff --check
  ```

  Also rerun canonical Core, prepare, trusted memory/context, routing, Codex
  bridge, adapter lifecycle, and update planner tests. Classify the known
  PowerShell Archive failures separately if unchanged.

- [x] **Step 2: Complete the Phase 9 report**

  Record the Claude tree, CLAUDE contract size, bridge frontmatter, wrapper
  provenance, auto-memory decision, Core authority, prepare compatibility,
  preservation/idempotency evidence, files changed, RED-to-GREEN targets,
  remaining future-phase RED tests, schema compatibility, failures, discoveries,
  spec adjustment, and `PHASE 10 READINESS`.

- [x] **Step 3: Stop after the report**

  Do not begin Phase 10 or any later phase, and do not create a release or tag.
