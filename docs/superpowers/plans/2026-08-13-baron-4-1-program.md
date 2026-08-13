# Baron 4.1 Program

Date: 2026-08-13
Status: owner-approved internal release; GitHub promotion in progress

## Execution order

- [x] Phase 77: freeze the Baron-only benchmark contract and fixtures.
- [x] Phase 78: implement semantic retrieval fusion and measure it.
- [x] Phase 79: implement bounded automatic session learning candidates.
- [x] Phase 80: implement temporal truth, consolidation, and rollback.
- [x] Phase 81: implement grounded memory synthesis and adapter handoff.
- [x] Phase 82: deepen Wiki entities, links, freshness, and citations.
- [x] Phase 83: deepen CodeGraph symbols, relations, impact, and languages.
- [x] Phase 84: add incremental sync, scale, corruption recovery, and rebuild.
- [x] Phase 85: run security, cost, and per-query 4.0 fallback gates.
- [x] Phase 86: run repeated Baron-only internal acceptance; Tencent is optional.
- [ ] Phase 87: bump 4.1.0, update README, tag, release, push GitHub, and verify
  fresh Windows installation.

## Invariants

- Do not create, distill, edit, or activate Skills from sessions.
- Vault Markdown and repository files remain durable truth.
- Candidate memory is evidence-linked and reviewable, never self-approved.
- Project identity and trust eligibility are checked before ranking.
- 4.0 remains a per-query and whole-engine fallback after 4.1 promotion.
- Every completed phase gets fresh tests, build-log evidence, and a checked
  task in `docs/BARON_STATUS.md` and `docs/BARON_STATUS.json`.

## Current implementation evidence (phases remain open)

- [x] Contract/report schema, five fixture cases, holdout hash, optional
  Tencent diagnostics, and honest Baron-only target behavior.
- [x] Pinned Tencent `v2.0.0` inspection is recorded as reference material only;
  it is not a release blocker.
- [x] Optional confidence evidence remains bound to the exact contract/source
  revision; missing external confidence cannot block the local release.
- [x] Deterministic semantic retrieval, project/trust firewall ordering,
  evidence-linked session candidates, risk quarantine, temporal ledger,
  grounded handoff, Wiki/CodeGraph v5, stale-cache rebuild, edge budget, and
  token/cost telemetry.
- [x] 4.1 is the default; `BARON_ENGINE_GENERATION=4.0` selects the guarded
  fallback, while `3.8`/`baseline` selects the older recovery path.
- [x] Format/check, warnings-denied Clippy, 28 Baron core tests, all CLI test
  groups, focused memory/context/CodeGraph tests, and an isolated seeded
  development-fixture benchmark pass.
- [x] Release-profile telemetry records separate indexing/query latency, peak
  working-set memory, cache bytes, estimated handoff tokens, and cost status;
-  the latest seeded development-fixture run measured `4533 ms` total,
  `2343 ms` query, and `167301120` peak bytes within the frozen budgets.
- [x] The Phase 86 acceptance runner is implemented at
  `scripts/phase86-acceptance.ps1`; it requires a frozen contract, repeats the
  release binary three times, checks source/contract hashes, and emits raw
  `docs/assessment/baron-4.1-phase86-runner.*` evidence. Tencent and external
  confidence are recorded only when supplied and never become fake scores.
- [x] Baron-only internal acceptance passed on the five surfaces and resource
  gates; Phase 87 is the remaining publication/reinstall task.
