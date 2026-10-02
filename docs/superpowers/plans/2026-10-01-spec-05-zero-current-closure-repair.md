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

- Start: FIX REQUIRED in maintained status artifacts; prior verification was historical. Current full verification is complete and status is READY FOR ADVERSARIAL REVIEW pending two fresh final-range reviews.
- Ruling: latest user repair prompt overrides the older inline skill's no-re-review restriction if review fixes change final HEAD; final acceptance must include the delivered source/docs.
- Review checkpoint: fresh reviews of `ecaf362..af65881` found three Important issues: completed correlated Stop blocked the host; new public `unknown_fields` members broke external config struct literals; shared Vault TEST_MATRIX upserts used checkout-local locks only. The repair adds focused regressions for each; two fresh final-range reviews remain pending.
- RED evidence: `public_config_compat` initially failed compilation with three missing `unknown_fields` fields; exact completed Stop returned `decision:block`; a child in a second checkout updated the shared matrix while the Vault capsule lock was held.
- GREEN evidence: `hook_identity` 18/18, `concurrency` 16/16, `config` 16/16, `plan` 51/51, `proof_trace` 29/29, `public_config_compat` 1/1; completed Stop mismatch-frontmatter rejection passes; config unknown root/nested/local TOML values survive init and setters. Workspace all-targets, focused Core/CLI/adapter suites, format, Clippy, locked release build, and explicit release smoke pass.
- Ruling: keep the original public config struct field surface and preserve unknown keys by merging from the existing TOML document during managed writes; adding `#[non_exhaustive]` or hidden state would further break source compatibility or be unsafe.
- Ruling: shared harness mutations acquire checkout lock before capsule lock; these call paths have no reverse-order acquisition, and timeout occurs before a shared matrix write. Lock ordering is documented in source and covered by a cross-process held-lock regression.
- Review Minor disposition: retain for fresh adversarial reassessment. Bounded journal response parsing reads the whole journal before scanning 512 lines; trace scoring walks the managed trace archive under the project lock; dedup capacity trimming may evict an in-flight claim, while token fencing prevents stale publication. These are not known Critical/Important defects; document a reasoned acceptance or fix if a fresh reviewer finds impact above Minor.
- Follow-up review checkpoint: fresh independent reviews of `ecaf362..fcc6b79` found Important gaps: session-only Stop missed a validated legacy active plan without an ACTIVE row; Stop could publish a stale pass after newer same-operation evidence appeared. Reviewer two confirmed the legacy gap; reviewer one independently found both. These findings keep status at FIX REQUIRED.
- Follow-up RED-to-GREEN: the unindexed legacy plan regression first returned `continue:true` for completed A while active B existed, then passed after session checks scan all validated identified managed plans. The synchronized publication-race test first published `reconciliation_passed:true` after a newer failing trace, then passed after final Stop response reconciliation was recomputed under the publication lock.
- Final package verification: `cargo test -p baron-core --all-targets --no-fail-fast -j 1` exit 0 (558 passed, 0 failed, 60 suites); CLI exit 0 (188 passed, 0 failed, 38 suites, 1 ignored release smoke separately passed); adapters exit 0 (124 passed, 0 failed, 14 suites, 3 historical ignored). Focused hook identity 20/20, automation unit 6/6, plan 51/51, concurrency 16/16, config 16/16, proof_trace 29/29, phase12_hooks 11/11, operation_evidence_cli 9/9 and phase12_hooks_cli 6/6 pass.
- Integrated gates: `cargo test --workspace --all-targets --no-fail-fast -j 1` exit 0 with isolated host session roots and bundled PowerShell 7 override; `cargo fmt --all -- --check`, workspace Clippy `-D warnings`, `cargo build --workspace --release --locked`, explicit ignored release smoke, and binary version check all pass. Version remains `5.0.0`.
- Authority audit: final production-source CURRENT/latest caller search is 244 Core/CLI matching lines, all represented in `2026-10-01-spec-05-authority-audit.md`; no known owned production BUG remains. Official host schemas are linked there. One Minor shared Vault `Plans/INDEX.md` lost-update risk remains deferred as discovery-only.
- Review readiness: local verification has no known owned Critical/Important finding. SPEC-05 is READY FOR ADVERSARIAL REVIEW. Two fresh independent read-only reviews over the exact final committed range are required before CLOSED or push.
