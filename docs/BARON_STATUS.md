# Baron Build Status

Last updated: 2026-06-19

## Overall

- Stable source release: `v3.1.0`
- Baron 2.0 completion: 100%
- Target source release: `v3.1.0`
- Baron 3.0 completion: 100%
- Remaining planned phases: 0
- Current phase: Phase 24 - Public Trust Release
- Current phase status: completed
- Current next action: Baron `v3.1.0` is published and verified; continue future hardening only after new requirements are explicit.
- Build confidence: Baron 3.1.0 keeps the Baron 3 engine intact and adds public-trust packaging: concise README, public demo, honest repository-harness comparison, certification snapshot, release/latest instructions, and synchronized status/version metadata.

## Baron 3.0 Direction

Baron 3.0 has two non-negotiable goals:

- Fix weak runtime assets. Baron skills and agents must be self-contained, local, tested, and strong enough to guide AI without relying on external GitHub links.
- Learn from `nousresearch/hermes-agent` without cloning it. Baron should adopt the useful ideas: skill lifecycle, safe self-improvement, session replay, background review, capability/runtime awareness, and continuity autopilot.

Core remains unchanged:

- Superpowers remains the workflow core.
- The three mandatory quality gates remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains Baron's durable source of truth.
- Optional skills and optional agents remain lazy-routed, never core.

## Why The Roadmap Is Complete

The `v1.0.0` release was the working foundation. Baron `v2.0.0` completes the
long-horizon program by adding:

- observable automation and stable project identity
- massive shared-Vault memory indexing and semantic recall
- strict skill/agent control-plane routing
- self-improving Product Harness audits
- certification and release hardening gates

## What Baron 2.0 100% Means

Baron 2.0 is considered 100% complete only when it safely supports both new and
old repositories across multiple agent tools without requiring a dedicated
Baron launcher or forcing the user to run workflow commands manually during
normal AI work.

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
- project identity remains isolated when repositories share the same name
- memory indexing has no silent fixed file-count truncation
- task-aware recall understands close meanings and common Vietnamese/English terms
- supported Codex/Claude sessions are imported, redacted, and deduplicated automatically
- custom skill/agent files and their routing registrations survive updates
- skill/agent selection is contract-based, explainable, and evidence-backed
- Baron detects missed automation instead of assuming instructions were followed
- harness drift, context gaps, and repeated friction produce measured improvement proposals
- extreme-scale tests pass for large repositories, shared Vaults, interruption, corruption, moves, and renames

## Phase Table

| Phase | Name | Status | Earned / Weight | Proof |
| --- | --- | --- | --- | --- |
| 0 | Foundation Skeleton | completed | 3% / 3% | `cargo test`, help smoke, initial commit |
| 1 | Survey Engine | completed | 12% / 12% | `cargo test`, survey smoke, JSON smoke, shadow init smoke |
| 2 | Vault + Memory Firewall | completed | 15% / 15% | `cargo test`, memory CLI tests, multi-project firewall tests, smoke commands |
| 3 | Context Compiler | completed | 15% / 15% | `cargo test`, context core/CLI tests, adapter/risk/why smoke |
| 4 | Agent Adapters | completed | 15% / 15% | adapter lifecycle, preservation, multi-adapter and nested-path tests |
| 5 | Plan/Harness/Proof/Trace | completed | 20% / 20% | plan, harness, proof, trace, completion-gate and mirror tests |
| 6 | Native Migration And Legacy Retirement | completed | 8% / 8% | transactional migration tests, rollback tests, core asset contracts, full suite, manual smoke |
| 7 | Baron Capability Registry | completed | 7% / 7% | provider-kind, compatibility, degradation, context, proof/trace, CLI, and adapter automation tests |
| 8 | Release Hardening | completed | 5% / 5% | local release gates, four-platform CI, tagged release workflow, and published installer lifecycle pass |

The table above records the completed `v1.0.0` program. Baron 2.0 has its own
remaining-program weights:

| Phase | Name | Status | Baron 2.0 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 9 | Automation Runtime And Project Identity | completed | 20% | collision, lifecycle, reconciliation, native-hook, and custom-routing preservation tests |
| 10 | Massive Memory And Semantic Recall | completed | 25% | incremental 350-source index, 6,000-file survey, multilingual recall, session import, and bounded-context tests |
| 11 | Skill And Agent Control Plane | completed | 20% | contract validation, conflict detection, explainable routing, quality-gate execution evidence |
| 12 | Self-Improving Harness | completed | 15% | context score, drift audit, interventions, verify-all, proposal, and outcome-loop tests |
| 13 | Extreme Scale Certification | completed | 15% | certification core/CLI tests, cache-corruption recovery, shared-Vault firewall, context-budget, and smoke certification |
| 14 | Baron 2.0 Release Hardening | completed | 5% | version `2.0.0`, release metadata tests, installer lifecycle tests, full suite, Clippy, release metadata smoke |
| 15 | Simple User Flow | completed | additive | user-facing setup/init/platform flow, hidden automation command help, README simplification, targeted setup/init/context tests |

Baron 2.2 planned additive program:

| Phase | Name | Status | Baron 2.2 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 16 | Agent Skills Refinement | completed | 60% | refined 3 core agents, upgraded optional frontend/security skills, optional performance/API/observability/migration routing, contract tests, adapter smoke |
| 17 | Continuity Ledger And Resume Discipline | completed | 40% | explicit resume ledger, interruption-safe current work packet, generated adapter rules, lifecycle tests, context recovery smoke |

Baron 3.0 planned program:

| Phase | Name | Status | Baron 3.0 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 18 | Asset Sovereignty And Skill/Agent Hardening | completed | 25% | asset sovereignty tests, rewritten self-contained skills, deepened agents, runtime-link scan, adapter lifecycle tests |
| 19 | Skill Lifecycle And Approval Engine | completed | 20% | asset audit, custom quarantine, staged skill proposal metadata, hidden CLI help, lifecycle tests |
| 20 | Session Replay And Conversation Search | completed | 20% | SQLite session replay index, current-project search, bounded replay, context integration, shared-Vault isolation tests |
| 21 | Background Learning And Continuity Autopilot | completed | 15% | autopilot core/CLI tests, context integration, candidate approval/rejection tests, and observed-automation resume tests |
| 22 | Capability Runtime And Safe Tool Backends | completed | 10% | runtime policy core/CLI tests, safe/unsafe/missing provider tests, context integration, and proof-evidence persistence tests |
| 23 | Baron 3.0 Release Certification | completed | 10% | release version tests, certification gates for autopilot/runtime policy, docs/status sync, and full verification batch |
| 24 | Public Trust Release | completed | additive | concise README, public demo, repository-harness comparison, certification snapshot, release/latest docs, source verification |

Phase 16-17 final verification:

- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Temp repo smoke for setup, init, optional skill routing, optional web performance agent routing, continuity checkpoint/status, and context resume: passed

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

- [x] Dry-run inventories legacy project and Vault assets without writing.
- [x] Baron creates a rollback backup inside the Vault migration artifacts.
- [x] Useful memory, plans, harness records, proofs, and traces are converted into Baron-native structures.
- [x] Custom skills and agents pass Baron contract validation before activation.
- [x] Invalid, weak, or conflicting custom assets are quarantined and reported.
- [x] Baron regenerates its own core skills, core agents, adapters, config, and indexes.
- [x] Imported record counts and content hashes are verified before cleanup.
- [x] Agent Bootstrap managed files and runtime are removed only after Baron verification passes.
- [x] Rollback restores the pre-migration project when verification fails.
- [x] Migration leaves no runtime dependency on Agent Bootstrap.
- [x] Migration smoke tests pass against representative old projects.

### Phase 7 - Baron Capability Registry

- [x] Baron registers tools by capability instead of hard-coded tool name.
- [x] Registry supports CLI, binary, MCP, skill, HTTP service, and agent adapter providers.
- [x] Presence checks report `present`, `missing`, or `unknown` with checked time and evidence.
- [x] Baron knows whether the active Codex, Claude, or generic adapter can use each provider.
- [x] Missing optional capabilities degrade cleanly instead of breaking normal work.
- [x] Missing registered capabilities lower Proof/Trace confidence and appear in diagnostics.
- [x] Context includes only a bounded capability summary.
- [x] AI cannot claim a tool-backed check ran unless execution evidence exists.
- [x] Registry, compatibility, fallback, and false-claim regression tests pass.

### Phase 8 - Release Hardening

- [x] Windows x64 binary release works.
- [x] Intel and Apple Silicon macOS binary releases work.
- [x] Linux x64 binary release works.
- [x] Checksums are generated.
- [x] PowerShell and shell installers verify checksums before installation.
- [x] Install, update, rollback, and uninstall docs exist.
- [x] Fresh project smoke test passes.
- [x] Old project smoke test passes.
- [x] Very large repository smoke test passes.
- [x] Shared Vault multi-project isolation smoke test passes.
- [x] Multi-agent adapter smoke test passes.
- [x] Capability Registry degradation smoke test passes.
- [x] GitHub release assets and version metadata are reproducible.

### Phase 9 - Automation Runtime And Project Identity

- [x] Replace basename-only project identity with a stable collision-resistant ID.
- [x] Migrate existing Vault capsules without memory loss or cross-project merging.
- [x] Add IDE-compatible lifecycle events without requiring `baron run`.
- [x] Use native hooks where supported and observable reconciliation where hooks are absent.
- [x] Record which automatic actions actually ran instead of trusting instructions.
- [x] Preserve custom skill/agent routing registrations during every adapter update.
- [x] Add duplicate-name, moved-project, renamed-project, and missed-lifecycle regression tests.

### Phase 10 - Massive Memory And Semantic Recall

- [x] Remove fixed project-memory and repository-entry truncation.
- [x] Add deterministic incremental indexing with deletion and rename handling.
- [x] Add recency, evidence, confidence, status, kind, project, and source metadata.
- [x] Add hybrid lexical and concept-semantic retrieval without a mandatory model.
- [x] Support common Vietnamese/English engineering meaning matches.
- [x] Import supported Codex/Claude sessions automatically with redaction and deduplication.
- [x] Compile task-aware bounded context from very large Vaults.
- [x] Rebuild all disposable indexes from Markdown without memory loss.

### Phase 11 - Skill And Agent Control Plane

- [x] Define validated contracts for skill/agent triggers, exclusions, ownership, conflicts, dependencies, inputs, outputs, and evidence.
- [x] Keep Superpowers as the workflow core and the three core quality agents as mandatory risk-aware gates.
- [x] Route optional assets narrowly without recursive loading.
- [x] Explain why each skill/agent was selected or skipped.
- [x] Detect duplicate ownership, conflicting instructions, weak contracts, and recursive orchestration.
- [x] Preserve custom files and custom routing through init, update, migration, and adapter changes.
- [x] Require execution evidence before a mandatory agent gate counts as passed.

### Phase 12 - Self-Improving Harness

- [x] Score whether required context was actually read.
- [x] Audit documentation drift, contradictions, stale rules, and harness entropy.
- [x] Record human, reviewer, CI, and agent interventions.
- [x] Verify open stories and proof gaps in bounded batches.
- [x] Group repeated friction and generate evidence-backed improvement proposals.
- [x] Track predicted impact against actual outcomes.
- [x] Require human approval before core policy or architecture is rewritten.

### Phase 13 - Extreme Scale Certification

- [x] Certify repositories from small fixtures through large repo fixtures.
- [x] Certify shared Vault isolation without contamination.
- [x] Certify large memory histories through existing no-fixed-limit and certification tests.
- [x] Test cache corruption recovery, deletion, move, rename, and duplicate-name contracts through Phase 9-13 tests.
- [x] Establish context-size and index-rebuild budgets in certification.
- [x] Keep cross-platform proof in release workflow contracts; publishing native assets remains an operator action.

### Phase 14 - Baron 2.0 Release Hardening

- [x] Provide verified Agent Bootstrap/native migration and rollback; `v1.0.0` to `v2.0.0` source continuity is covered by the same release/version and lifecycle tests.
- [x] Update adapter, Vault, CLI, README, architecture, command-surface, release, status, and audit documentation.
- [x] Keep deterministic native release asset and checksum contracts, now targeting `2.0.0`.
- [x] Pass the complete Phase 9-13 local acceptance suite, Clippy, and release smoke checks.
- [x] Mark `v2.0.0` source ready with no open identity, memory, automation, routing, or evidence blocker; native GitHub release publishing remains manual.

### Phase 15 - Simple User Flow

- [x] Add `baron setup --vault`, defaulting to the current folder when no Vault path is passed.
- [x] Let `baron init --codex`, `baron init --claude`, and `baron init --agent` use the machine default Vault after setup.
- [x] Add platform focus flags such as `--frontend`, `--backend`, `--fullstack`, `--mobile`, `--desktop`, `--tool`, `--library`, `--data`, and `--cloud`.
- [x] Support shortcut init such as `baron init --codex --fullstack`.
- [x] Keep top-level help and README focused on normal user commands while keeping advanced commands available for AI automation and diagnostics.
- [x] Surface platform focus in generated context and adapter startup guidance.

### Phase 16 - Agent Skills Refinement

- [x] Upgrade the 3 Baron core agents using strong external rubric ideas without copying another repo as Baron's architecture.
- [x] Keep `code-reviewer`, `security-auditor`, and `test-engineer` as the only core quality agents.
- [x] Improve `frontend-design` and `vibe-security-scan` instead of adding duplicate frontend/security skills.
- [x] Add only narrow optional skills or agents when Baron lacks that domain, such as web performance, API/interface design, observability, performance optimization, or migration/deprecation guidance.
- [x] Update Baron control-plane routing so optional assets auto-trigger only for matching tasks and never replace Superpowers.
- [x] Add contract tests proving no recursive loading, no duplicate ownership, no unsafe security instructions, and no optional asset is treated as core.
- [x] Run full workspace tests, Clippy, adapter smoke, and context/routing smoke before marking complete.

### Phase 17 - Continuity Ledger And Resume Discipline

- [x] Productize the existing build-log, active plan, trace, and automation journal behavior into one explicit Baron resume contract.
- [x] Ensure every meaningful feature implementation writes a current-work checkpoint before edits, after direction changes, before interruption, and before completion.
- [x] Add or tighten adapter startup guidance so Codex, Claude, and generic agents read the resume packet before continuing interrupted work.
- [x] Make context output show the current resume point without dumping noisy history.
- [x] Add tests that simulate a stopped session and verify the next agent can identify current task, last completed step, open risks, proof status, and next action.
- [x] Keep the feature automatic for AI; normal users should not need to run extra commands during ordinary work.

### Phase 18 - Asset Sovereignty And Skill/Agent Hardening

- [x] Remove live GitHub or external runtime dependencies from managed optional `SKILL.md` and agent instruction files.
- [x] Move attribution and license references into `NOTICE.md` or `LICENSE.txt` files where needed.
- [x] Rewrite `vibe-security-scan`, `api-and-interface-design`, `observability-and-instrumentation`, and `deprecation-and-migration` as self-contained Baron-native skills.
- [x] Deepen `code-reviewer`, `security-auditor`, `test-engineer`, and `web-performance-auditor` with scope, evidence, proof, trace, anti-hallucination, and output contracts.
- [x] Add tests that fail on thin skills, runtime external links, duplicate workflow ownership, recursive subagent orchestration, and missing proof/trace contracts.
- [x] Run full tests, Clippy, runtime asset scan, JSON parse, diff check, and temp repo smoke.

### Phase 19 - Skill Lifecycle And Approval Engine

- [x] Score skill quality before routing or activation through `baron asset audit`.
- [x] Quarantine weak, conflicting, externally dependent, or duplicate-ownership custom skills.
- [x] Stage agent-proposed skill edits as diffs instead of silently overwriting runtime assets.
- [x] Add approval metadata for skill updates and safe self-improvement proposals.
- [x] Preserve managed Superpowers while quarantining only failing custom assets.
- [x] Test rejection, quarantine, staged approval, and hidden CLI availability paths.

### Phase 20 - Session Replay And Conversation Search

- [x] Store imported agent session Markdown in a local searchable SQLite session replay cache.
- [x] Add exact message search and bounded surrounding-context replay.
- [x] Support current-project search without dumping full histories.
- [x] Link replay hits to project identity, Vault capsule, source path, role, ordinal, and content hash.
- [x] Enforce Memory Firewall rules so weak cross-project session hits cannot pollute the active project.
- [x] Test search, replay, bounded context, and shared-Vault isolation.

### Phase 21 - Background Learning And Continuity Autopilot

- [x] Add post-task review that proposes memory, skill, harness, and continuity improvements.
- [x] Keep uncertain learning as candidates, not trusted facts.
- [x] Require approval gates for sensitive or runtime-affecting writes when configured.
- [x] Resume interrupted work from continuity packet, session replay, plan, harness, proof, and trace state.
- [x] Record which automation actually ran instead of assuming the agent followed instructions.
- [x] Test interruption, candidate learning, approval, rejection, and resume behavior.

### Phase 22 - Capability Runtime And Safe Tool Backends

- [x] Distinguish tool availability from executed proof.
- [x] Track provider backend, adapter support, sandbox policy, and execution evidence.
- [x] Lower confidence when required tools are missing, unsafe, or unverified.
- [x] Recommend safe backend choices without forcing users into one IDE or one agent app.
- [x] Keep completion blocked when tool-backed proof is claimed without evidence.
- [x] Test missing tools, unsafe backends, optional degradation, and false-claim regressions.

### Phase 23 - Baron 3.0 Release Certification

- [x] Run full `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Smoke Codex, Claude, and generic adapter init/update flows.
- [x] Smoke old-repo migration and shared Vault stress cases.
- [x] Verify README, status JSON, build logs, command surface, and release metadata are synchronized.
- [x] Mark Baron 3.0 ready only after all Phase 18-22 proof is complete.

### Phase 24 - Public Trust Release

- [x] Keep Baron 3 engine behavior unchanged.
- [x] Rewrite README as a concise public landing page.
- [x] Add a public 10-year repo demo for Codex, Claude, and generic agents.
- [x] Add an honest Baron vs repository-harness comparison.
- [x] Add a public certification snapshot with concrete verification commands.
- [x] Update release docs so `releases/latest` and tag publication are explicit.
- [x] Bump source release metadata to `3.1.0`.

## Current Working Files

- Product spec: `docs/specs/2026-06-08-baron-product-spec-1.0.md`
- Roadmap: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Architecture: `docs/architecture/ARCHITECTURE.md`
- Memory model: `docs/architecture/MEMORY_MODEL.md`
- Context compiler: `docs/architecture/CONTEXT_COMPILER.md`
- Adapter model: `docs/architecture/ADAPTERS.md`
- Capability registry: `docs/architecture/CAPABILITY_REGISTRY.md`
- Phase 4-5 design: `docs/superpowers/specs/2026-06-14-agent-adapters-execution-engine-design.md`
- Phase 6-8 roadmap decision log: `notes/build-log/2026-06-15-phase-6-8-roadmap.md`
- Phase 6 design: `docs/superpowers/specs/2026-06-15-native-migration-legacy-retirement-design.md`
- Phase 6 plan: `docs/superpowers/plans/2026-06-15-phase-6-native-migration.md`
- Phase 6 build log: `notes/build-log/2026-06-15-phase-6-native-migration.md`
- Phase 7 design: `docs/superpowers/specs/2026-06-15-baron-capability-registry-design.md`
- Phase 7 plan: `docs/superpowers/plans/2026-06-15-phase-7-capability-registry.md`
- Phase 7 build log: `notes/build-log/2026-06-15-phase-7-capability-registry.md`
- Phase 8 design: `docs/superpowers/specs/2026-06-15-release-hardening-design.md`
- Phase 8 plan: `docs/superpowers/plans/2026-06-15-phase-8-release-hardening.md`
- Phase 8 build log: `notes/build-log/2026-06-15-phase-8-release-hardening.md`
- Baron 2.0 program design: `docs/superpowers/specs/2026-06-15-baron-2-program-design.md`
- Baron 2.0 roadmap decision log: `notes/build-log/2026-06-15-baron-2-roadmap.md`
- Release guide: `docs/RELEASE.md`
- Previous published release: `https://github.com/thienty1207/Baron-Engine/releases/tag/v1.0.0`
- Baron 2.0 final audit: `docs/assessment/2026-06-16-baron-2-final-audit.md`
- Phase 13-14 plan: `docs/superpowers/plans/2026-06-16-phase-13-14-certification-release.md`
- Phase 13-14 build log: `notes/build-log/2026-06-16-phase-13-14-certification-release.md`
- Phase 16-17 plan: `docs/superpowers/plans/2026-06-18-phase-16-17-agent-skills-continuity.md`
- Baron 2.2 planning log: `notes/build-log/2026-06-18-baron-2-2-agent-skills-roadmap.md`
- Baron 3.0 roadmap log: `notes/build-log/2026-06-19-baron-3-roadmap.md`
- Phase 18-20 plan: `docs/superpowers/plans/2026-06-19-phase-18-20-baron-3-foundation.md`
- Phase 18-20 build log: `notes/build-log/2026-06-19-phase-18-20-baron-3-foundation.md`
- Phase 21-23 plan: `docs/superpowers/plans/2026-06-19-phase-21-23-baron-3-release.md`
- Phase 21-23 build log: `notes/build-log/2026-06-19-phase-21-23-baron-3-release.md`
- Phase 24 plan: `docs/superpowers/plans/2026-06-19-phase-24-public-trust-release.md`
- Phase 24 build log: `notes/build-log/2026-06-19-phase-24-public-trust-release.md`
- Public demo: `docs/demo/README.md`
- Public comparison: `docs/assessment/baron-vs-repository-harness.md`
- Public certification: `docs/assessment/baron-3-public-certification.md`
- Temporary build note: `notes/build-log/CURRENT.md`

## Current Rule

Baron `3.1.0` is the current stable source release. Phase 24 is a public-trust
packaging release on top of the Baron 3 engine. It must preserve Superpowers as
the workflow core, keep the three mandatory quality gates stable, keep managed
runtime skills and agents self-contained Baron assets, keep autopilot learning
candidate-gated, and keep runtime backend claims blocked without safe providers
plus execution evidence.

Phase 18-20 final verification:

- `cargo fmt --all`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Runtime optional skill/agent live-link scan: passed
- Temp repo smoke for setup, init, asset audit, session replay index/search, and task context replay: passed
- `docs/BARON_STATUS.json` parse: passed
- `git diff --check`: passed

Phase 21-23 final verification:

- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Temp repo smoke for setup, Codex/Claude/generic init, shared Vault memory index, runtime check, context, autopilot review/status, recall, certification, and Agent Bootstrap migration dry-run: passed
- `docs/BARON_STATUS.json` parse: passed
- Static stale-release scan: passed
- `git diff --check`: passed

Public Trust 3.1.0 final verification:

- README public flow validation: passed by `cargo test -p baron-core --test public_trust_docs`
- demo, comparison, and certification docs validation: passed by `cargo test -p baron-core --test public_trust_docs`
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- `git diff --check`: passed
- GitHub release latest smoke: passed; `releases/latest` points to `v3.1.0`, release workflow `27839412902` passed, and Windows install/setup/init/context smoke from latest passed
