# SPEC-05 zero-CURRENT closure repair

Date: 2026-10-01. Base: ecaf362. Starting HEAD: 9bdb979.
Binding requirement: user-supplied `SPEC-05-FINAL-CLOSURE-REPAIR-ZERO-CURRENT-HOOK-IDENTITY.md` in the primary checkout's `fix-bug/prompt/`.

## Constraints and interfaces

Use the existing SPEC-05 worktree/branch. Preserve unrelated files and legacy single-active behavior when safe. No public version bump, release, tag, SPEC-06 framework, or Hotel Staff edits. Exact full identity and validated ACTIVE/frontmatter are correctness authority; CURRENT is presentation. Durable correlation uses the existing project lock with slow work outside it.

CLI and hook repairs share existing LifecycleIdentity/ACTIVE APIs but have disjoint file ownership. The main implementer owns CLI/plan APIs and docs. A focused worker owns automation hook normalization/correlation and hook regressions. A read-only authority auditor inventories remaining callers. No new unvalidated generic persistence or host field assumptions.

## Task 1: CLI ingress (main)

Write exact A-vs-CURRENT-B proof, receipt, trace and scoring regressions plus no-selector ambiguity/no-mutation tests. Observe RED. Add complete optional task/adapter/session/request selectors using the canonical identity helper; validate exact ACTIVE binding. Receipt identity selects independently of CURRENT. No-selector selects only a validated single active operation; ambiguity fails before evidence writes. Observe GREEN focused CLI/Core suites.

## Task 2: host correlation (focused worker)

Audit official Codex and Claude hook schemas independently. Write host-shaped Prompt/Stop, retry, restart, unknown/ambiguous/malformed/duplicate/stale/capacity regressions; observe RED. Persist bounded correlation with canonical task and full identity under the project lock; validate project/adapter/session/request, ACTIVE and frontmatter on correctness paths. Session-only resolution is permitted only when unique, never newest/CURRENT/guessing. Observe GREEN hook suites.

## Task 3: complete authority inventory (read-only audit, fixes main)

Search every CURRENT/active-plan/current-risk/latest-proof/latest-trace/reconcile caller. Record each as PRESENTATION_ONLY, LEGACY_SINGLE_ACTIVE_SAFE, EXACT_OPERATION_SCOPED, AMBIGUOUS_FAIL_CLOSED, BUG, or OUT_OF_SPEC. Fix owned BUG callers via RED-to-GREEN. Preserve all prior authority/concurrency fixes.

Treat confirmed intent as operation authority too: add an explicit exact-identity producer and durable Repo/Vault per-operation record; keep `CURRENT_INTENT.md` as a latest projection only. Prepare and Task State must read the exact record, fail closed on malformed/mismatched/oversized state, and never accept an unbound CURRENT intent for an identified operation. Keep the legacy intent writer and legacy intake behavior intact.

The UTF-8 boundary panic in replay formatting is a known baseline manifestation that can interrupt `prepare_cli` when matching host history is imported. Reproduce with a synthetic multibyte fixture; fix the byte-boundary truncation without reading real user session trees during routine deterministic tests.

## Task 4: integrated verification and evidence

Run fmt; Core, CLI, adapters and workspace all-targets with -j 1/no-fail-fast; warnings-denied workspace Clippy; locked workspace release build; all prompt focused suites. Parse status JSON, check maintained links and diff ecaf362..HEAD, binary version. Reproduce any claimed baseline exceptions at ecaf362 and final source, investigating changed causes/counts. Inspect actual GitHub status/Actions evidence; never equate absence with success.

## Task 5: independent review and delivery

Commit source/tests and final evidence/docs before independent read-only review of ecaf362..actual HEAD, including every new document. Resolve Critical/Important findings with regressions and fresh verification; re-review if required by the binding prompt. READY only after audit/verification; CLOSED only with independent acceptance. Push scoped branch, verify clean local state and exact remote HEAD. Report limitations honestly.

## Execution ledger

- Start: FIX REQUIRED. At `0536a27`, two reviewers returned one ACCEPT and one FIX REQUIRED; the latter found cached Stop replay could return an old pass after same-operation trace evidence changed. The older full verification does not cover the current fix.
- Ruling: latest user repair prompt overrides the older inline skill's no-re-review restriction if review fixes change final HEAD; final acceptance must include the delivered source/docs.
- Review checkpoint: fresh reviews of `ecaf362..af65881` found three Important issues: completed correlated Stop blocked the host; new public `unknown_fields` members broke external config struct literals; shared Vault TEST_MATRIX upserts used checkout-local locks only. The repair adds focused regressions for each; two fresh final-range reviews remain pending.
- RED evidence: `public_config_compat` initially failed compilation with three missing `unknown_fields` fields; exact completed Stop returned `decision:block`; a child in a second checkout updated the shared matrix while the Vault capsule lock was held.
- GREEN evidence: `hook_identity` 18/18, `concurrency` 16/16, `config` 16/16, `plan` 51/51, `proof_trace` 29/29, `public_config_compat` 1/1; completed Stop mismatch-frontmatter rejection passes; config unknown root/nested/local TOML values survive init and setters. Workspace all-targets, focused Core/CLI/adapter suites, format, Clippy, locked release build, and explicit release smoke pass.
- Ruling: keep the original public config struct field surface and preserve unknown keys by merging from the existing TOML document during managed writes; adding `#[non_exhaustive]` or hidden state would further break source compatibility or be unsafe.
- Ruling: shared harness mutations acquire checkout lock before capsule lock; these call paths have no reverse-order acquisition, and timeout occurs before a shared matrix write. Lock ordering is documented in source and covered by a cross-process held-lock regression.
- Review Minor disposition: retain for fresh adversarial reassessment. Bounded journal response parsing reads the whole journal before scanning 512 lines; trace scoring walks the managed trace archive under the project lock; dedup capacity trimming may evict an in-flight claim, while token fencing prevents stale publication. These are not known Critical/Important defects; document a reasoned acceptance or fix if a fresh reviewer finds impact above Minor.
- Follow-up review checkpoint: fresh independent reviews of `ecaf362..fcc6b79` found Important gaps: session-only Stop missed a validated legacy active plan without an ACTIVE row; Stop could publish a stale pass after newer same-operation evidence appeared. Reviewer two confirmed the legacy gap; reviewer one independently found both. These findings keep status at FIX REQUIRED.
- Follow-up RED-to-GREEN: the unindexed legacy plan regression first returned `continue:true` for completed A while active B existed, then passed after session checks scan all validated identified managed plans. The synchronized publication-race test first published `reconciliation_passed:true` after a newer failing trace, then passed after final Stop response reconciliation was recomputed under the publication lock.
- Latest review finding: after a passing Stop response was published, a newer failing trace followed by identical Stop delivery returned cached `continue:true/reconciliation_passed:true`. Dedup and journal response fast paths bypassed the publication-time freshness check. One fresh reviewer identified this Important defect; another accepted only the pre-fix range.
- Latest RED-to-GREEN: `stop_replay_reconciles_latest_operation_evidence_after_cached_pass` observed the stale pass before the fix. It now verifies a fresh block with dedup still present and again after removing dedup so the old passing journal response is the only cached source. Targeted test passes 1/1.
- Latest source change: active identified Stop deliveries bypass old dedup/journal response reuse, preserve live-claim fencing, recompute exact-operation reconciliation under the publication lock, and refresh the durable dedup response. Journal Stop responses are historical and never authorize retries.
- Final source/test commit: `c68e01d14cd5d6574b1574820bef8ee8843a3c5b`; the evidence/status commit follows it and must be included in the independent review range.
- Current verification status: Core 559/0 across 60 result suites; CLI 188/0 across 38 (one ignored release-only test); adapters 124/0 across 14 (three ignored); workspace 871/0 across 112 (four ignored). Fmt, warnings-denied Clippy, locked release build, explicit ignored release smoke, `baron 5.0.0`, JSON/relative-link/retired-adapter checks, and `git diff ecaf362 --check` pass. Two fresh independent reviews of the final committed range remain.
- Historical package verification at `0536a27` (before cached Stop replay fix): Core 558/0 across 60 suites; CLI 188/0 across 38 suites; adapters 124/0 across 14 suites. Focused hook identity 20/20, automation 6/6, plan 51/51, concurrency 16/16, config 16/16, proof_trace 29/29, phase12_hooks 11/11, operation_evidence_cli 9/9 and phase12_hooks_cli 6/6 passed at that HEAD.
- Historical integrated gates at `0536a27`: workspace all-targets, fmt, workspace Clippy `-D warnings`, locked release build, explicit ignored release smoke, and binary version check exited 0. Version remained `5.0.0`; rerun all gates on current source before readiness.
- Authority audit: rerun returns 249 Core/CLI source-tree matching lines including inline tests; the historical 239-vs-244 mismatch is corrected in maintained artifacts. Recheck classifications and owned BUG disposition after current full gates. Official host schemas are linked in the audit. Minor shared Vault `Plans/INDEX.md`, journal-size, dedup-claim trimming, and trace-scan lock findings remain documented.
- Review readiness: local verification is complete and no known owned Critical/Important issue remains, so status is READY FOR ADVERSARIAL REVIEW. SPEC-05 is not CLOSED and is not pushed; both fresh reviewers must accept the exact committed range with zero Critical/Important findings before closure or delivery.
