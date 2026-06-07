# Baron Workspace Agent Guide

Read this file first when working in the `Baron-Engine` repository.

## Purpose

Baron is a new Rust-first kit for multi-agent memory, context compilation, and
repo onboarding. It must become stronger and more durable than both:

- `agent-bootstrap-obsidian-cli`
- `repository-harness`

Baron does this by combining vault memory, repository harnessing, proof gates,
trace quality, and adapter-specific output for multiple agent tools.

## Current Phase

Phase 0 only creates the foundation:

- product spec
- roadmap
- architecture skeleton
- temporary build notes
- Rust workspace skeleton
- core asset and adapter blueprints

Do not implement the full Baron engine until the product spec and roadmap are
reviewed.

## Non-Negotiables

- Rust is the primary engine language.
- Vault Markdown remains the source of truth.
- SQLite/cache/index files are accelerators only.
- Baron must support old repos through shadow-first onboarding.
- Baron must support multiple agent tools through adapters.
- Baron must preserve Superpowers as the workflow core.
- Baron must include exactly three core quality agents:
  - `code-reviewer`
  - `security-auditor`
  - `test-engineer`
- Baron may ship bundled optional domain skills:
  - `frontend-design`
  - `vibe-security-scan`
- Optional skills and optional agents must stay lazy-loaded and routed.
- AI agents must not recursively read all skills, agents, docs, or memory.
- Unknown facts must be marked unknown instead of guessed.

## Read Order

1. `README.md`
2. `docs/specs/2026-06-08-baron-product-spec-1.0.md`
3. `docs/roadmap/2026-06-08-implementation-roadmap.md`
4. `docs/architecture/ARCHITECTURE.md`
5. `docs/architecture/MEMORY_MODEL.md`
6. `docs/architecture/ADAPTERS.md`
7. `notes/build-log/CURRENT.md`

## Build Notes Rule

Use `notes/build-log/` while Baron is being built. Keep `CURRENT.md` updated
when work starts, changes direction, gets interrupted, or finishes a phase.

This folder is temporary and can be deleted after Baron reaches a mature release.
Do not put source-of-truth product decisions only in build notes.

## Verification

For phase 0, verify:

```bash
cargo test
cargo run -p baron-cli -- --help
```

Later phases must add deeper smoke tests for `survey`, `init`, `context`,
`recall`, `memory`, `plan`, `harness`, and adapter outputs.
