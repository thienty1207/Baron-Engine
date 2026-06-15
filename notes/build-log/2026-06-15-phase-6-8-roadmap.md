# Baron Phase 6-8 Roadmap Decision Log

Date: 2026-06-15
Status: recorded

## Why The Roadmap Changed

Phase 4-5 completed the Baron adapters and execution gates. Before release, two
remaining concerns must be handled explicitly:

1. Existing Agent Bootstrap projects need a one-way, verified migration into
   Baron without carrying the old runtime or architecture forward.
2. Baron needs to know which external tools are really available so agents do
   not claim checks that never ran.

## Phase 6 Decision

Phase 6 is now **Native Migration And Legacy Retirement**.

Baron will inventory legacy data, create a Vault-contained rollback backup,
convert useful records into Baron-native structures, validate custom skills and
agents, quarantine invalid assets, verify counts and hashes, and only then
remove Agent Bootstrap managed files and runtime.

The migration must leave no runtime dependency on Agent Bootstrap.

## Phase 7 Decision

Phase 7 is now **Baron Capability Registry**.

Baron will register tools by capability, detect whether providers are present,
check compatibility with the active agent adapter, degrade safely when optional
tools are missing, and connect capability evidence to Proof and Trace quality.

Supported provider categories are planned as CLI, binary, MCP, skill, HTTP
service, and agent adapter.

## Phase 8 Decision

Release Hardening moves to Phase 8.

The v1.0.0 release gate remains cross-platform binaries, verified checksums,
install/update/rollback flows, and smoke tests for fresh, old, very large, and
shared-Vault multi-project repositories.

## Resume Point

The next session must:

1. Read `docs/BARON_STATUS.md`.
2. Read `notes/build-log/CURRENT.md`.
3. Design Phase 6 before writing implementation code.
4. Keep Agent Bootstrap only as an import source, never as Baron architecture.
5. Update this log or `CURRENT.md` after each meaningful implementation batch.
