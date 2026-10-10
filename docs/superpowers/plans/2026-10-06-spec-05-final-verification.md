# SPEC-05 Final Closure Verification Ledger

## Readiness verification complete — 2026-10-10

This checkpoint supersedes the pre-repair readiness data below. The exact-range reviewers Sagan and Hegel rejected `ecaf362..b481e3f49593806dcef569da23fc9332ee02d15c` with Important shared-Vault authority findings. The local repair now adds a hashed canonical checkout owner to identified plans and ACTIVE rows, rejects stale local/Vault divergence and foreign pending-journal replay before writes, detects orphaned shared plans before creating duplicate identity, and permits exact-owner ACTIVE recovery. Generic reconciliation marks a plan indexed only when the validated ACTIVE row matches its path, binding, status, and generation.

Fresh local results: focused Core matrix 327/0 across 23 targets plus the required repeated harness-improvement run (340 executions); CLI selector/evidence/hooks matrix 38/0 plus migration CLI 3/3. All-targets: Core 641/0/0 across 60 suites, CLI 190/0/1 ignored across 38, adapters 124/0/3 ignored across 14, and the separate full workspace command exited 0. fmt, warnings-denied Clippy, locked release build, required release smoke 1/1, public trust docs 12/12, binary version 5.0.0, five help commands, and scope/JSON/diff/retirement checks pass. Default host lifecycle is 4/7 because Microsoft.PowerShell.Archive cannot load; supported bundled PowerShell 7 passes 7/7.

The fresh production re-audit covers all 21 requested search terms in Core, CLI, and adapters and classifies 48 caller paths: PRESENTATION_ONLY=11, LEGACY_SINGLE_ACTIVE_SAFE=8, EXACT_OPERATION_SCOPED=17, AMBIGUOUS_FAIL_CLOSED=9, OUT_OF_SPEC=3, BUG=0. The plan lifecycle groups now additionally enforce checkout ownership and local/shared mirror agreement; generic reconcile uses the exact validated ACTIVE row. Full raw counts and disposition are in the current authority audit ledger.

Local readiness is complete. Commit and push this snapshot, verify local/remote SHA equality and exact-SHA hosted CI, freeze `FINAL_REVIEW_HEAD`, then obtain two fresh independent read-only `ACCEPT` reviews of the identical complete `ecaf362..<FINAL_REVIEW_HEAD>` range. Until those reviews accept with zero unresolved Critical/Important findings, SPEC-05 is not closed. No release, tag, version bump, SPEC-06, Hotel Staff, manifest/lockfile change, or history rewrite.

## Latest exact-range review checkpoint — 2026-10-09

Current status: `SPEC-05: FIX REQUIRED`. The readiness snapshot `b481e3f49593806dcef569da23fc9332ee02d15c` is pushed and matches `origin/codex/spec-05-multi-agent-concurrency-durable-state`. Two new independent reviewers, Sagan and Hegel, both inspected exactly `ecaf362c275431edfc5e590fd6a87d926b9bde9d..b481e3f49593806dcef569da23fc9332ee02d15c`, made no edits, and returned `FIX REQUIRED` with Important findings. Sagan found that a stale same-identity plan copy in another shared-Vault checkout can overwrite newer mirrored progress. Hegel found that a shared Vault plan mirror with a missing ACTIVE row can permit a second plan authority for the same lifecycle identity. Hegel also reported a lower-severity observation that generic reconciliation may falsely reject a valid indexed ACTIVE plan; verify before deciding its disposition.

The b481e3f exact-SHA GitHub Actions run is `37905287345`: Format/Clippy and Linux native tests are `success`; Windows native tests are still `in_progress`. Previous local test counts and the 48-group `BUG=0` audit remain valid evidence for b481e3f itself, but not post-repair acceptance; they must be rerun after the finding repairs. Safe next action: add failing regressions for stale overwrite and orphaned Vault identity, verify the reconcile observation, then implement the smallest fail-closed root-cause repair. Proof/trace artifacts and user Vault data remain untouched. No release/tag/version change, SPEC-06, Hotel Staff, manifests/lockfiles, or history rewrite.

Date: 2026-10-06. Historical status at that checkpoint: `READY FOR ADVERSARIAL REVIEW`; that readiness was later invalidated by the hosted Linux failure recorded below. The document remains the detailed record for the 2026-10-06 runs.

## Historical continuation checkpoint — 2026-10-09 (superseded)

Current status: `SPEC-05: FIX REQUIRED`; no current-source readiness, closure, or acceptance claim. Two fresh reviews of `ecaf362..02779815c87bf2df6373085125857302503f2e05` returned `FIX REQUIRED`; a later read-only authority audit found two additional Important caller paths: Task State showed a frontmatter-only plan as current, and generic completion/reconciliation could pass with ACTIVE missing. That audit snapshot counted 48 caller paths with `BUG=2` and no Critical; it predates the latest repairs and must be rerun.

The four findings now have observed RED→GREEN regressions and local repairs. Focused current-source results: `authority_ingress` 6/6, `plan` 58/58, `operation_context` 18/18, `continuity` 7/7, `prepare` 8/8, `harness_improvement` 13/13, and `automation` 9/9. Indexed-only status is used by Task State, Prepare, and identity continuity/resume views; compatibility plan diagnostics retain frontmatter fallback. Generic completion status rejects identified plans without indexed ACTIVE. Same-identity cross-checkout plan start is rejected before plan/Vault publication and leaves the owning checkout resumable.

Still pending on this source: the full Step 2 focused regression matrix, Core/CLI/adapters/workspace all-target commands, formatter, warnings-denied Clippy, locked release build, release/trust/docs/scope gates, complete production authority audit, a new commit/push and exact-SHA GitHub CI, and two fresh independent ACCEPT reviews. Previously recorded 329/38 focused totals, 634/190/124/948 all-target totals, 47-group `BUG=0`, and CI runs `37889435584`/`37892777783` are pre-repair evidence only. An initial current-source Core all-target attempt without the required isolated session-root overrides was stopped before completion; it is not test evidence. Subsequent runs set unique nonexistent Codex/Claude session roots.

## Source and scope

- Branch: `codex/spec-05-multi-agent-concurrency-durable-state`.
- Starting HEAD and remote HEAD: `3cd03c6793807f2a149b3ed6f56dc43027640955`.
- Review base: `ecaf362c275431edfc5e590fd6a87d926b9bde9d`.
- Public binary target remains `baron 5.0.0`.
- Test isolation: `BARON_CODEX_SESSIONS_ROOT=C:\__baron_spec05_no_codex_sessions__`; `BARON_CLAUDE_SESSIONS_ROOT=C:\__baron_spec05_no_claude_sessions__`.
- No release, tag, version bump, SPEC-06, Hotel Staff change, or pushed-history rewrite is authorized or performed.

## Fresh authority audit

Reran every required authority-search term over `crates/baron-core/src/**`, `crates/baron-cli/src/**`, and `crates/baron-adapters/src/**`, excluding test modules, imports, comments, and function declarations from production hit review. The current inventory in [SPEC-05 authority audit](2026-10-01-spec-05-authority-audit.md) has 48 caller-path groups, each assigned exactly one category: `PRESENTATION_ONLY=11`, `LEGACY_SINGLE_ACTIVE_SAFE=8`, `EXACT_OPERATION_SCOPED=17`, `AMBIGUOUS_FAIL_CLOSED=9`, `OUT_OF_SPEC=3`, `BUG=0`. No current SPEC-05-owned Critical/Important issue is known from source tracing; independent reviews remain outstanding.

## Fresh focused regression matrix

All listed focused tests below ran against the current working-tree source with isolated session roots. Counts are test-case pass/fail counts (all observed failed counts are zero).

| Target | Passed / failed / ignored | Invocation |
| --- | ---: | --- |
| `plan` | 64 / 0 / 0 | Core focused matrix |
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
| `intent` | 8 / 0 / 0 | Core focused matrix, separate invocation |
| `intent_operation_cli` | 2 / 0 / 0 | CLI focused matrix |
| `control_plane` | 9 / 0 / 0 | Core focused matrix |
| `phase7_routing` | 20 / 0 / 0 | Core focused matrix |
| `phase10_adapter_authority` | 15 / 0 / 0 | Core focused matrix |
| `capability` | 9 / 0 / 0 | Core focused matrix |
| `review_gate` | 6 / 0 / 0 | Core focused matrix |
| `migration` | 23 / 0 / 0 | Core focused matrix |
| `harness_experiment` | 5 / 0 / 0 | Core focused matrix |
| `public_config_compat` | 1 / 0 / 0 | Core focused matrix |
| `session_replay` | 5 / 0 / 0 | Core focused matrix; UTF-8 boundary regression passes |
| `execution_cli` | 15 / 0 / 0 | CLI focused matrix |

The fresh Core prompt matrix totals 327 passed across 23 unique targets; its separate repeated `harness_improvement` invocation adds 13 more, for 340 executions. The required CLI selector/evidence/hook targets total 38 passes, plus migration CLI 3/3, excluding lifecycle baseline checks below.

The tracked retired-adapter gate (case-insensitive tracked-source search) exited 1 with no output, meaning no tracked current-tree matches.

## Baseline and lifecycle environment

- Default host `powershell.exe`: `cargo test -p baron-cli --test lifecycle_scripts -j 1 --no-fail-fast` exited 1 with 4 passed, 3 failed. All three failures were `Compress-Archive` failing to load `Microsoft.PowerShell.Archive`; this is an environment limitation, not counted as a product pass.
- Repository-supported bundled PowerShell 7 (`BARON_TEST_POWERSHELL=C:\Users\Ty\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe`): same target passed 7/7.
- `session_replay` current UTF-8 boundary test passed within the current-source focused run (5/5 suite).

## Full verification and delivery gates

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| Core all-targets | PASS, exit 0; 641 passed, 0 failed, 0 ignored across 60 suites |
| CLI all-targets (supported bundled PowerShell 7) | PASS, exit 0; 190 passed, 0 failed, 1 ignored across 38 suites |
| Adapters all-targets | PASS, exit 0; 124 passed, 0 failed, 3 historical ignored across 14 suites |
| Workspace all-targets | PASS, exit 0; 955 passed, 0 failed, 4 ignored across 112 suites (sum of the fresh Core/CLI/adapters package outputs) |
| Clippy `-D warnings` | PASS, exit 0 |
| Locked workspace release build | PASS, exit 0 |
| Release binary version / smoke | PASS; binary reports `baron 5.0.0`, required ignored smoke passes 1/1 |
| Public trust docs | PASS, 12/12 |
| `BARON_STATUS.json` parse | PASS |
| Changed Markdown relative links | PASS, 18 local relative links across 10 changed Markdown files, 0 broken |
| `git diff ecaf362 --check` | PASS, exit 0 (Git emitted only line-ending conversion warnings) |
| CLI help checks | PASS; root, proof record, trace record, trace score, continuity recover all exit 0 |
| GitHub Actions and commit statuses | PENDING for readiness snapshot |
| Readiness push and local/remote SHA equality | PENDING |
| Two fresh exact-range independent reviews | PENDING |
| Closure evidence push and final local/remote SHA equality | PENDING |

The first workspace attempt before removing tracked status wording exited 101 with 944 passed, 1 failed, and 4 ignored across 112 suites. The failing retired-reference test found the literal in the status JSON; the wording was corrected without changing the gate. Staging the two new evidence docs exposed the same condition: the existing test went RED because the untracked docs had included the literal in their examples. After correcting both docs, the targeted tracked-reference/filename tests passed 2/2 and public trust docs passed 12/12. The exact complete verification script then passed all package/workspace suites, Clippy, locked release, release smoke, docs, version, and CLI help. No Rust source changed after that complete run.

Prior reviews of `ecaf362..4a3efc0` are historical `FIX REQUIRED` reviews, not acceptance. This 2026-10-06 snapshot's readiness label is superseded by the newer hosted CI failure below. Closure requires two fresh independent `ACCEPT` verdicts for one identical pushed range and zero unresolved Critical/Important findings.

## Latest checkpoint — 2026-10-08

Current status: `SPEC-05: READY FOR ADVERSARIAL REVIEW`; not closed. The previously pushed readiness commit `c2b91dbfc4cedd593912013751c85f209755c9e1` failed hosted Baron CI run [37461605214](https://github.com/thienty1207/Baron-Engine/actions/runs/37461605214) on Linux native tests. The test-only repair is now pushed at `5c2bb5bda74363be3532a9f5516d26848379bada`.

The Linux job artifact traced the failures to test portability: four `execution_cli` fixtures and two `operation_evidence_cli` fixtures invoked `cmd /C exit 0`; one `receipt_authority` unit test treated a Windows drive path and extended Windows path as a Unix descendant. The committed test-only repair selects `cmd /C exit 0` on Windows and `sh -c 'exit 0'` elsewhere, and splits the Windows and native-path assertions by target OS. No product runtime behavior changed.

Fresh repaired-tree package gates on Windows, with isolated nonexistent Codex/Claude session roots and the bundled PowerShell 7 test host:

| Package | Passed / failed / ignored | Suites | Command result |
| --- | ---: | ---: | --- |
| Core | 631 / 0 / 0 | 60 | PASS, exit 0 |
| CLI | 190 / 0 / 1 | 38 | PASS, exit 0 |
| adapters | 124 / 0 / 3 | 14 | PASS, exit 0 |

Focused repaired-tree matrix: Core 313 passed/0 failed across 22 named targets; the prompt-required second `harness_improvement` run passed 13/13; CLI plan identity, operation evidence, hook, intent, and execution targets passed 38/38 across five targets. Full workspace all-targets passed 945/0/4 ignored across 112 suites. `cargo fmt --all -- --check`, warnings-denied Clippy, and locked release build exited 0. The ignored release binary smoke passed 1/1; the binary reports exactly `baron 5.0.0`; public trust docs passed 12/12; five CLI help checks exited 0; updated JSON parsed with `FIX REQUIRED`; 12 changed Markdown relative links resolved; tracked-source retirement gate found no hit; `git diff ecaf362 --check` passed.

Fresh lifecycle baseline: default Windows PowerShell exited 101 with 4 passed/3 failed because `Microsoft.PowerShell.Archive` could not load for `Compress-Archive`; the CLI package all-target run with the supported bundled PowerShell 7 passed this target 7/7. Current `session_replay` passed 5/5 within the Core package and workspace runs. These host facts are reported separately from product results.

The production authority inventory remains 45 groups (`PRESENTATION_ONLY=11`, `LEGACY_SINGLE_ACTIVE_SAFE=8`, `EXACT_OPERATION_SCOPED=17`, `AMBIGUOUS_FAIL_CLOSED=6`, `OUT_OF_SPEC=3`, `BUG=0`). The source diff is confined to CLI test fixtures and the Core `#[cfg(test)]` module; it does not alter production authority paths. Scope review confirms no release/tag/version bump, SPEC-06, Hotel Staff, or manifest/lockfile change. Hosted Baron CI run [37718795708](https://github.com/thienty1207/Baron-Engine/actions/runs/37718795708) on `5c2bb5b` completed successfully: Format/Clippy job `113121400969`, Windows native job `113121401034`, and Linux native job `113121401053` all passed. Both native jobs completed full tests, release build, and CLI version smoke.

Previous independent-review dispatches ended at the usage limit, so their result is `INCONCLUSIVE`; neither can count as ACCEPT. The previous readiness SHA is invalidated by its failed Linux job. Fresh local gates and hosted CI are green on code SHA `5c2bb5b`. The pushed readiness candidate must pass its own exact-SHA hosted workflow; if every job succeeds, freeze that SHA as `FINAL_REVIEW_HEAD` and obtain two fresh independent read-only reviews of `ecaf362..<FINAL_REVIEW_HEAD>`. Do not close without both ACCEPTs and zero unresolved Critical/Important findings.

## 2026-10-08 current rerun after the two fresh review findings

This section supersedes the prior readiness checkpoint above. Two fresh reviewers rejected the pushed `c50b3b9bb29e58878f7edf00bcea4f0adb1bb06f` snapshot with one Important each. Both findings have now been repaired with tests-first regressions:

- Intent regression: before the fix, identity-less high-risk intake accepted the shared confirmed intent written by operation B while operation A's own intent was unconfirmed. The current implementation rejects operation-scoped `CURRENT_INTENT` on identity-less intake while preserving unbound legacy intent; Core intent passes 8/8.
- Migration regression: before the fix, a user edit to `.baron/project.toml` during the unlocked installer callback could be adopted into rollback authority and removed by a later failure. Core rejects project/local config as declared installer outputs, the CLI excludes them, and rollback preserves conflicting user bytes. Core migration passes 23/23; CLI migration passes 3/3, including explicit rollback failing closed with `needs_recovery` while preserving configuration.

The fresh production scan covered all required authority patterns under `crates/baron-core/src/**`, `crates/baron-cli/src/**`, and `crates/baron-adapters/src/**`. It found two omitted paths in the old inventory; after the repairs they are recorded as A07/A08 `AMBIGUOUS_FAIL_CLOSED`. Current inventory is 47 groups: PRESENTATION_ONLY 11, LEGACY_SINGLE_ACTIVE_SAFE 8, EXACT_OPERATION_SCOPED 17, AMBIGUOUS_FAIL_CLOSED 8, OUT_OF_SPEC 3, BUG 0. The source hit count vector and new rows are recorded in [the authority audit](2026-10-01-spec-05-authority-audit.md).

Current fresh test evidence, all against the repaired tree and isolated nonexistent Codex/Claude session roots:

| Gate | Result |
| --- | --- |
| Prompt Core focused matrix | 329 passed executions (316 unique across named targets plus the required repeated harness-improvement target), 0 failed |
| Prompt CLI selector/evidence/hook matrix | 38 passed, 0 failed across five targets |
| Core all-targets | PASS, exit 0; 634 passed, 0 failed, 0 ignored across 60 suites |
| CLI all-targets | PASS, exit 0; 190 passed, 0 failed, 1 ignored across 38 suites |
| Adapters all-targets | PASS, exit 0; 124 passed, 0 failed, 3 ignored across 14 suites |
| Workspace all-targets | PASS, exit 0; 948 passed, 0 failed, 4 ignored across 112 suites |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo build --workspace --release --locked` | PASS, exit 0 |
| Required ignored release smoke | PASS, 1/1 |
| Public trust docs | PASS, 12/12 |
| Release version and help | PASS; `baron 5.0.0`; five required help selectors exit 0 |
| Default-host PowerShell lifecycle | 4/7; three failures are the host's `Compress-Archive` module load failure (`Microsoft.PowerShell.Archive`), not product assertions |
| Supported bundled PowerShell 7 lifecycle | PASS, 7/7 in the CLI all-targets run |
| Current SHA hosted CI | PENDING until this source/evidence snapshot is pushed |
| `BARON_STATUS.json` parse | PASS after current status synchronization |
| Changed Markdown relative links | PASS; 13 checked across six changed Markdown files, 0 broken |
| Tracked retirement source and filename gates | PASS; no tracked matches |
| `git diff ecaf362 --check` | PASS, exit 0; only Git LF-to-CRLF working-copy warnings |
| Scope inventory | PASS; 13 changed tracked files, no manifest/lockfile, release, Hotel Staff, or SPEC-06 files |
| Two fresh reviews of the new exact pushed range | PENDING; old verdicts are rejected and cannot count |

`cargo test --workspace --all-targets --no-fail-fast -j 1` exited 0 after running the full workspace. Post-edit scope gates pass; this snapshot remains `FIX REQUIRED` until commit/push, remote SHA equality, and exact-SHA hosted CI succeed. After that, publish `READY FOR ADVERSARIAL REVIEW` and obtain two fresh independent read-only ACCEPTs for the identical `ecaf362..<FINAL_REVIEW_HEAD>` range. Any Critical/Important resets readiness and requires a new full cycle. No release/tag/version bump, SPEC-06, Hotel Staff, or manifest/lockfile change is in scope.

## Latest repaired-source delivery checkpoint — 2026-10-09

This update supersedes the pending hosted-CI statement above for the repaired executable-source snapshot. Branch `codex/spec-05-multi-agent-concurrency-durable-state` is at `1d02d3272b196c96ae84354a3478117af97459b9`, and `git ls-remote` returned the same remote SHA. Exact-SHA Baron CI run [37732260773](https://github.com/thienty1207/Baron-Engine/actions/runs/37732260773) concluded `success` for Format/Clippy, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc` jobs. The current repaired source is the one covered by the recorded fresh local matrix and 47-group production authority audit (`BUG=0`).

The maintained status is now `READY FOR ADVERSARIAL REVIEW`, not closed. The prior exact-range reviewers of `ecaf362..c50b3b9bb29e58878f7edf00bcea4f0adb1bb06f` returned `FIX REQUIRED` and their findings have been repaired; those verdicts do not count. Push this synchronized readiness-doc snapshot and require its exact-SHA CI to pass before freezing `FINAL_REVIEW_HEAD`. Then obtain two fresh independent read-only `ACCEPT` verdicts for the same `ecaf362..<FINAL_REVIEW_HEAD>` range. If either reports Critical/Important, repeat the repair and full review cycle. No release, tag, version bump, SPEC-06, Hotel Staff, or manifest/lockfile change is authorized.

## Fresh review finding and repair checkpoint — 2026-10-09

The frozen review candidate was `a46ac3ea663d2d59d4a6d4cc0d28ca806d9d51cd` (`ecaf362..a46ac3e`). Gibbs returned `ACCEPT` with 23/23 checks passing and no Critical/Important findings. Bernoulli returned `FIX REQUIRED` with Important I-1, so neither earlier verdict is closure evidence after a repair.

I-1: identified frontmatter fallback could authorize proof, trace, or receipt-bound gate publication when the corresponding indexed ACTIVE row was missing. Resuming the same identity could restore ACTIVE while leaving that evidence reusable. The regression-first repair now validates the exact ACTIVE/frontmatter pair under the project mutation lock for all three evidence writers. A persisted `authority_generation` is written to plan metadata/index and evidence; recovery after lost index authority rotates that generation so older proof/trace/gate evidence cannot satisfy completion. Public Rust `ActivePlanAuthority`, `ProofRecord`, and `TraceOperationBinding` shapes remain unchanged. Completion audit uses the linked plan's exact risk/generation and requires the mirrored Vault trace even after plan status is completed.

Focused current-source evidence:

| Target | Result |
| --- | ---: |
| `authority_ingress` | 6/6 |
| `control_plane` | 9/9 |
| `plan` | 57/57 |
| `proof_trace` | 36/36 |
| `execution_cli` | 15/15 |
| `operation_evidence_cli` | 11/11 |
| `plan_identity_cli` | 4/4 |
| `receipt_authority` | 28/28 |
| `trusted_proof` | 1/1 |
| `cargo fmt --all -- --check` | PASS |

The first full workspace attempt used parallel compilation and failed before tests completed with Windows `os error 1450` (insufficient system resources). A serial `-j 1` attempt then completed but exposed four receipt-authority tests and one trusted-proof test whose fixtures published receipt-bound evidence without first creating the now-required active plan. Those fixtures were updated to seed the exact operation plan; the two focused targets then passed 29/29. A second serial workspace run is currently in progress with isolated Codex/Claude session roots and the supported bundled PowerShell 7 host. Do not treat the historical workspace counts or this in-progress run as current full-gate evidence.

Current disposition: `READY FOR ADVERSARIAL REVIEW`, not CLOSED. The post-edit local gates and fresh 47-group authority audit pass; the only remaining closure sequence is readiness commit/push, exact-SHA hosted CI, and two fresh independent ACCEPT reviews of one identical `ecaf362..<FINAL_REVIEW_HEAD>` range. Any Critical/Important finding restarts the full repair and review cycle. No release/tag/version bump, SPEC-06, Hotel Staff, manifest/lockfile change, or history rewrite is in scope.

## Fresh repaired-source verification checkpoint — 2026-10-09

The in-progress language above is superseded by the completed serial run. On the repaired working tree, fresh all-target results are Core `634 passed / 0 failed / 0 ignored`, CLI `190 / 0 / 1 ignored`, adapters `124 / 0 / 3 ignored`, and workspace `948 / 0 / 4 ignored` across 112 suites. The prompt's focused matrix passes 329 Core executions (316 unique target executions plus the required repeated `harness_improvement`) and 38 CLI executions; the retired-adapter target passes 8/8. Repair-specific current results: `authority_ingress` 6/6, `control_plane` 9/9, `plan` 57/57, `proof_trace` 36/36, `receipt_authority` 28/28, `trusted_proof` 1/1, and `session_replay` 5/5.

Fresh additional local gates pass: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo build --workspace --release --locked`; release binary reports exactly `baron 5.0.0`; the required ignored release smoke passes 1/1; public trust docs pass 12/12; five release CLI help selectors exit 0. Default-host PowerShell lifecycle remains 4/7 because `Microsoft.PowerShell.Archive` cannot load `Compress-Archive`; the repository-supported bundled PowerShell 7 lifecycle passes 7/7. This is an environment limitation, not counted as a product pass.

The fresh production source re-audit is recorded in [the authority audit ledger](2026-10-01-spec-05-authority-audit.md): 47 caller-path groups, category counts `11/8/17/8/3`, `BUG=0`; generation-aware readers refine existing exact-operation paths. The source/evidence snapshot remains uncommitted and has not been checked by exact-SHA hosted CI or the new final reviewers. All fresh local and post-edit documentation/scope gates now pass, so the snapshot is READY FOR ADVERSARIAL REVIEW, not closed. Next: publish the readiness snapshot, confirm local/remote SHA equality and exact-SHA hosted CI, then obtain two fresh independent read-only ACCEPT verdicts on the same `ecaf362..<FINAL_REVIEW_HEAD>` range. Until both accept with zero Critical/Important, SPEC-05 remains open. No release, tag, version bump, SPEC-06, Hotel Staff, manifest/lockfile change, or history rewrite is in scope.

## Post-edit readiness gates — 2026-10-09

The final evidence synchronization passed: `docs/BARON_STATUS.json` parses; 16 relative links across 10 changed Markdown files resolve; `phase1_target_red` passes 8/8 with no tracked retired-adapter reference/filename hits; `git diff ecaf362 --check` exits 0 (only Git LF→CRLF working-copy warnings); the complete `ecaf362..working-tree` inventory contains 70 files, all within the SPEC-05 source/test/docs scope, with no Cargo manifest/lockfile, release workflow, Hotel Staff, or SPEC-06 changes. The current docs/status/build checkpoint now says `READY FOR ADVERSARIAL REVIEW`, because every required fresh local gate and the 47-group `BUG=0` authority audit pass.

The source/evidence snapshot is not yet committed or pushed, so no final review SHA or hosted CI claim exists. Next: commit and push this readiness snapshot without rewriting history, confirm `git ls-remote` equals local HEAD, query exact-SHA GitHub Actions/commit status, and freeze that SHA as `FINAL_REVIEW_HEAD`. Then dispatch two new independent read-only reviews of exactly `ecaf362..<FINAL_REVIEW_HEAD>` with the full 23-point prompt checklist. Do not close unless both return `ACCEPT` with zero unresolved Critical/Important findings. No release, tag, version bump, SPEC-06, Hotel Staff, or manifest/lockfile change is in scope.

## Pushed readiness snapshot and exact-SHA CI — 2026-10-09

The authority-generation repair and its full local verification were committed and pushed as `752ea138d7092acda56edc832472e3bae78ec820` on `codex/spec-05-multi-agent-concurrency-durable-state`; local HEAD and `git ls-remote` match. Exact-SHA Baron CI run [37889435584](https://github.com/thienty1207/Baron-Engine/actions/runs/37889435584) completed successfully. Format/Clippy, Linux native tests, Windows native tests, both release builds, and both CLI version smoke steps passed. The final production audit remains 47 groups (`11/8/17/8/3`), `BUG=0`. The source commit is READY FOR ADVERSARIAL REVIEW, not closed. This ledger update must itself be pushed, its exact-SHA CI verified, and only then can the final review head be frozen for two fresh independent reviews of the identical `ecaf362..<FINAL_REVIEW_HEAD>` range. Earlier reviewer verdicts do not count. No release/tag/version bump, SPEC-06, Hotel Staff, manifest/lockfile change, or history rewrite is in scope.

## Fresh exact-range review findings — 2026-10-09

Two independent read-only reviewers completed the exact range `ecaf362c275431edfc5e590fd6a87d926b9bde9d..02779815c87bf2df6373085125857302503f2e05`. Both returned `FIX REQUIRED`, with no Critical findings and two unique Important findings:

1. When the repo ACTIVE index is missing, `proof_for_operation` calls the unfiltered tuple lookup, allowing an old-generation proof to appear in current operation Task State. The identity completion status path can also frontmatter-discover the plan and compare evidence against the same old generation; add explicit assertions for the interval before recovery/reindex.
2. Starting the same lifecycle identity from another checkout sharing the Vault can create a second checkout-local plan and then fail while merging the shared ACTIVE index. Review indicates the failed transition may remain in the journal and block future plan operations; reproduce this with a two-checkout regression, inspect on-disk state and recovery behavior, then fix the root cause.

The missing-ACTIVE case is already partly exercised by `authority_ingress`, but it asserts only failed ingress and evidence invalidation after generation rotation; it does not assert read/current Task State or completion while ACTIVE is absent. The shared-checkout plan tests cover different operation identities only. Until both regressions are watched fail, fixed, and rerun with the entire local matrix and refreshed 47-group authority audit, status is `FIX REQUIRED`. Candidate CI runs `37889435584` on `752ea13` and `37892777783` on `0277981` passed, but neither removes these review findings. Both old review verdicts are `FIX REQUIRED`, not ACCEPT. No release, tag, version bump, SPEC-06, Hotel Staff, manifest/lockfile change, or history rewrite is in scope.

## Complete post-repair verification — 2026-10-09

This is the fresh result after the local fixes for missing indexed-ACTIVE proof/current-plan/completion authority and same-identity shared-Vault cross-checkout partial writes. The tracked source is based on `02779815c87bf2df6373085125857302503f2e05` plus the current local repair; prior counts and CI on `0277981` do not certify this source.

| Gate | Fresh current-source result |
| --- | --- |
| Prompt Core focused matrix | 325 passed across 23 named targets; required second `harness_improvement` run 13/13; combined 338 executions, 0 failed/ignored |
| Prompt CLI selector/evidence/hook matrix | 38/38 across `execution_cli` 15, `intent_operation_cli` 2, `operation_evidence_cli` 11, `phase12_hooks_cli` 6, and `plan_identity_cli` 4; additional `migration_cli` 3/3 |
| Core all-targets | PASS, 635 passed / 0 failed / 0 ignored across 60 suites |
| CLI all-targets | PASS, 190 passed / 0 failed / 1 ignored across 38 suites |
| Adapters all-targets | PASS, 124 passed / 0 failed / 3 ignored across 14 suites |
| Workspace all-targets | PASS, 949 passed / 0 failed / 4 ignored across 112 suites |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo build --workspace --release --locked` | PASS, exit 0 |
| Release binary | PASS; reports exactly `baron 5.0.0` |
| Required ignored release smoke | PASS, 1/1 via `phase14_release` |
| Public trust docs | PASS, 12/12 |
| CLI help | PASS; root, proof record, trace record, trace score, continuity recover all exit 0 |
| Default-host lifecycle baseline | 4 passed / 3 failed, exit 101; all three fail because host `Microsoft.PowerShell.Archive` cannot load `Compress-Archive` |
| Supported bundled PowerShell 7 lifecycle | PASS, 7/7 in CLI/workspace all-target runs |
| `session_replay` UTF-8 regression | PASS, 5/5 in focused/Core/workspace runs |
| Retirement gate | PASS; `phase1_target_red` 8/8, no tracked retired-name or filename hits |
| Status JSON | PASS; parses |
| Changed Markdown relative links | PASS; 18 local relative links in 10 changed Markdown files, 0 broken |
| `git diff ecaf362 --check` | PASS, exit 0; only working-copy LF/CRLF warnings |
| Scope | PASS; 70 changed paths, no manifest/lockfile, release workflow, Hotel Staff, or SPEC-06 changes |
| Fresh authority audit | PASS; 48 groups, `PRESENTATION_ONLY=11`, `LEGACY_SINGLE_ACTIVE_SAFE=8`, `EXACT_OPERATION_SCOPED=17`, `AMBIGUOUS_FAIL_CLOSED=9`, `OUT_OF_SPEC=3`, `BUG=0` |
| Current-source hosted CI | PENDING until this readiness snapshot is committed and pushed |
| Fresh exact-range reviewers | PENDING; earlier FIX REQUIRED reports do not count as acceptance |

The audit details and raw search counts are in [the current authority ledger](2026-10-01-spec-05-authority-audit.md). Local verification is complete and status is `READY FOR ADVERSARIAL REVIEW`, not `CLOSED`. Next: commit and push this source/evidence snapshot, verify local/remote SHA equality and exact-SHA GitHub checks, freeze `FINAL_REVIEW_HEAD`, then obtain two fresh independent read-only `ACCEPT` verdicts for the same full `ecaf362..<FINAL_REVIEW_HEAD>` range. Do not close unless both accept with zero unresolved Critical/Important findings. No release, tag, version bump, SPEC-06, Hotel Staff, manifest/lockfile change, or history rewrite is in scope.
