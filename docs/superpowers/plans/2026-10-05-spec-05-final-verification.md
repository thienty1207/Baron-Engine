# SPEC-05 Final Local Verification

Date: 2026-10-05. Status: `READY FOR ADVERSARIAL REVIEW`; not closed.

Scope: branch `codex/spec-05-multi-agent-concurrency-durable-state`, SPEC-04 closure base `ecaf362c275431edfc5e590fd6a87d926b9bde9d`. Current code/test/evidence changes are an uncommitted continuation on the local branch after `95ecd3f`; they have not been committed or pushed. Prior reviews of `ecaf362..868f7c1` were FIX REQUIRED and are not acceptance. No unrelated scope, release, tag, version bump, SPEC-06, or Hotel Staff change.

Current test runs use unique nonexistent Codex/Claude session-root paths and the supported bundled PowerShell 7 host. Rust test execution is limited to two test threads and Cargo jobs to `-j 1` for bounded Windows fixture memory. All results below refer to this continuation unless marked historical.

## Current-source gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS on current Rust source/test bytes |
| `cargo test -p baron-core --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 602 passed / 0 failed / 0 ignored across 60 targets; log `core-final-20261005-a2.log` |
| `cargo test -p baron-cli --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 190 passed / 0 failed / 1 ignored across 38 targets; isolated session roots and bundled PowerShell 7; log `cli-final-20261005-a3.log` |
| `cargo test -p baron-adapters --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 124 passed / 0 failed / 3 ignored across 14 targets; log `adapters-final-20261005-a4.log` |
| `cargo test --workspace --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 916 passed / 0 failed / 4 ignored across 112 targets; isolated session roots and bundled PowerShell 7; final rerun log `workspace-local-acceptance-20261005-a7.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS on current Rust source/test bytes |
| `cargo build --workspace --release --locked` | PASS, exit 0; log `release-build-20261005-a5.log` |
| `target/release/baron.exe --version` | PASS, `baron 5.0.0` |
| Ignored release smoke `release_binary_smokes_codex_claude_and_database_without_source_tree_assets` | PASS, 1 passed / 0 failed / 1 filtered out; log `release-smoke-20261005-a6.log` |
| CLI root, proof record, trace record/score, and continuity recovery help | PASS, all explicit commands exit 0 |
| `cargo test -p baron-core --test public_trust_docs -j 1 -- --test-threads=2` | PASS, 12 passed / 0 failed |
| `docs/BARON_STATUS.json` parse | PASS, `ConvertFrom-Json` |
| Maintained relative-link scan | PASS, 8 changed Markdown files / 2 relative inline links / 0 broken |
| Tracked retired-adapter gate | PASS, no current-tree matches |
| `git diff ecaf362 --check` | PASS on current source/evidence working tree; repeat on exact committed range |

Previous-run logs under `.superpowers/sdd/2026-10-01-spec-05-zero-current-closure-repair/verification-*` are historical and do not represent this continuation. Current package/release logs are `core-final-20261005-a2.log`, `cli-final-20261005-a3.log`, `adapters-final-20261005-a4.log`, `release-build-20261005-a5.log`, and `release-smoke-20261005-a6.log`; the last full workspace rerun is `workspace-local-acceptance-20261005-a7.log`.

Focused current results in those runs include concurrency `20/20`, operation-context `16/16`, plan `56/56`, migration `18/18`, hook identity `20/20`, Vault memory `15/15`, intent `7/7`, control-plane `9/9`, operation-evidence CLI `11/11`, phase12 hooks `11/11`, phase12 hooks CLI `6/6`, plan-identity CLI `4/4`, prepare CLI `6/6`, and harness improvement `11/11`. Source-lock ordering and capsule-resolution choices remain subject to fresh independent review.

## Contract and evidence checks

- `docs/BARON_STATUS.json` parses; `public_trust_docs` passes 12/12. The changed-Markdown scan checked 8 Markdown files and 2 relative inline links with 0 broken.
- The retired-adapter grep returned no tracked current-product matches. The current Core/CLI authority-pattern scan returns 258 matching lines; the caller audit classifies harness improvement as operation-evidence scoped and ambiguous fail-closed.
- `git diff ecaf362 --check` passes on the current working tree; exact committed-range checks and version/scope checks must be repeated after commit. Windows LF/CRLF notices are informational.
- No manifest, lockfile, changelog, release, or version metadata changed. Public release remains `5.0.0`; no tag is created by this work.
- Local verification only. No GitHub Actions/commit-status result is claimed.

## Remaining acceptance gate

Fresh current documentation verification is `public_trust_docs` 12/12; the changed-Markdown relative-link scan found 2 relative inline links and 0 broken links across 8 files. The retired-adapter grep returned no tracked current-product matches. The current Core/CLI authority-pattern scan returns 258 matching lines; harness improvement uses operation-scoped evidence and fails closed on ambiguity.

Baseline comparison (same default Windows PowerShell host, no `BARON_TEST_POWERSHELL` override): `lifecycle_scripts` is 4 passed / 3 failed at both `ecaf362` and current source, with the same three installer tests failing because `Microsoft.PowerShell.Archive` cannot load `Compress-Archive`. This is an environment limitation, not a SPEC-05 source regression. `prepare_cli` is 5/5 at `ecaf362` and 6/6 current; `session_replay` is 4/4 at baseline and 5/5 current. The baseline implementation's unchecked `String::truncate(limit - 3)` panics for the synthetic 216-ASCII-plus-`é` boundary; the source-equivalent reproduction exited 101, while the current regression `session_replay_rendering_never_splits_a_multibyte_character` passes. Thus the UTF-8 issue is fixed, not an unchanged current failure. The normal full suite used the supported bundled PowerShell 7 host and passed.

The two independent reviews of `ecaf362..868f7c1` returned `FIX REQUIRED`; their verdicts are not acceptance of the repaired tree. Local verification is complete and status is now `READY FOR ADVERSARIAL REVIEW`, not closed. Commit the scoped final code and evidence, then obtain two fresh independent, read-only ACCEPT reviews of exact `ecaf362..<final HEAD>`. Each reviewer must inspect proof/trace/score A while CURRENT=B, legacy multi-active ambiguity, Codex and Claude field normalization, Prompt→Stop identity continuity, restart persistence, Stop without task text, unknown-turn fail-closed behavior, ACTIVE/correlation/frontmatter agreement, CURRENT as presentation only, lock/no-write timeout behavior, and cross-operation evidence isolation. Reviewer quota/time failure is inconclusive. Mark CLOSED and push only after both ACCEPT; then verify local HEAD equals the remote branch SHA and report hosted status only if evidence exists.
