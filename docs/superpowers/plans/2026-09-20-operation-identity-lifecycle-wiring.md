# Operation Identity & Lifecycle Wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close SPEC-02 by creating one validated lifecycle identity at ingress and propagating canonical task and operation identity through Prepare, Task State, native hooks, and supported CLI plan creation.

**Status:** complete on 2026-09-20; implementation commits are `b58c701`,
`4ee56e4`, `3771df8`, and `95a98a7`, with final verification recorded in the
maintained status and build log. The public version remains `5.0.0`.

**Architecture:** Add a validated `LifecycleIdentity` value object in `operation.rs`. Prepare and native hooks resolve optional ingress IDs exactly once, derive a task ID from only project plus canonical task text, derive an operation ID from the complete operation tuple, and expose a compatibility `OperationContext` view for existing diagnostic APIs. Task State and plan creation gain identity-aware entry points; legacy persisted plans remain readable and untrusted.

**Tech Stack:** Rust 2021, `serde`, SHA-256, `getrandom`, Cargo integration tests, `assert_cmd`, Markdown/Vault mirrors.

**Spec:** `fix-bug/spec/SPEC-02-operation-identity-lifecycle-wiring.md`

## Global Constraints

- Preserve `PrepareRequestV1` optional `session_id` and `request_id` fields for external compatibility.
- Every successful Prepare must construct and return a complete identity; no internal correctness-sensitive path may continue with missing IDs.
- Task identity uses exactly `project_id` plus canonical task text and excludes adapter, session, request, and operation IDs.
- Operation identity uses exactly `project_id`, `task_id`, adapter, `session_id`, and `request_id`.
- Missing ingress IDs are synthesized once with bounded, persist-safe, cross-process collision-resistant values; they are returned unchanged.
- Reject empty, overlong, newline, and control-character identity fields before persistence or lifecycle writes.
- New supported plans persist task, operation, adapter, session, and request binding in the historical artifact, repository CURRENT plan, and Vault mirror.
- Legacy unbound plans remain readable and are never upgraded by inference or silently hijacked.
- Do not bump the project schema, public version, tag, release, or modify Hotel Staff; do not implement SPEC-03 or SPEC-04.
- Preserve unrelated user-owned untracked files and stage exact paths only.

## Review Focus

- Anonymous Prepare calls must keep the same task ID but receive different synthesized operation/session/request identity; covered by Task 2.
- Blank, overlong, newline, and control-character supplied IDs must fail before lifecycle work; covered by Tasks 1 and 2.
- CRLF versus LF and outer whitespace must map to one task identity without changing meaningful task content; covered by Tasks 1 and 2.
- A changed request, session, or adapter must isolate operation identity while preserving task identity; covered by Task 1 and native parity tests in Task 3.
- A historical unbound plan must remain readable but cannot be resumed as an identified operation; covered by Task 4.

---

### Task 1: Add the canonical lifecycle identity and derivation helpers

**Files:**
- Modify: `crates/baron-core/src/operation.rs`
- Modify: `crates/baron-core/src/prepare.rs` for compatibility wrappers and shared validation errors
- Create: `crates/baron-core/tests/operation_identity.rs`

**Interfaces:**
- Produces `LifecycleIdentity::new`, `LifecycleIdentity::resolve`, accessors for all six fields, `OperationContext::from_identity`, and `OperationContext::lifecycle_identity(project_id)`.
- Produces one canonical `task_id_for_task(project_id, task)` helper and one operation derivation helper using explicit `baron-task-v2` and `baron-operation-v2` domains.
- Consumes only `SupportedAdapter`, validated strings, and the existing SHA-256/getrandom dependencies.

- [x] **Step 1: Write the failing identity regression tests.**

Add tests that assert the intended API and behavior:

```rust
#[test]
fn task_identity_ignores_session_request_and_line_ending_representation() {
    let first = task_id_for_task("project-1", "  Review API\r\ncontract  ").unwrap();
    let second = task_id_for_task("project-1", "Review API\ncontract").unwrap();
    assert_eq!(first, second);
}

#[test]
fn operation_identity_is_stable_only_for_the_same_complete_tuple() {
    let task = task_id_for_task("project-1", "Review API").unwrap();
    let first = LifecycleIdentity::new(
        "project-1", task.clone(), "operation-a", SupportedAdapter::Codex,
        "session-a", "request-a",
    ).unwrap();
    let same = operation_id_for_parts(
        first.project_id(), first.task_id(), first.adapter(),
        first.session_id(), first.request_id(),
    );
    assert_eq!(same, first.operation_id());
    assert_ne!(operation_id_for_parts("project-1", &task, SupportedAdapter::Codex, "session-a", "request-b"), first.operation_id());
    assert_ne!(operation_id_for_parts("project-1", &task, SupportedAdapter::Claude, "session-a", "request-a"), first.operation_id());
}

#[test]
fn identity_validation_rejects_unsafe_and_overlong_authority_fields() {
    assert!(LifecycleIdentity::new(
        "project-1", "task-1", "operation-1", SupportedAdapter::Codex,
        "session\nunsafe", "request-1",
    ).is_err());
    assert!(LifecycleIdentity::new(
        "project-1", "task-1", "operation-1", SupportedAdapter::Codex,
        &"x".repeat(MAX_IDENTIFIER_CHARS + 1), "request-1",
    ).is_err());
}

#[test]
fn resolving_anonymous_identity_generates_distinct_operations() {
    let first = LifecycleIdentity::resolve(
        "project-1", "same task", SupportedAdapter::Codex, None, None,
    ).unwrap();
    let second = LifecycleIdentity::resolve(
        "project-1", "same task", SupportedAdapter::Codex, None, None,
    ).unwrap();
    assert_eq!(first.task_id(), second.task_id());
    assert_ne!(first.operation_id(), second.operation_id());
    assert_ne!(first.session_id(), second.session_id());
    assert_ne!(first.request_id(), second.request_id());
}
```

- [x] **Step 2: Run the new test to verify RED.**

Run: `cargo test -p baron-core --test operation_identity`

Expected: compile failure because `LifecycleIdentity`, the canonical helpers, and their validation API do not yet exist.

- [x] **Step 3: Implement the minimal validated value object.**

In `operation.rs`, add a value object with required fields and private storage, validate all string fields for non-empty, maximum 256 Unicode scalar values, and `char::is_control`, and expose read-only accessors. Use `getrandom` to synthesize two independent 16-byte hex identifiers when `resolve` receives missing or blank values. Normalize task text by trimming and converting CRLF/CR to LF before deriving the task hash. Hash length-delimited/zero-separated inputs with the two explicit domain labels. Keep `OperationContext` as a compatibility/diagnostic projection and add conversion methods that reject incomplete optional state.

- [x] **Step 4: Make the new tests pass and run the focused Core target.**

Run: `cargo test -p baron-core --test operation_identity`

Expected: all identity tests pass with no warnings.

Run: `cargo test -p baron-core --test prepare --no-fail-fast`

Expected: existing Prepare tests still pass; any compile errors are fixed by adapting only the compatibility wrappers, not by changing the external request schema.

- [x] **Step 5: Commit the canonical identity unit.**

```powershell
git add crates/baron-core/src/operation.rs crates/baron-core/src/prepare.rs crates/baron-core/tests/operation_identity.rs
git commit -m "feat: add canonical lifecycle identity"
```

### Task 2: Wire Prepare, Task State, and context to one task identity

**Files:**
- Modify: `crates/baron-core/src/prepare.rs`
- Modify: `crates/baron-core/src/task_state.rs`
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-core/tests/prepare.rs`
- Modify: `crates/baron-core/tests/phase6_trusted_memory.rs` only if the compatibility function signature requires a mechanical call-site update

**Interfaces:**
- Consumes `LifecycleIdentity::resolve` and `OperationContext::from_identity` from Task 1.
- Produces `compile_task_state_for_operation(repo_root, vault, identity, task)` and an identity-aware context compiler used by Prepare.
- Keeps `compile_task_state(repo_root, vault, task)` as a read-only compatibility entry point using `task_id_for_task`.

- [x] **Step 1: Add RED tests for Prepare completion and Task State parity.**

Append tests to `crates/baron-core/tests/prepare.rs`:

```rust
#[test]
fn anonymous_prepare_resolves_and_returns_complete_identity() {
    let (_temp, repo) = initialized_project(AdapterKind::Codex);
    let first = prepare(PrepareRequestV1 {
        schema_version: 1, task: "same logical task".into(), session_id: None, request_id: None,
    }, "codex", &repo, None).unwrap();
    let second = prepare(PrepareRequestV1 {
        schema_version: 1, task: "same logical task".into(), session_id: None, request_id: None,
    }, "codex", &repo, None).unwrap();
    assert_eq!(first.task.id, second.task.id);
    assert!(first.session_id.as_deref().is_some_and(|id| !id.is_empty()));
    assert!(first.request_id.as_deref().is_some_and(|id| !id.is_empty()));
    assert!(first.operation_id.as_deref().is_some_and(|id| !id.is_empty()));
    assert_ne!(first.operation_id, second.operation_id);
}

#[test]
fn prepare_task_id_matches_operation_task_state() {
    let (_temp, repo) = initialized_project(AdapterKind::Codex);
    let vault = baron_core::vault::ensure_vault(
        &repo.parent().unwrap().join("Vault"), &repo,
    ).unwrap();
    let packet = prepare(PrepareRequestV1 {
        schema_version: 1, task: "  same task\r\nwith lines  ".into(),
        session_id: Some("session-1".into()), request_id: Some("request-1".into()),
    }, "codex", &repo, None).unwrap();
    let identity = LifecycleIdentity::resolve(
        &packet.project_id, "same task\nwith lines", SupportedAdapter::Codex,
        packet.session_id.as_deref(), packet.request_id.as_deref(),
    ).unwrap();
    let state = compile_task_state_for_operation(&repo, &vault, &identity, Some("same task\nwith lines")).unwrap();
    assert_eq!(packet.task.id, state.task_id);
}

#[test]
fn prepare_rejects_unsafe_or_overlong_supplied_identity_before_writes() {
    let (_temp, repo) = initialized_project(AdapterKind::Codex);
    let error = prepare(PrepareRequestV1 {
        schema_version: 1, task: "task".into(),
        session_id: Some("bad\nvalue".into()), request_id: Some("request-1".into()),
    }, "codex", &repo, None).unwrap_err();
    assert_eq!(error.code, PrepareErrorCode::InvalidInput);
    assert!(!repo.join("docs/baron/plans/CURRENT.md").exists());
}
```

- [x] **Step 2: Run the new tests to verify RED.**

Run: `cargo test -p baron-core --test prepare anonymous_prepare_resolves_and_returns_complete_identity -- --exact`

Expected: fail because Prepare currently returns `None` session/request and anonymous operations collapse.

Run: `cargo test -p baron-core --test prepare prepare_task_id_matches_operation_task_state -- --exact`

Expected: fail or compile-fail because Task State has no identity-aware entry point and its private hash differs from Prepare for normalized input.

- [x] **Step 3: Resolve identity once at Prepare ingress.**

Validate the request, load project identity, call `LifecycleIdentity::resolve`, replace request optional IDs with the resolved values, build the compatibility `OperationContext` from the identity, and use identity accessors for `task_id`, `operation_id`, session, and request throughout the route, context, gate, runtime, and response construction. Keep `PreparePacketV1` optionals for schema compatibility but always return `Some` on success.

- [x] **Step 4: Replace Task State's private hash and pass identity through context.**

Move the current body behind an internal function that accepts a canonical task ID. Make `compile_task_state` derive it through `task_id_for_task`; add `compile_task_state_for_operation` that verifies the identity belongs to the Vault project and uses its task ID. Make the operation-aware context compiler call this function so Prepare and native SessionStart context cannot drift from the packet identity.

- [x] **Step 5: Run RED tests again and verify GREEN.**

Run: `cargo test -p baron-core --test prepare --no-fail-fast`

Expected: all Prepare tests, including complete identity, normalization, and Task State parity, pass.

Run: `cargo test -p baron-core --test phase6_trusted_memory --no-fail-fast`

Expected: existing generic Task State compatibility tests pass.

- [x] **Step 6: Commit Prepare and Task State wiring.**

```powershell
git add crates/baron-core/src/prepare.rs crates/baron-core/src/task_state.rs crates/baron-core/src/context.rs crates/baron-core/tests/prepare.rs crates/baron-core/tests/phase6_trusted_memory.rs
git commit -m "fix: unify prepare and task state identity"
```

### Task 3: Make Codex and Claude native hooks use the ingress resolver

**Files:**
- Modify: `crates/baron-core/src/automation.rs`
- Modify: `crates/baron-core/tests/automation.rs`
- Modify: `crates/baron-core/tests/phase12_hooks.rs`

**Interfaces:**
- Consumes `LifecycleIdentity::resolve`, canonical task derivation, and `OperationContext::from_identity`.
- Produces hook journal/metadata carrying the resolved IDs and calls Prepare with the exact resolved session/request values.

- [x] **Step 1: Add RED parity tests.**

Add tests covering both `HookAdapter::Codex` and `HookAdapter::Claude` with anonymous payloads. Assert each hook response contains a non-empty task and operation identity, the two adapters produce the same task ID for the same project/task, and a repeated anonymous delivery produces a new operation identity when its event key differs.

- [x] **Step 2: Run the hook tests to observe RED.**

Run: `cargo test -p baron-core --test phase12_hooks --no-fail-fast`

Expected: the new assertions fail because automation currently hashes missing IDs as `None` and derives operation identity before any synthesis.

- [x] **Step 3: Resolve and propagate one identity in `handle_hook`.**

For Codex/Claude, call `LifecycleIdentity::resolve` using the Vault project ID, payload task, adapter, and supplied optional IDs before constructing `LifecycleEventKey`, `PrepareRequestV1`, or `OperationContext`. Write resolved IDs into the request and journal key, and use `OperationContext::from_identity` for continuity/context paths. Neutral diagnostic hooks keep their existing non-authoritative behavior and use only the canonical task helper.

- [x] **Step 4: Verify hook parity and existing idempotency.**

Run: `cargo test -p baron-core --test automation --no-fail-fast`

Expected: all automation tests pass.

Run: `cargo test -p baron-core --test phase12_hooks --no-fail-fast`

Expected: all hook lifecycle, recursion, deduplication, cross-adapter, and new identity parity tests pass.

- [x] **Step 5: Commit native hook parity.**

```powershell
git add crates/baron-core/src/automation.rs crates/baron-core/tests/automation.rs crates/baron-core/tests/phase12_hooks.rs
git commit -m "fix: propagate canonical identity through native hooks"
```

### Task 4: Bind supported CLI plan start atomically

**Files:**
- Modify: `crates/baron-core/src/plan.rs`
- Modify: `crates/baron-core/tests/plan.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-cli/tests/execution_cli.rs`

**Interfaces:**
- Consumes `LifecycleIdentity` and its conversion to compatibility `OperationContext`.
- Produces `start_or_resume_plan_for_identity(repo_root, vault, title, identity)` while retaining the old unbound API only for legacy/read-only compatibility tests.
- CLI `plan start` requires one atomic explicit group: `--adapter`, `--session-id`, and `--request-id`; task and operation IDs are derived from the title and project identity.

- [x] **Step 1: Add RED plan and CLI tests.**

Add a Core test that starts a plan with a `LifecycleIdentity`, asserts all five binding fields exist in the dated artifact, repository CURRENT, and Vault mirror, and asserts a different identity cannot resume it. Add a CLI test that invokes `plan start` without the complete identity group, expects a non-zero error, and asserts no plan/current/Vault plan write occurred. Update the existing nested-directory success test to provide all three flags.

- [x] **Step 2: Run focused tests to observe RED.**

Run: `cargo test -p baron-core --test plan --no-fail-fast`

Expected: the new identity API test does not compile or the binding assertion fails.

Run: `cargo test -p baron-cli --test execution_cli plan_commands_work_from_nested_directory -- --exact`

Expected: the existing command fails because the new atomic identity flags are not yet defined/accepted.

- [x] **Step 3: Add identity-aware plan entry point and preserve legacy behavior.**

Make `PlanOperationBinding` construct directly from `LifecycleIdentity`. Add `start_or_resume_plan_for_identity`; make the existing `start_or_resume_plan_for_operation` validate its compatibility `OperationContext` into a complete identity before delegating. Keep `start_or_resume_plan` readable for historical fixtures, but do not call it from the supported CLI. Ensure validation occurs before `plan_content`, `write`, index append, or CURRENT/Vault writes.

- [x] **Step 4: Wire CLI atomic fields.**

Extend `PlanCommands::Start` with `Option<AdapterArg>`, `Option<String> session_id`, and `Option<String> request_id`. Reject any partial group before calling the plan writer. Resolve the project config and Vault, construct `LifecycleIdentity::resolve` with the title and supplied complete IDs, then call `start_or_resume_plan_for_identity`. Use the identity's canonical task ID and derived operation ID; never consult serialized active adapter state.

- [x] **Step 5: Verify plan persistence, no-write failure, resume isolation, and CLI behavior.**

Run: `cargo test -p baron-core --test plan --no-fail-fast`

Expected: all plan tests pass, including legacy readability, complete binding, same-binding resume, different-binding rejection, and no partial write.

Run: `cargo test -p baron-cli --test execution_cli plan_commands_work_from_nested_directory -- --exact`

Expected: the nested-directory command succeeds with explicit identity flags and the resulting status remains interrupted after the existing update/interrupt flow.

- [x] **Step 6: Commit plan and CLI wiring.**

```powershell
git add crates/baron-core/src/plan.rs crates/baron-core/tests/plan.rs crates/baron-cli/src/main.rs crates/baron-cli/tests/execution_cli.rs
git commit -m "fix: require operation identity for cli plans"
```

### Task 5: Close SPEC-01, document SPEC-02, and run acceptance verification

**Files:**
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Create: `docs/superpowers/plans/2026-09-20-operation-identity-lifecycle-wiring.md` (this plan, if not already committed)

- [x] **Step 1: Record the SPEC-01 closure and SPEC-02 checkpoint.**

Mark Graphify SPEC-01 as closed with its pushed commit and known unrelated blockers, then add a top SPEC-02 stabilization entry recording the canonical identity design, proof/trace status, persisted-state boundary, and safe next action. Update status Markdown and JSON consistently without changing the public version or release fields.

- [x] **Step 2: Run all required focused and repository checks.**

Run the exact focused targets:

```powershell
cargo test -p baron-core --test operation_identity --no-fail-fast
cargo test -p baron-core --test prepare --no-fail-fast
cargo test -p baron-core --test plan --no-fail-fast
cargo test -p baron-core --test automation --no-fail-fast
cargo test -p baron-core --test phase12_hooks --no-fail-fast
cargo test -p baron-core --test control_plane --no-fail-fast
cargo test -p baron-cli --test prepare_cli --no-fail-fast
cargo test -p baron-cli --test execution_cli --no-fail-fast
```

Then run:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
git diff --check
target/release/baron --version
```

Expected: focused identity/Prepare/plan/automation/control-plane checks and formatting, Clippy, release build, diff check, and version smoke pass. The full workspace may still report only the known `Microsoft.PowerShell.Archive` installer failures and `session_replay` UTF-8 panic; report those unchanged and do not classify them as SPEC-02 regressions.

- [x] **Step 3: Perform a final self-review against the acceptance checklist.**

Confirm B-05, B-20, B-21, and W-10 are closed; no untracked user-owned file is staged; no schema/version/tag/release/Hotel Staff/SPEC-03/SPEC-04 change exists; and all newly created supported plan artifacts are fully bound.

- [x] **Step 4: Commit the maintained documentation and push the complete SPEC-02 branch once.**

```powershell
git add docs/superpowers/plans/CURRENT.md notes/build-log/CURRENT.md docs/BARON_STATUS.md docs/BARON_STATUS.json docs/superpowers/plans/2026-09-20-operation-identity-lifecycle-wiring.md
git commit -m "docs: close spec 1 and record spec 2 identity wiring"
git push -u origin codex/spec-02-operation-identity
```

### Final verification evidence

- Core all-targets: pass; focused identity, Prepare, Task State/context,
  automation, hooks, plan, and control-plane suites pass.
- CLI identity/plan and execution suites pass. `prepare_cli` has 4/5 passing
  tests; its one failure is the pre-existing `session_replay.rs:383` UTF-8
  boundary panic. The workspace reproduces the same issue plus three
  `lifecycle_scripts` failures caused by the unavailable host
  `Microsoft.PowerShell.Archive` module.
- Formatter, workspace Clippy with warnings denied, release build, binary
  version smoke (`baron 5.0.0`), and diff checks pass.
- No project schema/public version bump, tag, release, Hotel Staff, SPEC-03,
  or SPEC-04 change was made.
