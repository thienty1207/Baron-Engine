# Baron 4.1 Benchmark

- Report: `1e2f255122be48e2c8c1bd8b01390d30567098fad5e6a29974bad15353c094cf`
- Contract: `86054c9a45c7d61df91b8b1468ed13347ef96a66091f69a7c404c646dab62af2`
- Project: `59b68ba36271a308126ddf49e5a8891fb6475665a0875b1a21fb094c990b46aa`
- Source revision: `cd0f21bc916bd9e8c607069442eeccfab52d1c700956ec613c1846b07831141d`
- Target achieved: `true`
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

## Optional external comparison

- Status: `unavailable`
- Revision: `unknown`
- Same corpus: `false`
- 95% confidence evidence: `false`
- Detail: Not supplied; external comparison is optional and non-blocking for the Baron 4.1 release.
- Release gate: `non-blocking`

## Local run metrics

- Execution profile: `release`
- Total elapsed: `4533` ms
- Index elapsed: `2189` ms
- Query elapsed: `2343` ms
- Estimated tokens: `367`
- Cache/artifact bytes: `44153904`
- Peak memory: `167301120`
- Cost status: `within_context_budget`
