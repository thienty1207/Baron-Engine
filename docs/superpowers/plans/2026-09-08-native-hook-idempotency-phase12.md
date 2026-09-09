# Phase 12: Native Hook Acceleration and Lifecycle Idempotency

## Scope

This phase adds native Codex and Claude lifecycle hooks as bounded accelerators
over Baron Core. Hooks will normalize SessionStart, UserPromptSubmit,
PreCompact, and Stop into one Core lifecycle path while AGENTS.md and CLAUDE.md
remain sufficient when hooks are absent, disabled, untrusted, or failing.

Phase 13 Autopilot UX, Phase 14 release/update hardening, Phase 15 broad
documentation rewrite, and Phase 16 final release verification are deferred.
No adapter retirement, schema bump, version bump, release, tag, push, or
history rewrite is allowed in this phase.

## Repository evidence

- `crates/baron-core/src/automation.rs` currently records hook events by
  rewriting the whole JSONL journal and has no persistent event-key dedup.
- Existing Phase 10 operation identity already carries supported adapter,
  session, and request provenance.
- Existing `prepare` already accepts structured PrepareRequestV1 and produces
  the bounded PreparePacketV1; the native hook path should reuse that authority.
- Existing continuity checkpoints write shared repo and Vault packets, but do
  not carry a lifecycle event key or perform byte-stable duplicate checks.
- Codex and Claude installers already preserve third-party hook/settings JSON
  and install optional Baron-owned hook entries.
- Phase 2 safe I/O already provides the project-scoped reentrant lock and
  preserve-first replacement primitive; the journal needs a durable append
  primitive under that lock.

## Execution order

- [x] Record the Phase 12 start checkpoint and inventory.
- [x] Add deterministic target tests for normalized events, fallback
      convergence, idempotency, parent/child isolation, recursion, recovery,
      concurrency, preservation, failure classification, and journal bounds.
- [x] Add safe durable append support and bounded persisted dedup state without
      changing project, managed-state, memory, or PreparePacket schemas.
- [x] Add normalized lifecycle event identity and canonicalize legacy Prompt and
      Checkpoint deliveries to UserPromptSubmit and PreCompact.
- [x] Reuse structured prepare for UserPromptSubmit and return a bounded host
      projection instead of a full packet dump.
- [x] Make SessionStart bounded and idempotent, PreCompact checkpoint-only and
      cheap, and Stop checkpoint/reconciliation-only with completion kept
      separate from stop delivery.
- [x] Add deterministic parent/child and recursion guards with bounded child
      evidence, preserving parent operation ownership.
- [x] Extend Codex and Claude hook projections with the normalized lifecycle
      commands while preserving all third-party entries and malformed-input
      fail-safe behavior.
- [x] Run focused Phase 12 tests, all relevant prior-phase regressions, the
      required workspace checks, and both retired-adapter zero gates.
- [x] Update status, build log, and this plan with final evidence; stop before
      Phase 13.

## Proposed test seams and contracts

- `LifecycleEventKey` is scoped by project ID, supported adapter, session ID,
  request ID or deterministic correlation, normalized event kind, task ID, and
  optional child execution ID.
- Hook retries reuse the same prepared response where available, avoid
  duplicate journal/checkpoint/plan effects, and retain only a bounded recent
  dedup window. Durable checkpoints remain evidence and are never pruned as
  dedup records.
- Child events are recorded as bounded evidence and cannot create top-level
  intent, plan, prepare, completion, memory promotion, adapter switching, or
  competing continuity state.
- Hook failures are classified as soft diagnostic failures or hard lifecycle
  blockers; optional hook failure must leave instruction fallback usable.
- Existing serialized schemas remain compatible. Any new journal/checkpoint
  fields are optional and additive.

## Completion evidence

Implemented the four normalized native lifecycle deliveries over Baron Core:
SessionStart, UserPromptSubmit, PreCompact, and Stop. Prompt and Checkpoint
remain source-compatible aliases and journal an additive canonical `event_kind`.
Each hook key is scoped by project ID, adapter, session ID, request ID, event
kind, deterministic task ID, optional child ID, and Stop retry mode. Duplicate
deliveries return the persisted response without another journal or checkpoint.

UserPromptSubmit consumes structured JSON and reuses PreparePacketV1, returning
only a bounded projection. SessionStart is capped at 6,000 UTF-8 bytes,
PreCompact writes a cheap event-keyed checkpoint, and Stop reconciles without
claiming completion. Child events write bounded evidence only; recursion depth
and an in-process active-key guard prevent re-entry into Core lifecycle state.

The journal now uses durable append under the Phase 2 project lock. A separate
schema-versioned `.baron/cache/automation-dedup.json` retains the last 256
responses for restart-safe retry. Continuity packets carry an optional event
key and are byte-stable on duplicate delivery. Codex and Claude native hook
configurations add UserPromptSubmit and PreCompact accelerators while retaining
third-party JSON and fail-safe behavior. CLI hook errors classify malformed or
optional failures as soft and unsafe state/lock failures as hard blockers.

Focused Phase 12 suites pass: Core 9/9, adapters 3/3, CLI 4/4. Relevant Core,
adapter, and CLI regression suites pass. The exact workspace command initially
reports two host-only installer failures before assertions because the default
Codex PowerShell runtime loads an incompatible `Microsoft.PowerShell.Archive`
module; with the system Windows PowerShell module path, the full workspace
sweep passes 568 tests with 3 intentionally ignored historical fixtures and
0 failures, including lifecycle 5/5. `cargo fmt --all -- --check`, workspace
Clippy, diff checks, status JSON, and the retired-adapter grep gate pass. No
project, managed-state, memory, or PreparePacket schema was bumped. Phase 13
has not started.
