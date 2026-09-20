# Graphify Provider Contract Stabilization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align Baron’s optional Graphify provider with the real Graphify 0.9.25 extraction contract, keep graph trust conservative across traversal, and protect the validated local graph artifact path with deterministic fallback and mandatory real-provider certification.

**Architecture:** Baron will pass the staging directory as Graphify’s `--out` root and continue validating/promoting the provider-created `graphify-out/graph.json`. Querying will use Option B from SPEC-01: read the checksum-validated local artifact, match query terms against graph nodes, traverse bounded graph edges deterministically, normalize hits through the existing path/identity boundary, and leave source verification to the existing caller. Baron will never assume Graphify’s unsupported JSON query mode or map `max_hits` to Graphify’s token budget.

**Tech Stack:** Rust 2021, `serde_json`, existing Baron code-graph normalization/source-verification APIs, PowerShell fake provider on Windows, Cargo integration tests.

**Spec:** `fix-bug/spec/SPEC-01-graphify-provider-contract.md`

## Global Constraints

- Keep Graphify optional; absence, incompatibility, timeout, malformed output, stale cache, or provider failure must preserve Survey fallback.
- Accept exactly Graphify `0.9.25`; no wrapper, shim, fabricated JSON, human-prose semantic conversion, or provider-owned Baron state.
- Keep extraction code-only, project-local, bounded, checksum-validated, source-verifiable, and last-known-good preserving.
- Do not modify Hotel Staff application code or historical Baron records; do not bump Baron’s public version.
- Keep `QueryLimits.max_hits` and `QueryLimits.max_chars` as Baron output limits; do not pass `max_hits` as Graphify `--budget`.

## Review Focus

- A provider-created `graphify-out` directory must be found under Baron’s staging root, not nested under a path Baron already named `graphify-out`; test the exact `--out` argument and promoted artifact.
- A valid Graphify graph with extra top-level metadata must parse while malformed node/edge shapes fail closed; test both successful normalization and rejection.
- A query must not spawn `graphify query` or trust terminal prose; test provider logs for the absence of a query command and assert deterministic local hits.
- Graph edges may reference nodes that are absent or duplicated; ignore dangling edges and reject duplicate node IDs without escaping the project cache.
- A query result may point at a deleted/foreign source path; route it through existing normalization/source verification and retain Survey fallback rather than granting proof.

### Task 1: Freeze the real contract in tests and fixtures

**Files:**
- Modify: `crates/baron-core/tests/graphify_provider.rs`
- Modify: `crates/baron-core/tests/fixtures/fake-graphify.ps1`
- Create: `crates/baron-core/tests/graphify_real.rs`

**Interfaces:**
- Consumes: current `GraphifyProvider`, `CodeGraphProvider`, `QueryLimits`, and the real Graphify 0.9.25 contract recorded during baseline reproduction.
- Produces: RED regression tests proving the staging root, local artifact query, separate Baron limits, fake-provider parity, optional real-provider certification, and fallback behavior.

- [x] **Step 1: Change the fake extraction fixture to emit the real nested artifact.**

  Require the existing six-argument code-only extraction command, reject an `--out` path whose final component is `graphify-out`, create `$out/graphify-out/graph.json`, and emit a small graph containing `entry` and `related` nodes plus an extracted `calls` edge. Keep the existing timeout, nonzero, wrong-version, malformed, and oversized modes for failure tests.

- [x] **Step 2: Rewrite the provider test’s query assertions around the local artifact.**

  Keep the pinned-version and local-only assertions, require two normalized hits from `trace entry ownership`, assert the fake log contains no `query trace entry ownership` command and no `--json --budget 8`, and assert the extraction `--out` value does not end in `graphify-out`.

- [x] **Step 3: Add explicit graph-artifact failure and bound tests.**

  Add cases for malformed/duplicate graph nodes, empty and oversized questions, stale source fingerprints, bounded hit count, deterministic order, and foreign source paths. The tests must fail against the current provider because it still invokes the unsupported query command and assumes provider JSON.

- [x] **Step 4: Add an opt-in real-provider certification test.**

  Create `graphify_real.rs` that returns early with a clear skip message unless `BARON_REAL_GRAPHIFY=1`; when enabled it must require exact `0.9.25`, run refresh without `GRAPHIFY_OUT`, query the local graph artifact, assert project-local cache placement and source paths, and fail rather than installing or silently substituting Graphify.

- [x] **Step 5: Run the focused tests and record the expected RED failures.**

  Run:

  ```text
  cargo test -p baron-core --test graphify_provider --no-fail-fast
  ```

  Expected: the updated provider test fails because v5.0.0 passes a nested output root and invokes the fake provider’s removed/unsupported query contract; no production code has been changed yet.

### Task 2: Repair extraction staging and local graph querying

**Files:**
- Modify: `crates/baron-core/src/graphify.rs`
- Modify: `crates/baron-core/tests/graphify_provider.rs`

**Interfaces:**
- Consumes: Task 1’s nested fake artifact and the existing `validate_graph_json`, `validate_code_graph_artifact`, `normalize_code_graph_hits`, and `QueryLimits` boundaries.
- Produces: `GraphifyProvider::refresh` that passes the staging root and `GraphifyProvider::query` that reads the validated artifact without invoking Graphify query.

- [x] **Step 1: Change refresh to pass the staging root as `--out`.**

  Keep promotion expecting `staging/graphify-out/graph.json`, but construct the extraction arguments with `provider_path(&staging)`. Preserve cleanup, validation, atomic promotion, and last-known-good rollback paths.

- [x] **Step 2: Add typed deserialization for the observed Graphify artifact.**

  Model `nodes` (`id`, `label`, optional `source_file`, optional `_origin`) and `edges` (`source`, `target`, optional `relation`, optional `confidence`) with ignored forward-compatible metadata. Require an object with array-shaped nodes and edges, reject duplicate node IDs, and ignore dangling edges.

- [x] **Step 3: Implement bounded deterministic artifact querying.**

  Tokenize the bounded question into stable lowercase terms, rank direct node matches before one- and two-hop neighbors, use only observed node/edge fields for labels, relations, confidence, source files, and explanations, sort ties by distance/confidence/source/label/id, deduplicate, and call `normalize_code_graph_hits` with the caller’s `QueryLimits`. Do not invoke `graphify query`, pass `--json`, pass `--budget`, or parse human-readable provider output.

- [x] **Step 4: Keep confidence and source boundaries honest.**

  A direct query match with `_origin: "ast"` may be `Extracted`. Every
  traversal/neighbor result is `Inferred`, even when its edge confidence and
  target node origin are both `EXTRACTED`, because source verification proves
  the target symbol but not the source-to-target relation. Preserve relative
  source paths for the existing `verify_graph_hit_source` call and reject
  unsafe paths through the existing normalizer.

- [x] **Step 5: Run the focused provider tests to verify GREEN.**

  Run:

  ```text
  cargo test -p baron-core --test graphify_provider --no-fail-fast
  ```

  Expected: all provider regression tests pass, including no external query invocation, last-known-good preservation, source-path validation, and fallback behavior.

### Task 3: Add certification/documentation and run the full gate

**Files:**
- Modify: `docs/architecture/CAPABILITY_REGISTRY.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `.github/workflows/release.yml`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/CURRENT.md`

**Interfaces:**
- Consumes: the proven Option B contract and Task 1 real-provider test.
- Produces: maintained documentation stating the actual 0.9.25 extraction/query boundary, a mandatory protected release certification gate, and a durable SPEC-01 completion checkpoint without a version bump.

- [x] **Step 1: Document the real provider contract.**

  State that extraction receives a staging root and writes `graphify-out/graph.json`; Baron query reads the validated local artifact because Graphify 0.9.25’s query command is human-readable traversal rather than a supported JSON result API; Baron’s hit/character limits remain independent and source verification remains mandatory.

- [x] **Step 2: Add the explicit real-provider command to the certification record.**

  Record the exact executable/version/path evidence and the opt-in command `BARON_REAL_GRAPHIFY=1 cargo test -p baron-core --test graphify_real -- --nocapture`. Do not claim real certification when the environment variable was not enabled.

- [x] **Step 3: Run all focused and repository-required verification.**

  Run:

  ```text
  cargo test -p baron-core --test graphify_provider
  cargo test -p baron-core --test graphify_real
  cargo test -p baron-core --test code_graph
  cargo test -p baron-core --test capability
  cargo test -p baron-core --test context_compiler
  cargo test -p baron-cli --test code_map_cli
  cargo test -p baron-cli --test automation_cli
  cargo fmt --all -- --check
  cargo test --workspace --all-targets --no-fail-fast
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --release --locked -p baron-cli
  git diff --check
  target/release/baron --version
  ```

- [x] **Step 4: Run opt-in real Graphify and inspect the final diff.**

  Run the real certification test with `BARON_REAL_GRAPHIFY=1`; inspect `git status`, `git diff`, and exact changed paths. Verify no Hotel Staff path, source file, historical record, version metadata, or public release artifact changed.

- [x] **Step 5: Stage exact files and commit/push after final verification.**

  The user explicitly requested that each completed spec is committed and pushed
  once. Preserve unrelated untracked context/spec files and do not create a
  tag or release.

## Verification result

- Focused Graphify, real opt-in Graphify, format, workspace Clippy, release
  build, binary version, Hotel Staff functional checks, and diff inspection
  were run on 2026-09-20.
- The workspace test command completed with two failures outside this SPEC:
  `lifecycle_scripts` cannot load the host `Microsoft.PowerShell.Archive`
  module; and `prepare_cli` reaches the existing UTF-8 boundary panic in
  `session_replay`. `phase10_adapter_authority` passed in the final run.
- The Hotel Staff update dry-run was not certifiable because the pinned fixture
  already reports a managed-baseline hash mismatch. The temporary fixture was
  not repaired, and its tracked tree stayed clean after the Graphify checks.

  Report the exact files, test outcomes, real-provider evidence, Hotel Staff status (not modified), remaining later-SPEC blockers, and commit status.

## Final review-fix checkpoint (2026-09-20)

- FIX-01: direct AST matches remain `Extracted`; every graph traversal result
  is forced to `Inferred`. The regression fixture contains an extracted
  `entry --calls--> related` edge and proves direct source verification is
  `Verified` while the traversed target is `Advisory`.
- FIX-02: `crates/baron-cli/tests/code_map_cli.rs` supplies a test-local empty
  provider PATH and always asserts `present=false`, `action=survey_fallback`,
  safe refresh failure, and no provider graph artifact. Host Graphify presence
  cannot change this result.
- FIX-03: `.github/workflows/release.yml` provisions the official
  `graphifyy==0.9.25` package in `$RUNNER_TEMP/graphify-venv`, checks the exact
  `graphify 0.9.25` version, exports `BARON_REAL_GRAPHIFY=1`, and runs
  `cargo test -p baron-core --test graphify_real -- --nocapture`. The dedicated
  job is required by the release build/sign/publish dependency chain.
- Fresh focused evidence: `graphify_provider` `6/6`, `code_map_cli` `1/1`,
  opt-in local real Graphify `1/1`, and `actionlint` for both workflows pass.
- Known unrelated full-gate blockers remain recorded only: host
  `Microsoft.PowerShell.Archive` availability and the existing
  `session_replay` UTF-8 boundary panic. Hotel Staff was not tested or modified
  in this review-fix pass.
