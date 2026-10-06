# SPEC-05 Final Closure Verification Ledger

Date: 2026-10-06. Status: `READY FOR ADVERSARIAL REVIEW`; not closed. Full local gates passed. The readiness snapshot still needs its final tracked-doc checks, push/SHA verification, hosted-status query, and two fresh exact-range reviews.

## Source and scope

- Branch: `codex/spec-05-multi-agent-concurrency-durable-state`.
- Starting HEAD and remote HEAD: `3cd03c6793807f2a149b3ed6f56dc43027640955`.
- Review base: `ecaf362c275431edfc5e590fd6a87d926b9bde9d`.
- Public binary target remains `baron 5.0.0`.
- Test isolation: `BARON_CODEX_SESSIONS_ROOT=C:\__baron_spec05_no_codex_sessions__`; `BARON_CLAUDE_SESSIONS_ROOT=C:\__baron_spec05_no_claude_sessions__`.
- No release, tag, version bump, SPEC-06, Hotel Staff change, or pushed-history rewrite is authorized or performed.

## Fresh authority audit

Reran every required authority-search term over `crates/baron-core/src/**`, `crates/baron-cli/src/**`, and `crates/baron-adapters/src/**`, excluding test modules, imports, comments, and function declarations from production hit review. The inventory is at [SPEC-05 authority audit](2026-10-01-spec-05-authority-audit.md): 45 caller-path groups, each with exactly one category; `PRESENTATION_ONLY=11`, `LEGACY_SINGLE_ACTIVE_SAFE=8`, `EXACT_OPERATION_SCOPED=17`, `AMBIGUOUS_FAIL_CLOSED=6`, `OUT_OF_SPEC=3`, `BUG=0`. No known owned Critical/Important issue remains in the current-source audit. Audit status alone is not readiness or closure.

## Fresh focused regression matrix

All listed focused tests below ran against the current working-tree source with isolated session roots. Counts are test-case pass/fail counts (all observed failed counts are zero).

| Target | Passed / failed / ignored | Invocation |
| --- | ---: | --- |
| `plan` | 57 / 0 / 0 | Core focused matrix |
| `plan_identity_cli` | 4 / 0 / 0 | CLI focused matrix |
| `authority_ingress` | 6 / 0 / 0 | Core focused matrix |
| `operation_evidence_cli` | 11 / 0 / 0 | CLI focused matrix |
| `proof_trace` | 36 / 0 / 0 | Core focused matrix |
| `automation` | 9 / 0 / 0 | Core focused matrix |
| `autopilot` | 4 / 0 / 0 | Core focused matrix |
| `hook_identity` | 20 / 0 / 0 | Core focused matrix |
| `phase12_hooks` | 11 / 0 / 0 | Core focused matrix |
| `phase12_hooks_cli` | 6 / 0 / 0 | CLI focused matrix |
| `harness_scoping` | 13 / 0 / 0 | Core focused matrix |
| `harness` | 5 / 0 / 0 | Core focused matrix |
| `harness_improvement` | 13 / 0 / 0 | Core focused matrix; second required invocation |
| `concurrency` | 20 / 0 / 0 | Core focused matrix |
| `continuity` | 7 / 0 / 0 | Core focused matrix |
| `operation_context` | 18 / 0 / 0 | Core focused matrix |
| `intent` | 7 / 0 / 0 | Core focused matrix, separate invocation |
| `intent_operation_cli` | 2 / 0 / 0 | CLI focused matrix |
| `control_plane` | 9 / 0 / 0 | Core focused matrix |
| `phase7_routing` | 20 / 0 / 0 | Core focused matrix |
| `phase10_adapter_authority` | 15 / 0 / 0 | Core focused matrix |
| `capability` | 9 / 0 / 0 | Core focused matrix |
| `review_gate` | 6 / 0 / 0 | Core focused matrix |
| `migration` | 21 / 0 / 0 | Core focused matrix |
| `harness_experiment` | 5 / 0 / 0 | Core focused matrix |
| `public_config_compat` | 1 / 0 / 0 | Core focused matrix |
| `session_replay` | 5 / 0 / 0 | Core focused matrix; UTF-8 boundary regression passes |
| `execution_cli` | 15 / 0 / 0 | CLI focused matrix |

The first Core focused command totalled 310 passed; the separate `intent` run added 7 (317). The duplicated `harness_improvement` invocation added another 13 passes per the prompt's repeated entry. CLI selector/evidence/hook targets totalled 38 passes, excluding lifecycle baseline checks below.

The tracked retired-adapter gate (case-insensitive tracked-source search) exited 1 with no output, meaning no tracked current-tree matches.

## Baseline and lifecycle environment

- Default host `powershell.exe`: `cargo test -p baron-cli --test lifecycle_scripts -j 1 --no-fail-fast` exited 1 with 4 passed, 3 failed. All three failures were `Compress-Archive` failing to load `Microsoft.PowerShell.Archive`; this is an environment limitation, not counted as a product pass.
- Repository-supported bundled PowerShell 7 (`BARON_TEST_POWERSHELL=C:\Users\Ty\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe`): same target passed 7/7.
- `session_replay` current UTF-8 boundary test passed within the current-source focused run (5/5 suite).

## Full verification and delivery gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| Core all-targets | PASS, exit 0; 631 passed, 0 failed, 0 ignored across 60 suites |
| CLI all-targets (supported bundled PowerShell 7) | PASS, exit 0; 190 passed, 0 failed, 1 ignored across 38 suites |
| Adapters all-targets | PASS, exit 0; 124 passed, 0 failed, 3 historical ignored across 14 suites |
| Workspace all-targets | PASS, exit 0; 945 passed, 0 failed, 4 ignored across 112 suites |
| Clippy `-D warnings` | PASS, exit 0 |
| Locked workspace release build | PASS, exit 0 |
| Release binary version / smoke | PASS; binary reports `baron 5.0.0`, required ignored smoke passes 1/1 |
| Public trust docs | PASS, 12/12 |
| `BARON_STATUS.json` parse | PASS |
| Changed Markdown relative links | PASS, 11 checked, 0 broken |
| `git diff ecaf362 --check` | PASS, exit 0 (Git emitted only line-ending conversion warnings) |
| CLI help checks | PASS; root, proof record, trace record, trace score, continuity recover all exit 0 |
| GitHub Actions and commit statuses | PENDING for readiness snapshot |
| Readiness push and local/remote SHA equality | PENDING |
| Two fresh exact-range independent reviews | PENDING |
| Closure evidence push and final local/remote SHA equality | PENDING |

The first workspace attempt before removing tracked status wording exited 101 with 944 passed, 1 failed, and 4 ignored across 112 suites. The failing retired-reference test found the literal in the status JSON; the wording was corrected without changing the gate. Staging the two new evidence docs exposed the same condition: the existing test went RED because the untracked docs had included the literal in their examples. After correcting both docs, the targeted tracked-reference/filename tests passed 2/2 and public trust docs passed 12/12. The exact complete verification script then passed all package/workspace suites, Clippy, locked release, release smoke, docs, version, and CLI help. No Rust source changed after that complete run.

Prior reviews of `ecaf362..4a3efc0` are historical `FIX REQUIRED` reviews, not acceptance. Current status is `SPEC-05: READY FOR ADVERSARIAL REVIEW`, not CLOSED. Closure requires two fresh independent `ACCEPT` verdicts for one identical pushed range and zero unresolved Critical/Important findings.
