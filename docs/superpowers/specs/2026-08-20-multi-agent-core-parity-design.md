# Baron Multi-Agent Core Parity Design

## Decision

Baron has one engine and one Baron-managed core. Codex, Claude, Reasonix, and
the generic agent surface are adapters over that core; no adapter may silently
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

## Reasonix materialization

Reasonix receives the shared core through these Baron-managed paths:

- `.reasonix/INDEX.md`;
- `.reasonix/skills/INDEX.md` and `.reasonix/skills/**`;
- `.reasonix/agents/INDEX.md` and `.reasonix/agents/**`;
- the existing `REASONIX.md`, command files, and settings/hooks bridge.

The Reasonix startup contract directs the agent to read the narrow index and
only the task-routed skill/agent body. It must not recursively load every
asset, and it must not create a Reasonix-only memory namespace.

## Preservation and switching

Installing or switching an adapter may create missing Baron-managed files. It
must preserve unmarked user files and changed managed files, report conflicts,
and rely on the existing managed-baseline planner for later three-way updates.
Switching Codex, Claude, Reasonix, and generic agents changes only the active
adapter and its bridge; it never copies memory, changes `project_id`, or
deletes another adapter's files.

## Acceptance gates

The correction is complete only when tests prove:

1. Codex and Reasonix expose the same embedded skill and agent inventory,
   indexes, and mandatory quality-agent contracts.
2. Reasonix startup/context points at the shared core and uses the same
   project/Vault commands as Codex.
3. Codex -> Reasonix -> Claude -> Generic -> Codex preserves every adapter's
   files, project identity, and shared history.
4. Existing user skills, agents, commands, settings, hooks, and instructions
   are never silently overwritten.
5. Missing managed Reasonix assets are reconciled, while changed or ambiguous
   assets are left untouched and reported.
6. Existing engine, memory, fallback, release, and cross-platform tests remain
   green.
