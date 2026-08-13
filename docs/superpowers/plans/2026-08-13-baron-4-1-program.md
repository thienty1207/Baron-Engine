# Baron 4.1 Program

Date: 2026-08-13
Status: implementation in progress; owner approved execution

## Execution order

- [ ] Phase 77: freeze the shared benchmark contract and fixtures.
- [ ] Phase 78: implement semantic retrieval fusion and measure it.
- [ ] Phase 79: implement bounded automatic session learning candidates.
- [ ] Phase 80: implement temporal truth, consolidation, and rollback.
- [ ] Phase 81: implement grounded memory synthesis and adapter handoff.
- [ ] Phase 82: deepen Wiki entities, links, freshness, and citations.
- [ ] Phase 83: deepen CodeGraph symbols, relations, impact, and languages.
- [ ] Phase 84: add incremental sync, scale, corruption recovery, and rebuild.
- [ ] Phase 85: run security, cost, and per-query 4.0 fallback gates.
- [ ] Phase 86: run sealed independent acceptance and Tencent comparison.
- [ ] Phase 87: only after acceptance, bump 4.1.0, update README, tag, release,
  push GitHub, and verify fresh Windows installation.

## Invariants

- Do not create, distill, edit, or activate Skills from sessions.
- Vault Markdown and repository files remain durable truth.
- Candidate memory is evidence-linked and reviewable, never self-approved.
- Project identity and trust eligibility are checked before ranking.
- 4.0 remains a per-query and whole-engine fallback until Phase 86 passes.
- Every completed phase gets fresh tests, build-log evidence, and a checked
  task in `docs/BARON_STATUS.md` and `docs/BARON_STATUS.json`.

## Current implementation evidence (phases remain open)

- [x] Contract/report schema, five fixture cases, holdout hash, explicit
  Tencent baseline validation, and honest `target_achieved=false` behavior.
- [x] Pinned Tencent `v2.0.0` inspection is recorded; its public PersonaMem
  number cannot substitute for the five-surface same-corpus baseline.
- [x] Confidence evidence is accepted only from a separate artifact bound to
  the exact contract/source revision with at least three repetitions; a local
  single run cannot mark the gate passed.
- [x] Deterministic semantic retrieval, project/trust firewall ordering,
  evidence-linked session candidates, risk quarantine, temporal ledger,
  grounded handoff, Wiki/CodeGraph v5, stale-cache rebuild, edge budget, and
  token/cost telemetry.
- [x] `BARON_ENGINE_GENERATION=4.1` is explicit opt-in; a structural, evidence,
  budget, or runtime failure returns the proven 4.0 result.
- [x] Format/check, warnings-denied Clippy, 28 Baron core tests, all CLI test
  groups, focused memory/context/CodeGraph tests, and an isolated seeded
  development-fixture benchmark pass.
- [x] Release-profile telemetry records separate indexing/query latency, peak
  working-set memory, cache bytes, estimated handoff tokens, and cost status;
-  the latest seeded development-fixture run measured `4735 ms` total,
  `2367 ms` query, and `168239104` peak bytes within the frozen budgets.
- [x] The Phase 86 acceptance runner is implemented at
  `scripts/phase86-acceptance.ps1`; it requires a frozen contract, repeats the
  release binary three times, checks source/contract hashes, and emits raw
  `docs/assessment/baron-4.1-phase86-runner.*` evidence without inventing a
  Tencent score.
- [ ] Real sealed corpus, repeated independent scorer, Tencent v2.0.0
  same-corpus baseline, statistical confidence, and three Baron 95/100
  surfaces are still missing; no release promotion is authorized. The runner
  status is `blocked_external_evidence` and the exact failures are recorded.
