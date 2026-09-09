# Baron Engine Deep Audit — 2026-09-06

Repository: `thienty1207/Baron-Engine`
Audited target: current `main` source around commit `9602e47e14517f87758d5584eb1299c4673fe6b9`

> **Historical evidence.** This audit records the pre-refactor state observed
> on 2026-09-06. It explains why the Core consolidation was needed and remains
> useful for migration and review. It is not the current public product contract;
> use the maintained README and architecture/compatibility pages for
> current behavior. Phase 1 through Phase 15 implementation has since landed;
> adversarial Phase 16 verification remains pending.

## Executive conclusion

Baron already contains a substantial engine: project identity, Vault-backed durable memory, deterministic semantic recall, temporal/trust filtering, session replay, platform intelligence, Product Harness, planning, proof, trace, continuity/recovery, capability checks, CodeGraph/Wiki accelerators, quality agents, skill routing, automation hooks, and a guarded self-learning/autopilot candidate flow.

The correct refactor is therefore **not** to simplify Baron into an adapter installer. It should extract and strengthen the existing intelligence into an adapter-independent Core, then make only Codex and Claude consume that Core.

Target product:

```text
Baron Core
  ├─ memory / recall / session replay
  ├─ intent / plan / harness
  ├─ continuity / recovery
  ├─ proof / trace / quality gates
  ├─ control plane / routing
  ├─ platform profiles
  ├─ autopilot learning candidates
  ├─ canonical skills
  └─ canonical quality-agent contracts
        │
        ├── Codex adapter
        └── Claude adapter
```

retired adapter must be removed completely from the tracked product source. Generic Agent is also removed as an active adapter because the final supported adapter set is exactly Codex + Claude. Core functionality currently materialized through the Generic adapter must be extracted before Generic is removed.

---

## 1. Existing strengths that must not regress

### Durable memory and project isolation

`baron-core/src/memory.rs`, `firewall.rs`, `semantic.rs`, and `intelligence41.rs` already provide:

- stable project-ID isolation;
- Vault Markdown as durable truth;
- SQLite/cache as rebuildable accelerator;
- project/global memory scopes;
- confidence, status, abstraction and trust states;
- supersession/contradiction provenance;
- deterministic multilingual lexical/semantic ranking;
- temporal filtering;
- current-project preference;
- blocking of global candidates and weak cross-project leakage;
- bounded session replay.

Do not replace this with naive embeddings or "more context".

### Task continuity

`continuity.rs`, `intent.rs`, `plan.rs`, `harness.rs`, `proof.rs`, and `trace.rs` already preserve much of the task state needed for long-running work:

- original/current intent;
- constraints/non-goals/decisions;
- plan state and next action;
- proof and trace state;
- changed files;
- recovery packets for interrupted/blocked/failed work;
- resume packets in repo and Vault.

The refactor should unify and automate these paths, not create a second competing task-state system.

### Platform intelligence

`platform.rs` already has real profiles for frontend, backend, fullstack, mobile, desktop, tool, library, data, and cloud. Profiles include concerns, architecture priorities, failure modes, security/performance expectations, skill/agent routing hints, verification layers, and release proof.

This is real functionality and must be retained. The main gap is that these profiles do not yet strongly control the core task router/verification engine.

### Safety/proof architecture

Baron already distinguishes configured capability from execution evidence, has mandatory core quality agents, and blocks completion when proof/trace requirements fail. Preserve this.

---

## 2. Adapter architecture problem

Current adapter install code embeds one `assets/core/**` source but materializes full copies into adapter-specific directories:

```text
Codex  -> .codex/skills/** + .codex/agents/**
Claude -> .claude/skills/** + .claude/agents/**
Generic -> .baron/core/skills/** + .baron/core/agents/**
legacy projection -> legacy/skills/** + legacy/agents/**
```

This means the build-time source is shared, but runtime skill trees are duplicated.

Final architecture should be:

```text
assets/core/**                  packaged source
      ↓
.baron/core/**                  one canonical project runtime
      ↓
 ┌─────────────┬─────────────┐
 │             │
Codex bridge   Claude bridge
```

Codex and Claude may still have native subagent wrapper files because the hosts require different native formats. Those wrappers are adapter renderings of canonical Baron quality-agent contracts, not independent workflow owners.

---

## 3. Codex compatibility issue

Current Baron places Codex skills under `.codex/skills/**`.

Current Codex repo-local skill discovery uses `.agents/skills/**`, not `.codex/skills/**`.

Required final integration:

```text
AGENTS.md
.agents/skills/baron-engine/SKILL.md
.agents/skills/baron-engine/agents/openai.yaml
.codex/agents/*.toml
.codex/hooks.json
.codex/INDEX.md
```

The `baron-engine` skill is only a thin bridge. Real Baron skills remain under `.baron/core/skills/**`.

Because Codex defaults skill implicit invocation to enabled, the bridge should set `policy.allow_implicit_invocation: false` in `agents/openai.yaml` when AGENTS.md is the automatic routing authority. This prevents AGENTS.md routing and implicit native skill routing from both starting the same Baron task lifecycle.

---

## 4. Claude compatibility and target integration

Current Claude locations are broadly native-compatible:

- project skills: `.claude/skills/<skill>/SKILL.md`;
- project subagents: `.claude/agents/*.md`;
- project hooks/settings: `.claude/settings.json`;
- root project instructions: `CLAUDE.md`.

However Baron currently copies the full Baron skill tree into `.claude/skills/**`.

Final integration should use:

```text
CLAUDE.md
.claude/skills/baron-engine/SKILL.md
.claude/agents/*.md
.claude/settings.json
.claude/commands/...   # may remain as diagnostics/convenience
```

The bridge should not duplicate the Baron skill library. If CLAUDE.md is the automatic Baron routing authority, the bridge should not independently auto-start the same lifecycle. Use Claude's supported invocation-control metadata accordingly.

---

## 5. Managed ownership gap

Current managed state is keyed around adapter strings and target paths. There is no Core owner.

That prevents clean canonical Core ownership because both Codex and Claude must not claim `.baron/core/**`.

Required ownership model:

```text
Core
  .baron/core/**

Adapter(Codex)
  AGENTS.md Baron block
  .agents/skills/baron-engine/**
  .codex/** Baron-owned files/entries

Adapter(Claude)
  CLAUDE.md Baron block
  .claude/skills/baron-engine/**
  .claude/** Baron-owned files/entries
```

One live managed path must have exactly one owner.

---

## 6. Managed I/O bugs

### Read-as-empty bug

Managed writers contain patterns equivalent to:

```rust
fs::read_to_string(path).unwrap_or_default()
```

A permission failure, invalid UTF-8, directory-at-file-path, or I/O failure can therefore be treated as an empty file. A later write may overwrite user data.

Only `NotFound` may mean absent. Other errors must propagate.

### Pseudo-atomic replacement

Several `atomic_write` helpers do:

```text
write deterministic temp
remove original
rename temp to original
```

Problems:

- crash window where target is missing;
- deterministic temp collision;
- concurrent writer hazards;
- metadata/ACL drift;
- misleading "atomic" naming.

A single cross-platform safe replacement primitive should replace these helpers.

### Direct durable-state writes

Continuity, intent, autopilot and automation still contain direct `fs::write` or read-whole-file/append/rewrite patterns.

Important examples:

- automation journal reads the entire JSONL file, appends in memory, rewrites it;
- repo/Vault mirrored state can diverge if the first write succeeds and second fails;
- many optional reads hide all errors as "missing".

Long-task reliability requires these paths to become recoverable and concurrency-safe.

---

## 7. Global adapter state bug

`ProjectConfig` contains `active_adapter`, and runtime code can infer the current adapter from that project-global value or the first registered adapter.

That is unsafe when Codex and Claude can use the same project concurrently or alternately without an explicit switch.

Final rule:

- new lifecycle/hook/context/route/proof events receive adapter identity explicitly or from the native session;
- `active_adapter` is not correctness authority;
- do not require the user to switch adapters before opening Codex or Claude;
- old `active_adapter` may be read only for migration/UI compatibility until retired.

---

## 8. Control-plane sensitivity gap

Current `route_task` is primarily keyword/risk based.

Good properties:

- bounded routing;
- Superpowers remains the workflow owner;
- quality agents are separate gates;
- optional skills are lazy.

Weaknesses:

- platform profile is not a first-class input to routing;
- task phase/recent failure/change scope are not strong routing inputs;
- broad keywords can over-route, e.g. API work can trigger security too aggressively;
- there is no dedicated database-engineering skill;
- mobile application engineering has no dedicated development skill (current APK mobile skill is analysis-oriented).

Required router inputs should be approximately:

```text
project profile
+ task intent
+ work shape/risk
+ repository evidence / changed area
+ current task phase
+ previous failure/proof evidence
+ available capabilities
= minimal sufficient skills + agents + verification
```

Platform is a prior, not a hard lock. A README typo in a fullstack project should not load frontend/backend/database skills.

---

## 9. Platform-profile gap

Current flags already generate meaningful profile documents, but the user-visible promise should be stronger:

- `--fullstack` must affect cross-layer routing, contract checks and E2E proof;
- `--backend` must prioritize API/service/data/auth/observability behavior;
- `--mobile` must prioritize lifecycle/offline/storage/permissions/API/performance concerns;
- `--data` remains data/analytics/pipeline oriented;
- add a distinct `--database` profile for database engineering rather than overloading `--data`.

A database profile should understand:

- schema modeling;
- constraints and referential integrity;
- normalization/denormalization;
- indexes;
- query plans;
- transactions/isolation/locking;
- migrations/backfills/rollback;
- ORM/query boundaries;
- concurrency;
- data-loss safety;
- backup/recovery concerns where relevant.

Add a `database-engineering` optional Baron skill. Consider a `mobile-application-engineering` skill because the existing APK skill serves a different purpose.

---

## 10. Memory retrieval mismatch

Baron's newest `recall_v5` path correctly filters untrusted states such as Candidate, Contested, Superseded and Expired before semantic reranking.

However `compact_memory_brief_for_task`, which is inserted into compiled task context, currently calls the older base `recall()` path.

That creates a policy mismatch: the engine has a stronger trusted retrieval generation, but a context-critical path can still use older eligibility behavior.

Required rule:

**Every context-critical memory path must use the current trusted retrieval authority.**

Do not maintain separate "smart recall" and "context recall" trust policies.

---

## 11. Context truncation bug

The context compiler builds many useful sections and then enforces a fixed 20,000-character limit by flatly truncating the final string from the end.

This can remove later sections such as task-focused memory, session replay, autopilot/control-plane material, warnings or skipped-context diagnostics.

For long-horizon work this is the wrong failure mode.

Replace flat truncation with priority-aware budgeting.

Example priority:

### Tier 0 — never silently dropped

- project/task identity;
- user intent and constraints;
- current plan/continuity/recovery;
- router decision;
- material unknowns/blockers;
- required safety/proof gates.

### Tier 1

- relevant verified decisions/memory;
- changed files/current source evidence;
- proof/trace state;
- immediate next action.

### Tier 2

- platform lens;
- relevant CodeGraph/Wiki excerpts;
- bounded session replay.

### Tier 3

- low-priority diagnostics;
- autopilot candidate summary;
- general capability/reporting material.

Compress or omit lower tiers first. Long memory should stay external and retrieved; do not solve this by dumping the whole Vault into model context.

---

## 12. Per-prompt automation gap

Current SessionStart hook compiles generic context, but `Prompt` handling primarily records lifecycle state and does not automatically produce a complete task-focused routing/context packet.

The strongest non-tech design is:

```text
User prompt
  ↓
native UserPromptSubmit hook when available
  ↓
Baron prepare/route task
  ↓
trusted task context + selected skills + gates
  ↓
Codex/Claude works
```

Fallback when hooks are unavailable/untrusted:

```text
AGENTS.md / CLAUDE.md
  ↓
one high-level agent-facing Baron command
```

Do not make the human run `baron context`, `baron route`, `baron plan`, etc.

---

## 13. Too many low-level agent calls

The current startup contract asks the agent to run multiple Baron commands in sequence: authority, work-shape, capability, runtime, context, code-map decisions, route, gates, etc.

This is powerful but increases the chance an agent skips a step, calls commands in the wrong order, or passes stale state.

Add a high-level agent-facing orchestration command/protocol that internally composes existing modules, for example:

```text
baron control-plane prepare --adapter codex --json
baron control-plane prepare --adapter claude --json
```

with task input provided safely through native hook payload/stdin rather than shell interpolation.

The returned packet should include:

- task/work-shape decision;
- current/resumed task identity;
- profile lens;
- selected canonical skills;
- selected quality agents;
- bounded trusted context;
- required verification;
- warnings/blockers;
- next action.

Keep low-level commands for diagnostics/tests/backward compatibility, but generated adapter instructions should prefer the high-level protocol.

---

## 14. Prompt-to-shell injection risk

Generated instructions often show commands containing raw task text inside shell quotes.

Arbitrary user prompts may contain quotes, shell metacharacters or newlines. The adapter protocol should not require unsafe textual interpolation into shell commands.

Native hooks should pass structured JSON/stdin. Fallback CLI should support task input through stdin or another non-shell-interpolated channel.

---

## 15. Autopilot UX gap

Autopilot correctly treats learning candidates as untrusted and requires approval before runtime/policy mutation. Preserve that safety boundary.

But a non-technical user should not need to run approval CLI commands for ordinary work.

Policy:

- routine evidence-backed task state/memory maintenance may occur automatically under existing trust rules;
- proposed changes to skills, routing policy or engine behavior remain approval-gated;
- when human judgment is genuinely required, ask naturally in conversation and let the adapter record the decision;
- do not ask the user to operate hidden Baron CLI commands.

---

## 16. AGENTS.md problem

The repository's current `AGENTS.md` contains valuable invariants but is overloaded with stale phase/version history and a very large command/history narrative. It references older release phases and is not a clean current contract.

There are two separate instruction artifacts that need attention:

1. Baron Engine repository development `AGENTS.md`.
2. The managed Baron block written into users' project `AGENTS.md` by the Codex adapter.

Both should be rewritten.

The generated Codex block should be compact because Codex combines instruction files under a finite project-doc budget. It should contain invariants and the automatic runtime protocol, not the entire Baron architecture.

Likewise keep `CLAUDE.md` precise and adapter-specific.

---

## 17. retired adapter removal strategy

The user requires complete current-product removal.

Tracked source after the refactor should contain no retired adapter-specific implementation, CLI, documentation, tests, blueprints, generated paths, enum variants, release-plan documents or active/historical current-branch references.

Do **not** keep a `UnsupportedLegacy` enum just for migration. That violates the product-cleanup goal.

Instead use generic tolerant legacy parsing:

```text
supported adapter string -> Codex / Claude
unknown old adapter string -> UnsupportedLegacy(String)
```

This lets old project state remain readable without hard-coding the retired product name.

For old managed assets:

- records targeting `.baron/core/**` transfer to Core ownership regardless of old adapter owner;
- supported Codex/Claude records migrate normally;
- other unsupported adapter records may be retired only when the managed baseline proves they are Baron-owned and unchanged;
- modified/unknown files are preserved/quarantined, never recursively deleted.

Acceptance for the tracked current branch can require `git grep -i retired adapter` to return no matches. Git history/tags are not rewritten.

---

## 18. Generic Agent removal strategy

Because final support is exactly Codex + Claude, remove Generic as an active adapter/CLI/context target.

But Generic currently owns the only `.baron/core/**` runtime materialization. Therefore the safe order is:

1. create Core ownership/materialization;
2. migrate any existing `.baron/core/**` managed records to Core;
3. make Codex and Claude depend on Core;
4. only then remove Generic active adapter surfaces.

Do not delete `.baron/core` when removing Generic.

---

## 19. Concurrency and shared Vault risk

Codex and Claude may run concurrently against the same project and shared Vault.

Current read-modify-write journal/index patterns can lose updates.

Required locking model should cover:

- managed repository mutations;
- shared Vault durable state;
- journal append;
- baseline publication;
- migration/update transactions.

If repo and Vault live on different filesystems, do not claim cross-filesystem atomicity. Use recoverable transaction receipts and deterministic reconciliation.

---

## 20. CI baseline

Current CI already runs:

- format and warnings-denied Clippy;
- full workspace tests;
- Windows x86_64;
- Linux x86_64;
- Intel macOS;
- Apple Silicon macOS;
- locked release build;
- CLI version smoke.

Preserve this matrix and add focused adapter/compatibility/migration/concurrency cases rather than weakening it.

---

## Deep-audit verdict

The Baron core is valuable and should be preserved. The highest-value changes are:

1. remove retired adapter completely;
2. reduce active adapter surface to Codex + Claude;
3. extract `.baron/core/**` into a real adapter-independent canonical runtime;
4. make Codex/Claude thin consumers of that Core;
5. fix file I/O and shared-state concurrency;
6. remove global active-adapter authority;
7. unify all context recall on current trusted memory policy;
8. replace flat context truncation with priority-aware budgeting;
9. make platform profiles directly influence routing and verification;
10. add database/mobile development skill coverage;
11. move adapter automation to a high-level per-task protocol so the human does not micromanage hidden CLI commands;
12. rewrite AGENTS.md/CLAUDE.md as compact current contracts.
