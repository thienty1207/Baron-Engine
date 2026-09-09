# Baron Engine — Codex + Claude Core Consolidation, Intelligence Optimization, and retired adapter Removal Spec

Status: implementation completed through Phase 15; Phase 16 verification pending
(local verification completed, release blocked pending trust-boundary remediation)
Target: Baron Engine with exactly two active adapters — **Codex** and **Claude**
Primary priority: **remove retired adapter completely from the tracked current product**
Primary architecture: **Baron Core owns intelligence, skills and workflows; adapters consume Core**
User model: **non-technical user; normal work must not require Baron CLI micromanagement**

Phase 16 release note: local adversarial and regression gates pass, but v5.0.0
publication is intentionally blocked until the release contract defines an
authenticity mechanism for archive metadata and a stronger authority for
persisted capability/gate evidence. HTTPS, host allowlisting, and unkeyed local
receipts are insufficient against a compromised trusted source or same-account
local writer. This note records a release constraint; it does not authorize
the implementation of a later phase.

---

# 1. Product intent

This refactor is not a feature reduction and not an adapter-only cleanup.

The final product must preserve or improve Baron's existing intelligence while narrowing host integration to Codex and Claude.

The product contract is:

```text
Baron Core = brain + durable state + workflows + skills + routing + evidence
Codex      = native adapter/consumer
Claude     = native adapter/consumer
```

Baron must become easier for a non-technical user to use:

```text
one-time setup/init
        ↓
open Codex or Claude normally
        ↓
ask for work naturally
        ↓
Baron automatically resumes/routs/plans/verifies/remembers
```

The human must not need to know hidden commands such as `baron context`, `baron plan`, `baron route`, `baron proof`, or `baron continuity` during normal work.

Those commands may remain as internal/diagnostic surfaces used by the adapters, tests, maintainers and recovery tooling.

---

# 2. Non-negotiable outcomes

1. Active adapters are exactly:
   - Codex
   - Claude
2. retired adapter is completely removed from tracked current product source/docs/tests/assets/CLI.
3. Generic Agent is removed as an active adapter, but no Core capability is removed with it.
4. `assets/core/**` remains the packaged Baron source of truth.
5. `.baron/core/**` becomes the single project-local canonical runtime copy.
6. Codex does not own a copied Baron skill tree.
7. Claude does not own a copied Baron skill tree.
8. Superpowers remains the single workflow owner.
9. Existing Baron memory, continuity, Harness, proof, trace, CodeGraph, Wiki, session replay, capability, recovery and safety behavior may not regress.
10. Platform flags must change real routing/verification behavior, not merely write labels/docs.
11. Normal user work is automatic; CLI micromanagement is not part of the product UX.
12. Existing user-owned files are never silently overwritten or deleted.
13. Existing project/Vault identity and durable history must survive migration.
14. All completion claims remain evidence-backed.

---

# 3. Preserve-first intelligence rule

No existing Baron Core capability may be removed or weakened merely to simplify this refactor.

A change that makes any of these worse is a regression unless backed by an explicit correctness reason and replacement proof:

- project-ID isolation;
- Vault Markdown durability;
- memory trust filtering;
- multilingual retrieval;
- bounded session replay;
- task resume;
- intent/constraints/non-goals;
- plan continuity;
- recovery packets;
- Product Harness;
- proof/trace gates;
- capability execution evidence;
- CodeGraph/Wiki source verification;
- redaction;
- skill contracts;
- mandatory quality-agent gates;
- update rollback;
- user-file preservation;
- cross-platform support.

The expected direction is:

```text
existing Baron intelligence
+ safer state management
+ better routing
+ better task continuity
+ better profile awareness
+ simpler automatic adapter protocol
= new Baron
```

---

# 4. Final architecture

## 4.1 Packaged source

```text
assets/core/
  skills/
  agents/
```

These are embedded/build-time Baron assets.

## 4.2 Canonical runtime Core

Every initialized project receives:

```text
.baron/
  project.toml
  local.toml
  managed-state/
  core/
    skills/
      INDEX.md
      superpowers/
      frontend-design/
      api-and-interface-design/
      observability-and-instrumentation/
      performance-optimization/
      deprecation-and-migration/
      vibe-security-scan/
      database-engineering/
      mobile-application-engineering/
      [other approved Baron core skills]
    agents/
      INDEX.md
      code-reviewer.*
      security-auditor.*
      test-engineer.*
      [canonical shared agent contracts]
```

There is exactly one canonical runtime copy of every Baron-managed skill per project.

## 4.3 Adapter relationship

```text
                     .baron/core/**
                           │
             ┌─────────────┴─────────────┐
             │                           │
           Codex                       Claude
             │                           │
       native bridge/wrappers      native bridge/wrappers
```

Adapter files may translate Core contracts into host-native formats, but must not become new workflow sources of truth.

---

# 5. Managed ownership model

Current `(adapter, path)` ownership is insufficient for canonical Core.

Introduce an explicit owner concept, e.g.:

```rust
enum ManagedOwner {
    Core,
    Adapter(SupportedAdapter),
}

enum SupportedAdapter {
    Codex,
    Claude,
}
```

Equivalent representations are acceptable.

Invariant:

```text
one live managed path -> exactly one owner
```

Examples:

```text
Core:
  .baron/core/**

Adapter(Codex):
  AGENTS.md Baron block
  .agents/skills/baron-engine/**
  .codex/INDEX.md
  .codex/agents/**
  .codex/hooks.json Baron-owned entries

Adapter(Claude):
  CLAUDE.md Baron block
  .claude/skills/baron-engine/**
  .claude/agents/**
  .claude/settings.json Baron-owned entries
  .claude/commands/** Baron-owned diagnostic commands
```

Validate duplicate live target ownership before any mutation.

Core payloads are planned once per init/update even if both adapters are installed.

---

# 6. Complete retired adapter removal — highest priority

## 6.1 Remove active implementation

Delete every retired adapter-specific active surface, including:

- adapter enum variants;
- CLI flags/shortcuts;
- init/context/update routes;
- runtime/capability mappings;
- automation hook adapter variants;
- context targets;
- install functions;
- settings/hook generators;
- commands;
- blueprints;
- tests;
- docs;
- README mentions;
- CHANGELOG entries on the current tracked branch if they retain the retired product surface;
- status/build plans/specs dedicated to that adapter;
- legacy projection/** generation;
- legacy instruction file generation.

Do not leave dead feature flags or commented code.

## 6.2 Zero tracked references target

After refactor:

```bash
git grep -in retired adapter
```

must return no tracked current-branch matches.

Do not rewrite `.git` history or old tags.

## 6.3 Legacy compatibility without a retired adapter type

Do not retain `UnsupportedLegacy`, `UnsupportedLegacy`, or equivalent named variants.

Use generic tolerant parsing:

```text
known supported adapter string -> Codex / Claude
other historical adapter string -> UnsupportedLegacy(String)
```

New runtime code may never activate an `UnsupportedLegacy` value.

This preserves old project/history readability while satisfying total product removal.

## 6.4 Existing legacy managed assets

During migration:

- any legacy managed record targeting `.baron/core/**` transfers to Core ownership when its baseline is valid;
- supported Codex/Claude records migrate normally;
- unsupported-adapter records outside canonical Core are retired only when Baron can prove the file is Baron-owned and unchanged;
- modified or ambiguous files are preserved/quarantined;
- never recursively delete an unknown adapter directory without per-file ownership proof.

The cleanup logic must be generic and not contain a retired-adapter-specific path literal.

---

# 7. Remove Generic Agent as an active adapter safely

Final supported adapters are exactly Codex + Claude.

Remove active Generic surfaces:

- `--agent` init/context/update adapter option;
- Generic adapter enum values;
- `ContextTarget::Generic`;
- Generic adapter switching;
- Generic-specific adapter docs/tests/blueprints;
- generated `AGENT.md` and portable adapter context as an active supported integration.

However Generic currently materializes `.baron/core/**`.

Safe order:

1. introduce Core ownership;
2. transfer existing `.baron/core/**` records to Core;
3. install Core independently of adapter;
4. make Codex/Claude consume Core;
5. only then delete Generic active adapter implementation.

No Core skill/agent/workflow may disappear because Generic was removed.

Old Generic project state is handled by the same generic `UnsupportedLegacy(String)` parser.

---

# 8. Codex adapter

## 8.1 Final layout

```text
AGENTS.md
.agents/
  skills/
    baron-engine/
      SKILL.md
      agents/
        openai.yaml
.codex/
  INDEX.md
  agents/
    code-reviewer.toml
    security-auditor.toml
    test-engineer.toml
    [optional native Baron agent wrappers]
  hooks.json
```

New installs must not create `.codex/skills/**`.

## 8.2 Thin bridge

`.agents/skills/baron-engine/SKILL.md` is only a bridge to Baron Core.

It must not copy real Baron skill bodies.

It should:

- identify the project as Baron-managed;
- route through the high-level Baron control-plane protocol;
- read only selected `.baron/core/skills/<skill>/SKILL.md` entrypoints;
- resolve resources relative to each canonical skill root;
- never recursively preload all skills;
- fail clearly if Core/runtime is unavailable.

## 8.3 Implicit routing policy

Because AGENTS.md is the automatic Baron contract, avoid duplicate lifecycle creation from implicit native bridge activation.

Ship:

```yaml
policy:
  allow_implicit_invocation: false
  products: [codex]
```

in the supported Codex metadata location.

Explicit `$baron-engine` may remain a diagnostic/escape hatch, but normal users should never need it.

## 8.4 Native agents

Keep project custom agents under `.codex/agents/*.toml` using current Codex format.

They are host-native wrappers around Core quality-agent roles.

They must not own the global workflow.

---

# 9. Claude adapter

## 9.1 Final layout

```text
CLAUDE.md
.claude/
  skills/
    baron-engine/
      SKILL.md
  agents/
    code-reviewer.md
    security-auditor.md
    test-engineer.md
    [optional native Baron wrappers]
  settings.json
  commands/
    [optional diagnostics/convenience]
```

Do not materialize the entire Baron skill tree into `.claude/skills/**`.

## 9.2 Thin bridge

`.claude/skills/baron-engine/SKILL.md` routes Claude to the same `.baron/core/**` skills as Codex.

If `CLAUDE.md` owns automatic routing, configure the bridge so it cannot independently auto-start a duplicate lifecycle. Keep explicit invocation only as a fallback/diagnostic surface.

## 9.3 Native agents and hooks

Keep current Claude-native project locations:

- `.claude/agents/*.md`;
- `.claude/settings.json` hooks.

Hooks may fire inside subagents. Adapter lifecycle processing must distinguish root session vs quality subagent and must never let a subagent replace the parent task/plan or claim parent completion.

---

# 10. Adapter-independent Core installation

Create an explicit Core install/materialization operation used by every adapter init/update.

Logical init:

```text
resolve project + Vault
    ↓
install/reconcile canonical Core once
    ↓
initialize selected adapter bridge/wrappers
    ↓
verify Core + adapter compatibility
```

Both of these must work from a clean project:

```bash
baron init --codex --fullstack
baron init --claude --fullstack
```

Neither may depend on the other adapter ever having been initialized.

Installing the second adapter must reuse the existing Core rather than duplicate it.

---

# 11. No project-global adapter authority

`active_adapter` must not decide runtime correctness.

Codex and Claude can coexist without switching.

Required rules:

- Codex hooks/runtime calls pass `codex` explicitly;
- Claude hooks/runtime calls pass `claude` explicitly;
- session/task/proof/trace/journal provenance uses explicit/session-scoped identity;
- an explicit adapter always wins over legacy project-global state;
- if adapter identity is genuinely unavailable and required, fail closed instead of selecting the first adapter;
- root adapter-switch shortcuts are not part of normal UX;
- deprecate/remove switching if it no longer has a meaningful diagnostic purpose.

Legacy `active_adapter` may be tolerated during config migration but is not authoritative.

Add concurrent/interleaved Codex+Claude tests.

---

# 12. High-level automatic agent protocol

## 12.1 Goal

Reduce the chance that Codex/Claude must remember a long sequence of hidden Baron calls.

Normal adapter instructions should invoke one high-level Core operation per meaningful task.

Recommended conceptual API:

```text
baron control-plane prepare --adapter codex --json
baron control-plane prepare --adapter claude --json
```

Task input should come from native hook structured input or stdin, not unsafe shell interpolation.

Exact naming may differ if an existing command can be extended cleanly.

## 12.2 Prepare packet

Return a versioned machine-readable packet containing at least:

```json
{
  "schema_version": 1,
  "project_id": "...",
  "adapter": "codex",
  "session_id": "...",
  "task": {
    "id": "...",
    "intent": "...",
    "work_shape": "focused|durable|read_only|requires_confirmation",
    "risk": "low|medium|high",
    "resumed": true
  },
  "profile": {
    "primary": "fullstack",
    "extensions": []
  },
  "selected_skills": [],
  "selected_agents": [],
  "required_verification": [],
  "context": { "...": "bounded task context" },
  "blockers": [],
  "warnings": [],
  "next_action": "..."
}
```

Do not force this exact JSON shape if the existing architecture has a better representation, but keep it versioned, bounded and testable.

## 12.3 Internal composition

The high-level operation composes existing Baron modules rather than replacing them:

```text
authority
work shape
project/Vault identity
runtime/capability check
continuity/recovery
intent/plan state
trusted memory recall
session replay
platform profile
code/source evidence
skill/agent routing
verification gates
```

Keep low-level commands available for diagnostics and tests.

---

# 13. Non-technical user contract

After setup/init, the user should work naturally.

Expected UX:

```text
User: "Build the booking backend."

Adapter automatically:
- prepares Baron task packet;
- resumes compatible work if present;
- reads relevant trusted memory;
- selects profile-appropriate skills;
- creates/updates lifecycle state when required;
- performs work;
- runs proportional verification;
- checkpoints evidence;
- remembers durable facts/decisions;
- continues until complete or genuinely blocked.
```

The adapter must not tell the normal user:

```text
run baron context ...
run baron plan ...
run baron proof ...
run baron continuity ...
```

If explicit human judgment is required, ask the decision in normal language and record it automatically after the user answers.

Never fabricate confirmation.

---

# 14. Automatic per-prompt routing

## 14.1 Native hook path

When supported/trusted, `UserPromptSubmit` should feed the prompt/session context into the high-level prepare operation and return bounded additional context to the host.

SessionStart provides baseline resume/runtime context.

Where supported, use lifecycle hooks such as PreCompact/Stop/Subagent events to checkpoint durable work at safe boundaries.

## 14.2 Fallback path

Hooks may be disabled, untrusted or blocked by policy.

Therefore:

```text
hooks = accelerator
AGENTS.md / CLAUDE.md = correctness fallback contract
Core state = source of truth
```

Generated instructions must run the same high-level prepare/checkpoint protocol when native hooks did not.

## 14.3 Idempotency

Hook and instruction fallback may both fire.

All lifecycle mutations must use an idempotency key based on project/session/task/event so the same logical prompt cannot create duplicate task/plan/checkpoint entries.

---

# 15. Safe task-input transport

Do not embed arbitrary user prompts directly into shell strings.

Support structured input:

- native hook JSON/stdin;
- explicit stdin mode for fallback agent calls;
- safe task file/IPC only if necessary.

The engine must handle quotes, Unicode, newlines and shell metacharacters without command injection or corrupted task text.

Add adversarial prompt-input tests.

---

# 16. Long-horizon task continuity

Use existing Baron durable subsystems as the source rather than building another competing state database.

Create/compile a stable Task State view linking:

- project ID;
- current user intent;
- constraints/non-goals;
- confirmed decisions;
- current plan and phase;
- affected files;
- completed work;
- pending work;
- blockers;
- latest proof;
- latest trace;
- recovery state;
- next safe action;
- adapter/session provenance.

For durable/full work, maintain a stable task identity across prompts and sessions.

Resume rules:

1. Reconcile current source/repo state before continuing.
2. Resume an active task only when the new request is compatible with that task.
3. If request materially changes scope, create a new task/intent relationship instead of mutating history silently.
4. Never infer completion from silence, shutdown, quota exhaustion or network failure.
5. Before compaction/stop/interruption, checkpoint meaningful state.
6. On restart, task continuity must not depend on the model remembering the old conversation.

---

# 17. Memory optimization

## 17.1 Preserve current trust firewall

Do not weaken:

- project identity filter;
- GlobalCandidate exclusion;
- cross-project firewall;
- Candidate/Contested/Superseded/Expired filtering in trusted semantic recall;
- temporal/currentness checks;
- source redaction;
- unknown/abstention behavior.

## 17.2 Unify retrieval authority

Context-critical memory must use the same current trusted retrieval generation as direct modern recall.

Specifically, remove the mismatch where task context's compact memory brief uses older `recall()` eligibility while `recall_v5` has stronger trust filtering.

Expose one authoritative internal retrieval API, e.g.:

```text
TrustedRecallPolicy::Current
```

and make context, resume, routing and direct recall call it.

Legacy retrieval may remain only for frozen regression benchmarking.

## 17.3 Long memory means better retrieval, not larger prompts

Optimize for:

```text
large durable corpus
+ project isolation
+ trust/authority
+ temporal currentness
+ supersession
+ task relevance
+ diversity
+ evidence citations
= useful long memory
```

Do not load the full Vault.

## 17.4 Authority ordering

When facts conflict, favor evidence approximately in this order where applicable:

```text
current source/test/proof
confirmed current user intent/decision
verified project invariant
current plan/harness evidence
verified memory
older likely/stale memory
candidate/autopilot suggestion
```

Do not globally hard-code this simplistic list without considering memory kind, but ensure old polished summaries cannot outrank current source/proof.

## 17.5 Supersession and consolidation

Keep candidate learning conservative, but make maintenance more automatic:

- detect duplicate records;
- detect stale/superseded records;
- stage deterministic consolidation automatically;
- do not auto-promote ambiguous policy;
- surface only material conflicts to the user/agent.

---

# 18. Priority-aware context compiler

Replace flat final-string truncation with a section-aware budget allocator.

## Tier 0 — protected

Never silently truncate away:

- project identity;
- current task identity/intent;
- material constraints/non-goals;
- current plan/continuity/recovery;
- selected route;
- material unknowns/blockers;
- required proof/safety gates.

## Tier 1 — high priority

- relevant verified decisions/memory;
- affected source areas;
- current proof/trace;
- immediate next action.

## Tier 2

- profile lens;
- relevant CodeGraph/Wiki evidence;
- bounded session replay;
- capability details required by the task.

## Tier 3

- general diagnostics;
- autopilot candidate summary;
- low-priority metadata.

Rules:

- budget each section;
- summarize lower tiers before removing higher tiers;
- preserve explicit diagnostics telling the agent what was omitted;
- include source pointers so the agent can pull more context on demand;
- do not simply raise `MAX_CONTEXT_CHARS` and call it solved.

Add tests with intentionally oversized contexts proving Tier 0 survives.

---

# 19. Smarter control-plane routing

Current keyword routing becomes one feature, not the whole decision.

Routing inputs should include:

```text
normalized task intent
+ project platform profile
+ work shape/risk
+ repository survey/changed areas
+ current task phase
+ relevant verified memory/decisions
+ previous failed proof/recovery
+ available capabilities
```

Output remains bounded:

- one workflow owner (`superpowers`);
- minimum sufficient optional skills;
- minimum sufficient quality agents;
- proportional verification.

Do not load all skills because the project is fullstack.

Explicit task evidence outranks broad project profile.

Add explainable route reasons and exclusion reasons.

---

# 20. Skill sensitivity and false-positive control

For every skill define structured routing metadata or equivalent:

- trigger concepts;
- exclusion concepts;
- applicable profiles;
- required evidence/signals;
- dependencies;
- conflicts/overlap;
- verification contribution;
- workflow ownership = false for domain skills.

Examples:

- API work does not automatically require a full security gate solely because the token `api` appears;
- security skill becomes mandatory for auth/permission/secret/payment/tenant/trust-boundary/high-risk cases;
- frontend skill is not loaded for a backend-only task in a fullstack project;
- database skill is not loaded merely because a project has a database dependency if the task is a README edit.

Build false-positive and false-negative route tests.

---

# 21. Project profiles must affect real behavior

A project profile is a first-class routing/verification prior, not decorative Markdown.

It must influence:

- skill scoring;
- agent scoring;
- context lens;
- architecture guidance;
- default verification layers;
- task decomposition;
- relevant memory query expansion/tags;
- completion proof expectations.

The profile must never override clear task evidence.

---

# 22. Fullstack profile

`--fullstack` should optimize for cross-layer product work.

Concerns include:

- frontend state/UI;
- backend/API contracts;
- service boundaries;
- persistence/data model;
- auth/validation;
- integration boundaries;
- deployment/config contracts;
- end-to-end user flows.

Routing examples:

- UI-only change -> frontend skill, focused UI verification;
- API-only change -> API/backend guidance, no unnecessary UI skill;
- feature crossing UI/API/database -> route all required domains and require contract/integration proof;
- shared schema/type change -> verify both producers and consumers.

---

# 23. Backend profile

`--backend` should prioritize:

- API/interface design;
- service/domain boundaries;
- input validation;
- errors/status semantics;
- authentication/authorization when relevant;
- persistence boundaries;
- transactions/idempotency;
- observability;
- performance where relevant;
- focused tests/integration tests.

Do not force frontend guidance unless the task crosses into client contracts.

---

# 24. Database profile — new

Add `--database` as a distinct first-class platform profile.

Do not alias it semantically to `--data`; Data and Database serve different work.

Database profile concerns:

- relational/document schema modeling as appropriate;
- primary/foreign keys;
- nullability;
- constraints;
- uniqueness;
- referential integrity;
- normalization and deliberate denormalization;
- index design;
- query plans;
- transactions;
- isolation/locking;
- concurrency anomalies;
- migration/backfill safety;
- rollback strategy;
- ORM/repository boundaries;
- N+1/query-shape issues;
- data retention/destructive operations;
- backup/recovery implications where relevant.

Add a canonical optional skill:

```text
.baron/core/skills/database-engineering/SKILL.md
```

The skill must be domain guidance only and must not become another workflow owner.

If adding a new persisted platform value requires a ProjectConfig schema bump, bump it explicitly and add migration/downgrade tests. Do not bump schemas for cosmetic reasons.

---

# 25. Data profile

Keep `--data` for data engineering/analytics/pipeline work.

It should focus on:

- data contracts;
- lineage;
- ingestion/transformation;
- batch/stream boundaries;
- schema evolution;
- reproducibility;
- data quality;
- backfills;
- analytical correctness.

Database and Data can be extension profiles in mixed projects.

---

# 26. Mobile profile

`--mobile` should influence real app-development routing:

- app lifecycle;
- navigation/state;
- offline/cache behavior;
- local storage;
- permissions;
- secure storage;
- API/network failures;
- device/platform differences;
- background/foreground transitions;
- performance/battery;
- adaptive UI;
- platform-specific testing/release concerns.

The existing `apk-mobile-analysis` skill serves analysis/reverse use cases and must not be treated as the general mobile application development skill.

Add a separate optional:

```text
mobile-application-engineering
```

when profile/task evidence requires it.

---

# 27. Automatic verification by profile + task

Verification must be selected from actual changed behavior, not a single fixed pipeline.

Examples:

Rust backend:

```text
fmt
clippy relevant scope
focused tests
integration tests when boundary changed
```

Frontend:

```text
typecheck/lint
focused component/unit tests
browser/E2E only when user-visible flow requires it
```

Database:

```text
schema/migration validation
forward migration
rollback/recovery where supported
constraint/index/query verification
integration tests for affected persistence behavior
```

Mobile:

```text
static/type checks
unit/integration tests
platform/device lifecycle checks when relevant
build/package verification when appropriate
```

Do not run expensive unrelated gates automatically.

Record executed evidence; configured tool presence is not proof.

---

# 28. Autopilot and self-learning

Preserve the rule that learning candidates are not automatically trusted policy.

Improve UX:

- agent automatically runs post-task learning review;
- safe deterministic maintenance can be staged without user commands;
- runtime skill/policy/router mutation remains approval-gated;
- approval can occur conversationally and be recorded by the adapter;
- do not require a non-technical user to run `baron autopilot approve` manually.

No unapproved candidate may silently change active skill or routing policy.

---

# 29. AGENTS.md rewrite

There are two scopes.

## 29.1 Baron Engine repository development AGENTS.md

Rewrite the repo's own `AGENTS.md` to describe current architecture and current invariants.

Remove stale phase/version narrative from the instruction contract.

Keep:

- purpose;
- current architecture;
- read order;
- non-negotiable safety rules;
- development workflow;
- required verification;
- source-of-truth docs.

Move historical phase detail to history/status docs.

## 29.2 Generated Codex AGENTS.md Baron block

Make it compact and operational.

It should define:

1. Baron Core is state/routing authority.
2. Normal user prompts are automatically prepared through Baron.
3. Resume compatible active task before creating a new one.
4. Load only routed canonical skills.
5. Use proportional work shape and verification.
6. Preserve user instructions and files.
7. Never fabricate unknowns or completion.
8. Checkpoint meaningful state automatically.
9. Use hooks when available; fall back to the same idempotent protocol when not.
10. Do not ask user to run hidden Baron commands.

Keep the Baron managed block well below Codex's project-instruction budget and preserve user text outside Baron markers.

Do not embed all Baron skills/workflows in AGENTS.md.

---

# 30. CLAUDE.md rewrite

Apply the same architectural contract with Claude-native terminology.

CLAUDE.md must not duplicate the full Baron skill/workflow library.

It should tell Claude how to:

- auto-prepare each meaningful task;
- resume durable state;
- use canonical `.baron/core` skills;
- use native Claude quality agents only when routed;
- avoid subagent lifecycle corruption;
- checkpoint/recover;
- verify before completion;
- avoid asking the user for Baron CLI operations.

Preserve user text outside Baron-managed markers.

---

# 31. Core skill resource semantics

A routed canonical skill root is:

```text
.baron/core/skills/<name>/
```

Its `SKILL.md`, scripts, references and assets resolve relative to that directory.

Adapter bridges must make this explicit.

Audit all current skills for assumptions that their resources live under `.codex/skills` or `.claude/skills`.

Add integration fixtures containing:

- SKILL.md;
- script;
- reference;
- asset.

Prove both Codex and Claude can consume the same canonical skill package without copies.

---

# 32. Canonical skill integrity

A file existing under `.baron/core/skills/**` is not enough to trust it as Baron-managed.

Before routing a built-in skill as trusted Core:

- verify managed owner;
- verify baseline/hash/integrity according to Baron policy;
- detect local modification;
- preserve modified files;
- report conflict/tamper state;
- do not silently execute modified Core instructions as trusted product policy.

Custom skills need explicit provenance distinct from Core.

Prefer a separate root such as:

```text
.baron/custom/skills/**
```

if migration can support it cleanly.

---

# 33. Managed read hardening

Audit all durable/managed read paths.

Replace patterns equivalent to:

```rust
fs::read_to_string(path).unwrap_or_default()
fs::read_to_string(path).unwrap_or_else(|_| missing)
```

when they can hide meaningful errors.

Use a shared helper with semantics:

```text
Ok(content) -> content
NotFound -> explicit None/missing when absence is allowed
PermissionDenied -> error
InvalidData/UTF-8 -> error
IsDirectory -> error
other I/O -> error
```

Status-only UI may render a diagnostic instead of failing the entire program, but it must distinguish "missing" from "unreadable/corrupt".

A failing read must never cause a later writer to treat existing content as empty.

---

# 34. Safe file replacement

Replace all pseudo-atomic managed writers with one tested primitive.

Required properties:

1. unique temp file;
2. same parent directory / same filesystem as target;
3. complete write before commit;
4. flush temp contents;
5. safe platform replacement preserving old target until commit;
6. appropriate metadata/permissions handling;
7. identical-content no-op;
8. target-path diagnostics;
9. temp cleanup;
10. Windows/Linux/macOS correctness;
11. directory sync on Unix where practical;
12. never claim cross-filesystem atomicity.

If platform-specific APIs or a small proven crate are needed, use them rather than inventing a fragile replacement.

---

# 35. Durable state and journal safety

Automation journal must not use read-whole-file + append-in-memory + rewrite under concurrent sessions.

Use locked append or another durable append protocol.

For mirrored repo/Vault state:

- do not pretend two different roots can be atomically committed in one filesystem operation;
- use recoverable transaction/receipt semantics;
- Vault remains durable truth where the memory model defines it as such;
- startup reconciliation repairs an incomplete mirror safely.

Critical paths include:

- continuity;
- intent;
- plan/harness state;
- proof/trace indexes;
- automation journal;
- autopilot candidates;
- managed baseline/migration state.

---

# 36. Concurrency locking

Codex and Claude may mutate the same project/Vault concurrently.

Introduce a consistent mutation-lock strategy.

At minimum protect:

- managed repo files and baseline;
- shared Vault project records;
- automation journal;
- plan/intent state transitions;
- migration/update transaction publication.

Requirements:

- bounded wait or clear busy state;
- stale-lock recovery with owner/process metadata where safe;
- no permanent deadlock after crash;
- stable lock acquisition order if more than one lock is required;
- read-only commands avoid exclusive lock where possible;
- baseline/state revalidation after lock acquisition.

Add two-process tests where practical.

---

# 37. Managed-state migration

Managed-state ownership changes likely require a schema bump.

Do not bump merely for naming changes; bump when persisted representation changes.

Migration from current state:

```text
legacy record path under .baron/core/**
    -> Core owner

legacy supported Codex path
    -> Adapter(Codex)

legacy supported Claude path
    -> Adapter(Claude)

legacy unsupported adapter path
    -> preserved/retired according to baseline ownership
```

Do not publish the new manifest until required file operations succeed.

Keep a recovery copy/transaction receipt for the previous valid baseline.

Migration is idempotent.

---

# 38. ProjectConfig migration

Final active adapter set is Codex + Claude.

Parse old adapter identifiers tolerantly without named retired variants.

When rewriting mutable current config:

- retain registered Codex/Claude adapters;
- unsupported old adapters are not re-persisted as active;
- preserve project ID;
- preserve Vault routing;
- preserve platform/profile settings;
- preserve automation settings.

If an old project has no supported adapter after migration, keep project state intact and require explicit one-time `baron init --codex` or `--claude`; do not silently guess.

If the new `database` platform or representation requires a config schema bump, document the exact reason.

---

# 39. Historical provenance compatibility

Old journals/history may contain arbitrary adapter strings.

Do not deserialize historical provenance through `SupportedAdapter`.

Use a tolerant historical representation such as a validated/escaped string wrapper.

New writes accept only supported runtime adapters.

Historical unsupported values remain displayable/searchable as history but cannot activate runtime behavior.

This removes the need for retired adapter enum variants.

---

# 40. Downgrade protection

A Baron binary older than the new managed/project schema must not silently reconcile/write newer state.

Persist explicit compatibility metadata sufficient for:

- current schema version;
- minimum writer/reader version where needed;
- installed Baron version;
- managed baseline generation.

New Baron must refuse unsafe downgrade writes and provide a clear diagnostic/recovery path.

Old binary behavior should be tested where feasible using a fixture or pinned prior release binary.

---

# 41. Adapter hooks

## Codex

Keep `.codex/hooks.json` native and merge only Baron-owned entries.

Preserve all third-party hooks and unrelated keys.

Project hooks may be untrusted/disabled; correctness does not depend solely on them.

## Claude

Keep `.claude/settings.json` native.

Preserve user settings and hooks outside Baron-owned entries.

Hooks also run in subagents; inspect root/subagent identity before mutating parent lifecycle state.

## Both

Use supported current host event schemas.

Prefer:

- SessionStart baseline resume;
- UserPromptSubmit task prepare;
- edit/post-tool checkpoints only when due;
- PreCompact checkpoint where supported;
- Stop completion/recovery check;
- subagent lifecycle awareness where supported.

Do not create high-frequency disk churn for every trivial tool event.

---

# 42. Quality agents

Keep exactly three mandatory Core quality-agent roles:

- code-reviewer;
- security-auditor;
- test-engineer.

They are gates, not workflow owners.

Codex/Claude native files are wrappers generated from canonical Core contracts.

Rules:

- route only proportional gates;
- no recursive agent fan-out by default;
- parent task owns intent/plan/completion;
- subagents return evidence;
- a subagent cannot complete or replace the parent task;
- actual gate execution must be recorded before it counts as proof.

---

# 43. Profile-aware memory and routing

Project profile may enrich retrieval queries and route priors, but cannot change memory truth.

Examples:

- database task may expand relevant concepts to migration/index/transaction/constraint terminology;
- mobile task may expand lifecycle/offline/permission/storage concepts;
- fullstack cross-layer task may retrieve prior API contract and schema decisions.

Profile expansion happens after project identity/trust gates and must never allow semantic similarity to bypass the Memory Firewall.

---

# 44. Failure and recovery behavior

For any meaningful failure/interruption:

record:

- cause;
- last successful step;
- affected files;
- evidence;
- current task/plan;
- safe next action;
- retry conditions.

If a state write partially succeeds, startup reconciliation must identify the incomplete operation rather than treating it as complete.

If hooks fail, adapter instructions keep state coherent.

If Baron runtime is missing, adapter must fail clearly and avoid repeated blind calls or pretending memory/routing succeeded.

---

# 45. Self-hosting Baron development

The Baron Engine repository itself must remain developable while Baron is being rebuilt.

Avoid bootstrap loops where the project instructions require a newly built binary before that binary can be compiled/tested.

Document fallback order:

```text
installed compatible Baron
-> workspace-built Baron/cargo invocation for development
-> explicit degraded development mode when necessary
```

Do not weaken final user-project contracts because the engine repo is special; isolate self-hosting behavior to development guidance.

---

# 46. Documentation cleanup

Update active docs to the exact final architecture:

- README.md;
- AGENTS.md;
- docs/architecture/ARCHITECTURE.md;
- docs/architecture/ADAPTERS.md;
- docs/architecture/COMMAND_SURFACE.md;
- docs/architecture/MEMORY_MODEL.md;
- docs/architecture/CONTEXT_COMPILER.md;
- docs/BARON_STATUS.md/json;
- relevant blueprints;
- setup/install docs;
- compatibility docs;
- CHANGELOG/current release notes.

Remove active Generic/retired-adapter claims.

Historical development documents dedicated to retired adapter support should be removed from tracked current source if required to satisfy the complete purge contract.

Do not let current docs claim copied adapter skill trees after canonical Core consolidation.

---

# 47. Compatibility documents

Maintain explicit host compatibility notes:

```text
docs/compatibility/CODEX.md
docs/compatibility/CLAUDE.md
```

Record:

- verification date;
- tested product/version where available;
- instruction file;
- skill path;
- agent/subagent path;
- hook/settings path;
- bridge metadata behavior;
- known trust/policy caveats.

Do not make CI depend nondeterministically on "latest" host behavior. Prefer pinned/scheduled compatibility jobs plus release-time manual verification.

---

# 48. Required regression fixtures before implementation

Before production behavior changes, create frozen fixtures for:

1. current Codex-only project;
2. current Claude-only project;
3. current Generic-only project with `.baron/core/**`;
4. current multi-adapter project;
5. current project containing unsupported legacy adapter strings;
6. legacy `.codex/skills/**` managed tree;
7. legacy `.claude/skills/**` copied tree;
8. modified legacy managed skill;
9. third-party Codex hooks;
10. third-party Claude settings/hooks;
11. large context >20k;
12. superseded/contested/candidate memory;
13. interrupted task/continuity state;
14. concurrent automation journal writers;
15. malformed managed manifest;
16. invalid UTF-8/unreadable managed target;
17. symlink/junction path escape.

Use representative old-state files rather than regenerating fixtures with new code.

---

# 49. Required tests — adapter/core

Test:

- clean Codex init installs Core exactly once;
- clean Claude init installs Core exactly once;
- Codex -> Claude reuses same Core;
- Claude -> Codex reuses same Core;
- repeated init is idempotent;
- Core ownership has no duplicate live target;
- new Codex install has no `.codex/skills/**`;
- new Claude install has no full copied Baron skill tree;
- both bridges read same canonical skill;
- native agent wrappers remain valid;
- adapter explicit identity overrides stale legacy active state.

---

# 50. Required tests — retired adapter/Generic purge

- active supported adapter enum has exactly Codex + Claude;
- no retired/Generic active CLI option;
- no tracked current-product retired-adapter references;
- unsupported historical adapter string parses generically;
- unsupported history remains readable;
- unsupported legacy managed file unchanged -> safe retirement/preservation policy works;
- unsupported modified file survives;
- old `.baron/core/**` records transfer to Core before Generic removal;
- project/Vault identity survives.

---

# 51. Required tests — memory

- current project isolation;
- cross-project weak match blocked;
- GlobalCandidate blocked;
- Candidate/Contested/Superseded/Expired absent from trusted task context;
- context memory brief and direct current recall use equivalent trust eligibility;
- verified current decision outranks stale historical summary;
- unknown evidence produces abstention;
- large Vault does not imply large prompt;
- session replay stays bounded and exact-project matched;
- temporal currentness enforced when ledger exists.

---

# 52. Required tests — context priority

Construct oversized contexts where low-priority sections would exceed budget.

Assert that the final context retains:

- project ID;
- task/intent;
- continuity/current plan;
- blockers/unknowns;
- route decision;
- proof requirements.

Assert lower-priority sections are summarized/omitted first with diagnostics.

Test both Codex and Claude targets.

---

# 53. Required tests — profiles

For each public profile prove behavior changes, not just generated text.

Examples:

### Fullstack

Cross-layer feature selects front/API/data-related guidance and integration proof; UI-only task stays focused.

### Backend

Backend task prioritizes API/service/persistence guidance without frontend noise.

### Database

Schema/migration task selects database-engineering and database-specific proof.

### Mobile

Offline/permission/lifecycle task selects mobile-application-engineering and appropriate validation.

### Data

Pipeline/backfill task uses data-specific concerns distinct from database profile.

Add negative tests proving profile does not over-trigger unrelated skills.

---

# 54. Required tests — automation/non-tech UX

Simulate normal user prompts without manual Baron commands.

Assert adapter flow automatically:

- prepares task;
- routes skills;
- loads trusted context;
- resumes compatible task;
- creates appropriate lifecycle depth;
- checkpoints;
- invokes/records required verification;
- produces completion/recovery state.

Test hooks enabled and hooks unavailable/disabled fallback.

The user must not be instructed to operate hidden CLI commands.

---

# 55. Required tests — I/O and concurrency

- NotFound handled as absent only where allowed;
- PermissionDenied is not empty;
- invalid UTF-8 is not empty;
- directory-at-file-path fails clearly;
- malformed JSON/TOML fails closed;
- malformed markers fail closed;
- safe replacement preserves prior target on pre-commit failure;
- unique temps;
- identical no-op;
- concurrent managed update serialized/rejected safely;
- concurrent journal writes lose no events;
- interrupted mirrored repo/Vault state is recoverable;
- stale lock recovery;
- symlink/junction escape blocked;
- Windows ACL/read-only behavior where practical.

---

# 56. Required tests — hooks/subagents

## Codex

- third-party same-event hooks survive;
- Baron hooks deduplicate;
- untrusted hooks do not break fallback correctness;
- changed hook trust does not silently imply success;
- subagent start/stop does not replace parent lifecycle.

## Claude

- existing user settings survive;
- third-party hooks survive;
- disableAllHooks/policy-degraded flow still works through CLAUDE.md;
- quality subagent cannot complete parent task;
- root/subagent provenance remains distinct.

---

# 57. Cross-platform CI

Preserve existing matrix:

```text
Windows x86_64 MSVC
Linux x86_64 GNU
macOS Intel
macOS Apple Silicon
```

Required commands include at least:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --no-fail-fast
cargo build --release --locked -p baron-cli
```

Add adapter smoke tests for Codex and Claude on supported runners where practical.

---

# 58. Current-host integration verification

At release time verify against current official host behavior.

## Codex

Confirm:

- root AGENTS.md loaded;
- `.agents/skills/baron-engine` discovered;
- bridge implicit invocation disabled;
- `.codex/agents/*.toml` discovered;
- `.codex/hooks.json` parsed/trust behavior understood;
- automatic task flow works without `.codex/skills` copies.

## Claude

Confirm:

- CLAUDE.md loaded;
- `.claude/skills/baron-engine` valid;
- `.claude/agents/*.md` discovered;
- `.claude/settings.json` hooks valid;
- automatic task flow works without copied real Baron skills.

Record verification evidence in compatibility docs.

---

# 59. Hidden-risk register

The implementation agent must explicitly resolve or test every item.

1. Duplicate Core ownership between adapters.
2. Removing Generic before transferring `.baron/core` ownership.
3. Legacy unsupported adapter strings making old ProjectConfig unparsable.
4. Historical journal adapter enums becoming unreadable.
5. Legacy modified adapter skill preserved but silently no longer effective.
6. Codex bridge collision with user `.agents/skills/baron-engine`.
7. Codex global/user skill with same name.
8. Claude bridge collision with user project skill.
9. Claude personal skill precedence shadowing project bridge.
10. Double routing from root instructions + implicit bridge activation.
11. Hooks disabled/untrusted and automatic lifecycle silently disappearing.
12. Hook + instruction fallback both mutating state twice.
13. Subagent lifecycle corrupting parent task.
14. Project-global `active_adapter` mislabeling Codex/Claude sessions.
15. Raw prompt interpolated into shell command.
16. Platform profile over-routing unrelated skills.
17. Keyword router over-triggering security/API gates.
18. Database profile accidentally replacing Data semantics.
19. APK analysis skill incorrectly used for normal mobile app development.
20. Context flat truncation dropping critical continuity/memory/router state.
21. Context memory brief using older trust policy than modern recall.
22. Long-memory optimization increasing prompt size instead of retrieval quality.
23. Superseded/contested memory leaking through an alternate retrieval path.
24. Self-learning candidate becoming runtime policy without approval.
25. Read error treated as empty and overwriting content.
26. Delete-before-rename data-loss window.
27. Deterministic temp collision.
28. Replacement metadata/ACL regression on Windows.
29. Symlink/junction write escape.
30. Two adapter processes publishing different managed baselines.
31. Concurrent automation journal lost update.
32. Repo/Vault mirror partial write divergence.
33. Crash after file mutation but before baseline publication.
34. Crash after baseline publication but before final receipt.
35. Older Baron binary rewriting newer state.
36. New database platform unreadable by old binary without downgrade guard.
37. Core skill local tamper treated as trusted product instruction.
38. Adapter bridge recursively loading every Core skill.
39. Canonical skill relative scripts/assets resolving from wrong directory.
40. Root Baron repository self-hosting bootstrap loop.
41. Host upstream path/schema change breaking discovery.
42. Large AGENTS.md crowding out user/project instructions.
43. Generated Baron marker update deleting user text.
44. Claude settings merge overwriting unrelated user keys.
45. Codex hooks merge deleting unrelated hook groups.
46. Expensive automatic verification running on trivial changes.
47. Insufficient verification because profile replaces actual task evidence.
48. Multi-session task resume attaching a new unrelated request to old task.
49. Task scope change overwriting old intent history.
50. Completion inferred from model response rather than proof/trace state.
51. User forced to use Baron CLI for ordinary approval/continuation.
52. Missing Baron runtime causing repeated blind agent calls.
53. Core install performed once but adapter update assumes old copied skill layout.
54. Existing `.codex/skills` custom user content accidentally retired.
55. Existing `.claude/skills` custom user content accidentally retired.
56. Current status/docs tests still encode removed adapter counts/claims.
57. Release installer/update candidate reintroducing stale adapter assets from old package manifests.
58. Migration logic containing retired-adapter-specific strings, violating complete purge.
59. retired adapter-specific historical documents remain discoverable as active current product.
60. Profile-specific routing changes without explainability/evidence tests.

---

# 60. Acceptance criteria

The refactor is complete only when all are true:

1. Supported runtime adapters are exactly Codex + Claude.
2. `git grep -in retired adapter` returns no tracked current-branch product matches.
3. Generic is not an active adapter.
4. Removing Generic did not remove any Core intelligence.
5. Every initialized project has one canonical `.baron/core/**` runtime.
6. Core owns canonical skills/agent contracts independently of adapter.
7. Codex does not create `.codex/skills/**` on new install.
8. Codex has one valid thin native Baron bridge.
9. Claude does not receive copied real Baron skill trees on new install.
10. Claude has one valid thin native Baron bridge.
11. Both adapters can initialize first independently.
12. Both adapters coexist without global switching.
13. Explicit/session adapter provenance is correct under concurrent sessions.
14. Superpowers remains the only workflow owner.
15. Exactly three mandatory Core quality-agent roles remain.
16. Canonical skill resources resolve correctly.
17. Core skill integrity is verified before trusted routing.
18. User custom adapter files/skills/hooks/settings are preserved.
19. Legacy unsupported adapter strings remain generically readable but not activatable.
20. Old `.baron/core` records migrate to Core ownership safely.
21. Modified legacy copied skills cannot be silently sidelined by a successful migration.
22. Modern trusted recall is the authority for task-context memory.
23. Candidate/contested/superseded/expired memory does not leak into trusted task context.
24. Priority-aware context budgeting protects Tier 0 state.
25. Long memory remains retrieval-based and bounded.
26. Task continuity survives session restart without conversation-memory dependence.
27. Compatible active tasks resume automatically.
28. Material scope changes preserve old task/intent history.
29. Completion remains evidence-backed.
30. Normal user prompts do not require hidden Baron CLI commands.
31. Hooks accelerate automation but are not required for correctness.
32. Hook/fallback lifecycle writes are idempotent.
33. Raw user task input is never unsafely shell-interpolated.
34. `--fullstack` changes real routing/verification behavior.
35. `--backend` changes real routing/verification behavior.
36. `--mobile` changes real routing/verification behavior.
37. `--data` retains data-engineering semantics.
38. `--database` exists and changes real database routing/verification behavior.
39. `database-engineering` canonical skill passes Baron skill contracts.
40. General mobile-development guidance is distinct from APK analysis.
41. Profile priors do not override clear task evidence.
42. Unsafe read-as-empty patterns are removed from durability-critical paths.
43. Managed replacement no longer intentionally deletes original before commit.
44. Concurrent journal/state writes are safe/recoverable.
45. Symlink/junction escape is blocked.
46. Managed migration is transactional/idempotent/recoverable.
47. Older binary downgrade writes are blocked where schemas are incompatible.
48. AGENTS.md is rewritten as a current concise development contract.
49. Generated Codex AGENTS managed block is concise, automatic and non-manual.
50. CLAUDE.md is rewritten as a concise automatic adapter contract.
51. Current Codex compatibility is verified/documented.
52. Current Claude compatibility is verified/documented.
53. Cross-platform CI remains green.
54. Format/Clippy/full tests/release build pass.
55. No active docs/blueprints/status claim unsupported adapters.
56. No release/update package can regenerate removed adapter assets.
57. Existing project ID and Vault history survive migration.
58. No user-owned file is silently overwritten/deleted.
59. No evidence-free success claim is introduced to improve UX.
60. The new engine is behaviorally at least as capable as the old engine on frozen Baron intelligence regression cases.

---

# 61. Implementation order

Use small verified phases. Do not implement the entire spec in one uncontrolled patch.

## Phase 1 — Freeze behavioral regression fixtures

- capture current memory/continuity/profile/proof behavior;
- capture old adapter/project/managed-state fixtures;
- add failing target architecture tests;
- no production behavior changes.

## Phase 2 — Unified safe I/O and locking primitives

- safe optional read;
- safe replacement;
- journal append safety;
- mutation locks;
- failure injection tests.

Do this before migration relies on file mutation.

## Phase 3 — Core managed ownership

- introduce Core owner;
- migrate `.baron/core` ownership;
- schema migration/rollback;
- core payload generation independent of adapter.

## Phase 4 — Canonical Core installation

- every adapter init installs/reconciles Core exactly once;
- preserve all existing core skills/agents/workflows.

## Phase 5 — High-level control-plane prepare protocol

- compose current modules;
- structured safe task input;
- explicit adapter/session identity;
- versioned packet;
- no user CLI requirement.

## Phase 6 — Trusted memory/context unification

- context-critical recall uses current trusted retrieval;
- priority-aware context budgeting;
- long-horizon task/resume tests.

## Phase 7 — Profile-aware routing

- route fusion with profile/task/repo/risk/history;
- false-positive controls;
- database/mobile skills;
- profile-specific verification.

## Phase 8 — Codex adapter bridge

- `.agents/skills/baron-engine`;
- native agents;
- hooks;
- rewrite generated AGENTS block;
- migrate old `.codex/skills` safely.

## Phase 9 — Claude adapter bridge

- thin `.claude/skills/baron-engine`;
- native agents/settings/hooks;
- rewrite CLAUDE block;
- migrate copied `.claude/skills` safely.

## Phase 10 — Remove global adapter authority

- explicit/session-scoped provenance;
- concurrent Codex+Claude tests;
- retire switching from correctness paths.

## Phase 11 — Complete unsupported adapter cleanup

- remove Generic active adapter after Core extraction;
- remove retired adapter completely;
- generic legacy parser only;
- purge tracked docs/tests/assets/blueprints;
- verify zero tracked retired adapter references.

retired adapter is the highest product priority, but actual deletion is intentionally after generic migration infrastructure exists so the cleanup does not destroy old state or Core ownership.

## Phase 12 — Hook/subagent/idempotency hardening

- per-prompt prepare automation;
- PreCompact/Stop checkpoints where supported;
- fallback path;
- parent/subagent isolation.

## Phase 13 — Autopilot non-tech UX

- automatic candidate staging;
- conversational approval path where needed;
- no hidden CLI instructions to user.

## Phase 14 — Downgrade/update/release hardening

- schema compatibility guard;
- candidate package does not reintroduce stale assets;
- interrupted update recovery.

## Phase 15 — Rewrite current docs and development AGENTS.md

- current architecture only;
- compatibility docs;
- remove obsolete tracked adapter-specific history required by purge policy.

## Phase 16 — Full verification and adversarial review

- workspace CI;
- four native platforms;
- old-project migration fixtures;
- current Codex integration;
- current Claude integration;
- concurrent sessions;
- process-kill/failure injection;
- hidden-risk register review;
- acceptance criteria evidence.

---

# 62. Implementation-agent rules

Before changing production code:

1. Read the deep audit and this spec.
2. Map every affected module.
3. Write regression/failing tests first where practical.
4. Do not delete an old adapter until Core/migration paths that depend on its current layout have been extracted.
5. Do not weaken assertions just to make CI pass.
6. Do not treat documentation claims as source truth when code differs.
7. Do not invent host compatibility; verify current Codex/Claude docs/source.
8. Do not introduce new persisted schema versions without explicit need and migration tests.
9. Do not claim a file operation is atomic unless its platform semantics justify the claim.
10. Do not report completion from unit tests alone; retain cross-platform/migration/integration evidence.

After each phase report:

- changed files;
- tests added;
- tests run;
- failures/risks;
- spec adjustment needed or none;
- exact next phase.

Do not proceed through multiple destructive migration phases without review checkpoints.

---

# 63. Final architectural definition

The finished Baron should behave as:

```text
                    BARON ENGINE CORE

   Memory / Recall / Session Replay / Temporal Trust
   Intent / Plan / Harness / Continuity / Recovery
   Control Plane / Profiles / Skills / Autopilot
   Quality Agents / Proof / Trace / Capability Evidence
                      │
         canonical .baron/core/**
                      │
           ┌──────────┴──────────┐
           │                     │
        CODEX                  CLAUDE
     thin adapter            thin adapter
           │                     │
    native AGENTS          native CLAUDE
    native agents          native agents
    native hooks           native hooks
           │                     │
           └──── same Baron brain ────┘
```

Normal human experience:

```text
baron setup --vault ...
baron init --codex --fullstack
# or
baron init --claude --fullstack

Then just use Codex/Claude normally.
```

Baron performs its own internal routing, context, continuity, memory, verification and recovery automatically.

The human gives product intent; Baron handles engine mechanics.
