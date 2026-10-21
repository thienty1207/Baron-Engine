# SPEC-05 Multi-Agent Concurrency and Durable State

## Goal

Implement the attached SPEC-05 requirements in the Baron Core repository so
concurrent agents cannot lose shared durable state, cannot overwrite distinct
proof/trace/plan instances, and fail closed with an explicit no-write result
when the project mutation lock times out. Close the implementation at
`SPEC-05: READY FOR ADVERSARIAL REVIEW`; do not claim the spec is closed.

## Architecture

Use the existing project-scoped OS mutation lock in `safe_io.rs` as the single
serialization boundary for every shared read-modify-write mutation. Acquire it
before the first shared read and hold it through the corresponding repository,
Vault, index, current-state, and runtime-evidence publication. Keep child
processes, network calls, capability probes, and other slow external work
outside the lock. Use durable append for true append-only logs, preserve-first
atomic replacement for upserts/caches, and create-new publication for unique
proof/trace/plan artifacts. Reentrant same-thread locking must continue to
support nested Core writers.

## Tech stack

- Rust workspace, `baron-core` primary implementation crate.
- Existing `safe_io` lock and preserve-first atomic file primitives.
- Existing `getrandom` dependency for collision-resistant IDs.
- Cargo unit/integration tests and deterministic multiprocess fixtures.
- Maintained Markdown/JSON status and build-log documents.

## Source requirements

- Specification: `D:\Works\Baron-Engine\fix-bug\spec\SPEC-05-multi-agent-concurrency-durable-state.md`
- Execution prompt: `D:\Works\Baron-Engine\fix-bug\prompt\README-PROMPT-SPEC-05-multi-agent-concurrency-durable-state.md`

The attached documents are technical requirements and review criteria. The
user request authorizes implementation, verification, commit, and push of this
spec only; it does not authorize unrelated cleanup, release/tag/version work,
or a pull request.

## Global constraints

- Preserve unrelated user-owned files and existing SPEC-04 history.
- Keep Baron Core as the source of product semantics and preserve operation,
  receipt, proof, trace, and managed-plan authority invariants.
- Do not hold the mutation lock around child execution, network access,
  capability probing, or other slow external work.
- Legacy proof/trace/plan IDs and paths remain readable; new writes use the
  new collision-resistant identity only.
- A failed lock acquisition must perform no shared-state write and must return
  an explicit timeout error.
- Do not introduce the SPEC-06 repository/Vault transaction or rollback
  framework.
- Do not change the product version, release, tag, or branch history.

## Review focus

- B-09/W-05: complete mutation inventory and honest classification.
- B-10/B-11/W-08: collision-resistant proof and trace IDs, no overwrite,
  serialized publication, runtime evidence, and concurrent score/record.
- B-18: unique plan instance paths, exact-path resume, legacy compatibility,
  and managed path authority.
- B-25: config and first-init races with identity/unknown-field preservation.
- B-26: harness intake, friction, intervention, validation, proposals,
  outcomes, `TEST_MATRIX`, and `STORIES/CURRENT` concurrency.
- B-27: capability registry and runtime evidence concurrency; cache semantics
  explicitly non-authoritative where replacement is retained.
- Lock-timeout no-write behavior and absence of lock coverage around child or
  probe work.

## Implementation tasks

### Task 1 — Safe I/O identity and create-new primitives

Files:

- `crates/baron-core/src/safe_io.rs`
- focused safe I/O tests in the existing module or `crates/baron-core/tests/`

RED:

1. Add tests proving two generated instance IDs are distinct under a tight
   same-process loop and include a timestamp-sortable component plus a
   cryptographically random suffix.
2. Add a test proving the create-new publication primitive refuses an existing
   regular file without changing its bytes.
3. Add a test proving unsafe existing targets (symlink/reparse/non-file) are
   rejected and failed creation leaves no temporary managed artifact.

GREEN:

1. Implement a shared collision-resistant artifact ID helper using the existing
   direct `getrandom` dependency and a bounded encoding.
2. Implement a durable `create_new_file`/text helper with safe parent-chain
   validation, `create_new(true)`, flush/sync, and parent-directory sync.
3. Keep `replace_text` and `append_text` behavior unchanged for their existing
   callers and document which caller owns cross-process locking.

### Task 2 — Proof, trace, and plan serialization

Files:

- `crates/baron-core/src/proof.rs`
- `crates/baron-core/src/trace.rs`
- `crates/baron-core/src/plan.rs`

RED:

1. Add deterministic concurrent writer tests for proof and trace recording,
   including two processes, barrier release, unique IDs, both repo/Vault
   artifacts, complete indexes, and no truncated files.
2. Add concurrent trace record/score coverage and assert scoring preserves the
   selected trace rather than racing a different artifact.
3. Add same-day same-title concurrent plan-start coverage, exact resume-path
   coverage, legacy-path readability, and managed-path/frontmatter assertions.
4. Add lock-timeout fixtures for proof/trace/plan mutation paths and assert
   that no target or index/current file changes when acquisition fails.

GREEN:

1. Acquire the project lock before shared reads in proof/trace/plan mutation
   paths and retain it through all related publication.
2. Replace timestamp-only new proof and trace IDs with the shared random
   instance ID; publish new repo/Vault artifacts create-new and never replace
   an existing instance.
3. Give new plan filenames a random instance component; resume the exact
   persisted path and keep legacy plan paths readable.
4. Preserve SPEC-03 operation-bound authority and SPEC-04 managed plan-path
   authority; do not put locks around execution/probe work.

### Task 3 — Config and first-initialization races

Files:

- `crates/baron-core/src/config.rs`
- focused config tests and the multiprocess concurrency fixture

RED:

1. Add two-process first-init coverage against one empty project and assert
   valid parseable project/local config, one stable project identity, and no
   lost unknown fields.
2. Add concurrent `set_project_platform`, `set_active_adapter`, and
   reinitialization coverage with lock-timeout no-write assertions.

GREEN:

1. Acquire the project lock before config existence/load/read-modify-write
   decisions, including first initialization after safe `.baron` creation.
2. Keep config writes atomic and preserve identity, unknown keys, and existing
   user-owned configuration semantics.

### Task 4 — Harness, intent, recovery, and capability evidence

Files:

- `crates/baron-core/src/harness.rs`
- `crates/baron-core/src/harness_improvement.rs`
- `crates/baron-core/src/intent.rs`
- `crates/baron-core/src/continuity.rs`
- `crates/baron-core/src/capability.rs`

RED:

1. Add concurrent friction, intervention, validation/current-state, proposals,
   outcomes, and `TEST_MATRIX`/story publication tests; require every valid
   append or upsert to survive.
2. Add concurrent capability registry register/remove and runtime evidence
   append tests; assert each JSONL record remains complete and parseable.
3. Add lock-timeout no-write coverage for a representative mutator in each
   family.

GREEN:

1. Lock harness and improvement mutators before shared reads and use
   `append_text` for true append-only files; retain locked atomic replacement
   for upsert rows and current-state documents.
2. Lock intent and recovery read-modify-write flows while preserving existing
   continuity checkpoint locking and nested reentrancy.
3. Lock capability registry mutations before load and serialize runtime
   evidence with append; keep capability probes outside the lock and document
   the capability-state cache as last-completed-writer, non-authoritative
   replacement state.
4. Audit automation, autopilot, migration, control-plane, and existing safe
   writers so the final inventory distinguishes locked, atomic append,
   replace-only cache, read-only, out-of-scope, and bug cases.

### Task 5 — Concurrency matrix, documentation, and delivery

Files:

- `crates/baron-core/tests/concurrency.rs` (or the smallest existing test
  location that supports the required multiprocess barriers)
- `docs/BARON_STATUS.md`
- `docs/BARON_STATUS.json`
- `docs/superpowers/plans/CURRENT.md`
- `notes/build-log/CURRENT.md`

RED/GREEN:

1. Build a deterministic barrier-controlled multiprocess matrix covering
   proof, trace, plan, config, first-init, friction, intervention,
   `TEST_MATRIX`, capability registry, runtime evidence, concurrent score/
   record, and lock timeout. Report exact pass/fail counts.
2. Run focused Core suites, workspace tests, fmt, Clippy, release locked,
   diff check, status JSON parsing, and repository-relative link checks.
3. Record known unrelated baseline blockers without relabelling them expected
   red; a SPEC-05-owned failure remains a bug and prevents READY.
4. Run a fresh adversarial review package against the SPEC-04 closure merge
   base, fix Critical/Important findings with RED→GREEN tests, and record any
   accepted minor residuals.
5. Update maintained status/build-log/plan documents with the mutation
   inventory, dispositions, matrix, evidence boundary, and next action.
6. Commit the scoped implementation and docs, push
   `codex/spec-05-multi-agent-concurrency-durable-state`, verify local and
   remote SHA equality, and leave the worktree clean. Do not create a PR.

## Verification commands

```text
cargo fmt --all -- --check
cargo test -p baron-core --all-targets --no-fail-fast -j 1
cargo test --workspace --all-targets --no-fail-fast -j 1
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --release --locked
git diff --check <spec-04-closure>..HEAD
```

Also parse `docs/BARON_STATUS.json`, validate repository-relative Markdown
links, confirm version `5.0.0`, and run the retired-adapter gate required by
`AGENTS.md`. Report unrelated baseline failures exactly as observed.
