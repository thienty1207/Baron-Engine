# Baron Memory Model

Baron uses two layers:

1. Markdown vault as source of truth.
2. SQLite/cache/index as accelerator.

Phase 2 implements the first working version of this model through
`baron memory status`, `baron memory index`, `baron memory compact`, and
`baron recall`. Memory commands require `--vault <path>` or `BARON_VAULT`.

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
        TEST_MATRIX.md
      Proofs/
      Traces/
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

## Phase 2 Behavior

- `memory status` inspects Vault health and does not create files.
- `memory index` creates the Vault scaffold and project capsule, then rebuilds
  `memory-index.sqlite` from Markdown.
- `memory compact` rebuilds the index and prints a bounded Memory Firewall Brief.
- `recall` rebuilds the index and returns ranked memory after firewall gating.
- Current-project memory is preferred over all other project memory.
- `APPROVED_GLOBAL.md` may be used when relevant.
- `GLOBAL_CANDIDATES.md` is indexed for diagnostics but not trusted as fact.
- Plan, Product Harness, proof, and trace Markdown are indexed as distinct
  memory kinds so later sessions can recall both intent and verification.
- Product Harness `TEST_MATRIX.md` ties the current story to proof status and
  evidence while remaining plain Markdown that can be rebuilt or inspected.
