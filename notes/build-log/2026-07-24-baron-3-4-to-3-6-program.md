# Baron 3.4 To 3.6 Program Planning Log

Date: 2026-07-24
Checkpoint type: design and implementation planning only
Stable source: `3.3.0`
Current release target: `3.4.0`
Program target: `3.6.0`

## What Was Decided

Baron will advance in three independently certified releases:

1. `3.4.0`, Phases 35-38: remove the stale core blueprint tree and deliver safe,
   recoverable updates.
2. `3.5.0`, Phases 39-41: deepen existing Baron skill owners with selected
   Hallmark and Matt Pocock techniques.
3. `3.6.0`, Phases 42-45: add an optional local, project-scoped code-map
   provider with source verification and Survey fallback.

The order is mandatory. A later release cannot start while the preceding
release has failed or missing certification evidence.

## Blueprint Evidence

The repository audit found:

- `assets/core/`: 145 files and the only tree embedded by
  `crates/baron-adapters/src/install.rs`.
- `blueprints/core/`: 7 historical files.
- Six overlapping files have different content.
- No Rust source, installer, manifest, test, or workflow reads
  `blueprints/core/`.

Decision: Phase 35 starts with a RED/GREEN runtime-source contract and deletion
of `blueprints/core/`. The managed baseline is recorded only after Codex,
Claude, and generic adapters prove parity from `assets/core/`.

## External Source Boundaries

### Superpowers

- Source: `obra/superpowers`
- Pinned release: `v6.2.0`
- Pinned commit: `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`
- Status: already refreshed, locally vendored, and verified.
- Ownership: remains Baron's sole workflow core.

### Hallmark

- Source: `nutlope/hallmark`
- Audited commit: `aeb42fb354ff4efa36ab475773a082315a3af2ce`
- Planned use: brief fingerprint, anti-template gates, responsive/state proof.
- Rejected use: a new Hallmark skill, command router, installer, or live runtime
  dependency.

### Matt Pocock Skills

- Source: `mattpocock/skills`
- Audited commit: `ed37663cc5fbef691ddfecd080dff42f7e7e350d`
- Planned use: deep-module/interface techniques and project-owned domain
  language.
- Rejected use: duplicate planning, TDD, debugging, review, grilling, handoff,
  setup, deprecated, in-progress, or personal workflow skills.

### Graphify

- Source: `Graphify-Labs/graphify`
- Audited version: `0.9.25`
- Audited commit: `2fa6cd3d5548577f8c5f591b713f0bf80c1af183`
- Planned use: optional local `--code-only` extraction plus bounded JSON query.
- Rejected use: installer, hooks, instruction writing, global graph, work
  memory, semantic backends, or provider-owned query logs.
- Cache boundary: current repository `.baron/cache/code-graph/` only.
- Failure behavior: fall back to Baron Survey Engine.

## Phase Map

| Phase | Release | Scope | Status |
| --- | --- | --- | --- |
| 35 | 3.4.0 | Single runtime source and managed baseline | planned |
| 36 | 3.4.0 | Verified release candidate and binary handoff | planned |
| 37 | 3.4.0 | Conflict-safe activation and recovery | planned |
| 38 | 3.4.0 | Automation contract and certification | planned |
| 39 | 3.5.0 | Hallmark frontend distillation | planned |
| 40 | 3.5.0 | Deep modules and product domain language | planned |
| 41 | 3.5.0 | Routing, preservation, and certification | planned |
| 42 | 3.6.0 | Code-graph provider contract | planned |
| 43 | 3.6.0 | Graphify local code-only adapter | planned |
| 44 | 3.6.0 | Automatic bounded context and source verification | planned |
| 45 | 3.6.0 | Isolation, scale, and certification | planned |

## Authoritative Documents

- Program design:
  `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`
- Master program:
  `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`
- 3.4 plan:
  `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
- 3.5 plan:
  `docs/superpowers/plans/2026-07-24-phase-39-41-baron-3-5-skill-intelligence.md`
- 3.6 plan:
  `docs/superpowers/plans/2026-07-24-phase-42-45-baron-3-6-code-graph.md`

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `notes/build-log/CURRENT.md`.
3. Read the master program.
4. Open the 3.4 release plan.
5. Write the Phase 35 RED runtime-source test.
6. Delete `blueprints/core/` only after the RED assertion is demonstrated.
7. Record exact commands and proof before changing phase status.

## Not Performed At This Checkpoint

- No runtime source was modified.
- `blueprints/core/` was not deleted yet.
- No Hallmark or Matt-derived runtime content was added.
- Graphify was not installed or integrated.
- The source version remains `3.3.0`.
- No release tag or GitHub Release was created.
- No push was performed.
