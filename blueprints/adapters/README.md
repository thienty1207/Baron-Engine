# Baron Adapter Blueprints

Baron adapters translate Baron core state into the format each agent tool can
read.

Adapters must not fork workflow logic. Baron core decides memory, context, plan,
harness, proof, and trace behavior.

Initial adapters:

- `codex`
- `claude`
- `generic-agent`
