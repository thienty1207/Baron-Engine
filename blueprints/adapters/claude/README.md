# Claude Adapter Blueprint

Target command:

```bash
baron init --claude
baron context --claude
```

Target generated assets:

```text
CLAUDE.md
.claude/
  commands/
  hooks/
```

Claude adapter rules:

- `CLAUDE.md` must make Baron visible to fresh Claude sessions.
- Claude should be instructed to run Baron context automatically.
- Claude-specific command/hook surfaces are adapter output, not Baron core.
