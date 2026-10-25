# Local v5.0.1 Version Bump

## Goal

Prepare a reviewable local source candidate for Baron Engine 5.0.1 after the
post-merge end-to-end verification. The public stable/downloadable release
remains v5.0.0. At initial completion, this plan did not authorize a tag,
commit, push, or release. The owner later authorized committing and pushing
this candidate branch with the AI handoff documentation task; no tag or
release is authorized.

> Scope update — 2026-10-25: the owner subsequently requested stable and
> downloadable v5.0.1 and authorized PR creation, merge, and immutable release
> promotion after all gates pass. The restriction above records the original
> version-bump task scope only; [`CURRENT.md`](CURRENT.md) is the live handoff
> for the release follow-on and its current authorization/status.

## Baseline

- Repository: `thienty1207/Baron-Engine`
- Base commit: `cf319a12cfc19e2739303b1b26ee71357ae6c0cd`
- Work branch: `codex/version-5-0-1`
- Worktree: managed isolated worktree for this task
- Previous source verification: the full workspace suite, fmt, warnings-denied
  Clippy, and hosted Windows/Linux native CI passed on the unchanged merge
  source, as recorded in the previous checkpoint.

## Tasks

- [x] Record the authorized scope and release boundary in maintained status docs.
- [x] Set the workspace package version and certification target to 5.0.1.
- [x] Add an unreleased 5.0.1 changelog entry and label README source version as
  an unpublished candidate; keep installer instructions and stable release at
  5.0.0.
- [x] Refresh Cargo.lock and candidate status metadata.
- [x] Run required `cargo fmt`, full workspace tests, and warnings-denied
  Clippy, plus focused documentation/certification tests, a locked workspace
  check, release build and `baron --version` smoke check.
- [x] Validate format, JSON, links, and diff; update status docs to final local
  candidate state.

## Acceptance

- All three workspace crates resolve to 5.0.1 and the release binary reports
  `baron 5.0.1`.
- Existing 5.0.0 installer instructions, public release URL, and historical
  changelog remain accurate.
- Certification reports target release 5.0.1.
- Focused contract checks and locked build pass.
- No tag or publication is created. Commit and branch push were authorized
  separately by the owner on 2026-10-24 for the handoff documentation task.

## Review Focus

- Verify no reference that describes the published 5.0.0 artifacts was changed
  to imply that 5.0.1 is already downloadable.
- Verify workspace crate lock entries and certification target match 5.0.1.
- Preserve unrelated user files and all pre-existing primary-checkout changes.

## Verification Ledger

- Baseline confirmed at `cf319a12cfc19e2739303b1b26ee71357ae6c0cd`; worktree
  branch created as `codex/version-5-0-1`.
- Ruling: keep `targetRelease` and `programTargetRelease` at 5.0.0 while the
  unpublished source candidate is 5.0.1 — the maintained status contract uses
  those fields for the current release program, and `status_tracks_current_program`
  fails if they are advanced for a local-only version bump. Cost if wrong: the
  dashboard could understate the next release target; 5.0.1 remains explicit in
  `candidateSourceVersion` and the current phase.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --all-targets --no-fail-fast -j 1`: passed with
  `BARON_TEST_POWERSHELL` set to bundled PowerShell 7. Focused Core suites
  passed (certification 1/1, control plane 15/15, public docs 12/12); adapter
  `--all-targets` passed. Five CLI help checks passed.
- Initial unconfigured Windows run had three `lifecycle_scripts` failures
  because the test-spawned Windows PowerShell 5.1 process could not auto-load
  `Microsoft.PowerShell.Archive`; serial rerun had the same issue. A direct
  archive command in Windows PowerShell succeeded, and the suite's supported
  `BARON_TEST_POWERSHELL` override to PowerShell 7 passed all 7 lifecycle tests
  and the subsequent full workspace run.
- `cargo check --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo build --workspace --release --locked`: passed; release executable
  prints `baron 5.0.1`.
- Status JSON parsed; 45 repository-relative links resolved; `git diff --check`
  passed. `git grep -in deepseek` found one historical roadmap mention only;
  current-product retired-adapter regression tests passed.
- At the time this candidate-preparation plan completed, no commit, tag, push,
  or release had been created. The owner later authorized a branch push in the
  AI handoff follow-up. Hosted Windows/Linux CI has not run on this candidate.
- Final review findings: relabeled the stale v5.0.0 overall snapshot and Phase
  16 release instructions as historical/superseded; removed the duplicate
  post-merge plan link. The focused public-docs suite (12/12), local links
  (45/45), and diff check passed after these documentation fixes.
- Deferred reviewer recommendation: add a direct test assertion that the
  current `candidateSourceVersion` matches the workspace package version. The
  current candidate was verified through Cargo metadata and release CLI smoke;
  expanding the test contract is outside this version-only bump.
