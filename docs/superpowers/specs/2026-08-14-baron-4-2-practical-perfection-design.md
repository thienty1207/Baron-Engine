# Baron 4.2 Practical-Perfection Design

Date: 2026-08-14
Status: owner-approved; Phases 88-99 implemented and accepted; Phase 100 release in progress
Release authority: Phase 100 only

## Decision

Baron 4.2 is a local, evidence-first intelligence release. It closes the
quality gaps that Baron 4.1 did not prove: calibrated retrieval, task-aware
session learning, temporal truth, grounded synthesis, fresh Wiki state, and
parser-backed CodeGraph impact. The release is judged against a frozen corpus
and explicit no-regression gates, not against a rounded self-reported score or
an external Tencent comparison.

Baron 4.1 remains the public whole-engine baseline until the Phase 100 Release
is verified.
Baron 4.0 remains the mandatory per-query safety fallback when a 4.2 result is
missing, ambiguous, stale, unsupported, ungrounded, unsafe, or over budget.
No public version/tag/Release promotion before the Phase 99 promote verdict and
explicit Phase 100 publication authority. The local candidate is versioned
4.2.0 so exact-source tests exercise the release artifact before publication.

## Durable truth and isolation

Vault Markdown and current repository source are the only durable truth. Every
memory event, extracted candidate, Wiki node, graph edge, and retrieval hit is
bound to:

- stable project ID and source identity;
- exact source path/span/hash and source revision;
- observed time and valid time interval;
- trust state and calibrated confidence;
- sensitivity and authority class;
- revision, supersession, contradiction, expiry, and deletion lineage.

SQLite, vectors, reranking indexes, temporal projections, Wiki/CodeGraph
caches, and evaluator artifacts are disposable. A cache with the wrong project,
schema, model/parser generation, or source revision is rejected and rebuilt;
failure to validate the temporal projection fails closed instead of skipping a
filter.

## Runtime pipeline

```text
project identity + source revision
  -> provenance/trust/temporal eligibility
  -> exact + lexical + dense + Wiki/CodeGraph candidate channels
  -> calibrated reranking and negative-evidence filtering
  -> claim-level grounded synthesis
  -> structural/identity/freshness/safety/budget arbiter
  -> 4.2 result OR unknown OR Baron 4.0 result
```

Retrieval may never create trust. A positive similarity or RRF rank without
relevant evidence is not a candidate. A missing or contested source is visible
as uncertainty, not silently removed.

## Retrieval contract

The default supported path uses exact identifier/path matching, BM25/full text,
bilingual concepts, a pinned deterministic local dense/hash-vector backend,
temporal and trust signals, Wiki links, and CodeGraph evidence. A bounded
second-stage confidence reranker records component scores, exclusions,
eligibility, rerank reason, confidence, calibration, and abstention reason.

The evaluator requires Recall@10 and nDCG@10 of at least 0.95 on answerable
semantic cases, no exact-lookup regression, and abstention precision of at
least 0.99 on unanswerable/unsafe cases. A no-evidence query must return
`unknown` or the guarded Baron 4.0 result; it must not receive an arbitrary
positive hit because every document was assigned an RRF rank.

## Session-learning contract

Session ingestion is exact-project, message/event scoped, redacted, bounded,
idempotent, and receipt-backed. It segments by project/worktree/branch,
objective, subtask, attempt, interruption, outcome, and resume evidence. It
keeps role and source spans, extracts facts/decisions/constraints/blockers/
failures/outcomes/files/commands/proof/unknowns/next actions, and classifies
temporary hypotheses, quoted text, rejected alternatives, summaries, and
unverified claims separately from durable candidates.

Stable event/source identity and semantic equivalence prevent duplicate imports
from gaining authority. Prompt injection, forged tool output, destructive
instructions, secret exfiltration, project mismatch, and unsupported claims are
quarantined before candidate output. Candidates remain reviewable and cannot
write verified Vault truth, change policy, or create/edit/activate Skills.

The evaluator requires task-boundary F1 and critical-fact recall of at least
0.95, evidence-span precision of at least 0.98, and exactly zero false durable
promotions, cross-project candidates, unredacted secrets, forged evidence, or
automatic Skill changes.

## Temporal, Wiki, and CodeGraph truth

Temporal state is bi-temporal (observed/transaction time and valid/effective
time), project-partitioned, append-only, conflict-aware, and reversible. Current
and `as of` queries preserve superseded, expired, deleted, contested, and
minority evidence. Consolidation is atomic and evidence-preserving.

Wiki and CodeGraph are derived, source-verified views. Wiki nodes retain
heading/span citations, document version/freshness, typed links, and bounded
multi-hop paths. CodeGraph uses pinned parsers for Rust, TypeScript/JavaScript,
Python, and Go; verified definitions, references, calls, dependencies, tests,
and impact hops have exact source spans and direction. Dynamic or ambiguous
relations remain inferred/unknown.

## Evaluation and promotion

The bounded private evaluation root contains eight redacted disposable case
repositories/Vaults, gold answers, negative/ambiguous cases, and a sealed
holdout. No raw owner-session corpus was supplied, so none is read or copied.
Private content and expected answers never enter Git or runtime indexes. The
runner executes every holdout case once, binds output to source/contract/
corpus/parser hashes, repeats clean and warm runs, and reports every failure.

Raw 4.2, 4.1, and forced 4.0 are scored independently. Fallback protects the
user but contributes zero to raw 4.2 quality. Phase 99 has promoted the bounded
local candidate only; public source remains 4.1.0 until Phase 100's exact-source
native Release and reinstall gate passes. If a final gate fails, the public
source remains 4.1.0 and the failed gate stays visible.

## Release boundary

Phase 100 alone may bump `4.2.0`, update the README latest target, commit and
push, create the immutable GitHub Release, and run the fresh Windows reinstall
plus 4.1/4.0 rollback smoke. The final release must leave the working tree and
remote synchronized and must remove only disposable build/test/download
caches, never Vault, project, source, evidence, or rollback data.
