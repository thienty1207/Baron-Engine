# Current Build Note

Date: 2026-07-24
Target: Baron 3.4.0
Program target: Baron 3.6.0

## Current Phase

Phase 39 - Hallmark Frontend Distillation (`planned`).

## Verified Checkpoint Before Phase 35

The bundled Superpowers workflow core has been refreshed to the pinned upstream
`v6.2.0` skill tree without changing Baron's core ownership:

- exact upstream commit:
  `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`
- upstream subtree: 50 files with no missing or extra file; one recorded Baron
  patch removes the visual companion's remote branding/telemetry request
- Baron-owned root routing and adapter integration remain local
- Codex, Claude, and generic adapter contract test passed
- provenance, MIT license, and offline runtime ownership are recorded
- Unix installs preserve executable mode for embedded shebang scripts
- a short Baron-owned entry contract protects the SDD review breaker and
  behavior-test rules from long-context summarization drift
- extension boundaries for code graphs and optional skill sources are accepted
  in `docs/decisions/0002-extension-ownership-and-code-graph.md`

This checkpoint starts Phase 35 only. It does not bump the Baron version, add
Graphify, or change the 3.4 phase order.

## Blueprint Audit

The two apparent core trees are not equal maintained sources:

- `assets/core/` has 145 runtime asset files and is embedded by
  `baron-adapters`.
- `blueprints/core/` has 7 stale historical files.
- Six paths overlap and all six have different content.
- No Rust crate, installer, manifest, test, or workflow reads
  `blueprints/core/`.

Phase 35 completed the source-of-truth contract, deletion of
`blueprints/core/`, adapter parity proof, managed baseline, conservative
three-way planner, malformed-marker refusal, and no-write CLI preview. The
RED test failed on the stale directory; the same focused test passed after
removal. A fresh install now records only repository-relative Baron-owned
content under `.baron/managed-state/`; custom assets and project state remain
outside its write set.

## What Is Being Built

- Phase 35 has removed stale blueprints, proved `assets/core/` is the only
  runtime source, and recorded the managed baseline plus safe read-only
  three-way preview.
- Phase 36 resolved and verified the exact native release candidate before project activation.
- Phase 37 applies project/runtime changes transactionally, stages conflicts, and recovers interrupted updates; its focused transaction/recovery suites now pass.
- Phase 38 separates human update authority from AI local repair and certifies Baron 3.4; the full certification gate has passed and `3.4.0` is the current stable source baseline.
- Phases 39-41 produce Baron 3.5 by distilling selected Hallmark and Matt Pocock
  techniques into existing Baron owners, with no second workflow or frontend
  skill.
- Phases 42-45 produce Baron 3.6 with an optional project-scoped Graphify
  code-only provider, bounded context, source verification, strict isolation,
  and Survey fallback.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`.
3. Read
   `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`.
4. Execute
   `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`.
5. Begin Phase 39 with RED frontend asset/routing tests. Do not add Hallmark as
   a new skill, runtime dependency, or second frontend owner.
6. Update status Markdown/JSON, `CURRENT.md`, and a dated build log after every
   verified phase checkpoint.
7. Do not begin the 3.5 or 3.6 plans before the preceding release is certified.

## Verified Baseline

- Isolated branch: `codex/baron-3-4-to-3-6`.
- Planning baseline commit: `b6e619b`.
- Baseline `cargo test --workspace --all-targets`: passed on 2026-07-23.
- Source and stable-source version are now `3.4.0` after Phase 38’s full gate.
- Baron 3.4 has Phases 35-38 evidence and source certification.
- Baron 3.5 and 3.6 currently have design/plan evidence only.
- The program has 7 planned phases remaining: 39 through 45.
- Superpowers `v6.2.0` adapter contract: passed on 2026-07-24.
- Full post-refresh workspace tests, Clippy, release build, adapter smoke, SDD
  semantic smoke, and visual-server behavior tests: passed on 2026-07-24.

## Non-Negotiables

- Superpowers remains workflow core.
- Core agents remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains source of truth.
- Normal users keep the simple install/setup/init/update flow.
- AI never silently downloads or activates a release.
- Conflicting managed edits never overwrite live project content.
- No release tag or GitHub Release exists before its source and artifacts pass proof.
- `assets/core/` is the only runtime source for bundled skills and agents.
- `blueprints/core/` must be removed before any managed baseline is trusted.
- Optional providers and skills cannot become a second workflow, memory, or
  instruction owner.
- Baron 3.5 may deepen existing owners but must not create Hallmark or Matt
  workflow skills.
- Baron 3.6 may use Graphify only as a local optional code map; it cannot own
  hooks, instructions, Vault memory, global context, or workflow state.
