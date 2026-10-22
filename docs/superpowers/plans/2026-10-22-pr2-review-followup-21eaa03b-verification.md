# PR #2 Review Follow-up — Verification Ledger

- **Date:** 2026-10-22
- **Reviewed HEAD:** `21eaa03b314d479d6524dba34d0c6bc34be8368a`
- **Branch:** `codex/spec-05-maintenance-01-logic-integrity`
- **Worktree:** `C:/Users/tytyb/.codex/worktrees/spec-05-maintenance-01-logic-integrity/Baron-Engine`
- **Source commits:** `61d111dff35a5586a490d496868309ca68557097`, `195bdf7a1ffae166e67e9fdb5436098598678da1`

## Review classification

| Finding | Initial classification | Source evidence | Required proof |
|---|---|---|---|
| F-01 | CONFIRMED BUG | `session.rs::path_matches_repo` canonicalizes then accepts any descendant; it does not distinguish a nested repository/worktree owner. | Import from a parent containing nested Git repo/worktree must exclude the nested session while accepting an ordinary subdirectory session. |
| F-02 | CONFIRMED BUG | `review_gate.rs::finding_status_field` notices the Summary boundary but gathers status-shaped lines across the whole document before requiring exactly one. | Status-shaped evidence and affected-file text must remain verbatim, while exactly one valid header field controls list and closure. Duplicate header status must still fail closed. |
| F-03 | TEST GAP — NO SOURCE BUG CLAIMED | Existing transaction code validates all four target states before replay; the current deterministic failure test covers only the boundary before Vault trace publication. | Test idempotent recovery and external-edit rejection after Vault trace/before first index, between indexes, and after final index/before journal cleanup. |

## Checkpoint and mutation boundary

- Exact PR head and remote branch were `21eaa03b314d479d6524dba34d0c6bc34be8368a` before this work; the managed worktree was clean.
- The new plan/checkpoint was recorded before source modifications. Tests use temporary repositories, Vaults, and session files; the primary checkout and user-owned local data are untouched.
- Before source edits, focused RED tests reproduced both confirmed bugs:
  F-01 imported one session from a nested independent Git repository and one
  from a linked worktree into the parent; expected zero in both cases. F-02
  returned an ambiguous status when valid body evidence contained a
  `Status: closed` line.
- Proof/trace status: tests use temporary repositories, Vaults, and session
  roots. No project-scoped Baron proof or trace was produced.

## Test and implementation evidence

### F-01 — session workspace ownership

- RED: parent repo import incorrectly accepted the nested-repository session
  (actual 2, expected 1) and linked-worktree session (actual 1, expected 0).
- GREEN: canonicalize the recorded cwd and intended repo; require the nearest
  effective Git/workspace root to match the intended repo. Ordinary
  subdirectories remain valid. Unresolvable cwd/Git metadata fails closed.
- Coverage: real nested Git repository, real linked worktree whose `.git` is a
  file, ordinary subdirectory, missing/unresolvable cwd, and symlink escape.
  The symlink assertion could not run locally because Windows denied symlink
  creation; hosted Linux CI must execute it.

### F-02 — review finding status parsing

- RED: a `Status: closed` line in body evidence made status ambiguous even
  though the header had exactly one valid status.
- GREEN: require `## Summary`; parse status only in the header before it;
  preserve all body evidence/affected files; reject duplicate or malformed
  header fields. Closure and listing tests pass, including body text containing
  status-shaped values.

### F-03 — trace score transaction recovery coverage

- Classification remains **test gap only; no transaction source defect found**.
- Added deterministic interruptions after Vault trace publication/before the
  first index, after the first index/before the second, and after the final
  index/before journal cleanup. At each boundary, recovery reaches a coherent
  state; a second recovery is a no-op; externally edited targets are rejected
  and preserved.
- Additional adjacent defect found during repeated-scoring regression: scoring
  the same trace twice appended an extra newline. RED reproduced byte drift in
  the trace files; `replace_score` now preserves one stable separator, and the
  repeated-score test confirms byte-identical repo/Vault traces and indexes.
  This is separate from F-03 transaction correctness.

### Local verification

- Focused F-01 session-import regressions: **4 passed** on Windows; symlink
  creation was denied by the OS, so that assertion is not counted as locally
  executed.
- Review Gate integration suite: **12 passed**.
- Trace score transaction unit suite: **4 passed**, including all three
  recovery boundaries and repeated-score byte-idempotency.
- `cargo fmt --all -- --check`: **PASS**.
- `cargo test --workspace --all-targets --no-fail-fast -j 1`: **PASS**, exit 0
  on the final source state, using temporary `BARON_HOME` and session roots.
- `cargo clippy --workspace --all-targets -- -D warnings`: **PASS**.
- `cargo build --workspace --locked --release`: **PASS**; release CLI reports
  `baron 5.0.0`.
- CLI help smoke for top-level, `setup`, `init`, `update`, and `context`:
  **PASS**. Status JSON parses; all **32** repository-relative Markdown links
  in the five changed Markdown files resolve;
  retired-adapter content/filename gate and `git diff --check` pass.
- No project-specific state, user Vault, machine configuration, or primary
  checkout was modified. No proof/trace artifact was written to Baron state.

## Repository gates

Local source and documentation gates pass on Windows. Windows denied symlink
creation, so exact-head hosted Linux CI must execute that test. The evidence
commit and push are pending. Hosted CI for the new PR head and fresh
independent review remain pending.

## Safe next action

Commit the final evidence ledger, push the commits to the same PR #2 branch,
verify checks on the exact pushed head, and request fresh independent review.
Keep PR #2 open for owner review; do not merge or release.
