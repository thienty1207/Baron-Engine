# Baron 3.4 To 3.6 Program Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver Baron 3.4 safe updates, Baron 3.5 skill intelligence, and Baron 3.6 optional code-graph intelligence in a strict order that keeps the engine coherent and recoverable.

**Architecture:** Treat each release as an independent proof boundary. Complete and certify update safety before modifying skill intelligence, then certify skill ownership before adding the optional Graphify provider. `docs/BARON_STATUS.md` and JSON remain the progress dashboard; release-specific plans contain executable details.

**Tech Stack:** Rust 2021 workspace, local Markdown runtime assets, Baron adapters, Product Harness, Context Compiler, capability registry, optional Graphify CLI, GitHub release workflow.

---

## Program Order

| Phases | Release | Outcome | Authoritative plan |
| --- | --- | --- | --- |
| 35-38 | `3.4.0` | One runtime asset source and safe recoverable updates | `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md` |
| 39-41 | `3.5.0` | Hallmark/Matt techniques distilled into existing Baron owners | `docs/superpowers/plans/2026-07-24-phase-39-41-baron-3-5-skill-intelligence.md` |
| 42-45 | `3.6.0` | Optional local project-scoped code graph with Survey fallback | `docs/superpowers/plans/2026-07-24-phase-42-45-baron-3-6-code-graph.md` |

## Task 1: Complete Baron 3.4.0

- [ ] Phase 35 begins by deleting stale `blueprints/core/` and proving
  `assets/core/` is the only runtime source.
- [ ] Phase 35 records and plans managed three-way updates without writing live
  targets.
- [ ] Phase 36 verifies immutable native update candidates.
- [ ] Phase 37 activates project/runtime changes transactionally with conflict,
  abort, continuation, rollback, and recovery behavior.
- [ ] Phase 38 proves AI local repair cannot silently authorize a remote release.
- [ ] Run the full 3.4 certification before changing any source version.
- [ ] Bump to `3.4.0`, commit, and push only after certification.
- [ ] Do not start Phase 39 while any 3.4 conflict/recovery test is failing.

## Task 2: Complete Baron 3.5.0

- [ ] Audit the pinned Hallmark and Matt source revisions recorded in the design.
- [ ] Add Hallmark-derived brief fingerprint, anti-template, and responsive/state
  proof references under the existing `frontend-design` skill.
- [ ] Add Matt-derived deep-module guidance under the existing
  `api-and-interface-design` skill.
- [ ] Add Product Harness domain language without inventing or sharing terms
  across projects.
- [ ] Reject all overlapping workflow, TDD, debugging, review, planning,
  grilling, handoff, setup, deprecated, in-progress, and personal source skills.
- [ ] Prove three-adapter parity, narrow routing, custom preservation, bounded
  context, and behavior pressure tests.
- [ ] Bump to `3.5.0`, commit, and push only after certification.
- [ ] Do not start Phase 42 while duplicate skill ownership remains.

## Task 3: Complete Baron 3.6.0

- [ ] Add a provider-neutral, project-isolated, rebuildable code-graph contract.
- [ ] Add exact-version Graphify `0.9.25` compatibility through local
  `--code-only` extraction and bounded JSON queries.
- [ ] Never invoke Graphify installers, hooks, global graph, work memory, or
  semantic backend behavior.
- [ ] Keep all graph state under the current repository `.baron/cache`.
- [ ] Load only task-relevant graph hits and label inferred relationships.
- [ ] Require current source verification before graph guidance supports proof,
  decisions, or durable memory.
- [ ] Fall back to Baron Survey for missing, stale, incompatible, failed,
  malformed, timed-out, or oversized providers.
- [ ] Prove same-name project isolation, old/large repository behavior, and no
  instruction/hook mutation.
- [ ] Bump to `3.6.0`, commit, and push only after certification.

## Task 4: Maintain Interruption-Safe State

After every phase:

- [ ] Update `docs/BARON_STATUS.md`.
- [ ] Update `docs/BARON_STATUS.json`.
- [ ] Update `notes/build-log/CURRENT.md`.
- [ ] Append a dated phase build log with exact commands and results.
- [ ] Update `docs/superpowers/plans/CURRENT.md`.
- [ ] Commit the phase independently.
- [ ] Leave the next phase as `planned` with one exact next action.

If a session stops unexpectedly, resume from:

1. `docs/BARON_STATUS.md`
2. `notes/build-log/CURRENT.md`
3. this program plan
4. the current release-specific plan

## Task 5: Enforce Release Boundaries

- [ ] Keep source at `3.3.0` until 3.4 Phases 35-37 are green.
- [ ] Keep source at `3.4.0` until 3.5 Phases 39-40 are green.
- [ ] Keep source at `3.5.0` until 3.6 Phases 42-44 are green.
- [ ] Never mark a release complete from planned tests or source presence.
- [ ] Require fresh full-suite evidence for every release.
- [ ] Keep tag and GitHub Release promotion explicit; source push does not imply
  binary publication.

## Plan Self-Review

- The program has eleven remaining phases, 35 through 45.
- Existing Phase 35-38 safety work remains first.
- Blueprint cleanup occurs before the managed baseline is recorded.
- 3.5 adds no new bundled workflow or quality agent.
- 3.6 adds no mandatory external runtime.
- User-facing commands remain small.
- Every future session has a single resume point and exact authoritative plan.
