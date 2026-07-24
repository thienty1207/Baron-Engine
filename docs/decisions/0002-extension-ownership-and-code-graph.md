# 0002 - Extension Ownership And Optional Code Graph

Status: accepted
Date: 2026-07-24

## Decision

Baron accepts new skills, agents, providers, and analysis features only when
they strengthen an existing responsibility without creating a second owner for
the same decision.

Every accepted extension must be:

- optional unless it is an existing Baron core contract
- lazy-routed by task need
- bounded in time, output, and persisted state
- isolated by Baron project identity
- backed by a local fallback
- prohibited from silently editing Baron instructions or durable memory
- covered by routing, preservation, failure, and adapter tests

Superpowers remains the only workflow core. The three mandatory quality gates
remain `code-reviewer`, `security-auditor`, and `test-engineer`.

## Superpowers 6.2

Baron vendors the complete upstream Superpowers `v6.2.0` skill tree at commit
`3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`, then applies one recorded local
hardening patch to keep the visual companion offline.

The upstream files are pinned local assets. Baron owns the root routing wrapper,
adapter installation, memory, context, plan, harness, proof, and trace
integration. Runtime behavior must not depend on a live GitHub download.

The 6.2 behavior Baron relies on includes:

- plan-scoped SDD workspaces instead of one shared progress ledger
- bounded review-fix loops that resume the original implementer first
- behavior-focused tests rather than text-presence checks alone
- portable per-agent tool references and Windows-safe session startup

The visual brainstorming server's remote brand image, telemetry signal, and
live branding link are removed. Both the original upstream digest and Baron's
patched digest are stored in `UPSTREAM.json`.

## Optional Code Graph

Graphify may be evaluated as an optional, project-scoped code-map provider. It
does not become Baron's memory engine, workflow owner, Vault source of truth, or
automatic instruction installer.

An eventual provider must:

- default to local code analysis
- cache by Baron project identity and source revision
- expose only bounded query results to context
- label inferred relationships and require source verification before durable
  memory or proof
- never use a cross-project global graph for normal project context
- never install external hooks, skills, or instruction blocks into a Baron
  project
- fall back to Baron's Survey Engine and Context Compiler when unavailable,
  stale, or failed

No Graphify runtime dependency is added by this decision.

## Optional Skill Sources

Hallmark is not added as a second frontend workflow owner. Useful anti-template
and anti-generic-design checks may be distilled into Baron's existing
`frontend-design` domain skill and its verification references.

Matt Pocock's workflow, grilling, planning, and TDD material is not imported
because Superpowers already owns those responsibilities. Only future
domain-specific techniques with a clear non-overlapping contract may be
considered.

External source material must be vendored or rewritten into self-contained
Baron assets with a pinned source revision, license/notice, local tests, and no
live operational dependency.

## Runtime Asset Source

`assets/core/` is the only runtime source for bundled Baron skills and agents.
`blueprints/core/` is a stale historical duplicate and must not receive new
changes. A later controlled cleanup will remove it after a test proves no
installer, migration, release, or documentation path reads it.

## Consequences

- Adding more integrations cannot silently widen Baron's core.
- Shared Vault memory remains isolated from third-party graph caches.
- Optional providers can improve orientation without making normal startup
  depend on them.
- New assets must prove a unique responsibility and measurable value before
  they ship.
- Baron 3.4 safe self-update remains the active implementation program; these
  decisions do not change its current phase order or release version.
