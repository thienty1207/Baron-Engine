# Baron Multi-Agent Core Parity Plan

This maintenance plan repairs the adapter boundary so every supported agent
receives the same Baron core. It is shipped in the `4.2.2` patch release and
does not create a `4.3` release.

- [x] Phase 113: shared core inventory contract
  - [x] Centralize the adapter payload rule around the embedded `assets/core`
    skill and agent inventory.
  - [x] Keep adapter-specific bridge paths and native hook formats separate
    from the shared core source.

- [x] Phase 114: full Reasonix core materialization
  - [x] Install all bundled skills and agents into Baron-managed Reasonix
    paths, with root, skill, and agent indexes.
  - [x] Extend Reasonix startup/context/status guidance to route the same
    workflow, quality gates, and evidence rules as Codex.
  - [x] Preserve existing Reasonix bridge, settings, and user files.

- [x] Phase 115: safe reconciliation and switching
  - [x] Add Reasonix core assets to managed payloads and baseline/update
    reconciliation.
  - [x] Restore missing Baron assets, but preserve changed/unmarked assets and
    report conflicts.
  - [x] Keep Codex, Claude, generic, and Reasonix files available during
    adapter switching with one project ID and one Vault.

- [x] Phase 116: parity and regression proof
  - [x] Add tests for identical embedded inventories, indexes, and mandatory
    quality agents across Codex and Reasonix.
  - [x] Add switch round-trip, custom-file preservation, conflict, missing
    asset, and shared-history tests using an isolated fixture shaped like the
    consumer project.
  - [x] Run the existing adapter, core, CLI, memory, fallback, and release
    suites without weakening any gate. The two installer archive cases, one
    update-recovery candidate case, and one WDAC-blocked test executable remain
    environment-only failures; focused parity, public-doc, Clippy, release
    build, and release-binary smoke all passed.

- [x] Phase 117: durable documentation and handoff
  - [x] Update adapter architecture, README command guidance, status Markdown,
    status JSON, and build log with the completed parity evidence.
  - [x] Mark every completed task `[x]` and record exact commands/results.
  - [x] Keep the parity implementation behavior unchanged while packaging it
    in the separately authorized `4.2.2` release.

## Verification record

- `cargo fmt --all`: passed.
- `cargo test -p baron-adapters --all-targets --no-fail-fast`: 3 unit,
  30 lifecycle, 15 planner, and 1 transaction test passed.
- `cargo test -p baron-cli --test reasonix_adapter_cli --no-fail-fast`: 6/6
  passed, including release-binary-style init/switch/shortcut asset checks and
  the full Codex -> Reasonix -> Claude -> Generic -> Codex round trip.
- `cargo test -p baron-core --test public_trust_docs --no-fail-fast`: 9/9
  passed after the README/status truth update.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo build --release --locked -p baron-cli`: passed; the parity release
  binary reports `baron 4.2.2` after the version bump.
- Release binary isolated smoke: Codex init -> `baron --reasonix` created the
  complete Reasonix core view and preserved one project/Vault identity.
- Full workspace sweep: all relevant engine, memory, adapter, CLI, fallback,
  and release tests passed; four existing Windows environment gates could not
  execute because PowerShell archive autoload/WDAC policy blocked them.

## Stop conditions

- Any user-owned or changed managed file is preserved and reported.
- Any project/Vault identity mismatch stops the switch before writes.
- A parity test that only checks file presence without checking content,
  inventory, or runtime routing is insufficient.
- A failing existing engine, memory, fallback, or release gate blocks completion.
