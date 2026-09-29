# BARON ENGINE v5.0.0 / 5.0.1 STABILIZATION — DEEP ENGINEERING CONTEXT FOR AI AGENTS

> **Purpose:** This file is a high-density engineering context package for AI agents (Codex/Claude/other code agents) that must inspect, plan, or repair Baron Engine v5.0.0 without first rediscovering the repository from scratch.
>
> **Target snapshot:** Public Baron Engine **v5.0.0** plus the active Baron 5.0.1 stabilization program, observed **2026-09-29**.
>
> **Pinned public release commit:** `cb2940e79fe4ed4cda7c52ea64c46c049b189514`
>
> **Repository:** `thienty1207/Baron-Engine`
>
> **Audit scope:** release-relevant production surfaces, Rust core/adapters/CLI, lifecycle, trusted execution receipts, proof/trace/gates, Vault/state, memory/session/code graph, Graphify provider, concurrency/durability, migration, update/release, and CI/release verification.
>
> **Important:** This context deliberately distinguishes:
> - **design intent / documentation claims**
> - **actual production wiring**
> - **confirmed defects**
> - **high-confidence latent defects**
> - **future warnings / architectural debt**
>
> Do not assume that a documented invariant is true merely because a phase or status page says “completed”. Verify production call paths.

---

# 0. Current State — 2026-09-29

This section is the live status for the repository. The detailed v5.0.0 audit
and defect descriptions below are retained as engineering provenance; where a
historical section conflicts with this section, this section and the active
SPEC/status documents are authoritative.

## 0.1 Public release boundary

- Public/stable Baron version: `5.0.0`.
- Public release commit: `cb2940e79fe4ed4cda7c52ea64c46c049b189514`.
- The stabilization program is named **Baron 5.0.1 Stabilization**, but no
  `5.0.1` version bump, tag, release, or publication has happened.
- The stabilization branches are delivery branches layered after the public
  `5.0.0` release. Their work is not assumed to be merged into `main`.
- Codex and Claude remain the only supported native adapters.

## 0.2 Stabilization progress

The program has completed implementation and closure through SPEC-04. SPEC-05
implementation is complete and locally self-reviewed, but its formal status is
still **READY FOR ADVERSARIAL REVIEW**, not closed.

| Spec | Current status | Current branch head | Scope result |
| --- | --- | --- | --- |
| SPEC-01 Graphify Provider Contract | CLOSED | `codex/spec-01-graphify-provider-contract` @ `adccb6e` | Real Graphify 0.9.25 output/query contract, bounded fallback, and release gate are implemented. |
| SPEC-02 Operation Identity & Lifecycle Wiring | CLOSED | `codex/spec-02-operation-identity` @ `3a52668` | Canonical lifecycle identity, ingress synthesis/validation, task identity, and operation-bound Plan wiring are implemented. |
| SPEC-03 Trusted Execution Receipt Architecture | CLOSED | `codex/spec-03-trusted-receipt-architecture` @ `fa1bd91` | Cross-process receipt authority, canonical task/seed follow-up, and safe receipt persistence are implemented and freshly verified. |
| SPEC-04 Proof / Trace / Gate / Completion Integrity | CLOSED | `codex/spec-04-proof-trace-gate-completion-integrity` @ `ecaf362` | Exact-operation proof/trace/gate completion and managed-plan path authority are closed. |
| SPEC-05 Multi-Agent Concurrency & Durable State | READY FOR ADVERSARIAL REVIEW | `codex/spec-05-multi-agent-concurrency-durable-state` @ `eb7624b` | Authoritative mutation locking, collision-resistant durable IDs, no-overwrite publication, and concurrency coverage are implemented; independent adversarial acceptance is still pending. |
| SPEC-06 Repo/Vault Transaction & Safe Persistence | NOT STARTED | — | Next stabilization scope; do not infer implementation from SPEC-05. |
| SPEC-07 Execution Runner & Resource Hardening | NOT STARTED | — | Deferred until SPEC-06 is complete. |
| SPEC-08 Cache, Session, E2E & Release Certification | NOT STARTED | — | Final certification scope; deferred until SPEC-07 is complete. |

## 0.3 Current SPEC-05 truth

SPEC-05 covers only B-09, B-10, B-11, B-18, B-25, B-26, B-27, W-05, and
W-08. It does not introduce the SPEC-06 repository/Vault transaction
framework, does not repair the execution process-tree/resource items, and does
not close the session/replay/cache items mapped to SPEC-08.

The implementation now provides:

- one project-scoped OS mutation lock for authoritative read→modify→write
  paths, with reentrant same-thread support and bounded timeout/no-write
  failure;
- locked publication for proof, trace, plan, config, harness, intent,
  continuity, control-plane, review-gate, experiment, Autopilot, automation
  hook claims/journals/responses, and capability registry/runtime evidence;
- collision-resistant immutable IDs and create-new/no-overwrite publication
  for new proof, trace, experiment, and plan instances;
- exact plan resume paths and preservation of legacy proof/trace/plan
  readability;
- slow child commands, network work, capability probes, Git status, and
  context compilation outside the mutation critical section;
- migration lock boundaries and fail-closed post-handoff rollback that refuse
  to clobber changed state.

The implementation evidence is on the pushed SPEC-05 branch at `eb7624b`.
The local self-review found no currently known SPEC-05-owned Critical or
Important issue. Two independent read-only adversarial review retries hit the
Codex usage limit before returning a verdict, so this context must not claim
independent acceptance or SPEC-05 closure.

## 0.4 Current verification boundary

Fresh evidence for SPEC-05 includes:

- `cargo fmt --all -- --check`: pass;
- `cargo test -p baron-core --all-targets --no-fail-fast -j 1`: pass;
- `cargo clippy --workspace --all-targets -- -D warnings`: pass;
- `cargo build --workspace --release --locked`: pass;
- release binary reports `baron 5.0.0`;
- `git diff --check`, status JSON parsing, repository-relative link checks,
  and the retired-adapter gate: pass;
- deterministic concurrency matrix: concurrency `15/15`, config `16/16`,
  plan `44/44`, proof/trace `20/20`, capability `9/9`, automation `5/5`,
  harness `5/5`, harness improvement `5/5`, intent `4/4`, continuity `6/6`,
  control-plane `9/9`, migration `10/10`.

The complete workspace test run is not entirely green because of two known
baseline/host blockers outside SPEC-05: three `baron-cli` lifecycle-script
tests cannot load the host `Microsoft.PowerShell.Archive` module for
`Compress-Archive`, and one `prepare_cli` test reaches the existing
`crates/baron-core/src/session_replay.rs:383` UTF-8 boundary panic. These are
reported failures, not expected-red evidence and not SPEC-05-owned findings.

## 0.5 Safe next action

Obtain an independent adversarial verdict for the complete SPEC-05 range
`ecaf362..eb7624b` when reviewer quota is available. Until that happens, keep
the status exactly **READY FOR ADVERSARIAL REVIEW**. Do not bump the public
version, create a release/tag, begin SPEC-06, or describe SPEC-05 as closed.

The checkout at `D:\Works\Baron-Engine` may be on an earlier stabilization
branch and may contain user-owned untracked files. Preserve those files and
inspect the active branch/worktree before making changes.

---

# 1. Executive Summary

Baron Engine is a **Rust-first project memory, task context, routing, continuity, and evidence-backed work engine**. Its intended architecture is:

```text
assets/core/**
      ↓
.baron/core/**
      ↓
Codex bridge / Claude bridge
```

The product model is intentionally adapter-neutral:

- **Baron Core** owns project identity, Vault memory, intent, task state, plans, routing, proof, trace, gates, continuity, recovery, session replay, code intelligence, and Autopilot semantics.
- **Codex** and **Claude** are intended to be thin native bridges.
- A project should have:
  - one stable project identity,
  - one Vault boundary,
  - one task history,
  - one shared lifecycle,
  - one authoritative evidence model,
  regardless of which supported host is used.

The workspace contains three Rust crates:

```text
crates/
  baron-core/
  baron-adapters/
  baron-cli/
```

Workspace version at the audited release:

```toml
version = "5.0.0"
```

The remainder of this executive-summary audit describes the original public
v5.0.0 baseline. The live implementation status is in Section 0; do not read
the historical examples below as proof that the same defect is still open.

Baron v5.0.0 has several areas that are genuinely hardened, especially:

- signed release/update verification,
- update transaction/rollback logic,
- safe file replacement primitives,
- symlink/reparse-point protection in hardened paths,
- project/Vault identity validation,
- current trusted-memory filtering.

At the original v5.0.0 audit boundary, the dominant failure pattern was:

> **The primitive exists and is often correct, but the real production workflow does not always use it.**

The historical examples were addressed by the stabilization branches as
follows:

- the SPEC-02 branch wires operation-aware Plan creation and canonical
  lifecycle identity;
- the SPEC-03 branch resolves receipt authority across the real process
  boundary and removes the generic trusted-execution path;
- the SPEC-01 branch aligns Graphify with the real 0.9.25 provider contract
  and adds real-provider release coverage;
- the SPEC-04 branch binds proof/trace/gates/completion to exact operation
  authority;
- the SPEC-05 branch covers the owned authoritative writers with the project
  mutation lock and collision-resistant publication, pending independent
  adversarial acceptance.

The remaining stabilization boundary is therefore:

> **Strong primitives with SPEC-01..SPEC-05 lifecycle/concurrency repairs in
> place, while repo/Vault transactions, runner resources, cache/session
> recovery, and final cross-spec certification remain future scope.**

---

# 2. Release Boundary — Do Not Mix Versions

All repair work derived from this context must begin from this exact release boundary unless a spec explicitly says otherwise:

```text
Baron version: 5.0.0
Release commit: cb2940e79fe4ed4cda7c52ea64c46c049b189514
```

Important release facts:

- `Cargo.toml` workspace version = `5.0.0`.
- `README.md` identifies `5.0.0` as source/public release.
- `CHANGELOG.md` says v5.0.0 completes Codex + Claude Core consolidation and the hardened release/update boundary.
- Production Graphify integration pins:
  - `SUPPORTED_GRAPHIFY_VERSION = "0.9.25"`
  - audited Graphify revision:
    `2fa6cd3d5548577f8c5f591b713f0bf80c1af183`

An agent repairing v5.0.0 must **not** silently read a newer `main` implementation and assume it reflects the release under audit.

When comparing a proposed repair:

1. inspect v5.0.0 behavior;
2. identify the exact failing invariant;
3. design the minimal architectural repair;
4. use newer code only as a reference if explicitly requested;
5. do not silently transplant future behavior into the release.

---

# 3. Product Intent

From the release README, Baron prepares a bounded, evidence-aware work packet before an agent changes a repository.

Core owns:

- project identity;
- Vault-backed trusted memory;
- task intent;
- constraints and non-goals;
- task state;
- plan and recovery;
- profile-aware routing;
- work-shape decisions;
- Superpowers workflow;
- mandatory quality agents;
- proof;
- trace;
- control-plane gates;
- continuity;
- optional hooks;
- session replay;
- Wiki/CodeGraph accelerators;
- Autopilot housekeeping;
- reviewable learning candidates.

The supported native adapters in v5.0.0 are:

```text
Codex
Claude
```

The adapter layer must not create a second memory or workflow system.

---

# 4. Workspace and Ownership Boundaries

## 4.1 Rust workspace

```text
Cargo.toml
├── crates/baron-core
├── crates/baron-adapters
└── crates/baron-cli
```

### `baron-core`

The semantic brain.

It contains modules for:

- identity,
- project config,
- Vault,
- task state,
- prepare protocol,
- operation identity,
- plans,
- proof,
- trace,
- execution receipts,
- control plane,
- automation/hooks,
- continuity/recovery,
- memory/firewall,
- session import/replay,
- Wiki/code graph,
- Graphify provider,
- migration,
- release/certification,
- safe I/O,
- evaluation/intelligence.

### `baron-adapters`

Owns native adapter projection/install/update behavior.

Key conceptual rule:

```text
Core owns semantics.
Adapters own host-native projection.
```

`AgentAdapter` only supports:

```rust
Codex
Claude
```

The adapter crate also owns preserve-first managed projections and update reconciliation.

### `baron-cli`

Binary:

```text
baron
```

The CLI is the orchestration surface for setup/init/update/diagnostics and many internal lifecycle commands.

A major audit lesson is that **CLI wiring must be audited independently of Core APIs**. A correct Core API does not prove the CLI uses it.

---

# 5. `baron-core` Source Map

`crates/baron-core/src/lib.rs` exports the following production modules.

This section gives an AI agent a semantic map before editing.

## 5.1 Architecture / identity / project state

### `architecture.rs`
Architecture context/governance and structural understanding.

### `identity.rs`
Stable project identity primitives.

Important functions include:

```rust
project_id_for_path(...)
capsule_key(...)
identity(...)
new_identity_binding(...)
```

`new_identity_binding()` generates a random 32-byte project binding rendered as 64 hex characters.

### `config.rs`
Project/local configuration, profile, adapter selection, Vault binding, schema validation.

Important audit note:
- newer schema rejection is correctly fail-closed in v5.0.0;
- some config mutation paths still have read→modify→replace concurrency risk.

### `state_guard.rs`
Validates coherent execution state before trusted work.

Checks include:

- project config readable,
- exact project schema,
- non-empty project ID,
- valid 64-hex identity binding,
- configured Vault equals requested Vault,
- project capsule exists,
- capsule schema is correct,
- capsule identity matches repo identity.

This is one of the stronger v5.0.0 boundaries.

### `vault.rs`
Vault/capsule construction and project-bound durable state.

---

## 5.2 Request authority / operation identity / preparation

### `authority.rs`
Classifies a user request into:

```text
read_only
change
ambiguous
```

Uses explicit change/read-only term matching and defaults ambiguous requests to read-only.

### `operation.rs`
Shared operation identity model for supported adapters.

Operation identity is intended to carry:

- project/task,
- adapter,
- session,
- request,
- operation identity.

### `operations.rs`
Higher-level operation helpers and shared operation behavior.

### `prepare.rs`
Structured preparation protocol.

Key constants include:

```rust
PREPARE_SCHEMA_VERSION = 1
PREPARE_MAX_INPUT_BYTES = 128 KiB
PREPARE_MAX_TASK_CHARS = 96 KiB
PREPARE_MAX_OUTPUT_CONTEXT_CHARS = 8_000
MAX_IDENTIFIER_CHARS = 256
```

`PrepareRequestV1` currently allows:

```rust
session_id: Option<String>
request_id: Option<String>
```

This is important because later authoritative lifecycle stages often require those identities to exist.

Key functions:

```rust
decode_request(...)
prepare(...)
task_id_for_request(...)
operation_id_for_request(...)
```

---

## 5.3 Intent / plan / task lifecycle

### `intent.rs`
Intent capture and shared-understanding state.

### `task_state.rs`
Protected task-state projection used in prepared context.

Intended to retain:

- task identity,
- intent,
- constraints,
- non-goals,
- plan/work state,
- last successful step,
- failure/interruption,
- affected files,
- proof/trace,
- blockers,
- recovery,
- routing,
- mandatory gates,
- next safe action.

### `work_shape.rs`
Determines lifecycle depth/durability/judgment requirements.

### `risk.rs`
Risk classification.

### `plan.rs`
Durable plan lifecycle.

Public API:

```rust
start_or_resume_plan(...)
start_or_resume_plan_for_operation(...)
update_plan(...)
interrupt_plan(...)
complete_plan(...)
plan_status(...)
```

Critical distinction:

```rust
start_or_resume_plan(...)
```

is legacy/title-only.

```rust
start_or_resume_plan_for_operation(...)
```

captures exact operation binding.

`PlanOperationBinding` includes:

```rust
task_id
operation_id
adapter
session_id
request_id
```

The source comments explicitly state that legacy title-only plans remain readable, but medium/high-risk completion cannot authorize them without a complete binding.

---

## 5.4 Proof / trace / quality control

### `execution_receipt.rs`
Baron-owned command execution evidence.

Important constants:

```rust
RECEIPT_PATH = ".baron/cache/execution-receipts.jsonl"
MAX_CAPTURE_BYTES = 64 KiB
MAX_ARG_BYTES = 16 KiB
```

Public API includes:

```rust
execute_command(...)
execute_command_with_context(...)
load_receipts(...)
receipt_is_current(...)
receipt_is_current_authority(...)
receipt_matches_context(...)
```

`ReceiptContext` includes:

```rust
task_id
operation_id
adapter
session_id
request_id
gate_kind
```

The module has:

```rust
CURRENT_RECEIPTS: OnceLock<Mutex<BTreeMap<String, String>>>
RECEIPT_SEQUENCE: AtomicU64
```

This process-local authority registry is central to a major v5.0.0 defect described later.

### `proof.rs`
Records and evaluates proof artifacts.

Important concept:
- free-form/legacy proof is readable;
- trusted medium/high completion should require operation-bound current receipt evidence.

### `trace.rs`
Records and scores trace artifacts.

Public API:

```rust
record_trace(...)
score_trace(...)
latest_trace_score(...)
```

Important audit finding:
- Trace is not equivalently bound to an operation identity in v5.0.0.

### `review_gate.rs`
Reviewer closure/quality gate logic.

### `control_plane.rs`
Routing and mandatory quality gates.

Core mandatory agents:

```text
code-reviewer
security-auditor
test-engineer
```

Public gate APIs include:

```rust
record_gate_evidence(...)
record_gate_evidence_with_receipt(...)
record_gate_evidence_with_receipt_bound(...)

gate_evidence_status_strict(...)
gate_evidence_status_strict_for_scope(...)
gate_evidence_status_strict_for_request(...)
gate_evidence_status_strict_for_operation(...)
gate_evidence_status_strict_for_context(...)
```

---

## 5.5 Automation / continuity / recovery

### `automation.rs`
Hook lifecycle, journaling, automation coordination.

This subsystem is a positive reference because it uses project mutation locking in important write paths.

### `continuity.rs`
Checkpoint and recovery state.

### `autopilot.rs`
Post-task review, bounded housekeeping, improvement proposals/candidates.

---

## 5.6 Memory / trusted recall

### `memory.rs`
Durable memory structures, indexing/consolidation support.

### `firewall.rs`
Current trusted-memory eligibility.

Important APIs:

```rust
trusted_recall(...)
trusted_recall_current(...)
trusted_current_records(...)
record_is_currently_trusted(...)
```

Current trusted recall filters out:

- candidate,
- contested,
- superseded,
- expired,
- invalid temporal records,
- non-current project memory unless explicitly allowed,
- unapproved global candidates.

The v5 semantic ranking stage is applied after the trust/identity gate.

Audit result:
- no release-blocking semantic bypass of current-project/global-verified eligibility was found in the reviewed path.

### `semantic.rs`
Ranking/semantic scoring utilities.

### `knowledge.rs`
Wiki/local knowledge/code intelligence support.

### `intelligence.rs`
Legacy/default intelligence flow.

### `intelligence41.rs`
4.1-generation intelligence behavior.

### `intelligence42.rs`
4.2-generation behavior and candidate learning.

### `evaluation41.rs`
4.1 benchmark/evaluation.

### `evaluation42.rs`
4.2 benchmark/evaluation.

---

## 5.7 Session intelligence

### `session.rs`
Imports Codex/Claude session artifacts into project-scoped Vault session notes.

Important characteristics:

- repository-segment matching,
- message extraction,
- redaction,
- content-hash deduplication,
- imported Markdown notes,
- `session-import-state.json`.

Important latent defects are documented later:
- state parse errors silently default,
- read→modify→replace is not globally serialized.

### `session_replay.rs`
SQLite replay index over imported session Markdown.

Public API:

```rust
index_session_replay(...)
search_session_replay(...)
replay_session_context(...)
render_session_replay_hits(...)
```

Query paths correctly include current `project_id`.

The SQLite database is a rebuildable accelerator, not durable truth.

---

## 5.8 Code intelligence

### `survey.rs`
Repository survey and project type detection.

### `code_graph.rs`
Internal/local code graph cache state and query cache.

Important safety helpers:

```rust
validate_code_graph_cache_path(...)
compute_code_source_fingerprint(...)
write_code_graph_state(...)
load_code_graph_state(...)
write_code_graph_query_cache(...)
load_code_graph_query_cache(...)
verify_graph_hit_source(...)
```

Cache path validation includes repository containment and symlink/junction checks.

### `graphify.rs`
Optional native Graphify provider.

Pinned contract in v5.0.0:

```rust
SUPPORTED_GRAPHIFY_VERSION = "0.9.25"
AUDITED_GRAPHIFY_REVISION =
  "2fa6cd3d5548577f8c5f591b713f0bf80c1af183"
```

Important limits:

```rust
GRAPHIFY_PROBE_TIMEOUT   = 3s
GRAPHIFY_REFRESH_TIMEOUT = 120s
GRAPHIFY_QUERY_TIMEOUT   = 10s
MAX_PROVIDER_STDOUT_BYTES = 2 MiB
MAX_PROVIDER_STDERR_BYTES = 256 KiB
```

Important path constants:

```rust
GRAPH_OUTPUT_DIRECTORY = "graphify-out"
GRAPH_FILE_NAME = "graph.json"
```

The original v5.0.0 adapter had a confirmed compatibility-contract defect
with actual Graphify 0.9.25. SPEC-01 repaired this contract and added a real
provider certification gate; retain the constants above as the supported
provider boundary, not as an open defect claim.

---

## 5.9 Harness / product evidence

### `harness.rs`
Product Harness story/intake and state.

### `harness_experiment.rs`
Experiment lifecycle.

### `harness_improvement.rs`
Audits/interventions/improvement proposals and outcomes.

### `domain_language.rs`
Product Domain Language state and context.

### `platform.rs`
Configured platform/profile behavior.

---

## 5.10 Safe persistence / migration / release

### `safe_io.rs`
Hardened read/write/append and project mutation lock.

Important public API:

```rust
read_bytes(...)
read_text(...)
read_text_required(...)
ensure_directory_chain(...)
project_lock_path(...)
acquire_project_lock(...)
acquire_project_lock_with_timeout(...)
replace_file(...)
replace_text(...)
append_bytes(...)
append_text(...)
```

Mutation lock:

```text
.baron/.baron-mutation.lock
```

Default lock timeout:

```text
5 seconds
```

Properties:

- OS-level advisory lock,
- crash releases OS lock,
- safe directory validation,
- staging temp files,
- atomic activation,
- sync behavior,
- symlink/reparse checks.

Important audit conclusion:

> `safe_io` itself is not the main problem. The problem is incomplete adoption/coverage.

### `migration.rs`
Legacy migration/rollback.

Reviewed strengths include:

- project lock,
- manifest identity validation,
- repo/Vault ownership validation,
- symlink/reparse rejection,
- safe copy/restore,
- backup manifests.

### `asset_lifecycle.rs`
Managed asset lifecycle.

### `release.rs`
Release metadata/trust behavior.

### `certification.rs`
Release/certification checks.

---

# 6. Intended Runtime Lifecycle

A healthy high-risk operation should conceptually look like:

```text
User request
   ↓
Prepare / operation identity
   ↓
Task State + intent + risk + route
   ↓
Operation-bound Plan
   ↓
Agent executes change
   ↓
Baron-owned command execution
   ↓
Trusted operation-bound receipt
   ↓
Proof
   ↓
Mandatory quality gates
   ↓
Trace
   ↓
Plan completion
   ↓
Continuity / durable state
```

The authoritative identity should remain coherent across:

```text
project
task
operation
adapter
session
request
gate
source fingerprint
receipt
proof
trace
completion
```

The v5.0.0 audit found that this identity chain is incomplete in production wiring.

---

# 7. Intended Trust Model

## 7.1 Durable truth

Baron documents:

```text
Vault Markdown = durable source of truth
SQLite / replay / Wiki / graph caches = rebuildable accelerators
```

Do not elevate cache state into authority.

## 7.2 Evidence

A configured capability is not proof that a command ran.

A generated sentence is not proof.

Trusted execution should originate from the Baron-owned runner.

## 7.3 Project isolation

Project identity should separate same-name repositories.

Current trusted recall should prefer/bind current project evidence.

## 7.4 Unknown facts

Unknown evidence should remain unknown instead of being guessed.

---

# 8. Historical Graphify Failure — Resolved by SPEC-01

This section records the pre-SPEC-01 v5.0.0 production failure for audit
provenance. It is not the current Graphify status. SPEC-01 is closed on the
pushed `codex/spec-01-graphify-provider-contract` branch; the provider now
targets the real Graphify 0.9.25 artifact/query contract, keeps bounded Survey
fallback, and has a protected real-provider release gate.

This is an observed real-world failure and one of the clearest v5.0.0 release regressions.

## 8.1 Installation/capability detection is healthy

Real environment evidence showed:

```text
provider: graphify-local
presence: present
compatible: true
resolved on PATH: C:\Users\Ty\.local\bin\graphify.exe
```

Therefore:

> Do not misdiagnose this as “Graphify not installed”.

## 8.2 Output-directory contract mismatch

Baron creates staging and uses:

```text
<staging>/graphify-out
```

as the `--out` target, then expects:

```text
<staging>/graphify-out/graph.json
```

But the audited Graphify 0.9.25 CLI behavior writes:

```text
<OUT_DIR>/graphify-out/
```

Therefore Baron effectively causes output like:

```text
<staging>/graphify-out/graphify-out/...
```

while reading one directory higher.

A diagnostic environment override:

```powershell
$env:GRAPHIFY_OUT = '.'
baron automation code-map refresh --json
```

allowed refresh to complete, confirming the output-path mismatch.

This override is diagnostic only, not a valid product repair.

## 8.3 Query contract mismatch

Baron’s provider invokes a query contract equivalent to:

```text
graphify query <question>
  --graph <graph_path>
  --json
  --budget <max_hits>
```

Actual Graphify 0.9.25 does not expose `--json` as Baron assumes.

Observed result:

```text
error: Graphify query returned malformed JSON; Survey fallback remains active
```

Therefore:

> Refresh-path workaround does not repair query semantics.

## 8.4 Budget semantic mismatch

Baron uses its own `max_hits` value (default seen as 8) as Graphify `--budget`.

But Graphify’s budget is a token/context budget concept rather than Baron’s result-count concept.

So even after fixing the JSON/output issues, this semantic mismatch must be corrected.

## 8.5 Fake-green test oracle

The fake Graphify fixture encodes Baron’s assumptions:

- it writes `graph.json` directly to the supplied `--out`,
- accepts/expects `--json`,
- emits an invented JSON array,
- accepts `--budget 8` in Baron’s interpretation.

This means the test double confirms Baron’s own expected behavior rather than verifying Graphify’s actual behavior.

## 8.6 Required repair invariant

A fixed provider must prove against the **real supported Graphify binary**:

```text
capability detect
refresh
query
source verification
bounded result
fallback behavior
```

No fake executable or wrapper may be used as final certification.

---

# 9. Full Defect Registry and Current Disposition

Severity model used in this context:

```text
P0 = release-blocking / primary lifecycle broken
P1 = high integrity / concurrency / security / durability
P2 = medium reliability / correctness / robustness
P3 = warning / architectural debt / future risk
```

The entries below preserve the original v5.0.0 findings and reproduction
reasoning. Their live disposition at 2026-09-29 is:

| Findings | Current disposition |
| --- | --- |
| B-01..B-04, W-04 | **CLOSED in SPEC-01**. Real Graphify 0.9.25 contract and release-gate coverage are implemented. |
| B-05, B-20, B-21, W-10 | **CLOSED in SPEC-02**. Canonical lifecycle identity, ingress identity, task identity, and operation-bound Plan wiring are implemented. |
| B-06, B-07, B-12, B-13, B-14, W-09, W-12 | **CLOSED in SPEC-03**. Trusted receipt authority, collision-resistant receipt persistence, safe path handling, and diagnostics-vs-authority boundaries are implemented. |
| B-08, B-17, B-32 | **CLOSED in SPEC-04**. Proof, trace, gate, completion, and managed-plan authority are exact-operation/fail-closed. |
| B-09, B-10, B-11, B-18, B-25, B-26, B-27, W-05, W-08 | **IMPLEMENTED; SPEC-05 READY FOR ADVERSARIAL REVIEW**. Local self-review has no known Critical/Important finding, but independent acceptance is still pending. |
| B-15, B-16, B-19, B-22, B-23, B-24 | **OPEN / deferred** to SPEC-06 and SPEC-07. SPEC-05 intentionally did not claim these. |
| B-28, B-29, B-30, B-31, W-01, W-02, W-03, W-06, W-07, W-11 | **OPEN or partially addressed / deferred** to SPEC-06 and SPEC-08. Treat these as future scope, not as SPEC-05 completion evidence. |

---

# 10. P0 — Historical Release-Blocking Findings and Current Disposition

## B-01 — Graphify output-path contract mismatch

**Status:** Closed in SPEC-01; this is the historical pre-fix finding.

**Component:**
`crates/baron-core/src/graphify.rs`

**Problem:**
Baron supplies an output path that already ends in `graphify-out`, while Graphify 0.9.25 itself writes an internal `graphify-out` directory.

**Impact:**
Native code-map refresh fails under the default contract.

**Required fix:**
Align Baron staging/output semantics with the real Graphify 0.9.25 contract.

---

## B-02 — Graphify query JSON contract mismatch

**Status:** Closed in SPEC-01; this is the historical pre-fix finding.

**Problem:**
Baron expects a JSON query mode/shape that the real Graphify 0.9.25 CLI does not provide as assumed.

**Impact:**
Refresh may be forced to work diagnostically, but native Graphify query still fails.

**Required fix:**
Define and test a real supported result contract. Do not fabricate Graphify JSON.

---

## B-03 — Graphify budget semantic mismatch

**Status:** Closed in SPEC-01; this is the historical pre-fix finding.

**Problem:**
Baron maps result count to Graphify budget.

**Impact:**
Severe under-budgeting / incorrect provider semantics.

**Required fix:**
Separate:
- Baron hit count
- provider token/context budget

---

## B-04 — Fake Graphify tests validate the wrong contract

**Status:** Closed in SPEC-01; this is the historical pre-fix finding.

**Problem:**
The fake provider implements Baron’s incorrect assumptions.

**Impact:**
CI green does not mean the supported native provider works.

**Required fix:**
Keep unit fake if useful, but add a real Graphify 0.9.25 integration gate.

---

## B-05 — CLI plan start uses legacy unbound lifecycle

**Status:** Closed in SPEC-02; this is the historical pre-fix finding.

**Core has:**

```rust
start_or_resume_plan_for_operation(...)
```

**CLI actually uses:**

```rust
PlanCommands::Start { ... } => {
    ...
    let plan = start_or_resume_plan(&repo_root, &vault, &title)?;
}
```

**Impact:**
Medium/high plan completion later requires operation identity that the CLI-created plan never captured.

**Required fix:**
Production start must originate from an operation context or a high-level orchestration surface that owns it.

---

## B-06 — Trusted receipt authority is process-local but CLI workflow is multi-process

**Status:** Closed in SPEC-03; the historical process-boundary defect has an
explicit trusted authority architecture now.

`execution_receipt.rs` uses:

```rust
CURRENT_RECEIPTS: OnceLock<Mutex<...>>
```

Persisted receipts are intentionally diagnostic after restart.

But exposed CLI usage is effectively:

```text
process A: baron proof execute ...
process B: baron proof record / record-gate ...
process C: baron plan complete ...
```

Once process A exits, its in-memory authority disappears.

**Impact:**
An apparently legitimate multi-command workflow cannot carry trusted authority across commands.

**Required architectural decision:**
Choose one:

1. authoritative cryptographically verifiable persisted handoff; or
2. a single high-level Baron process owns:
   execute → receipt → proof → gates → trace → completion.

Do not weaken authority merely to make existing commands pass.

---

## B-07 — CLI `proof execute` creates generic receipt

**Status:** Closed in SPEC-03; this is the historical pre-fix finding.

CLI path:

```rust
let receipt = execute_command(ExecutionRequest { ... })?;
```

It does **not** use:

```rust
execute_command_with_context(...)
```

**Impact:**
No exact task/operation/adapter/session/request/gate identity.

**Required fix:**
Trusted execution must be contextual from the start.

---

# 11. P1 — Historical Integrity / Concurrency / Security / Durability Findings

## B-08 — Trace is not operation-bound

**Status:** Closed in SPEC-04; this is the historical pre-fix finding.

`latest_trace_score(repo_root)` selects latest trace state without exact operation identity.

Plan completion consumes latest trace state.

**Impact:**
A trace from a different concurrent task/operation can affect completion of the active plan.

**Required fix:**
Trace must carry and validate exact operation identity and current-source/receipt evidence.

---

## B-09 — Authoritative lifecycle writers lack full project-lock coverage

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Strong lock primitive exists:

```rust
acquire_project_lock(...)
```

But lock usage is incomplete across important writers including portions of:

- Plan,
- Proof,
- Trace,
- Control Plane,
- Harness,
- config mutations.

**Impact:**
Atomic replacement prevents torn files but does not prevent:

```text
Process A reads old state
Process B reads old state
Process A writes state A
Process B writes state B
=> A update lost
```

**Required fix:**
All authoritative read→modify→write operations must be serialized or moved to transaction/event semantics.

---

## B-10 — Proof ID collision

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Timestamp IDs use millisecond precision.

**Impact:**
Concurrent processes can target the same proof ID/path.

**Fix direction:**
Collision-resistant immutable IDs:
- random UUID,
- cryptographic random nonce,
- or equivalent globally safe identity.

---

## B-11 — Trace ID collision

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance. Same historical class as B-10.

---

## B-12 — Receipt ID cross-process collision

**Status:** Closed in SPEC-03; this is the historical pre-fix finding.

Receipt ID input includes timestamp plus a **process-local** sequence.

Two processes starting first receipt in the same millisecond can share the same sequence value.

**Fix direction:**
Use process-independent collision-resistant IDs.

---

## B-13 — Receipt append bypasses Baron safe I/O

**Status:** Closed in SPEC-03; this is the historical pre-fix finding.

Receipt path:

```text
.baron/cache/execution-receipts.jsonl
```

Production append uses direct filesystem append rather than Baron’s hardened append abstraction.

Missing guarantees include:

- project mutation serialization,
- unified target validation,
- hardened directory traversal checks,
- Baron-standard durable write semantics.

---

## B-14 — Receipt path symlink/reparse protection is weaker than hardened writers

**Status:** Closed in SPEC-03; this is the historical pre-fix finding.

Because receipt storage bypasses `safe_io`, it does not inherit the same path safety.

**Security impact:**
A hostile/untrusted project filesystem layout may redirect writes outside the expected project path.

---

## B-15 — Execution timeout kills direct child, not full descendant tree

**Status:** Open; deferred to SPEC-07.

Runner captures child stdout/stderr via reader threads.

On timeout:

- direct child is killed,
- direct child is waited,
- reader threads are joined.

If a descendant inherited output handles, pipe EOF may not occur.

**Impact:**
The “bounded timeout” can extend beyond the configured timeout or hang.

**Fix direction:**

Windows:
- Job Object or equivalent process-tree ownership.

Unix:
- process group/session + group kill.

Add a grandchild timeout regression test.

---

## B-16 — Repo ↔ Vault mirrored writes are not consistently transactional

**Status:** Open; deferred to SPEC-06. SPEC-05 intentionally did not introduce
the repository/Vault transaction framework.

Common pattern:

```text
write repo
write Vault
update index
update second index
```

No universal transaction/recovery envelope exists for all authoritative state.

**Impact:**
Crash/disk error after one side succeeds can produce mirror divergence.

**Required fix:**
Recoverable multi-file transaction:
- intent record,
- staged writes,
- checksums,
- commit marker,
- reconciliation/rollback.

Do not claim cross-filesystem atomic rename.

---

## B-17 — Receipt-bound proof has a half-commit window

**Status:** Closed in SPEC-04; this is the historical pre-fix finding.

Observed conceptual order:

1. create ordinary proof;
2. write indexes/mirrors;
3. append trusted receipt reference;
4. append mirror reference.

If step 3/4 fails:

- API returns error,
- but ordinary proof already exists,
- it may already be indexed/latest.

**Fix direction:**
Validate/build full proof first, then publish transactionally.

---

## B-18 — Plan history path is not immutable identity

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Plan path concept:

```text
docs/baron/plans/<date>/<date>-<slug(title)>.md
```

**Collision cases:**

- same title restarted same day;
- punctuation variants with same slug;
- concurrent same title.

**Impact:**
Historical plan can be overwritten/aliased.

**Fix direction:**
Unique immutable plan ID in filename and metadata.

---

## B-19 — Plan title newline/state injection

**Status:** Open; deferred to SPEC-06.

Title validation is insufficiently strict.

`CURRENT.md` contains raw title and a `- Plan:` field.

If title contains injected lines, parser may encounter a fake `- Plan:` before the real one.

**Impact:**
Later plan update/interruption/completion can resolve the wrong in-repo relative file.

**Required fix:**

- enforce single-line bounded title;
- reject control characters/newlines/backticks where structurally meaningful;
- use structured state instead of parse-sensitive free-form Markdown fields for authority.

---

## B-20 — Prepare accepts missing session/request identity but trusted lifecycle later requires them

**Status:** Closed in SPEC-02; this is the historical pre-fix finding.

`PrepareRequestV1`:

```rust
session_id: Option<String>
request_id: Option<String>
```

Operation-aware trusted Plan/Receipt/Gate paths require complete identities.

**Impact:**
Baron can accept a request that cannot later satisfy authoritative completion.

**Required fix:**
At ingress:
- either require identity;
- or synthesize stable Baron-owned identity.

Do not defer failure until completion.

---

# 12. P2 — Medium Reliability / Correctness Findings and Deferred Scope

## B-21 — Task ID algorithm inconsistency

**Status:** Closed in SPEC-02; this is the historical pre-fix finding.

Prepare task identity and Tier-0 Task State use different identity inputs.

Observed distinction:

- one incorporates session/request,
- another is based more narrowly on project + task.

**Impact:**
Correlation/debug confusion now; future authority mismatch risk later.

**Fix direction:**
Define one canonical typed identity hierarchy.

---

## B-22 — Receipt secret redaction is incomplete

**Status:** Open; deferred to SPEC-07.

Known receipt stdout/stderr redaction is pattern-based.

Potential leakage classes:

- alternate `TOKEN:` / `PASSWORD:` styles,
- connection URLs with credentials,
- Bearer tokens,
- service-specific keys,
- secrets passed in argv.

Important:
`arguments: Vec<String>` are persisted independently from stdout/stderr redaction.

**Fix direction:**

- avoid storing sensitive argv where possible;
- configurable argument redaction;
- broader structured patterns;
- explicit tests for GitHub/OpenAI/AWS/DB URL/JWT/bearer cases.

---

## B-23 — Source fingerprint has weak resource bounds

**Status:** Open; deferred to SPEC-07.

Source fingerprint recursively hashes regular files outside hard-coded skip paths.

A large included file may be read fully into memory.

**Impact:**
Heavy I/O / memory pressure or accidental local DoS.

**Fix direction:**

- streaming hashing,
- max file size,
- max total bytes/files,
- ignore-aware traversal,
- bounded diagnostics when skipped.

---

## B-24 — I/O errors sometimes collapse to empty/default state

**Status:** Open; deferred to SPEC-06.

Patterns such as:

```rust
read_to_string(...).unwrap_or_default()
```

or equivalent fallback can treat:

- permission error,
- malformed/corrupt data,
- transient I/O failure

as though the state never existed.

**Impact:**
Existing durable state may be replaced with a fresh/partial projection.

**Rule:**

```text
NotFound => initialize if allowed
Any other I/O error => propagate/fail closed
Parse error => report corruption; do not silently reset
```

---

## B-25 — Config mutations are not fully serialized

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Example class:

```text
load config
modify one field
atomic replace
```

Two concurrent processes modifying different fields may overwrite each other.

---

## B-26 — Harness mutable state has similar lost-update exposure

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Examples include mutable:

- CURRENT,
- test matrix,
- decisions,
- friction/intervention state.

Severity is lower than proof/plan authority but still relevant in multi-agent workflows.

---

## B-27 — Diagnostic/capability runtime writes can lose concurrent updates

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Mostly diagnostic and lower authority than trusted receipts, but same read→modify→replace smell exists.

---

## B-28 — Session import state can lose concurrent updates

**Status:** Open; deferred to SPEC-08.

`session.rs` loads state, updates a map, and writes the whole state.

No global project transaction is held around the full import state lifecycle.

**Impact:**
Concurrent imports can overwrite each other’s source metadata.

---

## B-29 — Session import state silently resets on read/parse error

**Status:** Open; deferred to SPEC-08.

Implementation shape:

```rust
fs::read_to_string(path)
    .ok()
    .and_then(|content| serde_json::from_str(&content).ok())
    .unwrap_or_default()
```

**Impact:**
Corruption is treated as empty state.

**Fix direction:**
Return `Result<ImportState>` and distinguish:
- absent,
- corrupt,
- unreadable.

---

## B-30 — Session replay rebuild is not one SQLite transaction

**Status:** Open; deferred to SPEC-08.

Observed rebuild sequence:

```text
DELETE current project rows
DELETE FTS rows
insert message 1
insert message 2
...
```

Not wrapped as one atomic rebuild transaction.

**Impact:**
Crash/error during rebuild leaves a partial replay index.

This is rebuildable cache state, so severity is below durable truth.

**Fix direction:**
Single SQLite transaction + appropriate busy/retry behavior.

---

## B-31 — Code graph/query cache concurrent rebuild churn

**Status:** Open; deferred to SPEC-08.

Cache safety/path containment is relatively strong, but rebuildable cache writers do not universally serialize concurrent refreshes.

**Impact:**
Stale/duplicate/rebuild churn rather than direct durable truth corruption.

---

## B-32 — “Latest global artifact” design remains too common

**Status:** Closed in SPEC-04 for the reviewed completion authority paths; any
remaining diagnostic/latest convenience is future architectural scope.

Trace is the most serious case, but several diagnostic consumers still use latest-state ordering rather than exact operation identity.

**Future direction:**
Exact operation query first; latest should be diagnostics/UI convenience only.

---

# 13. P3 — Warnings / Architectural Debt

## W-01 — `AGENTS.md` is stale relative to the released Phase 16 state

**Status:** Open documentation debt; this context must not be used to infer
the current phase from stale `AGENTS.md` wording.

The v5.0.0 release snapshot still contains maintainer guidance that references earlier phase state.

**Risk:**
An AI agent working on Baron itself may follow stale project instructions.

---

## W-02 — Status/build-log/current-plan drift

**Status:** Partially addressed by maintained SPEC-01..SPEC-05 status/build-log
checkpoints; historical drift remains a reason to prefer the live snapshot in
Section 0.

Historical planning files and current release state are not always aligned.

**Rule for agents:**
Use source code + release boundary + explicit current spec as authority.
Do not treat every historical “CURRENT” file as current product truth.

---

## W-03 — Tests are stronger at primitive/unit boundaries than real workflow boundaries

**Status:** Partially addressed by SPEC-01..SPEC-05 real-provider, executable,
and deterministic multiprocess coverage; full cross-spec E2E and crash/recovery
certification remains deferred to SPEC-08.

Examples of missing/weak integration coverage:

- real Graphify binary,
- separate CLI processes across receipt lifecycle,
- concurrent Plan/Proof/Trace mutation,
- crash between repo and Vault writes,
- process-tree timeout.

---

## W-04 — No real Graphify release gate

**Status:** Closed in SPEC-01.

This is the testing gap that allowed B-01/B-02 to ship.

---

## W-05 — Multi-agent product claim is broader than lock coverage

**Status:** Implemented in SPEC-05; READY FOR ADVERSARIAL REVIEW, not formally
closed until independent acceptance.

Baron explicitly supports Codex and Claude sharing a project/Vault.

That makes process concurrency a first-class correctness requirement, not an edge case.

---

## W-06 — `safe_io` adoption is inconsistent

**Status:** Partially addressed in SPEC-05 for the owned authoritative writers;
remaining transaction/adoption work is deferred to SPEC-06.

Do not rewrite `safe_io` first.

Instead:
- inventory all authoritative writes;
- make them use safe primitives + correct transaction/lock scope.

---

## W-07 — Repo/Vault transaction semantics need a first-class abstraction

**Status:** Open; deferred to SPEC-06.

Today transaction/reconciliation semantics are not uniformly represented by one durable state machine.

---

## W-08 — Millisecond timestamp identity should be removed from durable record identity

**Status:** Implemented in SPEC-05 for new proof, trace, experiment, and plan
instances; READY FOR ADVERSARIAL REVIEW, not formally closed until independent
acceptance.

Use timestamps as metadata, not uniqueness.

---

## W-09 — Receipt authority lifetime needs an explicit architecture

**Status:** Closed in SPEC-03.

A receipt cannot simultaneously be:
- authoritative only in the creating process,
- yet expected to authorize later independent CLI commands.

Resolve the contradiction explicitly.

---

## W-10 — Lifecycle identity should be one typed object

**Status:** Closed in SPEC-02.

Avoid separately generating/passing:

```text
task_id
operation_id
adapter
session_id
request_id
```

through unrelated APIs with different optionality.

Use one validated typed identity propagated end-to-end.

---

## W-11 — Immutable event/state projection may fit Baron better than mutable “latest/current” files

**Status:** Open architectural direction; not required for SPEC-05 and deferred
to later stabilization/certification design.

Many races exist because multiple agents mutate shared Markdown state.

Potential future architecture:

```text
append immutable event
       ↓
validated operation ledger
       ↓
materialized CURRENT/index views
```

This is not mandatory for v5.0.1, but should influence design choices.

---

## W-12 — Cache vs authority distinction should be encoded in APIs/types

**Status:** Closed for the reviewed receipt/capability authority boundaries in
SPEC-03 and SPEC-05; broader cache/session typing remains future scope.

Rebuildable indexes should be structurally difficult to use as authorization evidence.

---

# 14. Areas That Are Strong / Already Correct Enough

Do not destabilize these areas without direct evidence.

## 14.1 Safe I/O primitive

`safe_io.rs` has strong design:

- directory-chain validation,
- symlink/reparse checks,
- atomic replacement,
- temporary staging,
- sync behavior,
- OS project lock,
- reentrant locking.

Main defect is **coverage**, not the primitive.

## 14.2 State guard

`require_coherent_execution_state(...)` is appropriately fail-closed around:

- config schema,
- identity binding,
- Vault binding,
- capsule schema,
- identity mismatch.

## 14.3 Memory firewall

Current trusted retrieval correctly filters:

- candidate,
- contested,
- superseded,
- expired,
- invalid temporal,
- unrelated project memory,
- unapproved global candidates.

No major v5 semantic trust bypass was found in the audited path.

## 14.4 Migration

Migration has:

- mutation locking,
- manifest ownership checks,
- project/Vault association validation,
- symlink/reparse rejection,
- backups,
- safe file copy behavior.

## 14.5 Release signing / self-update

This is one of the strongest v5.0.0 subsystems.

Reviewed properties include:

- HTTPS-only client path,
- GitHub release host constraints,
- redirect limits,
- manifest size bounds,
- candidate size bounds,
- connection/request timeouts,
- detached Ed25519 manifest verification,
- compiled public trust anchor,
- candidate hash/size/version verification,
- exact source/release checks,
- staged update transaction,
- rollback/recovery,
- release workflow verification.

Do not weaken release trust to simplify v5.0.1 development.

---

# 15. CI / Release Verification Reality

This section describes the original release-certification gap. SPEC-01 through
SPEC-05 add real-provider, executable, and deterministic multiprocess coverage
for their owned boundaries, but final cross-spec crash/recovery and release
certification remain deferred to SPEC-08.

The existing release flow is valuable, but it missed several integration classes.

Current CI/release behavior includes:

- formatting;
- Clippy with warnings denied;
- workspace/native tests;
- release build;
- version smoke;
- Windows/Linux matrix;
- release metadata verification;
- installer lifecycle.

But green CI did **not** prove:

```text
real Graphify 0.9.25 works
multi-process CLI receipt authority works
concurrent lifecycle writers are serialized
repo/Vault partial-write recovery works everywhere
descendant processes are killed on timeout
```

Therefore future certification must add those test classes.

---

# 16. AI Agent Rules for Repair Work

An agent receiving this context must follow these rules.

## 16.1 Do not patch symptoms

Bad examples:

```text
manually edit STACK_MAP.md
fake Graphify JSON
put a wrapper graphify.exe earlier on PATH
write fake trusted receipt Markdown
disable strict gate checks
make persisted receipts automatically trusted without a secure authority design
```

These are prohibited.

## 16.2 Do not weaken fail-closed behavior just to make tests green

If a trusted gate fails because production wiring is incomplete, fix wiring.

Do not change:

```text
strict gate => permissive gate
```

unless the spec explicitly redesigns trust.

## 16.3 Do not conflate cache with truth

Cache may be rebuilt.

Vault/project durable state may not be silently regenerated from cache.

## 16.4 Do not manually invent operation identity

Identity should come from Baron-owned lifecycle code.

## 16.5 Treat Codex + Claude as concurrent processes

Design tests and writers as if:

```text
Codex process A
Claude process B
```

can mutate the same project at nearly the same time.

## 16.6 Preserve user-owned project data

Baron repair work must not:

- rewrite application code to bypass Baron defects,
- mutate application DB data,
- alter unrelated migrations,
- modify product behavior to satisfy tooling,
- destroy custom agent/skill/hook content.

## 16.7 Pin tests to the exact supported provider version

For Graphify, “works with my wrapper” is not certification.

---

# 17. Baron 5.0.1 Stabilization Program — Live Progress

The audit should **not** become 32 separate specs.

The defects group naturally into eight architectural specs.

---

## SPEC-01 — Graphify Provider Contract

**Status at 2026-09-29:** CLOSED. Branch head:
`codex/spec-01-graphify-provider-contract` @ `adccb6e`.

### Scope

Fix the complete Baron ↔ Graphify 0.9.25 native contract.

### Covers

```text
B-01
B-02
B-03
B-04
W-04
```

### Required outcomes

- real output contract;
- real query contract;
- proper provider budget semantics;
- no fake JSON;
- real binary integration test;
- bounded fallback remains safe.

### Minimum certification

```text
baron capability check --adapter codex
baron automation code-map refresh
baron automation code-map query "backend HTTP entrypoint"
```

against actual supported Graphify.

---

## SPEC-02 — Operation Identity & Lifecycle Wiring

**Status at 2026-09-29:** CLOSED. Branch head:
`codex/spec-02-operation-identity` @ `3a52668`.

### Scope

Make operation identity coherent from prepare/hook into plan/execution.

### Covers

```text
B-05
B-20
B-21
W-10
```

### Required outcomes

- canonical identity type;
- ingress validation/synthesis;
- operation-aware plan start;
- no medium/high plan can start in an identity state that cannot complete;
- canonical task ID semantics.

---

## SPEC-03 — Trusted Execution Receipt Architecture

**Status at 2026-09-29:** CLOSED. Branch head:
`codex/spec-03-trusted-receipt-architecture` @ `fa1bd91`.

### Scope

Resolve process-local authority vs multi-process workflow.

### Covers

```text
B-06
B-07
B-12
B-13
B-14
W-09
W-12
```

### Required outcomes

- clear receipt authority lifetime;
- operation-bound execution;
- collision-resistant IDs;
- hardened receipt persistence;
- explicit diagnostics-vs-authority semantics.

---

## SPEC-04 — Proof / Trace / Gate / Completion Integrity

**Status at 2026-09-29:** CLOSED. Branch head:
`codex/spec-04-proof-trace-gate-completion-integrity` @ `ecaf362`.

### Scope

Bind all completion evidence to the exact operation.

### Covers

```text
B-08
B-17
B-32
```

### Required outcomes

- operation-bound trace;
- transactional trusted proof publication;
- strict exact-operation lookup;
- completion rejects evidence from other tasks/operations.

---

## SPEC-05 — Multi-Agent Concurrency & Durable State

**Status at 2026-09-29:** IMPLEMENTATION COMPLETE; **READY FOR ADVERSARIAL
REVIEW**, not closed. Branch head:
`codex/spec-05-multi-agent-concurrency-durable-state` @ `eb7624b`.

### Scope

Make simultaneous Codex/Claude mutation safe.

### Covers

```text
B-09
B-10
B-11
B-18
B-25
B-26
B-27
W-05
W-08
```

### Core invariant

> Every authoritative read→modify→write mutation is serialized, and every durable record has a collision-resistant immutable identity.

---

## SPEC-06 — Repo/Vault Transaction & Safe Persistence

**Status at 2026-09-29:** NOT STARTED. This is the next implementation scope
only after SPEC-05 receives independent acceptance.

### Scope

Unify mirrored durable writes and strict error semantics.

### Covers

```text
B-16
B-19
B-24
W-06
W-07
```

### Required outcomes

- recoverable mirrored transaction;
- strict parse/I/O failure handling;
- plan field validation;
- no silent empty-state reset;
- reconciliation after crash.

---

## SPEC-07 — Execution Runner & Resource Hardening

**Status at 2026-09-29:** NOT STARTED. Deferred until SPEC-06.

### Scope

Bound execution/runtime resource behavior.

### Covers

```text
B-15
B-22
B-23
```

### Required outcomes

- kill process tree;
- stronger secret handling;
- sensitive argv policy;
- streaming/bounded fingerprint;
- resource-limit tests.

---

## SPEC-08 — Cache, Session, E2E & Release Certification

**Status at 2026-09-29:** NOT STARTED. Deferred until SPEC-07.

### Scope

Close rebuildable-state races and certify the repaired system end-to-end.

### Covers

```text
B-28
B-29
B-30
B-31
W-01
W-02
W-03
W-11
```

### Required outcomes

- transactional session replay rebuild;
- strict session import state parsing;
- cache concurrency behavior;
- cross-process CLI lifecycle tests;
- concurrency tests;
- crash/restart tests;
- docs/AGENTS truth cleanup;
- final adversarial release certification.

---

# 18. Recommended Spec Execution Order

```text
SPEC-01 CLOSED
   ↓
SPEC-02 CLOSED
   ↓
SPEC-03 CLOSED
   ↓
SPEC-04 CLOSED
   ↓
SPEC-05 IMPLEMENTED / READY FOR ADVERSARIAL REVIEW
   ↓
SPEC-06 NOT STARTED
   ↓
SPEC-07 NOT STARTED
   ↓
SPEC-08 NOT STARTED
   ↓
Final adversarial verification
   ↓
Version bump to 5.0.1
```

Do **not** bump to 5.0.1 halfway through the program.

A recommended policy is:

```text
keep development version semantics at 5.0.0
complete all stabilization specs
run final certification
then bump/release 5.0.1
```

---

# 19. Mandatory Regression Test Matrix

A repair is not complete until tests cover the production boundary that originally failed.

## 19.1 Real Graphify

Must test actual supported binary:

```text
detect
version
refresh
query
malformed/error fallback
source-hit verification
bounded output
```

---

## 19.2 Cross-process trusted lifecycle

Must launch actual separate `baron` processes.

Test:

```text
prepare operation
start plan
execute trusted command
record proof
record three gates
record trace
complete plan
```

If design changes to single-process orchestration, test that high-level command and prove low-level commands cannot accidentally bypass authority.

---

## 19.3 Concurrent writers

Run barriers to force interleaving between two processes for:

```text
Plan
Proof
Trace
Control Plane
Harness
config mutation
session import state
```

Assert:

- no lost updates;
- no ID collisions;
- no mirror divergence;
- deterministic recovery.

---

## 19.4 Repo/Vault fault injection

Inject failures after:

```text
repo write before Vault
Vault write before index
first index before second index
trusted evidence validation before publish
```

Restart and reconcile.

---

## 19.5 Plan injection

Test titles containing:

```text
newline
backtick
Markdown field syntax
fake "- Plan:"
control characters
very long input
```

Expected result:
- invalid title rejected before durable write.

---

## 19.6 Receipt secret tests

Include:

```text
token=
token:
Authorization: Bearer
JWT
OpenAI-style key
GitHub-style token
AWS-style key
postgres://user:password@host/db
--password secret
--token secret
```

Assert sensitive value is not persisted.

---

## 19.7 Process-tree timeout

Spawn:

```text
baron runner
  └── child
       └── grandchild holding stdout open
```

Assert entire owned tree is terminated and command returns within bounded grace.

---

## 19.8 Session/index crash tests

Interrupt during:

```text
session import state publish
session replay rebuild after DELETE
code graph cache publish
```

Assert:
- durable truth remains intact;
- cache is either valid or rebuildable;
- corruption is reported, not silently accepted.

---

# 20. Source-of-Truth Priority for AI Agents

When sources disagree, use this priority:

```text
1. Explicit stabilization spec currently being implemented
2. Actual v5.0.0 production code at pinned release commit
3. Security/trust invariant proven by code/tests
4. Architecture docs that match production
5. Release README / CHANGELOG
6. Historical plans/build logs
7. Stale AGENTS/current notes
```

Never use an old checkbox such as:

```text
[x] completed
```

as proof the current production path satisfies that invariant.

---

# 21. Historical Documentation Claim vs Current Production Reality

## Historical claim: operation identity is explicit across supported adapters

Reality:
- SPEC-02 closed the ingress, canonical task, operation-bound Plan, and hook
  wiring gaps on the stabilization branch.
- Treat the SPEC-02 branch/status evidence as current for this finding; do not
  re-open the historical v5.0.0 diagnosis without new reproduction evidence.

## Historical claim: proof/trace/gates are tied to fresh trusted execution

Reality:
- SPEC-03 and SPEC-04 closed the trusted receipt, proof, trace, gate, and
  completion authority gaps for the reviewed lifecycle.
- SPEC-05 further serializes concurrent publication, but remains READY FOR
  ADVERSARIAL REVIEW pending independent acceptance.

## Historical claim: simultaneous agents use safe local coordination

Reality:
- SPEC-05 now covers the owned authoritative mutation inventory with the
  project-scoped lock, collision-resistant IDs, no-overwrite publication, and
  deterministic multiprocess tests.
- This claim is implementation-complete but not formally closed until the
  independent adversarial review returns a verdict.

## Current boundary: project/Vault transaction rules cover durable writes

Reality:
- Safe primitives and migration lock boundaries exist.
- A unified recoverable repository/Vault transaction framework is explicitly
  deferred to SPEC-06 and must not be inferred from SPEC-05.

## Historical claim: Graphify 0.9.25 supported provider

Reality:
- SPEC-01 repaired and tested the real Graphify 0.9.25 output/query contract,
  bounded fallback, and release gate. The old mismatch is retained above only
  as audit provenance.

---

# 22. What Not to “Fix” Without Evidence

Do not rewrite these areas just because they are complex:

```text
release signature verification
Ed25519 trust anchor
self-update download verification
state_guard identity validation
memory trusted-record firewall
safe_io atomic replacement primitive
migration ownership/symlink validation
```

These were not identified as primary v5.0.0 blockers.

Changes here create unnecessary risk.

---

# 23. Expected Engineering Style for v5.0.1

Every implementation task should include:

```text
Invariant
Current production path
Failure mechanism
Minimal design change
Files/functions changed
RED test
Implementation
GREEN test
Cross-process/adversarial test if applicable
Backward compatibility
Recovery semantics
Docs update
```

Avoid tasks that merely say:

```text
fix Graphify
fix receipt
add lock
```

Those are too vague.

---

# 24. Example Required Spec Language

A good invariant:

> A medium/high-risk operation may complete only when Plan, trusted execution receipt, Proof, all mandatory gates, Trace, and current source fingerprint belong to the exact same validated OperationIdentity.

A weak requirement:

> Make operation_id work.

---

# 25. Recommended Typed Identity Direction

This is a design recommendation for SPEC-02/03, not a mandate to copy exact naming.

Conceptually:

```rust
struct OperationIdentity {
    project_id: ProjectId,
    task_id: TaskId,
    operation_id: OperationId,
    adapter: SupportedAdapter,
    session_id: SessionId,
    request_id: RequestId,
}
```

Properties:

- fully validated on construction;
- no newline/control characters;
- bounded length;
- no optional authoritative fields after preparation;
- serializable;
- propagated intact into:
  - Plan,
  - Receipt,
  - Proof,
  - Gate Evidence,
  - Trace,
  - Completion.

Do not repeatedly reconstruct it from loosely related strings.

---

# 26. Recommended Durable Record Identity Direction

For:

```text
plan
proof
trace
receipt
possibly harness story/recovery events
```

Use:

```text
immutable random/collision-resistant ID
```

Store timestamp separately.

Do not use:

```text
timestamp-to-millisecond as identity
date + slug(title) as identity
process-local counter as uniqueness
```

---

# 27. Recommended Transaction Direction

For authoritative mirror writes:

```text
begin mutation lock
   ↓
read + validate current state
   ↓
build all new content in memory
   ↓
write durable transaction intent
   ↓
stage repo side
   ↓
stage Vault side
   ↓
fsync/validate hashes
   ↓
activate each target
   ↓
commit marker
   ↓
materialize indexes/views
   ↓
release lock
```

If repo and Vault are on different filesystems:

> Do not call this “atomic”.

Call it:

```text
recoverable transaction
```

with deterministic replay/rollback/reconciliation.

---

# 28. Recommended Receipt Architecture Decision

SPEC-03 must choose explicitly.

## Option A — Persisted authoritative receipt

Requires:

- tamper-resistant authenticity;
- exact operation binding;
- project/source binding;
- replay prevention;
- freshness;
- trusted key/secret ownership model;
- cross-process verification.

A plain unkeyed digest is not enough.

## Option B — Single owner process

A high-level command/process owns:

```text
execute
receipt
proof
gates
trace
completion
```

and process-local authority becomes coherent.

Persisted receipts remain diagnostics after process exit.

Do not keep the current ambiguous middle state.

---

# 29. Graphify Repair Constraints

Do not solve Graphify with:

```text
fake graphify.exe
PATH wrapper
temporary JSON converter
manual graph.json copy
manual STACK_MAP.md edit
special Hotel Staff product workaround
```

Correct sequence:

1. confirm exact supported Graphify 0.9.25 CLI contract;
2. adapt Baron provider;
3. add real integration fixture/gate;
4. prove Survey fallback still works when provider fails;
5. verify source evidence before graph-derived context is trusted.

---

# 30. Session/Cache Repair Philosophy

Remember:

```text
Session imported Markdown = durable-ish project evidence
session replay SQLite = rebuildable
code graph state/query cache = rebuildable
memory SQLite = rebuildable
Vault Markdown = durable truth
```

Therefore:

- cache corruption should not destroy durable state;
- cache errors should cause rebuild/degradation;
- durable state corruption must fail visibly;
- do not silently replace corrupted durable metadata with defaults.

---

# 31. Final Definition of “Fixed”

Baron v5.0.1 should not be called fixed merely because:

```text
cargo test passes
Graphify refresh works once
one unit receipt test passes
one Plan completion test passes in-process
```

The stabilization is complete only when:

1. real Graphify supported binary passes;
2. normal production lifecycle has one coherent operation identity;
3. trusted receipt authority matches real process boundaries;
4. Plan/Proof/Gates/Trace/Completion all bind exact same operation;
5. concurrent Codex/Claude writers do not lose authoritative state;
6. repo/Vault partial failure is recoverable;
7. timeout kills owned descendant processes;
8. secret/resource bounds are hardened;
9. session/cache corruption is recoverable and visible;
10. final cross-platform release certification passes.

---

# 32. Current Engineering Verdict — 2026-09-29

Baron remains a public/stable `5.0.0` engine while the `5.0.1` stabilization
program is being delivered on separate branches. SPEC-01 through SPEC-04 are
closed. SPEC-05 is implementation-complete and has passed the local Core,
formatter, Clippy, release-build, compatibility, and deterministic
concurrency evidence described in Section 0.

The current honest verdict is:

> **SPEC-05 has no known locally found Critical/Important issue, but it is not
> formally closed because independent adversarial acceptance is still pending.**

The two full-workspace blockers are known host/baseline failures outside
SPEC-05: the PowerShell archive module failure in `lifecycle_scripts` and the
existing UTF-8 boundary panic in `prepare_cli`/`session_replay`.

The remaining stabilization work is not “finish SPEC-05 again”. It is:

1. obtain the independent SPEC-05 adversarial verdict;
2. implement SPEC-06 repository/Vault transaction and strict persistence
   semantics;
3. implement SPEC-07 runner/resource hardening;
4. implement SPEC-08 cache/session/E2E/release certification;
5. perform final adversarial certification, then bump and release `5.0.1`.

Do not call the whole Baron engine “5.0.1 fixed” until those later scopes and
the final certification are complete.

---

# 33. Immediate Instruction to the Next AI Agent

Before editing code:

1. read Section 0 of this context and the current `docs/BARON_STATUS.md`,
   `docs/BARON_STATUS.json`, build log, and active plan;
2. inspect the actual branch/worktree and do not assume `D:\Works\Baron-Engine`
   is the latest stabilization checkout;
3. do not restart SPEC-01..SPEC-05 implementation from the historical defect
   list;
4. if reviewer quota is available, obtain the independent SPEC-05 verdict on
   `ecaf362..eb7624b`; otherwise preserve the READY status and report the
   evidence boundary honestly;
5. after SPEC-05 acceptance, author/read SPEC-06 before touching its code;
6. write the invariant and reproduction before implementing, use RED→GREEN
   tests at the production boundary, and run the required verification;
7. avoid weakening trust/security or converting known failures into expected
   red;
8. preserve unrelated user/application files and untracked `fix-bug` assets;
9. do not bump the public version, tag, release, or rewrite history early;
10. finish each spec with independent adversarial verification and a precise
    status/build-log/context update.

If a new stabilization spec is not authored yet:

> Do not begin implementation from the bug list alone. First write the
> relevant spec so the architectural invariant, process boundary, backward
> compatibility, ownership boundary, and required tests are explicit.

---

# 34. Compact Defect Index (Identifiers; Live Status in Section 9)

The identifier list below is preserved for cross-reference with the original
audit. It is not a current open-defect list. Use Section 9 and Section 0 for
the 2026-09-29 disposition; in particular, SPEC-01..SPEC-04 are closed and
SPEC-05 is implementation-complete but still READY FOR ADVERSARIAL REVIEW.

```text
P0
B-01 Graphify output path mismatch
B-02 Graphify query JSON mismatch
B-03 Graphify budget semantics mismatch
B-04 Fake Graphify oracle
B-05 CLI Plan start unbound
B-06 Process-local receipt authority vs multi-process CLI
B-07 CLI proof execute uses generic receipt

P1
B-08 Trace not operation-bound
B-09 Missing lock coverage
B-10 Proof ID collision
B-11 Trace ID collision
B-12 Receipt ID collision
B-13 Receipt bypasses safe_io
B-14 Receipt path safety weaker
B-15 Timeout does not own process tree
B-16 Repo/Vault non-transactional mirrors
B-17 Receipt-bound proof half-commit
B-18 Plan path/history collision
B-19 Plan title state injection
B-20 Optional ingress identity vs strict lifecycle

P2
B-21 Task ID inconsistency
B-22 Secret redaction gaps
B-23 Source fingerprint resource bounds
B-24 Silent I/O/default state fallback
B-25 Config mutation lost update
B-26 Harness concurrency
B-27 Diagnostic runtime state concurrency
B-28 Session import lost update
B-29 Session import silent reset
B-30 Session replay partial rebuild
B-31 Code graph cache concurrency
B-32 Latest-global artifact smell

P3 / Warnings
W-01 Stale AGENTS.md
W-02 Status/build-log drift
W-03 Missing real E2E process-boundary tests
W-04 Missing real Graphify release gate
W-05 Multi-agent claim > lock coverage
W-06 Incomplete safe_io adoption
W-07 Missing unified repo/Vault transaction abstraction
W-08 Timestamp IDs
W-09 Ambiguous receipt authority lifetime
W-10 Fragmented lifecycle identity
W-11 Mutable current/latest architecture pressure
W-12 Cache/authority boundary should be typed
```

---

# 35. Stabilization Spec Index (Current Status in Section 0)

```text
SPEC-01 Graphify Provider Contract
SPEC-02 Operation Identity & Lifecycle Wiring
SPEC-03 Trusted Execution Receipt Architecture
SPEC-04 Proof / Trace / Gate / Completion Integrity
SPEC-05 Multi-Agent Concurrency & Durable State
SPEC-06 Repo/Vault Transaction & Safe Persistence
SPEC-07 Execution Runner & Resource Hardening
SPEC-08 Cache, Session, E2E & Release Certification
```

---

# 36. Closing Constraint

The objective of v5.0.1 is not to maximize code changes.

The objective is:

> **Make Baron’s documented trust and lifecycle invariants true in the real production path, under real provider binaries, real separate processes, concurrent agents, crashes, and recovery.**

That is the standard every subsequent spec and implementation task should be measured against.
