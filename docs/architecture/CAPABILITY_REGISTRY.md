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

## Optional Project Code Map

Baron may register `graphify-local` as an optional `code-map` CLI provider for
an initialized project. Its registration is a capability declaration, not a
permission to run it and not proof that the provider is installed.

- The provider remains optional; absence never blocks a task or a proof gate.
- Baron registers it only when the project has no existing `code-map` provider.
  A project-owned provider keeps ownership unchanged.
- Code-map cache and state live only under
  `.baron/cache/code-graph/` in the current repository. They never enter Vault
  Markdown, cross-project recall, or durable memory.
- A future local provider must be bounded and source-verified. Survey Engine
  remains the fallback when no usable map is available.

## Adapter Awareness

Presence is evaluated for Codex, Claude, or generic-agent context. A cached
Codex observation is not reused as Claude evidence. MCP, skill, and
agent-adapter providers must declare their compatible adapters.

## Automation

Baron-managed adapter instructions run `baron capability check` silently before
`baron context`. Context reads only a bounded summary. It does not recursively
scan tools or perform network probes.

