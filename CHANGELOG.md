# Changelog

## 5.0.0 - 2026-09-09

Baron Engine 5.0.0 completes the Codex + Claude Core consolidation and ships
the hardened release/update boundary:

- Baron Core is the canonical owner for skills, workflow, memory, trusted
  context, Task State, recovery, routing, proof, trace, gates, and Autopilot;
- Codex and Claude are thin native bridges over one `.baron/core` tree, with
  explicit operation identity and preserve-first managed projections;
- trusted memory/context continuity, profile-aware routing, the Database
  domain, native hook idempotency, and safe Autopilot are available through the
  structured prepare path;
- updates, downgrades, migration, rollback, and dry-run preserve user-owned
  project/Vault state and fail closed on incompatible or tampered state;
- release metadata is a deterministic detached Ed25519 manifest authenticated
  by the pinned `baron-release-2026` production key before artifact selection,
  size/hash checks, or installation; the signing workflow consumes the
  protected base64 raw-seed `BARON_RELEASE_SIGNING_KEY` contract.

## 4.2.2 - 2026-08-20

Baron 4.2.2 published the multi-agent core parity correction that preceded the
Codex and Claude-only product boundary:

- Codex and Claude shared the same embedded Baron skill and agent core;
- all adapter views shared one project ID, Vault, memory, Wiki, CodeGraph, plan,
  proof, trace, continuity, and session history;
- missing Baron assets are reconciled safely while changed and custom user files
  remain preserved and conflicts are reported;
- cross-adapter round-trip, preservation, parity, README, and release tests were
  included in the public patch release.

## 4.2.1 - 2026-08-19

Baron 4.2.1 was the adapter-packaging patch release. Its historical adapter
compatibility work is retained only in Git history; the current source exposes
Codex and Claude as the supported integrations:

- both integrations keep one project identity, Vault, memory, Wiki, CodeGraph,
  continuity ledger, and session history;
- adapter installation remains preserve-first for user-owned files;
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
- Codex and Claude adapter projections
- Superpowers and three core quality agents
- optional frontend and defensive security skills
- active plans, Product Harness, proof, and trace quality gates
- transactional Agent Bootstrap migration and rollback
- capability-aware execution evidence
- native Windows x64 and Linux x64 release flow
- checksum-verified install, update, rollback, and uninstall lifecycle
