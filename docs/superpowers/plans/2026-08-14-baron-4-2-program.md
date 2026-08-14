# Baron 4.2 Program

Date: 2026-08-14
Status: release-complete; Phases 88-100 complete; normal maintenance remains
Design: `docs/superpowers/specs/2026-08-14-baron-4-2-practical-perfection-design.md`
Local and public source are `4.2.0`; `releases/latest` resolves to the verified
immutable `v4.2.0` Release. Baron 4.1 and 4.0 remain recovery paths.

## Execution rules

- Do not promote, tag, or publish 4.2 before Phase 99 passes and Phase 100 is
  explicitly authorized.
- Keep Baron 4.1 as the normal public baseline during development and Baron
  4.0 as the per-query/whole-engine safety fallback.
- A task is complete only with current-source tests and recorded evidence in
  `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, and `notes/build-log/CURRENT.md`.
- Never use a fallback result to inflate the raw 4.2 score.
- Keep private session corpus and gold answers outside Git and runtime indexes.

## Phase 88 - Contract and baseline

- [x] Audit every open 4.1 breadth item and map it to 4.2 ownership.
- [x] Add correctness oracles instead of existence-only hit/edge checks.
- [x] Freeze supported scope, corpus, rubrics, thresholds, failure taxonomy,
  budgets, and fallback/no-regression rules.
- [x] Reproduce 4.1 and forced 4.0 clean baselines.
- [x] Create the 4.2 design, this executable plan, status JSON program, build
  log checkpoint, and continuity recovery packet.

## Phase 89 - Private corpus and evaluator

- [x] Build a redacted local-only eight-case private holdout outside Git and
  runtime indexes. Raw owner sessions were not supplied or copied.
- [x] Add current, conflict, missing, stale/wrong, and cross-project gold cases.
- [x] Seal a disjoint holdout and ensure the runner executes every case once.
- [x] Add negative, prompt-injection, forged-citation, duplicate, corruption,
  and missing-source mutations.

## Phase 90 - Provenance and firewall

- [x] Implement the versioned evidence envelope and project-partitioned durable
  lineage.
- [x] Enforce provenance/trust/temporal eligibility before ranking or synthesis.
- [x] Migrate existing 4.1 state transactionally with backup, rollback, and
  cache/source equivalence.
- [x] Prove zero leakage over the bounded adversarial isolation matrix.

## Phase 91 - Retrieval

- [x] Implement exact, lexical, bilingual, dense, temporal, Wiki, and graph
  candidate channels with pre-ranking eligibility.
- [x] Add a pinned deterministic local dense backend and measured bounded
  reranker.
- [x] Add calibration, negative-evidence filtering, abstention thresholds,
  explanations, incremental invalidation, and 4.0 fallback.
- [x] Pass per-slice Recall@10/nDCG@10, exact-regression, and abstention gates
  on the frozen development/holdout contract.

## Phase 92 - Session learning

- [x] Implement task/attempt/outcome segmentation with role and evidence spans.
- [x] Separate durable candidates from hypotheses, quotes, summaries, rejected
  alternatives, noise, and unverified claims.
- [x] Add stable-event/semantic deduplication and visible omission receipts.
- [x] Add poisoning, forged-tool-output, secret, destructive-command, and
  project-mismatch quarantine.
- [x] Keep output candidate-only and idempotent; pass bounded private metrics.

## Phase 93 - Temporal truth

- [x] Implement project-partitioned bi-temporal state, conflict sets,
  supersession/expiry/deletion/revalidation, and authority rules.
- [x] Implement atomic evidence-preserving consolidation, checkpoints, and
  rollback under concurrency and fault injection.
- [x] Pass current/as-of/conflict/stale/expiry correctness gates.

## Phase 94 - Grounded synthesis and fallback

- [x] Generate claim-level cited Resume Briefs with conflicts and unknowns.
- [x] Add independent structural/grounding/identity/freshness/safety/budget
  arbitration and accurate generation labels.
- [x] Prove 4.0/unknown behavior for low-confidence, missing, corrupt, stale,
  unsupported, and over-budget paths.

## Phase 95 - Wiki

- [x] Add document/entity/version/freshness nodes and typed links.
- [x] Add bounded cited multi-hop retrieval and negative/no-answer filtering.
- [x] Add incremental rename/delete/tombstone/rebuild parity and injection/path
  safety.

## Phase 96 - CodeGraph

- [x] Add the pinned offline source extractor for Rust, TypeScript/JavaScript,
  and Go with source spans and provenance.
- [x] Extract directionally correct definitions, references, calls, imports,
  dependencies, tests, config/API/database edges, and impact paths.
- [x] Add relation-specific confidence, dynamic unknowns, incremental
  invalidation, rename/delete freshness, and gold edge/impact evaluation.

## Phase 97 - Adapters and shadow

- [x] Integrate equivalent bounded context/fallback behavior across Codex,
  Claude, and generic adapters.
- [x] Run 4.2 beside the retained 4.1 path, preserve custom assets/hooks, and
  verify fresh-agent
  resume across interruption, branch/worktree, Vault restore, and offline mode.

## Phase 98 - Scale and safety

- [x] Run bounded long-history/large-repository/multi-agent resource profiles.
- [x] Add backpressure, cancellation, checkpoints, concurrent-reader/writer
  safety, and crash/disk/schema/model/parser fault injection.
- [x] Pass memory, disk, latency, token, cost, secret, poisoning, path, and
  asset-integrity gates.

## Phase 99 - Integrated acceptance

- [x] Freeze release-candidate source, corpus, evaluator, asset, and config
  hashes.
- [x] Run raw 4.2, 4.1, and forced 4.0 on every bounded case with three
  clean/warm
  repetitions and complete raw artifacts.
- [x] Run all workspace, adapter, migration, security, recovery, scale,
  lifecycle, and cross-platform candidate checks.
- [x] Obtain fresh trusted receipts from the three mandatory quality agents.
- [x] Produce `promote` or `reject and keep 4.1`; any open hard gate rejects.

## Phase 100 - Public release

- [x] Obtain final publication authority after Phase 99.
- [x] Bump all source/docs/manifests/installers exactly to `4.2.0`.
- [x] Update README with install, Vault restore, fallback, rollback, limits,
  local assets, and expected version.
- [x] Pass native matrix, exact-source verification, checksums, manifest,
  installers, immutable tag/Release, `releases/latest`, fresh Windows install,
  4.1 rollback, and 4.0 fallback. CI run
  `31771633229`; release run `31771646989`; source/tag commit
  `af42a2d3fcf37f315c6a24c5cebbef59ee6a4bc0`.
- [x] Mark only evidenced tasks `[x]`, commit/push final docs, verify remote and
  local state, and clean only disposable smoke/download caches. Public Release:
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.0`.
