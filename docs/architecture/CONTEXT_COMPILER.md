# Baron Context Compiler

The Context Compiler is a read-only Core service that turns repository
evidence, trusted memory, Task State, and a task request into a bounded brief
for a native adapter. It does not create a second adapter brain or write host
files.

## Structured inputs

The compiler receives an `OperationContext` through `PrepareRequestV1`:

- repository path and stable project ID;
- Vault routing and the current session/request identity;
- explicit adapter identity (`codex` or `claude`);
- task text, intent, constraints, profile, work shape, and risk;
- current Task State and relevant prior proof or recovery evidence.

The request is structured JSON or an equivalent native hook payload. Arbitrary
task text is never placed inside a shell command. `control-plane prepare` is the
high-level orchestration boundary; low-level context commands remain diagnostic
interfaces.

## Trusted selection

Core uses `TrustedRecallPolicy::Current` for every context-critical memory path.
Selection is bounded and project-aware:

1. project identity and repository survey;
2. current Task State, intent, plan, recovery, and blockers;
3. execution receipts, proof, trace, and Product Harness state;
4. trusted current-project memory;
5. relevant approved global memory;
6. profile lens and route-selected Core skills and agents;
7. optional Wiki, CodeGraph, and session-replay accelerators;
8. explicit unknowns, warnings, and skipped-context diagnostics.

Candidate, contested, superseded, expired, stale, and unrelated project records
remain labelled or blocked. They cannot become current truth merely because a
semantic score is high. Host-local memory is context only and cannot override
Core intent, decisions, continuity, or recovery.

## Priority-aware context budget

The compiler returns a `PreparePacketV1` with a bounded context section. It
compresses or omits lower priorities before touching higher priorities:

| Tier | Content | Rule |
| --- | --- | --- |
| Tier 0 | project/task identity, intent, constraints, non-goals, current plan/work state, recovery, blockers, route, next action, mandatory proof/completion gates | never silently dropped |
| Tier 1 | trusted decisions and memory, changed files, current source evidence, proof/trace, immediate action | retain when relevant |
| Tier 2 | profile lens, bounded Wiki/CodeGraph excerpts, session replay | include when route-selected |
| Tier 3 | low-priority diagnostics, capability reports, Autopilot candidate summary | compress or omit first |

Task State is projected into Tier 0 so an interrupted task carries its original
intent, constraints, plan, last successful step, failed/interrupted state,
proof/trace state, affected files, blocker, and safe next action. The next
session can resume from evidence without asking the user to restate it.

## Adapter projections

Codex and Claude receive the same semantic packet and only the selected
canonical Core resources. Codex renders its result through `AGENTS.md` and the thin
`.agents/skills/baron-engine` bridge. Claude renders it through `CLAUDE.md` and
the thin `.claude/skills/baron-engine` bridge. Neither bridge loads the complete
Core tree recursively or owns the task route.

## Optional accelerators and fallback

Native hooks, session replay, Wiki, CodeGraph, capability probes, and cached
summaries are optional accelerators. A missing, stale, malformed, untrusted, or
timed-out accelerator produces a warning and the bounded Core fallback. Hook
absence never becomes a claim that automation ran.

## Write boundary

Compilation may refresh disposable indexes through Core's memory service. Durable
Task State, continuity, proof, and trace writes use the project/Vault transaction
and lock rules. Adapter files are written only by initialization, reconciliation,
or the preserve-first update transaction; user source and Vault Markdown remain
outside that write set.
