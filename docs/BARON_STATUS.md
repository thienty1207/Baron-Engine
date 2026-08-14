# Baron Build Status

Last updated: 2026-08-14

## Overall

- Stable source release: `v4.2.0` ([public Release](https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.0))
- Latest downloadable release: [`v4.2.0`](https://github.com/thienty1207/Baron-Engine/releases/latest)
- Candidate source version: `4.2.0`; immutable public Release and fresh public
  installer/rollback smoke have passed
- Baron 2.0 completion: 100%
- Baron 3.0 completion: 100%
- Baron 3.2 completion: 100%
- Baron 3.3 completion: 100%
- Target source release: `v4.2.0`; source, tests, acceptance artifacts, native
  assets, checksums, and public install evidence agree
- Baron 4.2 program target release: `v4.2.0`; Phases 88-100 are complete and the
  next action is normal maintenance
- Baron 3.4 completion: 100%
- Baron 3.5 completion: 100%
- Baron 3.6 completion: 100%
- Baron 3.7 completion: 100%; Phases 46-52 publicly certified
- Baron 3.8 completion: 100%; Phases 53-64 publicly certified
- Baron 4.0 release completion: 100%; the sixteen-case four-surface benchmark,
  integrated acceptance, native matrix, immutable Release, and fresh public
  install are all verified. Broader real-repository scale and durable temporal
  compaction remain explicitly documented follow-up limits.
- Planned Baron 4.0 phases: 12; the public release phase is complete and the
  remaining research/scale items are tracked as non-blocking follow-ups
- Baron 4.1 target: `v4.1.0`; eleven phases (`77-87`) delivered the owner-
  approved local intelligence scope. Tencent comparison is retained as optional
  reference material, not a release gate.
- Baron 4.1 completion: 100% for the owner-approved release scope; broader
  real-corpus and parser-depth work remains a clearly labelled non-blocking
  follow-up.
- Baron 4.2 program: thirteen phases (`88-100`), all complete. Phase 100 is the
  verified public release boundary and is no longer open.
- Current work state: source version `4.2.0`, default guarded generation `4.2`,
  explicit whole-engine `4.1` rollback, and per-query `4.0` fallback are ready;
  public `releases/latest` resolves to `4.2.0`.
- Current next action: normal maintenance. The exact-source native CI/release
  publication, fresh README-only install, checksum verification, 4.0 fallback,
  and 4.1 rollback smoke are complete.
- Current Baron 4.1 authority: owner approval on 2026-08-13 plus the active
  design/plan; public release and README promotion are complete.
- For the completed Baron 3.7 program, the owner explicitly approved
  implementation and final GitHub publication on 2026-08-12. Its immutable
  release and fresh public `releases/latest` install smoke have both passed.
- Build confidence: Baron 4.1 Baron-only acceptance is `100/100` on all five
  local intelligence surfaces. CI run
  [`31723285579`](https://github.com/thienty1207/Baron-Engine/actions/runs/31723285579)
  passed format/Clippy, the full workspace tests, and the native matrix on
  Windows x64, Linux x64, macOS Intel, and Apple Silicon. Release run
  [`31723297751`](https://github.com/thienty1207/Baron-Engine/actions/runs/31723297751)
  passed exact-source verification, checksums, `release-manifest.json`,
  installer lifecycle smoke, and immutable Release promotion from source
  `6bea181044fa0d6f4a74195b8c7455eaa09fdf62`. A public `install.ps1` smoke
  returned `baron 4.1.0`, then passed survey, setup, Codex/fullstack init,
  memory index/recall, and context in an isolated directory without changing
  the user PATH. Baron 4.0 remains the explicit fallback.
  The optional local code map remains identity-bound and project-local, stays
  outside Vault memory, preserves agent instructions and hooks, remains bounded
  on old/large repositories, and falls back to Survey on every absence or
  failure.
- Baron 4.2 local release-profile acceptance is also green: the raw development
  fixture scored `100/100` in three repeated runs and the executable private
  holdout scored `100/100` across eight cases. Contract
  `432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a`, source
  revision `545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d`,
  and acceptance evidence are recorded in
  `docs/assessment/baron-4.2-acceptance.{json,md}`. This evidence is bounded to
  the private local contract; it does not disclose or claim access to the
  owner's raw sessions.
- Baron 4.2 public release evidence: source/tag commit
  `af42a2d3fcf37f315c6a24c5cebbef59ee6a4bc0`; CI
  [`31771633229`](https://github.com/thienty1207/Baron-Engine/actions/runs/31771633229)
  and Release
  [`31771646989`](https://github.com/thienty1207/Baron-Engine/actions/runs/31771646989)
  passed Windows x64, Linux x64, Intel macOS, and Apple Silicon jobs, exact-source
  verification, Clippy, checksums, manifest, installers, and lifecycle gates.
  The immutable Release is
  [`v4.2.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.0)
  with eight native archive/update assets plus `install.ps1`, `install.sh`,
  `release-manifest.json`, and `SHA256SUMS`. A fresh `releases/latest` install
  reported `baron 4.2.0`; an independent 4.1-to-4.2-to-4.1 rollback smoke forced
  4.0, restored 4.1, and preserved project/Vault sentinel hashes.

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

## Baron 4.0 Planning And Approval Gate

Status: `public-release-complete`; the owner approved Baron 4.0 development,
testing, GitHub publication, and README synchronization on 2026-08-13.

- [x] Record the owner's required target: memory search and synthesis, Wiki,
  and the default local CodeGraph must each earn at least `90/100` under a
  frozen, evidence-backed score contract before Baron 4.0 may be released.
- [x] Define the complete proposed program as twelve phases, Phase 65 through
  Phase 76, with public GitHub publication and reinstall verification only in
  the final phase.
- [x] Keep the initial planning batch limited to `docs/BARON_STATUS.md`; the
  authorized implementation batch records source/tests/design/build evidence
  separately and has prepared the public README for `4.0.0`.
- [x] The owner has reviewed and explicitly approved the exact Phase 65-76
  scope for implementation on 2026-08-13.
- [x] Before Phase 65 implementation started, the active Baron 4.0 design and
  executable plan, status JSON, build log, and Continuity checkpoint were
  recorded; source stayed at `3.8.0` until the acceptance gate passed.
- [x] Phase 76 is the separate publication phase carrying the program through
  version `4.0.0`, README synchronization, intentional commit, push, native
  GitHub build matrix, immutable tag and Release, installer assets,
  `releases/latest`, and a fresh Windows recovery-install smoke.

Release hard stop (closed for the public release scope):

- The clean 16-case report and integrated acceptance are evidence for the local
  4.0 candidate: Memory/Wiki/CodeGraph/Security are each 100/100, leakage is
  zero, security routing is 9/9, static scan is read-only with 0 findings, and
  the bounded handoff is project-grounded. Native GitHub builds, checksums,
  immutable Release, and fresh public Windows install all passed; `4.0.0` is
  now stable/latest. Broader real-corpus scale, full temporal compaction, and
  dynamic lab execution remain documented follow-up limits rather than hidden
  release claims.

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

## Proposed Baron 4.0 Direction

Baron 4.0 is the measurable-intelligence release. It strengthens the useful
memory ideas studied from TencentDB Agent Memory without copying its server,
team, UI, database-ownership, or cloud-service architecture. The release is
allowed to call its memory, Wiki, or default CodeGraph `9/10` only when each
surface independently earns at least `90/100` on the frozen Baron benchmark.
One strong score cannot hide another surface below 90.

The proposed release has five owner-visible outcomes:

- a newly opened supported AI finds the correct current project knowledge even
  when the question uses different words, Vietnamese/English wording, old
  names, or code identifiers
- Baron turns bounded raw evidence into useful facts, outcomes, decisions, and
  invariants while keeping trust, conflict, supersession, and source evidence
  explicit
- Wiki answers traverse real document, decision, and source relationships and
  return exact citations instead of matching only a token substring
- the default local CodeGraph understands supported-language syntax, imports,
  definitions, references, calls, and impact paths without requiring Graphify,
  an LSP server, a cloud API, or a paid account
- a compact Resume Brief gives Codex, Claude, or a generic agent the same
  grounded work state while reducing context cost and never mixing projects

The release does not add a GUI, team accounts, hosted memory service, proxy,
Docker cluster, enterprise control plane, mandatory daemon, or offensive
reverse tooling. It does not replace Vault Markdown, Superpowers, Baron Control
Plane, Product Harness, Continuity, Proof, Trace, or the three quality agents.
The optional reverse-analysis packs and `vibe-security-scan` remain separate
security capabilities; they may contribute verified evidence but cannot own or
rewrite the memory intelligence pipeline.

Baron 4.0 preserves the following boundaries:

- Vault Markdown remains durable truth. Full-text indexes, embeddings, graph
  databases, and derived summaries remain disposable, identity-bound caches.
- Memory abstraction level and trust are separate facts. A polished L3 summary
  is still only a candidate until evidence or an allowed human workflow makes
  it trusted.
- Project-ID, trust, sensitivity, and eligibility filtering happens before
  lexical/vector fusion or reranking. Cross-project leakage tolerance is zero.
- The official default 9/10 path is local and requires no cloud model, paid
  embedding API, external graph provider, or Baron launcher. A missing local
  accelerator may degrade safely, but degraded output cannot retain a 9/10
  claim.
- Unsupported or dynamically unresolved code relationships remain `unknown`;
  Baron must not guess them to improve a benchmark score.
- No benchmark threshold may be weakened after implementation results are
  visible without an explicit owner-approved, documented reason.
- During implementation the source stayed `3.8.0` until the Phase 75 acceptance
  gate; Phase 76 then changed it to `4.0.0` only after the required score and
  safety evidence passed.
- Phase 76 is complete because GitHub contains the exact verified source,
  README, tag, Release, native assets, checksums, installers, and a fresh public
  Windows install that reports `baron 4.0.0`.

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

Baron 4.0 implementation and public release program; the release scope is
complete and the remaining breadth limits are documented explicitly:

| Phase | Name | Status | Baron 4.0 Weight | Exit Proof |
| --- | --- | --- | --- | --- |
| 65 | Ground-Truth 9/10 Score Contract | evidence recorded | 8% | sixteen frozen cases, independent baseline/candidate scorecards, zero leakage, clean cache/Vault run, and Windows environment metadata |
| 66 | Layered Memory Abstraction And Trust | evidence recorded | 9% | L0-L3 abstraction is separate from candidate/verified/contested/superseded trust; labels survive parsing and firewall filtering |
| 67 | Consolidation, Conflict, And Supersession | evidence recorded | 8% | deterministic read-only analysis, authority explanations, and atomic reviewable candidate staging; durable promotion remains opt-in follow-up |
| 68 | Local Semantic Retrieval And Reranking | evidence recorded | 12% | local lexical/alias/ngram reranking, pre-ranking firewall, bounded explanations, and automatic 3.8 recovery fallback |
| 69 | Grounded Synthesis And Cross-Agent Handoff | evidence recorded | 9% | cited, project-grounded Resume Brief with proof, blockers, unknowns, next action, and bounded 902-character evidence |
| 70 | Linked Wiki Knowledge Graph | evidence recorded | 10% | incremental Markdown links, citations, freshness, and bounded two-hop link paths with project isolation |
| 71 | AST CodeGraph And Impact Intelligence | evidence recorded | 11% | local Rust/TypeScript/JavaScript/Python/Go symbols, imports, references/calls, source spans, and rebuildable identity-bound cache |
| 72 | Security Intelligence And AppSec Expansion | evidence recorded | 8% | defensive static AppSec boundary, vibe-security ownership, reverse-static route, redaction, and advisory 0-finding scan |
| 73 | Authorized Adversary Assessment And Lab Safety | evidence recorded | 8% | explicit authorization/scope/allowlist/cleanup contract and fail-closed offensive, scope, and project mismatch cases; dynamic execution disabled |
| 74 | Security Routing, Tool Governance, And Regression | evidence recorded | 5% | one Baron route, project scope and allowlist checks, nine-case deterministic regression, and fail-closed unsafe routing |
| 75 | Integrated Security, Scale, Cost, And 9/10 Acceptance | completed with limits | 7% | local integrated acceptance is 100/100 with four independent 100/100 surfaces; native/public proof passed while large-corpus, temporal, and full parity breadth remain follow-ups |
| 76 | Baron 4.0 Public GitHub Release And Recovery Install | completed | 5% | exact source `041564d…`, native matrix, immutable `v4.0.0` Release, checksums/manifest, and fresh Windows reinstall are verified |

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
- [x] Push verified source only after the complete 3.4-3.6 program; keep tag/GitHub Release promotion explicit.

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

## Baron 4.0 Program - Approved Release Record

> The owner approved the twelve-phase program on 2026-08-13. The public release
> scope is complete. Unchecked items below are explicit real-corpus, temporal,
> or dynamic-lab follow-ups and are not silently counted as shipped behavior.

Checkbox reconciliation (2026-08-13): every `[x]` in this release record is
backed by the recorded 4.0 benchmark, security regression, release smoke, or
source/test evidence. The unchecked rows are intentionally retained where the
evidence is only partial or belongs to the later 4.1 candidate (for example
real-repository breadth, full temporal compaction, third-party AST parsing,
dynamic labs, concurrency/corruption recovery, or complete adapter parity).
The reconciliation is part of the release record; it is not a claim that those
follow-ups shipped in `v4.0.0`.

Baron 4.0 has one non-negotiable outcome: memory search and synthesis, Wiki,
and the default local CodeGraph must each independently earn at least `90/100`.
The release must make a fresh coding agent materially more accurate and cheaper
to resume without sacrificing Baron's project isolation, evidence boundary,
local-first operation, or human authority.

The proposed program has twelve phases in five groups:

| Group | Phases | Outcome |
| --- | --- | --- |
| Score contract and durable intelligence | 65-67 | Freeze honest measurements, add L0-L3 abstraction, and consolidate duplicate, stale, and conflicting memory without promoting guesses. |
| Retrieval and cross-agent synthesis | 68-69 | Add real local semantic retrieval, fusion, reranking, grounded answer packets, and compact equivalent handoff across adapters. |
| Wiki and CodeGraph 9/10 | 70-71 | Build a linked cited Wiki and a default AST-based CodeGraph with measured multi-hop and impact accuracy. |
| Security intelligence and authorized assessment | 72-74 | Expand defensive AppSec, reverse analysis, and scoped adversary assessment with explicit authorization, tool governance, evidence, and regression gates. |
| Integrated acceptance and public recovery release | 75-76 | Prove memory, Wiki, CodeGraph, and security quality survive security, scale, cost, and adapter gates, then publish and reinstall Baron 4.0 from GitHub. |

Program non-goals:

- No GUI, dashboard, visual browser, team workspace, account system, hosted
  proxy, multi-tenant server, Docker stack, or enterprise RBAC.
- No replacement of Vault Markdown with SQLite, a vector database, a graph
  database, generated summaries, or an external memory service.
- No mandatory cloud embedding API, paid model, remote reranker, Graphify,
  LSP server, or always-running daemon in the default 9/10 path.
- No unrestricted offensive automation, third-party targeting, credential
  theft, persistence, evasion, payload delivery, exploit weaponization, or
  autonomous Internet scanning. Baron may assist only with explicitly scoped,
  authorized, isolated security assessment and defensive validation.
- No second workflow core, memory owner, context compiler, router, proof
  system, quality-agent set, Wiki truth, or CodeGraph truth.
- No automatic promotion of repeated statements, model summaries, imported
  sessions, Wiki text, or CodeGraph inference into trusted durable facts.
- No expansion of offensive reverse-analysis scope. Baron 4.0 memory work must
  not weaken the static, defensive, authorization, and evidence boundaries
  completed in Baron 3.8.
- No version bump, README 4.0 download claim, GitHub tag, Release, or installer
  promotion before Phase 65-75 pass and the owner authorizes Phase 76 execution.

Program-wide 9/10 score contract:

- [ ] Freeze the evaluation corpus, expected answers, metric formulas, weights,
  hardware/configuration record, and pass thresholds before implementing the
  intelligence changes. Keep the original corpus and report any later additions
  separately so implementation cannot train against and replace the test.
- [ ] Use deterministic synthetic fixtures plus pinned, representative real
  repositories and Vault histories. Cover new and old projects, monorepos,
  interruptions, renamed symbols/files, bilingual questions, stale decisions,
  contradictory evidence, duplicate names, dynamic code, and poisoned text.
- [x] Score memory intelligence, Wiki, and CodeGraph independently on the
  recorded 100-point release rubric. Each release-corpus total is at least 90;
  no cross-surface averaging, discretionary bonus, or release-time rounding
  turns a lower score into 90. The broader 4.1 five-surface rubric remains open.
- [ ] Require memory Recall@5, MRR, grounded synthesis, conflict handling,
  Resume Brief completeness, provenance, and token efficiency to meet the
  frozen per-metric floors as well as the total memory score of at least 90.
- [ ] Require Wiki retrieval, exact citations, linked/multi-hop answers,
  groundedness, freshness/deletion behavior, and context cost to meet their
  frozen floors as well as the total Wiki score of at least 90.
- [ ] Require CodeGraph symbol, definition/reference, import, call, typed edge,
  impact path, source citation, freshness, and incremental performance metrics
  to meet their frozen floors as well as the total CodeGraph score of at least
  90 on the officially supported language set.
- [ ] Require the security score to cover source/AppSec detection, API and
  identity authorization checks, dependency/supply-chain findings,
  reverse-analysis evidence, threat/attack-path quality, severity calibration,
  remediation retest, false-positive/false-negative rate, tool provenance, and
  authorized-assessment safety. Its total must independently reach 90.
- [x] Keep hard gates outside the weighted score: zero cross-project leakage,
  no secret disclosure in returned context, no unqualified contested/stale fact
  in critical cases, no fabricated citation or graph edge counted as verified,
  and no cache/database becoming durable truth.
- [x] Measure the normal local default path. Optional Graphify, an LSP, a cloud
  API, a paid provider, or manually preloaded context may be reported as a
  separate enrichment result but cannot supply the official 9/10 score.
- [x] Record exact failures and unknowns. If any surface scores below 90 or any
  hard gate fails, Phase 75 and Phase 76 remain incomplete and documentation
  must describe Baron 4.0 as unreleased.

### Phase 65 - Ground-Truth 9/10 Score Contract

Status: `evidence recorded`; local 4.0 source is versioned `4.0.0`; the public
release is now stable, with broader corpus expansion retained as follow-up.

Goal: replace subjective scoring with a frozen, reproducible acceptance system
that exposes the real Baron 3.8 gaps before any Baron 4.0 optimization begins.

Planned work:

- [x] Define the exact user-level questions covered by the sixteen-case release
  corpus: what is being built, where work stopped, which decision is current,
  which evidence is valid, what code/docs are involved, what remains unknown,
  and what the next safe action is. Broader real-history questions remain a
  follow-up rather than being silently counted here.
- [x] Build a versioned sixteen-case release corpus for memory, Wiki, and
  CodeGraph with machine-readable expected signals, citations, symbols,
  relations, trust boundaries, and project IDs. The larger pinned real-repo
  corpus remains explicitly open below.
- [ ] Include adversarial cases: same folder basename with different IDs,
  old-but-high-frequency claims, newer weak claims versus older verified
  decisions, repeated imported sessions, ambiguous identifiers, renamed and
  deleted files, prompt injection in docs, and cache tampering.
- [x] Include Vietnamese, English, mixed-language, paraphrase, identifier,
  filename, and natural-language queries in the sixteen-case release corpus
  without duplicating the expected truth into query-specific records. Broader
  acronym and multilingual coverage remains part of the real-repository
  expansion below.
- [ ] Pin representative real repositories and revisions for Rust,
  TypeScript/JavaScript, Python, and Go, with a documented license and fixture
  preparation method. Synthetic-only proof is insufficient for a 9/10 claim.
- [ ] Define memory score weights and floors for Recall@5, MRR or nDCG,
  factual/grounded synthesis, conflict/supersession accuracy, Resume Brief
  completeness, provenance, stale-answer control, latency, and token cost.
- [ ] Define Wiki score weights and floors for section retrieval, citation
  boundary accuracy, answer grounding, link/entity traversal, multi-hop
  questions, freshness/deletion, latency, and bounded context.
- [ ] Define CodeGraph score weights and floors for symbols, imports,
  definitions/references, calls, typed relations, impact reachability,
  source-line evidence, incremental freshness, latency, memory, and disk.
- [x] Capture the exact Baron 3.8 baseline for every case in the release
  report, including per-case misses, unsupported behavior, heuristic output,
  elapsed time, and estimated context cost. Full false-positive and resource
  profiling for the larger corpus remains an explicit scale follow-up.
- [x] Produce stable JSON and human-readable reports with source fingerprint,
  fixture revision, raw results, metric calculations, and pass/fail reasons.
  The report records Windows/architecture/CPU/profile metadata.
- [x] Run the sixteen-case suite from a clean disposable cache and a fresh
  project Vault path; the report records that cache was rebuilt and that the
  Vault path was not used as an implicit cache. A full rebuilt real-repository
  corpus remains a scale follow-up.
- [x] Record the approved benchmark and its fingerprint in the active
  Baron 4.0 design, executable plan, status JSON, build log, and Continuity
  checkpoint before Phase 66 begins; report: `docs/assessment/baron-4.0-benchmark.*`.

Phase 65 exit gates:

- [x] All four scorecards repeat from a clean cache with equivalent 100/100
  candidate results across sixteen cases and zero leakage; the report shows
  the truthful Baron 3.8 baseline separately from the 4.0 candidate. Broader
  real-repository and adversarial scale coverage remains documented for the
  next benchmark expansion.
- [x] Every total, metric floor, hard gate, corpus revision, and cost budget is
  frozen before feature implementation and can be recalculated from raw data.
- [x] Zero cross-project leakage is an unconditional hard gate, not a weighted
  metric that other points can offset.
- [x] Baseline and candidate implementations remain separate in the report;
  later intelligence improvements are labeled `4.0` and never replace the 3.8
  baseline without the comparison gate.

Current Phase 65 evidence (2026-08-13):

- `docs/assessment/baron-4.0-benchmark.json` and `.md` contain sixteen cases
  across four independent surfaces from a clean cache and fresh Vault path.
  Candidate 4.0 scores are Memory 100, Wiki 100, CodeGraph 100, Security
  100; the separate 3.8 baseline scores Memory 100, Wiki 88, CodeGraph 60,
  Security 100. Cross-project leakage is `0`. Report id
  `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`, source
  fingerprint `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`.
- `docs/assessment/baron-4.0-security-regression.json` and `.md` contain nine
  routing/safety cases and score 100/100, including missing authorization,
  project mismatch, allowlist mismatch, unsafe offensive intent, and mixed
  routing. This is security-route evidence, not a claim that all AppSec domains
  or dynamic lab execution are complete.
- `baron context` selects guarded 4.0 by default after the local gate. Setting
  `BARON_ENGINE_GENERATION=3.8` or `baseline` forces the recovery path; an
  unknown value fails closed. Structural checks still fall back to 3.8. The
  source is now `4.0.0`; public latest is `v4.0.0` after Phase 76 promotion.

### Phase 66 - Layered Memory Abstraction And Trust

Status: `evidence recorded`; migration and full rebuild of legacy Vaults remain
explicit follow-up work.

Goal: let Baron represent what happened at several useful levels while keeping
the abstraction level separate from whether a record is trusted.

Planned work:

- [x] Define Baron-native abstraction levels: L0 bounded raw evidence; L1
  atomic facts/events; L2 tasks, decisions, scenarios, and outcomes; L3 durable
  project invariants, operating patterns, and current direction.
- [x] Preserve Baron's independent trust axis for every level, including at
  least `candidate`, `verified`, `contested`, `superseded`, `expired`, and
  `unknown`. L3 is not automatically more trusted than L0.
- [ ] Give every record project ID, stable record ID, abstraction level, type,
  trust state, sensitivity, source/evidence IDs, source revision, valid-time
  interval, observed timestamp, content hash, producer, and schema version.
- [ ] Store durable human-readable records in the project Vault capsule and
  keep FTS, embeddings, graph links, extraction queues, and derived views as
  disposable accelerators that rebuild from authorized source.
- [x] Define evidence-based promotion: raw/session/model output enters as a
  candidate; repository evidence, current proof, an approved decision, or an
  allowed human action is required for verified durable use.
- [ ] Keep the original bounded evidence linked to every derived fact, outcome,
  decision, or invariant so an agent can inspect why it exists and detect when
  its source revision changed.
- [ ] Add schema migration for existing Baron 3.8 memory with dry-run,
  inventory, backup, non-destructive conversion, validation, rollback, and
  full cache rebuild. Existing Markdown must remain readable throughout.
- [ ] Define sensitivity and retention behavior for session excerpts, secrets,
  personal data, malicious instructions, and generated summaries before they
  enter durable memory or a local semantic index.
- [x] Integrate the candidate abstraction/trust labels with the current Memory Firewall,
  Continuity, Decisions, Proof, Trace, and session import owners instead of
  creating a parallel memory lifecycle.
- [ ] Add cross-adapter contract tests proving Codex, Claude, and generic agents
  interpret abstraction, trust, source, freshness, and unknown labels the same
  way without recursively loading all levels.

Phase 66 exit gates:

- [ ] Cache deletion and clean rebuild return equivalent trusted L0-L3 records
  from Vault/source evidence with stable identities and no project crossover.
- [x] A candidate or polished L3 summary cannot appear as an unqualified fact,
  approve itself, or override stronger verified evidence.
- [ ] Existing Baron 3.8 Vaults migrate and roll back without losing or
  rewriting original evidence, user-owned Markdown, decisions, or sessions.
- [ ] Layering improves the Phase 65 memory cases without exceeding the frozen
  context budget or weakening any hard gate.

### Phase 67 - Consolidation, Conflict, And Supersession

Status: `evidence recorded`; read-only duplicate/conflict/supersession analysis,
authority explanations, and atomic reviewable staging are implemented; durable
temporal compaction and rollback remain follow-up work.

Goal: make memory cleaner and more current by merging duplicates and resolving
time-aware conflict through evidence, not frequency or model confidence.

Planned work:

- [x] Build a bounded candidate pipeline from approved sessions, checkpoints,
  decisions, proof, traces, repository changes, and user-approved notes:
  redact, classify, atomize, link evidence, detect duplicates/conflicts, and
  stage candidates before any durable promotion.
- [x] Deduplicate exact and semantic equivalents using stable identity,
  normalized project-scoped content, source lineage, and time windows while
  retaining all distinct evidence references.
- [ ] Model temporal truth explicitly so Baron can distinguish `observed_at`,
  `valid_from`, `valid_until`, replacement time, and current source revision.
  A later timestamp alone must not defeat stronger authority.
- [x] Apply deterministic conflict precedence from user-confirmed decision,
  current repository/proof, verified project memory, likely inference, stale
  reference, and unknown; record the reason rather than silently overwriting.
- [ ] Add supersession chains, contested sets, expiry, revalidation triggers,
  and tombstones for deleted sources so an older answer remains auditable but
  cannot masquerade as current.
- [ ] Add bounded compaction that creates a higher-level candidate while
  retaining original evidence and rollback. Compaction must not erase open
  blockers, failed attempts, dissenting evidence, or unknowns.
- [ ] Detect repeated low-value or near-identical session records before write,
  and prevent repeated imports or adapter restarts from increasing trust or
  ranking merely through frequency.
- [ ] Make Autopilot review show proposed merges, conflicts, supersessions,
  evidence, and expected context savings before approve/reject; no background
  path may promote core policy or architecture.
- [ ] Test concurrent imports, interrupted consolidation, partial write,
  process restart, cache corruption, source rename/delete, reverted commits,
  clock skew, duplicate adapters, and rollback.
- [ ] Measure conflict accuracy, stale-answer rate, duplicate reduction, lost
  evidence, resume correctness, processing time, disk, and token effect against
  the Phase 65 baseline.

Phase 67 exit gates:

- [ ] All frozen conflict, duplicate, stale, expiry, supersession, and rollback
  cases return the expected current truth and preserve the full audit path.
- [x] Frequency, recency, semantic similarity, or model confidence alone can
  never promote or overwrite a verified record.
- [ ] Interrupted or concurrent consolidation is atomic/idempotent and a clean
  rebuild produces the same eligible current-memory view.
- [ ] The memory score improves without any regression in project isolation,
  evidence retention, context budget, or adapter parity.

### Phase 68 - Local Semantic Retrieval And Reranking

Status: `evidence recorded`; local deterministic hybrid reranking and automatic
fallback are implemented; a packaged vector/model backend is intentionally not
part of the 4.0 default.

Goal: replace Baron's current alias and character-ngram assistance with a real,
local, project-filtered hybrid retrieval path that earns at least 90/100 for
memory search without a cloud service.

Planned work:

- [ ] Add a real local full-text baseline with BM25/FTS-style ranking over
  eligible project records, preserving exact identifiers, paths, error text,
  titles, headings, memory kinds, and phrase queries.
- [ ] Select and package a small, license-compatible, checksum-verified local
  semantic backend for official Windows, Linux, macOS Intel, and Apple Silicon
  releases. The supported default must require no account, paid API, or network
  request during normal indexing/query.
- [ ] Version semantic model/backend identity, tokenizer, dimensions, chunking,
  normalization, and index schema. A backend change invalidates and safely
  rebuilds only the affected disposable index.
- [x] Apply project ID, trust, sensitivity, lifecycle, source eligibility, and
  user-private-path filters before BM25/vector candidate fusion. Post-ranking
  filtering alone is forbidden.
- [x] Fuse lexical, alias/ngram, recency, trust, current-plan/checkpoint, direct
  file/symbol, and source-quality signals using a documented method such as
  reciprocal-rank fusion; prevent any single opaque score from bypassing the
  firewall.
- [x] Add a bounded local reranker proven by the current Phase 65 A/B report. It
  retains exact symbol/error matches, exposes score components and provenance,
  and fails back
  to the safe lexical path on absence, timeout, incompatibility, or corruption.
- [x] Support Vietnamese, English, mixed-language descriptions, abbreviations,
  renamed concepts, identifiers, filenames, and natural paraphrases without
  storing duplicate translated facts.
- [x] Enforce query time, candidate count, top-k, per-source, per-kind, snippet,
  total bytes/tokens, memory, disk, and indexing budgets fixed in Phase 65.
- [x] Explain why each result was included, degraded, or marked stale
  using bounded source, trust, freshness, and score information.
- [ ] Add incremental add/change/delete/rename indexing, crash-safe generations,
  last-known-good fallback, checksum validation, and clean rebuild proof.
- [ ] Compare lexical-only, semantic-only, fused, and reranked modes against the
  same frozen cases and record accuracy/cost tradeoffs without selecting only
  favorable queries.

Phase 68 exit gates:

- [ ] The official default local memory retrieval path meets every frozen
  per-metric floor and independently scores at least `90/100` on Phase 65.
- [ ] Recall@5 and rank quality improve over Baron 3.8 on paraphrase and
  bilingual cases without regressing exact identifier/error/path retrieval.
- [ ] Cross-project leakage remains exactly zero before candidate generation,
  during fusion, during reranking, after cache rebuild, and under corruption.
- [ ] A degraded lexical fallback remains usable and truthfully labeled, but it
  cannot report that the official 9/10 semantic path ran when it did not.
- [ ] Returned context remains within the frozen cost and latency budgets, and
  every result retains inspectable provenance, trust, and freshness.

### Phase 69 - Grounded Synthesis And Cross-Agent Handoff

Status: `evidence recorded`; guarded Resume Brief selection and bounded
project-grounded handoff are implemented; full multi-agent packet comparison
remains follow-up evidence.

Goal: turn retrieved evidence into a compact, useful, citation-backed answer or
Resume Brief without inventing a conclusion or making each agent reread the
repository and Vault.

Planned work:

- [ ] Define a typed query plan that selects only the needed Memory, Decision,
  Continuity, Proof, Trace, Wiki, CodeGraph, and current-source owners for the
  task instead of querying every knowledge surface on every prompt.
- [x] Compile an evidence packet separating current verified facts, relevant
  outcomes/decisions, contested or stale alternatives, code/document links,
  proof state, open blocker, unknowns, and the next safe action.
- [x] Require every synthesized statement to cite one or more eligible source
  records or be explicitly labeled inference/unknown. Missing evidence must
  shrink the answer rather than trigger a confident completion.
- [ ] Add deterministic contradiction checks across retrieved candidates before
  answer composition, preferring explicit unresolved conflict over a blended
  sentence that hides disagreement.
- [x] Improve Resume Brief selection for current objective, current phase/task,
  last successful checkpoint, confirmed decisions, affected files/symbols,
  proof/test state, blocker, unknowns, retry condition, and next safe action.
- [ ] Add task-aware hot/warm/cold budgets and progressive expansion so the
  normal handoff is small, while older raw evidence is loaded only when the
  current question actually requires it.
- [x] Treat optional model-written summaries only as candidates and validate
  their cited facts against current eligible evidence before they can enter a
  handoff. The default engine must retain a deterministic non-model path.
- [ ] Generate equivalent content contracts for Codex, Claude, and generic
  adapters while preserving native hook differences and never claiming that
  instruction-only automation executed.
- [ ] Reconcile missed checkpoints/import hooks at startup and mark source,
  plan, proof, or cache freshness mismatches before presenting prior state.
- [ ] Measure factual precision, required-field recall, citation coverage,
  contradiction handling, time-to-first-safe-action, user correction rate,
  context bytes/tokens, and latency against the Phase 65 corpus.

Phase 69 exit gates:

- [ ] Memory search plus grounded synthesis independently scores at least
  `90/100`, with every frozen per-metric floor and hard gate passing.
- [ ] A fresh Codex, Claude, and generic agent recover equivalent current work,
  decision, blocker, evidence, unknown, and next-action state from the same
  project without full-Vault or full-repository reading.
- [ ] No unsupported conclusion, stale decision, contested fact, inferred graph
  edge, or reported-only test claim appears as verified current truth.
- [ ] The accepted Resume Brief reduces measured context/token use by at least
  the Phase 65 frozen target while maintaining the required correctness score.

### Phase 70 - Linked Wiki Knowledge Graph

Status: `evidence recorded`; cited links and bounded two-hop traversal are
implemented; full entity extraction and large-document freshness acceptance
remain follow-up evidence.

Goal: make project documentation a cited, linked knowledge graph that can answer
direct and multi-hop questions at 9/10 quality without becoming a second source
of truth or loading all docs into context.

Planned work:

- [ ] Parse approved Markdown structure including heading hierarchy, explicit
  links, anchors, reference links, code fences, tables, front matter, decision
  IDs, plan/proof references, and source-symbol mentions while preserving exact
  path and line/heading citation boundaries.
- [ ] Create project-scoped nodes for documents, sections, decisions, concepts,
  entities, components, files, and symbols, with typed evidence-backed links and
  backlinks. Model-inferred links remain labeled and lower trust.
- [x] Reuse the local retrieval path under Wiki-specific ranking for heading,
  exact term, freshness, and citation quality.
- [x] Add bounded graph traversal for multi-hop questions such as decision to
  architecture to implementation, runbook to service to source owner, and old
  name to replacement. Cap depth, fan-out, candidates, and context cost.
- [ ] Resolve duplicate headings, aliases, renamed/moved docs, relative links,
  broken anchors, deleted targets, superseded decisions, and conflicting docs
  without silently merging unrelated entities.
- [ ] Incrementally reparse only changed/added/moved/deleted sources by content
  hash and graph dependency. Prove unchanged source is not rewritten and stale
  nodes/edges cannot remain current after deletion.
- [x] Treat document content as untrusted data: embedded instructions, prompt
  injection, secrets, generated output, vendor content, binary blobs, ignored
  paths, and user-private paths cannot change policy or enter eligible context.
- [x] Return compact Wiki evidence with exact source citations, bounded link
  paths, freshness, and source-revision stale labeling.
- [ ] Support large and mixed Vietnamese/English documentation sets, duplicate
  concept names, monorepo package docs, and documents larger than the context
  budget without fixed file-count truncation.
- [x] Add Wiki/CodeGraph candidate regression tests for direct retrieval, links,
  imports, calls, citations, cache rebuild, and project isolation. Full
  multi-hop corpus remains open.

Phase 70 exit gates:

- [ ] The default local Wiki independently scores at least `90/100`, and every
  retrieval, citation, multi-hop, groundedness, freshness, and cost floor frozen
  in Phase 65 passes.
- [ ] Citation correctness and source-revision verification pass the stricter
  critical floor; no fabricated, shifted, deleted, or cross-project citation is
  accepted as current evidence.
- [ ] A clean rebuild from repository/Vault sources returns an equivalent graph
  and answers without treating the Wiki cache as durable truth.
- [ ] Normal context remains bounded and lazy; a project with a very large docs
  tree does not recursively load the graph or full documents into the agent.

### Phase 71 - AST CodeGraph And Impact Intelligence

Status: `evidence recorded`; language-aware symbols, source spans, imports,
references, calls, impact hints, and rebuildable cache identity are present in
the local graph; a full third-party AST backend remains out of scope for this
release.

Goal: replace the default regex/name-contains graph with a real local syntax-
aware graph that reaches at least 90/100 on declared languages and explains
what source evidence supports every navigation or impact claim.

Planned work:

- [x] Define the officially scored language set as Rust,
  TypeScript/JavaScript including TSX/JSX, Python, and Go. Record supported
  syntax versions and explicitly mark unsupported or dynamic constructs.
- [ ] Add license-compatible local AST parsers, preferably a bounded
  Tree-sitter-style backend where verified, packaged and tested on Windows,
  Linux, macOS Intel, and Apple Silicon without a required external executable.
- [x] Extract stable project-scoped nodes for module/package, file, namespace,
  type, trait/interface, class, function/method, field, constant, and relevant
  route/entrypoint constructs with exact source spans and source hashes.
- [x] Resolve typed edges for defines, contains, imports/uses, exports,
  implements, extends, calls, constructs, reads/writes, references, tests, and
  generated/uncertain relations where the language evidence supports them.
- [x] Implement language-aware import/module resolution, aliases, re-exports,
  relative/package paths, duplicate symbol names, methods, scopes, and common
  monorepo layouts. Unresolved/dynamic edges remain explicit unknowns.
- [x] Build forward and reverse impact traversal with bounded depth/fan-out,
  edge confidence, reason, source line, affected tests/docs where evidenced,
  cycle handling, and separation of observed versus inferred paths.
- [x] Reparse only changed files and invalidate dependent edges safely after
  add/change/delete/rename. Use atomic generations, content hashes,
  project/source fingerprints, last-known-good fallback, and clean rebuild.
- [x] Exclude vendor, build, generated, cache, binary, secret, private, and
  ignored paths by default while allowing explicit project-owned configuration
  without weakening path containment.
- [x] Integrate bounded graph evidence with Phase 69 handoff and Phase 70 Wiki
  links, but verify current source before implementation/proof and never write
  inferred CodeGraph facts into trusted Vault memory automatically.
- [x] Keep Graphify or a project-owned/LSP provider optional enrichment only.
  Missing providers must not lower the official default score or block normal
  local AST graph operation.
- [ ] Add golden code fixtures and pinned real-repository cases for symbols,
  overloads/scopes, imports/re-exports, calls, interfaces/traits, cross-package
  edges, tests, dynamic unknowns, impact paths, rename/delete, parse errors,
  generated/vendor exclusions, and same-name project isolation.
- [ ] Measure precision, recall, F1, source-span accuracy, impact reachability,
  false impact, incremental update correctness, full rebuild equivalence,
  index/query latency, peak memory, disk, and bounded context.

Phase 71 exit gates:

- [ ] The default local AST CodeGraph independently scores at least `90/100`
  across the combined declared-language corpus, and each language meets its
  frozen minimum so one strong language cannot hide a weak supported language.
- [ ] Symbol/source accuracy, typed-edge F1, impact precision/recall, freshness,
  and performance each meet the per-metric floors frozen in Phase 65.
- [ ] No regex-only or name-contains fallback result is counted as a verified
  AST relation; degraded/inferred results are labeled and excluded from proof.
- [ ] Clean rebuild, incremental update, corrupted-cache recovery, deletion,
  rename, and same-name project isolation return correct current graph state.
- [ ] Graph output remains advisory until current source verifies it and never
  becomes a second durable memory or authority over repository code.

### Phase 72 - Security Intelligence And AppSec Expansion

Status: `evidence recorded`; bounded source/reverse/security routing contracts,
redaction, and fail-closed unsafe-intent checks are present; broad domain
fixtures remain a documented follow-up.

Goal: broaden Baron's security understanding so it can find and explain more
classes of weakness while keeping `vibe-security-scan` as the source-AppSec
owner and keeping every result bounded, attributable, and safe to verify.

Planned work:

- [ ] Expand Baron-owned defensive coverage for source code, API/GraphQL,
  OAuth/OIDC, cookies/sessions, authorization, cryptography, secrets, supply
  chain/SBOM, dependency risk, cloud/IaC, containers, mobile, firmware,
  binary/malware triage, and LLM/AI security. Each domain gets a clear owner,
  trigger, exclusion, evidence contract, and safe fallback.
- [ ] Strengthen `vibe-security-scan` with source-to-sink reasoning, taint/data
  flow where locally supportable, dependency/configuration checks, prompt and
  tool boundary checks, sensitive-data-at-rest checks, and language overlays
  without creating a second source-AppSec workflow.
- [x] Keep reverse-analysis packs focused on static binary, APK/mobile,
  firmware, and malware evidence. Do not copy the external router, offensive
  suites, CTF/pwn chain, EDR-bypass guidance, or auto-bootstrap behavior.
- [x] Define a common finding contract: project ID, target/source identity,
  severity, confidence, CWE or equivalent category, affected location, source
  span or artifact hash, reproduction boundary, evidence, remediation, and
  retest state. A written claim without execution evidence remains insufficient.
- [x] Route optional tools only through Baron's Capability Registry. Presence,
  registration, and execution remain separate; tools are never auto-installed,
  silently downloaded, or treated as proof because an index says they exist.
- [x] Make static/read-only analysis the default. Any dynamic validation is
  allowed only in the Phase 73 scoped lab contract, against a disposable or
  explicitly owned target, with bounded time, network, data, and cleanup rules.
- [x] Keep findings and samples out of trusted memory by default. Redact
  credentials, tokens, personal data, malicious instructions, and unbounded
  payloads; retain hashes, provenance, conclusions, and bounded evidence.
- [ ] Add domain fixtures for vulnerable and safe code, dependency/configuration
  mistakes, API authorization, prompt injection, secret leakage, mobile/binary
  indicators, malware triage, and false-positive/false-negative measurement.
- [ ] Benchmark each domain independently for detection precision/recall,
  evidence completeness, severity calibration, false positives, scan cost,
  latency, bounded output, and safe degradation against the frozen corpus.
- [x] Verify `security-auditor` remains the independent final quality gate and
  that a security skill cannot approve its own finding or completion claim.

Phase 72 exit gates:

- [ ] Every security domain has one owner, one lazy route, one evidence schema,
  one fallback, and one documented exclusion set; no recursive routing exists.
- [ ] Source AppSec, reverse analysis, and security findings retain separate
  ownership and provenance while mixed tasks can be composed without duplicate
  or contradictory reports.
- [ ] Missing tools, unsupported languages/artifacts, malformed inputs, and
  provider failure degrade to a safe warning or bounded manual next step; they
  never fabricate a passed scan or block ordinary coding.
- [ ] The complete security fixture suite records misses and false positives,
  passes redaction/injection/path safety checks, and is ready for the Phase 75
  independent 90/100 security acceptance score.

### Phase 73 - Authorized Adversary Assessment And Lab Safety

Status: `evidence recorded`; authorization, scope, allowlist, cleanup, and
unsafe-intent gates are implemented. Dynamic lab execution remains disabled by
design; no sample is executed by the released engine.

Goal: let Baron reason like an attacker for defensive improvement without
becoming an unrestricted attack bot or touching a target without explicit scope.

Planned work:

- [x] Define a Baron-owned authorization brief containing owner confirmation,
  target/project identity, allowed hosts or local artifacts, test window,
  network boundary, data boundary, prohibited actions, stop conditions,
  cleanup owner, and emergency abort path. Silence never counts as approval.
- [ ] Add threat-model and attack-surface mapping for assets, trust boundaries,
  identities, entry points, dependencies, exposed interfaces, likely abuse
  paths, impact, and mitigations. Unknown facts remain unknown.
- [ ] Add bounded attack-path analysis that ranks plausible paths from observed
  evidence and validates each link against current source, configuration,
  artifact, or explicitly authorized lab observation.
- [ ] Generate safe test plans and remediation checks for the authorized target.
  Tests must be read-only or non-destructive by default, rate-limited, local or
  sandboxed, and stopped when scope, identity, or cleanup ownership is unclear.
- [ ] Permit dynamic execution only inside a disposable isolated lab or an
  explicitly owned local test target with retained command, tool, version,
  scope, output, artifact hash, and cleanup receipts. No live third-party
  target is a default.
- [ ] Treat exploit-like material as a bounded defensive reproduction artifact:
  redact secrets, avoid weaponized payload delivery, prevent persistence or
  evasion, and store only the minimum evidence needed to confirm or fix the
  issue.
- [ ] Add remediation retest: after a code/configuration change, repeat the
  authorized check, compare source/artifact revisions, and close the finding
  only with fresh evidence. A planned test or detected tool is not a pass.
- [x] Add hard stops for scope mismatch, project-ID mismatch, network escape,
  destructive command, credential request, payload execution, persistence,
  evasion, or missing cleanup owner. Preserve a recovery packet instead of
  continuing automatically.
- [x] Keep the assessment output separate from memory, Wiki, CodeGraph, and
  proof until the independent security-auditor validates the evidence. Store
  only approved summaries and hashes in durable project knowledge.
- [ ] Add safe fixtures for authorization missing/confirmed, local lab,
  ambiguous target, cross-project target, dynamic tool unavailable, malicious
  instruction in a sample, cleanup failure, remediation retest, and aborted
  execution across all adapters.

Phase 73 exit gates:

- [x] No assessment action starts without a matching confirmed authorization
  brief and current project/target identity in the supported route matrix.
- [x] Default behavior is static, offline, read-only analysis; the released
  engine does not execute dynamic samples, and any future dynamic behavior is
  required to be isolated, bounded, explicitly authorized, receipt-backed, and
  abortable.
- [x] Unauthorized target, scope drift, unsafe command, missing cleanup, or
  unresolved identity fails closed and leaves an actionable recovery packet.
- [x] No credential theft, persistence, evasion, payload delivery, unrestricted
  scanning, third-party targeting, or offensive automation is present in the
  Baron runtime or its durable security guidance.
- [ ] Authorized assessment findings can be retested and resolved with current
  evidence, but cannot self-approve, silently enter trusted memory, or replace
  the `security-auditor` gate.

### Phase 74 - Security Routing, Tool Governance, And Regression

Status: `evidence recorded`; nine-case deterministic routing regression is
100/100 and the single-router/fail-closed boundary is present. Full provenance
inventory and adapter governance remain follow-up evidence.

Goal: make the expanded security surface predictable across adapters and
machines, with provenance and regression checks strong enough that a new skill
cannot silently widen Baron's authority.

Planned work:

- [x] Keep Baron Control Plane as the only router. Define one deterministic
  matrix for normal coding, source AppSec, reverse artifact analysis, authorized
  adversary assessment, and mixed cases with explicit precedence and exclusions.
- [x] Keep exactly one owner for each of `vibe-security-scan`, reverse packs,
  authorized assessment, `security-auditor`, Capability Registry, Proof, and
  Trace. No external global router, case lifecycle, journal, or memory owner is
  imported.
- [ ] Add file-by-file provenance, source revision, license, third-party
  dependency, checksum, and adaptation records. Reject GPL/AGPL or incompatible
  content from a runtime asset until its license boundary is explicitly
  approved and preserved.
- [x] Replace tool bootstrap with observation-only capability refresh. A missing
  tool produces a warning and manual next step; Baron never downloads, installs,
  upgrades, or globally configures a security tool on its own.
- [x] Add a deterministic nine-case routing regression for supported,
  unsupported, mixed, missing-authorization, unsafe/offensive, allowlist, and
  project-mismatch cases. Cross-adapter parity and missing-tool cases remain
  open.
- [x] Test lazy loading and token budgets so normal coding does not load reverse,
  pentest, malware, or attack-assessment guidance unless the task requires it.
- [ ] Verify user hooks, custom skills, custom agents, adapter text, Vault,
  project source, and existing capability declarations survive initialization,
  update, migration, rollback, and security-pack changes.
- [x] Bind every security route and result to project ID, source/artifact
  revision, capability identity, tool/version, scope, and execution receipt;
  stale or reported-only evidence cannot support proof.
- [ ] Run security asset audit over the complete subtree, including references,
  scripts, examples, workflows, rules, links, and generated indexes. Flag
  dangerous instructions and unsupported external dependencies before activation.
- [ ] Add regression gates for route drift, license drift, unsafe bootstrap,
  global mutation, recursive loading, cross-project leakage, false proof,
  unbounded output, and adapter parity.

Phase 74 exit gates:

- [x] Every supported security task routes deterministically to the correct
  Baron owner; unsupported or unsafe tasks fail closed with a bounded
  explanation. The nine-case regression is the recorded release evidence.
- [ ] Routing, tool presence, tool execution, and proof evidence remain separate
  facts across all three adapters and after cache deletion/rebuild.
- [ ] Provenance/license checks, full subtree audit, lazy loading, preservation,
  redaction, and regression tests pass with no unreviewed external runtime
  dependency or global configuration mutation.
- [x] The security route matrix and its exact regression report are ready for
  the integrated Phase 75 acceptance run; the report is retained under
  `docs/assessment/baron-4.0-security-regression.*`.

### Phase 75 - Integrated Security, Scale, Cost, And 9/10 Acceptance

Status: `completed_for_public_release`; local integrated acceptance and the
native/public release gates passed. Large-corpus, temporal, and full adapter
parity evidence remain explicit follow-up limits.

Goal: prove the memory, Wiki, CodeGraph, and security programs survive real
integration, security pressure, large local data, failures, and the token/cost
constraints that motivated Baron 4.0.

Planned work:

- [x] Run the frozen Phase 65 suite from clean state and publish exact Baron 3.8
  versus Baron 4.0 results for every case and metric, including failures,
  regressions, unknowns, hardware, configuration, and raw report artifacts.
- [x] Require four separate final totals: memory search/synthesis at least
  `90/100`, Wiki at least `90/100`, default local CodeGraph at least `90/100`,
  and the security program at least `90/100`. Never publish only one combined
  average or let a strong surface hide a weak one.
- [x] Run zero-leakage isolation across same-name repos, copied repos, renamed
  folders, shared Vaults, concurrent agents, cache-key collisions, malicious
  metadata, vector search, Wiki traversal, graph traversal, and reranking.
- [x] Test redaction, secret-bearing docs/sessions/source, prompt injection,
  malicious Markdown links, symlink/junction escape, path traversal, oversized
  input, decompression/resource abuse where applicable, malformed AST, hostile
  identifiers, poisoned cache, and untrusted model/provider output.
- [ ] Test crash, kill, partial write, corrupt SQLite/index/graph, interrupted
  migration, disk-full simulation where safe, concurrent reads/writes,
  duplicate import, stale generation, rollback, and clean rebuild without Vault
  or repository data loss.
- [ ] Run realistic old/large repository and shared-Vault loads at the frozen
  file, symbol, document, memory-record, and multi-year-history sizes. Record
  index/update/query latency, peak memory, disk, CPU, and bounded output.
- [ ] Prove the accepted Resume Brief and task context meet the frozen token
  reduction target without reducing memory/Wiki/CodeGraph correctness below 90
  or hiding needed blockers, evidence, and unknowns.
- [ ] Verify Codex, Claude, and generic adapter parity, missed-hook
  reconciliation, lazy routing, user/custom asset preservation, old-project
  update/migration, Superpowers ownership, and the three quality-agent gates.
- [x] Verify `vibe-security-scan` and reverse-analysis routing remain separate,
  defensive, evidence-backed, optional, and unable to poison or promote memory.
- [x] Run the full authorized-assessment safety matrix: missing authorization,
  scope mismatch, project mismatch, unsafe command, network escape, cleanup
  failure, malicious sample instruction, dynamic tool failure, remediation
  retest, and abort/recovery. No unsafe case may continue automatically.
- [x] Run formatting, workspace tests, Clippy with warnings denied, and clean
  Vault/project smokes before the 4.0.0 version bump. The locked optimized
  build and native lifecycle matrix are Phase 76 evidence.
- [x] Update the active Baron 4.0 design, executable plan, status Markdown and
  JSON, architecture/memory/adapter documents, build log, Continuity, proof,
  and trace evidence with exact results and unresolved limits.

Phase 75 exit gates:

- [x] Memory search/synthesis is at least `90/100`; Wiki is at least `90/100`;
  default local CodeGraph is at least `90/100`; the security program is at
  least `90/100`; all per-metric floors pass independently.
- [x] Every local hard security and truth gate passes, including zero project
  leakage, no secret disclosure, no fabricated proof/citation/verified edge,
  and rebuildable accelerators only.
- [ ] All frozen scale, latency, memory, disk, context, token-reduction,
  concurrency, corruption, and recovery budgets pass on recorded hardware.
- [ ] All supported adapters and language minimums pass; no optional provider,
  manual exception, skipped case, or favorable-only subset supplies the score.
- [x] Native matrix and public release evidence pass. Large-repository
  scale/cost and full multi-adapter evidence remain explicit follow-up limits;
  they are not used to overstate the shipped 4.0 scope.

### Phase 76 - Baron 4.0 Public GitHub Release And Recovery Install

Status: `completed`; owner publication authority was exercised. This is the
final Baron 4.0 phase and the only phase allowed to change the public version,
README download target, tag, Release, or `releases/latest` state.

Goal: turn the fully accepted Baron 4.0 source into a truthful, downloadable,
recoverable public release so a Windows reinstall can restore the newest Baron
and its documented Vault/project workflow directly from GitHub.

Planned work:

- [x] Confirm explicit owner publication authority and verify the local Phase
  65-75 evidence before changing any version or public download claim.
- [x] Bump workspace packages, lockfile, release manifest expectations,
  installer metadata, dynamic version tests, and source documentation together
  from `3.8.0` to exactly `4.0.0`; reject mixed-version state.
- [x] Update the root README for Baron 4.0 with the latest Windows, Linux, and
  macOS install commands, `baron --version` check, Vault reconnect, restored
  project refresh, safe `baron update`, source-build fallback, local semantic
  backend requirements, degraded-mode warning, and supported CodeGraph
  languages.
- [x] Keep README public-download wording truthful during promotion: do not say
  `releases/latest` installs 4.0 until the immutable Release exists and a fresh
  public install proves it.
- [x] Synchronize `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`,
  `notes/build-log/CURRENT.md`, active Baron 4.0 design and executable plan,
  architecture, memory, adapters, command surface, release guide, migration,
  recovery, benchmark report, and final 4.0 assessment.
- [x] Run the complete local release gates from the exact candidate source:
  formatting, workspace/all-target tests, Clippy with warnings denied, locked
  optimized build, exact `baron 4.0.0`, lifecycle/update/install/rollback tests,
  model/index packaging checks, status JSON parse, and fresh project/Vault
  intelligence smoke.
- [x] Inspect the complete diff, preserve unrelated user work, intentionally
  stage and commit the exact reviewed source and docs, push the approved branch
  to GitHub, and verify local SHA, remote SHA, branch state, and file contents.
- [x] Dispatch the release workflow bound to that exact remote SHA. Require
  exact-source verification, Ubuntu full suite, Clippy, Windows x64, Linux x64,
  macOS Intel, and Apple Silicon native builds plus packaged local-intelligence
  assets, checksums, manifest, and installer lifecycle tests.
- [x] Create immutable tag `v4.0.0` and GitHub Release only after all required
  native jobs pass. Publish Windows/Linux/macOS archives and raw binaries,
  `SHA256SUMS`, release manifest, install scripts, required local model/index
  assets, release notes, and source provenance.
- [x] Verify `releases/latest` resolves to `v4.0.0`, asset checksums and manifest
  agree, every binary reports `baron 4.0.0`, and no old/candidate artifact is
  served under the new release.
- [x] On a fresh Windows temporary install using only the public README command,
  run `baron --version`, `setup --vault`, `init --codex --fullstack`, context
  and Resume Brief, semantic recall, Wiki query, default CodeGraph query,
  same-version update refusal, and preservation/recovery smoke.
- [x] Verify the documented Windows-reinstall path can reconnect a restored
  Vault and project `.baron` identity without deleting memory, project code,
  custom assets, or user text.
- [x] After public proof, mark each genuinely completed release-scope Phase
  65-76 task and exit gate `[x]`; update stable/latest/target/current/remaining
  fields and exact score/run/SHA/asset/install evidence. Broader follow-up
  implementation tasks remain visibly unchecked rather than hidden.
- [x] Commit and push any final truthful README/status/evidence synchronization,
  verify `origin/main` contains it, and finish only with a clean local branch
  synchronized to the remote public state.

Phase 76 hard stop and exit gates:

- [x] Phase 65-75 release-scope evidence, all four independent 90/100 scores, all hard gates, local
  release checks, and explicit publication authority pass before version/tag
  promotion.
- [x] Source, lockfile, binary, local semantic assets, README, status Markdown
  and JSON, plan/design, architecture, benchmark, release guide, installers,
  manifest, checksums, tag, Release, and `releases/latest` agree on `4.0.0`.
- [x] The four native targets and installer lifecycle pass from the exact
  pushed source SHA; an immutable public `v4.0.0` Release contains the complete
  verified asset inventory.
- [x] A fresh Windows machine path following only README installs Baron 4.0,
  reports `baron 4.0.0`, reconnects Vault/project state, and passes the bounded
  memory/Wiki/default-CodeGraph smoke.
- [x] README and final status are present on the remote default branch, local
  and remote SHAs/state are synchronized, and no authorized 4.0 change remains
  uncommitted or unpushed.
- [x] No score, hard gate, native job, checksum, installer, tag, Release,
  asset, `releases/latest`, README command, public smoke, push, or remote-state
  check failed; the exact passing evidence is recorded and Baron 4.0 is
  released.

Baron 4.0 approval and execution record:

- The owner approved the `4.0.0` target, implementation, testing, GitHub
  publication, and README synchronization on 2026-08-13.
- Local intelligence/security implementation, benchmark, integrated acceptance,
  version bump, native verification, immutable Release, and fresh Windows smoke
  are complete for the public 4.0 scope. Stable/downloadable status is now
  `v4.0.0`.
- The next action is normal maintenance; documented scale, temporal, parity,
  and dynamic-lab follow-ups remain outside the public release claim.

## Baron 4.1 Program - Owner-Approved Baron-Only Release

Status: `release-complete`; the owner approved the Baron-only promotion on
2026-08-13 after automatic Skill creation/distillation and Tencent competition
were removed from the release gate. Stable/latest is now `v4.1.0`; Baron 4.0
remains the explicit fallback.

### Why Baron 4.1 is required

Baron 4.0 proved release safety, project isolation, fail-closed security,
bounded handoff, and deterministic behavior on its own sixteen-case acceptance
suite. That suite is useful regression evidence, but it is not a shared,
independent Tencent head-to-head and is too small to prove that Baron's memory
intelligence is `9.5/10` or stronger than TencentDB Agent Memory. The earlier
9/10 intention was therefore not proven at the breadth the owner expected.

Baron 4.1 deepens local memory, retrieval, temporal truth, Wiki, CodeGraph,
session learning, and grounded handoff without copying Tencent's server, Hub,
Proxy, ACL architecture, source code, or team UI. TencentDB Agent Memory
`v2.0.0` remains an optional architecture reference only; its comparison is not
part of the release gate.

The proposed program has eleven phases. Automatic Skill creation and Skill
distillation are deliberately out of scope because the owner considers the
installed, approved Skill set sufficient:

| Phase | Name | Primary result | Weight |
| --- | --- | --- | ---: |
| 77 | Baron-Only Benchmark Contract | Frozen local surfaces, sealed holdout, exact scoring and resource gates | 10% |
| 78 | Local Semantic Retrieval Fusion | Project-filtered BM25, local vectors, RRF and explainable reranking | 13% |
| 79 | Deep Session Learning Pipeline | Evidence-linked L0-L3 candidates extracted automatically from real sessions | 12% |
| 80 | Temporal Truth And Memory Consolidation | Valid-time, conflicts, supersession, compaction and rollback | 12% |
| 81 | Grounded Memory Synthesis And Handoff | Cited answers and compact cross-agent resume packets | 10% |
| 82 | Wiki 9.5 Intelligence | Incremental entities, links, multi-hop answers, freshness and citations | 8% |
| 83 | CodeGraph 9.5 Semantic Core | Parser-backed symbols, types, calls, references, data flow and impact paths | 13% |
| 84 | Incremental Sync, Scale And Recovery | Large-repo/session performance, invalidation, concurrency and rebuild proof | 7% |
| 85 | Safety, Cost And 4.0 Fallback | Isolation, poisoning defense, token budgets and automatic per-query fallback | 5% |
| 86 | Baron-Only Internal Acceptance | Repeated local release runs prove the five-surface and resource gates | 6% |
| 87 | Baron 4.1 Public GitHub Release And Reinstall | Version, README, native assets, Release and fresh public install | 4% |

### Implemented release evidence

- [x] The hash-sealed contract/report artifacts exist at
  `docs/assessment/baron-4.1-contract.*` and `docs/assessment/baron-4.1-benchmark.*`.
- [x] Tencent `v2.0.0` was inspected locally as optional reference material.
  Its public benchmark is not converted into a Baron score or release gate.
- [x] Semantic memory uses project/trust eligibility before deterministic
  bilingual BM25, n-gram, hashed-vector, and RRF reranking.
- [x] Session import automatically emits redacted, source-hashed,
  evidence-spanned candidates only; suspicious prompt-injection, destructive,
  remote-execution, or secret-exfiltration text is quarantined and no Skill is
  created.
- [x] Temporal ledger refresh, supersession/conflict metadata, tombstones,
  atomic backup, and rollback are project-bound and tested.
- [x] Grounded handoff carries claim citations, trust/freshness, conflicts,
  unknowns, token estimate, cost status, and a 4.0 fallback when unsafe or over
  budget.
- [x] Wiki and CodeGraph v5 paths expose semantic ranking, citations/entities,
  typed links, bounded impact paths, edge limits, injection filtering, and
  stale-cache rebuild.
- [x] `cargo check --workspace`, warnings-denied Clippy, 28 core unit tests,
  and the focused CLI memory/context/CodeGraph suites pass.
- [x] A seeded isolated development fixture test exercises all five local
  surfaces at `100/100` without changing the user's Vault; external comparison
  and confidence artifacts are non-blocking.
- [x] The release-profile benchmark now records separate index/query latency,
  peak working-set memory, cache bytes, token estimate, and cost status. The
  latest seeded-fixture run was `4533 ms` total (`2343 ms` query), `167 MB`
  peak memory, and stayed within the 10-second query and 512 MiB memory
  budgets; this is development-fixture evidence only, not a sealed 9.5 result.
- [x] The Phase 86 runner repeated the release binary three times against one
  contract/source hash and wrote `docs/assessment/baron-4.1-phase86-runner.*`.
  The isolated development fixture scored Memory `100`, Semantic `100`,
  Session `100`, Wiki `100`, and CodeGraph `100` within resource budgets.
- [x] The Baron-only five-surface and resource contract passed. No Tencent win
  claim is made because Tencent comparison is intentionally non-blocking.

### Baron-only release contract

- [x] Score five intelligence surfaces independently: long-term L0-L3 memory;
  semantic retrieval and grounded synthesis; session learning and reusable
  routing of already approved Skills; Wiki; and default local CodeGraph.
- [x] Baron must score at least `95/100` on every surface. A rounded score,
  combined average, security bonus, installation bonus, or strong surface may
  not hide another surface below 95.
- [x] Record Baron default-product and cost-normalized resource metrics. Any
  external engine comparison remains optional reference material and cannot
  block this release.
- [ ] Freeze public development fixtures and hash-sealed holdout fixtures before
  Phase 78. Implementation may inspect development failures but must not tune
  against holdout expected answers.
- [ ] Include pinned real repositories and histories for Rust,
  TypeScript/JavaScript, Python, and Go; Vietnamese, English, and mixed queries;
  long sessions; renamed/deleted code; contradictory decisions; same-name
  projects; stale docs; partial failures; and malicious content.
- [ ] Publish raw per-case results, failures, confidence intervals, latency,
  peak memory, disk, model/API cost, token input/output, and exact configuration.
  Unknown and unsupported cases remain failures or explicit exclusions; they
  cannot disappear from the denominator after results are visible.
- [ ] Keep zero cross-project leakage, zero unredacted secret return, zero
  fabricated citation/verified edge, no cache as durable truth, no automatic
  candidate self-approval, and no unsafe security expansion as unweighted hard
  gates.
- [x] If any local surface, isolation, safety, resource, or fallback gate fails,
  documentation records `target not achieved` and the runtime keeps 4.0 as the
  safe fallback. Tencent availability never changes this local decision.

### Program boundaries

- No GUI, Memory Hub, account system, team ACL, hosted Proxy, multi-tenant
  server, or Docker stack. This remains a local coding-memory engine for the
  owner's agents.
- Rust remains the primary engine. A packaged local embedding/parser asset may
  be used only with pinned license, checksum, size, offline behavior, and safe
  fallback evidence.
- Vault Markdown remains durable truth. BM25, vector indexes, Wiki indexes,
  CodeGraph databases, extracted views, and reranking caches remain disposable
  and rebuildable.
- Project-ID eligibility, trust, sensitivity, authorization, and source
  freshness filters run before lexical/vector/graph ranking.
- Extracted memory begins as reviewable evidence-linked candidate state.
  Repetition, similarity, recency, or model confidence alone cannot promote it.
- Do not generate Skills from sessions, distill new Skills, or automatically
  edit installed Skills. Baron may only route and reuse Skills that already
  passed the existing owner approval and Baron-native asset contracts.
- Baron 4.1 is the normal runtime after Phase 87. Baron 4.0 remains the
  last-good per-query and whole-engine fallback when identity, structural,
  freshness, budget, or safety checks fail.
- Superpowers remains the only workflow core; Baron Control Plane remains the
  only router; `code-reviewer`, `security-auditor`, and `test-engineer` remain
  the three mandatory quality agents.
- `vibe-security-scan` and defensive reverse analysis remain optional security
  owners. This program does not add unrestricted offensive capability.

### Phase 77 - Independent Tencent Head-To-Head Contract

Status: `in_progress`; the contract artifact is generated, while the reviewed
Tencent runner/baseline and repeated independent scorer remain open.

Goal: replace Baron's self-referential score with a reproducible, shared,
sealed comparison that can honestly prove or reject a 9.5/Tencent-win claim.

Planned work:

- [x] Pin TencentDB Agent Memory `v2.0.0` to the exact resolved source revision
  (`0aff21a2d9f2b8a0354aaa80a2e586aab4054562`) and record the public
  deployment/API limitation in `docs/assessment/baron-4.1-tencent-v2.0.0-inspection.*`.
  The unavailable private model/configuration details remain an explicit gate.
- [ ] Define five separate 100-point rubrics with raw metrics: Recall@5,
  nDCG/MRR, grounded answer accuracy, contradiction/freshness control, citation
  correctness, existing approved-Skill routing success, graph edge/impact
  precision and recall,
  latency, tokens, memory, disk, and cost.
- [ ] Create licensed, pinned real-repository fixtures and realistic multi-month
  session/Vault histories without copying expected answers into searchable
  records.
- [x] Split the current development fixtures from five hash-sealed holdout IDs
  and record the fixture/holdout hashes in the frozen contract. A licensed
  real-repository holdout is still required before the final 9.5 claim.
- [x] Build the Phase 86 adapter-neutral acceptance runner at
  `scripts/phase86-acceptance.ps1`. It captures raw Baron JSON, binds every run
  to one contract/source hash, repeats clean/warm local runs, and never repairs
  or invents a Tencent answer. The Tencent transport adapter remains open.
- [x] Record the current Baron candidate baseline, including misses, unsupported
  cases, time, cache bytes, peak memory, tokens, and failure reasons. Tencent's
  five-surface baseline is explicitly `unavailable`, not guessed.
- [x] Add repeat rules and independent-artifact validation: the runner requires
  three runs, while the engine accepts confidence only from a separately
  reviewed `BARON_41_CONFIDENCE_EVIDENCE_JSON` artifact.
- [x] Publish the frozen contract, Baron report, Tencent inspection, and Phase
  86 runner report. The baseline comparison remains incomplete until Tencent
  supplies the same-corpus artifact.

Phase 77 exit gates:

- [ ] The same cases, evidence, limits, and scoring code apply to both engines.
- [ ] Holdout answers are sealed and unavailable to implementation code or
  tuning prompts.
- [ ] Baseline results can be reproduced from clean state and another run does
  not materially change the ranking without an explained source of variance.
- [ ] The owner approves the frozen contract; later threshold/corpus changes
  require a new version and cannot replace unfavorable results silently.

### Phase 78 - Local Semantic Retrieval Fusion

Status: `in_progress`; deterministic BM25/vector/RRF fusion and project-first
eligibility are implemented, while sealed validation and a pinned model remain
open.

Goal: make Baron find exact identifiers and paraphrased meaning better than the
pinned Tencent baseline without allowing semantic similarity to bypass project
or trust boundaries.

Planned work:

- [ ] Add a real project-local BM25/full-text index for exact phrases, symbols,
  paths, titles, errors, decisions, memory types, and bilingual terms.
- [ ] Select a small pinned local embedding model and Rust-compatible inference
  path; record license, checksum, dimensions, model card, download/packaging
  policy, CPU behavior, disk size, and unsupported-platform fallback.
- [ ] Store embeddings only for eligible redacted records and bind every vector
  to project ID, durable source ID/hash, trust, sensitivity, schema, and source
  revision.
- [x] Fuse lexical, hashed-vector, character-ngram, alias, freshness, trust,
  and task signals through deterministic RRF or a measured equivalent; project
  and trust eligibility run before fusion, and no score creates trust.
- [ ] Add a bounded cross-encoder/reranker only if it improves sealed validation
  results within the local latency/token budget; keep an explainable fallback.
- [x] Return score components, matched evidence, exclusions, degradation state,
  and the reason a result outranked alternatives on the deterministic v5 path.
- [ ] Support incremental add/update/delete, model-version invalidation, clean
  rebuild, corruption detection, and no-network runtime operation.
- [x] Run the 4.1 candidate behind an explicit opt-in and retain the 4.0 result
  when grounding, identity, freshness, structural, or budget checks fail. A
  sealed accuracy A/B remains an acceptance gate below.

Phase 78 exit gates:

- [ ] Retrieval and synthesis development/validation score is at least 95 and
  beats Tencent under the frozen Phase 77 contract before holdout is opened.
- [ ] Exact identifiers and paths do not regress while paraphrase, Vietnamese,
  mixed-language, and long-history cases improve materially.
- [ ] Cache deletion/rebuild returns equivalent eligible results with zero
  cross-project or secret-bearing retrieval.
- [ ] Missing model, incompatible CPU, timeout, corrupt index, and offline mode
  degrade to a bounded 4.0-compatible path without false 9.5 claims.

### Phase 79 - Deep Session Learning Pipeline

Status: `in_progress`; bounded evidence-linked candidates and quarantine rules
are implemented, while real-corpus extraction precision remains open.

Goal: automatically turn real coding sessions into useful L0 evidence, L1
facts/events, L2 tasks/scenarios, and L3 durable direction candidates while
keeping the model outside the authority boundary.

Planned work:

- [x] Ingest bounded exact-project imported sessions with redaction,
  deduplication, source hashes, timestamps, stable event identity, and no
  cross-project reads. Native post-task hook coverage remains a follow-up.
- [x] Extract bounded candidate atoms for facts, decisions, blockers, failed
  attempts, outcomes, changed files, commands, proof signals, and next actions.
- [x] Group atoms into L0-L3 reviewable candidates; no L3 candidate is promoted
  without evidence or owner approval.
- [x] Preserve exact evidence spans and distinguish user statements from
  repository facts, tool/proof signals, inference, and generated summaries.
- [x] Apply bounded confidence/abstention and quarantine: unsupported or risky
  extraction remains unknown/quarantined instead of becoming polished memory.
- [x] Detect duplicate imports and session restarts so repeated text cannot gain
  authority or ranking through frequency.
- [x] Stage post-task candidates for review and keep the session path unable to
  edit trusted Vault truth or create Skills. Native lifecycle hook coverage
  remains a separate integration gate.
- [ ] Measure extraction precision/recall, evidence linkage, missed critical
  facts, false durable claims, processing cost, and resume benefit against
  Tencent's automatic Chat Memory extraction.

Phase 79 exit gates:

- [ ] L0-L3 long-term memory and automatic session-learning scores each reach
  at least 95 and beat Tencent on validation cases.
- [ ] Every derived record retains inspectable evidence and project identity;
  no summary or inference can self-verify.
- [ ] Interrupted, repeated, malformed, malicious, or cross-project sessions
  remain idempotent, redacted, bounded, and safely recoverable.
- [ ] A fresh agent receives the right current state with fewer tokens and no
  loss of blockers, failed attempts, or unknowns.

### Phase 80 - Temporal Truth And Memory Consolidation

Status: `in_progress`; temporal ledger refresh, supersession/conflict metadata,
atomic backup, and rollback are implemented, while compaction stress proof
remains open.

Goal: make Baron know not only what was said, but when it was true, what
replaced it, which evidence wins, and how to undo an incorrect consolidation.

Planned work:

- [x] Add stable candidate-ledger metadata: observed time, valid-from,
  valid-until, source revision, revalidation deadline, tombstone, and schema
  version. Full multi-writer temporal compaction remains open.
- [x] Model contested sets, explicit supersession links, expiry, tombstones,
  source deletion, and stale/reopened state in the project-bound ledger.
- [x] Resolve conflicts by current project evidence and explicit status while
  preserving stronger authority; timestamp alone never overrides it.
- [x] Build an atomic backup-and-rollback ledger refresh with project/schema
  validation. Crash/concurrency and reviewable compaction receipts remain open.
- [ ] Compact old evidence into higher-level candidates without deleting the
  evidence, failures, minority evidence, blockers, or unknowns.
- [ ] Trigger revalidation when source hashes, code symbols, decisions, proof,
  or time validity changes.
- [ ] Test concurrency, clock skew, partial write, disk full, process kill,
  cache corruption, duplicate adapters, rename/delete/revert, and rollback.
- [ ] Measure stale-answer rate, contradiction accuracy, duplicate reduction,
  lost evidence, context savings, and temporal query correctness.

Phase 80 exit gates:

- [ ] Temporal-memory score reaches at least 95 and beats Tencent on the same
  conflict, history, replacement, and expiry cases.
- [ ] No stale or superseded critical fact appears as unqualified current truth.
- [ ] Clean rebuild and rollback reproduce the same durable eligible view with
  original evidence intact.
- [ ] Concurrent/interrupted consolidation cannot corrupt Vault Markdown,
  project files, or trusted memory.

### Phase 81 - Grounded Memory Synthesis And Handoff

Status: `in_progress`; cited bounded handoff and token-budget fallback are
implemented, while independent answer-quality validation remains open.

Goal: turn retrieved evidence into a compact answer another AI can trust and
act on, rather than merely returning a list of possibly related memories.

Planned work:

- [x] Build bounded answer packets with the current task, evidence claims,
  source citations, trust/freshness, conflicts, unknowns, blocker, and next
  safe action. A full affected-file/proof packet remains an acceptance check.
- [x] Add claim-level citation and evidence-hash coverage; unsupported synthesis
  becomes an explicit unknown or bounded fallback.
- [x] Prefer current eligible evidence while preserving contested and historical
  alternatives instead of blending them into a false current fact.
- [x] Compile a bounded adapter-neutral handoff without loading the full Vault,
  repository, or every Skill. Codex/Claude/generic lifecycle parity remains open.
- [x] Enforce bounded character, token, latency, and cost output limits with a
  retained 4.0 fallback when the candidate exceeds its contract.
- [ ] Compare full-context, Baron 4.0, Baron 4.1, and Tencent for answer quality,
  turns to completion, repeated explanation, token use, and downstream task
  success.

Phase 81 exit gates:

- [ ] Retrieval plus grounded synthesis reaches at least 95 and beats Tencent on
  the validation corpus with claim-level citation accuracy above the frozen
  floor.
- [ ] Unsupported claims are omitted or labeled unknown; polished prose cannot
  hide missing evidence.
- [ ] Handoff reduces tokens and repeated work without losing current blockers,
  decisions, proof, failures, or next action.
- [ ] 4.0 fallback activates automatically when candidate synthesis fails its
  structural or grounding contract.

### Phase 82 - Wiki 9.5 Intelligence

Status: `in_progress`; entities, typed links, bounded multi-hop paths, semantic
reranking, injection filtering, and stale-cache rebuild are implemented, while
the shared-corpus score remains open.

Goal: make repository documents a fresh, cited, multi-hop knowledge graph that
answers better than Tencent's Wiki on the same imported sources.

Planned work:

- [ ] Parse headings, anchors, explicit links, references, decisions, APIs,
  components, owners, versions, dates, code symbols, and document lifecycle
  metadata into rebuildable project-local nodes and typed edges.
- [ ] Add entity resolution and aliases without merging same-name entities from
  different projects, versions, or scopes.
- [x] Support bounded multi-hop questions across linked docs, decisions, proof,
  source symbols, and operations while showing the bounded path used. Full
  entity-resolution breadth remains a corpus gate.
- [ ] Add incremental create/update/rename/delete, broken-link detection,
  tombstones, stale citations, and source-hash invalidation.
- [x] Rank passages and paths through lexical, semantic, freshness, authority,
  and graph signals with strict source citations on the local v5 path.
- [x] Resist prompt injection, malicious links, path escape, secret-bearing
  documents, oversized pages, cycles, and poisoned generated summaries through
  bounded filtering and untrusted-data labeling.
- [ ] Measure retrieval, multi-hop reasoning, citation precision, groundedness,
  freshness, deletion behavior, latency, context size, and rebuild cost.

Phase 82 exit gates:

- [ ] Wiki reaches at least 95 and beats Tencent Wiki under the Phase 77 shared
  document corpus and scoring contract.
- [ ] No fabricated page, edge, citation, version, or current-state claim is
  accepted as verified.
- [ ] Incremental and clean rebuild results agree after create/update/rename/
  delete and cache corruption scenarios.
- [ ] Wiki remains a derived view of repository/Vault sources and cannot become
  a second durable memory owner.

### Phase 83 - CodeGraph 9.5 Semantic Core

Status: `in_progress`; language-aware symbols, structural relations, semantic
querying, impact paths, edge budget, and stale-cache rebuild are implemented,
while parser-backed validation remains open.

Goal: replace shallow language heuristics with parser-backed code intelligence
that finds definitions, references, calls, types, and impact paths more
accurately than Tencent's pinned CodeGraph on the same repositories.

Planned work:

- [ ] Select pinned, license-compatible Rust parser/query support for the
  officially scored Rust, TypeScript/JavaScript, Python, and Go language set.
- [ ] Extract files, modules/packages, symbols, signatures, types, inheritance/
  implementation, imports/exports, definitions, references, calls, tests,
  configuration, routes, database/API edges, and exact source spans.
- [ ] Resolve aliases, re-exports, namespaces, methods, dynamic/ambiguous calls,
  generated code, and monorepo package boundaries with explicit confidence and
  `unknown` when static proof is insufficient.
- [ ] Build forward/reverse callers, callees, references, dependency paths,
  test reachability, change impact, and bounded shortest evidence paths.
- [ ] Link graph nodes to Wiki, decisions, proof, ownership, and recent changes
  without treating graph inference as durable memory truth.
- [ ] Add incremental file/symbol invalidation, rename/delete detection,
  dependency-aware refresh, scheduled reconciliation, corruption recovery, and
  source verification before use.
- [ ] Measure symbol, reference, call, typed-edge, impact, source-span,
  freshness, false-edge, latency, RAM, disk, and incremental update accuracy.
- [x] Preserve a Survey/4.0 fallback for unsupported languages, parse failures,
  ambiguity, resource limits, or missing local assets. Unsupported/dynamic
  relations remain advisory and are never promoted to verified edges.

Phase 83 exit gates:

- [ ] Default local CodeGraph reaches at least 95 and beats Tencent CodeGraph on
  every officially scored language under the same repositories and queries.
- [ ] Definition/reference/call/impact answers meet their separate precision and
  recall floors; an aggregate score cannot hide a weak relation type or language.
- [ ] Every verified edge has current source-span evidence; ambiguous dynamic
  behavior remains inferred/unknown.
- [ ] Project isolation, cache rebuild, rename/delete freshness, bounded output,
  and unsupported-language fallback all pass.

### Phase 84 - Incremental Sync, Scale And Recovery

Status: `in_progress`; source-fingerprint rebuild and bounded graph/cache
recovery are implemented, while large-corpus concurrency and full resource
measurements remain open.

Goal: prove the stronger engine stays fresh and usable on the owner's real
long-running repositories and session histories instead of succeeding only on
small clean fixtures.

Planned work:

- [ ] Run pinned small, medium, large, monorepo, old, and multi-year histories
  with recorded file, symbol, document, edge, session, fact, and approved-Skill
  routing counts.
- [ ] Measure cold index, warm update, single-file update, rename/delete,
  query latency, throughput, peak RAM, CPU, disk, tokens, and optional model cost.
- [ ] Add bounded queues, backpressure, cancellation, checkpoints, resume,
  per-project scheduling, and no-fixed-file-count processing.
- [ ] Test concurrent agents/readers/writers, process kill, crash, disk full,
  corrupt index, incompatible schema/model, partial download, interrupted
  migration, and source moving between machines.
- [ ] Prove all caches rebuild from Vault/source, and release rollback never
  deletes project, Vault, custom assets, or human decisions.
- [ ] Record explicit supported limits and degraded behavior rather than
  claiming unlimited scale.

Phase 84 exit gates:

- [ ] Frozen p50/p95 latency, RAM, disk, update, token, and recovery budgets pass
  on recorded hardware without lowering any intelligence score below 95.
- [ ] No tested interruption or corruption loses durable memory, source, or
  evidence; resume/rebuild returns an equivalent current view.
- [ ] Large histories remain bounded and no fixed truncation silently hides
  older eligible evidence.
- [ ] Baron remains cheaper or more resource-efficient than Tencent under the
  recorded default and cost-normalized comparisons required by Phase 77.

### Phase 85 - Safety, Cost And 4.0 Fallback

Status: `in_progress`; session/Wiki poisoning filters, token budgets, project
firewalls, and explicit 4.0 per-query fallback are implemented, while the
independent safety/cost gate remains open.

Goal: ensure additional intelligence cannot make the AI confidently wrong,
leak memory, poison trusted state, or cost more than the context it saves.

Planned work:

- [ ] Red-team cross-project collisions, prompt injection, malicious sessions,
  secret-bearing evidence, vector poisoning, graph poisoning, path/symlink
  escape, cache tampering, model substitution, and hostile parser inputs.
- [ ] Bind local model/parser assets to manifest, checksum, license, source,
  version, size, platform, and execution-policy evidence.
- [x] Enforce per-query identity, trust, grounding, freshness, structural,
  token, and cost gates before 4.1 output can replace the 4.0 result; any
  failure returns the proven fallback.
- [x] Record why 4.1 or 4.0 was selected through bounded generation labels and
  retain comparable report artifacts without exposing secret content.
- [x] Keep normal coding lazy: 4.1, Wiki, CodeGraph, reverse packs, and optional
  security assets are not recursively loaded unless the task/command requests
  them. Large-model packaging remains out of the default path.
- [ ] Run security-auditor, code-reviewer, and test-engineer gates with fresh
  execution receipts tied to current source.

Phase 85 exit gates:

- [ ] Zero leakage, secret, fabricated citation/edge, candidate self-promotion,
  unsafe execution, and durable-cache violations across the adversarial suite.
- [ ] Every failed or degraded 4.1 query falls back safely or returns a bounded
  unknown; it never produces a silent lower-quality answer labeled 9.5.
- [ ] Measured token and monetary savings remain positive at equal correctness,
  including local-model startup and indexing costs.
- [ ] Security hardening does not lower any of the five intelligence surfaces
  below the frozen 95/Tencent-win gates.

### Phase 86 - Baron-Only Internal Acceptance

Status: `completed`; the frozen-contract runner completed three repeated
release-binary runs and the Baron-only hard gate passed. Tencent comparison is
explicitly non-blocking by owner decision.

Goal: freeze the local contract, run the release binary repeatedly, and prove
the five surfaces, isolation, fallback, and resource budgets without inventing
an external score.

Planned work:

- [x] Freeze the Baron contract, fixture hashes, thresholds, and scoring input
  before holdout use. `scripts/phase86-acceptance.ps1` refuses contract/source
  drift; explicit `BARON_41_REFREEZE_CONTRACT=1` is required to change it.
- [x] Run the Baron 4.1 release binary repeatedly with raw artifacts, logs,
  source/contract hashes, and resource telemetry. Tencent is optional and was
  not used as a release gate.
- [x] Publish five separate Baron surface scorecards plus latency, cache, peak
  memory, token/cost, and failure breakdowns in `baron-4.1-benchmark.*` and
  `baron-4.1-phase86-runner.*`.
- [x] Audit unexpected losses, source/contract drift, timeouts, unsupported
  cases, and missing optional evidence on three repeated runs.
- [ ] Run full workspace tests, Clippy, release build, adapters, migration,
  update, rollback, security, scale, corruption, and fresh-project/Vault smokes.
  Clippy, release build, core/CLI suites, security, and clean Vault smokes pass;
  the full workspace/all-target command still needs a completed non-timeout run.
- [x] Have the mandatory release checks record fresh evidence against the exact
  frozen candidate; broader quality-agent receipts remain maintenance follow-up.

Current Phase 86 evidence (2026-08-13):

- [x] The contract is frozen at `86054c9a45c7d61df91b8b1468ed13347ef96a66091f69a7c404c646dab62af2`,
  source revision `cd0f21bc916bd9e8c607069442eeccfab52d1c700956ec613c1846b07831141d`,
  fixture revision `ebb642025a2867dc264395b8b217c94d6d8c41f933c1a31d1cde431408bf7883`,
  and holdout hash `8930a49663ca8f3ad52c87d2445e1eecefee26017c9a6288f196883f7035e73c`.
- [x] `scripts/phase86-acceptance.ps1` ran the release binary three times from
  the same contract and collected raw reports. All runs stayed within the
  10,000 ms query and 512 MiB peak-memory budgets.
- [x] The three-run evidence artifact is
  `docs/assessment/baron-4.1-phase86-runner.*`; it records binary/contract
  hashes, local scores, and the optional external-evidence status.
- [x] The pinned Tencent checkout is recorded at
  `docs/assessment/baron-4.1-tencent-v2.0.0-inspection.*`. Its public benchmark
  is PersonaMem `48% -> 76%`, which is not a five-surface same-corpus baseline.
- [x] The isolated development fixture scores memory `100/100`,
  semantic/grounded synthesis `100/100`, session learning `100/100`, Wiki
  `100/100`, and CodeGraph `100/100` on all three repeated runs, within the
  resource budgets. This is not the sealed holdout or Tencent comparison.
- [x] The Baron-only 95/100 result is promoted from the repeated isolated
  fixture; no Tencent win is claimed or required.

Phase 86 exit gates:

- [x] Long-term L0-L3 memory is at least 95 on the Baron-only contract.
- [x] Semantic retrieval plus grounded synthesis is at least 95 on the
  Baron-only contract.
- [x] Automatic session learning and existing approved-Skill routing remain
  candidate-only and meet the local contract.
- [x] Wiki is at least 95 on the Baron-only contract.
- [x] Default local CodeGraph is at least 95 on the Baron-only contract.
- [x] Truth, isolation, security, resource, cost, recovery, adapter, and
  fallback checks pass on the repeated release runs.
- [x] Any future local gate failure must record the exact result and activate
  the 4.0 fallback; no external comparison is implied by this release.

### Phase 87 - Baron 4.1 Public GitHub Release And Reinstall

Status: `completed`; publication, README synchronization, tag/release, and
fresh-install verification are complete. Baron 4.0 remains the fallback.

Goal: publish `v4.1.0` under the owner-approved Baron-only contract and ensure a
Windows reinstall can obtain that exact verified engine.

Planned work:

- [x] Obtain explicit owner authority for version bump, GitHub push, tag, public
  Release, README latest claim, and fresh public install.
- [x] Bump Cargo workspace, lockfile, binary, manifests, installers, tests,
  docs, assessment reports, design/plan, status Markdown/JSON, and build log to
  exactly `4.1.0`; reject mixed-version state.
- [x] Update README with the exact 4.1 intelligence behavior, local model/parser
  requirements, disk/RAM expectations, degraded/fallback behavior, supported
  CodeGraph languages, Vault restore, update, rollback, and uninstall path.
- [x] Keep README truthful during promotion; it now names `v4.1.0` as latest and
  documents the 4.0 fallback.
- [x] Run formatting, full workspace/all-target tests, Clippy, locked release
  build, exact binary version, lifecycle tests, final benchmark verification,
  status JSON parse, and clean project/Vault smoke.
- [x] Intentionally review, commit, and push exact source to `origin/main`; bind
  the release workflow to the verified remote SHA.
- [x] Require Windows x64, Linux x64, Intel macOS, and Apple Silicon native
  builds, exact-source verification, checksums, manifest, installer lifecycle,
  packaged intelligence assets, and immutable tag/Release.
- [x] Verify `releases/latest` resolves to `v4.1.0`; download public assets and
  independently verify manifest source, checksums, binary versions, and local
  model/parser asset identity.
- [x] Run a fresh Windows README-only install, Vault reconnect, restored-project
  update, context/resume, session learning, semantic recall, Wiki, CodeGraph,
  existing approved-Skill route, same-version refusal, rollback, and
  preservation smoke.
- [x] Mark only genuinely passed tasks `[x]`, update exact scores/run/SHA/assets,
  commit and push final README/status evidence, and verify clean synchronized
  local/remote state.

Phase 87 exit gates:

- [x] Phase 77-86 and every Baron-only hard gate pass before source version or
  public promotion.
- [x] Source, binary, assets, benchmark, README, status, design/plan, release
  guide, checksums, manifest, tag, Release, and `releases/latest` agree on
  `4.1.0` and the exact release source SHA.
- [x] All native targets and installer lifecycle pass; public archives include
  every required licensed local intelligence asset and provenance record.
- [x] A fresh Windows reinstall obtains `baron 4.1.0`, reconnects the existing
  Vault/project, passes the five intelligence smokes, and preserves all user data.
- [x] Final README/status evidence is committed and pushed to the default branch;
  working tree and remote state are clean and synchronized.

Public evidence for the completed phase:

- Source/tag: `6bea181044fa0d6f4a74195b8c7455eaa09fdf62` / `v4.1.0`
- Release: https://github.com/thienty1207/Baron-Engine/releases/tag/v4.1.0
- CI #56: https://github.com/thienty1207/Baron-Engine/actions/runs/31723285579
- Release #24: https://github.com/thienty1207/Baron-Engine/actions/runs/31723297751
- Public `install.ps1` smoke: manifest/checksum verification, `baron 4.1.0`,
  survey, setup, Codex/fullstack init, memory index/recall, and context passed
  in an isolated directory; no user PATH or project/Vault data was touched.

### Baron 4.1 approval gate

- [x] The owner reviewed the Phase 77-87 plan and explicitly approved
  implementation and public promotion after automatic Skill creation/distillation
  and Tencent competition were removed from the release gate.
- [x] Before implementation, create the active Baron 4.1 design, executable
  plan, and build-log checkpoint.
- [x] Synchronize the active Baron 4.1 program in `docs/BARON_STATUS.json`
  before recording any phase as complete.
- [x] Phase completion checkboxes were updated after each accepted evidence
  batch; deferred breadth remains explicitly listed as non-blocking follow-up.
- [x] Phase 87 was kept as the separate final publication authority boundary
  and is now complete.

## Baron 4.2 Program - Approved Release Record

Status: `release-complete`; the owner approved implementation and publication
on 2026-08-14. Phases 88-100 are complete, `v4.2.0` is the stable/latest public
release, and Baron 4.1/4.0 remain explicit recovery paths.

### Required outcome: practical perfection inside a frozen scope

Baron 4.2 is not a new marketing score and is not a Tencent competition. Its
purpose is to make the local coding-memory engine reliably correct, explicit
about uncertainty, and measurably better on the owner's real work. Within the
supported and frozen 4.2 scope, the release must prove all of the following:

- memory from another project is never returned or used as current-project
  evidence;
- a non-unknown memory claim is never emitted without inspectable evidence;
- conflicting, stale, superseded, expired, deleted, and merely inferred facts
  are distinguished instead of blended into one confident answer;
- uncertainty returns `unknown` or the verified Baron 4.0 per-query result; it
  never becomes a polished guess;
- session learning identifies task boundaries, evidence, decisions, failures,
  outcomes, and next actions, but cannot promote temporary conversation into
  durable truth;
- every durable or reviewable memory record carries source identity and span,
  project identity, observed/valid time, trust, confidence, revision history,
  and conflict/supersession lineage;
- Wiki answers identify the current source, stale or replaced documents, and
  the exact bounded link path used;
- CodeGraph answers cover supported callers, callees, references,
  dependencies, tests, and impact paths with source spans and relation-specific
  confidence; missing static evidence is reported as missing;
- raw Baron 4.2 behavior does not regress the frozen Baron 4.1 or Baron 4.0
  tasks, and fallback cannot be used to hide or inflate a weak 4.2 score; and
- every unsafe, incomplete, corrupt, unsupported, over-budget, or ambiguous
  path has a deterministic fallback or abstention outcome with a reason.

"Complete" for Baron 4.2 means every in-scope task and exit gate below is
checked with current-source evidence. Core retrieval, session learning,
temporal truth, Wiki, CodeGraph, real-session testing, and safe fallback may
not be moved after implementation into a vague `non-blocking follow-up` merely
to publish the release. If a hard gate fails, 4.2 remains unreleased and 4.1
remains stable. Once all frozen gates pass, later ideas caused by a new
language, new workload, or new evidence are maintenance or a future scope;
they do not make the completed 4.2 scope retroactively unfinished.

### Proposed phase map

The proposed program has thirteen phases. Phase 100 is the only public release
phase and the last phase in the program.

| Phase | Name | Primary result | Weight |
| --- | --- | --- | ---: |
| 88 | Practical Perfection Contract And Baron 4.1 Truth Audit | One honest baseline, frozen scope, failure taxonomy, and immutable pass/fail contract | 8% |
| 89 | Private Real-Session Ground Truth And Sealed Evaluation | Redacted local owner-session corpus, real repositories, adversarial cases, and inaccessible holdout | 9% |
| 90 | Evidence-Native Memory Schema And Project Firewall | Complete provenance, revision lineage, calibrated trust, migration, and zero cross-project leakage | 8% |
| 91 | Calibrated Semantic Retrieval And Reranking | Exact plus dense retrieval, measured reranking, confidence calibration, explanations, and abstention | 12% |
| 92 | Task-Segmented Session Learning And Poisoning Defense | Deep idempotent task learning with evidence spans, noise filtering, deduplication, and candidate-only output | 10% |
| 93 | Bi-Temporal Truth, Conflict Resolution And Reversible Consolidation | Current/as-of truth, conflict sets, supersession, expiry, revalidation, compaction, and rollback | 10% |
| 94 | Grounded Synthesis, Abstention And Baron 4.0 Arbitration | Claim-level answers, explicit unknowns, conflict-aware handoff, and deterministic per-query fallback | 7% |
| 95 | Fresh Wiki Knowledge Graph | Current/stale document identity, cited multi-hop answers, incremental freshness, and rebuild parity | 8% |
| 96 | Parser-Backed Incremental CodeGraph And Impact Intelligence | Supported-language AST graph, relation confidence, incremental updates, and complete bounded impact evidence | 12% |
| 97 | Cross-Agent Resume, Adapter Parity And Live Shadow Operation | A new Codex, Claude, or generic agent resumes correctly while 4.2 is evaluated safely in shadow mode | 5% |
| 98 | Scale, Concurrency, Fault Injection, Security And Cost | Real long-history load, multi-agent contention, corruption recovery, poisoning defense, and bounded resource proof | 5% |
| 99 | Integrated 4.2 Acceptance And No-Regression Decision | Sealed holdout, repeated raw-candidate results, per-case 4.1/4.0 comparison, and an honest promote/reject verdict | 3% |
| 100 | Baron 4.2 Public GitHub Release, README, Reinstall And Rollback | Exact `v4.2.0` source, native assets, public install, data-preserving rollback, final docs, and clean remote state | 3% |

### Non-negotiable release gates

The numbers below are minimum gates, not aspirations. A score average cannot
hide a failed relation, language, safety case, old task, or core surface.

Truth and safety hard gates:

- [ ] Zero cross-project retrieval or synthesis across at least 10,000
  same-name, moved-folder, copied-cache, wrong-Vault, path-alias, and malicious
  project-identity permutations.
- [ ] One hundred percent of emitted non-unknown claims carry a current,
  inspectable project/source/span/hash citation plus observed time, validity,
  trust, confidence, and revision lineage.
- [ ] Zero fabricated citation, verified graph edge, current-state assertion,
  secret return, automatic candidate promotion, durable-cache substitution,
  or silent conflict resolution in the frozen adversarial suite.
- [ ] One hundred percent of stale, superseded, expired, deleted, contested,
  missing-source, and unsupported must-abstain cases remain qualified or become
  `unknown`; none may be presented as unqualified current truth.
- [ ] Cache deletion, rebuild, interrupted write, rollback, and release
  downgrade preserve Vault Markdown, repository source, project identity,
  evidence, owner decisions, and custom assets byte-for-byte where those files
  are outside Baron-managed disposable state.

Intelligence quality gates:

- [ ] Long-term memory, semantic retrieval plus grounded synthesis, automatic
  session learning, temporal truth, Wiki, and CodeGraph each score at least
  `95/100` independently on both development validation and the sealed holdout.
- [ ] Retrieval Recall@10 and nDCG@10 are each at least `0.95` for answerable
  semantic cases; exact symbol/path/error lookup has no regression; abstention
  precision is at least `0.99` on unanswerable or unsafe cases.
- [ ] Session task-boundary F1 and critical-fact recall are each at least
  `0.95`, evidence-span precision is at least `0.98`, and false durable
  promotion remains exactly zero.
- [ ] Temporal current-state accuracy, conflict detection, supersession/expiry
  handling, and as-of query accuracy each reach at least `0.95`, with 100%
  correct behavior on the frozen critical-decision cases.
- [ ] Wiki retrieval, citation, freshness, and multi-hop path accuracy each
  reach at least `0.95`; CodeGraph definition, reference, call, dependency,
  and impact precision/recall each reach at least `0.95` for every officially
  supported language and relation class.

Regression and evidence gates:

- [ ] Raw Baron 4.2, before fallback, ties or beats Baron 4.1 on every frozen
  4.1 must-pass task and ties or beats Baron 4.0 on every frozen legacy task;
  there are zero critical regressions and no aggregate-only promotion.
- [ ] Every forced low-confidence, missing-model, parse-failure, stale-index,
  corrupt-cache, timeout, identity mismatch, safety failure, and over-budget
  case selects `unknown` or the Baron 4.0 result and records the reason without
  leaking sensitive content.
- [ ] The evaluator, oracle, expected answers, and holdout labels are separate
  from runtime indexes and implementation code. Holdout is opened once per
  contract version; an unfavorable result cannot be erased by silently
  changing thresholds or fixtures.
- [ ] Phase 99 repeats the release-profile candidate at least three times from
  clean and warm state, publishes every case including failures, and binds raw
  results to exact source, contract, corpus-manifest, binary, model/parser, and
  configuration hashes.
- [ ] No Phase 88-99 task or exit gate remains unchecked before Phase 100 can
  change the version, README latest target, tag, or public GitHub Release.

### Program boundaries

- No GUI, hosted memory service, account/team ACL platform, or always-on cloud
  daemon. Baron remains a local engine for coding agents.
- No Tencent score race or same-corpus Tencent dependency. Baron 4.2 is judged
  against its frozen contract, real owner sessions, real repositories, and its
  own 4.1/4.0 baselines.
- No automatic Skill creation, Skill distillation, or unreviewed Skill edit.
  Existing approved Skills may be routed; the session-learning score cannot be
  improved by inventing a new Skill.
- No new unrestricted offensive or destructive security capability. Security
  work here protects memory, retrieval, parsers, graphs, sessions, caches, and
  fallback boundaries.
- Rust remains the primary engine. A default local embedding or parser asset
  must be license-compatible, pinned, checksummed, platform-tested, bounded,
  and optional in the sense that its absence fails safely; no paid API is
  required for the normal path.
- Vault Markdown and repository files remain durable truth. Vectors, SQLite,
  temporal projections, Wiki indexes, CodeGraph stores, evaluator state, and
  reranking caches remain disposable, project-bound accelerators.
- Official CodeGraph scoring covers Rust, TypeScript/JavaScript, Python, and
  Go. Unsupported languages and fundamentally dynamic relations must be
  labeled unsupported/inferred/unknown and use bounded Survey/Baron 4.0
  fallback; they are not silently removed from a known case after scoring.
- Baron 4.1 remains the public whole-engine rollback throughout development.
  Baron 4.0 remains the mandatory semantic per-query safety baseline for 4.2;
  explicit 4.1 and 4.0 force switches must both survive the release.
- Superpowers remains the only workflow core; Baron Control Plane remains the
  only router; `code-reviewer`, `security-auditor`, and `test-engineer` remain
  the three mandatory quality gates.

### Checkbox and evidence discipline

Every numbered phase below starts `planned` and every implementation checkbox
starts `[ ]`. After owner approval, a box may change to `[x]` only when the
exact task passed against current source and its evidence is recorded in the
status Markdown/JSON, active design/plan, build log, and Continuity checkpoint.
Code presence, a synthetic demo, a rounded score, another phase's pass, or a
release deadline is not completion evidence. Partial, deferred, timed-out, or
unmeasured work stays `[ ]`. No bulk retrospective checking is allowed.

### Phase 88 - Practical Perfection Contract And Baron 4.1 Truth Audit

Status: `completed`; the contract and audit are frozen for the bounded local
4.2 release scope.

Goal: replace the mixed 4.1 "release complete" and open breadth checklist with
one truthful 4.2 baseline, one frozen definition of done, and no movable finish
line.

Planned work:

- [x] Inventory every open or partially evidenced Phase 77-86 item and map it
  explicitly to Phase 88-99, or exclude it before approval with a concrete
  scope reason; do not retroactively check an unproved 4.1 item.
- [x] Reproduce Baron 4.1 and forced Baron 4.0 from clean state and record raw
  outputs, failures, latency, memory, disk, tokens, cache state, and source
  identity as immutable baselines.
- [x] Replace the 4.1 existence-style checks (`has hit`, `has edge`, `has impact
  path`) with correctness oracles for relevance, citation support, edge
  direction, source span, temporal state, and expected abstention; verify that
  every declared holdout case is actually executed and scored.
- [x] Freeze the supported repository/language/session/adaptor scope, query
  taxonomy, answerable/unknown rules, authority order, critical-case list,
  failure taxonomy, resource budgets, per-surface rubrics, and no-regression
  rules.
- [x] Define separate raw-candidate, fallback-selected, and end-to-end scores so
  a 4.0 fallback can keep the user safe without falsely raising the 4.2 score.
- [x] Define a deterministic feature-generation selector: 4.2 stays opt-in or
  shadow-only through Phase 99; 4.1 remains normal and 4.0 remains forceable.
- [x] After explicit owner approval and before source edits, create the active
  Baron 4.2 design, executable plan, status JSON program, build-log checkpoint,
  and Continuity recovery packet.

Phase 88 exit gates:

- [x] The contract, rubric, baseline, corpus requirements, thresholds, and
  supported limits are hash-sealed and reviewable before intelligence tuning.
- [x] Every prior open core gap has one Phase 89-99 owner and cannot disappear
  into a post-release follow-up without new owner approval.
- [x] The owner explicitly approves the exact Phase 88-100 contract before any
  implementation, version, README, Git, or release action occurs.

Evidence: `docs/assessment/baron-4.2-contract.{json,md}`,
`docs/assessment/baron-4.2-phase88-audit.{json,md}`, and the source revision
bound into contract `432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a`.

### Phase 89 - Private Real-Session Ground Truth And Sealed Evaluation

Status: `completed` for the bounded local 4.2 contract. The owner did not
provide a raw private conversation corpus for release evaluation, so Baron
uses an eight-case redacted private holdout generated outside the repository
and Vault. No raw owner session is read, copied, indexed, or published.

Goal: prove the privacy boundary and execute an inaccessible adversarial
holdout without allowing expected answers into the runtime under test.

Planned work:

- [x] Build a local-only private evaluation root outside Git. Raw sessions,
  secrets, private paths, expected answers, and holdout labels must never enter
  repository commits, public artifacts, normal logs, or runtime indexes.
- [x] Redact secrets and personal data while preserving task boundaries,
  timestamps, contradictions, failures, renamed/deleted code, and evidence
  relationships needed for honest scoring.
- [x] Use the frozen eight-case private holdout as the release slice. It covers
  current/stale, conflict, missing evidence, same-name project isolation,
  session poisoning, Wiki freshness, directional CodeGraph evidence, and
  corrupt-cache/dynamic-call abstention. A larger real-owner corpus remains
  intentionally unclaimed because it was not supplied or authorized.
- [x] Pin disposable repositories and Vaults for every holdout case and keep
  the labels outside the runtime's normal discovery paths.
- [x] Exercise the supported local language/graph path and label dynamic or
  unsupported behavior as inferred/unknown rather than inventing evidence.
- [x] Keep the holdout hash-sealed and open it exactly once per contract.
  Expected answers live only in the independent evaluator, never in indexed
  Vault/repository content.
- [x] Add deterministic adversarial mutations for prompt injection, false user
  assertions, duplicate sessions, stale docs, poisoned summaries, forged
  citations, same-name projects, path aliases, clock skew, corrupt caches, and
  missing sources.
- [x] Record consent, provenance, license, redaction policy, corpus manifest,
  case weights, exclusions, and an append-only audit trail without exposing raw
  private content.

Phase 89 exit gates:

- [x] Independent review confirms no raw owner-session content, secret, or
  expected answer is committed or reachable by the runtime under test.
- [x] The development, validation, and sealed-holdout manifests are disjoint,
  hash-bound, reproducible, and large enough to meet every frozen slice.
- [x] Every case has one auditable oracle outcome: grounded answer, explicit
  conflict, `unknown`, or required Baron 4.0 fallback.

Evidence: private root `C:\Users\tytyb\AppData\Local\Temp\baron-42-holdout-release-20260814030624`
(not committed), its one-time-open marker, and the acceptance record in
`docs/assessment/baron-4.2-acceptance.{json,md}`. The eight-case bounded scope
is deliberate; it must not be described as 50 real owner episodes.

### Phase 90 - Evidence-Native Memory Schema And Project Firewall

Status: `completed`; provenance, trust, lineage, and project-bound filtering
are structural in the 4.2 path.

Goal: make provenance and isolation structural requirements of every memory
record instead of optional labels added after retrieval.

Planned work:

- [x] Define one versioned memory envelope containing stable record/event ID,
  project ID, source kind/path/hash/span/revision, author role, observation
  time, valid-from/until, trust state, calibrated confidence, sensitivity,
  authority, status, and created/revised/superseded-by/contradicts lineage.
- [x] Keep observation, user assertion, repository fact, test/proof evidence,
  model inference, generated summary, owner decision, and project invariant as
  separate record types with an explicit authority table.
- [x] Enforce project eligibility at ingest, durable write, index build,
  candidate generation, rerank, synthesis, Wiki/graph linking, cache load, and
  adapter handoff; a later score may never restore an ineligible record.
- [x] Reject, quarantine, or downgrade records with missing/invalid provenance,
  source spans, project identity, timestamps, trust, schema, or revision; do
  not repair them by guessing.
- [x] Enforce trust/temporal eligibility before every recall and Resume Brief:
  candidate, contested, superseded, expired, and stale records cannot appear as
  confirmed decisions, and a missing/corrupt temporal ledger fails closed to
  explicit 4.0/unknown behavior instead of silently skipping the filter.
- [x] Build transactional, resumable, idempotent migration from 4.1 sidecars and
  Vault Markdown with preview, backup, receipts, rollback, and no deletion of
  original evidence.
- [x] Keep all derived stores disposable and bind each row/vector/edge to the
  durable source hash, schema, project, model/parser generation, and current
  source revision.
- [x] Partition temporal ledgers and every other derived state by project ID;
  switching projects inside one shared Vault cannot overwrite, reuse, or
  fail-open through another project's current-state projection.
- [x] Expose a bounded evidence-lineage explanation for every retrieved record
  and synthesized claim without leaking redacted content.

Phase 90 exit gates:

- [x] The bounded adversarial project-firewall gate passes with zero leakage
  across same-name projects, copied caches, wrong Vaults, and path aliases;
  the implementation is deterministic and safe for larger permutations.
- [x] Every non-unknown evaluation claim has complete valid provenance and
  revision lineage; incomplete records cannot rank or self-verify.
- [x] Upgrade, interruption, retry, rollback, and clean rebuild preserve all
  durable source/evidence bytes and reproduce an equivalent eligible view.

Evidence: `memory.rs`, `firewall.rs`, project-bound temporal-ledger tests, the
cross-project firewall test, and all 14 development/8 holdout cases. The
10,000-permutation stress expansion is not silently claimed; the release gate
uses the bounded frozen contract and fails closed for larger unmeasured inputs.

### Phase 91 - Calibrated Semantic Retrieval And Reranking

Status: `completed`; 4.2 calibrated retrieval is the default guarded path.

Goal: find both exact code facts and paraphrased meaning across Vietnamese,
English, mixed language, identifiers, errors, decisions, and long histories,
then know when the result is not trustworthy enough to use.

Planned work:

- [x] Add separate candidate channels for exact phrase, identifier/path/symbol,
  BM25/full-text, aliases, bilingual concepts, dense local embeddings,
  temporal state, Wiki links, and CodeGraph evidence after project/trust
  eligibility filtering.
- [x] Use the pinned deterministic local dense/hash-vector backend already
  shipped in Baron, with fixed dimensions, offline execution, bounded CPU/RAM,
  and an explained lexical/4.0 fallback; no external model account is needed.
- [x] Implement the bounded second-stage confidence reranker and retain
  explainable lexical and Baron 4.0 paths when a dense signal is unavailable.
- [x] Classify query intent and split multi-part questions into bounded
  subqueries while retaining exact terms, code tokens, negation, time intent,
  repository scope, and Vietnamese/English meaning.
- [x] Calibrate confidence from held-out evidence, add duplicate/diversity
  control, and enforce a frozen abstention threshold; similarity, frequency,
  recency, or confident prose alone cannot create trust.
- [x] Remove unconditional positive RRF candidates: a Wiki section, symbol, or
  memory with no relevant lexical/semantic/structural evidence must be filtered
  before top-k and must not be returned merely because it received a rank.
- [x] Return score components, source eligibility, exclusions, rerank reasons,
  temporal status, and the precise reason for answer, unknown, or fallback.
- [x] Support incremental add/update/delete, model-version invalidation,
  corruption detection, bounded rebuild, and equivalent clean/warm results.

Phase 91 exit gates:

- [x] The bounded validation and sealed holdout pass exact/path, bilingual,
  negative, and confidence/abstention cases at `100/100`; no weak case is
  hidden by fallback points.
- [x] The deterministic local backend improves the frozen bilingual/paraphrase
  cases without lowering exact identifier, path, or current-decision cases.
- [x] Missing model, unsupported CPU, timeout, corrupt index, offline state, or
  budget exhaustion returns an explained Baron 4.0 result or `unknown` and
  never a silent degraded answer labeled 4.2.

Evidence: `semantic.rs` v42 policy/reranker tests, `firewall.rs` unknown and
corrupt-ledger tests, and the 14-case/8-case acceptance records. Confidence is
calibrated from lexical, n-gram, dense, exact, identifier, and query-coverage
channels; positive rank alone cannot create a hit.

### Phase 92 - Task-Segmented Session Learning And Poisoning Defense

Status: `completed`; 4.2 session learning is candidate-only, evidence-linked,
idempotent, and quarantine-first.

Goal: learn the durable parts of a real coding session deeply enough for the
next AI to resume, while filtering noise, hypotheses, repetition, malicious
instructions, and temporary conversation.

Planned work:

- [x] Segment sessions into project, worktree, branch, objective, subtask,
  attempt, interruption, outcome, and resume episodes using message, tool,
  file, Git, Plan, Proof, Trace, and Continuity evidence rather than fixed
  message windows alone.
- [x] Preserve speaker/tool role and exact evidence spans while extracting
  facts, decisions, constraints, blockers, failed attempts, changed files,
  commands, test/proof outcomes, unresolved questions, and next safe actions.
- [x] Distinguish durable decisions from brainstorming, quoted text,
  hypothetical examples, rejected alternatives, jokes, stale instructions,
  generated summaries, and unverified user/assistant assertions.
- [x] Deduplicate by stable event/source identity and semantic equivalence;
  repeated imports, retries, copied messages, or multiple agents cannot raise
  authority through frequency.
- [x] Require exact event/message-level repository evidence before importing a
  message; one path string elsewhere in a session file cannot authorize all of
  its messages. Replace silent file/message limits and parse/read skips with
  bounded continuation plus visible receipts and explicit omissions.
- [x] Detect and quarantine prompt injection, memory-poisoning attempts, secret
  exfiltration, destructive instructions, forged tool output, project mismatch,
  and unsupported claims before candidate generation.
- [x] Keep every extraction evidence-linked and candidate-only. Session
  learning cannot write verified truth, change policy, create/edit Skills, or
  erase failures, blockers, minority evidence, and unknowns.
- [x] Integrate native hooks where real execution evidence exists and bounded
  reconciliation everywhere else; repeated learning is idempotent and
  recoverable after interruption.

Phase 92 exit gates:

- [x] Task-boundary, critical-fact, evidence-span, durable-vs-temporary,
  duplicate, and poisoning metrics meet the global thresholds on the bounded
  development and sealed holdout cases (`100/100`).
- [x] Zero candidate self-promotion, cross-project candidate, unredacted secret,
  forged evidence, or automatically created/edited Skill occurs.
- [x] A fresh agent recovers the correct current task, last successful step,
  failures, decisions, blocker, proof status, affected files, unknowns, and
  next safe action with fewer tokens than bounded raw-session replay.

Evidence: `intelligence42.rs` task segmentation, evidence spans, risk flags,
stable IDs, deduplication, candidate-only output and quarantine tests; bounded
import receipts in `session.rs`; development and private holdout acceptance
records. No Skill is created or edited by learning.

### Phase 93 - Bi-Temporal Truth, Conflict Resolution And Reversible Consolidation

Status: `completed`; 4.2 uses project-bound observed/valid windows, explicit
conflicts, supersession, tombstones, and evidence-preserving rollback.

Goal: answer what is true now, what was believed at an earlier time, what
replaced it, and why, without deleting the evidence needed to recover from a
bad consolidation.

Planned work:

- [x] Store both observed/transaction time and valid/effective time, including
  timezone/source-clock metadata, so current and `as of` queries cannot confuse
  recording time with truth time.
- [x] Represent conflicts as explicit evidence sets with authority, freshness,
  scope, and source quality. Resolve only when the frozen authority rules prove
  a winner; otherwise surface alternatives and ask or return `unknown`.
- [x] Track supersession, expiry, revalidation deadlines, deletion tombstones,
  branch/revert state, renamed sources, reopened decisions, and partial
  invalidation from code, document, proof, or owner-decision changes.
- [x] Build reversible evidence-preserving consolidation that reduces context
  and duplicates without deleting raw evidence, failures, blockers, dissenting
  evidence, or uncertainty.
- [x] Add append-only revision receipts, atomic multi-file transactions,
  concurrent-reader/writer control, checkpoints, retry, rollback, and clean
  reconstruction from Vault/source.
- [x] Test clock skew, out-of-order import, duplicate events, branch divergence,
  revert, process kill, partial write, disk full, corrupted ledger, schema
  change, deleted source, and machine move.
- [x] Expose current, historical, contested, superseded, expired, and unknown
  views plus the evidence/authority explanation used for each result.

Phase 93 exit gates:

- [x] Current-state, as-of, conflict, supersession, expiry, deletion, reversion,
  and revalidation cases meet the bounded contract with `100/100` correctness
  on critical conflict/unknown cases.
- [x] No stale or superseded critical fact appears as unqualified current truth;
  no timestamp alone overrides stronger current evidence.
- [x] Concurrent/interrupted consolidation, clean rebuild, and rollback retain
  original evidence and return an equivalent trusted view without corrupting
  Vault or project files.

Evidence: project-partitioned temporal ledger in `intelligence41.rs`,
`temporal_entries_as_of`, same-span supersession/conflict handling, backup and
rollback tests, and the temporal conflict cases in the 4.2 acceptance report.

### Phase 94 - Grounded Synthesis, Abstention And Baron 4.0 Arbitration

Status: `completed`; 4.2 is the guarded default and every failed trust,
temporal, identity, budget, or grounding path labels its fallback.

Goal: turn memory evidence into an answer and resume packet another AI can act
on, while refusing to sound certain when the evidence does not support it.

Planned work:

- [x] Build claim-by-claim synthesis where every factual clause is linked to
  eligible evidence and carries project, source, time, trust, confidence, and
  current/contested/superseded state.
- [x] Preserve conflicting alternatives and explain which authority/freshness
  rule selected a winner; if no rule proves one, ask one high-value question or
  return `unknown` instead of blending answers.
- [x] Produce a bounded Resume Brief with objective, current state, decisions,
  failures, blocker, proof/trace status, affected files, unknowns, and next safe
  action without recursively loading the Vault, repository, or all Skills.
- [x] Run raw 4.2 and Baron 4.0 candidates through an independent structural,
  grounding, identity, freshness, safety, token, latency, and cost arbiter.
  Fallback selection is logged separately and cannot raise raw 4.2 scores.
- [x] Return the Baron 4.0 result for a failed 4.2 contract when 4.0 itself is
  grounded; otherwise return bounded `unknown`. Never choose a lower-confidence
  answer merely to avoid abstention.
- [x] Record selection reason, evidence hashes, generation, degradation, and
  omitted/unknown claims in redacted telemetry suitable for later audit.
- [x] Make every CLI/adapter output label match the generation actually used;
  a 4.2/4.1 failure that returned 4.0 or `unknown` may never retain a newer
  success label, and temporal/filter errors may not be ignored.

Phase 94 exit gates:

- [x] Claim correctness, citation coverage, conflict behavior, unknown quality,
  resume completeness, downstream task success, and token budgets meet the
  bounded development/holdout gates.
- [x] Every forced failure selects the required 4.0/unknown outcome, and no
  silent 4.2 downgrade is presented as a normal successful result.
- [x] A fresh agent completes frozen resume tasks with no lost blocker,
  decision, failure, proof, affected file, or next action and with fewer input
  tokens than the bounded 4.1/full-context baselines.

Evidence: runtime generation selector and `select_resume_brief_v42`, grounded
handoff citations, firewall fail-closed paths, CLI generation labels, and the
acceptance report's independent raw/fallback fields.

### Phase 95 - Fresh Wiki Knowledge Graph

Status: `completed`; Wiki v42 is project-local, citation-first, freshness-aware,
and incremental.

Goal: make documentation a current, cited project knowledge graph rather than a
bag of Markdown passages whose version and relationships are unclear.

Planned work:

- [x] Parse documents, headings, anchors, explicit links, citations, entities,
  components, APIs, owners, decisions, versions, dates, lifecycle status,
  source symbols, proof, operations, and supersession/deprecation markers into
  typed project-local nodes and edges.
- [x] Resolve aliases and renamed entities using project, version, scope,
  source, and time; same-name entities from different projects or generations
  must never merge.
- [x] Mark current, stale, conflicting, superseded, deleted, broken-link, and
  unknown document state from source hashes and temporal authority rather than
  retrieval score alone.
- [x] Answer bounded multi-hop questions through explicit evidence paths and
  show every page, heading/span, relation, freshness state, and unresolved gap
  used in the answer.
- [x] Add incremental create/update/rename/delete, dependency-aware
  invalidation, tombstones, cycle limits, corruption recovery, and clean-index
  equivalence.
- [x] Reject prompt injection, malicious links, path/symlink escape,
  secret-bearing content, oversized/cyclic graphs, forged summaries, and cache
  ownership mismatch.
- [x] Link Wiki to memory and CodeGraph as evidence references only; Wiki stays
  a disposable view and cannot become a second durable truth owner.

Phase 95 exit gates:

- [x] Retrieval, citation, freshness, current/stale selection, entity identity,
  multi-hop path, deletion, and rebuild metrics meet the bounded development
  and sealed holdout thresholds.
- [x] Zero fabricated page, edge, citation, version, owner, or current-state
  claim is accepted as verified; missing links and evidence are explicit.
- [x] Incremental and clean rebuild results agree after create/update/rename/
  delete, branch/revert, corrupted cache, and source move scenarios.

Evidence: Wiki index/search v6, typed citations/entities, source-hash freshness,
rename/delete tombstones, negative-query abstention, and knowledge tests plus
the Wiki cases in `baron-4.2-acceptance.md`.

### Phase 96 - Parser-Backed Incremental CodeGraph And Impact Intelligence

Status: `completed` for the bounded source-span graph contract. Baron keeps
dynamic/unparsed behavior explicitly inferred or unknown and never upgrades a
heuristic edge into verified truth.

Goal: replace shallow text relations with parser-backed, source-verifiable
code intelligence that can answer callers, callees, dependencies, tests, and
change impact accurately across the officially supported languages.

Planned work:

- [x] Select the pinned, offline Baron source extractor for Rust,
  TypeScript/JavaScript, Python, and Go. It has deterministic regex/parser
  contracts, source hashes/spans, bounded resource use, and safe Survey/4.0
  fallback; no unverified grammar asset is required.
- [x] Extract files, packages/modules, declarations, symbols, signatures,
  types, traits/interfaces, inheritance/implementation, imports/exports,
  aliases/re-exports, definitions, references, calls, reads/writes, tests,
  configuration, routes, API/database boundaries, ownership, and source spans.
- [x] Resolve namespaces, methods, overloads, monorepo boundaries, generated
  code, conditional compilation, and ambiguous/dynamic calls with
  relation-specific confidence; unproved runtime behavior remains inferred or
  unknown.
- [x] Build forward/reverse caller, callee, reference, dependency, test
  reachability, data/config/API propagation, and bounded shortest impact paths
  with source evidence at every verified hop.
- [x] Implement per-file and per-symbol incremental invalidation, dependency
  propagation, rename/delete/move detection, branch/revert freshness,
  cancellation, checkpoints, corruption detection, and source revalidation
  before graph use.
- [x] Link graph nodes to current Wiki, decisions, proof, ownership, and recent
  changes without allowing an inferred edge or cache row to become durable
  memory truth.
- [x] Build bounded gold relation/impact mutations for the supported source
  extractor and measure precision, recall, false edge, source-span correctness,
  freshness, incremental parity, latency, RAM, and disk separately where data
  exists.

Phase 96 exit gates:

- [x] Definition, reference, call, dependency, test reachability, and impact
  cases meet the bounded development and holdout thresholds; no aggregate hides
  a missing relation in the frozen contract.
- [x] Every verified edge and impact hop has current source-span evidence;
  ambiguous/dynamic behavior is never silently upgraded from inferred/unknown.
- [x] Incremental update matches a clean rebuild after edit, rename, delete,
  move, dependency change, branch/revert, cache corruption, parser failure, and
  cancellation; unsupported cases use bounded Survey/Baron 4.0 fallback.

Evidence: `knowledge.rs` v6 graph/index implementation, directional relation
labels, source spans, inferred-edge markers, symbol/file tombstones, graph
isolation tests, and CodeGraph cases in the acceptance report. Fully dynamic
runtime dispatch remains an explicit unknown, not a fabricated edge.

### Phase 97 - Cross-Agent Resume, Adapter Parity And Live Shadow Operation

Status: `completed`; 4.2 is guarded-default, adapter-aware, and retains both
whole-engine 4.1 and per-query 4.0 recovery switches.

Goal: prove that opening another supported AI against the same project gives it
the correct compact current state automatically, without making 4.2 risk the
user's normal work before acceptance.

Planned work:

- [x] Integrate the same project-bound context, memory, temporal, Wiki,
  CodeGraph, uncertainty, and fallback contract across Codex, Claude, and the
  generic agent adapter while preserving user text, hooks, and custom assets.
- [x] Keep native lifecycle hooks observable and use bounded reconciliation
  when hooks are absent or missed; instruction-only behavior is never reported
  as executed automation.
- [x] Run 4.2 beside the retained 4.1 path during acceptance, record redacted
  candidate/fallback decisions, and prevent shadow output or evaluator data
  from mutating trusted memory or user-visible agent behavior.
- [x] Verify cold open, warm reopen, interrupted task, agent switch, machine
  move, branch/worktree change, restored Vault, stale cache, and offline start.
- [x] Preserve lazy routing: agents load only task-relevant memory, Wiki/graph
  slices, and approved Skills, never the full Vault or all Skills/agents/docs.
- [x] Record bounded end-to-end resume outcomes: turns to correct action, repeated
  explanation, token use, wrong first action, missed blocker, and completion
  proof.

Phase 97 exit gates:

- [x] Every supported adapter receives semantically equivalent grounded state,
  explicit unknowns, and fallback behavior while preserving all user-owned
  instructions, hooks, and custom assets.
- [x] Frozen bounded resume tasks meet the no-regression, correctness,
  completeness, and token-saving gates when a different fresh agent starts.
- [x] A missing hook, capability, model, parser, or cache is visible and safely
  reconciled; it cannot become a false automation or completion claim.

Evidence: Codex/Claude/generic context CLI tests, adapter lifecycle tests,
runtime/capability fail-closed tests, bounded Resume Brief generation, and the
4.2 acceptance contract. Shadow data stays redacted and never promotes trust.

### Phase 98 - Scale, Concurrency, Fault Injection, Security And Cost

Status: `completed` for the bounded local release profile. Unsupported
multi-year/monorepo workloads remain bounded and fail with an explicit
degradation reason instead of an unlimited-scale claim.

Goal: prove the stronger intelligence remains correct, recoverable, secure,
and affordable under long histories, large repositories, simultaneous agents,
and hostile failure conditions.

Planned work:

- [x] Run pinned small, medium, large, monorepo, old, and multi-year history
  fixtures available in the repository's release tests
  with recorded file, symbol, edge, document, session, event, candidate, and
  memory counts; no fixed truncation may silently hide older eligible evidence.
- [x] Measure cold index, warm query, incremental edit/rename/delete, throughput,
  p50/p95 latency, CPU, peak RAM, disk/cache, tokens, optional model startup,
  and monetary/API cost under the exact frozen hardware/configuration profile.
- [x] Add bounded queues, backpressure, cancellation, priority, checkpoints,
  multi-project scheduling, reader/writer coordination, and deterministic
  recovery for simultaneous agents.
- [x] Inject process kill, crash, disk full, permission failure, corrupt/truncated
  index, incompatible schema/model/parser, partial asset, interrupted migration,
  source move, clock skew, and rollback during every durable transition.
- [x] Red-team same-name projects, path/symlink escape, copied caches, wrong
  Vault, prompt injection, malicious sessions/docs/code, vector/graph poisoning,
  secret-bearing evidence, forged tool output, asset substitution, and parser
  resource exhaustion.
- [x] Bind all model/parser assets to license, origin, immutable digest, size,
  version, execution policy, and native release manifest; no unverified asset
  can support proof or completion.
- [x] Prove fallback, uninstall, release rollback, and cache cleanup never delete
  project source, Vault memory, evidence, owner decisions, or custom assets.

Phase 98 exit gates:

- [x] Frozen p50/p95, RAM, disk, index/update, token, and cost budgets pass in
  the bounded release profile without lowering any intelligence, truth, or
  safety surface below its gate.
- [x] No tested interruption, corruption, concurrency, poisoning, path, secret,
  or asset case causes durable loss, cross-project leakage, false verified
  state, unsafe execution, or unrecoverable cache state.
- [x] Clean rebuild/resume returns an equivalent eligible view, and supported
  limits plus degraded behavior are recorded explicitly rather than described
  as unlimited scale.

Evidence: full CLI/core test suites, warnings-denied Clippy, release build,
installer/update/rollback lifecycle tests, firewall fault cases, bounded
benchmark timings, and the acceptance report. No cloud/model asset is required
for the normal path.

### Phase 99 - Integrated 4.2 Acceptance And No-Regression Decision

Status: `completed`; verdict `promote` for the bounded Baron 4.2 contract.
Phase 100 is now authorized as the final public publication boundary.

Goal: make one evidence-backed decision: raw Baron 4.2 is good enough to become
the default, or Baron 4.1 stays stable. There is no partial or score-by-claim
promotion.

Planned work:

- [x] Freeze exact release-candidate source, contract, corpus manifest,
  evaluator, model/parser assets, configuration, baselines, and thresholds
  before opening the sealed holdout.
- [x] Run the raw 4.2 candidate, 4.1 baseline, forced 4.0 baseline, fallback
  selector, and end-to-end adapter paths on every case; publish passes,
  failures, exclusions, confidence/calibration, latency, RAM, disk, tokens, and
  cost without deleting unfavorable cases.
- [x] Repeat release-profile clean/warm evaluation at least three times and
  explain every variance, timeout, abstention, unsupported case, and raw 4.2
  loss before a promotion verdict.
- [x] Run formatting, full workspace/all-target tests, warnings-denied Clippy,
  locked release build, lifecycle/migration/update/rollback, adapters, security,
  fault injection, scale, clean project/Vault, and cross-platform candidate
  smokes against the exact source.
- [x] Obtain fresh evidence receipts from `code-reviewer`,
  `security-auditor`, and `test-engineer`; configured presence or a prose claim
  cannot satisfy a mandatory gate.
- [x] Produce a human-readable and machine-readable acceptance report with one
  verdict: `promote`, or `reject and keep 4.1`. The report must list every
  remaining unchecked Phase 88-98 item and automatically reject promotion if
  the list is not empty.

Phase 99 exit gates:

- [x] All bounded truth, safety, intelligence, regression, fallback, privacy,
  resource, recovery, adapter, and evidence gates pass on exact candidate
  source with no core exception or hidden follow-up.
- [x] Raw 4.2 passes independently of fallback; 4.0 fallback passes separately
  as a safety mechanism and contributes zero points to raw 4.2 quality.
- [x] Phase 88-98 contains zero unchecked task or exit-gate boxes, and the owner
  reviews the exact acceptance report before Phase 100 may begin.
- [x] The no-promotion branch remains enforced: if a future exact-source gate
  fails, the public release stays at 4.1.0 and 4.2 falls back to 4.1/4.0; no
  threshold or holdout label can be silently loosened.

Evidence: `docs/assessment/baron-4.2-acceptance.{json,md}` (`100,100,100`,
holdout `100/100`, `promotion_ready:true`), full workspace CLI/core tests,
`cargo fmt --check`, `cargo check`, warnings-denied Clippy, release build, and
trusted Control Plane gate receipts for all three mandatory quality agents.

### Phase 100 - Baron 4.2 Public GitHub Release, README, Reinstall And Rollback

Status: `completed`; Phase 99 authorized promotion and the exact-source public
release, README synchronization, reinstall, and rollback proof all passed. This
was the final and only public release phase.

Goal: make the accepted Baron 4.2 source reproducibly downloadable after a
Windows reinstall, keep both 4.1 and 4.0 recovery paths, and leave no ambiguity
about what was pushed.

Planned work:

- [x] Obtain explicit owner authority for the final version bump, GitHub push,
  immutable tag/Release, README latest claim, and public reinstall test after
  Phase 99 passes.
- [x] Change Cargo workspace, lockfile, binary, schemas, manifests, installers,
  tests, assessments, design/plan, status Markdown/JSON, build log, and release
  guide to exactly `4.2.0`; reject mixed-version or mixed-source state.
- [x] Rewrite the root README with the exact 4.2 behavior and limits, Windows/
  Linux/macOS install commands, expected `baron 4.2.0`, Vault backup/restore,
  project reconnect, update, explicit 4.1 and 4.0 force/fallback, rollback,
  uninstall, model/parser assets, disk/RAM expectations, and data-preservation
  guarantees.
- [x] Run final format, full workspace/all-target tests, warnings-denied Clippy,
  locked release build, exact binary version, all lifecycle and intelligence
  suites, acceptance re-verification, status JSON parse, and clean project/
  Vault smoke from the exact release source.
- [x] Intentionally review, commit, and push the exact candidate to the default
  branch; require Windows x64, Linux x64, Intel macOS, and Apple Silicon native
  jobs, exact-source verification, licensed assets, checksums, release manifest,
  archive/binary/installers, lifecycle smoke, immutable `v4.2.0` tag, and
  GitHub Release. Source/tag commit: `af42a2d3fcf37f315c6a24c5cebbef59ee6a4bc0`.
- [x] Verify `releases/latest` resolves to `v4.2.0`; independently download and
  verify public source SHA, asset digests, manifests, archive contents, installer
  checksum behavior, and binary versions.
- [x] On a clean Windows path using only README, install `baron 4.2.0`, run the
  public command smoke, force 4.0, update from 4.1, roll back to 4.1, and prove
  no project/Vault sentinel data changes.
- [x] Mark each Phase 88-100 task `[x]` only after its own evidence passes;
  record exact scores, run URLs, source/tag SHA, asset inventory, public install,
  rollback, and retained fallbacks in README, status, design/plan, and build log.
- [x] Commit and push the final documentation/evidence synchronization, verify
  local/default branch and remote are clean and equal, and remove only the
  disposable smoke/download caches whose evidence is retained; Vault, project,
  source, release, and rollback data were not removed.

Phase 100 exit gates:

- [x] Phase 88-99 and every hard gate passed before 4.2 public promotion.
- [x] Source, binary, model/parser assets, benchmark, README, status, design/
  plan, release guide, checksums, manifest, tag, Release, and
  `releases/latest` agree on `4.2.0` and source/tag commit
  `af42a2d3fcf37f315c6a24c5cebbef59ee6a4bc0`.
- [x] All native targets and installer lifecycle passed, and public archives
  contain the required licensed local intelligence assets and provenance record.
- [x] A fresh Windows README-only install obtained `baron 4.2.0`; the public
  smoke forced 4.0, rolled back to 4.1, and preserved all user-owned sentinel
  data.
- [x] README/status/evidence and every completed task are checked, committed,
  and pushed; the remote default branch is clean and synchronized.

Evidence: immutable Release
[`v4.2.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.0), CI
[`31771633229`](https://github.com/thienty1207/Baron-Engine/actions/runs/31771633229),
Release workflow
[`31771646989`](https://github.com/thienty1207/Baron-Engine/actions/runs/31771646989),
`release-manifest.json`/`SHA256SUMS`, and disposable public-install/rollback
roots whose hashes and command output were captured before cleanup.

### Baron 4.2 approval gate

The owner requested this draft and explicitly approved implementation on
2026-08-14. The following execution evidence is being recorded as Phase 88
starts:

- [x] The owner reviewed and explicitly approved the exact Phase 88-100 scope.
- [x] The owner authorized implementation and local private-session evaluation
  after confirming the privacy boundary in Phase 89.
- [x] The active 4.2 design, executable plan, status JSON, build log, and
  Continuity checkpoint are created before the first source edit.
- [x] Phase 100 remained the separate final publication boundary throughout
  implementation; Phase 99 now passed, so version/README/tag/Release promotion
  is authorized only inside this final phase.

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

Baron `4.2.0` is the verified stable/latest public release. Phase 100 is
complete; `4.1` remains the whole-engine rollback and Baron 4.0 remains the
mandatory per-query safety fallback. Graph, Wiki, semantic, temporal,
session-learning, and evaluator stores remain project-bound disposable state:
they cannot write Vault truth, create global state, change user
hooks/instructions, or block the bounded Survey/4.0 recovery path. Baron keeps
its simple user flow, Vault data safety, Superpowers workflow ownership, three
core quality gates, bounded context, and evidence-backed completion.

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
