# Baron Build Status

Last updated: 2026-06-15

## Overall

- Completion: 80%
- Current phase: Phase 6 - Native Migration And Legacy Retirement
- Current phase status: design_pending
- Current next action: design and implement Baron-native migration without carrying Agent Bootstrap runtime or architecture forward
- Build confidence: full Rust tests, Clippy, format, JSON validation, three-adapter preservation smoke, and high-risk completion smoke are verified

## What 100% Means

Baron is considered 100% complete only when it can safely support both new and
old repositories across multiple agent tools without forcing the user to run
manual workflow commands during normal AI work.

Completion requires:

- repo survey and shadow mode work on old repositories
- Vault Markdown source of truth is working
- SQLite/cache acceleration is working and rebuildable
- memory firewall prevents shared-vault cross-project noise
- context compiler produces bounded, task-relevant context
- Codex, Claude, and generic agent adapters are real
- Superpowers remains the workflow core
- 3 core quality agents are shipped and routed
- optional frontend/security skills are shipped and lazy-routed
- active plan state works
- Product Harness works
- proof requirements are risk-aware
- trace quality scoring works
- legacy project migration imports only useful data into Baron-native structures
- invalid or conflicting legacy skills and agents are quarantined instead of activated
- Baron knows which project tools and capabilities are really available
- missing tools reduce proof confidence instead of causing false completion claims
- release binaries and smoke tests pass on Windows, macOS, and Linux

## Phase Table

| Phase | Name | Status | Earned / Weight | Proof |
| --- | --- | --- | --- | --- |
| 0 | Foundation Skeleton | completed | 3% / 3% | `cargo test`, help smoke, initial commit |
| 1 | Survey Engine | completed | 12% / 12% | `cargo test`, survey smoke, JSON smoke, shadow init smoke |
| 2 | Vault + Memory Firewall | completed | 15% / 15% | `cargo test`, memory CLI tests, multi-project firewall tests, smoke commands |
| 3 | Context Compiler | completed | 15% / 15% | `cargo test`, context core/CLI tests, adapter/risk/why smoke |
| 4 | Agent Adapters | completed | 15% / 15% | adapter lifecycle, preservation, multi-adapter and nested-path tests |
| 5 | Plan/Harness/Proof/Trace | completed | 20% / 20% | plan, harness, proof, trace, completion-gate and mirror tests |
| 6 | Native Migration And Legacy Retirement | design_pending | 0% / 8% | none |
| 7 | Baron Capability Registry | not_started | 0% / 7% | none |
| 8 | Release Hardening | not_started | 0% / 5% | none |

## Completion Checklist

### Phase 0 - Foundation Skeleton

- [x] Rust workspace exists.
- [x] Product spec exists.
- [x] Roadmap exists.
- [x] Architecture docs exist.
- [x] Temporary build notes exist.
- [x] Core asset blueprints exist.
- [x] Adapter blueprints exist.
- [x] `cargo test` passes.
- [x] `cargo run -p baron-cli -- --help` works.
- [x] Phase 0 committed.

### Phase 1 - Survey Engine

- [x] `baron survey` reads repo without modifying files.
- [x] `baron survey --json` outputs machine-readable survey.
- [x] Project Atlas Markdown is generated to stdout.
- [x] Project Atlas JSON is generated to stdout.
- [x] Stack, entrypoint, build, test, and risky surfaces are detected.
- [x] Shadow mode init does not overwrite project files.
- [x] Old repo smoke test passes against the Baron repo itself.

### Phase 2 - Vault + Memory Firewall

- [x] Vault scaffold exists.
- [x] Project capsule exists.
- [x] SQLite/cache index can be rebuilt from Markdown.
- [x] Current project memory is prioritized.
- [x] Verified global memory is allowed only when relevant.
- [x] Cross-project memory is blocked unless explicitly matched.
- [x] Stale and unknown memory are marked correctly.
- [x] Multi-project vault smoke test passes.

### Phase 3 - Context Compiler

- [x] `baron context --codex` works.
- [x] `baron context --claude` works.
- [x] `baron context --agent` works.
- [x] `baron context --why` explains loaded/skipped context.
- [x] Context output stays bounded.
- [x] Context changes by task, risk, phase, and adapter.
- [x] Context compiler smoke test passes.

### Phase 4 - Agent Adapters

- [x] Codex adapter generates `AGENTS.md` and `.codex/`.
- [x] Claude adapter generates `CLAUDE.md`.
- [x] Generic adapter generates portable agent files.
- [x] Adapters preserve user-written content.
- [x] Adapters refresh managed blocks safely.
- [x] Adapter update smoke tests pass.

### Phase 5 - Plan/Harness/Proof/Trace

- [x] Active plan state works.
- [x] Product Harness intake works.
- [x] Risk flags and lanes work.
- [x] Proof requirements are risk-aware.
- [x] Validation matrix links stories to proof evidence.
- [x] Weak proof remains insufficient and Baron state cannot fake product-file evidence.
- [x] Trace recording works.
- [x] Trace scoring works.
- [x] Friction backlog works.
- [x] High-risk completion without proof is blocked.

### Phase 6 - Native Migration And Legacy Retirement

- [ ] Dry-run inventories legacy project and Vault assets without writing.
- [ ] Baron creates a rollback backup inside the Vault migration artifacts.
- [ ] Useful memory, plans, harness records, proofs, and traces are converted into Baron-native structures.
- [ ] Custom skills and agents pass Baron contract validation before activation.
- [ ] Invalid, weak, or conflicting custom assets are quarantined and reported.
- [ ] Baron regenerates its own core skills, core agents, adapters, config, and indexes.
- [ ] Imported record counts and content hashes are verified before cleanup.
- [ ] Agent Bootstrap managed files and runtime are removed only after Baron verification passes.
- [ ] Rollback restores the pre-migration project when verification fails.
- [ ] Migration leaves no runtime dependency on Agent Bootstrap.
- [ ] Migration smoke tests pass against representative old projects.

### Phase 7 - Baron Capability Registry

- [ ] Baron registers tools by capability instead of hard-coded tool name.
- [ ] Registry supports CLI, binary, MCP, skill, HTTP service, and agent adapter providers.
- [ ] Presence checks report `present`, `missing`, or `unknown` with checked time and evidence.
- [ ] Baron knows whether the active Codex, Claude, or generic adapter can use each provider.
- [ ] Missing optional capabilities degrade cleanly instead of breaking normal work.
- [ ] Missing registered capabilities lower Proof/Trace confidence and appear in diagnostics.
- [ ] Context includes only a bounded capability summary.
- [ ] AI cannot claim a tool-backed check ran unless execution evidence exists.
- [ ] Registry, compatibility, fallback, and false-claim regression tests pass.

### Phase 8 - Release Hardening

- [ ] Windows binary release works.
- [ ] macOS binary release works.
- [ ] Linux binary release works.
- [ ] Checksums are generated.
- [ ] PowerShell and shell installers verify checksums before installation.
- [ ] Install, update, rollback, and uninstall docs exist.
- [ ] Fresh project smoke test passes.
- [ ] Old project smoke test passes.
- [ ] Very large repository smoke test passes.
- [ ] Shared Vault multi-project isolation smoke test passes.
- [ ] Multi-agent adapter smoke test passes.
- [ ] Capability Registry degradation smoke test passes.
- [ ] GitHub release assets and version metadata are reproducible.

## Current Working Files

- Product spec: `docs/specs/2026-06-08-baron-product-spec-1.0.md`
- Roadmap: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Memory model: `docs/architecture/MEMORY_MODEL.md`
- Context compiler: `docs/architecture/CONTEXT_COMPILER.md`
- Adapter model: `docs/architecture/ADAPTERS.md`
- Phase 4-5 design: `docs/superpowers/specs/2026-06-14-agent-adapters-execution-engine-design.md`
- Phase 6-8 roadmap decision log: `notes/build-log/2026-06-15-phase-6-8-roadmap.md`
- Temporary build note: `notes/build-log/CURRENT.md`

## Current Rule

Do not call Baron complete until every phase checklist is finished and verified.
If a session is interrupted, read this file and `notes/build-log/CURRENT.md`
before continuing.
