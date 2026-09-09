# Remove Project-Global Adapter Authority Phase 10 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove correctness dependence on the serialized project-global `active_adapter` while retaining that field for legacy configuration compatibility. Codex and Claude operations will carry explicit runtime identity, and adapter-neutral Baron Core state will remain shared across both integrations.

**Architecture:** Keep `AdapterKind` and `ProjectConfig.active_adapter` as compatibility data. Add a strict runtime operation identity containing only Codex or Claude plus optional session/request correlation. Route, context, capability/runtime checks, proof-bearing capability evidence, continuity attribution, journal events, and bridge preparation will use explicit identity or fail closed. Shared task, memory, plan, trace, and Core semantics remain adapter-neutral. UI/switch helpers may continue to expose the legacy field but cannot authorize runtime work.

**Tech Stack:** Rust workspace, Cargo tests, serde/TOML/JSON fixtures, clap CLI, deterministic filesystem fixtures, existing Baron Core and adapter test helpers.

**Spec:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md` and the accepted Phase 10 request supplied with this task.

## Global Constraints

- Phase 10 only. Do not start Phase 11 retired-adapter cleanup, Phase 12 hook/idempotency work, Phase 13 Autopilot UX, Phase 14 release hardening, Phase 15 documentation rewrite, or Phase 16 final verification.
- Preserve the serialized `active_adapter` field and existing legacy switch/config parsing; do not bump the persisted schema.
- Do not add Generic or the retired adapter to the new runtime authority model. Historical parsing remains a later migration concern.
- Preserve user-owned adapter files and the existing PreparePacket v1 shape.
- Write target tests first, observe their current failures, then add the smallest behavior-neutral or required implementation changes.
- Use deterministic concurrency/barrier fixtures; do not rely on sleeps or scheduler timing.
- Update `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, and `notes/build-log/CURRENT.md` at phase start and completion.

---

## Task 1: Freeze the active-adapter audit and add Phase 10 target tests

- [x] Record every current `active_adapter` use in a table or test comments classified as compatibility, diagnostics/UI, or runtime correctness.
- [x] Add `crates/baron-core/tests/phase10_adapter_authority.rs` with fixtures for explicit Codex and Claude prepare under the opposite serialized active value, adapter-neutral route parity, missing identity failure, capability-state mismatch, proof evidence attribution, shared continuity/task state, and deterministic cross-adapter resume.
- [x] Add `crates/baron-cli/tests/phase10_adapter_authority_cli.rs` covering reciprocal `control-plane prepare`, explicit capability/runtime identity, fail-closed omission, explicit continuity attribution, and the non-authoritative switch command.
- [x] Add deterministic concurrent-operation fixtures that interleave two explicit identities without allowing one to inherit the other project's global adapter.
- [x] Run `cargo test -p baron-core --test phase10_adapter_authority -- --nocapture` and the CLI target before implementation. Document each compile/behavior failure as expected target RED; classify any baseline regression separately.

## Task 2: Introduce strict runtime operation identity

- [x] Add a non-persisted `OperationContext` (or equivalent) with strict `SupportedAdapter::{Codex, Claude}`, optional `session_id`, and optional `request_id`.
- [x] Provide explicit conversion/parsing from the prepare/CLI adapter input and fail closed for missing or unsupported runtime identity. Keep `AdapterKind` conversion for compatibility at the boundary only.
- [x] Export the type from `baron-core` without changing `PreparePacketV1`, managed-state schemas, project config schema, or Vault source-of-truth formats.
- [x] Add unit tests proving Codex/Claude are the only runtime values and Generic/retired historical values cannot become new operation authority.

## Task 3: Thread explicit identity through Core correctness paths

- [x] Add an explicit-operation route seam while keeping `route_task` adapter-neutral; capability availability comparisons must use the operation adapter when supplied and never load `active_adapter` to decide correctness.
- [x] Add an explicit-operation context compiler seam that maps only the strict runtime adapter to the existing edge projection; preserve shared semantic route/memory/plan/Core content.
- [x] Update `prepare` to build one `OperationContext`, use explicit route/context/runtime inputs, and keep the PreparePacket v1 wire contract byte/field compatible.
- [x] Replace proof capability-evidence dependence on `default_adapter` with an explicit operation-bearing API and fail closed when evidence requires identity. Keep adapter-neutral proof/trace summaries adapter-neutral.
- [x] Replace certification/runtime-policy correctness dependence on the project-global adapter with explicit or deterministic registered-adapter evaluation suitable for diagnostics, without introducing a new active authority.
- [x] Add regression tests for proof/trace equivalence across Codex and Claude and for explicit capability evidence under an opposite serialized active value.

## Task 4: Make journal, automation, continuity, and CLI attribution explicit

- [x] Add an identity-aware lifecycle/journal path that accepts adapter/session/request from the operation or hook payload. Preserve legacy hook entry points for compatibility and do not implement Phase 12 deduplication.
- [x] Add an identity-aware continuity checkpoint path; shared task state remains one project record while provenance is explicit when present.
- [x] Remove `hook_adapter_for_repo` and active-adapter inference from correctness-sensitive CLI paths. Context, prepare, capability/runtime, proof, and continuity commands must use explicit identity or fail closed; adapter-neutral plan/harness/trace work must not guess an adapter.
- [x] Keep adapter status/switch UI and serialized field behavior as diagnostics/convenience only, with output/tests stating that switching does not authorize runtime operations.
- [x] Add CLI tests for missing identity, opposite active value, session/request propagation, and preserved switch compatibility.

## Task 5: Prove shared state and cross-adapter behavior

- [x] Verify the same task/session/request produces shared adapter-neutral plan, memory, context semantics, trace, and continuity state when prepared from Codex and Claude.
- [x] Verify switching from Codex to Claude (and Claude to Codex) resumes the shared task from persisted evidence without requiring restatement and without importing the other adapter's host-local semantics.
- [x] Verify route decisions are identical for both explicit operation identities for the same project/profile/task/risk and remain independent of changed-file presence.
- [x] Verify concurrent explicit operations retain their own adapter/session/request attribution using deterministic barriers and stable output ordering.

## Task 6: Focused verification and phase records

- [x] Run focused Phase 10 tests, then the existing Core/prepare/trusted-memory/context/profile-routing/Codex-bridge/Claude-bridge/proof/trace/automation/session/continuity/update/lifecycle suites.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace --all-targets --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.
- [x] Classify known host-limited failures separately from unexpected failures; do not relabel regressions as target RED.
- [x] Update status/build records and this plan with exact test evidence, persisted-schema result, architectural discoveries, and Phase 11 readiness. Stop after the Phase 10 report.

## Completion Evidence (2026-09-08)

- Target evidence: `cargo test -p baron-core --test phase10_adapter_authority`
  passed `14/14`; `cargo test -p baron-cli --test
  phase10_adapter_authority_cli` passed `8/8`.
- Regression evidence: all Core tests passed; the focused prepare, trusted
  memory, context, profile routing, Codex bridge, Claude bridge, proof/trace,
  automation, session, continuity, update, and lifecycle targets passed. The
  workspace sweep passed every product target except the two existing Windows
  installer tests that require the unavailable `Microsoft.PowerShell.Archive`
  module.
- Quality evidence: `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`,
  and status JSON parsing passed.
- Schema result: `PreparePacketV1`, managed-state v2, project config schema,
  and Vault source-of-truth formats are unchanged. The serialized
  `active_adapter` field remains compatibility/UI data only.
- Architectural discovery: neutral lifecycle events need an explicit
  adapter-neutral journal value, while Codex/Claude events carry operation
  session/request provenance. Registered-adapter diagnostics can evaluate all
  supported adapters without selecting a correctness authority.
- Phase 11 remains deferred; this checkpoint stops after Phase 10.
