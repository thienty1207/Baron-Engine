# SPEC-05 Post-Closure Hardening Verification

## Status and boundary

- Status: in progress; local hardening changes are implemented, complete workspace gates and whole-lineage review remain pending.
- Branch: `fix/spec-05-post-closure-hardening`.
- Source checkpoint: `6527d83e73634bae1e67388e06ca6769dc517edf`.
- SPEC-05 source tip used as the base: `1772124d7e49adf99775c4df62d706b7311a4a3f`.
- Original accepted SPEC-05 closure remains `ecaf362c275431edfc5e590fd6a87d926b9bde9d..af2e1db020355af4fe0f621e8de3f8f780fbd2ad`; this follow-up does not reopen or rewrite that closure record.
- Public version remains `5.0.0`. No release, tag, version bump, history rewrite, or merge has occurred.

## Hardening findings

### H-01 — CONFIRMED BUG, fixed

Ordinary identified-plan progress publication updated local plan, Vault mirror, ACTIVE records, and CURRENT without a durable transition intent. A child-process regression first failed at the interruption before local publication because no recoverable intent existed. Progress publication now reuses the existing hash- and owner-checked plan transition journal, and an identical immediate retry does not duplicate the last progress note.

Evidence: the exact recovery test passed across four subprocess interruption points, two retries, separate operation A/B state, mirror equality, and no false completion. Focused plan/concurrency tests passed 158/158.

### H-02 — ALREADY FIXED, verified

The existing canonical checkout-owner fence rejects a moved checkout even when the temporary fixture retains the same Project ID and identity binding. The relocation regression verified that local plan/frontmatter, CURRENT, ACTIVE, INDEX, shared Vault plan, and Vault ACTIVE bytes were unchanged after rejection. The canonical alias unit test checks that aliases resolving to one canonical path retain one owner where supported.

Recovery contract: restore the checkout at the original canonical path. Automatic owner reassignment is unsupported; intentional relocation requires a future explicit audited operation validating both local and shared identity/plan authority under checkout and Vault locks.

Evidence: `cargo test -p baron-core --test plan moving_checkout_fails_closed_without_changing_shared_plan_authority -- --exact --nocapture` passed 1/1.

### H-03 — CONFIRMED BUG, fixed

The prior cache trim could evict a live hook claim, and journal lookup silently skipped malformed JSONL rows. Cache updates now stage candidates, evict completed responses first and expired claims afterward, preserve non-expired/malformed/future claims, and reject an all-live capacity overflow without mutating prior state. Every non-empty row in the latest 512 journal lines is parsed strictly, and hook replay evidence is validated before correlation or publication.

Replay guarantee is finite: a response can be recovered while retained in the latest 256 dedup entries or its event row remains among the latest 512 journal lines. After both windows expire, a delivery may execute again and add a journal row. Its operation-scoped continuity checkpoint remains immutable and deduplicated by event identity. Stop retries continue to reconcile current evidence.

Evidence: real hook stress coverage replays the first delivery after 279 later distinct hook events and observes one event row and an unchanged checkpoint. A child-process replay after 513 later valid journal rows and an evicted cache observes the bounded replay behavior and the same single checkpoint. Red/green unit and integration regressions cover live-claim retention, capacity failure, expired-claim eviction, malformed/future-claim retention, malformed journal rejection before turn-map publication, and crash recovery from a journal-published response.

Focused verification: Core unit + automation + phase12 hooks + hook identity + concurrency completed with 140 passed, 0 failed across 5 suite summaries.

## Maintained documentation gates

- `docs/BARON_STATUS.json` parsed successfully with PowerShell `ConvertFrom-Json`; the original SPEC-05 phase remains `closed` while the separate follow-up is `in_progress`.
- Repository-relative Markdown links in five changed follow-up Markdown files resolved with zero broken targets.
- `cargo run --locked -p baron-cli -- --version` completed and printed exactly `baron 5.0.0`.
- Retired-adapter content grep (`git grep -in reasonix`) and tracked-filename search returned no matches.
- `git diff --check` is run on the staged evidence before the Task 4 commit.

## Recovery and persisted-state boundary

In the original checkout, `.env`, `.baron/project.toml`, `.baron/local.toml`, machine configuration, Vault root, capsule, and Project ID were not found in the checked recovery locations. No `baron init` ran, and no replacement Project ID, Vault, or capsule was created. All regression tests use temporary identities, repos, and Vaults. No Baron proof or trace was produced against the original project identity.

Changes and generated test fixtures are confined to the managed worktree and temporary directories. The original checkout and its user-owned files remain untouched.

## Verification still pending

- `cargo fmt --all -- --check` and complete workspace Core/CLI/adapter/all-target tests after the final source state.
- Warnings-denied workspace Clippy, locked release build, CLI version/help, status JSON, relative Markdown links, diff check, and retired-adapter gate.
- Inspection of the full SPEC-05 integration lineage against `origin/main`, including the 82 pre-existing commits and 98 paths, followed by an independent read-only whole-branch review.
- Push, PR creation, and hosted checks on the actual PR head. Stop before merge until the user approves the reviewed integration scope.
