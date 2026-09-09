# Baron Workspace Agent Guide

Read this file before changing the `Baron-Engine` repository. Baron is a
Rust-first engine whose canonical product semantics live in Baron Core. Codex
and Claude are the supported native adapters over that Core.

## Current implementation boundary

The Codex + Claude Core consolidation is implemented through Phase 15. Phase 15
finishes the public product model and maintained documentation. Phase 16 is the
next adversarial verification phase and has not started. Do not release, tag,
push, publish, bump the version, or rewrite history as part of Phase 15.

The durable status dashboard is [`docs/BARON_STATUS.md`](docs/BARON_STATUS.md),
the machine-readable status is [`docs/BARON_STATUS.json`](docs/BARON_STATUS.json),
and the active plan is in `docs/superpowers/plans/`.

## Product model

```text
assets/core/** → .baron/core/** → Codex bridge / Claude bridge
```

Core owns project identity, Vault memory, trusted recall, Task State, plans,
Superpowers workflow, profiles, routing, continuity, recovery, proof, trace,
quality gates, and Autopilot policy. One live managed path has one owner. Core
owns `.baron/core/**`; Codex and Claude own only their host projections.

Adapters are thin translations. They preserve user instructions, source files,
custom skills and agents, unknown configuration keys, and third-party hooks.
Native hooks are observable accelerators. The managed `AGENTS.md` or
`CLAUDE.md` contract remains the fallback when hooks do not run.

## Normal user flow

Users install Baron, set up a Vault, initialize Codex or Claude with an optional
profile, and ask for work in the host normally. The native integration sends a
structured `PrepareRequestV1`; Core returns a `PreparePacketV1` with bounded
trusted context, selected resources, gates, blockers, and the next action.
Users do not operate the internal command catalog for ordinary work.

Supported initialization profiles are `frontend`, `backend`, `fullstack`,
`mobile`, `desktop`, `tool`, `library`, `data`, `database`, `cloud`, and
`unknown`. Data and Database are separate domains. Routing uses task intent,
configured profile, repository evidence, work shape, risk, and current Task
State; it must work even before any file has changed.

## Core invariants

- Rust remains the primary engine language.
- Vault Markdown is durable source of truth; SQLite and indexes are
  rebuildable accelerators.
- Project ID, never folder basename, is the memory isolation boundary.
- `TrustedRecallPolicy::Current` guards every context-critical retrieval path.
  Candidate, contested, superseded, expired, stale, and unrelated project
  records cannot become current truth.
- Task State and recovery preserve intent, constraints, non-goals, plan,
  last successful step, proof/trace state, affected files, blockers, and next
  safe action.
- Superpowers is the workflow core. The exactly three mandatory quality agents
  are `code-reviewer`, `security-auditor`, and `test-engineer`.
- Optional skills, agents, hooks, Wiki, CodeGraph, session replay, and
  capability probes remain lazy, bounded accelerators.
- A mandatory gate counts only when Core records execution evidence. Presence
  or configuration alone is not proof.
- Autopilot may perform bounded housekeeping and write candidates for review.
  It cannot silently change policy, routing, workflow, memory truth, or Core
  assets.
- Unknown facts remain unknown. High-risk completion requires valid proof and a
  passing trace.
- `.baron/project.toml` stores committed project routing and identity;
  `.baron/local.toml` stores ignored machine-local Vault routing.
- Updates and migration are transactional and preserve user-owned files.
  Missing, malformed, changed, or ambiguous baselines fail closed and leave a
  recoverable receipt. Rollback and uninstall never delete project or Vault
  data.
- Legacy serialized adapter values are tolerated generically for migration and
  diagnostics. They are never activated as a supported integration.

## Read order

1. `README.md`
2. `docs/BARON_STATUS.md`
3. `notes/build-log/CURRENT.md`
4. `docs/architecture/ARCHITECTURE.md`
5. `docs/architecture/MEMORY_MODEL.md`
6. `docs/architecture/ADAPTERS.md`
7. `docs/architecture/CAPABILITY_REGISTRY.md`
8. `docs/architecture/COMMAND_SURFACE.md`
9. `docs/architecture/CONTEXT_COMPILER.md`

The dated refactor files under `docs/refractor/` are engineering provenance;
their status and historical framing explain which claims are current.

## Build and continuity rules

Keep `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, the active plan, and
`notes/build-log/CURRENT.md` updated whenever a phase starts, changes direction,
or completes. Before meaningful edits, record the current task, checkpoint,
proof status, trace status, persisted-state boundary, and safe next action.
Never hide an unexpected failure by labelling it expected red.

Preserve user-owned files and migration safety. Write tests first where
practical, run focused tests after each batch, and keep the workspace buildable.
Do not add a production seam unless it is behavior-neutral, deterministic, and
covered by a test explaining why it is needed.

## Command boundary

Normal users need only installation, `setup --vault`, `init --codex` or
`init --claude` with a profile, and `update`. Survey, memory, plan, harness,
proof, trace, continuity, capability, runtime, automation, Autopilot,
control-plane, migration, asset, replay, certification, and release commands
remain available for diagnostics, hooks, tests, migration, or maintainers.
See `docs/architecture/COMMAND_SURFACE.md` for the complete catalog and the
structured prepare protocol.

## Verification

At minimum for a completed change, run:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
```

For adapter or documentation changes also run the focused Core and adapter
contract suites, CLI help checks, repository-relative link checks, status JSON
parsing, and the retired-adapter gate:

```text
git grep -in <retired-adapter-name>
```

The gate must return no tracked current-product references. Historical Git
objects and tags are outside the working-tree contract.
