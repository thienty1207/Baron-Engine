# Baron Build Status

Last updated: 2026-08-12

## Overall

- Stable source release: `v3.8.0`
- Latest downloadable release: [`v3.8.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v3.8.0)
- Candidate source version: `3.8.0`; public promotion completed
- Baron 2.0 completion: 100%
- Baron 3.0 completion: 100%
- Baron 3.2 completion: 100%
- Baron 3.3 completion: 100%
- Target source release: `v3.8.0`, completed
- Program target release: `v3.8.0`, completed
- Baron 3.4 completion: 100%
- Baron 3.5 completion: 100%
- Baron 3.6 completion: 100%
- Baron 3.7 completion: 100%; Phases 46-52 publicly certified
- Baron 3.8 completion: 100%; Phases 53-64 publicly certified
- Approved or in-progress Baron 3.8 phases: 12; completed phases: 12
- Remaining Baron 3.8 phases: 0
- Current phase: Baron 3.8 public release complete
- Current phase status: completed; source, tag, Release, assets, README, and
  fresh public install smoke agree on `3.8.0`
- Current next action: normal maintenance from the published Baron 3.8 baseline.
- Current Baron 3.8 authority: committed source, status records, release
  evidence, and public `v3.8.0` artifacts are synchronized.
- For the completed Baron 3.7 program, the owner explicitly approved
  implementation and final GitHub publication on 2026-08-12. Its immutable
  release and fresh public `releases/latest` install smoke have both passed.
- Build confidence: Baron 3.8 is public-release certified. GitHub Actions run
  [`31603251123`](https://github.com/thienty1207/Baron-Engine/actions/runs/31603251123)
  passed exact-source verification, the full Ubuntu workspace suite, Clippy,
  Windows x64, Linux x64, macOS Intel, and Apple Silicon native builds,
  checksums, manifest verification, installer lifecycle smoke, immutable tag,
  and Release promotion from source
  `57c5d94a5459e4597bd8a63bb1a6e65f7c197c23`.
  A fresh Windows smoke installed directly from
  `releases/latest`, returned `baron 3.8.0`, passed `setup`,
  `init --codex --fullstack`, and `context`; the same-version update guard
  refused a redundant update without changing the user marker. The optional
  local code map remains identity-bound and project-local, stays outside Vault
  memory, preserves agent instructions and hooks, remains bounded on old/large
  repositories, and falls back to Survey on every absence or failure.

## Baron 3.6 Final Public Release Checklist

- [x] Source, lockfile, tests, README, release guide, status files, and
  certification agree on `3.6.0`.
- [x] README includes the Windows reinstall, Vault restore, project refresh,
  and exact-version check a normal user needs.
- [x] The initial source candidate was pushed and GitHub Actions stopped before
  tagging when Ubuntu found a cross-platform compile error.
- [x] The second attempt also stopped before tagging when the Unix self-update
  test fixture copied an oversized debug binary; the fixture now uses a bounded
  wrapper while the production download limit remains unchanged.
- [x] The third attempt also stopped before tagging when the Unix update
  transaction fixture copied an oversized patched debug binary; it now stages a
  bounded wrapper around the patched backing binary.
- [x] The fourth attempt passed the Ubuntu test suite, then stopped before
  tagging when Clippy found three Windows-only lint owners in Unix compilation;
  the empty Unix test was removed and the two Windows-only runtime owners are
  now explicitly platform-scoped.
- [x] All repair passes have passed the local full test, Clippy, locked
  release-build, YAML, and release-profile Vault/project smoke gates.
- [x] The repaired source candidate has passed local checks and is pushed to
  `origin/main`; immutable promotion will derive its exact SHA from the current
  remote branch rather than copied text.
- [x] GitHub Actions run `30246729740` built all four native targets from exact
  source `c89486694d9a4431e04106274d0c9f997db42683`.
- [x] GitHub created immutable tag `v3.6.0` and Release assets with checksums,
  Windows and Unix installers, plus native archives.
- [x] `releases/latest` downloaded `3.6.0`; a fresh Windows install/setup/init
  and context smoke passed from the public installer.

## Baron 3.7 Planning And Approval Gate

- [x] The owner requested a complete Baron 3.7 phase plan on 2026-08-12.
- [x] The plan incorporates only the useful evidence-first lessons from the
  reviewed upstream harness release, while keeping its source and workflow
  boundary independent from Baron.
- [x] Baron keeps its independent product boundary: Vault memory, rebuildable
  SQLite caches, Product Harness, Plan, Proof, Trace, Continuity, Autopilot,
  Superpowers, and the three core quality agents are not removed.
- [x] The original planning batch changed only `docs/BARON_STATUS.md`; the
  approved implementation batch now changes source, tests, design, plan, and
  build-log evidence while keeping the public README truthful at `3.6.0`.
- [x] The user explicitly approved the Phase 46-52 plan and authorized
  implementation, source/version changes, GitHub push, tag, Release, and public
  install smoke on 2026-08-12.
- [x] Before Phase 46 began, the active Baron 3.7 design, executable plan,
  build log, status JSON update, and continuity checkpoint were created.
- [x] Phase 52 has separate release authority and must carry the work all the
  way to a publicly installable `v3.7.0`; source-ready status alone is not an
  acceptable program outcome.

## Baron 3.8 Planning And Approval Gate

Status: `completed`; the owner approved and the full 12-phase Baron 3.8
implementation and public-release program is complete.

- The owner requested a written plan covering the selected TencentDB Agent
  Memory lessons and a narrow, optional reverse-analysis capability.
- The approved implementation batch updated source, tests, design, plan,
  status JSON, build log, README, architecture, security assets, and release
  evidence. Every Phase 53-64 item below is checked after verification.
- Baron 3.8 is a local coding-memory program for one heavy user. It is not a
  team-memory service, enterprise platform, or UI program.
- The memory reference is TencentDB Agent Memory `v2.0.0`; Baron adopts only
  layered memory, hybrid retrieval, Wiki, and CodeGraph ideas that strengthen
  coding continuity and token efficiency.
- The reverse-analysis reference was reviewed at
  `zhaoxuya520/reverse-skill` commit
  `0816b124358a010eb70ec919cb6c295d946cc9d6`. That repository is evidence and
  source material only; it is not an approved runtime dependency.
- `vibe-security-scan` remains Baron's source-code AppSec owner. It will be
  repaired and benchmarked, not deleted or replaced.
- A future reverse-analysis pack may cover only selected binary, APK/mobile,
  and malware-analysis needs. It must not import the external router, global
  rules, offensive suites, auto-bootstrap behavior, or case lifecycle.
- Superpowers remains the only workflow core. Baron Control Plane remains the
  only router. The mandatory quality gates remain `code-reviewer`,
  `security-auditor`, and `test-engineer`.
- The owner explicitly approved implementation, GitHub publication, README
  synchronization, and the complete Phase 53-64 release scope. The active
  design, executable plan, status JSON, build log, continuity checkpoint, and
  release certification record were updated before closure.
- Phase 64 carried the authorized commit, push, immutable tag, GitHub Release,
  native assets, checksums, installer metadata, and fresh `releases/latest`
  Windows verification. The latest public downloadable release is `v3.8.0`.

## 2026-07-24 Verified Core Refresh And Decisions

- Superpowers remains Baron's only workflow core and is now pinned to upstream
  `v6.2.0` commit `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`.
- The complete 50-file upstream skill subtree is vendored locally with one
  documented offline hardening patch, provenance, MIT license, adapter parity,
  and no runtime download dependency.
- Graphify is accepted only as a future optional project-scoped code-map
  provider. It cannot own Baron memory, instructions, hooks, or cross-project
  context.
- Hallmark-derived design checks may strengthen the existing
  `frontend-design` skill; they do not create a second frontend workflow owner.
- Matt Pocock workflow, TDD, planning, and grilling content will not duplicate
  Superpowers. Only clearly non-overlapping domain techniques may be evaluated.
- `assets/core/` is the sole runtime source. The stale historical
  `blueprints/core/` duplicate was removed in Phase 35 after a RED/GREEN
  runtime-source contract and adapter parity verification.
- Every future extension must have one owner, lazy routing, bounded output,
  project isolation, a fallback, and automated proof before it can ship.
- Full rationale: `docs/decisions/0002-extension-ownership-and-code-graph.md`.

Phase 35 through Phase 45 are complete. Baron 3.6 certifies the optional
code-map boundary without altering the public command flow or the release
promotion order. Version `3.6.0` is the current source-certified baseline.

## Baron 3.0 Direction

Baron 3.0 has two non-negotiable goals:

- Fix weak runtime assets. Baron skills and agents must be self-contained, local, tested, and strong enough to guide AI without relying on external GitHub links.
- Learn from `nousresearch/hermes-agent` without cloning it. Baron should adopt the useful ideas: skill lifecycle, safe self-improvement, session replay, background review, capability/runtime awareness, and continuity autopilot.

Core remains unchanged:

- Superpowers remains the workflow core.
- The three mandatory quality gates remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains Baron's durable source of truth.
- Optional skills and optional agents remain lazy-routed, never core.

## Baron 3.4 Direction

Baron 3.4 makes the simple public `baron update` command safe enough to own both
runtime updates and Baron-managed project refreshes.

Phase 35 removed the obsolete `blueprints/core/` tree before the update
baseline can record any core assets. The program now has four safety boundaries:

- keep `assets/core/` as the one runtime asset source
- remember the exact managed baseline before deciding what changed
- verify the new release candidate before it can touch the project
- stage conflicts and recover interrupted activation without data loss
- keep human release authority separate from AI local repair

The normal user still runs only `baron update`. Hidden continuation, abort, and
local reconciliation surfaces exist for recovery and AI automation without
crowding the public command flow.

## Baron 3.5 Direction

Baron 3.5 improves the knowledge already owned by Baron without adding another
workflow core or another frontend owner:

- distill Hallmark's useful brief, anti-template, responsive, and state checks
  into the existing `frontend-design` skill
- distill only Matt Pocock techniques that do not overlap Superpowers into the
  existing interface/architecture guidance and Product Harness domain language
- reject duplicate planning, TDD, debugging, review, handoff, setup, and
  workflow ownership
- keep all operational guidance local, version-pinned, lazy-routed, and tested

## Baron 3.6 Direction

Baron 3.6 adds an optional local code map for large and old repositories:

- Baron owns the provider contract, cache boundary, context budget, and fallback
- Graphify may supply a project-scoped code-only graph when the exact supported
  version is available
- Graphify cannot install hooks, own memory, write instructions, create a
  global graph, or become mandatory
- every graph result remains advisory until current source files verify it
- missing, stale, failed, or incompatible graph providers fall back to Baron
  Survey Engine without breaking the workflow

## Proposed Baron 3.7 Direction

Baron 3.7 is a quality-and-trust release, not a feature-volume release. Baron
already has the broader engine. The new program makes routine work lighter,
makes completion evidence harder to fake, and proves that Harness improvements
actually help a fresh agent before retaining them.

The proposed release has four user-visible outcomes:

- small, bounded changes avoid unnecessary durable Plan/Harness/Trace ceremony
  while risky, ambiguous, destructive, coordinated, or multi-session work keeps
  the full Baron safety path
- tests, builds, tools, and quality gates count only through structured
  execution receipts tied to the current project and source state, not because
  an agent wrote words such as `passed` or `verified`
- Baron Harness improvements require a comparable fresh-agent rerun before
  they can be kept or described as effective
- repositories can own a verified application runbook for starting, observing,
  validating, and cleaning up real application runs without Baron inventing
  commands, credentials, ports, product policy, or resource ownership

Baron 3.7 also distills a small set of evidence-first boundary checks from the
audited Harness source into Baron's existing owners. It does not install a new
`engineering-wisdom` workflow or create another quality agent.

The release must preserve these boundaries:

- Superpowers remains the only workflow core.
- The only mandatory quality agents remain `code-reviewer`,
  `security-auditor`, and `test-engineer`.
- Vault Markdown remains durable memory truth; SQLite remains a rebuildable
  accelerator only.
- Product Harness, Plan, Proof, Trace, Continuity, Autopilot, capabilities,
  adapters, and the optional code map remain Baron-owned systems.
- `assets/core/` remains the only managed runtime asset source.
- Normal users keep the small install, Vault setup, adapter/platform init, and
  update flow. New deep commands stay hidden and agent-facing where possible.
- No phase may weaken project-ID isolation, bounded context, redaction,
  user-file preservation, safe update transactions, or human release authority.
- During implementation, source version stayed `3.6.0` until Phases 46-51
  passed; Phase 52 then bumped and published `3.7.0`.
- Phase 52 is now complete: GitHub and the public README deliver `v3.7.0`
  through `releases/latest`, and a fresh public install succeeds.

## Completed Foundation

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
| 24 | Public Trust Release | completed | additive | concise README, public demo, certification snapshot, release/latest docs, source verification |
| 25 | Memory Index Resilience | completed | maintenance | duplicate-record RED/GREEN test, shared-Vault scanjob smoke, full verification, v3.1.3 release |
| 26 | API-Independent Latest Installer | completed | maintenance | no-API RED/GREEN contract, full lifecycle tests, one-block latest install, v3.1.4 release |

Baron 3.2 planned program:

| Phase | Name | Status | Baron 3.2 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 27 | Intent Clarity And Actionable Recovery | completed | 15% | shared-understanding gate, persisted intent brief, bounded questions, interruption/failure recovery packet, resume tests |
| 28 | Platform Intelligence Profiles | completed | 25% | deep profile packs for every public platform flag, repo-derived stack map, generated guidelines, task-aware context routing tests |
| 29 | Architecture Governor And Project Expansion | completed | 30% | primary-plus-extension model, safe `baron init --<platform>` expansion, structure/boundary/dependency contracts, old-repo no-destructive-change tests |
| 30 | Baron Design Quality And Reviewer Closure | completed | 15% | Baron-native frontend quality checks, design-context guidance, bounded post-edit validation, evidence-backed reviewer closure tests |
| 31 | Automation Certification And Baron 3.2 Release | completed | 15% | automatic adapter behavior, all-platform fixtures, fullstack-to-mobile expansion smoke, full verification, docs/status sync, v3.2.0 source proof |

Baron 3.3 planned program:

| Phase | Name | Status | Baron 3.3 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 32 | Request Authority Contract | completed | 30% | read-only/change/ambiguous classifier, multilingual and mixed-intent tests, generated adapter no-mutation rules |
| 33 | Coherent State And Completion Integrity | completed | 35% | state identity guard, no-write failure behavior, read-only SQLite queries, tampered-completion detection |
| 34 | Immutable Release Promotion And Baron 3.3 Certification | completed | 35% | exact-source manifest proof, proof-before-tag workflow, native/installer smoke, full verification, v3.3.0 source push |

Baron 3.4 planned program:

| Phase | Name | Status | Baron 3.4 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 35 | Single Runtime Source And Managed Baseline | completed | 25% | `blueprints/core/` removed after a no-runtime-reference RED/GREEN test, `assets/core/` proven as the sole source, managed baseline manifest, deterministic three-way decisions, custom/user asset exclusion, malformed-marker refusal, and read-only dry-run tests |
| 36 | Verified Release Candidate And Binary Handoff | completed | 25% | exact candidate target/version/source/checksum proof, downgrade refusal, raw release assets, installer compatibility |
| 37 | Conflict-Safe Activation And Recovery | completed | 30% | no-write conflict staging, frozen continuation, abort, transactional rollback, Windows/Unix recovery tests |
| 38 | Automation Contract And Baron 3.4 Certification | completed | 20% | local-only AI reconcile, one-command user update, full native/lifecycle certification, version and docs synchronization |

Baron 3.5 planned program:

| Phase | Name | Status | Baron 3.5 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 39 | Hallmark Frontend Distillation | completed | 35% | existing `frontend-design` gains local brief fingerprint, anti-template, responsive, and state proof without a second frontend owner |
| 40 | Deep Modules And Product Domain Language | completed | 30% | non-overlapping deep-module guidance and project-owned domain language are bounded, source-grounded, and isolated |
| 41 | Routing, Preservation, And Baron 3.5 Certification | completed | 35% | one workflow owner, three-adapter parity, lazy routing, custom preservation, behavior pressure tests, and version/docs synchronization |

Baron 3.6 planned program:

| Phase | Name | Status | Baron 3.6 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 42 | Code-Graph Provider Contract | completed | 25% | provider-neutral local graph model, explicit confidence, bounded output, project identity, rebuildable cache, and Survey fallback |
| 43 | Graphify Local Code-Only Adapter | completed | 30% | exact-version local extraction/query, no installers/hooks/global memory, timeout/size guards, malformed-output fallback |
| 44 | Automatic Bounded Context And Source Verification | completed | 25% | AI automation loads only task-relevant graph hits, labels inference, verifies source, and never crowds the public command flow |
| 45 | Isolation, Scale, And Baron 3.6 Certification | completed | 20% | same-name project isolation, old/large repository tests, adapter preservation, full certification, and version/docs synchronization |

Baron 3.7 program, implementation, certification, and publication completed:

| Phase | Name | Status | Baron 3.7 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 46 | Adaptive Work Shape And Decision Boundaries | completed | 15% | bounded changes avoid unnecessary durable state; durable/risky/ambiguous work retains the correct plan, intent, recovery, and proof path across all three adapters |
| 47 | Structured Execution Proof Receipts | completed | 20% | tests, builds, tools, and artifacts count only through bounded, tamper-checked receipts tied to project identity and current source state |
| 48 | Gate, Trace, And Completion Integrity | completed | 15% | mandatory agent gates and completion use fresh execution receipts; stale, reported-only, failed, skipped, or degraded evidence cannot become a false pass |
| 49 | Measured Harness Improvement Experiments | completed | 15% | an authorized intervention records a baseline, earliest gap, owner, hypothesis, comparable fresh-agent rerun, and keep/revise/remove decision |
| 50 | Application Runbook And Real-System Proof | completed | 10% | project-owned, evidence-backed start/readiness/state/interface/runtime-evidence/cleanup guidance supports isolated real-system validation without invented facts |
| 51 | Ownership-Safe Guidance And Integrated Certification | completed | 10% | selected boundary heuristics strengthen existing Baron owners; full preservation, adapter, scale, update, Vault, and regression gates pass without a second workflow or skill owner |
| 52 | Baron 3.7 Public GitHub Release And User Installability | completed | 15% | README and release docs identify `3.7.0`; exact source is committed and pushed; four native GitHub jobs pass; immutable tag/release/assets exist; `releases/latest` installs and reports `baron 3.7.0` in a fresh public smoke |

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
- [x] Historical Phase 0 blueprints existed for the original skeleton.
- [x] Phase 35 retired the stale `blueprints/core/` tree after proving no
  runtime path depends on it; `assets/core/` remains the only managed source.
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
- [x] Add Baron-owned public proof docs without pointing readers at external harness repositories.
- [x] Add a public certification snapshot with concrete verification commands.
- [x] Update release docs so `releases/latest` and tag publication are explicit.
- [x] Bump source release metadata to `3.1.2`.

### Phase 25 - Memory Index Resilience

- [x] Reproduce `UNIQUE constraint failed: records.id` on Baron `3.1.2`.
- [x] Identify repeated excerpts inside imported session Markdown as the source.
- [x] Add a failing regression test before changing production code.
- [x] Deduplicate identical records inside one source without rewriting Vault Markdown.
- [x] Verify the targeted Memory Firewall suite and real `scanjob` init with source build.
- [x] Run full workspace tests, Clippy, formatting, and status checks.
- [x] Verify real shared-Vault indexing without changing Vault Markdown.
- [x] Publish and verify the `v3.1.3` memory fix.
- [x] Record the installer quota issue as a separate follow-up instead of rewriting the release tag.

### Phase 26 - API-Independent Latest Installer

- [x] Reproduce latest installation failure under exhausted anonymous API quota.
- [x] Add a RED contract test for API-independent latest resolution.
- [x] Resolve latest through the published release manifest in both installers.
- [x] Add mirror override support for the latest-manifest URL.
- [x] Run full installer lifecycle, workspace, formatting, and Clippy tests.
- [x] Prove manifest-based latest installation works while the API is rate-limited.
- [x] Publish and smoke `v3.1.4` through the normal one-block install flow.

### Phase 27 - Intent Clarity And Actionable Recovery

- [x] Make Baron inspect project, Vault, current plan, and prior decisions before asking the user for information already available.
- [x] Ask one high-value question at a time only when the answer cannot be discovered safely.
- [x] Persist a concise intent brief covering current behavior, target behavior, scope, non-goals, constraints, decisions, proof, and remaining unknowns.
- [x] Require explicit shared understanding before medium/high-risk implementation while keeping tiny, obvious maintenance work lightweight.
- [x] Record an actionable recovery packet when work fails, blocks, or is interrupted: root cause, last successful step, evidence, affected files, safe next action, and retry conditions.
- [x] Preserve failed-attempt evidence instead of rewriting failure as success.
- [x] Auto-load the latest recovery packet during the next context/resume cycle.
- [x] Add core, CLI, adapter, interruption, failure, deduplication, and bounded-context tests.

Phase 27 final verification:

- intent and recovery RED/GREEN regression suites: passed
- all 17 adapter lifecycle tests: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- temp Codex/fullstack repo plus Vault smoke: unconfirmed risky intake blocked, confirmed intake passed, intent/recovery auto-loaded, low-risk intake remained lightweight, repo/Vault mirrors passed
- Survey self-noise RED/GREEN: Baron execution-state paths no longer masquerade as product risk surfaces

### Phase 28 - Platform Intelligence Profiles

- [x] Replace one-line platform hints with Baron-owned profile packs for `frontend`, `backend`, `fullstack`, `mobile`, `desktop`, `tool`, `library`, `data`, and `cloud`.
- [x] Define each profile's product concerns, architecture priorities, common failure modes, security/performance expectations, skill/agent routing, verification layers, and release proof.
- [x] Combine the selected profile with Survey Engine evidence so framework, language, database, deployment, and test guidance comes from the actual repo.
- [x] Mark missing stack facts as unknown instead of guessing.
- [x] Generate and maintain `PROJECT_PROFILE.md`, `STACK_MAP.md`, platform engineering guidance, and platform-specific quality gates using refreshable managed sections.
- [x] Make compact context load only the active task's relevant profile sections, not every platform pack.
- [x] Ensure custom project rules remain authoritative where they intentionally refine the Baron defaults.
- [x] Add focused fixtures and context assertions for every supported platform flag.

### Phase 29 - Architecture Governor And Project Expansion

- [x] Add a primary-platform plus extension-platform model without complicating the public user flow.
- [x] Keep first-time usage simple, for example `baron init --codex --fullstack`.
- [x] On an initialized project, make `baron init --mobile` add a mobile extension instead of silently replacing the fullstack foundation.
- [x] Let generated agent instructions run the same simple init command automatically when the user explicitly expands the product to a new platform.
- [x] Generate `CURRENT_ARCHITECTURE.md`, `PROJECT_STRUCTURE.md`, `BOUNDARIES.md`, `DEPENDENCY_RULES.md`, and `EXPANSION_RULES.md` under `docs/baron/architecture/`.
- [x] Provide adaptive reference structures for frontend, backend, database, shared contracts, infrastructure, documentation, and tests without forcing framework-incompatible folders.
- [x] Require every new module or top-level directory to have a clear responsibility, owner, allowed dependencies, and validation path.
- [x] Keep shared API/data contracts in one declared location so web, mobile, backend, and database changes stay compatible.
- [x] Scaffold new projects safely while preserving existing repositories; never move or delete existing code merely to match a preferred layout.
- [x] For structural migration of an old repo, require a plan, dry-run inventory, rollback path, and proof before file moves.
- [x] Detect architecture drift and report a bounded correction proposal rather than auto-restructuring the repo.
- [x] Add new-project, old-project, repeated-init, fullstack-to-mobile, multi-extension, conflict, preservation, and rollback tests.

### Phase 30 - Baron Design Quality And Reviewer Closure

- [x] Strengthen the existing optional `frontend-design` skill instead of installing a competing workflow or a second frontend owner.
- [x] Add Baron-native product/design context guidance that reuses existing Product Harness and architecture facts.
- [x] Distill useful frontend quality ideas into local, testable checks for overflow, contrast, typography, spacing, responsive behavior, accessibility, interaction states, motion safety, and design-system drift.
- [x] Route frontend checks only for matching UI work; backend, data, and tool-only edits must not pay the frontend validation cost.
- [x] Keep post-edit validation bounded, observable, and non-destructive; no mandatory third-party runtime or unreviewed vendored script bundle.
- [x] Require reviewer findings to close with fix evidence and verification, while preserving unresolved findings and previous failed evidence.
- [x] Keep the three core quality agents unchanged and prevent optional design checks from replacing `code-reviewer`, `security-auditor`, or `test-engineer`.
- [x] Add routing, conflict, false-positive, responsive, reviewer-closure, and evidence-preservation tests.

### Phase 31 - Automation Certification And Baron 3.2 Release

- [x] Update Codex, Claude, and generic adapter instructions so platform profiling, architecture reconciliation, intent checks, recovery, design validation, proof, and reviewer closure run automatically when relevant.
- [x] Keep the normal user-facing commands limited to install/update, Vault setup, adapter init, platform init/expansion, and update.
- [x] Prove AI automation uses task/path/risk evidence and does not load every profile, skill, agent, or guideline at once.
- [x] Certify all platform profiles against representative repositories and mixed-stack fixtures.
- [x] Smoke a fullstack project that expands to mobile while preserving backend, database, shared contracts, custom assets, plans, and Vault memory.
- [x] Smoke an existing irregular repository and prove Baron does not destructively rearrange it.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, installer lifecycle, and release smoke tests.
- [x] Synchronize README, architecture docs, generated adapter docs, status Markdown/JSON, build logs, version metadata, release assets, and public installation guidance.
- [x] Push `v3.2.0` source only after all Phase 27-30 evidence is complete and the simple user flow remains intact; keep binary GitHub Release publication explicit.

### Phase 32 - Request Authority Contract

- [x] Classify requests as `read_only`, `change`, or `ambiguous` before Baron automation mutates durable state.
- [x] Make explicit change outcomes win over review words, for example review plus apply fixes.
- [x] Keep ambiguous requests read-only until change authority is explicit.
- [x] Support common English and Vietnamese request intent without claiming perfect language understanding.
- [x] Expose a hidden AI command and keep normal user help uncluttered.
- [x] Update Codex, Claude, and generic adapters so answers, explanations, reviews, diagnoses, plans, and status reports do not create plan/Harness/proof/trace noise.
- [x] Prove classification and no-mutation behavior with core, CLI, and adapter tests.

Phase 32 verification:

- authority RED test failed because the module did not exist
- 4 core authority classification tests: passed
- 3 hidden CLI and no-write tests: passed
- all 19 adapter lifecycle tests: passed
- normal user help remains uncluttered: passed
- `cargo fmt --all -- --check`: passed

### Phase 33 - Coherent State And Completion Integrity

- [x] Validate project config, local Vault, capsule, and project identity before mutating execution state.
- [x] Fail with an actionable `baron update` recovery path and leave files unchanged when state is missing or mismatched.
- [x] Keep init/update as the only scaffold repair owners.
- [x] Open SQLite-backed memory/session query paths read-only so inspection cannot fabricate an empty database.
- [x] Detect completed plan text that lacks verification, proof, or passing trace evidence.
- [x] Surface integrity diagnostics in plan status without treating hand-edited state as truth.
- [x] Prove missing-state, mismatch, no-write, query, tampering, and preservation behavior.

Phase 33 verification:

- 4 coherent-state core tests: passed
- 14 Vault memory tests, including incompatible read-only cache preservation: passed
- 4 session replay tests, including incompatible read-only cache preservation: passed
- 6 plan tests, including hand-edited completion detection and valid completion integrity: passed
- 9 execution CLI tests, including identity mismatch no-repair behavior: passed
- full `baron-core`, `baron-adapters`, and `baron-cli` all-target suites: passed
- `cargo fmt --all -- --check`: passed

### Phase 34 - Immutable Release Promotion And Baron 3.3 Certification

- [x] Build releases from an exact source candidate before any release tag exists.
- [x] Match requested version, Cargo version, source SHA, manifest, native archives, binary versions, and checksums.
- [x] Prove all supported native targets and installer lifecycle before promotion.
- [x] Give write permission only to the final promotion job.
- [x] Refuse existing tags/releases and never replace published assets.
- [x] Bump Baron to `3.3.0` and synchronize README, release docs, status, JSON, plan, build log, and certification.
- [x] Pass the final no-skip workspace verification and static scans.
- [x] Push verified source to `origin/main` at `34a6cf4`; keep binary release publication separate unless explicitly requested.

Phase 34 final verification:

- 8 release identity, checksum, tamper, and exact-source tests: passed
- 2 release CLI identity tests: passed
- release workflow proof-before-tag and single-writer contract: passed
- installer install/update/rollback/uninstall lifecycle: passed
- release binary reports `baron 3.3.0`: passed
- real Vault/Codex/fullstack/context/plan/proof/trace/update preservation smoke: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets` with no skipped tests: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- `cargo build --release --locked -p baron-cli`: passed
- status JSON, release YAML lint, static immutability scan, and `git diff --check`: passed

### Phase 35 - Single Runtime Source And Managed Baseline

- [x] Add a RED contract proving runtime installers, adapters, tests, and
  manifests embed/read `assets/core/` only.
- [x] Delete all seven stale files under `blueprints/core/`.
- [x] Remove active documentation that treats blueprints as a maintained source.
- [x] Prove installed Codex, Claude, and generic assets still match
  `assets/core/` after cleanup.
- [x] Record the exact last-installed managed content and merge policy under `.baron/managed-state/`.
- [x] Keep all manifest paths repository-relative, canonical, and unable to escape the project.
- [x] Refuse a symlink or Windows junction anywhere in the managed-state write path.
- [x] Verify each baseline copy hash before treating it as a three-way merge ancestor.
- [x] Compute `BASE`, `LOCAL`, and `UPSTREAM` decisions before writing any managed target.
- [x] Plan all registered adapters together and include newly introduced upstream assets under their own adapter.
- [x] Preserve text outside Baron markers and custom routing blocks.
- [x] Preserve custom skills, custom agents, source, plans, Harness records, and Vault memory.
- [x] Treat uncertain dual edits as conflicts instead of guessing.
- [x] Prove the prepared baseline replacement advances the next comparison ancestor only after a successful activation boundary supplies it.
- [x] Keep the prior manifest loadable when a replacement fails before the new manifest is ready.
- [x] Pass focused planner, adapter lifecycle, and dry-run CLI tests.

### Phase 36 - Verified Release Candidate And Binary Handoff

- [x] Extend immutable release metadata with one raw update candidate per supported target.
- [x] Resolve production candidates through bounded HTTPS and deterministic injected test sources.
- [x] Verify product, schema, version ordering, target, size, checksum, source revision, and candidate-reported version.
- [x] Refuse downgrades, wrong targets, malformed identities, redirects outside trusted hosts, and tampered candidates.
- [x] Keep project managed targets and installed runtime unchanged until all candidate proof passes.
- [x] Add Unix atomic and Windows delayed-finalizer handoff primitives.
- [x] Preserve existing checksum-verified PowerShell/Bash installer behavior.
- [x] Pass release, workflow, candidate, and installer lifecycle tests.

### Phase 37 - Conflict-Safe Activation And Recovery

- [x] Store bounded transaction state plus `BASE`, `LOCAL`, `UPSTREAM`, and `RESOLVED` conflict packets.
- [x] Freeze hashes and project identity so stale or edited continuations are refused.
- [x] Let the verified candidate render the new managed project assets.
- [x] Apply managed writes atomically with per-target backup and rollback.
- [x] Keep conflicts out of live files and require explicit authority before hidden continuation.
- [x] Make abort remove only staged update state.
- [x] Recover or roll back after interruption, locked files, receipt failure, or runtime handoff failure.
- [x] Prove project assets and active runtime always return to one compatible version.

### Phase 38 - Automation Contract And Baron 3.4 Certification

- [x] Make public `baron update` the human-authorized complete update command.
- [x] Make hidden `baron automation reconcile` local-only and unable to download or replace Baron.
- [x] Update Codex, Claude, and generic instructions so AI never silently authorizes a release update.
- [x] Keep top-level help and README limited to the simple user flow.
- [x] Document the one-time installer bootstrap from pre-3.4 Baron.
- [x] Bump source and lockfile to `3.4.0` only after Phases 35-37 pass.
- [x] Pass full format, workspace tests, Clippy, release build, YAML, installer, candidate, conflict, recovery, and real project/Vault smoke.
- [ ] Push verified source only after the complete 3.4-3.6 program; keep tag/GitHub Release promotion explicit.

### Phase 39 - Hallmark Frontend Distillation

- [x] Keep `frontend-design` as the single frontend skill owner.
- [x] Add a project-evidence brief fingerprint before visual decisions.
- [x] Add bounded anti-template checks that reject interchangeable AI-looking
  layouts without forcing one house style.
- [x] Add responsive, loading, empty, error, focus, disabled, and reduced-motion
  proof states.
- [x] Keep source attribution and licensing in local provenance/notice files,
  never as a live runtime dependency.
- [x] Prove behavior through three-adapter, routing, static pressure contracts,
  and source-baseline comparison.

### Phase 40 - Deep Modules And Product Domain Language

- [x] Distill only Matt Pocock deep-module and domain-language techniques that
  do not overlap Superpowers.
- [x] Strengthen the existing `api-and-interface-design` owner instead of
  creating a Matt workflow skill.
- [x] Add project-scoped Product Harness domain language with
  evidence/unknown/conflict states.
- [x] Keep invented terms and cross-project terminology out of trusted context.
- [x] Prove bounded loading, project isolation, divergence withholding, and no
  duplicate ownership.

### Phase 41 - Routing, Preservation, And Baron 3.5 Certification

- [x] Re-audit every bundled skill trigger, exclusion, owner, and dependency.
- [x] Prove Codex, Claude, and generic adapter parity.
- [x] Prove custom skills, agents, routing blocks, plans, Harness records, and
  Vault memory survive updates.
- [x] Run static scans against live links, duplicate workflow ownership, and
  Hallmark/Matt installer instructions.
- [x] Run full Baron 3.5 verification before bumping source/lock/docs to
  `3.5.0`.
- [x] Reconcile missing managed Domain Language documents without overwriting
  user-written terms or custom routing assets.
- [x] Certify source `3.5.0` with fresh workspace tests, Clippy, locked release
  build, public-flow, adapter, and preservation evidence.

### Phase 42 - Code-Graph Provider Contract

- [x] Add a Baron-owned provider-neutral graph contract with explicit extracted
  versus inferred confidence.
- [x] Store graph state only in a project-scoped rebuildable
  `.baron/cache/code-graph/`.
- [x] Bound hit count, characters, paths, and graph size; reserve subprocess
  limits for the provider adapter phase.
- [x] Reject unsafe paths, stale project identity, and cache symlink/junction
  traversal before any state write.
- [x] Keep Survey Engine as the mandatory fallback; no provider invocation
  exists in this phase.
- [x] Register `graphify-local` only as an optional `code-map` capability and
  preserve any project-owned `code-map` provider.

### Phase 43 - Graphify Local Code-Only Adapter

- [x] Support only the pinned compatible Graphify version and local code-only
  extraction/query surfaces.
- [x] Never invoke Graphify installers, hooks, instruction writers, work
  memory, global graph, semantic backends, or platform setup.
- [x] Disable provider-side query logging for Baron-owned calls.
- [x] Fall back cleanly on absence, mismatch, timeout, malformed output, or
  oversize results.
- [x] Prove no target source, Vault memory, or agent instruction is mutated.

### Phase 44 - Automatic Bounded Context And Source Verification

- [x] Let AI automation refresh/query the graph only when task scope benefits.
- [x] Keep normal user commands limited to install, Vault setup, agent/platform
  init, and update.
- [x] Put a small `Optional Code Map` section in context only when useful.
- [x] Require direct source verification before graph guidance supports a
  decision, proof, trace, or durable memory.
- [x] Explain provider/fallback decisions without dumping the graph.

### Phase 45 - Isolation, Scale, And Baron 3.6 Certification

- [x] Prove same-name repositories cannot share graph state or graph-derived
  memory.
- [x] Prove large and ten-year repository fixtures stay bounded and recover from
  stale/corrupt caches through Survey fallback.
- [x] Prove custom assets and all three adapters survive update without any
  Graphify instruction, hook, or root-output mutation.
- [x] Run full workspace, Clippy, release build, installer, Vault, adapter, and
  old-repository smoke verification.
- [x] Bump source/lock/docs to `3.6.0` only after all Phase 42-44 evidence passes.

### Phase 46 - Adaptive Work Shape And Decision Boundaries

Status: `completed`; implementation, English/Vietnamese fixtures, adapter
startup parity, and focused lifecycle tests pass.

Goal: keep Baron's full safety systems while making their activation
proportional to the actual work instead of treating every meaningful code edit
as the same lifecycle shape.

- [x] Model mutation authority, durable-memory need, human-judgment need, risk,
  proof type, and lifecycle depth as separate decisions.
- [x] Keep answers, explanations, reviews, diagnoses, plans, and status reports
  read-only with no Plan, Harness, Proof, Trace, friction, or learning writes.
- [x] Let a bounded, single-session, reversible change with clear expected
  behavior use an ephemeral execution path and focused proof without creating
  unnecessary durable Plan or Harness story state.
- [x] Require durable Plan and Continuity state when work spans sessions,
  coordinates contributors or agents, has meaningful dependencies, needs a
  recovery procedure, or cannot safely resume from its diff.
- [x] Keep high-risk proof and mandatory gates strong even when the code change
  itself is short or mechanically small.
- [x] Stop before mutation when materially different externally observable
  choices remain open; a configurable default is not product authority.
- [x] Extend intent evidence so the current authority source, unresolved
  choices, policy owner, and required user decision are explicit.
- [x] Preserve the rule that the agent reads repository, Vault, plan, Harness,
  continuity, and prior decisions before asking one missing high-value question.
- [x] Make routing output explain why lifecycle state was skipped or required so
  a lighter workflow never becomes silent proof weakening.
- [x] Add English and Vietnamese fixtures for read-only work, bounded changes,
  multi-session work, short high-risk changes, unclear product policy, and
  difficult recovery.
- [x] Prove Codex, Claude, and generic adapters make the same work-shape decision
  without loading every profile, skill, or agent.

Phase 46 exit gate:

- [x] A routine bounded change completes with focused evidence and no unrelated
  durable lifecycle files.
- [x] A bounded authentication or permission change still receives confirmed
  authority, security review, test proof, and completion protection.
- [x] A long documentation migration receives durable planning and recovery
  without being misclassified as a security task.
- [x] Ambiguous product policy produces no source or Baron-state mutation.

### Phase 47 - Structured Execution Proof Receipts

Status: `completed`; trusted runner, source/project binding, integrity checks,
bounded output, redaction, stale/tamper/cross-project checks, and receipt-backed
proof references pass focused and integrated tests. Existing 3.6 Markdown remains
readable as legacy reported evidence.

Goal: make tool-backed proof describe an execution Baron can validate instead
of accepting an agent-written sentence containing words such as `passed`,
`verified`, `test`, `build`, or `smoke`.

- [x] Define a versioned execution-receipt schema containing project ID, task or
  plan identity when present, source revision or worktree fingerprint, provider,
  capability, executable, argument vector, working directory, start/end time,
  exit code, result, bounded output digests, and produced artifact digests.
- [x] Accept proof only from a Baron-owned execution runner or a registered,
  verified backend that creates the receipt atomically from the execution it
  actually observed. Handwritten, imported, agent-authored, or schema-shaped
  receipt data remains reported evidence and cannot satisfy an execution gate.
- [x] Give each trusted receipt an integrity chain that binds the registered
  backend, invocation, source state, result, bounded output, and artifacts;
  reject a receipt when any bound field cannot be independently checked.
- [x] Run supported local proof commands as executable plus arguments; do not
  interpolate caller-controlled values into shell source.
- [x] Record `passed`, `failed`, `skipped`, and `degraded` as distinct outcomes.
  A missing optional provider may degrade but must never appear as passed.
- [x] Bind receipts to Baron project identity so two repositories with the same
  name cannot reuse each other's execution evidence.
- [x] Detect source changes after a receipt and mark that receipt stale for
  claims affected by the change.
- [x] Reject malformed, oversized, path-escaping, symlink/junction-traversing,
  wrong-project, wrong-revision, tampered, or replayed receipt data.
- [x] Bound stdout, stderr, metadata, and artifact inventories. Store digests or
  redacted excerpts where full output would leak secrets or flood Vault memory.
- [x] Keep durable human-readable proof in Vault Markdown while treating any
  machine receipt/cache as rebuildable or independently verifiable metadata.
- [x] Keep historical free-form proofs readable and clearly label them as
  legacy reported evidence; do not silently upgrade them to executed proof.
- [x] Add a safe migration path for initialized Baron 3.6 projects without
  deleting their existing Proof, Trace, Harness, or Vault records.

Phase 47 exit gate:

- [x] The sentence `tests passed` without a matching receipt cannot satisfy an
  execution-required proof gate.
- [x] A command with a non-zero exit cannot be recorded as passed.
- [x] Output from an old or different source revision cannot prove the current
  source.
- [x] A missing freshly produced artifact cannot be hidden by a stale artifact
  left from an earlier build.
- [x] Receipt tampering, cross-project reuse, and secret-bearing output fixtures
  fail safely without corrupting durable memory.

### Phase 48 - Gate, Trace, And Completion Integrity

Status: `completed`; medium/high completion now requires current receipt-backed
proof and all three current quality-gate receipts; legacy Markdown cannot close
work.

Goal: connect quality-agent gates, Proof, Trace, reviewer closure, capability
execution, and Stop reconciliation to the structured receipts from Phase 47.

- [x] Give each mandatory quality-gate run a stable run ID, project/source
  identity, agent name, findings or no-finding result, evidence digest, and the
  verification receipts it relied on.
- [x] Count `code-reviewer`, `security-auditor`, or `test-engineer` only when a
  matching gate run actually occurred for the relevant current task and source.
- [x] Invalidate stale gate evidence after relevant source changes instead of
  finding only an old occurrence of the agent name in a Markdown file.
- [x] Require reviewer closure to reference both fix evidence and a fresh
  verification receipt; preserve every failed or superseded attempt.
- [x] Replace keyword-based proof satisfaction with receipt-backed rules matched
  to focused, integration, end-to-end, recovery, security/data-impact, or
  measurement claims.
- [x] Make Trace scoring distinguish verified facts, reported summaries,
  degraded optional checks, missing proof, and unattempted work.
- [x] Keep a failed Trace score as a hard completion stop.
- [x] Make Stop reconciliation reject reported-only, failed, stale, mismatched,
  or incomplete evidence without entering a hook loop.
- [x] Explain the exact missing or stale receipt and safe next action so a user
  is not left with a generic `proof insufficient` message.
- [x] Preserve custom quality agents as optional assets without letting them
  replace the three mandatory Baron gates.

Phase 48 exit gate:

- [x] A hand-edited gate Markdown entry cannot close a required gate.
- [x] A gate run against an older source revision cannot complete changed code.
- [x] Failed, skipped, or degraded checks remain visible and cannot become a
  false green through summary wording.
- [x] High-risk completion passes only with current required gates, relevant
  execution receipts, and a detailed passing Trace.

### Phase 49 - Measured Harness Improvement Experiments

Status: `completed`; approval, baseline/hypothesis, fresh rerun, and
keep/revise/remove/pending lifecycle pass focused and CLI tests.

Goal: retain a Harness intervention only after a comparable fresh-agent rerun
shows that the intervention was available, used, relevant, and beneficial.

- [x] Keep ordinary post-task Autopilot review candidate-only. It may report
  repeated friction but must not start or apply an experiment by itself.
- [x] Require explicit user authority before an experiment changes guidance,
  tools, runbooks, validation, skills, agents, core policy, or architecture.
- [x] Record a representative task, accepted outcome, baseline repository
  revision, adapter/worker, tools, authority, external conditions, proof,
  retries, human steering, and known limitations.
- [x] Classify the earliest useful gap as context, capability, domain ownership,
  authority, proof, environment, or another explicitly justified owner.
- [x] Assign the correction to Baron, the consumer repository, the external
  environment, or a human decision; do not copy consumer-specific policy into
  generic Baron assets.
- [x] State one falsifiable intervention hypothesis, evidence that would weaken
  it, its maintenance owner, expected cost, and removal condition before edits.
- [x] Apply only the approved bounded intervention and run native validation for
  the changed owner.
- [x] Require a fresh agent/session with materially equivalent task class,
  authority, tools, starting state, and relevant external conditions.
- [x] Record separately whether the intervention was available, retrieved or
  invoked, relevant, and causally connected to the observed outcome.
- [x] Compare outcome, proof strength, human steering, retries, authority
  behavior, context cost, and maintenance cost.
- [x] Finish as `keep`, `revise`, `remove`, or `pending fresh rerun`. Never call
  the Harness improved while the rerun is pending or did not exercise the
  intervention.
- [x] Bind experiments to project identity and preserve their history in Vault
  without promoting unapproved results to verified facts.

Phase 49 exit gate:

- [x] A free-form `outcome improved` message cannot close an experiment.
- [x] A rerun that did not retrieve or invoke the intervention stays pending.
- [x] An intervention that adds noise without improving the representative task
  can be removed while its evidence remains preserved.
- [x] Same-name repositories cannot share experiment baselines or results.

### Phase 50 - Application Runbook And Real-System Proof

Status: `completed`; project-owned bounded runbook loading and runtime-task
routing pass focused tests. Baron preserves unknown application facts and leaves
live interface execution and resource ownership to the project repository.

Goal: make application operation legible when a task needs it, while leaving
application truth and resource ownership with the project.

- [x] Define a project-owned application runbook contract under
  `docs/baron/operations/` for scope, prerequisites, exact start command,
  process/project identity, ports and writable state, readiness, deterministic
  scenario state, real interface, runtime evidence, ownership/cleanup,
  validation, and unknowns.
- [x] Populate or refresh only facts supported by repository, user, runtime, or
  verified proof evidence. A heading or template field is not operational truth.
- [x] Distinguish fixed, configured, defaulted, observed, likely, stale, and
  unknown values; do not turn a detected command into a verified run command
  until its relevant execution succeeds.
- [x] Never invent credentials, ports, fixtures, product policy, readiness
  signals, log fields, cleanup commands, or resource ownership.
- [x] Route the runbook only for operate, reproduce, runtime-debug, end-to-end,
  deployment-smoke, or comparable tasks; unrelated work must not pay its context
  cost.
- [x] Start only an isolated instance or explicitly approved shared target,
  prove readiness, create known state without touching unowned state, exercise
  the real interface, inspect correlated runtime evidence, and validate through
  the same interface.
- [x] Track resources created by the current run and clean up only those
  resources. Missing cleanup authority remains an explicit unknown.
- [x] Keep secrets and private runtime data out of reports, receipts, traces,
  session replay, and Vault memory.
- [x] Integrate with platform profiles and the existing observability skill
  instead of adding a competing operations workflow owner.

Phase 50 exit gate:

- [x] Missing readiness evidence cannot be reported as ready.
- [x] A likely but unexecuted command cannot satisfy application proof.
- [x] The runtime flow cannot stop or delete resources it does not own.
- [x] Frontend, backend/API, CLI/tool, desktop, data, and old irregular
  repository fixtures preserve unknowns and use bounded task context.

### Phase 51 - Ownership-Safe Guidance And Integrated Certification

Status: `completed`; existing owners expose work-shape, trusted receipt,
experiment, and runbook contracts. Core/CLI suites, Clippy, locked release build,
release-profile certification smoke, adapter/preservation/scale/regression gates,
and public documentation checks pass on the certified 3.7 release source.

Goal: strengthen existing Baron owners with selected evidence-first boundary
checks, then prove Phases 46-50 work together without damaging Baron 3.6
capabilities or creating a second workflow.

- [x] Record provenance and license information for the audited
  reviewed upstream harness release source outside operational skill
  guidance.
- [x] Distill composition-root and shipped-artifact verification into
  `test-engineer`, not a new general engineering skill.
- [x] Distill honest automation-failure handling into Proof/Test ownership so
  a wrapper cannot hide a failed build or reuse a stale artifact.
- [x] Distill decode/validate/recover boundary rules and pass-external-values-as-
  data rules into Baron's existing security owners.
- [x] Distill adapter semantic-parity checks into adapter certification and the
  existing code-review owner.
- [x] Distill bounded cumulative-state checks into the existing performance
  owner.
- [x] Preserve the counter-pressure rule: a heuristic is advice supported by
  repository evidence, not universal project policy or authority to rewrite an
  architecture.
- [x] Prove no `engineering-wisdom`, duplicate upstream workflow, fourth core
  agent, duplicate planning system, or live upstream dependency enters the
  runtime.
- [x] Prove Codex, Claude, and generic adapter parity for work shape, receipts,
  gates, experiments, and application-operation context.
- [x] Prove custom skills, agents, routing blocks, user text, hooks, source,
  plans, Harness records, Proof, Trace, Continuity, Vault memory, and code-map
  state survive init, local reconcile, and update planning.
- [x] Prove Baron 3.6 projects update without destructive migration and retain
  readable historical evidence.
- [x] Run focused scale, corruption, interruption, same-name project,
  shared-Vault, stale-receipt, redaction, context-budget, and recovery tests.
- [x] Run the complete local workspace, Clippy, locked release build, installer,
  adapter, old-repository, and release-workflow contract gates.
- [x] Keep package/source/public version at `3.6.0` through Phase 51; the
  certified source is ready for the separately authorized Phase 52 bump.

Phase 51 exit gate:

- [x] Phases 46-50 have no open correctness, security, isolation, preservation,
  or false-completion blocker.
- [x] The normal public command flow remains install, Vault setup,
  adapter/platform init, and update.
- [x] A detailed certification report names exact commands, results,
  limitations, and the source revision proposed for Phase 52.
- [x] The user has authorized Phase 52's GitHub push and public release action;
  otherwise stop with all local evidence preserved and do not publish.

### Phase 52 - Baron 3.7 Public GitHub Release And User Installability

Status: `completed`; Baron 3.7 is publicly released and verified from
`releases/latest`.

Goal: finish the release completely. This phase does not end at `source ready`,
`source certified`, `pushed`, or `workflow started`. It ends only when a normal
user can follow the public README, download `releases/latest`, and obtain a
working `baron 3.7.0`.

Final public evidence:

- Source commit: `cc14c222130ac2047d36b3b752d9140521d3538e`
- GitHub Actions: [`31582187832`](https://github.com/thienty1207/Baron-Engine/actions/runs/31582187832)
- Immutable tag/Release: [`v3.7.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v3.7.0)
- Native assets: Windows x64, Linux x64, macOS Intel, macOS Apple Silicon;
  raw candidates, `SHA256SUMS`, `release-manifest.json`, `install.ps1`, and
  `install.sh` are present.
- Public Windows smoke: fresh install from
  `releases/latest/download/install.ps1` returned `baron 3.7.0`; setup,
  fullstack Codex init, context, same-version update guard, and user-marker
  preservation passed in `C:\Users\tytyb\AppData\Local\Temp\baron-3-7-public-smoke-ec0b95dd0c56419cbb93f87292510cd0`.

- [x] Confirm the user's approval explicitly includes source version changes,
  commits, push to GitHub, immutable tag/Release publication, and public install
  smoke. If approval covered local implementation only, stop and ask before any
  external write.
- [x] Confirm a clean, understood release worktree; preserve unrelated user
  changes and resolve the exact source revision to release.
- [x] Bump workspace crates, `Cargo.lock`, release metadata, manifests, tests,
  documentation references, and stable-source assertions from `3.6.0` to
  `3.7.0` only after all Phase 46-51 gates pass.
- [x] Use a release-candidate branch/ref and a truthful two-step README
  transaction so the default branch never claims `v3.7.0` is publicly
  downloadable before `releases/latest` proves it. If the existing workflow
  requires `main`, keep the candidate README explicit that `v3.6.0` is still
  the current public version and flip the public status only in the final
  post-release documentation commit.
- [x] Update the root `README.md` in the release candidate with the complete
  `v3.7.0` installation, verification, reinstall, and recovery instructions,
  using time-stable verification-first wording that remains true inside the
  immutable tag: accept the install only when `releases/latest` resolves to
  `v3.7.0` and the installed binary reports `baron 3.7.0`; otherwise stop and
  preserve the currently verified stable installation instead of claiming the
  candidate is publicly available.
- [x] Make README provide a copyable Windows PowerShell install block using
  `releases/latest/download/install.ps1`, followed by `baron --version` and an
  explicit requirement that it print `baron 3.7.0`.
- [x] Make README provide the supported Linux/macOS install block using
  `releases/latest/download/install.sh`, plus the same exact-version check.
- [x] Keep README's Windows reinstall instructions explicit: restore the Vault
  and project folders, install Baron from `releases/latest`, reconnect with
  `baron setup --vault <path>`, and run `baron update` in every restored Baron
  project.
- [x] Update `docs/RELEASE.md`, `docs/BARON_STATUS.md`,
  `docs/BARON_STATUS.json`, `notes/build-log/CURRENT.md`, the active design and
  plan, certification, command-surface/version references, and release notes so
  they agree on the candidate state without claiming publication early.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets --no-fail-fast` with no hidden
  release-relevant skip accepted as proof.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run `cargo build --release --locked -p baron-cli` and verify the exact
  built binary reports `baron 3.7.0`.
- [x] Validate status JSON, release workflow YAML, release manifest generation,
  installer lifecycle, documentation links, stale-version scans,
  `git diff --check`, and the complete pre-release certification profile.
- [x] Run a fresh local release-binary smoke with a new temporary Vault and
  project: `setup --vault`, `init --codex --fullstack`, bounded context,
  work-shape behavior, proof receipt verification, and safe update planning.
- [x] Commit the exact verified release candidate intentionally and push it to
  the certified GitHub release ref; verify the remote commit SHA equals the
  candidate SHA. Do not update the default branch to a false public state.
- [x] Dispatch the immutable GitHub release workflow from that exact remote
  source SHA. Verify the workflow accepts that certified ref, and do not create
  or move the release tag before the proof jobs pass.
- [x] Wait for exact-source verification, Ubuntu verification, Windows x64,
  Linux x64, Intel macOS, Apple Silicon macOS, archive/checksum generation, and
  installer lifecycle proof. Record the workflow run ID and every target result.
- [x] Verify GitHub created immutable tag `v3.7.0` at the certified source SHA
  and a non-draft, non-prerelease GitHub Release.
- [x] Verify the Release contains all expected native archives, raw update
  candidates if required by the updater contract, `SHA256SUMS`,
  `release-manifest.json`, `install.ps1`, and `install.sh`, with valid checksums
  and binary-reported versions.
- [x] Verify GitHub `releases/latest` resolves to `v3.7.0`, not `v3.6.0` or a
  cached/older release.
- [x] Download `install.ps1` from the public `releases/latest` URL into a fresh
  Windows temporary directory, install without relying on the local build, and
  verify `baron --version` prints exactly `baron 3.7.0`.
- [x] Using that public installation, run a fresh `setup --vault`,
  `init --codex --fullstack`, `context`, proof-receipt smoke, and `baron update`
  preservation smoke.
- [x] Verify the README install commands themselves work as written and that a
  user following only the README cannot accidentally receive the older Baron
  release without an explicit version-mismatch warning.
- [x] After public proof succeeds, set stable source, latest downloadable
  release, target release, program completion, current phase, remaining phase
  count, and public install guidance to final `v3.7.0` values. Update this
  status file, status JSON, current build log, current plan, root README, and
  certification with the exact tag SHA, GitHub Actions run ID, assets, and
  public-smoke result.
- [x] Commit and push the certified source plus final public-proof documentation
  to `origin/main`, then verify the remote default branch contains the final
  README/status state and the local branch is clean and synchronized with it.

Phase 52 hard stop rules (permanent release policy, not outstanding work):

- If any native job, checksum, installer, tag, Release asset,
  `releases/latest`, public README command, or fresh public install fails, keep
  Phase 52 incomplete and record the exact failure, last successful step,
  affected source SHA, GitHub run ID, safe next action, and retry condition.
- If publication cannot be completed during the authorized release run,
  restore and push a truthful default-branch README/status state before
  stopping: it must name the version that `releases/latest` really installs.
  Never leave an unverified `3.7.0` download claim on `origin/main` for a later
  session to discover.
- Never report Baron 3.7 as released merely because local tests passed,
  source was committed, source was pushed, a workflow started, or a tag exists.
- Never leave README claiming `3.7.0` is publicly downloadable while
  `releases/latest` still installs another version; either complete publication
  promptly or record and repair the mismatch before declaring success.
- The final success statement must include the public release URL, exact tag
  SHA, GitHub Actions run ID, verified asset inventory, public installer smoke,
  and confirmation that README installation returns `baron 3.7.0`.

## Baron 3.8 Program - Completed

> Completed implementation and release record. Every task and exit gate in
> Phase 53 through Phase 64 is checked below with evidence recorded in the
> active design, plan, build log, status JSON, tests, and public release notes.

Baron 3.8 has one primary outcome: when a different supported AI agent opens an
existing project, Baron should give it a small, trustworthy resume brief that
explains what the project is, what is being built, where work stopped, which
decisions are current, what evidence exists, what remains blocked, and what the
next safe action is. It should do this without rereading the whole repository or
loading the whole Vault into the model context.

The completed program has 12 phases in three groups:

| Group | Phases | Outcome |
| --- | --- | --- |
| Smart memory and measurable continuity | 53-56 | Establish an honest baseline, durable layered memory, better recall, and automatic low-token handoff between agents. |
| Wiki and CodeGraph knowledge | 57-59 | Connect memory, documentation, decisions, and current source relationships without creating a second source of truth. |
| Safety, scale, security, and closure | 60-64 | Protect memory, prove realistic local scale, harden `vibe-security-scan`, add a narrow optional reverse pack, then close documentation and GitHub backup. |

Program non-goals:

- No GUI, dashboard, visual memory browser, or separate user-facing application.
- No team accounts, RBAC, multi-tenant server, REST platform, hosted proxy,
  Docker stack, mandatory daemon, or enterprise control plane.
- No replacement for Vault Markdown as durable truth and no cloud or paid model
  dependency for Baron's default memory path.
- No second workflow core, router, proof system, case lifecycle, memory owner,
  or mandatory quality-agent set.
- No import of the full TencentDB Agent Memory or `reverse-skill` repositories
  as runtime dependencies.
- No offensive automation, authorization shortcuts, auto-installed security
  tooling, global MCP mutation, payload execution, persistence, evasion, CTF,
  pwn, red-team, EDR-bypass, or Active Directory attack pack.
- No Baron 3.8 public certification matrix, immutable release tag, GitHub
  Release, native archive set, or public installer promotion. Those require a
  separately approved future release phase if the owner wants them later.

Program-wide acceptance rules:

- [x] Keep project ID, never folder basename, as the isolation key at storage,
  retrieval, ranking, caching, handoff, Wiki, and CodeGraph boundaries.
- [x] Keep Vault Markdown authoritative and human-recoverable; every database,
  vector index, graph, and derived summary must be disposable and rebuildable.
- [x] Load only bounded, task-relevant context. A large Vault or repository must
  not become a large prompt by default.
- [x] Label source, revision, timestamp, trust, freshness, and uncertainty so an
  agent can distinguish current evidence from stale or inferred knowledge.
- [x] Require execution receipts for tests, tools, and quality gates. Retrieved
  text, configured providers, or an agent's written claim do not count as proof.
- [x] Preserve user text, custom hooks, custom skills, custom agents, and old
  repositories through shadow-first, non-destructive update and migration paths.
- [x] Keep Codex, Claude, and generic-agent behavior equivalent at the contract
  level, while allowing each adapter to use its native capabilities safely.
- [x] Make every optional provider fail closed for proof and degrade gracefully
  for ordinary context; normal coding must still work without embeddings,
  CodeGraph tools, or reverse-analysis tools.
- [x] Record benchmark evidence before and after optimization. Baron must not
  claim better memory, lower token cost, stronger detection, or higher scale
  from feature presence alone.
- [x] Do not check a phase as complete until its listed exit gates pass and the
  required status, plan, design, build-log, continuity, proof, and trace records
  have been updated during the later authorized implementation batch.

### Phase 53 - Memory And Resume Benchmark

Status: `completed`; implementation and exit gates passed.

Goal: define an honest, repeatable measurement system before changing memory.
This phase establishes what Baron 3.7 can and cannot remember today, how much
context it spends, and the exact thresholds Baron 3.8 must beat.

Planned work:

- [x] Define a canonical Resume Brief answer contract containing at least:
  project identity, source revision, current objective, current phase or task,
  last successful checkpoint, confirmed decisions, open blocker, affected
  files, proof/test status, unknowns, and next safe action.
- [x] Build deterministic fixtures for a new repository, a legacy repository,
  a large repository, an interrupted task, stale and superseded decisions,
  conflicting evidence, a renamed project, and two same-named folders with
  different project IDs.
- [x] Exercise fresh Codex, Claude, and generic-agent sessions against the same
  expected answers so adapter quality is compared on one contract.
- [x] Capture Baron 3.7 baseline correctness, missing-field rate, stale-answer
  rate, cross-project leakage, context tokens/bytes, index and recall latency,
  and time-to-first-safe-action.
- [x] Compare the bounded Baron path with an explicitly measured full-repo or
  full-history reading path; do not estimate token savings from file counts.
- [x] Score required resume facts from deterministic expected evidence rather
  than accepting an LLM's self-reported confidence as the result.
- [x] Emit machine-readable and human-readable benchmark reports with the exact
  source revision, fixture revision, configuration, commands, and raw evidence.
- [x] Freeze realistic local scale, latency, memory, correctness, and token
  targets that Phase 54-61 must meet; do not move thresholds after seeing final
  results without recording and approving the reason.

Phase 53 exit gates:

- [x] The benchmark repeats with equivalent results from a clean local state.
- [x] The current Baron 3.7 baseline is recorded, including failures and
  unknowns, without retroactively describing planned features as present.
- [x] Cross-project leakage tolerance is exactly zero in every isolation
  fixture.
- [x] Required resume-field, token-budget, latency, and realistic scale targets
  are fixed before memory optimization begins.
- [x] Phase 53 adds only the measurement harness and fixtures needed to define
  the baseline; it does not silently implement Phase 54-59 features.

### Phase 54 - Layered Durable Coding Memory

Status: `completed`; implementation and exit gates passed.

Goal: let Baron remember long-running coding work at several useful levels
without turning raw session text into trusted facts or replacing Vault Markdown.

Planned work:

- [x] Define four explicit memory layers: bounded raw/session evidence; verified
  events and facts; decisions, scenarios, and work outcomes; durable project
  invariants and current direction.
- [x] Give durable records project ID, stable record ID, type, source file or
  session, source revision, timestamps, content hash, confidence, sensitivity,
  and lifecycle state such as `candidate`, `trusted`, `contested`,
  `superseded`, or `expired`.
- [x] Keep imported sessions, Autopilot observations, and model-generated
  summaries as candidates until evidence or an allowed human workflow promotes
  them; frequency alone must never turn a claim into truth.
- [x] Add explicit conflict, supersession, expiry, and revalidation behavior so
  the newest-looking record cannot silently override a stronger decision.
- [x] Make deduplication deterministic and identity-based across repeated
  imports, restarts, path normalization, and equivalent adapter sessions.
- [x] Keep Markdown records readable and recoverable while allowing SQLite or
  other local indexes only as disposable accelerators.
- [x] Provide a non-destructive migration and rollback path for existing Baron
  Vaults; old records remain readable and no successful migration deletes the
  original evidence.
- [x] Keep import bounded, redacted, exact-project-matched, and explicit about
  omitted or unknown content.
- [x] Test restart, full cache deletion/rebuild, duplicate import, decision
  supersession, stale evidence, renamed folders, and same-name project
  isolation.

Phase 54 exit gates:

- [x] A fresh process rebuilds all accelerators from Vault Markdown and returns
  the same trusted memory result.
- [x] Candidate, contested, superseded, and expired records cannot appear as
  unqualified current facts in context.
- [x] Existing Vault data migrates and rolls back without destructive loss.
- [x] Layering improves the Phase 53 resume-field result without violating its
  fixed token budget or zero-leakage gate.

### Phase 55 - Hybrid Project Recall

Status: `completed`; implementation and exit gates passed.

Goal: retrieve the right small set of project knowledge using local lexical
search plus optional semantic assistance, with transparent ranking and a free,
reliable fallback.

Planned work:

- [x] Preserve local lexical search as the mandatory baseline and add optional
  semantic/vector retrieval only behind a capability contract.
- [x] Filter by project identity, trust state, sensitivity, and eligibility
  before ranking or fusing candidates; post-ranking filtering is not an
  acceptable isolation boundary.
- [x] Define hybrid fusion and reranking using task relevance, current plan and
  checkpoint relevance, recency, lifecycle state, source quality, and direct
  file/symbol relationships.
- [x] Support Vietnamese and English coding queries without requiring every
  durable record to be duplicated in two languages.
- [x] Enforce top-k, per-source, and total context budgets, including bounded
  snippets rather than entire documents or sessions.
- [x] Expose a concise `why this was recalled` explanation with source path,
  revision, trust/freshness label, score components, and omitted-result reason
  where useful.
- [x] Key caches by project identity, source revision, index version, and
  retrieval configuration so stale ranking cannot cross a changed repository.
- [x] Ensure missing, offline, incompatible, rate-limited, or failed semantic
  providers fall back to the local lexical path without blocking normal coding.
- [x] Benchmark lexical-only and hybrid modes against the frozen Phase 53 cases,
  including stale distractors and same-name projects.

Phase 55 exit gates:

- [x] Hybrid recall meets the Phase 53 correctness target and improves the
  agreed metric over the Baron 3.7 baseline.
- [x] Lexical-only recall remains fully usable with no cloud account, paid API,
  embedding service, or network access.
- [x] Every returned durable claim has traceable provenance and an honest
  freshness/trust label.
- [x] Cross-project leakage remains zero before, during, and after ranking.

### Phase 56 - Automatic Checkpoints And Token-Aware Cross-Agent Handoff

Status: `completed`; implementation and exit gates passed.

Goal: strengthen the existing Continuity Ledger so a newly opened agent resumes
real work correctly from a compact brief instead of asking the owner to repeat
history or rereading the whole Vault.

Planned work:

- [x] Extend the existing Continuity owner rather than creating a parallel
  checkpoint, journal, handoff, or session-management system.
- [x] Capture idempotent, atomic checkpoints after meaningful milestones,
  direction changes, decision confirmation, relevant test/proof changes,
  commit boundaries, interruption, and detected failure.
- [x] Preserve cause, objective, last successful step, source revision, affected
  files, evidence, blocker, safe next action, retry conditions, and unknowns in
  every actionable recovery packet.
- [x] Add reconciliation for missed native hooks so instruction-only behavior is
  never described as guaranteed automatic capture.
- [x] Compile a bounded Resume Brief from current trusted memory, plan, Harness,
  Continuity, proof, trace, decisions, and current repository state.
- [x] Use explicit hot, warm, and cold context tiers: current task and next
  action first, supporting decisions/evidence on demand, and older raw history
  only through deliberate progressive recall.
- [x] Reject or clearly mark a brief whose project ID, source revision, plan
  revision, or evidence freshness no longer matches the working repository.
- [x] Keep checkpoint writes crash-safe and deduplicated when several adapters
  observe the same meaningful event.
- [x] Verify equivalent Resume Brief contracts for Codex, Claude, and the
  generic adapter without forcing the user to launch through Baron.
- [x] Measure actual context reduction and resume correctness against Phase 53,
  including abrupt termination and mid-change interruption cases.

Phase 56 exit gates:

- [x] A fresh supported agent identifies the correct current work, last safe
  step, blocker, proof state, and next action without loading the full Vault.
- [x] The Resume Brief stays within the frozen token budget and expands history
  only when the task needs it.
- [x] Abrupt interruption recovery is deterministic, preserves evidence, and
  never invents completion.
- [x] Missed-hook reconciliation works and remains visibly different from proof
  that a native hook actually ran.

### Phase 57 - Unified Project Knowledge Assets

Status: `completed`; implementation and exit gates passed.

Goal: let agents query Memory, Decisions, Wiki, CodeGraph, checkpoints, proof,
and relevant skill metadata through one bounded knowledge view while each asset
keeps its correct owner and storage semantics.

Planned work:

- [x] Define a typed asset registry or unified query contract; do not merge all
  knowledge into one opaque database or move durable memory out of the Vault.
- [x] Preserve ownership explicitly: Vault owns durable memory, decision files
  own approved decisions, repository docs own Wiki sources, source/cache owns
  CodeGraph facts, and Baron core assets own skills and agents.
- [x] Give queryable assets project ID, stable ID, type, source, version/hash,
  lifecycle state, freshness, sensitivity, and provenance fields.
- [x] Represent bounded typed relationships such as decision-to-document,
  document-to-symbol, symbol-to-file, checkpoint-to-proof, and task-to-skill.
- [x] Keep links advisory: a relationship or model-generated summary cannot
  promote an untrusted record into a trusted fact.
- [x] Add lazy task routing so normal prompts do not recursively load every
  memory layer, document, graph node, skill, agent, or evidence record.
- [x] Handle source rename, deletion, replacement, stale revision, and broken
  links without retaining silent ghost knowledge.
- [x] Compile adapter-specific output under one shared content and token-budget
  contract.

Phase 57 exit gates:

- [x] One bounded project query can return the relevant current memory,
  decision, document, code relationship, checkpoint, and proof references with
  their distinct provenance intact.
- [x] No asset type becomes a second durable memory truth or second workflow
  owner.
- [x] Renamed, deleted, stale, and conflicting assets are visible and cannot be
  silently presented as current.
- [x] Lazy routing prevents recursive full-asset loading in all three adapters.

### Phase 58 - Incremental Wiki Intelligence

Status: `completed`; implementation and exit gates passed.

Goal: turn existing project documentation into a searchable, cited, incremental
knowledge layer without copying the entire documentation tree into every agent
prompt.

Planned work:

- [x] Index approved repository sources such as README, architecture, roadmap,
  specification, decision, runbook, and relevant operational documentation.
- [x] Use heading-aware chunks that retain document path, heading path, source
  revision, content hash, and exact citation boundaries.
- [x] Reindex only changed, added, renamed, or deleted documents and prove that
  unchanged Markdown is not rewritten.
- [x] Build bounded links and backlinks among documentation, approved
  decisions, current plans, proof, and source symbols where evidence exists.
- [x] Exclude generated output, vendor trees, temporary directories, binary
  blobs, ignored secrets, and user-designated private paths by default.
- [x] Treat indexed documentation as untrusted data until its ownership and
  lifecycle say otherwise; embedded instructions must not override Baron or
  adapter policy.
- [x] Retrieve only task-relevant sections with citations and an explicit stale
  indicator when the source revision changed after indexing.
- [x] Test very large documents, old repositories, mixed Vietnamese/English
  headings, duplicate headings, renamed files, deletions, and broken links.

Phase 58 exit gates:

- [x] A fresh agent can answer project-architecture and current-decision
  questions from small cited excerpts without scanning the docs directory.
- [x] Incremental update touches only the derived records affected by source
  changes and never rewrites user documentation.
- [x] Deleted or stale document knowledge cannot masquerade as current.
- [x] Wiki retrieval remains bounded and project-isolated on the Phase 53 large
  and same-name fixtures.

### Phase 59 - Local CodeGraph Intelligence

Status: `completed`; implementation and exit gates passed.

Goal: strengthen Baron's existing optional local code map into revision-aware,
incremental source relationships that help agents understand impact and resume
work, while remaining advisory and disposable.

Planned work:

- [x] Extend the current project-scoped code-map contract instead of creating a
  global graph, remote graph service, or second memory owner.
- [x] Key every graph by project ID, exact repository revision or working-state
  fingerprint, parser/provider version, and graph schema version.
- [x] Capture bounded modules, files, symbols, imports, dependencies, calls, and
  references where reliable parsers exist, with a documented generic fallback
  for unsupported languages.
- [x] Prioritize Baron's primary coding ecosystems, including Rust,
  TypeScript/JavaScript, Python, and Go, while marking partial or heuristic
  coverage honestly.
- [x] Update changed-file subgraphs incrementally and remove nodes/edges for
  renamed or deleted source without forcing a full repository rebuild.
- [x] Support bounded impact questions that connect changed symbols to callers,
  tests, documentation, decisions, and current checkpoints.
- [x] Distinguish parser-observed, provider-reported, and inferred edges; verify
  current source before any graph result supports durable memory or proof.
- [x] Apply strict result and token budgets and never serialize an entire graph
  into agent context.
- [x] Fall back to the Survey Engine when graph support is absent, stale,
  corrupt, incompatible, oversized, or timed out.
- [x] Keep graph data outside Vault and prove it can be deleted and rebuilt from
  current source.

Phase 59 exit gates:

- [x] Impact and resume queries improve the agreed Phase 53 cases without
  exceeding the frozen context budget.
- [x] Incremental add/change/rename/delete behavior returns the same current
  result as a clean rebuild.
- [x] A missing or failed graph never blocks normal Baron context or causes a
  proof claim.
- [x] Same-name repositories remain isolated and no graph cache enters durable
  Vault memory.

### Phase 60 - Secure Memory Boundary

Status: `completed`; implementation and exit gates passed.

Goal: protect the new memory, Wiki, CodeGraph, cache, session, and adapter paths
from cross-project leakage, secret persistence, tampering, prompt injection, and
memory poisoning.

Planned work:

- [x] Update the threat model for Vault input, session import, repository docs,
  source files, derived summaries, Wiki, CodeGraph, local caches, adapters,
  backup/restore, and optional providers.
- [x] Enforce project identity and allowed-path checks before retrieval,
  ranking, linking, context compilation, export, and proof use.
- [x] Add trust and sensitivity labels plus quarantine paths for prompt
  injection, suspicious imported instructions, poisoned summaries, and records
  that conflict with stronger current evidence.
- [x] Redact credentials, tokens, private keys, connection strings, sensitive
  personal data, and configured secret patterns before persistence, context,
  logs, benchmark artifacts, and proof receipts.
- [x] Add provenance and integrity hashes so modified durable evidence or stale
  derived caches are detected and revalidated rather than silently trusted.
- [x] Resolve canonical paths and protect against traversal, symlink/junction
  escape, aliasing, case-folding, and same-name project confusion on supported
  platforms.
- [x] Ensure retrieved documents, sessions, code comments, malware strings, and
  external skill text are treated as data and cannot execute instructions or
  change Baron policy merely by being recalled.
- [x] Define safe local file permissions, temporary-file handling, cache
  cleanup, backup, restore, and lost/corrupt-index recovery. Any optional
  at-rest protection must preserve Markdown recovery and cannot become a hidden
  cloud dependency.
- [x] Add adversarial tests for project leakage, secret canaries, malicious
  instructions, forged proof text, tampered records, unsafe links, cache
  poisoning, and backup restore.

Phase 60 exit gates:

- [x] All same-name, traversal, symlink/junction, and mixed-adapter isolation
  tests preserve the zero-leakage requirement.
- [x] Secret canaries do not appear in unauthorized Vault records, contexts,
  reports, logs, indexes, graphs, or proof artifacts.
- [x] Untrusted imported content cannot promote itself, alter policy, invoke a
  tool, or count as execution evidence.
- [x] Tampered or conflicting knowledge fails safely with provenance retained
  and a recoverable next action.

### Phase 61 - Scale, Concurrency, Recovery, And Cost

Status: `completed`; implementation and exit gates passed.

Goal: prove Baron stays responsive and recoverable for the owner's realistic
large repositories, long-lived Vault, and simultaneous coding agents while
actually reducing context cost.

Planned work:

- [x] Use the realistic target sizes and budgets frozen in Phase 53; do not
  claim enterprise scale or invent a vanity maximum that the owner does not
  need.
- [x] Make memory, Wiki, and CodeGraph indexing incremental and content-hash
  based so ordinary changes do not rescan all history or source.
- [x] Bound CPU, RAM, disk growth, open files, recall latency, and compiled
  context independently for normal, large, and degraded-provider cases.
- [x] Make Markdown writes atomic and define safe locking, transactions, WAL or
  equivalent local coordination for derived indexes and simultaneous agents.
- [x] Test parallel reads, concurrent checkpoints, overlapping session import,
  indexing during recall, interrupted compaction, and adapter reconciliation
  without corruption or duplicate trusted records.
- [x] Kill processes at controlled write/index stages and verify restart,
  idempotent recovery, last-known-good evidence, and actionable resume packets.
- [x] Delete and corrupt disposable caches deliberately and rebuild them from
  Vault and repository sources without losing durable truth.
- [x] Verify local backup and restore across a fresh machine-style directory
  layout while preserving project identity and redaction boundaries.
- [x] Report measured index time, incremental time, recall latency, peak memory,
  disk use, context size, resume correctness, and token reduction against the
  Phase 53 baseline.
- [x] Keep the default path local and usable without a hosted service, paid API,
  mandatory daemon, or always-running background process.

Phase 61 exit gates:

- [x] All frozen realistic scale and latency targets pass on recorded hardware
  and configuration, or the phase remains incomplete with exact limits stated.
- [x] Concurrent-agent and crash tests show no durable memory loss, cross-project
  mixing, false completion, or irrecoverable index state.
- [x] Full cache deletion and corruption recover from source-of-truth data.
- [x] Measured context cost improves over Baron 3.7 without reducing required
  resume correctness or hiding retrieval failures.

### Phase 62 - Vibe Security Hardening And Detection Benchmark

Status: `completed`; implementation and exit gates passed.

Goal: keep `vibe-security-scan` as Baron's source-code AppSec owner, repair its
current workflow drift, make its read-only behavior safe, and measure whether it
finds expected problems rather than merely proving that Markdown was installed.

Planned work:

- [x] Reconcile stale references to missing steps, arguments, variables,
  language setup, and `--fresh` behavior so entry guidance and small/large
  workflows implement one coherent contract.
- [x] Resolve the current bounded-chunk contradiction and specify deterministic
  sequential scoping without creating a second orchestrator or recursively
  spawning uncontrolled reviewers.
- [x] Make review read-only by default. Do not create report directories,
  temporary trees, or `.gitignore` edits unless the user explicitly asks for a
  persistent artifact.
- [x] Replace raw or non-portable cleanup instructions with resolved,
  workspace-scoped, Windows-safe and Unix-safe lifecycle behavior.
- [x] Extend asset auditing across the complete skill subtree, including
  workflows, references, rules, links, examples, and potentially dangerous
  instructions, rather than checking only the entry `SKILL.md`.
- [x] Refine the fixed taxonomy where evidence requires distinct coverage such
  as OAuth/OIDC, cookies/sessions, cryptography, prompt injection/AI boundaries,
  and sensitive data at rest; do not force findings into a misleading nearest
  category.
- [x] Load only relevant rules and language overlays under an explicit token
  budget instead of placing the complete rule library into every scan context.
- [x] Route optional tools such as Semgrep, CodeQL, dependency advisories, or
  ecosystem audits only through the capability registry. Never auto-install
  them, and never count detection or presence as proof that a tool ran.
- [x] Require structured execution receipts with scope, source revision,
  command/tool identity, result, and artifact hash before a scan contributes to
  proof or completion.
- [x] Build vulnerable and safe fixtures with expected findings, expected
  non-findings, severity, source-to-sink evidence, deterministic JSON, and
  recorded false-positive/false-negative results.
- [x] Add versioned provenance, source/license notes, and a reproducible asset
  manifest for the complete security-skill tree.
- [x] Keep the independent `security-auditor` gate responsible for final
  validation; `vibe-security-scan` cannot approve its own output.

Phase 62 exit gates:

- [x] Packaging, routing, read-only behavior, complete-subtree audit, and all
  three adapter contracts pass.
- [x] The detection benchmark meets the threshold frozen for the fixture set
  and records misses and false positives rather than hiding them.
- [x] Large scans remain bounded, deterministic in scope, and safe on Windows
  and Unix paths.
- [x] No instruction performs offensive live action, automatic tool install,
  global configuration mutation, or unsupported proof claims.

### Phase 63 - Optional Reverse Analysis Pack

Status: `completed`; implementation and exit gates passed.

Goal: add only the defensive reverse-engineering capabilities useful to the
owner's coding workflow, as lazy Baron-owned optional assets that complement
rather than replace `vibe-security-scan`.

Planned work:

- [x] Pin the evaluated upstream source revision and perform file-by-file
  provenance and license review before adapting any text. MIT at repository
  root must not be assumed to cover GPL, AGPL-derived, CTF, or other nested
  content.
- [x] Create at most three narrowly owned optional capabilities:
  `binary-reverse-analysis`, `apk-mobile-analysis`, and `malware-triage`, with
  final names confirmed in the later design.
- [x] Rewrite accepted guidance as self-contained Baron assets under Baron
  contracts; do not create a live clone, runtime download, or dependency on the
  external repository.
- [x] Exclude the external router, global `RULES.md`, `README_AI` auto-execution,
  authorization-by-precedent, case/timeline/journal system, tool bootstrap,
  global MCP writes, CTF/pwn, red-team, EDR bypass, Active Directory, payload,
  persistence, evasion, and offensive exploitation content.
- [x] Keep Baron Control Plane as the only router and Superpowers as the only
  workflow core; reverse assets are optional domain guidance and never
  mandatory quality gates.
- [x] Discover installed reverse tools only through capability registration and
  presence/execution checks. Missing optional tools must produce a bounded
  warning and a safe manual next step, not automatic installation.
- [x] Make static, offline, read-only analysis the default. Dynamic execution
  requires explicit per-task authorization, an isolated disposable copy or
  sandbox, clear scope, and retained execution evidence.
- [x] Preserve source/sample hashes, tool/version provenance, commands, outputs,
  and uncertainty while preventing secrets, live credentials, raw malicious
  instructions, or unbounded sample content from entering durable memory.
- [x] Define distinct routing from source AppSec and a bounded mixed case:
  source code routes to `vibe-security-scan`; binary/APK/malware artifacts route
  to the relevant reverse asset; mixed work may use both without duplicating
  ownership.
- [x] Add safe fixture and routing benchmarks for supported, unsupported,
  ambiguous, missing-tool, malicious-instruction, and mixed source/binary cases
  across Codex, Claude, and generic adapters.

Phase 63 exit gates:

- [x] Every included asset has approved provenance, compatible license,
  self-contained guidance, lazy routing, bounded output, safe fallback, and
  adapter parity.
- [x] No excluded router, offensive suite, auto-bootstrap, global mutation,
  authorization shortcut, or second case/memory lifecycle is present.
- [x] Missing tools and unsupported artifacts fail safely without blocking
  normal Baron coding or fabricating analysis evidence.
- [x] Reverse results remain advisory until current artifacts and the independent
  `security-auditor` gate validate claims used for proof.

### Phase 64 - Cross-Domain Closure, Documentation, And GitHub Backup

Status: `completed`; local implementation, native matrix, immutable Release, and
fresh public install all pass. This is the final Baron 3.8 phase and the only
phase that publishes the source.

Goal: prove the memory, Wiki, CodeGraph, safety, scale, source security, and
reverse-analysis paths work together; make the repository documentation
truthful; then back up and publicly publish the complete verified Baron 3.8
source with native installers and a fresh Windows recovery path.

Planned work:

- [x] Freeze and test the final routing matrix: normal coding loads neither
  security pack; source AppSec loads `vibe-security-scan`; binary, APK, or
  malware work loads only the matching reverse asset; mixed work uses the
  narrow applicable assets and still ends at `security-auditor` when security
  proof is required.
- [x] Prove that Memory, Wiki, CodeGraph, security skills, Product Harness,
  Continuity, Proof, Trace, Autopilot, Control Plane, Superpowers, and the three
  core agents retain one owner each and do not create recursive routing.
- [x] Run integrated project-isolation, bounded-context, adapter-preservation,
  old-repository, update/migration, crash/recovery, concurrency, scale, secret,
  injection, detection, and execution-receipt tests.
- [x] Repeat the frozen Phase 53 benchmark and publish exact before/after
  correctness, missing/stale result, leakage, latency, memory, disk, and context
  cost measurements, including regressions and hardware/configuration details.
- [x] Run the complete authorized local quality suite, including formatting,
  workspace tests, Clippy with warnings denied, locked release build, exact
  binary version, lifecycle tests, and fresh Vault/project smoke. This is local
  verification, not the excluded public certification matrix.
- [x] Update `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`,
  `notes/build-log/CURRENT.md`, the active design and executable plan, README,
  architecture, memory, adapter, command, security, provenance, migration, and
  recovery documentation wherever implementation changed their truth.
- [x] Update the root README with the public `v3.8.0` installer path, the
  source-build fallback, prerequisites, and an exact `baron 3.8.0` check.
- [x] Source truth is `3.8.0`; public download, installer, native-asset, tag, and
  `releases/latest` claims below are backed by the recorded release evidence.
- [x] The owner approved the complete Phase 53-64 implementation, including the
  final commit, GitHub push, tag, Release, README update, and install smoke.
- [x] Commit the exact reviewed Baron 3.8 source and documentation intentionally,
  push it to the approved GitHub branch, verify the remote commit SHA and final
  file state, and confirm the local branch is clean and synchronized.
- [x] Record the GitHub repository, branch, final remote SHA, verification
  results, README install distinction, and any deliberately excluded release
  work in the final status and handoff evidence.
- [x] If documentation sync, version truth, local gates, commit, push, remote
  verification, or clean synchronization fails, keep Phase 64 incomplete and
  preserve the exact failure, last successful step, affected SHA/files, safe
  next action, and retry condition.

Phase 64 exit gates:

- [x] Phase 53-63 are checked complete with their evidence, and the integrated
  routing/ownership matrix has no unresolved conflict.
- [x] The final benchmark proves the accepted resume-quality and context-cost
  targets without cross-project leakage or hidden fallback failure.
- [x] All required local formatting, test, Clippy, locked build, lifecycle, and
  fresh-project smoke gates pass from the exact source to be pushed.
- [x] Source, lockfile, binary version, README, architecture, status Markdown,
  status JSON, active plan/design, build log, provenance, and recovery guidance
  agree truthfully on Baron `3.8.0` source state.
- [x] The approved GitHub branch contains the exact verified commit, remote and
  local SHA/state agree, and no authorized change remains unpushed.
- [x] Public `releases/latest` resolves to `v3.8.0`; the tag, GitHub Release,
  native assets, installer, and certification are recorded and verified.

Baron 3.8 completion record:

- The owner approved the complete Phase 53-64 program and explicitly requested
  implementation, GitHub publication, README synchronization, and a downloadable
  `v3.8.0` release.
- Every phase and exit gate is checked only after its implementation evidence and
  verification result were recorded.
- The next action after the published release is normal maintenance; no Phase
  53-64 task remains open.
## Baron 3.8 Final Evidence

- Phase 53-56: bounded Resume Brief, layered memory classification, redacted
  persistence, project-filtered hybrid recall, and benchmark output are present;
  knowledge unit tests and CLI smoke passed.
- Phase 57-59: heading-aware Wiki index/search and local revision-bound CodeGraph
  index/query are disposable, project-isolated, and documented in the command
  surface and memory model.
- Phase 60-62: project identity checks, secret redaction, capability-gated
  optional tools, read-only `vibe-security-scan` workflows, and safe recovery
  guidance are covered by code/tests and the complete asset audit.
- Phase 63: the three narrow reverse-analysis packs are Baron-owned, lazy,
  offline, defensive guidance; `vibe-security-scan` remains the source-AppSec
  owner and the three core quality agents remain mandatory gates.
- Phase 64: formatting, Clippy, focused core/adapter/control-plane tests, locked
  release build, exact `baron 3.8.0` version, status/JSON validation, GitHub
  source synchronization, immutable `v3.8.0` tag/Release, native assets,
  checksums, and fresh `releases/latest` Windows install smoke are recorded in
  `docs/assessment/baron-3.8-certification.md` and `docs/RELEASE.md`.

Phase 28-31 final verification:

- all nine platform profile fixtures: passed
- Codex, Claude, and generic adapter automation contracts: passed
- fullstack-to-mobile extension with legacy/custom preservation: passed
- reviewer finding and evidence-backed closure: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- installer lifecycle and release metadata at 3.2.0: passed
- real temp Codex/fullstack-to-mobile shared Vault smoke: passed
- binary GitHub Release publication: not performed; remains explicit

Phase 25-26 final verification:

- duplicate imported-session record RED/GREEN regression: passed
- real shared-Vault index and Markdown hash preservation: passed
- real `scanjob` Codex/fullstack init: passed
- installer no-API RED/GREEN contract: passed
- manifest-based latest install during API rate limiting: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- GitHub main CI `28797881851`: passed
- GitHub release workflow `28797886356`: passed
- real machine update from Baron 3.1.2 to 3.1.4: passed

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
- Phase 25 plan: `docs/superpowers/plans/2026-07-06-memory-index-duplicate-record-fix.md`
- Phase 25 build log: `notes/build-log/2026-07-06-memory-index-duplicate-record-fix.md`
- Phase 26 plan: `docs/superpowers/plans/2026-07-06-api-independent-latest-installer.md`
- Phase 26 build log: `notes/build-log/2026-07-06-api-independent-latest-installer.md`
- Public demo: `docs/demo/README.md`
- Public certification: `docs/assessment/baron-3-public-certification.md`
- 3.1.4 installer certification: `docs/assessment/baron-3.1.4-installer-resilience.md`
- Baron 3.4 design: `docs/superpowers/specs/2026-07-23-baron-3-4-safe-self-update-design.md`
- Baron 3.4-3.6 controlled extension design: `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`
- Baron 3.4-3.6 master program: `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`
- Phase 35-38 plan: `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
- Phase 35-38 build log: `notes/build-log/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
- Phase 39-41 plan: `docs/superpowers/plans/2026-07-24-phase-39-41-baron-3-5-skill-intelligence.md`
- Phase 42-45 plan: `docs/superpowers/plans/2026-07-24-phase-42-45-baron-3-6-code-graph.md`
- Baron 3.4-3.6 planning log: `notes/build-log/2026-07-24-baron-3-4-to-3-6-program.md`
- Temporary build note: `notes/build-log/CURRENT.md`

## Current Rule

Baron `3.6.0` is the stable source baseline. Phases 42-45 completed in strict
release order: provider-neutral local cache, exact local-only adapter, bounded
source verification, then isolation/scale certification. Graphify remains
optional: it cannot install anything, change hooks/instructions, write Vault
memory, create global state, or block Baron Survey. Each release still needs
fresh certification and an independent version gate. A binary GitHub Release
remains an explicit promotion step, not an automatic side effect. The engine
keeps the simple user command flow, Vault data safety, Superpowers workflow
ownership, the three core quality gates, bounded context, and evidence-backed
completion.

## Baron 3.6 Final Verification

- same-name project graph/cache isolation: passed
- Vault memory exclusion and cache deletion isolation: passed
- 6,100+ mixed-language legacy repository survey/context bound: passed
- missing, incompatible, failed, stale, malformed, timed-out, and oversized
  provider fallback: passed
- hook/instruction/root-output/home-location preservation: passed
- full workspace tests, Clippy, locked release build, release-binary version,
  and diff validation: passed
- source certification: `docs/assessment/baron-3.6.0-code-graph-certification.md`
- source-certification checkpoint: tag/Release had not yet been created
- final public promotion: immutable tag and GitHub Release `v3.6.0` created by
  Actions run `30246729740`; public installer smoke passed

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

Public Trust 3.1.2 final verification:

- README public flow validation: passed by `cargo test -p baron-core --test public_trust_docs`
- demo and certification docs validation: passed by `cargo test -p baron-core --test public_trust_docs`
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- `git diff --check`: passed
- GitHub release latest smoke: passed; `releases/latest` points to `v3.1.2`, release workflow `27878352377` passed, main CI `27878348144` passed, and Windows same-terminal install/setup/init/context smoke from latest passed
