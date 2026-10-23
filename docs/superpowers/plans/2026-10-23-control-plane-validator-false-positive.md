# Control Plane Validator False Positive Bugfix

## Current checkpoint — implementation complete, PR creation blocked (2026-10-23)

- Objective: fix the Control Plane validator false positive where the generated Codex quality-agent projection forbids invoking subagents but its compound sentence is treated as an affirmative invocation.
- Scope: validator semantics, generated Codex projection regression, and CLI `status`/`prepare` consistency. This is a targeted bugfix, not SPEC-06.
- Baseline: dedicated branch `codex/control-plane-validator-false-positive`, based on verified `origin/main` commit `3cf40bb159f3f2522800f9ebc169a5ce10b346d1`; initial focused Core Control Plane tests passed 9/9.
- Root cause: `control_plane.rs` searched for `invoke other subagents` and exempted only exact `do not invoke other subagents`. The actual generated Codex projection says `Do not plan, implement, switch adapters, invoke other subagents, or change lifecycle ownership.`, so its coordinated negation was missed.
- Before-fix evidence: the new Core negation/descriptive regressions failed; actual Codex projections were flagged for all three mandatory agents; initialized CLI `control-plane status` reported `Passed: no` and `control-plane prepare` returned `control_plane_invalid` blockers. Reproduction used temporary project, Vault, and `BARON_HOME` fixtures.
- Implementation: explicitly negated coordinated actions, quoted wording, and descriptive mentions are ignored. Contrast clauses restart polarity so a positive invocation after `but` remains invalid. Positive imperative and modal invocation forms remain rejected.
- RED/GREEN results: Core false-positive cases were RED 0/2 before the fix and now Core Control Plane passes 15/15; adapter projection test was RED for all three agents and now the Codex bridge target passes 7/7; CLI reproduction was RED for `status` and `prepare` and now the Control Plane CLI target passes 3/3, with all three gates retained in the route.
- Verification: final Windows `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --all-targets --no-fail-fast -j 1` all passed. The full suite used `BARON_TEST_POWERSHELL` pointing to the repository-supported bundled PowerShell 7 executable and an isolated temporary `BARON_HOME`; four pre-existing tests were ignored. No hosted CI result exists yet for this branch.
- Commit `736d457` is pushed to `origin/codex/control-plane-validator-false-positive`. PR creation through the GitHub connector failed with HTTP 403 `Resource not accessible by integration`; the `gh` CLI is unavailable and the Brave window could not be activated after one refreshed-window retry. No PR has been created.
- Proof status: test evidence only; no Baron proof artifact. Trace status: no project-scoped Baron trace.
- Persisted-state boundary: isolated managed worktree and temporary test fixtures; the primary checkout, user-provided untracked files, existing docs-sync worktree/branch, Vault, and machine configuration are outside the mutation boundary.
- Safe next action: open a PR against `main` from [the pushed branch](https://github.com/thienty1207/Baron-Engine/pull/new/codex/control-plane-validator-false-positive); review it and stop before merge pending project-owner approval.

## Implementation plan

1. Add Core regressions for compound negation, descriptive prose, and a true positive.
2. Add adapter integration coverage that installs real Codex projections and validates all three mandatory agents.
3. Add CLI integration coverage proving status and prepare agree on the real initialized project.
4. Run the regressions to demonstrate the false positive, implement the validator fix, then rerun focused tests.
5. Run formatting, workspace tests, Clippy, and the Control Plane CLI smoke checks; record exact outcomes in the verification ledger.
6. Commit and push only this bugfix branch and open a dedicated PR. Stop before merge.
