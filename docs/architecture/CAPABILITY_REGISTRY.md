# Baron Capability Registry

Baron registers project tools by what they can do, not by a hard-coded product
name.

## Source And Cache

- `.baron/capabilities.toml` is the committed project contract.
- `.baron/cache/capability-state.json` is a machine observation cache.
- Proof and Trace Markdown contain durable execution evidence.

Deleting the cache loses no capability definitions and no proof.

## Provider Kinds

- `cli`: a command resolved from a project path or `PATH`
- `binary`: a standalone executable resolved from a project path or `PATH`
- `mcp`: an adapter-specific MCP configuration or marker path
- `skill`: an adapter-specific skill path
- `http`: a bounded endpoint reachability check
- `agent_adapter`: a Baron adapter registered in `.baron/project.toml`

## Three Separate Facts

Baron does not merge these facts:

1. Registered means the project intends to use a provider.
2. Present means the current machine appears equipped to use it.
3. Executed means task-specific proof names the capability, provider, and real
   result.

Only the third fact can support a tool-backed completion claim.

## Degradation

- No registered provider: capability is inactive.
- Missing optional provider: work continues with a warning.
- Missing required provider: Proof is insufficient.
- Present required provider without execution evidence: Proof is insufficient.
- Trace scoring inherits failed capability gates and blocks completion.

## Adapter Awareness

Presence is evaluated for Codex, Claude, or generic-agent context. A cached
Codex observation is not reused as Claude evidence. MCP, skill, and
agent-adapter providers must declare their compatible adapters.

## Automation

Baron-managed adapter instructions run `baron capability check` silently before
`baron context`. Context reads only a bounded summary. It does not recursively
scan tools or perform network probes.

