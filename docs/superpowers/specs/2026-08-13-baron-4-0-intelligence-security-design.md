# Baron 4.0 Intelligence, Security, And Safe Promotion Design

Date: 2026-08-13
Status: implementation and local acceptance complete; public promotion is in
Phase 76 and remains blocked until the native matrix and recovery install pass.

## Decision

Baron 4.0 extends the Baron 3.8 engine in place. It keeps Vault Markdown and
current repository files authoritative, retains Baron 3.8 behavior as a
per-query and whole-engine fallback, and adds measurable local intelligence plus
defensive security assessment contracts. It does not import the external
`reverse-skill` router or make a database, model, graph cache, or generated
summary a new source of truth.

## Promotion model

The 3.8 path remains a separately measured baseline and permanent recovery
path. The released 4.0 path is selected by default only after the comparison
and release gates pass. Project identity, freshness, provenance, bounded cost,
and quality checks guard every candidate result. A low-confidence, stale,
conflicting, malformed, or unavailable 4.0 result falls back to 3.8. The
fallback is part of the runtime contract, not a release-time promise.

Set `BARON_ENGINE_GENERATION=3.8` (or `baseline`) to force the recovery path;
`4.0` is accepted explicitly and an unknown value fails closed to 3.8.

```text
question -> project/trust firewall -> 3.8 baseline
                              \\-> 4.0 candidate
                         compare + hard gates
                    4.0 wins / 3.8 fallback
```

## Intelligence boundary

Memory adds L0 evidence, L1 facts, L2 decisions/outcomes, and L3 invariants as
an abstraction axis separate from trust. Candidate, contested, superseded,
stale, and unknown records never become current facts from frequency or model
confidence alone. Hybrid recall combines exact lexical/BM25-style term scoring,
concept aliases, character n-grams, source quality, recency, trust, and direct
file/symbol relationships while filtering project identity before ranking.

Wiki remains a disposable, revision-bound index. Its 4.0 path adds typed
document/heading/link/entity relations and bounded multi-hop traversal; every
answer carries an exact source citation and stale indicator.

The default CodeGraph remains local and optional-provider independent. Its 4.0
path records language-aware symbols, imports, definitions, references, calls,
and impact hints with exact source spans. Unsupported dynamic relationships are
unknown rather than guessed.

## Security boundary

`vibe-security-scan` remains the source-AppSec owner. Reverse packs cover
defensive binary, mobile, firmware, and malware evidence. The authorized
adversary assessment path can threat-model and validate an explicitly scoped
local/lab target, but requires an authorization brief, bounded scope, isolated
execution, cleanup ownership, and receipts. Missing authorization, scope drift,
network escape, credentials, persistence, evasion, payload delivery, or
unbounded scanning fails closed.

Security findings are advisory until the independent `security-auditor` gate
validates them. Samples, secrets, exploit payloads, and untrusted instructions
do not enter trusted memory. Tool presence is not tool execution; optional tools
are capability-gated and never auto-installed.

## Measurement and logs

Phase 65 freezes the corpus and score formulas. The benchmark records separate
memory, Wiki, CodeGraph, and security totals, per-case before/after results,
hard-gate failures, source fingerprint, fixture revision, raw results, and
token/latency measurements. Full native machine metadata and cost/scale
receipts remain Phase 75 acceptance work. Reports live under
`docs/assessment/` and project evidence mirrors remain under Vault `Proofs/`,
`Traces/`, and `ProductHarness/TEST_MATRIX.md`.

No surface is promoted at a total below 90/100. A result that is better in
semantic cases but worse for exact paths, identifiers, stale decisions, old
repositories, or bounded context remains a 3.8 fallback for those tasks.

## Release boundary

Source and public download claims stay at 3.8.0 through the implementation
evidence phase. Phase 76 changes the workspace to 4.0.0 only after all four independent scorecards,
hard gates, native CI, installer lifecycle, README checks, and public Windows
reinstall smoke pass. The final release record contains the exact source SHA,
workflow run, tag, asset/checksum inventory, benchmark/certification reports,
and `releases/latest` result.
