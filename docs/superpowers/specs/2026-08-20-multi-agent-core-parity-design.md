# Baron Multi-Agent Core Parity Design

## Decision

Baron has one engine and one Baron-managed core. Codex and Claude are adapters
over that core; unsupported historical values remain opaque migration input and no adapter may silently
receive a thinner set of Baron skills, quality agents, routing indexes, or
startup lifecycle guidance.

This is a maintenance correction published on the `4.2.2` source line. It does
not change the intelligence generation, Vault schema, memory model, or
fallback policy.

## Source of truth

`assets/core/` remains the only embedded runtime source for bundled skills and
agents. Each adapter may materialize that same source tree under its native
project directory, but the adapter path is only a view, never a second owner.

The shared contract includes:

- the complete bundled skill tree, including Superpowers and optional domain
  skills;
- the three mandatory quality agents and optional agent assets;
- the adapter-specific skill and agent indexes with the same routing policy;
- the Baron startup contract, context/status commands, capability/runtime checks,
  control-plane routing, proof, trace, continuity, and autopilot guidance;
- one project ID, one Vault route, one session journal, and one memory/Wiki/
  CodeGraph namespace.

## Historical legacy projection materialization

The historical legacy projection received the shared core through these
Baron-managed paths:

- `legacy/INDEX.md`;
- `legacy/skills/INDEX.md` and `legacy/skills/**`;
- `legacy/agents/INDEX.md` and `legacy/agents/**`;
- the existing legacy instruction file, command files, and settings/hooks bridge.

The legacy startup contract directed the agent to read the narrow index and
only the task-routed skill/agent body. It must not recursively load every
asset, and it must not create a legacy-only memory namespace.

## Preservation and switching

Installing or switching an adapter may create missing Baron-managed files. It
must preserve unmarked user files and changed managed files, report conflicts,
and rely on the existing managed-baseline planner for later three-way updates.
Switching Codex and Claude changes only the active
adapter and its bridge; it never copies memory, changes `project_id`, or
deletes another adapter's files.

## Acceptance gates

The correction is complete only when tests prove:

1. The historical legacy projection exposed the same embedded skill and agent
   inventory, indexes, and mandatory quality-agent contracts as Codex.
2. The legacy startup/context surface pointed at the shared core and used the
   same project/Vault commands as Codex.
3. The historical multi-adapter switching sequence preserved files, project
   identity, and shared history.
4. Existing user skills, agents, commands, settings, hooks, and instructions
   are never silently overwritten.
5. Missing managed legacy assets are reconciled, while changed or ambiguous
   assets are left untouched and reported.
6. Existing engine, memory, fallback, release, and cross-platform tests remain
   green.
