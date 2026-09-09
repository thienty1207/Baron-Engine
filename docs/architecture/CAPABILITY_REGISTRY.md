# Baron Capability Registry

The Capability Registry is a Core service. It records what a project intends
to use and what the current machine can safely observe; Codex and Claude only
provide native host surfaces for those checks.

## Durable and cached state

- `.baron/capabilities.toml` is the committed project contract.
- `.baron/cache/capability-state.json` is a rebuildable machine observation.
- Proof and Trace Markdown contain durable execution evidence.

Deleting a cache loses no capability definition and no proof. Capability
observations are project-scoped and carry the adapter/session context that
produced them.

## Three separate facts

Baron never collapses these facts:

1. **Registered** means the project intends to use a provider.
2. **Present** means the current machine appears able to use it.
3. **Executed** means task-specific evidence names the capability, provider,
   command or operation, and real result.

Only executed evidence can support a tool-backed completion claim. A registered
or present provider is not proof that a check ran.

## Provider kinds and degradation

The registry supports bounded `cli`, `binary`, `mcp`, `skill`, `http`,
`agent_adapter`, and `code-map` providers. Optional providers degrade with a
warning. A missing required provider or missing execution evidence leaves Proof
insufficient and can block a high-risk completion. Trace scoring inherits that
gate.

Codex and Claude observations are not interchangeable evidence. A native hook
may accelerate a check, but the same Core receipt and project identity rules
apply when the managed fallback performs it.

## Core ownership and Task State

Capability decisions are inputs to `OperationContext` and `PreparePacketV1`.
Core combines them with profile, task intent, repository survey, work shape,
risk, current Task State, and previous proof failures to select the smallest
useful skills, agents, and verification. The adapters do not decide the route.

Hooks are optional accelerators. Autopilot may report bounded capability
housekeeping or a reviewable candidate, but it cannot turn an observation into
trusted memory or silently change a capability contract.

## Optional local CodeGraph provider

`graphify-local` is an optional project-scoped code-navigation accelerator. Its
registration is not permission to run it and is not proof that it is installed.
Output is staged, size-checked, path-checked, identity-bound, and checksummed
before `.baron/cache/code-graph/` changes. A missing, stale, malformed, timed
out, or failed provider leaves the last known-good cache and falls back to the
Survey Engine.

## Automation boundary

The managed Codex and Claude contracts can request a silent capability check as
part of Core preparation. They do not ask a normal user to operate internal
registry commands. Diagnostics remain available in the advanced command
catalog, while missing optional providers stay visible as warnings.
