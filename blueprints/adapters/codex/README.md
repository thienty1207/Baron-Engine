# Codex Adapter Blueprint

Codex is a thin native projection over Baron Core. Initialization materializes
the canonical project runtime first, then writes only the Codex bridge and
managed host entries.

```text
assets/core/** → .baron/core/**
                     ↓
AGENTS.md
.agents/skills/baron-engine/SKILL.md
.agents/skills/baron-engine/agents/openai.yaml
.codex/agents/*.toml
.codex/INDEX.md
.codex/hooks.json
```

`AGENTS.md` owns the automatic lifecycle contract. The bridge loads only
route-selected Core resources and uses the structured PrepareRequestV1 and
PreparePacketV1 protocol. It does not create a copied semantic tree under
`.codex/skills/**` and does not start a competing router.

Native hooks are optional accelerators with the managed contract as fallback.
Third-party hooks, user text, custom skills, agents, settings, and unknown
entries remain preserved by the ownership and update rules.
