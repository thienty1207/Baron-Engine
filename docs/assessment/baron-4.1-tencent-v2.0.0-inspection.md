# TencentDB Agent Memory v2.0.0 Baseline Inspection

Date: 2026-08-13

- Repository: `https://github.com/TencentCloud/TencentDB-Agent-Memory`
- Release: `v2.0.0`
- Resolved tag commit: `0aff21a2d9f2b8a0354aaa80a2e586aab4054562`
- Inspection mode: local, read-only source checkout; no Tencent service or
  private deployment credentials were used.
- Runtime check: the bundled local Node runtime was available, but the checkout
  contains no installed `node_modules`/lockfile for a self-contained scorer.
  The documented deployment starts multiple Docker services and requires LLM
  parameter sets; those services/credentials were not present or invoked.

## What the public release exposes

The pinned README describes layered L0-L3/Chat Memory, BM25 plus vector/RRF
recall, automatic session extraction, Wiki, CodeGraph, and optional Skill
extraction. Its public Benchmark section reports only a PersonaMem result:
`48%` without TencentDB Agent Memory and `76%` with it. It does not publish
per-surface scores for long-term memory, semantic grounded synthesis, session
learning, Wiki, or CodeGraph on the Baron 4.1 contract.

## Gate decision

The public release is a valid pinned architecture/comparison reference, but it
is not a same-corpus five-surface baseline. Baron therefore records Tencent as
`unavailable` for the 4.1 hard gate. No score is inferred from PersonaMem, and
no 4.1 release claim or Tencent win is made. A reviewed file supplied through
`BARON_TENCENT_BASELINE_JSON` must contain all five surface scores, revision
`v2.0.0`, `same_corpus=true`, and `confidence_95=true` before the gate can
accept it. A separate `BARON_41_CONFIDENCE_EVIDENCE_JSON` must bind the same
contract/source revision to at least three independently scored repetitions and
`confidence_95=true`; one local run is never promoted to statistical proof.
