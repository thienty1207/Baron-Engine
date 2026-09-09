# Claude Adapter Blueprint

Claude is a thin native projection over Baron Core. Initialization materializes
the canonical project runtime once and writes Claude's bridge, wrappers,
settings, and diagnostic indexes.

```text
assets/core/** → .baron/core/**
                     ↓
CLAUDE.md
.claude/skills/baron-engine/SKILL.md
.claude/agents/*.md
.claude/settings.json
.claude/commands/...  (diagnostic conveniences where retained)
```

`CLAUDE.md` owns the automatic lifecycle contract. The bridge uses
`PrepareRequestV1`/`PreparePacketV1`, loads only route-selected Core resources,
and sets `disable-model-invocation: true` so it cannot create a second
automatic route. Claude does not receive a copied Core skill library under
`.claude/skills/**`.

Native hooks are optional accelerators and `CLAUDE.md` is the fallback.
Personal skills, commands, settings, hooks, and text outside Baron markers are
preserved by the ownership and update rules.
