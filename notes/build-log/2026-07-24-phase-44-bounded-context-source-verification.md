# Phase 44 - Automatic Bounded Context And Source Verification

Date: 2026-07-24
Release target: Baron `3.6.0`

## Completed

- Added hidden AI-facing code-map status, refresh, and query commands without
  adding any normal user command or help clutter.
- Added task routing for architecture, dependency, impact, ownership,
  entrypoint, call-flow, cross-module, and refactor work; docs/copy/config-text
  tasks are skipped.
- Kept `baron context` free of Graphify process calls. It reports only a
  bounded cache/fallback summary and leaves refresh/query to adapter automation.
- Added project-local task-query caching with eight-hit and character bounds.
- Added source verification: extracted hits need current symbol evidence,
  inferred hits stay advisory, and missing/escaping/foreign paths cannot become
  proof.

## Evidence

```text
cargo test -p baron-core --test code_graph --test code_map_context --test context_compiler --test graphify_provider
cargo test -p baron-cli --test code_map_cli
cargo test -p baron-adapters --test adapter_lifecycle
cargo clippy -p baron-core -p baron-cli -p baron-adapters --all-targets -- -D warnings
```

All passed. The hidden CLI status test proves a missing local provider does not
create a graph cache and the main public help stays free of automation/code-map
commands.

## Resume Point

Proceed to Phase 45 only. Certify same-name project isolation, source
fingerprint behavior for Baron-managed docs, old/large repo boundedness, stale
and corrupt cache fallback, custom adapter preservation, the full workspace
gate, and the 3.6.0 source/docs/lock bump.
