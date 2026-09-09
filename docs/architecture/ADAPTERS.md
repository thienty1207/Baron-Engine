# Baron Adapter Architecture

Baron supports exactly two active integrations: Codex and Claude. Each is a
thin adapter over the canonical Core. Core owns memory, context, workflow,
skills, Task State, routing, continuity, proof, trace, and Autopilot policy.
The host adapters render those decisions in native files and event formats.

```text
assets/core/** → .baron/core/** → Codex bridge / Claude bridge
```

## Shared Core contract

An adapter supplies an explicit `OperationContext` and a structured
`PrepareRequestV1`. Core returns `PreparePacketV1`, including the current
project identity, profile lens, work shape, trusted context, selected Core
skills, the three quality agents, verification gates, warnings, blockers, and
the safe next action. The packet is the same semantic contract for both hosts;
only native rendering differs.

Core applies `TrustedRecallPolicy::Current` to every context-critical memory
path. It projects Task State from intent, constraints, plan progress, recovery,
proof, trace, affected files, blocker, and next action. A task can resume from
that evidence when either host opens the project.

## Codex adapter

`baron init --codex` writes the native projection:

- `AGENTS.md` managed lifecycle block;
- `.agents/skills/baron-engine/SKILL.md` thin bridge;
- `.agents/skills/baron-engine/agents/openai.yaml` invocation policy;
- `.codex/agents/*.toml` wrappers for the three Core quality agents;
- `.codex/INDEX.md` and `.codex/hooks.json` managed entries.

Codex does not need a copied Baron semantic skill tree. The bridge loads only
route-selected files below `.baron/core/**`. Hooks are optional accelerators;
the managed `AGENTS.md` contract is the fallback when a hook is missing,
untrusted, or skipped. Existing Codex hooks, settings, text, and user-owned
skills remain intact.

## Claude adapter

`baron init --claude` writes the native projection:

- `CLAUDE.md` managed lifecycle block;
- `.claude/skills/baron-engine/SKILL.md` thin bridge;
- `.claude/agents/*.md` wrappers for the three Core quality agents;
- `.claude/settings.json` managed hook entries;
- compact command and routing indexes where they are Baron-owned.

Claude also loads selected resources from `.baron/core/**`; it does not need a
full copied Baron skill library under `.claude/skills/**`. Claude host auto
memory is local context and is never a replacement for Core trusted memory.
Hooks are accelerators, and `CLAUDE.md` is the fallback correctness contract.

## Legacy compatibility

Old project files may contain opaque adapter values or managed records from an
earlier layout. The parser retains unknown serialized values for inspection and
transactional migration, but only Codex and Claude can be initialized as active
integrations. Legacy data is imported, validated, preserved, or quarantined;
old architecture is not reactivated.

Records for `.baron/core/**` belong to Core even when an older record names a
different owner. Supported Codex and Claude records migrate through the normal
managed-state transaction. Modified or ambiguous files remain user-visible and
are never silently overwritten or deleted.

## Hooks, preservation, and updates

Native hooks observe lifecycle events and may call the same Core prepare path as
the managed bridge. They do not create a second router or proof authority.
Event keys are deduplicated under the project lock, and reconciliation remains
available when hooks do not run.

Updates operate on the adapter's managed set and the Core baseline. Unknown
JSON/TOML keys, user text outside markers, custom skills and agents, and
third-party hooks are preserved. Missing or changed baselines fail closed and
produce a recoverable review record. An update never downloads a release from
inside an agent task.

Autopilot can perform bounded housekeeping and present candidates for natural
review. It cannot promote a candidate into memory truth, routing policy, a
skill, or a Core asset without the existing approval authority.

## Adapter rule

Adapters must stay thin. They translate explicit Core inputs and outputs,
preserve host-owned content, and report hook or provider degradation. They do
not fork Baron workflow, load every skill recursively, or infer correctness
from a project-global preference.
