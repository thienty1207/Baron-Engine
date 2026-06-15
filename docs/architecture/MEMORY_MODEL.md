# Baron Memory Model

Baron uses two layers:

1. Markdown vault as source of truth.
2. SQLite/cache/index as accelerator.

Phase 2 implemented the first working version of this model through
`baron memory status`, `baron memory index`, `baron memory compact`, and
`baron recall`. Phase 10 replaces full destructive rebuild behavior with an
incremental source cache, project-ID isolation, multilingual concept ranking,
task-focused context, and automatic session ingestion. Memory commands require
`--vault <path>` or `BARON_VAULT`.

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
    <project-slug>--<project-id-prefix>/
      .baron-project.json
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
        Imported/
      Research/
      Notes/
      Open Questions.md
      Handoff.md
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
  or incrementally refreshes `memory-index.sqlite` from Markdown.
- `memory compact` rebuilds the index and prints a bounded Memory Firewall Brief.
- `recall` rebuilds the index and returns ranked memory after firewall gating.
- Current-project memory is preferred over all other project memory.
- `APPROVED_GLOBAL.md` may be used when relevant.
- `GLOBAL_CANDIDATES.md` is indexed for diagnostics but not trusted as fact.
- Plan, Product Harness, proof, and trace Markdown are indexed as distinct
  memory kinds so later sessions can recall both intent and verification.
- Migrated research, notes, open questions, and handoff Markdown are indexed as
  distinct memory kinds instead of becoming inert archive files.
- Product Harness `TEST_MATRIX.md` ties the current story to proof status and
  evidence while remaining plain Markdown that can be rebuilt or inspected.

## Phase 10 Behavior

- `.baron/project.toml` schema v2 stores a stable project ID.
- Vault capsules include the slug plus an ID prefix, preventing same-name
  projects from sharing memory.
- Existing slug-only capsules migrate to the identity capsule without deleting
  Markdown.
- SQLite tracks each Markdown source by path, modified time, size, and content
  hash. Unchanged files are reused; changed and deleted files are reconciled.
- Survey and memory discovery have no silent fixed file-count cutoff.
- Recall combines Unicode lexical overlap, concept aliases, title/path/kind,
  confidence, proof, recency, project identity, and firewall rules.
- Common Vietnamese/English engineering concepts are matched locally without a
  cloud model or API key.
- Initialized projects import a bounded recent batch of confidently matched
  Codex/Claude sessions during context startup.
- Imported sessions are redacted, deduplicated, and stored as clean Markdown;
  import state is rebuildable metadata under project `Artifacts/`.

## Legacy Import

Migration reads the source Vault from `vault.config.json`. An explicit
`--vault` is the Baron destination, so old and new Vaults may be different.

The source capsule is copied into the migration backup before import. Existing
Baron Markdown wins on conflict; compatible legacy Markdown is merged with an
explicit import marker, while non-Markdown conflicts are preserved separately.
SQLite is rebuilt from the resulting Markdown and remains disposable.
