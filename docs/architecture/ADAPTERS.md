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
- native project hooks in `.codex/hooks.json`

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
- native project hooks in `.claude/settings.json`

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
Every adapter must also route work through `baron control-plane route` before
loading optional skills or dispatching quality gates. A mandatory quality gate
does not count until `baron control-plane record-gate` records evidence.
Managed runtime skills and agents are local Baron assets. They must not depend
on live external guidance to operate. If a custom skill or agent looks weak,
conflicting, externally dependent, or recursively orchestrated, Baron can audit
and quarantine it through the hidden asset lifecycle commands before routing.

Context startup also refreshes the project-local session replay index. When a
task is provided, the context bundle may include a few matching prior messages
from imported sessions, but it never dumps full session history and it filters
results by project identity.

Codex and Claude hooks record SessionStart, prompt, edit checkpoint, and Stop
events. SessionStart injects bounded context. Stop reconciliation blocks one
premature completion attempt when active work lacks proof or a passing trace,
then avoids a hook loop. Project hook trust remains controlled by the agent
tool; Baron reports observed events instead of assuming hooks executed.

Hook JSON and skill/agent indexes are merged through Baron-managed entries.
Unknown user hook groups and custom routing text remain intact across update.
Skill and agent indexes include ownership, trigger, exclusion, evidence, and
conflict fields so custom routing can be preserved without weakening Baron
contracts.

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
