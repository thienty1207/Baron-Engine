# Baron Maintenance 01 — implementation verification

**State:** READY_FOR_REVIEW; not merged or released
**Date:** 2026-10-21
**Baseline:** main and origin/main at 9038afa9f840e3a62acc0f97c6aaa31849247525
**Branch:** codex/spec-05-maintenance-01-logic-integrity
**Worktree:** C:/Users/tytyb/.codex/worktrees/spec-05-maintenance-01-logic-integrity/Baron-Engine
**Source snapshot after code fixes:** 6333ec23e084b9b2d5a6f23c3d31be9c31aba99a
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
| LOG-04 | CONFIRMED | Corrupt import state was treated as empty and overwritten; synchronized importers lost one of two source updates. | Missing state alone defaults; import publication reloads under checkout then shared-Vault locks. Corrupt-state, union, retry, and session tests pass. |
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

Three additional bug groups were confirmed with RED/GREEN regressions:

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
3. **BUG_WITH_RED_REPRO — unreadable Harness CURRENT could lower trace risk.**
   current_harness_risk used Medium after any read error, allowing a high-risk
   legacy trace to bypass high-risk gates. Missing CURRENT retains the legacy
   Medium default; read errors conservatively return High. Regression:
   unreadable_current_harness_cannot_downgrade_legacy_trace_risk.

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
| cargo clippy --workspace --all-targets -- -D warnings | PASS, CLIPPY_EXIT_CODE=0 |
| cargo build --workspace --release --locked | PASS, RELEASE_BUILD_EXIT_CODE=0 |
| target/release/baron.exe --version | PASS, output: baron 5.0.0 |

These are local checks, not GitHub Actions results. Status JSON, relative
links, diff checks, retired-adapter search, and final scope inventory are
checked after the documentation commit. No PR exists yet, so exact-head hosted
CI is not available.

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

Unresolved Critical findings: none confirmed within LOG-01..LOG-06, G-01, or
the manually traced high-risk sibling paths. This is not a repository-wide
bug-free claim. Unresolved Important findings: none confirmed in the tested
scope. Known diagnostic and advisory limitations are listed above.

Version, release, tag, and lockfiles were not changed. No project/Vault
recovery identity was created or modified. The primary checkout and its
untracked user-provided inputs were preserved.

Independent review: pending. PR URL/state: not created. Hosted PR checks:
unavailable until a PR exists. Merge approval: no. Merged main SHA: none.

**Verdict:** READY_FOR_REVIEW
**Next action:** obtain independent read-only review of the complete
9038afa9-to-branch range; then push/open a scoped PR and check its exact head
if repository permissions allow. Stop before merge.
