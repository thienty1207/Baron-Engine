# Profile-Aware Routing, Database, and Mobile Engineering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Extend Baron's existing control-plane router so configured platform profiles, task/repository evidence, risk, work shape, and current project state produce a deterministic, bounded, explainable route; add distinct Database and mobile application engineering domains while preserving Data, reverse-analysis skills, Core ownership, and PreparePacket v1.

**Architecture:** Keep `baron_core::control_plane::route_task` as the only routing authority. Add a small metadata-backed candidate model and a route-input projection assembled from existing project config, survey, work shape, trusted Task State, continuity/recovery, capabilities, and known affected paths. Preserve the existing public route facade and have Task State, context, and Prepare consume the same report. Add `Database` to the existing `ProjectPlatform` enum without a schema bump after fixture proof, materialize two new skills only from `assets/core`, and let profile evidence influence scores rather than forcing every profile skill.

**Tech Stack:** Rust workspace, `serde`/TOML project config, existing Baron survey/platform/control-plane/context/prepare modules, embedded `include_dir!` Core assets, `cargo test`, `cargo fmt`, and `cargo clippy`.

**Spec:** Phase 7 request in `C:/Users/Ty/.codex/attachments/b417e138-ad94-4367-81ed-2d9f2e4ed6f7/pasted-text.txt`; authoritative architecture remains `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.

## Global Constraints

- Phase 7 only. Do not implement Codex/Claude bridge redesign, adapter retirement, hook/idempotency hardening, or release work.
- Preserve the existing control-plane as the single routing authority and Superpowers as the sole workflow owner.
- Keep Phase 8–12 target tests ignored/red; only Phase 7 targets and completed Phase 6 targets may be promoted.
- Do not add retired adapter/Generic product behavior, new adapter surfaces, or searchable retired-adapter literals.
- Preserve user-owned files, Core managed ownership, managed-state v2, memory schemas, and `PreparePacketV1`.
- Route with bounded metadata/evidence reads; do not load every skill body or perform an unbounded repository analysis.
- Keep equivalent inputs deterministic and classify portability for new fixtures.
- Write tests before implementation where practical, run focused tests after each task, and keep the workspace buildable.

## Task 1: Freeze Phase 6 regressions and add Phase 7 red evidence

- [x] Inspect `crates/baron-core/tests/phase1_target_red.rs` and existing Phase 6 suites; leave trusted-memory, Task State, and context-budget tests as normal tests when already green, while keeping cleanup/bridge tests ignored.
- [x] Add cross-platform routing fixtures/tests under `crates/baron-core/tests/` for Data vs Database, mobile development vs APK reverse analysis, API/security false positives, profile influence and task override, no-diff routing, bounded selection, dependencies/conflicts/exclusions, explanation categories, verification influence, and repeated deterministic routing.
- [x] Add CLI tests in `crates/baron-cli/tests/phase1_target_red.rs` or a focused Phase 7 test file for `--database` with Codex and Claude, preserving ignored later-phase targets.
- [x] Run the focused red tests and record why each target fails against the current router before changing production code.

## Task 2: Add the Database project profile without an unrelated schema migration

- [x] Add `ProjectPlatform::Database` to `crates/baron-core/src/config.rs`, update every exhaustive platform match, and retain `PROJECT_SCHEMA_VERSION` 4.
- [x] Add fixtures proving old schema-4 configs with existing platforms still load, new `platform = "database"` serializes/deserializes deterministically, and an old writer's known fields remain readable; document that older binaries may reject the new enum value rather than silently misroute it.
- [x] Add the `database` init flag to `InitArgs`, `parse_platform`, help/error text, and both Codex/Claude init paths in `crates/baron-cli/src/main.rs`; keep profile-only initialization behavior unchanged.
- [x] Update architecture/context/platform profile match arms and current profile tests to include Database while preserving Data semantics.

## Task 3: Add canonical Database and mobile application Core assets

- [x] Add practical, contract-complete `assets/core/skills/database-engineering/SKILL.md` covering modeling, query plans/N+1, transactions/concurrency, migrations/backfills/rollback, application boundaries, integrity/recovery, evidence, and exclusions.
- [x] Add practical, contract-complete `assets/core/skills/mobile-application-engineering/SKILL.md` covering lifecycle, navigation/state, offline/network behavior, storage/permissions, background transitions, device/platform differences, deep links, battery/performance, realistic tests, and release validation.
- [x] Add only simple machine-usable routing metadata to these skill frontmatters (triggers, exclusions, profile affinities, dependencies/conflicts, evidence and verification hints) and extend the existing parser/index generation to consume it.
- [x] Update the existing bundled-skill allowlists/index text needed for asset audit and migration inventory; do not duplicate semantic skill bodies under `.codex` or `.claude`.
- [x] Run Core payload/materialization and asset contract tests to prove both skills travel `assets/core/**` → `.baron/core/**`.

## Task 4: Make platform intelligence operational

- [x] Add a distinct Database `PlatformProfile` with database concerns, failure modes, skill affinities, proportional agents, and verification/release expectations; update Mobile affinities to include `mobile-application-engineering` while retaining APK/binary skills for reverse/security evidence only.
- [x] Add task-lens/context guidance for Database and mobile application work, keeping generated platform documents bounded and user-owned text outside managed markers intact.
- [x] Ensure profile selection influences route scoring, agent proportionality, verification hints, decomposition/work-shape recommendations, context emphasis, and existing automation recommendation signals through the route report rather than a parallel workflow.

## Task 5: Extend the existing control-plane router with bounded evidence and explainability

- [x] Introduce a small public route-input/report extension in `crates/baron-core/src/control_plane.rs` that preserves `route_task(repo, task, risk)` as a compatibility facade and obtains configured profile, bounded survey/work shape/Task State/continuity/capability/affected-path evidence when available.
- [x] Implement deterministic metadata-first candidate scoring with explicit task evidence, repo evidence, configured profile affinity, risk/work-shape requirements, dependencies, exclusions, conflicts, and missing-evidence handling; task evidence must be able to override profile affinity.
- [x] Select the minimum sufficient skill/agent/gate set with hard bounds, always retaining Superpowers and proportional core quality gates; report dependency-selected, profile-influenced, risk-required, excluded, conflict-suppressed, skipped, and evidence-missing reasons in bounded form.
- [x] Separate API/interface detection from security detection so ordinary API work does not select security skill/agent solely because it contains “api”; retain high-risk and explicit security evidence requirements.
- [x] Add database-positive/negative, data, mobile, reverse-analysis, and deterministic repeated-routing coverage and assert no filesystem iteration or adapter ordering changes the result.

## Task 6: Integrate route evidence through Task State, context, and Prepare

- [x] Update `task_state.rs` to call the enhanced existing route authority and retain only bounded useful route explanation/selection in Tier 0.
- [x] Update context/platform rendering to show the selected profile and concise route rationale without dumping metadata or consuming protected Tier-0 budget.
- [x] Keep `crates/baron-core/src/prepare.rs` as a façade: it must consume the enhanced `RouteReport` through the existing call, with no duplicate profile-routing logic and no `PreparePacketV1` schema change.
- [x] Add prepare/context assertions showing Database/Mobile route and verification hints survive bounded packet/context rendering.

## Task 7: Update durable phase records and run verification

- [x] Record Phase 7 start/checkpoints and completion in `notes/build-log/CURRENT.md`; update `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, and this plan checklist with exact evidence and next action Phase 8 planning.
- [x] Run focused Core/CLI Phase 7 tests, promoted Phase 6 tests, canonical Core tests, prepare tests, control-plane/platform/release smoke tests, then `cargo fmt --all -- --check`, `cargo test --workspace --all-targets --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`, and `git diff --check`.
- [x] Classify the two known Windows `Microsoft.PowerShell.Archive` lifecycle failures separately from unexpected product failures and stop after the Phase 7 report.

## Verification Commands

```text
cargo test -p baron-core --test phase1_target_red -- --ignored
cargo test -p baron-core --test phase7_routing
cargo test -p baron-cli --test phase1_target_red -- --ignored
cargo test -p baron-cli --test phase7_profile_cli
cargo test -p baron-core --test context_compiler
cargo test -p baron-core --test prepare
cargo test -p baron-core --test control_plane
cargo test -p baron-core --test platform_intelligence
cargo test -p baron-adapters --test phase4_core
cargo fmt --all -- --check
cargo test --workspace --all-targets --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

## Self-Review Checklist

- [x] No second router or profile-specific workflow was introduced.
- [x] Data remains separate from Database, and mobile application work remains separate from APK reverse analysis.
- [x] Profile evidence informs but does not override task/repository evidence.
- [x] Routing is bounded, explainable, deterministic, and works before a diff exists.
- [x] Core assets are the only semantic source; adapter-local copies are not added.
- [x] PreparePacket v1, managed-state v2, memory schemas, and Phase 8–12 scope remain unchanged.
- [x] Final report uses every required Phase 7 heading and states unexpected failures explicitly.
