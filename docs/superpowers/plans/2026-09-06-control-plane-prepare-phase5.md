# High-Level Control-Plane Prepare Protocol Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an explicit Codex/Claude `baron control-plane prepare --json` protocol that emits one bounded, versioned task packet by projecting existing Baron authorities.

**Architecture:** A new `baron_core::prepare` module will validate a minimal JSON request, resolve the Baron project from filesystem state, call the existing authority modules, and map their results into deliberate `PreparePacketV1` projection types. The CLI will only transport stdin/stdout, enforce bounded input, and map typed protocol errors to stable JSON envelopes and exit codes. Prepare will not classify risk, route skills, manage memory, or create a parallel task store.

**Tech Stack:** Rust, serde/serde_json, Clap, existing Baron Core authorities, existing bounded context compiler, existing Safe I/O/project state conventions, cross-platform `assert_cmd` integration tests.

**Spec:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md` (Phase 5 request: high-level `control-plane prepare` protocol).

## Global Constraints

- The request must contain schema version `1`, a non-empty task, and optional session/request identifiers only.
- Adapter identity is explicit and limited to `codex` or `claude`; it must never come from `active_adapter`, registration order, or installation order.
- Raw task text is stdin data. It is never interpolated into a shell command.
- Input is bounded before JSON parsing; task text remains Unicode/newline/quote/metacharacter data.
- Output is a bounded deliberate projection, not unrestricted serialization of every internal Rust struct.
- Existing authorities remain the source of truth for identity, authority, work shape, risk, intent, continuity, routing, profile, capabilities, context, proof, and trace.
- Core skill references point to `.baron/core/skills/<selected-skill>`; no adapter-local semantic tree is restored.
- Prepare does not add persisted project, managed-state, journal, transaction, or memory schema fields.
- Prepare does not implement Phase 6 trusted memory/Tier-0 changes, Phase 7 routing changes, bridge polish, active-adapter redesign, or adapter cleanup.

---

### Task 1: Freeze and promote the Phase 5 protocol contract

**Files:**
- Modify: `crates/baron-cli/tests/phase1_target_red.rs`
- Create: `crates/baron-cli/tests/prepare_cli.rs`
- Test fixtures: existing Phase 1 adversarial task fixture and deterministic temporary repositories created by the integration tests.

**Interfaces:**
- Consumes: the future `baron control-plane prepare --adapter <codex|claude> --json` command and stdin JSON request.
- Produces: executable success/error, adapter identity, resume, routing, verification, bounded-output, and preservation expectations for Tasks 2–3.

- [x] **Step 1: Update the existing adversarial target fixture to establish Baron project state.**

  Keep the Unicode, Vietnamese, quotes, newlines, shell metacharacters, JSON-looking text, and long task body. Initialize the temporary repository with the existing Codex helper before invoking prepare so the test exercises the specified missing-project error separately instead of relying on an uninitialized repository.

- [x] **Step 2: Change the malformed-input assertion to the Phase 5 JSON contract.**

  Assert non-zero exit, JSON on stdout containing `schema_version`, `ok: false`, and `error_code`, and an empty stdout-free diagnostic stream rather than requiring human prose on stderr.

- [x] **Step 3: Add focused CLI tests for the complete protocol boundary.**

  Cover:
  - explicit Codex and explicit Claude requests in the same initialized project;
  - unsupported `agent`/legacy adapter input with the unsupported-adapter exit code;
  - missing project state with the project-state exit code;
  - missing schema/task, malformed JSON, oversized JSON, and empty task with the invalid-input exit code;
  - confirmed intent and required-confirmation blocker projection;
  - interrupted plan/recovery projection with `task.resumed == true` and safe next action;
  - route-selected skills/agents and canonical `.baron/core` skill references;
  - proof/trace/gate verification projection;
  - deterministic repeated output for identical input and unchanged state;
  - no shell side effect from `$()`, backticks, pipes, redirection, CRLF, or Unicode task data;
  - bounded packet size and machine-readable stdout without human headings.

- [x] **Step 4: Run the new and promoted tests before implementation.**

  Run:

  ```text
  cargo test -p baron-cli --test phase1_target_red -- --ignored target_prepare_protocol_accepts_adversarial_structured_task_input target_prepare_protocol_reports_malformed_payload_as_structured_error
  cargo test -p baron-cli --test prepare_cli
  ```

  Expected: the prepare command tests fail because the command and protocol do not yet exist. Do not weaken the target assertions.

### Task 2: Add the deliberate Core `PreparePacketV1` projection

**Files:**
- Create: `crates/baron-core/src/prepare.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Test: `crates/baron-core/tests/prepare.rs`

**Interfaces:**
- Consumes: `PrepareRequestV1`, a repository start path, and an optional explicit Vault path.
- Produces: `decode_request(bytes) -> Result<PrepareRequestV1, PrepareError>` and `prepare(request, adapter, repo_start, vault_override) -> Result<PreparePacketV1, PrepareError>` plus serializable success/error projection types consumed by the CLI.

- [x] **Step 1: Define bounded request and typed error contracts.**

  Add:

  ```rust
  pub const PREPARE_SCHEMA_VERSION: u32 = 1;
  pub const PREPARE_MAX_INPUT_BYTES: usize = 128 * 1024;
  pub const PREPARE_MAX_TASK_CHARS: usize = 96 * 1024;

  #[derive(Debug, Deserialize)]
  pub struct PrepareRequestV1 {
      pub schema_version: u32,
      pub task: String,
      #[serde(default)] pub session_id: Option<String>,
      #[serde(default)] pub request_id: Option<String>,
  }

  #[derive(Debug, Serialize)]
  pub struct PreparePacketV1 {
      pub schema_version: u32,
      pub ok: bool,
      pub project_id: String,
      pub adapter: String,
      pub session_id: Option<String>,
      pub request_id: Option<String>,
      pub task: PrepareTask,
      pub intent: PrepareIntent,
      pub work_shape: PrepareWorkShape,
      pub risk: RiskLane,
      pub profile: PrepareProfile,
      pub route: PrepareRoute,
      pub context: PrepareContext,
      pub continuity: PrepareContinuity,
      pub verification: PrepareVerification,
      pub blockers: Vec<PrepareIssue>,
      pub warnings: Vec<PrepareIssue>,
      pub unknowns: Vec<String>,
      pub next_action: PrepareNextAction,
  }
  ```

  Define `PrepareErrorCode` values `invalid_input`, `unsupported_adapter`, `project_state`, and `internal`, with deterministic exit codes `2`, `3`, `4`, and `5`. Serialize failures as `{ "schema_version": 1, "ok": false, "error": { "error_code": ..., "message": ..., "details": {} } }`.

- [x] **Step 2: Implement request validation without shell or persisted-state behavior.**

  Validate schema version, non-empty task, task/identifier bounds, and explicit adapter values. Ignore unknown JSON fields for forward-compatible transport, but never accept a user-supplied project identity in place of Baron project state. Derive a deterministic task id from project id, task bytes, session id, and request id with the existing `sha2` dependency.

- [x] **Step 3: Compose existing authorities into projections.**

  Resolve the project with `find_project_root` and `load_project_config`; map the explicit adapter to `AdapterKind` and `ContextTarget`; resolve the Vault through `resolve_vault_path_for_repo`; then call:

  - `classify_request` and `decide_work_shape` for authority, work shape, risk, lifecycle, durability, proof requirement, and next action;
  - `validate_control_plane` and `route_task` for route explanation, selected skills, selected agents, skipped routes, and mandatory gates;
  - `intent_status`, `plan_status`, `harness_status`, and `continuity_status` for bounded current intent, plan, harness, recovery, and resume projections;
  - `render_platform_context` and the configured `ProjectPlatform` for the current profile lens;
  - `compile_context_for_task` for the existing bounded context compiler, truncated to a fixed packet limit;
  - `latest_proof`, `latest_trace_score`, `gate_evidence_status_strict`, and `runtime_backend_report` for existing proof, trace, gate, capability, and runtime requirements.

  The projection parser may extract known fields from existing bounded Markdown status output, but it must not reclassify risk, route skills, infer intent confirmation, or create a second continuity/memory engine.

- [x] **Step 4: Project canonical Core references and proportional issues.**

  Emit selected skill ids with `.baron/core/skills/<id>` references, mandatory/optional agent ids with `.baron/core/agents/<id>` references, and never enumerate the full Core tree. Add structured blockers for missing project state, invalid control-plane contracts, required confirmation, unresolved recovery, missing required capability evidence, or failing required proof/trace state. Add warnings for missing optional capability/profile data and keep unknown facts in `unknowns`.

- [x] **Step 5: Keep the packet bounded and side-effect explicit.**

  Cap task summary, status source excerpts, selected lists, and compiled context. Do not write a new task database or protocol state. The only permitted durable behavior is the existing context compiler's current Vault/cache/session-index lifecycle; document and test that no new repo task files, plan files, intent files, or recovery files are created by prepare itself.

- [x] **Step 6: Add Core unit tests and run them red first, then green after implementation.**

  Test request validation, deterministic task ids, explicit adapter projection, canonical skill paths, bounded context, missing/confirmed intent, interrupted recovery, route/verification projection, and typed error envelopes.

### Task 3: Wire the CLI transport and deterministic exit behavior

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-cli/tests/prepare_cli.rs`

**Interfaces:**
- Consumes: `baron_core::prepare::{prepare, PrepareRequestV1, PrepareError}`.
- Produces: `baron control-plane prepare --adapter codex|claude [repo-path] [--vault <path>] --json`.

- [x] **Step 1: Add the typed Clap command without a `ValueEnum` adapter restriction.**

  Add `ControlPlaneCommands::Prepare { repo_path: Option<PathBuf>, adapter: String, vault: Option<PathBuf>, json: bool }`. Keep adapter as a string so unsupported values reach the stable JSON error envelope instead of Clap's human-only parser failure.

- [x] **Step 2: Read bounded stdin safely.**

  Use `Read::take(PREPARE_MAX_INPUT_BYTES + 1)` and reject oversized input before JSON decoding. Pass bytes directly to `serde_json::from_slice`; never construct a shell command or invoke a shell with task content.

- [x] **Step 3: Add a CLI-only error bridge for JSON envelopes and exit codes.**

  Define a `PrepareCommandFailure` error carrying `PrepareError` and the requested output mode. Update the top-level error branch to print only the serialized error envelope to stdout in JSON mode and exit with the typed code; retain ordinary stderr diagnostics for non-JSON commands. Successful JSON mode prints exactly one serialized `PreparePacketV1` value.

- [x] **Step 4: Add a compact non-JSON rendering without changing packet semantics.**

  Human mode may show project, adapter, task id, route, blockers, warnings, and next action, but JSON mode must never include headings or log prose.

- [x] **Step 5: Run CLI tests for both explicit adapters and all error classes.**

  Verify Codex and Claude never read active-adapter state, unsupported adapters return exit code 3, malformed/oversized input returns 2, missing project returns 4, internal failures return 5, success returns 0, and stdout remains valid JSON.

### Task 4: Promote evidence, update durable status, and run the Phase 5 gates

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-09-06-control-plane-prepare-phase5.md`

- [x] **Step 1: Promote the two Phase 5 target tests once green.**

  Remove `#[ignore]` only from the adversarial structured-input and malformed-payload tests after their assertions pass. Keep Phase 6, 7, 8, 9, 10, and 11 target tests ignored with their original reasons.

- [x] **Step 2: Run focused verification.**

  ```text
  cargo test -p baron-core --test prepare
  cargo test -p baron-cli --test prepare_cli
  cargo test -p baron-cli --test phase1_target_red
  cargo test -p baron-adapters --test phase4_core
  ```

- [x] **Step 3: Run required repository verification.**

  ```text
  cargo fmt --all -- --check
  cargo test --workspace --all-targets --no-fail-fast
  cargo clippy --workspace --all-targets -- -D warnings
  git diff --check
  ```

  Classify the two known Windows `Microsoft.PowerShell.Archive` lifecycle failures separately from Baron product failures.

- [x] **Step 4: Record exact evidence and stop before Phase 6.**

  Update the status dashboard, build checkpoint, and this plan with packet fields, side effects, test counts, structured exit behavior, schema result, and any newly discovered authority limitations. Do not implement trusted memory/Tier-0 changes, profile-aware routing, bridge polish, active-adapter redesign, or adapter cleanup.

## Plan self-review

- Spec coverage: request/input bounds, explicit adapters, packet versioning, authority reuse, bounded context, resume/intent/work-shape/risk/routing/Core/agent/verification projections, blockers/unknowns, JSON errors, exit codes, idempotent read behavior, adversarial input, Phase 4 regression, schema discipline, and final evidence are covered above.
- No persisted schema bump is planned; the protocol version is independent.
- The only deliberate existing side effect is the current context compiler lifecycle already used by Baron context commands; prepare adds no parallel durable state.
- Phase 6–11 target tests remain ignored and will not be weakened.

## Phase 5 execution evidence (2026-09-06)

- Implemented `baron_core::prepare` and the CLI `control-plane prepare`
  transport. The request is version 1 JSON on stdin; adapter identity is an
  explicit `--adapter codex|claude` flag. Success is one bounded packet on
  stdout; JSON failures use typed error codes and exit codes 2–5.
- Focused commands and results:
  - `cargo test -p baron-core --test prepare --no-fail-fast` — `4/4` passed.
  - `cargo test -p baron-cli --test prepare_cli --no-fail-fast` — `5/5` passed.
  - `cargo test -p baron-cli --test phase1_target_red --no-fail-fast` — `2`
    passed, `5` ignored for later phases.
  - `cargo test -p baron-adapters --test phase4_core --no-fail-fast` — `9/9`
    passed.
- Required checks:
  - `cargo fmt --all -- --check` — passed.
  - `cargo clippy --workspace --all-targets -- -D warnings` — passed.
  - `git diff --check` — passed.
  - `cargo test --workspace --all-targets --no-fail-fast` — all observed
    product tests passed. The command exited non-zero only because
    `baron-cli --test lifecycle_scripts` has the two pre-existing Windows
    installer failures `powershell_installer_makes_baron_available_in_the_current_session`
    and `native_installer_supports_install_update_rollback_and_uninstall`; both
    require the unavailable `Microsoft.PowerShell.Archive` module and are an
    environment limitation.
  - `docs/BARON_STATUS.json` parsed successfully with PowerShell
    `ConvertFrom-Json`.
- Adversarial evidence includes Unicode/Vietnamese, quotes, apostrophes,
  backticks, newlines, JSON-looking text, shell metacharacters, a bounded
  long task, oversized input rejection, deterministic identity/route fields,
  and proof that no shell command or task-derived file is created.
- No persisted Baron schema changed. Existing context compilation may perform
  its established Vault/cache/session-index lifecycle; prepare itself creates
  no task, plan, intent, recovery, or parallel memory store.
- Phase 6–11 target tests remain intentionally ignored. Stop here; do not
  begin Phase 6 in this checkpoint.
