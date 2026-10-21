# SPEC-05 Post-Closure Hardening Verification

## Status and boundary

- Status: in progress; local implementation is present. Formatting, Clippy, locked release build, and the H-01 recovery regression pass. The fresh workspace all-target command exited 1 because Windows Application Control blocked four adapter test executables before startup (OS error 4551); all Core/CLI targets and the adapter targets that ran passed. Whole-lineage independent review, push, PR, and hosted checks remain pending.
- Branch: fix/spec-05-post-closure-hardening.
- Tested source checkpoint: 06efdf69ea5d41d58044b3738ef6bfef45e33cb7.
- SPEC-05 source tip used as the base: 1772124d7e49adf99775c4df62d706b7311a4a3f.
- Original accepted SPEC-05 closure remains ecaf362c275431edfc5e590fd6a87d926b9bde9d..af2e1db020355af4fe0f621e8de3f8f780fbd2ad; this follow-up does not reopen or rewrite that closure record.
- At tested source checkpoint 06efdf69ea5d41d58044b3738ef6bfef45e33cb7, the range from origin/main was 91 commits and 102 paths: the accepted 82-commit/98-path lineage plus nine follow-up commits. Subsequent commits only update verification evidence. The branch remains 0 commits behind origin/main.
- Public version remains 5.0.0. No release, tag, version bump, history rewrite, push, PR, or merge has occurred.

## Hardening findings

### H-01 — CONFIRMED BUG, fixed

The initial hypothesis was that ordinary identified-plan progress publication might lose recoverability across local/Vault/ACTIVE/CURRENT writes. A deterministic child-process test failed before the change because no durable transition intent existed. Progress publication now reuses the existing hash- and owner-checked plan transition journal; an identical immediate retry does not duplicate the last progress note. A second source review found that `start_or_resume_plan_internal` checked local/Vault mirror equality before calling the consistency helper that replays a pending journal. After an `after_local` crash, resume therefore failed with a stale-mirror error before recovery. A RED regression reproduced that error; moving the consistency/recovery call ahead of the redundant mirror check made it pass while retaining checkout/Vault lock order and the journal's owner/binding/hash validation.

Evidence: the exact recovery test passed across four subprocess interruption points, two retries on the update path, identified start/resume after the `after_local` boundary, separate operation A/B state, mirror equality, and no false completion. In the fresh workspace run, Core's 77 unit tests and all Core integration targets ran with zero failures, including plan 65/65 and phase12_hooks 16/16. The workspace command as a whole exited 1 because four adapter executables were blocked before startup by Windows Application Control (OS error 4551); this is not recorded as a full workspace pass.

### H-02 — ALREADY FIXED, verified

The existing canonical checkout-owner fence rejects a moved checkout even when the temporary fixture retains the same Project ID and identity binding. The relocation regression verified that local plan/frontmatter, CURRENT, ACTIVE, INDEX, shared Vault plan, and Vault ACTIVE bytes were unchanged after rejection. The canonical alias unit test checks that aliases resolving to one canonical path retain one owner where supported.

Recovery contract: restore the checkout at the original canonical path. Automatic owner reassignment is unsupported; intentional relocation requires a future explicit audited operation validating both local and shared identity/plan authority under checkout and Vault locks.

Evidence: the exact moved-checkout regression passed 1/1 and is included in the 65-test plan target.

### H-03 — CONFIRMED BUG, fixed

The prior cache trim could evict a live hook claim, and journal lookup silently skipped malformed JSONL rows. Cache updates now stage candidates, evict completed responses first and expired claims afterward, preserve non-expired/malformed/future claims, and reject all-live capacity overflow without mutating prior state. Every non-empty row in the latest 512 journal lines is parsed strictly, and replay evidence is validated before correlation or publication.

Replay guarantee is finite: a response can be recovered while retained in the latest 256 dedup entries or its event row remains among the latest 512 journal lines. After both windows expire, a delivery may execute again and add a journal row. Its operation-scoped continuity checkpoint remains immutable and deduplicated by event identity. Stop retries continue to reconcile current evidence.

Evidence: the pre-fix regressions demonstrated live-claim eviction, accepting a claim at all-live capacity, malformed rows being skipped, and correlation publication before malformed-journal rejection. RED/GREEN regressions cover those cases, response recovery after journal publication, the original 280-event retention stress, Stop replay after both windows expire across a child-process restart, and concurrent same-event delivery from two checkouts sharing one Vault. The final phase12_hooks target passed 16/16.

## Source audit coverage

Read and compared relevant files in the requested Core, CLI, and adapter areas, with focused tracing of the changed call paths and their callers. Directly inspected the plan transition/recovery path and safe_io contract; config, identity, Vault context, operation identity, proof/trace authority, automation hook claim/replay, receipt authority, CLI command dispatch, adapter installation, matching Core/CLI/adapter integration tests, repository AGENTS.md, status/plans, Cargo manifests, and CI/release workflows. This is scoped implementation and integration review, not a claim that every line in every source file was exhaustively read. The first independent reviewer found the H-01 entry-point gap but its run stopped at the account usage limit; a fresh exact-range independent review remains pending.

Observed integration scope: origin/main is afc0b1b1137c0684c33e7e45a4c9b0e56d012446; origin/codex/spec-05-multi-agent-concurrency-durable-state is 1772124d7e49adf99775c4df62d706b7311a4a3f. The tested source checkpoint was 91 commits and 102 changed paths from origin/main; later commits only update verification evidence. No Cargo.toml or Cargo.lock path differs from origin/main. Tracked path inspection found no .env, Baron project/local config, Vault/capsule data, user handoff document, or supplied untracked spec in the branch. The primary checkout remains on main with its user-owned untracked handoff and spec preserved.

## Local verification

- Core: Core unit and all integration targets in the fresh workspace all-target run passed; the run used a temporary BARON_HOME and bundled PowerShell 7.
- CLI: all CLI unit and integration targets in the fresh workspace all-target run passed with the bundled PowerShell 7 override. Earlier default Windows PowerShell lifecycle failures were due to unavailable Microsoft.PowerShell.Archive; the supported host rerun passed.
- Adapters: adapter unit and most integration targets that started passed. `adapter_lifecycle`, `phase12_hooks`, `phase15_docs`, and `phase1_current` could not start: Windows Application Control returned OS error 4551.
- Workspace: `cargo test --workspace --all-targets --no-fail-fast -j 1` — EXIT 1 because of those four blocked adapter executables; no assertion failures were reported in targets that ran.
- Formatting: `cargo fmt --all -- --check` — PASS after the H-01 change.
- Clippy: `cargo clippy --workspace --all-targets -- -D warnings` — PASS after the H-01 change.
- Release build: `cargo build --workspace --release --locked` — PASS after the H-01 change.
- Release binary smoke: `baron --version` printed exactly `baron 5.0.0` after the release build.
- CLI help smoke: root help and proof record, trace record, trace score, and continuity recover help commands all exited 0.
- Status JSON parsed; repository-relative links in five follow-up Markdown files resolved; retired-adapter tracked-content and tracked-filename checks found no matches; git diff --check passed.

### Local test-home caveat

The earlier full-workspace test invocation in the first implementation pass did not set BARON_HOME. Existing receipt-authority integration paths call the signing API, which can create machine-home authority state when no seed exists. A separate 32-byte machine authority seed is now present, but its pre-run existence cannot be established from the available evidence. Its value was not read, changed, removed, or staged. This seed is not a Project ID, project binding, Vault root, or capsule. All fresh Cargo test runs after that observation used a temporary BARON_HOME. Any further local Cargo test run must use an isolated BARON_HOME.

## Maintained documentation gates

- docs/BARON_STATUS.json parses successfully; the original SPEC-05 phase stays closed while the separate follow-up is in progress.
- Repository-relative links in the maintained follow-up Markdown files resolve.
- The release version remains exactly 5.0.0.
- The retired-adapter content and tracked-filename gates return no matches.
- Staging is limited to explicitly reviewed branch files; no machine state, environment file, Vault, generated build output, or user-owned untracked handoff is staged.

## Recovery and persisted-state boundary

In the original checkout and checked recovery locations, .env, .baron/project.toml, .baron/local.toml, machine config.toml, the former Vault root, capsule, and Project ID were not found. A distinct machine-level receipt-authority seed exists as noted above; it cannot establish the original project identity or Vault. No baron init ran, and no replacement Project ID, Vault, or capsule was created. All H regression fixtures use temporary repositories and Vaults. No Baron proof or trace was produced against the original project identity.

The managed worktree and temporary fixtures contain the project changes. The earlier full-workspace test command may have created per-user authority state, and the available timestamp evidence cannot determine whether it predated that command. It remains untouched. The original checkout and its user-owned files remain unchanged.

## Remaining work

- Complete product-constraint checks and read-only review of the entire origin/main..HEAD lineage, including all 82 pre-existing SPEC-05 commits / 98 paths and follow-up files.
- Obtain independent whole-branch review of the exact final source range; resolve Critical/Important findings with RED/GREEN tests and record deferred Minor findings.
- Commit final evidence, push the reviewed branch, create a PR into main, and verify required CI on the actual PR head SHA.
- Stop before merge until the user approves the reviewed integration scope. No merge or post-merge CI has occurred.
