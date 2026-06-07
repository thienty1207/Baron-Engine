# Baron Adapter Architecture

Baron core owns memory, context, plan, harness, proof, and trace behavior.
Adapters only translate that core into the shape expected by an agent tool.

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

## Adapter Rule

Adapters must not fork Baron behavior. They only translate Baron behavior.
