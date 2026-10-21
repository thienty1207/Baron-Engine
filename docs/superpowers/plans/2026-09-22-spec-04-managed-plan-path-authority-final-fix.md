# Implementation Plan: SPEC-04 Managed Plan Path Authority Final Fix

> **For the inline executor:** follow `superpowers:executing-plans` and
> `superpowers:test-driven-development`; the user explicitly requested
> immediate execution, so no plan-approval pause is needed.

## Goal

Resolve the reviewed SPEC-04 MUST-FIX where `docs/baron/plans/CURRENT.md` can
point at an arbitrary readable Markdown file and let that file become linked
plan authority. Constrain authority loading and mutation to a real, regular
Baron plan document under `docs/baron/plans/`, while preserving the existing
SPEC-04 lifecycle, identity, proof, trace, gate, and legacy behavior.

## Architecture and approach

- Baron Core `crates/baron-core/src/plan.rs` remains the single authority
  boundary for CURRENT pointer resolution, linked metadata parsing, lifecycle
  mutation, and Vault mirroring.
- `active_plan()` will resolve CURRENT pointers through a managed-root helper
  that checks relative path syntax, lexical/real containment, and regular-file
  safety. Existing `safe_io` reads remain the final symlink/reparse/non-regular
  protection at file access time.
- Linked metadata will be parsed only from one leading `---` frontmatter block;
  `type: baron-plan`, required fields, and unique known authority fields are
  required. Benign unknown frontmatter keys remain tolerated.
- Vault mirroring will derive a relative path only after proving the repository
  path is beneath the exact managed Baron plan root; the old outside-root
  fallback will be removed.
- Regression tests will exercise every hostile pointer/metadata shape in the
  prompt and retain a positive managed-plan completion path.

## Technology and verification

- Rust 2021 workspace; focused integration coverage in
  `crates/baron-core/tests/plan.rs`.
- Primary implementation file: `crates/baron-core/src/plan.rs`.
- Maintained evidence: `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`,
  `notes/build-log/CURRENT.md`, and `docs/superpowers/plans/CURRENT.md`.
- Requirement source: attached
  `D:\Works\Baron-Engine\fix-bug\prompt\README-PROMPT-SPEC-04-MANAGED-PLAN-PATH-AUTHORITY-FINAL-FIX.md`.
- No reachable canonical SPEC file is present in the clean managed worktree;
  the attached final-fix prompt is the operative requirement document for this
  pass.

## Global constraints

- Stay on `codex/spec-04-proof-trace-gate-completion-integrity` at the reviewed
  baseline `8b5279b06007578afaf47c4b20e66cef8fac2ab7`.
- Do not start SPEC-05 or SPEC-06; do not modify Hotel Staff; do not bump the
  version; do not create a tag or release; do not rewrite history.
- Do not implement the deferred independent historical-origin record design.
- Preserve unrelated user-owned files and stage only this fix, its tests, the
  plan, and maintained evidence.
- Keep the final implementation status exactly
  `SPEC-04: READY FOR FINAL ADVERSARIAL REVIEW`; never claim `CLOSED` here.
- Commit once after fresh verification and push the branch to GitHub. Do not
  create a pull request unless explicitly requested.

## Review focus

- A relative path outside `docs/baron/plans/` must fail before linked metadata
  can authorize completion or lifecycle mutation.
- Canonical/real containment must not be bypassed by a symlink or Windows
  reparse point, and only regular files may be authority targets.
- Authority metadata must come only from the leading frontmatter block; missing
  type, body-only fields, and duplicate known fields must fail closed.
- `set_plan_state`, `append_progress`, Vault mirroring, reconciliation, and Stop
  must not mutate or mirror an outside-root target.
- Existing legitimate managed plans and all previous SPEC-04 identity/proof/
  trace/gate invariants must remain green.

## Task 1: Add RED regressions for path and document authority

**Files:** `crates/baron-core/tests/plan.rs`

- Add focused fixtures/helpers that repoint CURRENT while preserving target
  bytes and, for hostile outside paths, supply canonical-looking metadata.
- Cover README, `src/fake.md`, unrelated `docs/fake-plan.md`, managed-root
  missing type marker, body-only metadata, duplicate `risk`, duplicate
  `operation_id`, and symlink/reparse targets where the platform permits.
- Assert rejection through the correctness-sensitive paths, no completion or
  Stop allowance, and unchanged repository/Vault target bytes where relevant.
- Run the new plan tests before production changes and record the expected RED
  failures in the execution ledger.

## Task 2: Implement managed path resolution and frontmatter-only parsing

**Files:** `crates/baron-core/src/plan.rs`

- Replace the generic CURRENT pointer join with a managed-plan resolver that
  rejects absolute/prefixed/backslash/parent paths, requires real containment
  below `docs/baron/plans/`, and rejects links/reparse points and non-regular
  files using the existing safe-I/O boundary.
- Require a valid leading frontmatter block and exact `type: baron-plan`.
- Parse required and optional authority fields only inside that block and
  reject duplicate known authority fields while tolerating benign unknown keys.
- Make `vault_plan_path()` return an error when the repository path is outside
  the managed root; update callers so mirroring/index mutation cannot fall
  back to an arbitrary repository path.
- Run the focused plan suite and the relevant regression matrix GREEN.

## Task 3: Verify all dependent consumers and preserve positive behavior

**Files:** `crates/baron-core/tests/plan.rs` (only if a missing consumer
regression is discovered), no unrelated production modules.

- Confirm `complete_plan`, `reconcile`, operation-bound trace ingress, and Stop
  all use the validated active-plan boundary.
- Confirm a normal Baron-created identified plan with valid proof, trace, and
  gates still completes, and legacy unbound compatibility remains intact.
- Run the required focused suites and Core all-targets. Investigate any new
  failure with systematic debugging; classify only the two documented baseline
  workspace blockers as out of scope if they remain unchanged.

## Task 4: Final review, evidence, and GitHub delivery

- Run formatter, focused suites, Core/workspace tests, Clippy, locked release
  build, binary version, diff checks, status JSON parsing, and the final static
  managed-path scan.
- Dispatch a fresh read-only code review against the branch diff; fix any
  Critical/Important finding with a RED→GREEN test pass before delivery.
- Update maintained current-state docs with the managed-root, regular-file,
  frontmatter, duplicate-field, mutation-confinement, test-count, blocker,
  and residual-origin evidence. Keep the required READY status.
- Review `git status`, `git diff`, and `git diff --check`; commit the narrow
  final fix and push `codex/spec-04-proof-trace-gate-completion-integrity`.
- Verify local and remote SHA equality and leave the managed worktree clean.
