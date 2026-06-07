# Baron Architecture

Baron is split into a core engine and adapter outputs.

```text
baron-cli
  -> baron-core
       -> survey engine
       -> memory engine
       -> context compiler
       -> plan engine
       -> harness engine
       -> proof engine
       -> trace engine
  -> baron-adapters
       -> codex adapter
       -> claude adapter
       -> generic agent adapter
```

## Data Flow

```text
repo + vault + user task
  -> survey/context compiler
  -> memory firewall
  -> active plan and harness state
  -> adapter-specific context output
  -> agent work
  -> proof + trace + memory write-back
```

## Source Hierarchy

1. User request and repo files.
2. Verified project memory.
3. Active plan and product harness state.
4. Verified global memory.
5. Cross-project memory only when explicitly matched.
6. Stale/unknown memory as reference only.

## Safety Model

- Shadow mode reads only.
- Update mode must preserve user-owned files.
- Adapter files use managed markers where possible.
- Baron must never mark completion without verification evidence.
- Baron must never promote cross-project memory as truth without confidence.
