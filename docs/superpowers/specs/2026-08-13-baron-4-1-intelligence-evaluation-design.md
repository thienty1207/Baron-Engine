# Baron 4.1 Intelligence And Evaluation Design

Date: 2026-08-13
Status: owner-approved internal release contract

## Decision

Baron 4.1 extends the released 4.0 engine with stronger local, deterministic
memory intelligence. It keeps Vault Markdown and current repository files as
the only durable sources of truth. SQLite, semantic indexes, temporal views,
Wiki indexes, graph indexes, and learning reports are rebuildable accelerators
or reviewable evidence; none can silently become a trusted instruction.

Automatic Skill generation is out of scope. Baron may route and reuse Skills
that already exist and have passed the existing owner approval and native asset
contracts, but session learning must not create, edit, or activate a Skill.

## Runtime boundary

The normal path is:

```text
project identity + trust firewall
        -> semantic memory retrieval
        -> temporal eligibility and evidence ranking
        -> grounded bounded handoff
        -> Wiki/CodeGraph evidence when relevant
        -> 4.1 result or per-query fallback to 4.0
```

The 4.0 result remains available for every query. After promotion, 4.1 is the
normal path and output is selected only when project identity, freshness,
evidence, budget, and safety checks all pass. Unknown, contested, superseded,
or unsupported content is labeled rather than guessed.

## Intelligence additions

1. Semantic retrieval uses deterministic BM25-style term weighting, bilingual
   concept expansion, character n-grams, hashed local vector similarity, and
   reciprocal-rank fusion. Eligibility filtering occurs before ranking.
2. Session learning extracts bounded facts, decisions, blockers, failures,
   outcomes, and next actions with source spans and hashes. It writes only a
   reviewable candidate report and never promotes a fact automatically.
3. Temporal memory stores observed time, valid interval, supersession,
   revalidation, source revision, and tombstone metadata in a project-scoped
   sidecar. Consolidation is atomic, reversible, and evidence-preserving.
4. Grounded handoffs carry claim-level citations, trust/freshness labels,
   conflicts, unknowns, proof, and the safe next action.
5. Wiki indexing adds typed link/entity metadata, bounded multi-hop traversal,
   deletion/rename freshness checks, and prompt-injection filtering.
6. CodeGraph indexing adds module/type/call/reference/import/test/config/API
   relation hints, reverse impact traversal, language-aware confidence, and
   incremental source fingerprints without claiming unsupported dynamic edges.

## Measurement

The 4.1 benchmark is deterministic and local. The current contract records
five development cases, a hash-sealed holdout, per-surface checks, evidence,
bounded handoff tokens, and hard failures. The Phase 86 runner emits repeated
index/query latency, cache/disk bytes, peak working-set memory, token, and
cost measurements, and refuses to continue when the frozen contract/source
hash changes. A Baron 4.1 release requires every intelligence surface to score
at least 95/100, preserve project isolation, and stay within the resource
budgets. Tencent comparison and independent external confidence are optional
diagnostics and are not release gates.

The frozen query budget is 10,000 ms for the complete five-surface query pass
on a release-profile binary; indexing is reported separately. Debug-profile
measurements remain diagnostic and cannot be used as release evidence.

The pinned public Tencent `v2.0.0` checkout is retained only as architectural
reference material. Its public benchmark is recorded in the inspection note,
but no Tencent number is converted into a Baron surface score or release gate.

The engine records optional statistical confidence only from a separate
reviewed `BARON_41_CONFIDENCE_EVIDENCE_JSON` artifact bound to the exact
contract and source revision. A missing confidence file is visible in the
report but cannot block the owner-approved local release.

## Safety and cost

No network or model download is required for the default path. Optional local
embedding/parser assets must be pinned by checksum, license, size, and
fallback behavior. Secret redaction, project isolation, path validation,
bounded output, cache rebuild, and 4.0 fallback remain hard gates.
