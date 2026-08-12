# Baron 3.8 Implementation Plan

Date: 2026-08-12  
Owner approval: granted for implementation, README update, GitHub push, and
public `v3.8.0` install verification. Status: candidate pending public workflow.

## Execution Order

1. Phase 53: benchmark the current resume contract and freeze bounded targets.
2. Phase 54: add layered memory labels, redaction, and durable/rebuildable
   boundaries.
3. Phase 55: strengthen recall with deterministic local hybrid ranking and
   explanations.
4. Phase 56: compile the bounded Resume Brief from current evidence and wire it
   into context and adapter startup.
5. Phases 57-59: add typed knowledge views, incremental Wiki citations, and the
   local CodeGraph fallback.
6. Phases 60-61: verify identity, secret, cache, concurrency, crash recovery,
   bounded context, and measured local cost.
7. Phases 62-63: repair `vibe-security-scan` contracts and add only the narrow
   Baron-owned reverse skills that pass provenance and safety review.
8. Phase 64: run integrated proof, update source/docs/status, publish exact
   `v3.8.0`, and verify `releases/latest` from a fresh Windows install. Complete.

## Required Evidence

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-targets --no-fail-fast`
- `cargo clippy --workspace --all-targets -- -D warnings`
- locked release build and exact `baron 3.8.0` output
- knowledge, adapter, routing, redaction, migration, cache-rebuild, and
  project-isolation tests
- release metadata/checksum verification and native GitHub workflow
- fresh public Windows install from `releases/latest/download/install.ps1`

## Stop Conditions

Stop and preserve a recovery packet if project identity can cross, secrets are
persisted, a derived cache cannot rebuild, a proof receipt is missing, a native
release target fails, or README and `releases/latest` disagree. A source commit
or workflow start alone is not release evidence; the final release record must
include the workflow run, tag, assets, checksums, and fresh installer smoke.

## Completion Record

The implementation is complete after the source version, README, status JSON,
status Markdown, build log, active design, command/memory architecture notes,
security assets, and release workflow were synchronized. The exact final SHA,
workflow run, asset inventory, and public installer output are recorded in
`docs/assessment/baron-3.8-certification.md` and linked from the status files.
