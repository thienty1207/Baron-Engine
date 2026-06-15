# Baron Capability Registry Design

Date: 2026-06-15
Status: approved for implementation

## Purpose

Baron must know which project capabilities are intentionally registered, which
providers are available on the current machine, which active agent adapter can
use them, and whether a claimed tool-backed verification has real execution
evidence.

The registry extends Baron. It does not copy the Repository Harness runtime,
database, or tool schema.

## Core Decisions

### Project Contract

The committed source of truth is `.baron/capabilities.toml`.

Each provider records:

- a unique provider name
- a normalized capability id
- provider kind: `cli`, `binary`, `mcp`, `skill`, `http`, or `agent_adapter`
- requirement: `optional` or `required`
- command or scan target when applicable
- compatible adapters when the provider is adapter-specific
- a concise description

Workflows request a capability such as `security-scan` or
`deploy-verification`; they do not hard-code a provider name.

### Machine Observation Cache

Presence observations are stored in
`.baron/cache/capability-state.json`. This file is ignored by Git and can be
deleted or rebuilt without losing the registry.

Every observation records:

- `present`, `missing`, or `unknown`
- checked time
- probe method
- bounded evidence
- compatibility with the selected adapter

Baron never stores credentials, command output bodies, or HTTP response bodies
in this cache.

### Presence Is Not Execution

Presence evidence proves only that a provider appears usable. It never proves
that the provider ran for the current task.

Tool-backed proof requires structured execution evidence attached to
`baron proof record`:

- capability id
- provider name
- exact bounded result summary

A required capability that is missing, incompatible, unknown, or lacks
execution evidence makes the latest proof insufficient. Trace scoring inherits
that failure and cannot accept the completion claim.

### Probes

- `cli` and `binary`: resolve an explicit path or the current `PATH`.
- `skill` and `mcp`: resolve a configured file or directory.
- `http`: explicit `capability check` may perform a bounded TCP reachability
  probe; context compilation never performs network I/O.
- `agent_adapter`: compare the provider target with adapters registered in
  `.baron/project.toml`.

Unsupported or underspecified probes return `unknown`; Baron does not guess.

### Adapter Compatibility

CLI, binary, and HTTP providers are portable unless adapters are explicitly
restricted.

MCP, skill, and agent-adapter providers must either name compatible adapters or
be reported incompatible for the current adapter. Compatibility and presence
are separate facts.

### Degradation

- No providers registered: capability is inactive and causes no penalty.
- Optional provider missing: continue with a visible warning and fallback.
- Required capability with no present compatible provider: proof confidence is
  insufficient and completion is blocked.
- Multiple providers: Baron exposes all compatible present providers in stable
  order; the agent selects the narrowest appropriate provider.

## CLI Contract

```text
baron capability register <capability> --name <provider> --kind <kind>
  [--required] [--command <command>] [--scan <target>]
  [--adapter <codex|claude|agent>]... --description <text>
baron capability check [capability] [repo-path] [--adapter <adapter>] [--json]
baron capability list [repo-path] [--adapter <adapter>] [--json]
baron capability remove <capability> --name <provider> [repo-path]
```

`baron proof record` gains repeatable structured evidence:

```text
--capability-evidence "<capability>|<provider>|<result summary>"
```

## Automatic Behavior

Generated adapter instructions require agents to:

1. run `baron capability check` silently at session start
2. read the bounded Capability Summary in `baron context`
3. use only compatible present providers
4. attach capability execution evidence after actually running a provider
5. never translate provider presence into a completed verification claim

Context remains read-only for product files. It reads the latest cache and
shows a bounded summary; the startup contract owns the automatic refresh.

## Data Boundaries

- Capability registry: committed project configuration.
- Capability state: rebuildable machine cache.
- Proof and trace: durable repo and Vault Markdown.
- Vault memory: unchanged source of truth for facts, decisions, plans, and
  execution history.

## Safety

- Provider names and capability ids are normalized and path-safe.
- Registry writes are atomic.
- HTTP checks have a short timeout and store reachability only.
- Commands are not executed during presence detection.
- No provider can replace Superpowers or the three core quality agents.

## Verification

Phase 7 is complete only when tests prove:

- all provider kinds serialize and probe predictably
- adapter compatibility is enforced
- optional missing providers degrade cleanly
- required missing providers weaken proof and trace
- presence alone cannot satisfy execution evidence
- context output is bounded
- adapter startup instructions trigger automatic checks
- existing Phase 0-6 behavior remains green

