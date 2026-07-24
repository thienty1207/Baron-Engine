# Phase 45 - Isolation, Scale, And Baron 3.6 Certification

Date: 2026-07-24
Release target: Baron `3.6.0`
Status: completed

## Delivered

- Added a same-name project fixture proving each repository has its own graph
  identity, cache root, task-query cache, and source hits even when both share
  one Vault and the same visible folder name.
- Proved code-graph content never becomes Vault Markdown memory or recall data.
  Removing one project cache leaves the other project and all Vault memory
  intact.
- Expanded the old-repository smoke to over 6,100 mixed TypeScript, Rust, Go,
  and Python source files with ignored dependency/build trees. The source
  fingerprint changes for a file after entry 6,000 and context stays bounded.
- Added end-to-end Survey fallback checks for a missing provider and a stale
  prior graph after failed/incompatible/timed-out/malformed/oversized provider
  operations.
- Added mutation snapshots for hooks, agent instruction files, adapter hook
  settings, root graph output, and a controlled home graph directory. Refresh
  and query may change only `.baron/cache/code-graph/`.
- Added the `optional-code-map-boundary` certification gate so release
  certification checks project-local cache placement and visible Survey
  fallback, instead of treating a graph as mandatory capability.

## Decisions

- Graph cache is intentionally excluded from Vault indexing rather than merely
  ranked low. It is derived navigation data, not durable project memory.
- The controlled home-location test uses a cleanup guard so a failed test cannot
  leak a temporary `HOME` or `USERPROFILE` into later Windows tests.
- The large test changes a source beyond 6,000 entries instead of relying on a
  timing-only assertion. This proves coverage without making a speed claim that
  depends on one machine.

## Verification

```text
cargo fmt --all -- --check
cargo test -p baron-core --test graphify_provider --test code_graph_isolation --test certification
cargo test -p baron-cli --test release_smoke
cargo test --workspace --all-targets --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
target\release\baron.exe --version
git diff --check
```

The final source, status, release docs, certification, and lockfile must all
agree on `3.6.0`. No tag or GitHub Release is created by this phase.
