# Baron Adapter Architecture

Baron core owns memory, context, plan, harness, proof, and trace behavior.
Adapters only translate that core into the shape expected by an agent tool.

All adapters share `.baron/project.toml` for committed project identity and
`.baron/local.toml` for the machine-local Vault path. Repeated init may register
multiple adapters. `baron update` refreshes registered adapters without
deleting unknown custom assets.

## Initial Adapters

### Codex

Command:

```bash
baron init --codex
baron context --codex
```

Outputs:

- `AGENTS.md`
- `.codex/skills/INDEX.md`
- `.codex/agents/INDEX.md`
- core Superpowers skill
- 3 core quality agents
- optional frontend/security skills
- managed root instructions that preserve user text outside Baron markers

### Claude

Command:

```bash
baron init --claude
baron context --claude
```

Outputs:

- `CLAUDE.md`
- Claude-readable imports or command guidance
- Baron context and harness instructions
- Claude-readable Superpowers, optional domain skills, and quality agents

### Generic Agent

Command:

```bash
baron init --agent
baron context --agent
```

Outputs:

- `AGENT.md`
- `baron-context.md`
- optional JSON context bundle
- portable core skills and quality-agent contracts under `.baron/core`

## Adapter Rule

Adapters must not fork Baron behavior. They only translate Baron behavior.
Every adapter requires automatic context, plan, harness, proof, and trace
behavior. Platform-specific hooks are accelerators, not separate workflow truth.

## Migration Boundary

Agent Bootstrap migration does not reuse a legacy adapter. Baron inventories
and backs up the old workspace, imports user-owned data, then installs a fresh
Baron adapter.

The Codex takeover preserves user text outside legacy managed markers and keeps
validated custom skills/agents. Baron bundled assets and the three core quality
agents are regenerated from `assets/core/`; invalid custom assets are
quarantined and never enter routing indexes.

After verification, the active adapter contains Baron commands only. The legacy
Node runtime, config, manifest, and managed hook are retired.
