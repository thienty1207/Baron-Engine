# Baron Context Compiler

The Context Compiler is the Phase 3 bridge between repository understanding,
Vault memory, and an agent tool. It produces a bounded Markdown brief on stdout.
It does not create or update adapter files.

## Inputs

- repository path
- Vault path from `--vault` or `BARON_VAULT`
- one adapter target: Codex, Claude, or generic agent
- optional task text

## Selection Order

1. Survey Engine Project Atlas
2. bounded current execution state
3. current-project memory
4. relevant approved global memory
5. adapter-specific guidance
6. explicit unknowns and skipped context

Weak cross-project memory, global candidates, full session history, and broad
documentation bodies are not loaded into the compact bundle.

## Risk Guidance

When `--task` is provided, Baron classifies the task into a simple risk lane:

- `low`: documentation, copy, and typo work
- `medium`: normal implementation work
- `high`: auth, permissions, tenant/RLS, payments, migration, security,
  secrets, uploads, or data-loss work
- `unknown`: no task was supplied and the survey found no clear risk signal

Risk guidance does not prove safety. It tells the later workflow how strong the
verification evidence should be.

## Bounded Output

- overall context is capped at 20,000 characters
- execution-state excerpts are capped at 2,000 characters
- lists such as commands, entrypoints, risks, and read-first files are capped
- output records what was skipped so agents do not mistake omission for absence

## Write Boundary

The Context Compiler may create or refresh Vault scaffold/index artifacts
through the Phase 2 memory engine. It does not write to the target repository.
Phase 4 owns managed adapter-file generation and update behavior.
