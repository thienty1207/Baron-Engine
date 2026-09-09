# Baron Memory Model

Baron Core uses a durable Vault plus disposable indexes:

```text
Vault Markdown                 source of truth
SQLite / replay / Wiki / graph caches   bounded accelerators
```

The memory boundary is the stable project ID. A folder name is only a display
hint, so two repositories with the same basename remain isolated.

## Trust and scope

Core retrieval uses `TrustedRecallPolicy::Current` for every context-critical
path before lexical or semantic ranking. Records carry both project scope and a
trust state:

- verified current-project evidence may be selected;
- approved global evidence may be selected when relevant;
- likely or stale evidence remains labelled and needs corroboration;
- candidate, contested, superseded, and expired evidence cannot become current
  truth;
- cross-project evidence stays blocked unless the existing firewall policy has
  an explicit match;
- unknown facts remain unknown rather than being guessed.

Host-local auto memory is context only. It does not replace Core memory or
override intent, decisions, Task State, continuity, recovery, proof, or trace.

## Vault shape

```text
Vault/
  AGENTS.md
  Init.md
  Projects/
    <project-slug>--<project-id-prefix>/
      .baron-project.json
      README.md
      Facts.md
      Decisions.md
      Tasks.md
      Plans/
      ProductHarness/TEST_MATRIX.md
      Proofs/
      Traces/
      Sessions/Imported/
      Research/
      Notes/
      Open Questions.md
      Handoff.md
  Artifacts/Baron/
    memory-index.sqlite
    session-replay.sqlite
    memory-engine-state.json
    APPROVED_GLOBAL.md
    GLOBAL_CANDIDATES.md
```

The Markdown capsule is readable, backed up, and portable. Index files may be
deleted and rebuilt without changing durable truth.

## Task State and context

Task State is the Core projection used by `PreparePacketV1`. It retains project
and task identity, intent, constraints, non-goals, current plan/work state,
last successful step, failed or interrupted status, affected files, proof/trace
state, blockers, recovery, route, mandatory gates, and the next safe action.

The Context Compiler places these fields in Tier 0, which is never silently
dropped under budget pressure. Trusted decisions, current source evidence, and
proof/trace occupy Tier 1. Profile and bounded Wiki/CodeGraph/session replay
occupy Tier 2. Low-priority diagnostics and Autopilot candidate summaries are
Tier 3 and are compressed or omitted first.

## Session, Wiki, and CodeGraph accelerators

Session import is bounded, redacted, deduplicated, and matched to the current
project. Replay returns a small surrounding window rather than a full history.
Wiki and CodeGraph caches retain source paths, spans, citations, hashes, and
project identity; they are navigation hints and never proof by themselves.

If an accelerator is absent, stale, malformed, or untrusted, Core reports a
warning and uses the bounded fallback. No cloud model or paid embedding account
is required for normal indexing and recall.

## Durable writes and migration

Memory, Task State, continuity, proof, trace, and Autopilot writes use the
project/Vault transaction and lock rules. A repo and Vault on different file
systems do not receive a false cross-filesystem atomicity claim; receipts and
reconciliation make partial progress recoverable.

Legacy import copies source capsules into a backup, validates identity and
ownership, merges compatible Markdown with an import marker, preserves user
conflicts, and rebuilds disposable indexes. It imports data and user-owned
assets, never an old adapter architecture.
