# Phase 43 - Graphify Local Code-Only Adapter

Date: 2026-07-24
Release target: Baron `3.6.0`

## Completed

- Added Baron-owned `GraphifyProvider` support for only the pinned local
  provider version `0.9.25`.
- Allowed exactly three provider surfaces: version probe, code-only extraction
  into the current project's cache, and a bounded local JSON query.
- Added process timeouts, stdout/stderr size caps, disabled provider-side query
  logging, and no-secret diagnostics.
- Added staged graph validation, checksum/state replacement, and restoration of
  the last known-good state if refresh fails.
- Kept the graph cache inside the current repository and outside Vault memory.

## Evidence

```text
cargo test -p baron-core --test graphify_provider
cargo test -p baron-core --test code_graph --test capability
```

All passed. The deterministic PowerShell fixture proves the exact command
allowlist, pinned-version refusal, missing-provider behavior, timeout,
non-zero, malformed JSON, oversized output, oversized graph, and last-known
good state preservation. It never invokes a real provider or network.

## Resume Point

Proceed to Phase 44 only. Keep normal user commands unchanged. Context must
route code-map use only for tasks that need code navigation, never block startup
on refresh, and require current source verification before graph hints support
proof or memory.
