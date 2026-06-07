# Generic Agent Adapter Blueprint

Target command:

```bash
baron init --agent
baron context --agent
```

Target generated assets:

```text
AGENT.md
baron-context.md
baron-context.json
```

Generic adapter rules:

- Keep output plain and portable.
- Avoid assuming a specific agent runtime.
- Prefer markdown for human/agent reading and JSON for automation.
