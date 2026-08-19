# Baron Reasonix Adapter Program

This is a seven-phase maintenance track. It adds one adapter to the verified
Baron `4.2.0` source; it does not create Baron `4.3` or change the intelligence
engine.

- [x] Phase 101: shared-brain adapter identity
- [x] Add `Reasonix` to core, CLI, config, capability, context, and automation
    mappings.
- [x] Add optional `active_adapter` with backward-compatible fallback to the
    first registered adapter.
- [x] Keep project ID, Vault routing, memory, Wiki, CodeGraph, plan, proof,
    trace, and continuity adapter-neutral.

- [x] Phase 102: native Reasonix assets
- [x] Add `REASONIX.md` and `.reasonix/commands` managed payloads.
- [x] Route startup/context/status commands through `--adapter reasonix` and
    `--reasonix`.
- [x] Preserve existing Codex/Claude/generic assets and custom content.

- [x] Phase 103: non-destructive install and update
- [x] Make Reasonix installation preserve unmarked user files and report
    conflicts instead of overwriting them.
- [x] Record managed baselines for safe future updates.
- [x] Cover existing user `REASONIX.md`, settings, hooks, and bridge scripts.

- [x] Phase 104: switch/rollback lifecycle
- [x] Add `baron adapter status` and `baron adapter switch --to <adapter>`.
- [x] Checkpoint before switching, update only active adapter, and emit shared
    adapter provenance in the common journal/Vault path.
- [x] Prove Codex -> Reasonix -> Codex continuity with one shared brain.

- [x] Phase 105: tests and real-project compatibility
- [x] Add core/config/context/automation/CLI/adapter tests for Reasonix.
- [x] Add conflict, preservation, malformed settings, shared Vault, rollback,
    and no-version-bump tests.
- [x] Run a read-only fixture modelled on
    `D:\Work\Project\Website\GoGetSomeFoodFerris`; do not modify that user
    project during the Baron test run.

- [x] Phase 106: documentation and release truth
- [x] Update README install/init/context examples for Reasonix and the shared
    brain/switch workflow.
- [x] Update command surface, adapter architecture, status JSON/Markdown,
    build log, and release notes while keeping `4.2.0` as the source/public
    version.

- [x] Phase 107: final verification and GitHub publication
- [x] Run the formatter, affected adapter/core suites, focused Reasonix CLI
    suite, Clippy, locked release build, CLI help/release smoke, shared-Vault
    switch, preservation, identity-mismatch, and README/status truth checks.
    The two broader Windows installer lifecycle cases remain environment-only
    failures because `Microsoft.PowerShell.Archive` cannot autoload here; they
    do not exercise the adapter change.
- [x] Mark every completed task `[x]`, commit only the intended adapter/docs
    changes, push `agent/baron-4-0` and `main`, and verify the remote commit and
    clean working tree.
- [x] Do not create a `4.3.0` tag or Release; publish the adapter-only change on
    the existing `4.2.0` source line.

- [x] Phase 108: root adapter shortcut UX on the 4.2.0 maintenance line
- [x] Add `baron --reasonix` and `baron --codex` shortcuts that resolve the
    current project, switch the active adapter, and preserve the shared brain.
- [x] Keep `baron adapter status/switch` as the explicit diagnostic/script API
    and verify both shortcut directions without a version bump.

## Stop conditions

- Any project/Vault identity mismatch stops the switch before writes.
- Any unmarked or changed user-owned adapter file is preserved and reported.
- Any regression in the affected adapter/core/CLI surfaces blocks Phase 107.
  The two Windows installer cases documented above are environment-only
  `Microsoft.PowerShell.Archive` autoload failures and are not adapter
  regressions.
