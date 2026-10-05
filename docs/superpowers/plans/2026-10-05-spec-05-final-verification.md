# SPEC-05 Final Local Verification

Date: 2026-10-05. Status: `READY FOR ADVERSARIAL REVIEW`; not closed.

Scope: branch `codex/spec-05-multi-agent-concurrency-durable-state`, SPEC-04 closure base `ecaf362c275431edfc5e590fd6a87d926b9bde9d`. The three final review repairs are in source commit `f3eecd7` (`fix(spec05): preserve shared vault authority under concurrency`). The branch is local and has not been pushed; before this repair commit it was 12 commits ahead of remote `9bdb979a604e897adc9c5830f9912aa8e960809e`. This repair commit changes SPEC-05 Core/migration code and regression tests; the review range contains the SPEC-05 Core/CLI/adapter implementation and maintained evidence. No unrelated scope, release, tag, version bump, SPEC-06, or Hotel Staff change.

The source/test tree verified below is byte-for-byte the source committed as `f3eecd7`; only documentation/status files changed after those code checks. Workspace and CLI test runs used unique nonexistent Codex/Claude session-root paths and the supported bundled PowerShell 7 host. Rust test execution was limited to two test threads and Cargo jobs to `-j 1` for bounded Windows fixture memory.

## Current-source gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo test -p baron-core --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, exit 0; the isolated workspace run also exercises all Core targets |
| `cargo test -p baron-cli --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 190 passed / 0 failed / 1 ignored across 38 suites |
| `cargo test -p baron-adapters --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 124 passed / 0 failed / 3 ignored across 14 suites |
| `cargo test --workspace --all-targets --no-fail-fast -j 1 -- --test-threads=2` | PASS, 911 passed / 0 failed / 4 ignored across 112 suites |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo build --workspace --release --locked` | PASS, exit 0 |
| `target/release/baron.exe --version` | PASS: `baron 5.0.0` |
| Ignored release smoke `release_binary_smokes_codex_claude_and_database_without_source_tree_assets` | PASS, 1/1 |
| CLI `--help`, proof/trace record/score help, and continuity recovery help | PASS |

The full workspace log is `.superpowers/sdd/2026-10-01-spec-05-zero-current-closure-repair/verification-final-432c3023716945d1a031755d2bd7fe69/workspace.log`; direct CLI log is in the same directory as `cli.log`. The adapter package log is `.superpowers/sdd/2026-10-01-spec-05-zero-current-closure-repair/verification-isolated-558b121b7fc548728de6be1b27fd1857/adapters.log`.

Current focused suites include concurrency `19/19`, plan `55/55`, migration `17/17`, hook identity `20/20`, phase12 hooks `11/11`, operation-evidence CLI `11/11`, phase12 hooks CLI `6/6`, plan-identity CLI `4/4`, prepare CLI `6/6`, and public trust docs `12/12`. The four repair regressions are `shared_vault_active_plan_index_preserves_operations_from_other_checkouts`, `trace_record_and_score_fail_without_writes_when_shared_vault_lock_is_held`, `plan_mutation_fails_without_writes_when_shared_vault_lock_is_held`, and `rollback_resumes_after_target_moved_marker_is_removed_before_cleanup_marker`; each was observed RED before its fix and GREEN afterward.

## Contract and evidence checks

- `docs/BARON_STATUS.json` parses as JSON; public trust-doc tests pass `12/12` after the status/docs update.
- Maintained/evidence Markdown check: 12 files, 12 relative local inline links, 0 broken targets.
- `git grep -in reasonix`: no tracked match. The maintained authority audit classifies current Core/CLI production caller groups and has no `BUG` disposition; four Minor topics remain disclosed for reviewers to reassess.
- `git diff ecaf362 --check` passes on the source and current readiness docs; Windows LF/CRLF notices are informational. Repeat against the final committed range after the evidence commit.
- No manifest, lockfile, changelog, release, or version metadata changed. Public release remains `5.0.0`; no tag is created by this work.
- Local verification only. No GitHub Actions/commit-status result is claimed.

## Remaining acceptance gate

The two fresh independent reviews of `ecaf362..868f7c1` returned `FIX REQUIRED`; their verdicts are not acceptance of the repaired tree. Commit the scoped readiness/status evidence, then obtain two fresh independent, read-only ACCEPT reviews of the exact full range `ecaf362..<final HEAD>`, including that evidence commit. Each reviewer must inspect proof/trace/score A while CURRENT=B, legacy multi-active ambiguity, Codex and Claude field normalization, Prompt→Stop identity continuity, restart persistence, Stop without task text, unknown-turn fail-closed behavior, ACTIVE/correlation/frontmatter agreement, CURRENT as presentation only, lock/no-write timeout behavior, and cross-operation evidence isolation. Reviewer quota/time failure is inconclusive. Mark CLOSED and push only after both ACCEPT; then verify local HEAD equals the remote branch SHA and report hosted status only if evidence exists.
