# Baron 3.7 Quality And Trust Implementation Plan

> Approved by the owner on 2026-08-12. Execute in order; keep the source
> version at `3.6.0` until the release phase.

## Task 0 - Baseline And Checkpoint

- [x] Confirm clean baseline and current public release `v3.6.0`.
- [x] Record the active design, plan, status JSON, build log, and continuity
  checkpoint before source edits.
- [ ] Keep `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, and
  `notes/build-log/CURRENT.md` synchronized after every meaningful batch.

## Task 1 - Phase 46 Work Shape

- [ ] Add an explicit work-shape model for authority, durability, judgment,
  risk, proof, and lifecycle depth.
- [ ] Keep read-only requests write-free and ambiguous material choices stopped.
- [ ] Route bounded reversible work through focused ephemeral proof while
  retaining the full path for risky, coordinated, multi-session, or uncertain
  work.
- [ ] Add English/Vietnamese fixtures and parity tests for all adapters.

## Task 2 - Phase 47 Trusted Receipts

- [ ] Add the versioned receipt schema, canonical source/project binding,
  bounded output/artifact digests, outcome states, and atomic storage.
- [ ] Add a Baron-owned command runner that emits receipts from actual process
  execution; reject handwritten/agent-authored receipt data for proof.
- [ ] Keep Vault Markdown durable and machine receipt state rebuildable.
- [ ] Add stale, tampered, cross-project, path-escape, secret-redaction, and
  3.6 migration tests.

## Task 3 - Phase 48 Completion Integrity

- [ ] Replace keyword-only proof and gate checks with trusted receipt matching.
- [ ] Bind quality gates and reviewer closure to current source and receipt IDs.
- [ ] Make failed/skipped/degraded/missing/stale evidence visible and keep Trace
  failure a hard stop.
- [ ] Add hand-edited Markdown and old-source false-green regression tests.

## Task 4 - Phase 49 Harness Experiments

- [ ] Extend intervention/proposal/outcome data with authority, baseline,
  hypothesis, owner, validation, fresh rerun, and keep/revise/remove/pending
  decision fields.
- [ ] Require explicit approval before applying an intervention.
- [ ] Require a comparable fresh-agent rerun and preserve experiment history.
- [ ] Add isolation and unused-intervention tests.

## Task 5 - Phase 50 Application Runbook

- [ ] Add project-owned `docs/baron/operations/` contract and bounded parser.
- [ ] Route only matching runtime tasks and preserve unknowns.
- [ ] Add safe operation evidence and ownership/cleanup rules without inventing
  application facts or adding a competing operations skill.

## Task 6 - Phase 51 Certification

- [ ] Integrate selected upstream harness lessons into existing owners.
- [ ] Verify three-adapter parity, user-content preservation, project/Vault
  isolation, migration, interruption, corruption, redaction, and scale.
- [ ] Run complete local certification while the source remains `3.6.0`.

## Task 7 - Phase 52 Public Release

- [ ] Bump source and synchronized metadata to `3.7.0` only after Tasks 1-6.
- [ ] Update root README with Windows/Linux/macOS `releases/latest` install,
  exact version checks, Windows reinstall/Vault/project recovery, and update
  guidance.
- [ ] Run formatting, tests, Clippy, locked release build, release manifest,
  installer, documentation, and stale-version checks.
- [ ] Push the exact certified source to the release ref, run immutable native
  GitHub promotion, and verify tag, Release, archives, checksums, manifest, and
  installers.
- [ ] Install from public `releases/latest` in a fresh Windows directory and
  run version, setup, init, context, receipt, and update-preservation smoke.
- [ ] Update final status/build-log/README/certification with exact SHA, run ID,
  assets, and public smoke; push the final documentation and verify a clean
  synchronized default branch.

## Stop Conditions

- A failed gate, trace, receipt, migration, adapter, or release job pauses the
  program at the failed task and records recovery evidence.
- No phase may claim completion from prose, stale Markdown, local-only tests,
  a pushed commit, a started workflow, or a tag alone.
- No default-branch README may claim a public 3.7 download while
  `releases/latest` installs another version.
