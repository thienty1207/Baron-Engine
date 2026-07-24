# Phase 42 - Code Graph Provider Contract

Date: 2026-07-24
Release target: Baron `3.6.0`

## Completed

- Added the provider-neutral Code Graph model with explicit `extracted` and
  `inferred` confidence.
- Added a project-local rebuildable cache under `.baron/cache/code-graph/`.
- Added source fingerprints based on gitignore-aware paths, sizes, and modified
  times without a fixed file-count limit.
- Added state checksum, project identity, cache-path, traversal, and
  symlink/junction validation.
- Added bounded graph hit normalization/rendering and a strict optional
  `graphify-local` capability registration that never overwrites a project
  `code-map` provider.

## Evidence

```text
cargo fmt --all -- --check
cargo test -p baron-core --test code_graph --test capability
cargo test -p baron-cli --test adapter_cli non_shadow_init_installs_codex_and_configuration
```

All passed. The Windows junction fixture proves the graph cache refuses a
reparse-point escape before it can write state. A static scan confirms Phase 42
contains no provider subprocess invocation.

## Resume Point

Proceed to Phase 43 only. Use a deterministic fake provider and permit exactly
the local version, code-only extract, and bounded JSON query calls documented in
the plan. Do not invoke installers, hooks, global graphs, Vault paths, semantic
backends, or networking.

