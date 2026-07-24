# Baron 3.4 Safe Update Roadmap Log

Date: 2026-07-23
Target: Baron `3.4.0`
Status: in progress

## Approved Scope

Baron will make the public `baron update` command responsible for a verified
runtime update and a conflict-safe refresh of Baron-managed project assets.
The implementation is split into four phases:

1. Phase 35 - Single Runtime Source And Managed Baseline
2. Phase 36 - Verified Release Candidate And Binary Handoff
3. Phase 37 - Conflict-Safe Activation And Recovery
4. Phase 38 - Automation Contract And Baron 3.4 Certification

## Decisions

- Keep the normal user flow to one update command.
- Keep remote release installation human-authorized.
- Use `baron automation reconcile` for AI local-only repair.
- Store a plain-file managed baseline under `.baron/managed-state/`.
- Use conservative three-way decisions; uncertain overlap becomes a staged
  conflict instead of a live overwrite.
- Let the verified candidate render new managed assets.
- Treat project activation and runtime activation as one recoverable
  transaction.
- Keep source, custom assets, plans, Harness records, and Vault memory outside
  the update write set.
- Preserve immutable exact-source release promotion.
- Require the existing installer once to move from a pre-3.4 binary to 3.4.

## Baseline Evidence

- Implementation worktree:
  `C:\Users\Ho Thien Ty\.config\superpowers\worktrees\Baron-Engine\baron-3-4-to-3-6`
- Baseline source commit: `d4ab320`
- Baseline `cargo test --workspace --all-targets`: passed on 2026-07-23.
- Current source version remains `3.3.0`.
- Phase 35 runtime-source RED test failed as expected because
  `blueprints/core/` existed.
- Removed exactly seven stale `blueprints/core/` files and empty child
  directories; preserved `blueprints/adapters/` because it is a separate
  historical documentation tree.
- The focused `assets_core_is_the_only_bundled_runtime_source` test passed after
  removal, including Codex, Claude, and generic adapter parity.
- No production update behavior or version metadata changed in this checkpoint.

## Phase 35 Runtime-Source Cleanup Verification

- RED: `cargo test -p baron-adapters --test adapter_lifecycle assets_core_is_the_only_bundled_runtime_source -- --exact` failed because `blueprints/core` existed.
- GREEN: the same command passed after deletion.
- `cargo test -p baron-core --test public_trust_docs`: passed after updating the
  expected Phase 35 status to `in_progress`.
- `rg -n 'blueprints/core|blueprints\\core' crates/baron-adapters/src crates/baron-core/src crates/baron-cli/src installers .github Cargo.toml`: no production runtime references.
- `git diff --check`: passed.

## Phase 35 Managed Baseline And Planner Verification

- RED: `cargo test -p baron-adapters --test update_planner` initially failed
  because the managed baseline and planner API did not exist.
- GREEN: `crates/baron-adapters/src/update.rs` now records only canonical
  repository-relative Baron-owned payloads under
  `.baron/managed-state/{manifest.json,base/}`. It rejects traversal,
  duplicate ownership, symlink/junction traversal, unsupported schemas,
  missing baseline copies, and malformed marker blocks.
- The planner makes conservative `BASE`/`LOCAL`/`UPSTREAM` decisions without
  writing the target: upstream-only changes apply, local-only changes remain,
  equal content deduplicates, managed blocks merge while preserving surrounding
  user text, and dual managed edits become conflicts.
- A fresh Codex installation is proved byte-compatible with the in-memory
  candidate renderer; no preview reads a target as its upstream source.
- `baron update --dry-run --installed` is hidden from the normal user flow. It
  displays a bounded safe preview and does not call platform/architecture
  writers, write project files, or create `.baron/update/` state.
- Superpowers provenance now hashes vendored text with LF normalization. This
  avoids false Windows worktree failures caused only by CRLF conversion; the
  pinned content and upstream revision remain unchanged.
- Focused GREEN evidence on 2026-07-24:
  - Independent-review closure added baseline-copy hash verification before a
    merge ancestor is trusted, symlink/junction refusal for managed-state,
    strict duplicate/inverted marker refusal, and preservation of the old
    manifest when a replacement fails before publishing a new one.
  - The planner now collects all registered adapter payloads in one pass,
    labels every action with its owning adapter, and includes new upstream
    assets instead of classifying them as user content.
  - `cargo test -p baron-adapters --all-targets`: passed (38 tests).
  - `cargo test -p baron-cli --test adapter_cli`: passed (10 tests), including
    a whole-repository byte snapshot around a multi-adapter dry run.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/specs/2026-07-23-baron-3-4-safe-self-update-design.md`.
3. Execute `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`.
4. Start with Phase 36 release candidate/resolver RED tests. Do not begin
   project activation or binary replacement before candidate verification is
   isolated and green.
5. Update this log and status Markdown/JSON after every phase checkpoint.

## Current Evidence State

- Phase 35: completed; runtime source, hash-verified and reparse-safe managed
  baseline, multi-adapter planner, malformed marker refusal, replacement
  preservation, and read-only dry-run preview are verified
- Phase 36: in progress; candidate source and release metadata code are being
  tested before any runtime activation work
- Phase 37: planned, no implementation evidence
- Phase 38: planned, no implementation evidence

## Non-Negotiables

- Superpowers remains workflow core.
- Core quality agents remain unchanged.
- Vault Markdown remains memory source of truth.
- AI does not silently install releases.
- No ambiguous managed-file overwrite.
- No completion claim without fresh tests and lifecycle smoke.
