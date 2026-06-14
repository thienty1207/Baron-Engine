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

Phase 5 has implemented the Survey Engine, Vault + Memory Firewall, bounded
Context Compiler, multi-agent adapters, and execution-state gates:

- `baron survey`
- `baron survey --json`
- `baron init --codex --shadow`
- `baron init --claude --shadow`
- `baron init --agent --shadow`
- `baron memory status [repo-path] --vault <vault-path>`
- `baron memory index [repo-path] --vault <vault-path>`
- `baron memory compact [repo-path] --vault <vault-path>`
- `baron recall "<query>" [repo-path] --vault <vault-path>`
- `baron context [repo-path] --codex --vault <vault-path>`
- `baron context [repo-path] --claude --vault <vault-path>`
- `baron context [repo-path] --agent --vault <vault-path>`
- `baron context [repo-path] --why --vault <vault-path>`
- `baron update [repo-path]`
- `baron plan <status|start|update|interrupt|complete>`
- `baron harness <status|intake|decision|friction>`
- `baron proof <status|record>`
- `baron trace <record|score>`

The next major phase is Agent Bootstrap Migration. Do not implement later phases
without updating `docs/BARON_STATUS.md` and `notes/build-log/CURRENT.md`.

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
- `.baron/project.toml` is shared project routing, never memory.
- `.baron/local.toml` is machine-local Vault routing and must stay ignored.
- Adapter updates must preserve user text outside Baron markers and unknown
  custom skills/agents.
- High-risk plans must not complete without valid proof and a detailed passing
  trace.
- Product Harness intake must maintain `docs/baron/harness/TEST_MATRIX.md`;
  proof recording updates the current story evidence in both repo and Vault,
  but weak evidence must remain `insufficient`.
- A failed trace score is a hard automation stop, not an informational warning.
- Baron-managed plan, harness, adapter, and config files do not count as
  product-file changes for detailed trace quality.

## Read Order

1. `README.md`
2. `docs/BARON_STATUS.md`
3. `notes/build-log/CURRENT.md`
4. `docs/specs/2026-06-08-baron-product-spec-1.0.md`
5. `docs/roadmap/2026-06-08-implementation-roadmap.md`
6. `docs/architecture/ARCHITECTURE.md`
7. `docs/architecture/MEMORY_MODEL.md`
8. `docs/architecture/ADAPTERS.md`

## Build Notes Rule

Use `notes/build-log/` while Baron is being built. Keep `CURRENT.md` updated
when work starts, changes direction, gets interrupted, or finishes a phase.

This folder is temporary and can be deleted after Baron reaches a mature release.
Do not put source-of-truth product decisions only in build notes.

Use `docs/BARON_STATUS.md` as the durable progress dashboard. Update it whenever
a phase starts, completes, changes proof status, or changes the next action.

## Verification

For the current foundation through Phase 5, verify:

```bash
cargo fmt --all
cargo test
cargo run -p baron-cli -- --help
cargo run -p baron-cli -- survey .
cargo run -p baron-cli -- survey . --json
cargo run -p baron-cli -- init . --codex --shadow
cargo run -p baron-cli -- memory status . --vault .tmp/baron-vault
cargo run -p baron-cli -- memory index . --vault .tmp/baron-vault
cargo run -p baron-cli -- memory compact . --vault .tmp/baron-vault
cargo run -p baron-cli -- recall "survey engine proof" . --vault .tmp/baron-vault
cargo run -p baron-cli -- context . --codex --task "implement auth login" --vault .tmp/baron-vault
cargo run -p baron-cli -- context . --claude --vault .tmp/baron-vault
cargo run -p baron-cli -- context . --agent --vault .tmp/baron-vault
cargo run -p baron-cli -- context . --why --vault .tmp/baron-vault
cargo run -p baron-cli -- init . --codex --vault .tmp/baron-vault
cargo run -p baron-cli -- update .
cargo run -p baron-cli -- plan status
cargo run -p baron-cli -- harness status
cargo run -p baron-cli -- proof status
cargo run -p baron-cli -- trace score
```

Later phases must add deeper smoke tests for `survey`, `init`, `context`,
`recall`, `memory`, `plan`, `harness`, and adapter outputs.
