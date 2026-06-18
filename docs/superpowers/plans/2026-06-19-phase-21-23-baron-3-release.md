# Baron 3.0 Phase 21-23 Implementation Plan

Date: 2026-06-19

## Goal

Finish the remaining Baron 3.0 phases and release the source tree as `3.0.0`:

- Phase 21: Background Learning And Continuity Autopilot
- Phase 22: Capability Runtime And Safe Tool Backends
- Phase 23: Baron 3.0 Release Certification

## Non-Negotiables

- Keep Superpowers as Baron's workflow core.
- Keep `code-reviewer`, `security-auditor`, and `test-engineer` as the only mandatory quality agents.
- Keep Vault Markdown as source of truth.
- Keep automation observable; do not assume an AI ran required steps unless Baron recorded evidence.
- Keep uncertain learning as candidates until approved.
- Keep tool/provider availability separate from executed proof.
- Do not make one IDE, one agent app, or one tool backend mandatory.

## Implementation Checklist

- [x] Add RED tests for autopilot candidate learning, approval/rejection, resume signal, and automation observation.
- [x] Add RED tests for runtime backend policy, unsafe provider detection, and false proof claims.
- [x] Add RED tests for `3.0.0` version/certification metadata.
- [x] Implement `baron_core::autopilot`.
- [x] Implement runtime backend safety policy in `baron_core::capability`.
- [x] Add hidden CLI automation commands for autopilot and runtime checks.
- [x] Include bounded autopilot/runtime summaries in compiled context.
- [x] Update adapter startup guidance so AI automation runs these checks silently when meaningful work happens.
- [x] Bump workspace version and release metadata to `3.0.0`.
- [x] Synchronize README, status Markdown/JSON, command docs, and build logs.
- [x] Run full tests, Clippy, smoke flows, and `git diff --check`.
- [x] Commit and push to `origin/main`.

## Acceptance Proof

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- Temp smoke for setup, Codex/Claude/generic init, context, autopilot review/status, runtime check, certification, and release metadata.
- Static scans for stale `2.2.0` release identity outside historical notes/tests where intentionally retained.
