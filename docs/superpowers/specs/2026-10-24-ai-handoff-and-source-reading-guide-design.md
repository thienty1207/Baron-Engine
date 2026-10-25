# AI Handoff and Source Reading Guide — Design

## Intent

Make it straightforward for a new AI agent to understand Baron Engine's
verified current state, resume work safely, and inspect the complete source
without treating old plans or status snapshots as current instructions.

## Goals

- Keep `docs/superpowers/plans/CURRENT.md` as the single canonical project
  handoff. It will summarize the current source/release boundary, completed
  and pending work, local verification, known limitations, and next safe
  action.
- Add `docs/architecture/SOURCE_READING_GUIDE.md` with a repeatable inventory
  and reading procedure for Core, CLI, adapters, canonical Core assets,
  integration tests, and CI workflows. The guide must require explicit
  coverage accounting and forbid claiming a complete scan unless every
  inventoried source file was read.
- Update `AGENTS.md`, `README.md`, `docs/BARON_STATUS.md`,
  `docs/superpowers/README.md`, and `notes/build-log/CURRENT.md` so they point
  to the canonical handoff and preserve their distinct roles.
- Remove a documentation file only when a repository-wide comparison proves
  it is an actual duplicate with no unique evidence and its references can be
  safely updated. Age, completion, or similar subject matter alone is not
  sufficient.

## Non-goals

- Do not create `docs/current/current.md` or another competing handoff.
- Do not delete unique historical specifications, plans, audits, assessments,
  or build evidence just because they are old or completed.
- Do not change product source, package versions, published release records,
  release targets, or artifacts; current status and navigation metadata may be
  updated to describe this docs task and the 5.0.1 candidate branch.
- Do not tag, publish, merge, release, or rewrite history. The owner explicitly
  authorized committing and pushing the candidate branch after this docs work.

## Approaches

1. Reuse `docs/superpowers/plans/CURRENT.md` and clarify the roles of the
   existing status/build-log documents. This follows the existing repository
   index and avoids a second current file. **Selected by the user.**
2. Add a separate `docs/current/current.md` and turn the existing current files
   into pointers. This would create a new handoff location and was declined.

## File responsibilities

- `docs/superpowers/plans/CURRENT.md`: canonical AI handoff, concise current
  facts and navigation; historical detail points to its dated source records.
- `docs/architecture/SOURCE_READING_GUIDE.md`: source-of-truth boundaries,
  dynamic file inventory commands, layered reading order, subsystem/test map,
  and coverage ledger template.
- `docs/BARON_STATUS.md` and `.json`: durable release/phase dashboard and
  machine-readable release state; link to the canonical handoff instead of
  repeating its full narrative.
- `notes/build-log/CURRENT.md`: current build/test evidence and environment
  caveats; link to the canonical handoff for project state.
- `AGENTS.md`, `README.md`, and `docs/superpowers/README.md`: route new agents
  and maintainers to the handoff and source-reading guide.

## Validation

- Confirm every proposed deletion is byte-identical or demonstrably redundant,
  has no unique evidence, and has no stale inbound link.
- Check repository-relative Markdown links, parse `docs/BARON_STATUS.json`,
  run the focused public documentation contract tests, and run `git diff
  --check`.
- Review the final current handoff against the actual branch, source version,
  release status, test evidence, and safe next action.
