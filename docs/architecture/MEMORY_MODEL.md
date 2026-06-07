# Baron Memory Model

Baron uses two layers:

1. Markdown vault as source of truth.
2. SQLite/cache/index as accelerator.

## Memory Classes

- `project_verified`: current project fact backed by repo, test, decision, or trace.
- `project_likely`: current project fact inferred from repo but not proven.
- `project_stale`: old memory that may no longer match code.
- `global_verified`: approved reusable memory across projects.
- `global_candidate`: possible reusable memory that is not trusted yet.
- `cross_project`: memory from another project; blocked unless explicitly matched.
- `unknown`: missing fact that must not be guessed.

## Firewall Rule

Default retrieval order:

```text
project_verified
project_likely
global_verified
project_stale as warning
cross_project only on strong explicit match
unknown remains unknown
```

## Vault Shape

Target vault shape:

```text
Vault/
  AGENTS.md
  Init.md
  Projects/
    <project-slug>/
      README.md
      Facts.md
      Decisions.md
      Tasks.md
      Plans/
      ProductHarness/
      Sessions/
      Artifacts/
  Artifacts/
    Baron/
      memory-index.sqlite
      memory-engine-state.json
      APPROVED_GLOBAL.md
      GLOBAL_CANDIDATES.md
```

SQLite is disposable. Markdown is durable.
