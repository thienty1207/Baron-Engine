# Baron Architecture

Baron Core owns the product semantics. Codex and Claude are native adapters that
translate a Core decision into host files, hooks, and agent wrappers. Neither
adapter owns memory, workflow, routing policy, or proof truth.

```text
assets/core/**                 packaged canonical source
      ↓ install / reconcile
.baron/core/**                 one project runtime and one owner
      ↓ selected resources
Codex native adapter            Claude native adapter
AGENTS.md + .agents/           CLAUDE.md + .claude/
```

Core contains project identity, trusted memory, Task State, Superpowers,
platform profiles, routing, continuity, recovery, proof, trace, quality gates,
and Autopilot candidate handling. Hooks, session replay, Wiki, and CodeGraph
are optional accelerators around that authority.

## Request lifecycle

Every task is represented by an explicit `OperationContext`. It carries the
project ID, session and request identity, adapter identity, task text, profile,
work shape, risk, current phase, and the evidence available to Core. The
adapter never infers correctness from a project-global adapter preference.

The native hook or managed host contract sends a structured `PrepareRequestV1`.
Core validates the request, applies the memory firewall and
`TrustedRecallPolicy::Current`, resumes Task State when present, and returns a
`PreparePacketV1`. The packet contains the selected profile lens, canonical
skills, quality agents, bounded context, required verification, blockers,
warnings, and next action. Task text is structured data; it is never assembled
into a shell command.

```text
prompt / native event
      ↓
OperationContext + PrepareRequestV1
      ↓
survey → trusted memory → Task State → profile-aware route
      ↓
PreparePacketV1
      ↓
Codex or Claude works through the selected Core resources
      ↓
proof / trace / continuity / Autopilot review
```

## Context and state priorities

The context compiler uses priority-aware budgeting rather than dropping the
tail of a flat string. `Tier 0` is never silently dropped and includes project
identity, task identity, intent, constraints, non-goals, current plan/work
state, recovery, blockers, next action, route, and mandatory proof/completion
gates. Tier 1 carries trusted decisions, current source evidence, proof/trace,
and the immediate action. Tier 2 carries profile and bounded Wiki, CodeGraph,
and session-replay material. Tier 3 carries low-priority diagnostics and
Autopilot candidate summaries.

Task State is the durable projection of an active task. It links intent,
constraints, plan progress, last successful step, affected files, proof and
trace state, blocker, recovery packet, and safe next action. A stopped session
can therefore resume from evidence without asking the user to repeat known
facts.

## Ownership and persistence

| Owner | Managed paths | Authority |
| --- | --- | --- |
| Core | `.baron/core/**`, project identity, Task State, memory and receipts | canonical semantics and durable state |
| Codex | `AGENTS.md` managed block, `.agents/skills/baron-engine/**`, `.codex/**` managed entries | native projection and optional hooks |
| Claude | `CLAUDE.md` managed block, `.claude/skills/baron-engine/**`, `.claude/**` managed entries | native projection and optional hooks |
| User | source, Vault Markdown, custom assets, unknown settings and hooks | preserved content and explicit instructions |

One live managed path has one owner. Reconciliation and updates preserve user
content outside Baron markers, quarantine ambiguous ownership, and fail closed
when a baseline is missing or changed.

Vault Markdown is the source of truth for durable memory. SQLite, replay,
Wiki, CodeGraph, and capability caches are disposable accelerators. A trusted
memory result must match the project firewall and current trust policy before
ranking; candidate, contested, superseded, expired, and unrelated project
records remain ineligible as current truth.

## Hooks, fallback, and safety

Codex and Claude hooks observe session start, prompt, compaction, and stop
events when the host trusts and runs them. They accelerate the same Core
prepare, journal, and reconciliation paths. The managed `AGENTS.md` or
`CLAUDE.md` contract is always the fallback, so missing or untrusted hooks do
not create a second behavior model.

Proof gates require execution evidence, not provider presence. High-risk work
cannot complete without valid proof and a passing trace. Autopilot may perform
bounded housekeeping and write reviewable candidates; it cannot silently change
skills, routing policy, workflow, or Core assets.

## Optional accelerators

The local CodeGraph, Wiki, session replay, capability probes, and native hooks
are optional accelerators. They are bounded, project-scoped, and disposable.
Their absence produces a warning or a Core fallback and never changes ownership
or turns an observation into proof. Core remains fully responsible for the
decision and for the evidence recorded in the project and Vault.
