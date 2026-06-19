# Baron Workspace Agent Guide

Read this file first when working in the `Baron-Engine` repository.

## Purpose

Baron is a Rust-first kit for multi-agent memory, context compilation, and repo
onboarding. It must stand on its own as an independent engine: vault memory,
repository readiness, proof gates, trace quality, and adapter-specific output
for multiple agent tools.

## Current Phase

Baron `v1.0.0` completed the first stable foundation. Baron `v3.0.0` is the
current stable source release. Phase 18-23 complete the Baron 3.0 program:
self-contained runtime assets, skill lifecycle approval, bounded session
replay/search, background learning autopilot, safe runtime backend policy, and
release certification.

The current source command surface is:

- `baron survey`
- `baron survey --json`
- `baron setup --vault [vault-path]`
- `baron init --codex --shadow`
- `baron init --claude --shadow`
- `baron init --agent --shadow`
- `baron init --codex|--claude|--agent`
- `baron init --frontend|--backend|--fullstack|--mobile|--desktop|--tool|--library|--data|--cloud`
- `baron memory status [repo-path] --vault <vault-path>`
- `baron memory index [repo-path] --vault <vault-path>`
- `baron memory compact [repo-path] --vault <vault-path>`
- `baron memory import-sessions [repo-path] --vault <vault-path>`
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
- `baron migrate agent-bootstrap [repo-path] --dry-run`
- `baron migrate agent-bootstrap [repo-path]`
- `baron migrate <status|rollback>`
- `baron capability <register|check|list|remove>`
- `baron runtime check`
- `baron automation <status|reconcile|hook>`
- `baron continuity <status|checkpoint>`
- `baron autopilot <status|review|approve|reject>`
- `baron control-plane <status|route|record-gate|evidence>`
- `baron asset <audit|quarantine|propose-skill>`
- `baron session-replay <index|search|replay>`
- `baron harness <audit|verify-all|intervention|propose|outcome>`
- `baron certify <run|status>`

Phase 6 imports useful legacy data into Baron-native structures, validates or
quarantines custom assets, verifies parity, and removes Agent Bootstrap managed
runtime only after Baron passes. Phase 7 adds adapter-aware capability
registration, presence checks, graceful degradation, and structured execution
evidence. Phase 8 ships native releases, checksums, lifecycle installers, and
cross-platform smoke proof without changing Baron's core. Phase 9-10 add
IDE-compatible observable automation, collision-resistant identity,
incremental massive memory, multilingual task-aware recall, and automatic
session ingestion. Phase 11-12 add strict skill/agent contracts, explainable
routing, mandatory gate evidence, context-read scoring, drift audits,
interventions, improvement proposals, and outcome tracking. Phase 13-14 add
extreme-scale certification and Baron 2.0 release hardening.
Phase 15 keeps those internals available but hides the command clutter from the
normal README and top-level help so users only handle install, Vault setup,
adapter init, platform focus, and update. Phase 16-17 refine optional
skill/agent routing and add Continuity Ledger checkpoints so interrupted work
resumes from evidence instead of memory guesses. Phase 18-23 make managed
runtime skills and agents self-contained Baron assets, stage skill edits behind
approval metadata, let context/search replay bounded prior messages for the
current project without flooding context, propose background learning as
candidates, and block unsafe or unverified runtime backends from proof claims.

Do not implement a phase without updating `docs/BARON_STATUS.md`,
`docs/BARON_STATUS.json`, `notes/build-log/CURRENT.md`, and the active design or
plan. After every meaningful batch, record what passed, what remains, and the
exact resume point.

## Non-Negotiables

- Rust is the primary engine language.
- Do not require a Baron launcher for normal use; users may open agents through
  an IDE or the agent tool directly.
- Use native lifecycle hooks where supported, managed adapter instructions
  where hooks are unavailable, and reconciliation/evidence checks to detect
  missed automatic actions.
- Do not call instruction-only behavior guaranteed automation.
- Treat native hooks as observable accelerators: preserve custom hooks, report
  trust requirements, and keep reconciliation available when hooks do not run.
- Use project ID, not folder basename, as the memory isolation boundary.
- Keep session import bounded, exact-repo-matched, redacted, and deduplicated.
- Do not restore fixed file-count truncation in survey or memory indexing.
- Keep Superpowers as the only workflow core and the three core quality agents
  as evidence-backed gates.
- Do not count a mandatory quality gate unless `baron control-plane record-gate`
  captured evidence that it actually ran.
- Product Harness may propose improvements, but must not rewrite core policy or
  architecture without human approval.
- Vault Markdown remains the source of truth.
- SQLite/cache/index files are accelerators only.
- Baron must support old repos through shadow-first onboarding.
- Baron must support multiple agent tools through adapters.
- Baron must preserve Superpowers as the workflow core.
- Baron must never depend on Agent Bootstrap runtime after migration.
- Legacy migration imports data and user-owned assets, not old architecture.
- Legacy migration must remain transactional: inventory, backup, import,
  validation, native install, verification, cleanup, receipt.
- Modified legacy runtime and invalid custom assets must be quarantined rather
  than deleted.
- Skills and agents must pass Baron-native contracts before activation.
- Managed runtime skills and agents must be self-contained local Baron assets;
  attribution and license notes belong outside operational `SKILL.md` guidance.
- Weak, conflicting, or externally dependent custom assets should be quarantined
  or staged for approval instead of silently trusted.
- Session replay must stay bounded and filtered by project identity.
- Tool-backed proof requires execution evidence; configured or detected presence
  alone is not proof that a check ran.
- Autopilot learning writes candidates only; unapproved learning is not a fact,
  not policy, and not a runtime asset change.
- Required runtime backends must be safe enough and have execution evidence
  before they can support proof or completion claims.
- At session start, agents silently run `baron capability check` before
  `baron context`; context reads only the bounded cache summary.
- At session start, agents silently run `baron runtime check` and
  `baron autopilot status`; after meaningful work they run
  `baron autopilot review`.
- Missing optional providers degrade with warnings. Missing required providers
  or missing execution evidence block Proof/Trace completion gates.
- Baron must include exactly three core quality agents:
  - `code-reviewer`
  - `security-auditor`
  - `test-engineer`
- Baron may ship bundled optional domain skills:
  - `frontend-design`
  - `vibe-security-scan`
  - `api-and-interface-design`
  - `observability-and-instrumentation`
  - `performance-optimization`
  - `deprecation-and-migration`
- Baron may ship optional agents such as `web-performance-auditor`, but optional
  agents are not core gates and do not count as mandatory gate evidence.
- Optional skills and optional agents must stay lazy-loaded and routed.
- AI agents must not recursively read all skills, agents, docs, or memory.
- Unknown facts must be marked unknown instead of guessed.
- `.baron/project.toml` is shared project routing, never memory.
- `.baron/local.toml` is machine-local Vault routing and must stay ignored.
- Adapter updates must preserve user text outside Baron markers and unknown
  custom skills/agents, including their routing registrations.
- High-risk plans must not complete without valid proof and a detailed passing
  trace.
- Product Harness intake must maintain `docs/baron/harness/TEST_MATRIX.md`;
  proof recording updates the current story evidence in both repo and Vault,
  but weak evidence must remain `insufficient`.
- A failed trace score is a hard automation stop, not an informational warning.
- Baron-managed plan, harness, adapter, and config files do not count as
  product-file changes for detailed trace quality.
- Release installers must verify checksum and staged binary version before
  replacement.
- Release rollback and uninstall must never delete project or Vault data.
- Do not call Phase 8 complete from local Windows proof alone; hosted Windows,
  Linux, Intel macOS, and Apple Silicon macOS jobs must pass.

## Read Order

1. `README.md`
2. `docs/BARON_STATUS.md`
3. `notes/build-log/CURRENT.md`
4. `docs/specs/2026-06-08-baron-product-spec-1.0.md`
5. `docs/roadmap/2026-06-08-implementation-roadmap.md`
6. `docs/architecture/ARCHITECTURE.md`
7. `docs/architecture/MEMORY_MODEL.md`
8. `docs/architecture/ADAPTERS.md`
9. `docs/architecture/CAPABILITY_REGISTRY.md`

## Build Notes Rule

Use `notes/build-log/` while Baron is being built. Keep `CURRENT.md` updated
when work starts, changes direction, gets interrupted, or finishes a phase.

During meaningful implementation, record a continuity checkpoint before edits,
after direction changes, before interruption, and before final response. The
resume packet must identify current task, last checkpoint, proof status, trace
status, and next action.

This folder is temporary and can be deleted after Baron reaches a mature release.
Do not put source-of-truth product decisions only in build notes.

Use `docs/BARON_STATUS.md` as the durable progress dashboard. Update it whenever
a phase starts, completes, changes proof status, or changes the next action.

## Verification

For the current Baron 3.0 source, verify:

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
cargo run -p baron-cli -- asset audit
cargo run -p baron-cli -- session-replay index . --vault .tmp/baron-vault
cargo run -p baron-cli -- session-replay search "auth login" . --vault .tmp/baron-vault
cargo run -p baron-cli -- plan status
cargo run -p baron-cli -- harness status
cargo run -p baron-cli -- proof status
cargo run -p baron-cli -- trace score
cargo run -p baron-cli -- capability list
cargo run -p baron-cli -- capability check
cargo run -p baron-cli -- runtime check
cargo run -p baron-cli -- autopilot status
cargo test -p baron-core --test release
cargo test -p baron-cli --test lifecycle_scripts
cargo test -p baron-cli --test release_smoke
cargo test -p baron-cli --test certification_cli
cargo test -p baron-cli --test workflow_contract
cargo clippy --workspace --all-targets -- -D warnings
```

Release completion also requires the GitHub Actions native runner matrix and a
tagged GitHub Release with verified archives, checksums, manifest, and
installers.
