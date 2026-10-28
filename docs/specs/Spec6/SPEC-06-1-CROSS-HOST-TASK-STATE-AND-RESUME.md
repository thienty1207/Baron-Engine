# SPEC-06.1 — Cross-Host Task State and Resume

- **Status:** Optional future design; deferred. Implementation is not approved or scheduled. Begin only after the owner explicitly requests SPEC-06 implementation.
- **Parent scope:** SPEC-06, cross-model continuation between Codex and DeepSeek Harness.
- **Depends on:** Existing Baron Core Task State, operation identity, continuity, and recovery contracts.

## Goal

Allow one unfinished project task to move between Codex and DeepSeek Harness repeatedly without creating a new Baron project identity, losing the latest work, or relying on either host's private conversation history.

## User outcome

The user initializes a project with Codex, works until the usage limit, opens the same checkout in DeepSeek Harness, and resumes the same Baron task. Later, the user can return to Codex and resume again. The same flow works for more than one switch in either direction.

## Scope

- Define a durable, host-neutral task checkpoint and handoff contract.
- Extend lifecycle identity safely for the `dsh` host while preserving existing Codex and Claude records.
- Reuse the existing Project ID, identity binding, Vault route, task state, plan, proof, trace, and recovery evidence.
- Reconcile the latest durable checkpoint with the actual checkout after a clean handoff or abrupt process termination.
- Prevent simultaneous write ownership of the same managed project path.

## Non-goals

- Persist or transfer model chain-of-thought, private reasoning, or an entire host transcript as the canonical task state.
- Replace or alter Codex and Claude session stores.
- Create a new Project ID when another host opens an initialized project.
- Guarantee that an external model remembers context which Baron did not durably record.
- Implement provider-specific model APIs or store provider credentials.

## Identity and authority contract

1. `project_id` and the validated project identity binding remain stable across hosts and model providers.
2. A task has one stable task identity across handoffs. Each host session and each correctness-sensitive operation has its own session/operation identity and records its host (`codex`, `claude`, or `dsh`).
3. Provider and model names are diagnostic session metadata only. They do not select a project, authorize memory access, or define task identity.
4. Existing operation receipts remain bound to the operation and host that produced them. A new host may consume prior verified evidence, but cannot mint or relabel it.
5. If the same task cannot be identified unambiguously, Core must request an explicit task selection or return a blocker. It must not select by folder basename or latest timestamp alone.

## Checkpoint contract

The latest checkpoint must preserve the fields already required by Baron Core: user intent, constraints and non-goals, plan progress, last successful step, affected files, proof/trace status, blockers, and next safe action. It may reference bounded execution receipts and event IDs. It must not copy API keys or environment values.

Checkpoints are written at meaningful task transitions and after completed host tool operations that change project state. An explicit handoff writes a final checkpoint when available, but recovery must not depend on a successful end-of-session hook: quota exhaustion, app closure, or process crash can occur first.

The checkout remains authoritative for source contents. On resume, Core compares the checkpoint's recorded state with the current repository/worktree evidence, identifies changes made since the checkpoint, and marks unresolved progress as unknown. It must not silently repeat a non-idempotent command or claim an unrecorded verification passed.

Checkpoint/event writes must be idempotent under a stable event identity. A retry must not duplicate progress, search evidence, or execution receipts. Corrupt, stale, or ambiguous records fail closed with an actionable recovery packet and leave the original evidence intact.

## Concurrency

Only one host may own write-capable work on a project path at a time. The handoff protocol releases or expires the current host lease before another host receives write authority. A second live writer must be blocked or remain read-only. Stale lease recovery must be bounded and auditable; it must not delete task or Vault data.

## Acceptance criteria

- A Codex-initialized project keeps the same Project ID, identity binding, Vault route, and task identity through Codex → DSH → Codex and at least one additional repeated switch.
- A fresh DSH session receives the latest task checkpoint and current source/worktree changes without importing Codex private transcript data.
- An abrupt Codex or DSH termination resumes from the last durable checkpoint and reconciles newer source changes without inventing the missing interval.
- Identical hook/event retries do not duplicate checkpoints or receipts.
- Missing, malformed, cross-project, or ambiguous task state returns an explicit blocker and does not authorize a write.
- A concurrently active second writer cannot mutate the same managed project path.
- Existing Codex/Claude identity fixtures and persisted records remain readable and retain their previous authority boundaries.

## Required verification cases

1. Codex writes multiple files and checkpoints; DSH resumes, adds a change, and checkpoints; Codex resumes the same task.
2. Repeat the switch cycle at least three times and verify the latest checkpoint always wins without losing earlier evidence.
3. Terminate a host after a source write but before its next checkpoint; resume and reconcile the real diff without claiming an unknown step completed.
4. Retry a completed hook event and verify idempotent state and evidence.
5. Attempt resume with the wrong project identity, ambiguous active tasks, corrupt checkpoint data, and an active writer lease; each case fails closed.
