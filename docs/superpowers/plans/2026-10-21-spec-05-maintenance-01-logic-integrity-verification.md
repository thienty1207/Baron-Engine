# Baron Maintenance 01 — implementation verification

**State:** Source/evidence and status-document updates pushed to PR #2; latest-head CI and independent review pending
**Date:** 2026-10-22
**Baseline:** main and origin/main at 9038afa9f840e3a62acc0f97c6aaa31849247525
**Branch:** codex/spec-05-maintenance-01-logic-integrity
**Worktree:** C:/Users/tytyb/.codex/worktrees/spec-05-maintenance-01-logic-integrity/Baron-Engine
**Source snapshot after PR #2 review fixes:** 8ea372bfc5e7ecef961fb9eefce92e2219445d1b
**Version:** 5.0.0

This maintenance is separate from the historical SPEC-05 closure. The source
snapshot contains the fixes and tests; the current branch HEAD will advance
with this evidence and status documentation. No merge approval has been
requested or received.

## Finding results

| Finding | Classification | RED evidence | Fix and GREEN evidence |
|---|---|---|---|
| LOG-01 | CONFIRMED | Stop hook malformed JSON and corrupt journal returned a soft continue response. | Typed failure disposition defaults to blocking; only the documented malformed UserPromptSubmit payload remains advisory. CLI hook suite: 8/8. |
| LOG-02 | CONFIRMED | Six public mutators could replace unreadable existing indexes or managed documents with defaults. | Strict missing-only reads and trace index preflight preserve existing bytes. Six targeted regressions pass, including the sibling harness story index. |
| LOG-03 | CONFIRMED | Invalid UTF-8 in a discovered Rust source was treated as empty input; rebuild succeeded and changed the graph cache. | Source read errors propagate before graph/cache publication. Cache-preservation regression passes; intentional source deletion remains supported. |
| LOG-04 | CONFIRMED | Corrupt import state was treated as empty and overwritten; synchronized importers lost one of two source updates. The initial review also found that retry after note publication could overwrite a user edit before state publication. | Missing state alone defaults; import publication reloads under checkout then shared-Vault locks. Notes are create-only on first publication; retry adopts an existing note only when its frontmatter matches the imported-session type, adapter, and source hash, preserving edited bytes. Corrupt-state, union, retry, session, and interruption regressions pass. |
| LOG-05 | CONFIRMED | Invalid/missing/unknown/duplicate finding status and malformed findings paths could appear closed or disappear from context. | Strict enumeration, path, UTF-8, and status validation returns an unavailable/error result. Review-gate 9/9 and context 2/2 focused tests pass. |
| LOG-06 | CONFIRMED | Status and Autopilot readers held project and Vault locks while reading the full journal, skipped malformed rows, and conflated missing/empty/corrupt inputs. | Scanner captures a regular-file length under the ordered locks, then validates the exact snapshot outside them with bounded buffers, a 1 MiB row cap, exact totals/latest event, and capped diagnostic kinds. Automation 14/14, Autopilot 6/6, scanner unit 4/4, phase12_hooks 16/16. |
| G-01 | TEST GAP CLOSED — NO SOURCE BUG | No deterministic interruption test existed after CURRENT publication and before pending-journal cleanup. | Test-only failpoint and A/B fixture prove repeated subprocess recovery applies A once, preserves B byte-for-byte, aligns projections, and clears the journal. Plan unit 5/5 and integration 67/67. |

## Expanded sibling-caller audit

The production Rust trees under crates/baron-core/src, crates/baron-cli/src,
and crates/baron-adapters/src were searched for the spec's fallback, ignored
read, JSON parsing, CURRENT authority, journal, and multi-file publication
patterns. High-risk callers were then traced through their callers and state
publication paths. This was a lexical whole-tree sweep plus targeted semantic
review. It was not a manual line-by-line review of every match and is not a
claim that every possible defect was found.

Three additional sibling bug groups and one LOG-04 crash-retry subissue were
confirmed with RED/GREEN regressions:

1. **BUG_WITH_RED_REPRO — shared automation journal readers.**
   harness_improvement::audit_harness read the whole journal under both locks,
   substring-matched event names, and treated read errors as empty. The
   continuity packet reader skipped malformed rows and could choose a stale
   event. Both now use the strict bounded scanner; harness audit distinguishes
   missing and empty, and continuity propagates corrupt input before writing a
   packet. Regressions: harness_audit_rejects_a_corrupt_automation_journal,
   harness_audit_distinguishes_missing_from_empty_automation_journal, and
   continuity_checkpoint_rejects_a_corrupt_automation_journal.
2. **BUG_WITH_RED_REPRO — unreadable explicit intent could permit approval.**
   intent_status used read failure as “no intent”, while Autopilot ignored
   intent errors during approval. Strict text reading now propagates through
   the authority check. Regression:
   approval_fails_closed_when_current_intent_is_unreadable; the candidate
   remains unapproved and no decision file is written.
3. **BUG_WITH_RED_REPRO — untrusted Harness CURRENT could lower trace risk.**
   current_harness_risk previously returned Medium after read errors and for
   readable files with missing or unknown risk, allowing a high-risk legacy
   trace to bypass high-risk gates. Missing CURRENT retains the legacy Medium
   default; unreadable, missing-risk, unknown-risk, duplicate-risk, and other
   malformed present files return High. Exactly one explicit `high`, `medium`,
   or `low` field is accepted. Regression
   untrusted_current_harness_cannot_downgrade_legacy_trace_risk covers those
   malformed cases and requires a failing high-risk trace.
4. **BUG_WITH_RED_REPRO — session-note retry could erase user edits.** LOG-04
   published a deterministic note before import state; an interruption at
   that boundary left the note eligible for replacement on retry. The initial
   review reproduced loss of an edit made before retry. Imported notes now use
   create-only publication and an existing note is adopted only when its
   frontmatter identity matches the source. Regression
   session::tests::retry_after_interruption_preserves_an_edited_imported_note
   injects an interruption after note publication, edits the note, retries,
   and asserts byte preservation and state recovery.

Other traced candidates and dispositions:

- **CORRECT:** authoritative receipt/proof callers use strict verified
  loaders; diagnostic-only receipt enumeration may drop invalid rows without
  changing gate authority.
- **DIAGNOSTIC_ONLY:** CURRENT/status and optional-context readers can omit
  unreadable diagnostic projections. Identified plan, proof, and trace
  authority comes from validated indexed operation state rather than CURRENT.
  This may reduce diagnostic visibility; it did not authorize a traced write
  or passing gate.
- **SAFE_MISSING_DEFAULT:** callers using safe_io::read_text(path)? and then
  defaulting only on None preserve the distinction between absent files and
  read/UTF-8 errors.
- **CONTRACT_LIMITATION:** CodeGraph source selection/cache fingerprinting and
  advisory call-edge extraction can omit metadata or edge details on transient
  filesystem errors. Graph source reads and cache publication are strict; the
  inspected omission paths do not establish proof or execution authority.
  A deterministic race seam was not added for these advisory paths.
- **OUT_OF_SCOPE:** test-only Result::ok/filter_map cleanup helpers and
  optional presentation fields that do not authorize, complete, or publish
  durable state.

Residual visibility limits: continuity's diagnostic status/read-optional
projections may show an unreadable CURRENT file as absent; CodeGraph's
advisory metadata and edge extraction can be incomplete under a transient
filesystem race. These remain bounded to diagnostics/advisory data in the
traced callers. The broader lexical candidate set was not exhaustively
reviewed line by line.

## Verification evidence

Environment: Windows; rustc 1.99.0; cargo 1.99.0; bundled PowerShell 7.6.5
selected by BARON_TEST_POWERSHELL. BARON_HOME, Codex session root, and Claude
session root were set beneath the temporary directory
baron-maintenance01-verify-20261021. No secret environment values were printed.

| Command | Result |
|---|---|
| cargo fmt --all -- --check | PASS |
| cargo test -p baron-core --all-targets --no-fail-fast -j 1 | PASS, CORE_EXIT_CODE=0 |
| cargo test -p baron-cli --all-targets --no-fail-fast -j 1 | PASS, CLI_EXIT_CODE=0 |
| cargo test -p baron-adapters --all-targets --no-fail-fast -j 1 | PASS, ADAPTERS_EXIT_CODE=0 |
| cargo test --workspace --all-targets --no-fail-fast -j 1 | PASS, WORKSPACE_EXIT_CODE=0 |
| cargo test -p baron-core --lib retry_after_interruption_preserves_an_edited_imported_note -- --nocapture | PASS, 1/1; observed RED before create-only note publication |
| cargo test -p baron-core --test proof_trace untrusted_current_harness_cannot_downgrade_legacy_trace_risk -- --exact --nocapture | PASS, 1/1; unreadable, unknown, missing, and duplicate risk cases |
| cargo clippy --workspace --all-targets -- -D warnings | PASS, CLIPPY_EXIT_CODE=0 |
| cargo build --workspace --release --locked | PASS, RELEASE_BUILD_EXIT_CODE=0 |
| target/release/baron.exe --version | PASS, output: baron 5.0.0 |

These are local checks, not GitHub Actions results. After the status
documentation commit, `docs/BARON_STATUS.json` parsed successfully, all 26
repository-relative Markdown links checked had valid targets, and
`git diff --check` passed. The documentation delta contains only the six
status/evidence files; no source changed at that historical checkpoint. At
that time the PR had not yet been created. Current PR #2 and the later R01-R03
follow-up are recorded at the end of this ledger.

## Source history and handoff

Implementation commits from baseline:

- 612231e61a33e3e7d577f64c760773344312b775 — fail closed on unknown hook failures
- a3edb0055b94b0807bcf6e9fd995eb9394e1085d — preserve managed data on read failures
- 1dcf23b28f73b53f0ee667ac117f26cae81deb31 — reject unreadable CodeGraph sources
- 14b2832dffee91fe2ccc9469086c74e85021134d — serialize and preserve session imports
- a29c0897ee3f3e30f15a70a02df5a17b49049524 — fail closed on incomplete review status
- 75561b330a5c22bc5ce845deeb982252012d6bca — bound automation journal diagnostics
- 1b2098fe0c5d140d8ed71aa0859db6dc377aae1b — cover recovery after CURRENT publication
- 6333ec23e084b9b2d5a6f23c3d31be9c31aba99a — close verified sibling fail-open paths
- 60cce4b421499a4e7c9ee3c9ced6a7e7f233c485 — fail closed on malformed Harness CURRENT risk
- c502fa7e21ffd79e429856d66a483760593b7a23 — preserve imported notes during recovery

The initial independent review returned FIX REQUIRED with two Important
findings: readable malformed Harness CURRENT risk and session-note overwrite
on retry. Both are addressed by the commits above and the added regressions.
The updated independent review ACCEPTED the complete range
`9038afa9f840e3a62acc0f97c6aaa31849247525..3005f5f67a352d62bf60aa1ddb166eead90b5222`
with no Critical or Important findings. A subsequent read-only review of the
documentation-only commit `b6f09de56fb8cc48dab7cb1d2b3117b576ffdfd8` found two
stale statements in this ledger; this update corrects them. The source snapshot
remains unchanged. Known diagnostic and advisory limitations are listed above.
This is not a repository-wide bug-free claim.

Version, release, tag, and lockfiles were not changed. No project/Vault
recovery identity was created or modified. The primary checkout and its
untracked user-provided inputs were preserved.

Initial independent review: FIX REQUIRED; two Important findings fixed.
Updated independent review: ACCEPT for the complete prior source/evidence
range
`9038afa9f840e3a62acc0f97c6aaa31849247525..3005f5f67a352d62bf60aa1ddb166eead90b5222`;
no Critical or Important findings. The reviewer made no edits and did not run
tests. Follow-up review of the docs-only `b6f09de` commit found only stale
statements; that review predates the PR #2 R01-R03 follow-up recorded below.
PR #2 URL: https://github.com/thienty1207/Baron-Engine/pull/2. Merge
approval: no. Merged main SHA: none.

**Earlier verdict:** REVIEW_ACCEPTED for the prior range above. It does not
cover the R01-R03 follow-up.
**Next action:** verify hosted checks on the latest PR #2 head and await
independent review of R01-R03. Stop before merge.

## PR #2 independent-review follow-up — 2026-10-22

The user's review file is
`docs/specs/BARON-PR2-INDEPENDENT-REVIEW-2026-10-22.md` in the primary checkout
and remains untracked there. It described R01/R02 as confirmed bugs requiring
regression evidence and R03 as a likely failure path that needed deterministic
interruption before being classified. All three were reproduced in the
isolated worktree, fixed, and covered by tests.

| Finding | Classification | RED evidence | Fix and GREEN evidence |
|---|---|---|---|
| R01 — session source binding and path identity | CONFIRMED BUG | A user message exactly equal to the repository path caused the unrelated session to be imported; the regression expected one import and observed two. Case-folded source keys also merged distinct directories under an NTFS case-sensitive parent. | Import binding now reads explicit cwd/repo metadata in recognized context records and absolute project paths only; message text and project labels cannot bind a repo. Source identity uses canonical case-preserving paths. A legacy folded key migrates only when the single current source, content hash, and destination note identity match. RED/GREEN tests cover spoofing, case-distinct paths, edited-note preservation, and migration. |
| R02 — review finding closure status | CONFIRMED BUG | A closed-status line in Summary made the old substring check return success while the authoritative header remained open. | Status parsing requires exactly one exact open/closed header field before `## Summary`, and closure updates replace only that field. A regression also verifies retry repairs a Vault-closed/repository-open interruption without replacing original closure evidence. |
| R03 — trace score multi-file recovery | CONFIRMED BUG | A public Windows test held the Vault trace mirror with a sharing mode that allowed preflight read but denied replacement. The first call failed after the repository trace write; before the fix, retry failed because the mirrors diverged. | A bounded project/Vault-bound journal stores previous and target bytes for both trace mirrors and both indexes. Recovery validates all targets before writing, completes missing publications idempotently, and runs before mirror preflight. Private writer-failure injection and public Windows interruption/retry tests pass, clear the journal, preserve unrelated trace/index content, and refuse to overwrite changed targets. |

### Follow-up verification

Source/test commit: `8ea372bfc5e7ecef961fb9eefce92e2219445d1b`.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --workspace --all-targets --no-fail-fast -j 1` | PASS, exit 0; no failed tests. Historical ignored tests remained ignored. Temporary BARON_HOME and Codex/Claude session roots were used. |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo build --workspace --locked --release` | PASS |
| `target/release/baron.exe --version` | PASS, `baron 5.0.0` |
| `cargo test -p baron-core --lib session::tests::case_distinct_session_sources_keep_separate_import_state_keys -- --exact` | PASS, 1/1 after final test-helper cleanup |
| `git diff --check` | PASS after source, tests, and status documentation updates |
| `docs/BARON_STATUS.json` parsing | PASS |
| Repository-relative links in the five changed Markdown files | PASS, 26 checked and none broken |
| `target/release/baron.exe --help` | PASS; normal user commands and adapter-specific shortcuts display |
| Tracked retired-adapter content and filename searches | PASS, zero references or filenames |

The workspace suite ran before the final test-helper-only lint cleanup. After
that cleanup, the exact case-sensitive session test was rerun; formatting,
warnings-denied Clippy, and the locked release build also passed on the cleaned
source.

Source/evidence head `b9cbbecb0ede08516590ff0d784693936e5cb79d` and subsequent
status-document head `7ceb9a3352660d38bb4d15784d5a2068bcb2e40d` were pushed to
PR #2. Baron CI run `39704008984` was queued for the latter; the final
status-document checkpoint still requires exact-head verification. The
workspace all-target run included Core (91 unit tests), `proof_trace`
(39), `review_gate` (11), `session` (6), and every remaining workspace target;
each completed target reported zero failures. The R03 public interruption test
ran on Windows. These are local checks. Exact-head GitHub Actions results and
independent review of the updated PR head remain pending. No release, version
bump, tag, merge, or project/Vault mutation was performed.
