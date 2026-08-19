# Changelog

## 4.2.1 - 2026-08-19

Baron 4.2.1 is the adapter-packaging patch release. It contains the
Reasonix implementation that landed after the original 4.2.0 tag, so a fresh
install now has the same CLI surface documented by the source and README:

- `baron init --reasonix` and `baron --reasonix` are present in the released
  binary;
- Reasonix and Codex keep one project identity, Vault, memory, Wiki, CodeGraph,
  continuity ledger, and session history;
- adapter installation remains preserve-first for user-owned Reasonix files;
- the intelligence engine, 4.1 rollback, 4.0 fallback, and existing project
  data are unchanged;
- release metadata, installers, checksums, and README now point to `v4.2.1`.

## 4.2.0 - 2026-08-14

Baron 4.2 makes memory answers evidence-first and measurable:

- calibrated exact, bilingual lexical, local dense, temporal, Wiki, and
  CodeGraph reranking with negative-query abstention and 4.0 fallback
- task-segmented, idempotent session learning with deduplication, evidence
  spans, candidate-only output, omission receipts, and poisoning quarantine
- source-span-aware bi-temporal lineage, conflict detection, expiry,
  project-bound ledgers, backups, and rollback
- Wiki deletion/rename tombstones and CodeGraph symbol tombstones with
  directional relation confidence and bounded impact paths
- a hash-sealed development contract, private one-open holdout runner, and
  three-repeat acceptance report; no fallback result inflates the raw score
- README/install target, source version, and native release metadata are
  synchronized at `v4.2.0`; 4.1 and 4.0 remain explicit fallbacks

## 4.1.0 - 2026-08-13

Baron 4.1 makes the stronger local intelligence path the default while
retaining Baron 4.0 as a guarded fallback:

- deterministic bilingual semantic retrieval with lexical/vector/RRF evidence
- bounded session learning that produces redacted, evidence-linked candidates
  without creating or activating Skills
- temporal memory with supersession, conflicts, freshness, backup, and rollback
- grounded, cited handoffs with bounded token/cost budgets
- semantic Wiki and CodeGraph retrieval with typed evidence and impact paths
- Baron-only repeated acceptance across five local surfaces and resource gates
- `BARON_ENGINE_GENERATION=4.0` fallback and `3.8`/`baseline` recovery switch
- checksum-verified native release and reinstall documentation for `v4.1.0`

TencentDB Agent Memory remains an optional architectural reference; it is not a
release gate for Baron 4.1. Automatic Skill creation remains out of scope.

## 2.0.0 - 2026-06-16

Baron 2.0 turns the first stable engine into a long-horizon agent harness:

- observable automation and stable project identity
- massive shared-Vault memory indexing with project firewalling
- multilingual task-aware recall and automatic session import
- strict skill and agent control plane with mandatory gate evidence
- self-improving Product Harness audits, interventions, proposals, and outcomes
- Baron certification gate for scale, memory, isolation, cache recovery, context budget, and release readiness
- workspace release version and native archive contract bumped to `2.0.0`

## 1.0.0 - 2026-06-15

Baron's first stable release combines:

- bounded repository survey and context compilation
- shared-Vault memory with cross-project firewalling
- Codex, Claude, and generic agent adapters
- Superpowers and three core quality agents
- optional frontend and defensive security skills
- active plans, Product Harness, proof, and trace quality gates
- transactional Agent Bootstrap migration and rollback
- capability-aware execution evidence
- native Windows, Linux, Intel macOS, and Apple Silicon macOS release flow
- checksum-verified install, update, rollback, and uninstall lifecycle
