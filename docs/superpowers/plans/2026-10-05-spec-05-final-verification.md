# SPEC-05 Final Local Verification

Date: 2026-10-06. Status: `READY FOR ADVERSARIAL REVIEW`; not closed.

Scope: branch `codex/spec-05-multi-agent-concurrency-durable-state`, SPEC-04 closure base `ecaf362c275431edfc5e590fd6a87d926b9bde9d`. Review of committed range `ecaf362..6100e21` found two Important findings; both now have regression-backed repairs. Fresh integrated verification passed on 2026-10-06. Source and evidence are not yet committed, the exact final range has not received the required two fresh independent reviews, and this branch is not pushed. No closure or hosted CI result is claimed. No release, tag, version bump, SPEC-06, Hotel Staff change, or unrelated file change is in scope.

Current test runs use unique nonexistent Codex/Claude session-root paths and the supported bundled PowerShell 7 host. Rust test execution is limited to two test threads and Cargo jobs to `-j 1` for bounded Windows fixture memory. All results below refer to this continuation unless marked historical.

## Current-source gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0; `verification-430c4b36c0da42a298af9fdc6e1eee5b/fmt.log` |
| `cargo test -p baron-core --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 608 passed / 0 failed / 0 ignored across 60 suites; `.../core.log` |
| `cargo test -p baron-cli --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 190 passed / 0 failed / 1 ignored across 38 suites; isolated roots and bundled PowerShell 7; `.../cli.log` |
| `cargo test -p baron-adapters --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 124 passed / 0 failed / 3 ignored across 14 suites; `.../adapters.log` |
| `cargo test --workspace --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 922 passed / 0 failed / 4 ignored across 112 suites; `.../workspace.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0; `.../clippy.log` |
| `cargo build --workspace --release --locked` | PASS, exit 0; `.../release.log` |
| `target/release/baron.exe --version` | PASS, `baron 5.0.0` |
| Ignored release smoke `release_binary_smokes_codex_claude_and_database_without_source_tree_assets` | PASS, explicitly executed 1/1; `.../release-smoke.log` |
| CLI root, proof record, trace record/score, and continuity recovery help | PASS, all explicit commands exit 0 |
| `cargo test -p baron-core --test public_trust_docs -j 1 -- --test-threads=2` | PASS, 12 passed / 0 failed; `.../docs.log` |
| `docs/BARON_STATUS.json` parse | PASS, parsed after final status/evidence updates |
| Maintained relative-link scan | PASS, 5 changed Markdown files / 2 relative links / 0 broken |
| Tracked retired-adapter gate | PASS in verifier, no tracked current-product matches |
| `git diff ecaf362 --check` | PASS, rerun after final evidence edit; rerun on committed range before review |

Fresh verification logs are under `.superpowers/sdd/2026-10-01-spec-05-zero-current-closure-repair/verification-430c4b36c0da42a298af9fdc6e1eee5b/`. The verifier used unique nonexistent Codex/Claude session roots and bundled PowerShell 7. Cargo ran serial jobs with two Rust test threads.

Focused current results include plan `56/56`, proof/trace `29/29`, automation `8/8`, concurrency `20/20`, config `16/16`, capability `9/9`, control-plane `9/9`, continuity `6/6`, intent `7/7`, operation-context `16/16`, operation identity `10/10`, execution receipt `5/5`, receipt authority `28/28`, hook identity `20/20`, migration `19/19`, harness `5/5`, harness experiment `5/5`, harness improvement `12/12`, operation-evidence CLI `11/11`, phase12-hooks CLI `6/6`, plan-identity CLI `4/4`, adapter lifecycle `23/23`, and adapter phase12 hooks `3/3`. Source-lock ordering and capsule concurrency repairs remain subject to fresh independent adversarial review.

## Contract and evidence checks

- `docs/BARON_STATUS.json` parses; `public_trust_docs` passes 12/12. The post-edit changed-Markdown scan checked 5 maintained Markdown files and 2 relative inline links with 0 broken.
- The retired-adapter grep returned no tracked current-product matches. The current Core/CLI authority-pattern scan returns 258 matching lines; the caller audit classifies harness improvement as operation-evidence scoped and ambiguous fail-closed.
- `git diff ecaf362 --check` passes on the current working tree; exact committed-range checks and version/scope checks must be repeated after commit. Windows LF/CRLF notices are informational.
- No manifest, lockfile, changelog, release, or version metadata changed. Public release remains `5.0.0`; no tag is created by this work.
- Local verification only. No GitHub Actions/commit-status result is claimed.

## Remaining acceptance gate

Fresh current documentation verification is `public_trust_docs` 12/12; the post-edit changed-Markdown relative-link scan found 2 relative inline links and 0 broken links across 5 maintained files. The retired-adapter grep returned no tracked current-product matches. The current Core/CLI authority-pattern scan returns 258 matching lines; harness improvement uses operation-scoped evidence and fails closed on ambiguity.

Baseline comparison: the current exact source was rechecked on 2026-10-06 with no `BARON_TEST_POWERSHELL` override: `lifecycle_scripts` is 4 passed / 3 failed, with the same three installer tests failing because this host cannot load `Microsoft.PowerShell.Archive`/`Compress-Archive`; the same 4/3 outcome and cause were recorded at `ecaf362`. With bundled PowerShell 7, current `lifecycle_scripts` is 7/7 (it is included in the passing CLI package run). This is a host limitation, not a SPEC-05 regression. `prepare_cli` is 5/5 at `ecaf362` and 6/6 current; `session_replay` is 4/4 at baseline and 5/5 current. The baseline implementation's unchecked `String::truncate(limit - 3)` panics for the synthetic 216-ASCII-plus-`é` boundary; the source-equivalent reproduction exited 101, while current `session_replay_rendering_never_splits_a_multibyte_character` passes. The UTF-8 issue is fixed, not an unchanged current failure. The complete accepted local workspace run uses isolated nonexistent session roots and the supported bundled PowerShell 7 host.

Prior independent reviews of `ecaf362..6100e21` returned `FIX REQUIRED` for migration durability and shared-Vault harness locking; they do not accept the repairs. Local source/package/workspace/lint/release checks now pass and status is `READY FOR ADVERSARIAL REVIEW`, not closed. After committing the final source and evidence/docs snapshot, two fresh independent read-only reviewers must inspect exact `ecaf362..<actual HEAD>`, including all 14 prompt checks: proof/trace/score A while CURRENT=B, legacy multi-active ambiguity, Codex/Claude normalization, Prompt→Stop continuity, restart persistence, taskless Stop, unknown-turn fail-closed behavior, ACTIVE/correlation/frontmatter agreement, CURRENT presentation-only, lock/no-write timeouts, and cross-operation isolation. Reviewer quota/time failure is inconclusive. Only two ACCEPTs permit CLOSED and push; verify local HEAD equals remote SHA afterward. No hosted CI evidence is currently claimed.
