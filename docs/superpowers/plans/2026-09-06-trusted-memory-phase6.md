# Trusted Memory Retrieval and Priority Context — Phase 6 Implementation Plan

> **Execution boundary:** Phase 6 only. Do not begin Phase 7 profile-aware
> routing, Phase 8/9 adapter bridges, Phase 10 active-adapter redesign, Phase
> 11 adapter retirement, release work, or final documentation rewrite.

**Goal:** Make context-critical memory use one named trusted retrieval
authority, compile a bounded priority-aware context with protected Tier-0
state, and expose one bounded Task State projection over existing Baron
authorities.

**Architecture:** Preserve the existing memory schema, Vault, semantic ranker,
project firewall, trust states, and temporal ledger. Add a named current
retrieval boundary in `baron-core::firewall`, route context/resume/direct
current recall through it, and compile Task State plus tiered context without
creating persisted task state or a second memory engine.

**Required compatibility:** Keep `PreparePacketV1` fields and serialization
compatible. Prepare continues to call `compile_context_for_task`; Phase 6
changes the context compiler behind that existing boundary.

## Global constraints

- Candidate, contested, superseded, expired, and stale/invalid current records
  must not enter trusted task context.
- Likely records remain eligible only where the existing trust policy already
  permits them; do not replace the policy with a Verified-only rule.
- Global verified memory follows the existing firewall; global candidates and
  weak cross-project records remain blocked.
- Existing temporal/currentness and provenance semantics remain authoritative.
- Task State is a runtime projection. No new persisted schema or task store.
- Tier 0 is protected by per-field bounds and is never lost to lower tiers.
- Context output remains deterministic and bounded by the existing public
  context limit.
- Legacy `recall()` remains only for frozen compatibility/benchmark paths and
  is documented as non-authoritative for current task context.

## Task 1: Freeze and promote Phase 5 evidence

**Files:**

- Modify: `crates/baron-core/tests/phase1_target_red.rs`
- Add: `crates/baron-core/tests/phase6_trusted_memory.rs`
- Modify: `crates/baron-core/tests/context_compiler.rs`

- [x] Promote `target_context_critical_retrieval_excludes_candidate_memory`.
- [x] Promote `target_large_context_preserves_tier_zero_continuity_state`.
- [x] Promote `target_context_projects_one_complete_task_state_record`.
- [x] Add deterministic trust-matrix, temporal, cross-project, global-memory,
  repeated-compilation, and Tier-0/Tier-1 pressure assertions.
- [x] Keep all Phase 5 prepare tests normal and rerun them unchanged.

## Task 2: Establish the trusted retrieval boundary

**Files:**

- Modify: `crates/baron-core/src/firewall.rs`
- Modify: `crates/baron-core/src/knowledge.rs`
- Modify: `crates/baron-core/src/intelligence.rs`
- Modify: `crates/baron-core/src/intelligence41.rs` only where the existing
  current resume path needs the named boundary
- Modify: `crates/baron-cli/src/main.rs`

- [x] Add `TrustedRecallPolicy::Current` (or an equivalent named API) that
  delegates to the existing semantic/trust implementation.
- [x] Centralize current eligibility and temporal filtering so context and
  current recall do not duplicate filters.
- [x] Preserve current-project preference, approved global handling, global
  candidate exclusion, cross-project blocking, trust-state exclusion, and
  temporal ledger currentness.
- [x] Route the context memory brief, current resume memory, and direct modern
  `recall` command through the named current authority.
- [x] Leave legacy `recall()` available only for explicit frozen compatibility
  or benchmark paths and document that boundary in code/tests.

## Task 3: Add the canonical Task State projection

**Files:**

- Add: `crates/baron-core/src/task_state.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/context.rs`

- [x] Define a bounded serializable/renderable Task State projection containing
  project/task identity, intent, constraints, non-goals, plan/work state,
  continuity, recovery, blockers, unknowns, affected files, proof/trace, route,
  and safe next action.
- [x] Read existing intent, plan, continuity, recovery, proof, trace, control
  plane, work-shape, and trusted-memory authorities without writing them.
- [x] Detect material intent/plan conflicts and stale recovery using existing
  authority/revision evidence; expose conflicts instead of choosing silently.
- [x] Keep missing values explicitly unknown and apply per-field bounds.
- [x] Render one `## Task State` section with stable field labels and source
  pointers; do not persist a Task State file.

## Task 4: Replace flat context truncation with tiered budgeting

**Files:**

- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-core/tests/context_compiler.rs`

- [x] Preserve existing context sections and user-visible headings where
  possible, but classify them into Tier 0–3.
- [x] Render Tier 0 first from the Task State projection with protected
  per-field bounds for identity, intent, constraints, non-goals, plan,
  recovery/blockers, route, proof gates, and next action.
- [x] Allocate deterministic budgets to Tier 1 trusted evidence, Tier 2
  project intelligence, and Tier 3 diagnostics/optional context.
- [x] Drop or summarize lower tiers before cutting higher tiers.
- [x] Emit concise truncation metadata distinguishing missing from bounded
  information.
- [x] Remove the old final-string tail truncation as the primary allocator;
  retain only bounded field/section helpers.

## Task 5: Integrate and verify

- [x] Run focused trusted-memory, context, Task State, continuity, intent,
  firewall, semantic, Phase 4 Core, and Phase 5 prepare tests.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets --no-fail-fast`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `git diff --check` and validate `docs/BARON_STATUS.json`.
- [x] Classify only the known Windows `Microsoft.PowerShell.Archive`
  installer failures as environment limitations.
- [x] Update the Phase 6 status, build checkpoint, and this plan with exact
  evidence; stop before Phase 7.

## Phase 6 target inventory after implementation

- `target_context_critical_retrieval_excludes_candidate_memory`
- `target_large_context_preserves_tier_zero_continuity_state`
- `target_context_projects_one_complete_task_state_record`

All three promoted targets are green. The remaining ignored targets are
Phase 7/11 work and remain intentionally out of scope.

Phase 7 database/profile, Phase 8/9 bridge, Phase 10 adapter, and Phase 11
retirement tests remain ignored and were not weakened or promoted here.

## Phase 6 verification evidence

- Promoted Phase 6 targets: `3/3` green.
- Dedicated Phase 6 trust/context tests: `4/4` green.
- Existing context compiler: `18/18` green; Phase 1 current: `8/8` green;
  Phase 5 Core prepare: `4/4` green; Phase 5 CLI prepare: `5/5` green.
- Workspace sweep: all targets green except the two known Windows lifecycle
  installer tests that require the unavailable `Microsoft.PowerShell.Archive`
  module (`Compress-Archive`).
- Workspace Clippy, formatter, diff check, and status JSON validation pass.
- No persisted Task State schema or PreparePacketV1 field changed.
