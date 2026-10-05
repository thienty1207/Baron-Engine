# SPEC-05 Final Local Verification

Date: 2026-10-05. Status: `READY FOR ADVERSARIAL REVIEW`; not closed.

Scope: branch `codex/spec-05-multi-agent-concurrency-durable-state`, SPEC-04 closure base `ecaf362c275431edfc5e590fd6a87d926b9bde9d`. Before the evidence update, local HEAD was `c5f9b0be22765e023ebb908363620d23bbdf4dd4` and `origin/codex/spec-05-multi-agent-concurrency-durable-state` was `9bdb979a604e897adc9c5830f9912aa8e960809e`. The remote was not changed. All current source edits are within SPEC-05 Core/hooks/migration and their tests; current readiness/status evidence is maintained separately. No release, tag, version bump, SPEC-06, or Hotel Staff change.

## Current-source gates

The package and workspace all-target runs below each exited 0. Rust test execution was bounded to two test threads to keep Windows integration-fixture memory bounded; Cargo build jobs remained `-j 1`. The CLI, adapters, and workspace runs used the supported PowerShell 7 host override:

```powershell
$env:BARON_TEST_POWERSHELL='C:\Users\Ty\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe'
```

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo test -p baron-core --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, exit 0; `hook_identity` 20/20 (2,544.97 sec) |
| `cargo test -p baron-cli --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, exit 0 |
| `cargo test -p baron-adapters --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, exit 0 |
| `cargo test --workspace --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, exit 0; `hook_identity` 20/20 (2,506.94 sec) |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo build --workspace --release --locked` | PASS, exit 0 |
| `target/release/baron.exe --version` | PASS: `baron 5.0.0` |
| `cargo test -p baron-cli --test phase14_release release_binary_smokes_codex_claude_and_database_without_source_tree_assets -- --ignored --exact --test-threads=1` | PASS, 1/1 |

The all-target suites include the required focused gates: plan `54/54`; plan identity CLI `4/4`; proof/trace `29/29`; automation `8/8`; phase12 hooks `11/11`; phase12 hooks CLI `6/6`; adapter hook projection `3/3`; concurrency `17/17`; config `16/16`; capability `9/9`; control plane `9/9`; continuity `6/6`; intent `7/7`; harness `5/5`; harness improvement `10/10`; migration integration `17/17`; operation identity `10/10`; execution receipt `5/5`; retired-adapter gate `8/8`. The current migration unit suite is `9/9`, and hook-correlation unit tests are `2/2`.

## Contract and evidence checks

- `docs/BARON_STATUS.json` parses as JSON.
- Maintained/evidence Markdown check: 12 files, 14 local inline links, 0 broken targets; the Core maintained-doc contract test also passes in the workspace suite.
- The current Core/CLI CURRENT/latest-authority pattern scan returns 261 matching lines. The maintained authority audit classifies every production caller group and contains no `BUG` disposition; four Minor topics remain for fresh reviewers to reassess.
- `git diff ecaf362 --check` passes on the current working tree; Git emits only Windows LF/CRLF notices. The exact committed-range form is checked again after the evidence commit.
- No changed Cargo manifest/lockfile or changelog indicates a version bump; no tag points at the current HEAD. Public release remains `5.0.0`.
- No GitHub Actions/commit-status evidence is claimed. Local tests are not described as CI.

## Remaining acceptance gate

Earlier independent reviews on `ecaf362..7d052dc` returned `FIX REQUIRED`; those issues and subsequent local findings now have regression-backed repairs, but those old verdicts are not acceptance of this tree. Commit the scoped code/tests and this final evidence/status update, then obtain two fresh independent, read-only ACCEPT reviews of the exact `ecaf362..<final HEAD>` range. They must inspect the authority/hook/locking/compatibility criteria in the binding prompt. A quota or time failure is inconclusive. Close and push only after both accept; then verify local HEAD equals the remote branch SHA and report hosted status only if available.
