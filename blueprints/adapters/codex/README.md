# Codex Adapter Blueprint

Target command:

```bash
baron init --codex
baron context --codex
```

Target generated assets:

```text
AGENTS.md
.codex/
  INDEX.md
  agents/
    INDEX.md
    code-reviewer.toml
    security-auditor.toml
    test-engineer.toml
  skills/
    INDEX.md
    superpowers/
    frontend-design/
    vibe-security-scan/
```

Codex adapter rules:

- `AGENTS.md` is the automatic startup contract.
- `.codex/skills/INDEX.md` is the skill routing surface.
- `.codex/agents/INDEX.md` is the subagent routing surface.
- Do not recursively load every skill or agent.
