# Baron 3.4 To 3.6 Controlled Extension Program Design

Date: 2026-07-24
Status: approved release direction; implementation not started
Current source release: `3.3.0`
Current release target: `3.4.0`
Program target: `3.6.0`

## Goal

Grow Baron through three independently certifiable releases without creating a
second workflow owner, a second runtime asset source, unbounded context, or a
new manual command burden for normal users.

The program has three jobs:

1. Baron `3.4.0` removes the stale core blueprint copy and makes `baron update`
   safe and recoverable.
2. Baron `3.5.0` strengthens existing frontend, interface, and product-language
   guidance with selected ideas from Hallmark and Matt Pocock's skills.
3. Baron `3.6.0` adds an optional, local, project-scoped Graphify code-map
   provider behind Baron capability, context, and memory-firewall boundaries.

## Source Audit Baseline

The design was checked against these exact source revisions:

| Source | Revision | Intended use |
| --- | --- | --- |
| `obra/superpowers` | `v6.2.0`, `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9` | Already vendored and verified as Baron's sole workflow core |
| `nutlope/hallmark` | `aeb42fb354ff4efa36ab475773a082315a3af2ce` | Design differentiation and anti-template quality gates |
| `mattpocock/skills` | `ed37663cc5fbef691ddfecd080dff42f7e7e350d` | Deep-module and domain-language techniques only |
| `Graphify-Labs/graphify` | `v8` branch, `0.9.25`, `2fa6cd3d5548577f8c5f591b713f0bf80c1af183` | Optional local code-only graph provider |

These repositories are research inputs, not runtime authorities. Baron does not
run their installers, subscribe to changing remote instructions, or let them
write Baron hooks and agent instruction files.

## Non-Negotiable Ownership

- Superpowers remains the only workflow core.
- The mandatory quality gates remain `code-reviewer`, `security-auditor`, and
  `test-engineer`.
- `assets/core/` is the only bundled runtime source.
- Vault Markdown remains the durable memory source of truth.
- SQLite and code graph artifacts remain rebuildable caches.
- Product Harness owns product intent, risk, proof, trace, and domain language.
- Baron Survey and Context Compiler remain the fallback when an optional
  provider is missing, stale, incompatible, or failed.
- Normal users keep the small install, setup Vault, init adapter/platform, and
  update flow.
- AI automation may use hidden maintenance commands, but it cannot silently
  install a Baron release or a third-party provider.

## Why `blueprints/core` Must Be Removed First

The current tree contains 145 files under `assets/core/` and seven files under
`blueprints/core/`. Six blueprint files reuse active asset paths, but every one
has different content from the corresponding runtime asset. No Rust module,
installer, test, workflow, or manifest reads `blueprints/core/`; only historical
documentation still names it.

Keeping that directory creates two false signals:

- an agent may edit the obsolete copy instead of the embedded runtime source
- the 3.4 managed baseline may accidentally be designed around two owners

Phase 35 therefore begins by deleting `blueprints/core/`, removing obsolete
documentation claims, and adding a regression test that the bundled installer
reads only `assets/core/`. This cleanup is completed before the managed
baseline manifest is introduced.

## Release 3.4.0 - Safe Update Foundation

Baron `3.4.0` stays focused on one promise: `baron update` can refresh Baron and
Baron-managed project assets without losing user work or Vault memory.

Phase 35 first establishes one runtime asset source, then records exact managed
baseline bytes and computes `BASE`, `LOCAL`, and `UPSTREAM` decisions without
writing live targets.

Phase 36 verifies the exact native release candidate before it receives any
activation authority.

Phase 37 stages conflicts and applies project/runtime changes as one recoverable
transaction.

Phase 38 separates human remote-update authority from AI local repair, performs
full certification, and only then changes the source version to `3.4.0`.

No Hallmark, Matt Pocock, or Graphify behavior enters the runtime before Baron
3.4 update safety is certified.

## Release 3.5.0 - Skill Intelligence Without Duplicate Skills

Baron does not add `hallmark`, `matt-skills`, `tdd`, `grilling`, `implement`, or
another workflow directory.

### Hallmark Distillation

Selected Hallmark ideas strengthen the existing `frontend-design` skill:

- choose a brief-specific visual fingerprint before selecting components
- detect generic AI defaults and repeated template composition
- preserve product information architecture and copy during redesign work
- require responsive, interaction-state, accessibility, and long-content proof
- distinguish an audit from an implementation or redesign
- perform a bounded pre-final self-critique against the changed UI surface

The large theme/component catalogue is not copied wholesale. Baron keeps a
small set of principle-oriented references so task context stays bounded and
the agent designs for the actual product instead of choosing a named template.

### Matt Pocock Distillation

Only two non-overlapping technique families are accepted:

- `codebase-design` deep-module principles strengthen Baron's existing
  `api-and-interface-design` guidance and architecture governor
- `domain-modeling` terminology discipline strengthens Product Harness through
  a project-owned `docs/baron/harness/DOMAIN_LANGUAGE.md`

The following source skills are explicitly rejected from the bundled runtime
because Superpowers or existing Baron systems already own them:

- TDD, debugging, implementation, code review, prototyping, planning, grilling,
  handoff, research, issue triage, ticket conversion, and workflow routing
- setup scripts that write foreign conventions into a project
- in-progress, deprecated, personal, or product-specific utilities

Product Harness may create the domain-language file when missing, but Baron
never invents domain terms. Agents add or correct terms only from repository,
user, product, or verified runtime evidence. Compact context loads a bounded
active glossary excerpt rather than the entire history.

### 3.5 Runtime Shape

The release modifies existing ownership surfaces:

```text
assets/core/skills/frontend-design/
  SKILL.md
  NOTICE.md
  references/
    anti-template-gates.md
    brief-fingerprint.md
    responsive-state-proof.md

assets/core/skills/api-and-interface-design/
  SKILL.md
  NOTICE.md
  references/
    deep-module-boundaries.md

docs/baron/harness/
  DOMAIN_LANGUAGE.md
```

No new bundled skill or core agent is introduced.

## Release 3.6.0 - Optional Project-Scoped Code Graph

Graphify can improve architecture and dependency questions, but its stock
installer also supports hooks, instruction-file edits, global graphs, optional
semantic backends, and its own work-memory layer. Baron must not delegate those
responsibilities.

### Provider Boundary

Baron integrates Graphify as an optional `code-map` capability provider. The
initial supported contract is Graphify `0.9.25` at the audited revision.
Unsupported versions degrade to Baron Survey instead of being executed.

Baron never invokes:

- `graphify install`
- `graphify hook install`
- `graphify global`
- `graphify save-result`
- `graphify reflect`
- any Graphify platform installer or strict read-blocking hook

Baron invokes only a local code-only extraction and bounded query contract:

```text
graphify extract <repo> --code-only --out <baron-cache> --no-cluster
graphify query <question> --graph <graph-json> --json --budget <bounded>
```

`--code-only` prevents document/media semantic backends. The provider disables
Graphify query logging for Baron-owned calls and never sends source, docs, or
memory to a remote service.

### Cache And Identity

Graph state is rebuildable and project-local:

```text
.baron/cache/code-graph/
  state.json
  graphify/
    <source-revision>/
      graphify-out/
        graph.json
```

`state.json` records:

- Baron project identity
- canonical repository root
- source revision or source fingerprint
- Graphify version
- graph checksum and byte size
- build time, freshness, and diagnostics

No graph is written to the Vault. No global or cross-project graph is queried.
Two repositories with the same directory name still receive distinct graph
state through Baron project identity.

### Bounded Context Flow

For architecture, dependency, impact, and code-navigation tasks:

1. Context Compiler checks for a compatible fresh provider.
2. Baron sends one bounded query derived from the current task.
3. Baron parses a strict JSON response and keeps only a small number of hits.
4. `EXTRACTED` edges may guide source selection.
5. `INFERRED` edges are visibly labeled and must be verified against repository
   source before they can support proof, decisions, or durable memory.
6. If any check fails, Baron emits a short diagnostic and uses Survey Engine.

Graph output does not outrank current repository source, approved Vault facts,
or verified proof. It is a navigation aid, not truth.

### Automatic Behavior

Normal users receive no new primary command. Generated agent instructions may
silently use hidden Baron code-map maintenance when the task benefits from it.
The work is skipped for small copy, documentation, and unrelated tasks.

Automatic refresh is bounded by:

- a tested provider version
- source revision/fingerprint freshness
- one project identity
- command timeout
- output and graph-size caps
- no network and no external hook installation

## Version Gates

The source version changes only at release certification:

- `3.3.0` remains current while Phases 35-37 are incomplete.
- Phase 38 may bump to `3.4.0` after all 3.4 proof passes.
- Phase 41 may bump to `3.5.0` after routing, adapter, behavior, and preservation
  proof passes.
- Phase 45 may bump to `3.6.0` after provider absence, incompatibility, failure,
  isolation, scale, and context-bound proof passes.

Planning documents never change the package version.

## Certification Standard

Every release runs:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
git diff --check
```

Additional release-specific proof:

- 3.4: update planner, candidate identity, conflict, rollback, Windows handoff,
  installer, custom assets, and shared Vault preservation
- 3.5: three-adapter asset parity, narrow routing, source provenance, controlled
  skill pressure tests, responsive/state proof, and domain-language boundedness
- 3.6: no-provider fallback, wrong-version refusal, timeout/malformed/oversized
  output fallback, two-project isolation, stale graph invalidation, old/large
  repository smoke, and proof that no hooks or instruction files were modified

## Consequence

Baron becomes stronger by deepening existing owners and adding one isolated
accelerator. It does not become stronger by loading more instructions into
every session. The release sequence prevents skill work and code-graph work
from obscuring update-safety failures.
