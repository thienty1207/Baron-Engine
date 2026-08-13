# Baron 4.0 Implementation Plan

Date: 2026-08-13
Owner approval: granted for implementation, testing, GitHub publication, and
README synchronization. Local Phase 75 acceptance passed; public promotion is
now executing under Phase 76 release authority.

## Execution order

1. Phase 65: freeze the benchmark, capture the Baron 3.8 baseline, and create
   machine-readable score/certification logs.
2. Phases 66-67: add L0-L3 memory abstraction, trust separation, consolidation,
   conflict, supersession, candidate staging, and rollback.
3. Phases 68-69: strengthen local hybrid retrieval, add shadow/A-B selection,
   grounded synthesis, source citations, and bounded cross-agent handoff.
4. Phases 70-71: add linked Wiki traversal and syntax-aware local CodeGraph
   evidence while preserving rebuildable caches and Survey fallback.
5. Phases 72-74: add defensive AppSec coverage, authorized lab assessment,
   security routing, provenance/license/tool governance, and regression tests.
6. Phase 75: run the complete four-surface scorecards, no-critical-regression
   comparison to 3.8, security/scale/recovery/cost gates, and local release
   proof. The local integrated acceptance passed; native/large-scale evidence
   remains a release follow-up.
7. Phase 76: bump to 4.0.0, synchronize README/status/JSON/build log and
   certification, commit and push exact source, let the native release workflow
   publish the immutable Release, then perform a fresh public Windows smoke.

## Required runtime behavior

- Baron 3.8 remains the measured baseline and permanent per-surface recovery
  fallback after 4.0 promotion; the released default is guarded 4.0.
- 4.0 candidates are filtered by project ID, trust, sensitivity, freshness,
  source revision, and bounded budget before ranking or synthesis.
- Exact path, identifier, error, stale-decision, and old-repository cases may
  never regress merely because semantic retrieval improves paraphrase cases.
- Semantic, Wiki, CodeGraph, and security engines have independent feature
  switches and cache generations; one failure falls back without corrupting
  Vault or forcing a whole-engine downgrade.
- Security assessment requires an explicit authorization brief and fails closed
  for scope mismatch, unsafe command, credential request, network escape,
  persistence, evasion, payload delivery, or missing cleanup ownership.

## Required evidence and logs

- `docs/assessment/baron-4.0-benchmark.json` and `.md`: frozen fixtures,
  baseline/4.0 results, four independent scores, raw case outcomes, and
  environment/configuration fingerprint.
- `docs/assessment/baron-4.0-certification.md`: Phase 75 acceptance, fallback
  proof, security hard gates, scale/cost results, and final promotion decision.
- `docs/BARON_STATUS.md` and `docs/BARON_STATUS.json`: checked tasks only after
  their evidence exists; score, source SHA, workflow, assets, and smoke details.
- `notes/build-log/CURRENT.md`: checkpoint, last passing command, failures,
  affected files, proof/trace state, and exact resume action after every batch.
- Project Vault `Proofs/`, `Traces/`, and `ProductHarness/TEST_MATRIX.md`:
  project-bound receipts and case-level evidence; weak evidence remains
  insufficient.

## Current implementation checkpoint (2026-08-13)

- [x] Phase 65 runner executes separate 3.8 baseline and 4.0 candidate paths
  across sixteen cases and four surfaces; the candidate cannot be a cloned
  baseline result.
- [x] Candidate memory reranking is firewall-first; released context selects
  guarded 4.0 by default, while `BARON_ENGINE_GENERATION=3.8` or `baseline`
  explicitly selects the recovery path.
- [x] Wiki candidate carries Markdown links, exact citations, stale labels, and
  bounded two-hop link expansion with retained link paths.
- [x] CodeGraph candidate carries source spans, imports, reference/call edges,
  and advisory provenance while preserving the old query path.
- [x] Security routing has explicit project/scope fields, fail-closed unsafe
  intent, owner-confirmed authorization, path allowlists, and a nine-case
  regression report.
- [x] Memory has read-only consolidation analysis, deterministic authority
  explanations, atomic reviewable candidate staging, and independent L0-L3
  versus trust labels; no Vault record is rewritten or auto-promoted.
- [x] Local integrated acceptance records four 100/100 surfaces, zero leakage,
  nine security regression cases, bounded static AppSec, no auto-promotion, and
  cache/source identity on a clean run.
- [ ] Pinned real-repository/adversarial corpus, durable temporal compaction,
  large-scale cost measurements, and native public release gates remain open.

## Stop conditions

Stop and keep Baron 3.8 as the active baseline if any surface is below 90/100,
any critical 3.8 case regresses, project isolation is not zero-leakage, a
source/citation/graph relation is fabricated, a security action lacks scope,
a cache cannot rebuild, a hard gate lacks execution evidence, or the public
release/installer/README claims disagree. Preserve a recovery packet and do
not bump the version to 4.0.0.

## Completion record

The program is complete only when every Phase 65-75 task has checked evidence,
all four surfaces independently pass the frozen acceptance contract, the exact
source has passed native GitHub verification, `v4.0.0` is immutable and
downloadable, README installation reports `baron 4.0.0`, a fresh Windows Vault/
project smoke passes, and the branch is clean and synchronized with GitHub.
