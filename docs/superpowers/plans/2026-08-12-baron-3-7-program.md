# Baron 3.7 Quality And Trust Implementation Plan

> Approved by the owner on 2026-08-12. Execute in order; keep the source
> source version is `3.7.0`; the public release phase is complete.

## Task 0 - Baseline And Checkpoint

- [x] Confirm clean baseline and current public release `v3.6.0`.
- [x] Record the active design, plan, status JSON, build log, and continuity
  checkpoint before source edits.
- [x] Keep `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, and
  `notes/build-log/CURRENT.md` synchronized after every meaningful batch.

## Task 1 - Phase 46 Work Shape

- [x] Add an explicit work-shape model for authority, durability, judgment,
  risk, proof, and lifecycle depth.
- [x] Keep read-only requests write-free and ambiguous material choices stopped.
- [x] Route bounded reversible work through focused ephemeral proof while
  retaining the full path for risky, coordinated, multi-session, or uncertain
  work.
- [x] Add English/Vietnamese fixtures and parity tests for all adapters.

## Task 2 - Phase 47 Trusted Receipts

- [x] Add the versioned receipt schema, canonical source/project binding,
  bounded output/artifact digests, outcome states, and atomic storage.
- [x] Add a Baron-owned command runner that emits receipts from actual process
  execution; reject handwritten/agent-authored receipt data for proof.
- [x] Keep Vault Markdown durable and machine receipt state rebuildable.
- [x] Add stale, tampered, cross-project, path-escape, secret-redaction, and
  3.6 migration tests.

## Task 3 - Phase 48 Completion Integrity

- [x] Replace keyword-only proof and gate checks with trusted receipt matching.
- [x] Bind quality gates and reviewer closure to current source and receipt IDs.
- [x] Make failed/skipped/degraded/missing/stale evidence visible and keep Trace
  failure a hard stop.
- [x] Add hand-edited Markdown and old-source false-green regression tests.

## Task 4 - Phase 49 Harness Experiments

- [x] Extend intervention/proposal/outcome data with authority, baseline,
  hypothesis, owner, validation, fresh rerun, and keep/revise/remove/pending
  decision fields.
- [x] Require explicit approval before applying an intervention.
- [x] Require a comparable fresh-agent rerun and preserve experiment history.
- [x] Add isolation and unused-intervention tests.

## Task 5 - Phase 50 Application Runbook

- [x] Add project-owned `docs/baron/operations/` contract and bounded parser.
- [x] Route only matching runtime tasks and preserve unknowns.
- [x] Add safe operation evidence and ownership/cleanup rules without inventing
  application facts or adding a competing operations skill.

## Task 6 - Phase 51 Certification

- [x] Integrate selected upstream harness lessons into existing owners.
- [x] Verify three-adapter parity, user-content preservation, project/Vault
  isolation, migration, interruption, corruption, redaction, and scale.
- [x] Run complete local certification before the `3.7.0` release promotion.

## Task 7 - Phase 52 Public Release

- [x] Bump source and synchronized metadata to `3.7.0` only after Tasks 1-6.
- [x] Update root README with Windows/Linux/macOS `releases/latest` install,
  exact version checks, Windows reinstall/Vault/project recovery, and update
  guidance.
- [x] Run formatting, tests, Clippy, locked release build, release manifest,
  installer, documentation, and stale-version checks.
- [x] Push the exact certified source to the release ref, run immutable native
  GitHub promotion, and verify tag, Release, archives, checksums, manifest, and
  installers.
- [x] Install from public `releases/latest` in a fresh Windows directory and
  run version, setup, init, context, receipt, and update-preservation smoke.
- [x] Update final status/build-log/README/certification with exact SHA, run ID,
  assets, and public smoke; push the final documentation and verify a clean
  synchronized default branch.

## Stop Conditions

- A failed gate, trace, receipt, migration, adapter, or release job pauses the
  program at the failed task and records recovery evidence.
- No phase may claim completion from prose, stale Markdown, local-only tests,
  a pushed commit, a started workflow, or a tag alone.
- No default-branch README may claim a public 3.7 download while
  `releases/latest` installs another version.
