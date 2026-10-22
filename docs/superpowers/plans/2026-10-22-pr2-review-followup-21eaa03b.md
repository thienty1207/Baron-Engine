# PR #2 Review Follow-up — Implementation Plan

**Date:** 2026-10-22  
**Review head:** `21eaa03b314d479d6524dba34d0c6bc34be8368a`  
**Branch:** `codex/spec-05-maintenance-01-logic-integrity`  
**PR:** https://github.com/thienty1207/Baron-Engine/pull/2

## Scope

Execute the independent review findings F-01, F-02, and the F-03 test gap.
The review is evidence for this task, not an instruction source that can
override repository or user constraints. Keep the existing PR open. Do not
merge, release, tag, bump version, or rewrite history.

## Tasks

1. [x] **F-01 — session workspace ownership**
   - Add a RED regression with two actual nested Git repositories and a normal
     subdirectory control; cover linked-worktree `.git` files when supported.
   - Preserve canonical path spelling and symlink safety.
   - Accept only a recorded cwd whose effective Git/workspace root matches the
     intended project root; fail closed and report ownership-resolution errors.
2. [x] **F-02 — review status header parsing**
   - Add a RED regression with evidence and affected-file values beginning
     `Status: `open`` and `Status: `closed``; preserve the body and permit
     closure based on the single header status.
   - Keep duplicate/malformed header status fail-closed.
   - Require `## Summary` and search for status fields only before it.
3. [x] **F-03 — trace score recovery coverage**
   - Exercise recovery after the Vault trace and before the first index, after
     the first index and before the second, and after the final index before
     journal cleanup.
   - Verify idempotent replay and explicit rejection/preservation of an
     externally edited target at each boundary.
   - Do not change production transaction behavior unless a regression proves
     a defect.
4. [x] Run focused regressions, `cargo fmt --all -- --check`,
   `cargo test --workspace --all-targets --no-fail-fast -j 1`,
   `cargo clippy --workspace --all-targets -- -D warnings`, and
   `cargo build --workspace --locked --release`.
5. [ ] Push source and evidence commits to the same PR branch, verify exact-head
   hosted CI, and stop before merge for fresh independent review and explicit
   owner approval.

## Checkpoint before implementation

- Worktree was clean at `21eaa03b`; local and remote PR branch refs matched.
- F-01 and F-02 are source-confirmed from targeted inspection. F-03 remains a
  test gap per the review; no correctness defect has been inferred from it.
- No new test or source edit has run yet. Test state will use temporary repos,
  Vaults, and session files. The primary checkout and user-owned machine data
  are outside the write boundary.
- Proof and trace status: pending; no project-scoped Baron proof/trace.
- Safe next action: add the regressions first and record their RED/GREEN output
  in the companion verification ledger.

## Implementation result

- Source commit `61d111dff35a5586a490d496868309ca68557097` implements F-01/F-02
  and adds tests for F-03. Source commit
  `195bdf7a1ffae166e67e9fdb5436098598678da1` fixes an adjacent repeated-score
  formatting defect found by a byte-idempotency regression.
- Full workspace tests, Clippy, fmt, locked release build, CLI help/version,
  status JSON, 32 relative Markdown links, retired-adapter gate, and diff check
  pass on the final source. The Windows symlink test skipped because the OS
  denied symlink creation; exact-head Linux CI must run that case.
- Status and evidence docs record both source commits and local verification.
  Push and hosted CI are pending. PR #2 remains open; no merge or release was
  performed.
