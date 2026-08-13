# Baron 4.1 Benchmark

- Report: `c26d78b29ee45251528b4b527956b4282879397b20de6acc950feed555eec559`
- Contract: `9a79b2dd385c814127453dabda5edfcf36af16b9ad563a26785fa76518fc3f5f`
- Project: `59b68ba36271a308126ddf49e5a8891fb6475665a0875b1a21fb094c990b46aa`
- Source revision: `6c490703ad22d4d1b1bee6cb02e78ae34133e6d47e019bb54172e2ca07ff2be2`
- Target achieved: `false`
- Same-corpus Tencent win: `false`
- Statistical confidence 95%: `false`
- Repetitions: `1`

## Baron surfaces

- long_term_memory_l0_l3: **100/100** (3 / 3 cases)
  - evidence: memory_hits=4
- semantic_retrieval_grounded_synthesis: **100/100** (4 / 4 cases)
  - evidence: Projects/baron-engine--59b68ba36271/Facts.md:2326
  - evidence: Projects/baron-engine--59b68ba36271/Decisions.md:1620
  - evidence: Projects/baron-engine--59b68ba36271/Sessions/Imported/phase86-session.md:1611
  - evidence: Projects/baron-engine--59b68ba36271/Sessions/Imported/phase86-session.md:1328
  - evidence: handoff_claims=4
- automatic_session_learning: **100/100** (6 / 6 cases)
  - evidence: sources=1
  - evidence: messages=1
  - evidence: candidates=1
  - evidence: quarantined=0
  - evidence: skills_created=0
- codegraph: **100/100** (6 / 6 cases)
  - evidence: files=127
  - evidence: symbols=1817
  - evidence: edges=178239
  - evidence: impact_paths=8
- wiki: **100/100** (4 / 4 cases)
  - evidence: wiki_hits=8
  - evidence: temporal_active=4
  - evidence: temporal_superseded=7

## Tencent baseline

- Status: `unavailable`
- Revision: `unknown`
- Same corpus: `false`
- 95% confidence evidence: `false`
- Detail: Set BARON_TENCENT_BASELINE_JSON to a reviewed same-corpus score file; no implicit Tencent score is invented.

## Local run metrics

- Execution profile: `release`
- Total elapsed: `4735` ms
- Index elapsed: `2367` ms
- Query elapsed: `2367` ms
- Estimated tokens: `367`
- Cache/artifact bytes: `44155262`
- Peak memory: `168239104`
- Cost status: `within_context_budget`

## Hard failures

- Tencent baseline is unavailable; comparison cannot claim a win
- Baron did not prove a same-corpus Tencent win
- Independent repeated-run confidence evidence is missing or invalid: Set BARON_41_CONFIDENCE_EVIDENCE_JSON to an independently scored repeated-run file
