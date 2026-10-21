# SPEC-05 Final Local Verification

Date: 2026-10-06. Status: `READY FOR ADVERSARIAL REVIEW`; not closed.

Scope: branch `codex/spec-05-multi-agent-concurrency-durable-state`, SPEC-04 closure base `ecaf362c275431edfc5e590fd6a87d926b9bde9d`. The committed candidate `cd12561` received two fresh full-range reviews; both returned FIX REQUIRED. Findings included Important gaps in operation-bound paired trace publication/selection and shared-Vault review-gate locking, plus same-thread migration rollback re-entry that failed check 13. All findings have RED→GREEN regressions. Integrated run `verification-9b7496b7b7864142a40c7206c5bb0732` passed Core 618/0/0, CLI 190/0/1 ignored, adapters 124/0/3 ignored, and workspace 932/0/4 ignored across 112 suites. Its first Clippy attempt found two behavior-neutral style lints; after those corrections, affected Core suites passed migration 21/21, concurrency 20/20, plan 56/56, proof/trace 31/31, followed by warnings-denied Clippy, locked release build/smoke and docs 12/12. No owned Critical/Important finding is currently known. The branch is not pushed; closure and hosted CI are not claimed. No release, tag, version bump, SPEC-06, Hotel Staff change, or unrelated file change is in scope.

The all-target run used unique nonexistent Codex/Claude session-root paths and the supported bundled PowerShell 7 host. Cargo jobs were serial (`-j 1`). Two Clippy-only source edits followed; these do not change behavior, and the directly affected Core targets plus Clippy/release/docs gates were rerun afterward.

## Current-source gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS on final formatted source |
| `cargo test -p baron-core --all-targets --no-fail-fast -j 1` | PASS, 618 passed / 0 failed / 0 ignored across 60 suites; `verification-9b7496b7b7864142a40c7206c5bb0732/core.log` |
| `cargo test -p baron-cli --all-targets --no-fail-fast -j 1` | PASS, 190 passed / 0 failed / 1 ignored across 38 suites; isolated roots and bundled PowerShell 7; `.../cli.log` |
| `cargo test -p baron-adapters --all-targets --no-fail-fast -j 1` | PASS, 124 passed / 0 failed / 3 ignored across 14 suites; `.../adapters.log` |
| `cargo test --workspace --all-targets --no-fail-fast -j 1` | PASS, 932 passed / 0 failed / 4 ignored across 112 suites; `.../workspace.log` |
| Post-lint focused Core rerun | PASS: concurrency 20/20, migration 21/21, plan 56/56, proof/trace 31/31 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS after correcting two lint-only findings from the first attempt |
| `cargo build --workspace --release --locked` | PASS after final source edits |
| `target/release/baron.exe --version` | PASS, `baron 5.0.0` |
| Ignored release smoke `release_binary_smokes_codex_claude_and_database_without_source_tree_assets` | PASS, explicitly executed 1/1 |
| CLI root, proof record, trace record/score, and continuity recovery help | PASS, all explicit commands exit 0 |
| `cargo test -p baron-core --test public_trust_docs -j 1 -- --test-threads=2` | PASS, 12 passed / 0 failed |
| `docs/BARON_STATUS.json` parse | PASS after current status update |
| Maintained relative-link scan | PASS, 8 changed Markdown files / 3 relative links / 0 broken |
| Tracked retired-adapter gate | PASS, no tracked current-product matches |
| `git diff ecaf362 --check` | PASS before the final docs refresh; rerun on final commit before review |

Fresh all-target logs are under `.superpowers/sdd/2026-10-01-spec-05-zero-current-closure-repair/verification-9b7496b7b7864142a40c7206c5bb0732/`. The run stopped at its first Clippy attempt, which reported two style lints; those were fixed, the affected suites were rerun, and Clippy/release/docs gates then passed. Cargo used serial jobs and isolated nonexistent session roots.

Focused follow-up results: migration `20/20` including explicit rollback rejection during a live installer callback; phase7 routing `20/20` including A-versus-CURRENT-B path isolation; phase10 adapter authority `15/15` including partial identity fail-closed. These and earlier lock-order/capsule-concurrency repairs are included in the fresh full-package/workspace runs. They remain subject to fresh independent adversarial review.

## Contract and evidence checks

- `docs/BARON_STATUS.json` parses after the final evidence refresh; `public_trust_docs` passes 12/12. The final changed-Markdown relative-link scan checked 6 maintained Markdown files and found 3 relative links / 0 broken.
- The retired-adapter grep returned no tracked current-product matches. The current Core/CLI authority-pattern scan returns 258 matching lines; the caller audit classifies identified routing as operation-scoped and harness improvement as operation-evidence scoped/ambiguous fail-closed.
- `git diff ecaf362 --check` passes after the final documentation refresh and will be repeated on the committed range. Windows LF/CRLF notices are informational.
- No manifest, lockfile, changelog, release, or version metadata changed. Public release remains `5.0.0`; no tag is created by this work.
- Local verification only. No GitHub Actions/commit-status result is claimed.

## Remaining acceptance gate

The first explicit-context rerun, `verification-f21fe964047d4dfda6657eb840b3edf7`, stopped at Core with 8 failures before the fix; it is retained as failure history, not pass evidence. The corrected run and focused post-lint rerun are the current local evidence. Two fresh independent adversarial reviews of exact `ecaf362..<final HEAD>` remain before closure and push; prior FIX REQUIRED verdicts do not count as acceptance.

Fresh documentation verification is `public_trust_docs` 12/12; the post-refresh relative-link scan checked 6 changed Markdown files and found 3 links / 0 broken. The status JSON parses, and the diff/scope checks pass on the final working tree. The retired-adapter grep returned no tracked current-product matches. The current Core/CLI authority-pattern scan returns 258 matching lines; identified routing and harness improvement use exact operation-scoped state/evidence and fail closed on ambiguity.

Baseline comparison: the current exact source was rechecked on 2026-10-06 with no `BARON_TEST_POWERSHELL` override: `lifecycle_scripts` is 4 passed / 3 failed, with the same three installer tests failing because this host cannot load `Microsoft.PowerShell.Archive`/`Compress-Archive`; the same 4/3 outcome and cause were recorded at `ecaf362`. With bundled PowerShell 7, current `lifecycle_scripts` is 7/7 (it is included in the passing CLI package run). This is a host limitation, not a SPEC-05 regression. `prepare_cli` is 5/5 at `ecaf362` and 6/6 current; `session_replay` is 4/4 at baseline and 5/5 current. The baseline implementation's unchecked `String::truncate(limit - 3)` panics for the synthetic 216-ASCII-plus-`é` boundary; the source-equivalent reproduction exited 101, while current `session_replay_rendering_never_splits_a_multibyte_character` passes. The UTF-8 issue is fixed, not an unchanged current failure. The complete accepted local workspace run uses isolated nonexistent session roots and the supported bundled PowerShell 7 host.

Two reviewers of `ecaf362..cd12561` returned FIX REQUIRED, so that candidate is not accepted. After all Critical/Important findings are repaired, the final source/evidence commit and complete local matrix must be prepared, then two fresh independent read-only reviewers must inspect exact `ecaf362..<actual HEAD>`, including all 14 prompt checks: proof/trace/score A while CURRENT=B, legacy multi-active ambiguity, Codex/Claude normalization, Prompt→Stop continuity, restart persistence, taskless Stop, unknown-turn fail-closed behavior, ACTIVE/correlation/frontmatter agreement, CURRENT presentation-only, lock/no-write timeouts, and cross-operation isolation. Reviewer quota/time failure is inconclusive. Only two ACCEPTs permit CLOSED and push; verify local HEAD equals remote SHA afterward. No hosted CI evidence is currently claimed.
