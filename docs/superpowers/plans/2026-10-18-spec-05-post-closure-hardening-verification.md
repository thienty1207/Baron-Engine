# SPEC-05 Post-Closure Hardening Verification

## Status and boundary

- Status: in progress; H-01/H-02/H-03 source fixes and independent final-range review are complete. The full local workspace all-target suite, formatting, warnings-denied Clippy, locked release build, and release version smoke pass at source checkpoint `e452b62`. Final review found no Critical or Important findings and recorded two Minor notes. The review was scoped, not line-by-line; push, PR, and hosted checks remain pending.
- Branch: fix/spec-05-post-closure-hardening.
- Tested source checkpoint: e452b62a4aac08def36e51af1d559679d73bd1b3.
- SPEC-05 source tip used as the base: 1772124d7e49adf99775c4df62d706b7311a4a3f.
- Original accepted SPEC-05 closure remains ecaf362c275431edfc5e590fd6a87d926b9bde9d..af2e1db020355af4fe0f621e8de3f8f780fbd2ad; this follow-up does not reopen or rewrite that closure record.
- At tested source checkpoint `e452b62`, the range from origin/main was 94
  commits and 102 paths. The final reviewed checkpoint was
  `74c231269d2042e5bf0dc3d3a97f2ff9af04b972`, 95 commits ahead and 0 behind
  `origin/main`, still 102 paths. Subsequent commits update documentation only
  and preserve that path manifest; separate read-only review covered the status,
  compatibility, and next-action updates. The reviewer verified the reviewed
  commit list and changed-path manifest and performed detailed H-path and
  related authority review; this was not a line-by-line review.
- Public version remains 5.0.0. No release, tag, version bump, history rewrite, push, PR, or merge has occurred.

## Hardening findings

### H-01 — CONFIRMED BUG, fixed

The initial hypothesis was that ordinary identified-plan progress publication might lose recoverability across local/Vault/ACTIVE/CURRENT writes. A deterministic child-process test failed before the change because no durable transition intent existed. Progress publication now reuses the existing hash- and owner-checked plan transition journal; an identical immediate retry does not duplicate the last progress note. A second source review found that `start_or_resume_plan_internal` checked local/Vault mirror equality before calling the consistency helper that replays a pending journal. After an `after_local` crash, resume therefore failed with a stale-mirror error before recovery. A RED regression reproduced that error; moving the consistency/recovery call ahead of the redundant mirror check made it pass while retaining checkout/Vault lock order and the journal's owner/binding/hash validation.

Evidence: the exact recovery test passed across four subprocess interruption points, two retries on the update path, identified start/resume after the `after_local` boundary, separate operation A/B state, mirror equality, and no false completion. In the latest full workspace run, the Core library unit suite passed 82/82 and all integration targets passed, including plan 65/65 and phase12_hooks 16/16; the workspace command exited 0.

### H-02 — CONFIRMED BUG, fixed

The moved-checkout fence rejects a different canonical path and the relocation regression verified local/Vault authority bytes remain unchanged. Independent review found that Windows owner hashing lowercased canonical paths, conflating distinct checkout roots under a case-sensitive directory. A Windows regression enabled case sensitivity on its fixture and observed the old collision RED. The implementation detects a case-sensitive ancestor and derives a case-preserving, domain-separated v2 owner hash; ordinary case-insensitive paths keep the prior owner ID. The regression now distinguishes `Repo` and `repo`; companion tests preserve case-insensitive alias behavior and the existing legacy hash.

Recovery contract: restore a moved checkout at its original canonical path. Automatic owner reassignment is unsupported; intentional relocation requires a future explicit audited operation validating both local and shared identity/plan authority under checkout and Vault locks. Compatibility note: the fix changes the owner hash for Windows paths under a case-sensitive ancestor. A plan created before this fix stores the old case-folded v1 owner and now fails strict owner validation even when the checkout remains at the same path. Because v1 cannot distinguish case-only sibling roots, this behavior is fail-closed; recovery requires a future explicit audited migration.

Evidence: the moved-checkout test passed 1/1 in the plan target; Windows case-sensitive and case-insensitive owner tests passed 2/2.

### H-03 — CONFIRMED BUG, fixed

The prior cache trim could evict a live hook claim, and journal lookup silently skipped malformed JSONL rows. Cache updates now stage candidates, evict completed responses first and expired claims afterward, preserve non-expired/malformed/future claims, and reject all-live capacity overflow without mutating prior state. Every non-empty row in the latest 512 journal lines is parsed strictly, and replay evidence is validated before correlation or publication.

Replay guarantee is finite: a response can be recovered while retained in the latest 256 dedup entries or its event row remains among the latest 512 journal lines. After both windows expire, a delivery may execute again and add a journal row. Its operation-scoped continuity checkpoint remains immutable and deduplicated by event identity. Stop retries continue to reconcile current evidence. Independent review found that tail selection and append boundary detection read the full JSONL history. The reverse reader now reads bounded 8 KiB chunks up to 4 MiB, trims before UTF-8 decoding to the selected 512 lines, and fails closed if it cannot find the boundary. Append separator detection reads one byte.

Evidence: the pre-fix regressions demonstrated live-claim eviction, accepting a claim at all-live capacity, malformed rows being skipped, and correlation publication before malformed-journal rejection. The large-prefix bounded-read regression failed on invalid UTF-8 at byte 0 before the fix and passed afterward. Counted-reader tests prove tail reads do not consume the full history and append separator detection reads exactly one byte. Other RED/GREEN regressions cover cache/replay cases, response recovery after journal publication, the original 280-event retention stress, Stop replay after both windows expire across a child-process restart, and concurrent same-event delivery from two checkouts sharing one Vault.

## Source audit coverage

Read and compared relevant files in the requested Core, CLI, and adapter areas, with focused tracing of the changed call paths and their callers. Directly inspected the plan transition/recovery path and safe_io contract; config, identity, Vault context, operation identity, proof/trace authority, automation hook claim/replay, receipt authority, CLI command dispatch, adapter installation, matching Core/CLI/adapter integration tests, repository AGENTS.md, status/plans, Cargo manifests, and CI/release workflows. The first independent review covered the 93-commit history and 102-path manifest, with detailed review of H-01/H-02/H-03 paths and related authority/migration/CI areas; it was not line-by-line. It found no Critical issues and two Important findings (H-02 case-sensitive owner collision; H-03 unbounded journal reads), both fixed with RED/GREEN regressions. Final review verified HEAD `74c231269d2042e5bf0dc3d3a97f2ff9af04b972`, the 95-commit/102-path range, review package manifest, plan/spec, and AGENTS.md; no Critical or Important findings remain. It recorded two Minor notes: the diagnostic `automation_status` path reads the full journal, and pre-fix plans in case-sensitive Windows namespaces retain an ambiguous folded v1 owner ID and need an explicit audited recovery migration. This was a scoped final review, not a line-by-line review; the reviewer did not run tests.

Observed integration scope: origin/main is afc0b1b1137c0684c33e7e45a4c9b0e56d012446; origin/codex/spec-05-multi-agent-concurrency-durable-state is 1772124d7e49adf99775c4df62d706b7311a4a3f. At final review HEAD `74c231269d2042e5bf0dc3d3a97f2ff9af04b972`, the branch is 95 commits ahead, 0 behind, and changes 102 paths from origin/main. No Cargo.toml or Cargo.lock path differs from origin/main. Tracked path inspection found no .env, Baron project/local config, Vault/capsule data, user handoff document, or supplied untracked spec in the branch. The primary checkout remains on main with its user-owned untracked handoff and spec preserved.

## Local verification

- Focused Core verification: the library and plan, automation, phase12_hooks, hook_identity, and concurrency suites passed; 82 lib tests, plan 65/65, automation 9/9, concurrency 20/20, hook_identity 20/20, and phase12_hooks 16/16.
- Workspace: `cargo test --workspace --all-targets --no-fail-fast -j 1` — EXIT 0; all targets completed without test failures. The run used an isolated temporary BARON_HOME and bundled PowerShell 7.
- Formatting: `cargo fmt --all -- --check` — PASS.
- Clippy: `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- Release build: `cargo build --workspace --release --locked` — PASS.
- Release binary smoke: `baron --version` printed exactly `baron 5.0.0`.
- CLI help smoke: root help and proof record, trace record, trace score, and continuity recover help commands all exited 0.
- These verification commands passed at source checkpoint `e452b62`. The final documentation gates also pass: status JSON parses, 20 repository-relative links in six maintained Markdown files resolve, release CLI reports `baron 5.0.0`, both tracked retired-adapter content and filename searches return no matches, and `git diff --check` is clean.

### Local test-home caveat

The earlier full-workspace test invocation in the first implementation pass did not set BARON_HOME. Existing receipt-authority integration paths call the signing API, which can create machine-home authority state when no seed exists. A separate 32-byte machine authority seed is now present, but its pre-run existence cannot be established from the available evidence. Its value was not read, changed, removed, or staged. This seed is not a Project ID, project binding, Vault root, or capsule. All fresh Cargo test runs after that observation used a temporary BARON_HOME. Any further local Cargo test run must use an isolated BARON_HOME.

## Maintained documentation gates

- Passed: docs/BARON_STATUS.json parse; 20 repository-relative links in six maintained Markdown files; release version `5.0.0`; retired-adapter tracked content and filename gates; `git diff --check`. Staging/scope review confirms exactly six maintained status/plan/log files are staged; no machine data, environment file, Vault, generated output, or primary-checkout user file is staged.

## Recovery and persisted-state boundary

In the original checkout and checked recovery locations, .env, .baron/project.toml, .baron/local.toml, machine config.toml, the former Vault root, capsule, and Project ID were not found. A distinct machine-level receipt-authority seed exists as noted above; it cannot establish the original project identity or Vault. No baron init ran, and no replacement Project ID, Vault, or capsule was created. All H regression fixtures use temporary repositories and Vaults. No Baron proof or trace was produced against the original project identity.

The managed worktree and temporary fixtures contain the project changes. The earlier full-workspace test command may have created per-user authority state, and the available timestamp evidence cannot determine whether it predated that command. It remains untouched. The original checkout and its user-owned files remain unchanged.

## Remaining work

- Push the independently reviewed branch, create a PR into main, and verify required CI on the actual PR head SHA.
- Stop before merge until the user approves the reviewed integration scope. No merge or post-merge CI has occurred.
