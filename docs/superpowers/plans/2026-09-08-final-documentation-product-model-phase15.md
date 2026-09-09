# Baron Codex + Claude Core Consolidation — Phase 15

**Status:** `completed`

**Goal:** Align the maintained public product model, architecture and
generated integration contracts with the completed Phase 1–14 implementation.

**Authority:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
The current source and generated payloads are the evidence; this plan records
where documentation must be corrected rather than inventing new behavior.

## Scope boundary

- Phase 15 only: README, maintained architecture and compatibility docs,
  generated AGENTS/CLAUDE contract wording, command-surface positioning,
  changelog, status/spec framing, link/command checks, and documentation tests.
- Do not redesign working Core, memory, routing, Autopilot, update, or adapter
  behavior. A concrete correctness mismatch may receive one focused fix only.
- Phase 16 adversarial verification, release/tag/push/publication, version
  bumps, and history rewrites remain out of scope.
- Preserve the zero retired-adapter content and filename gates.

## Actual documentation dependency map

1. `README.md` is the public entry point but still describes explicit adapter
   switching and deep commands as part of normal use. It must present one
   Baron Core with Codex and Claude native projections and a natural ask-first
   workflow.
2. `docs/architecture/ARCHITECTURE.md`, `ADAPTERS.md`,
   `CAPABILITY_REGISTRY.md`, `COMMAND_SURFACE.md`, and `CONTEXT_COMPILER.md`
   contain accurate implementation facts but mix chronological phase notes,
   low-level commands, and older adapter terminology. They need a current
   authority model with diagnostics clearly separated.
3. `docs/compatibility/CODEX.md` and `CLAUDE.md` already describe most Phase
   8–14 generated surfaces. They need explicit compactness, parent lifecycle,
   host-memory, selected-resource, and fallback wording aligned to the actual
   generated payloads.
4. `crates/baron-adapters/src/install.rs` is the generated-contract source of
   truth. Documentation tests must assert invariants against generated AGENTS,
   CLAUDE, bridge metadata, Core ownership, hooks, and selected resources rather
   than brittle paragraph text.
5. `CHANGELOG.md` starts at the released 4.2.2 entry. An Unreleased section is
   required to describe the uncommitted Core-consolidation work without
   claiming a 4.2.3 release.
6. The refactor spec and deep audit remain engineering provenance. Their status
   must make Phase 15 completion and Phase 16 verification pending explicit;
   neither should be the normal user entry point.

## Test-first batches

### Batch A — public model and compatibility docs

- Add/extend documentation invariants for supported adapters, canonical Core,
  no manual adapter switching, Autopilot trust, hook acceleration, Prepare
  boundary, Database/Data distinction, and no semantic adapter-local copies.
- Rewrite README and compatibility pages using executable current commands and
  accurate Codex/Claude layouts.

### Batch B — architecture and command alignment

- Rewrite maintained architecture pages around authority ownership, explicit
  `OperationContext`, PrepareRequestV1/PreparePacketV1, TrustedRecallPolicy,
  Task State projection, Tier 0–3 context, routing, hooks, Autopilot, and safe
  update/downgrade behavior.
- Audit documented command examples against current `--help` output; keep
  internal commands under advanced/diagnostic positioning.
- Add a lightweight internal Markdown-link check if existing tooling supports
  it without a new docs toolchain.

### Batch C — status, spec, changelog, and final documentation verification

- Add an Unreleased changelog entry, mark the authoritative spec as complete
  through Phase 15 with Phase 16 pending, and retain the deep audit as clearly
  historical evidence.
- Update status JSON/Markdown, the current plan pointer, and build log.
- Run focused documentation/generated-contract tests, command/help checks,
  retired-adapter gates, formatter, workspace tests, Clippy, and diff checks.

## Implementation rules

- Prefer invariant assertions over whole-paragraph snapshots.
- Keep generated AGENTS and CLAUDE contracts compact; do not add operational
  behavior solely to satisfy documentation.
- Do not introduce retired-adapter literals, filenames, or fake compatibility
  examples.
- Treat local PowerShell archive-module failures as host findings, not a reason
  to weaken documentation or installer gates.
- No release artifact rebuild is required unless a generated runtime template
  changes; if it does, run the focused embedded-contract tests.

## Exit evidence

Phase 15 ends with the exact report headings required by the accepted brief,
including final public model, README, supported adapters, Core ownership,
normal/advanced command boundaries, Codex/Claude contracts, memory/Task State/
context, routing/profiles, hooks, Autopilot, update/downgrade, command/link
checks, historical docs, changelog, documentation tests, persisted schemas,
retired-adapter gates, and Phase 16 readiness. No Phase 16 work starts here.

## Completion evidence (2026-09-08)

- Maintained README, architecture, memory, compatibility, blueprint, release,
  demo, assessment, and refactor-index docs now describe one Baron Core with
  thin Codex and Claude projections and a normal ask-first workflow.
- Generated payload invariants pass for Core ownership, bridge metadata,
  selected-resource loading, hook fallback, Task State, recovery, proof, and
  compact root contracts. Installer templates were unchanged.
- Focused docs, adapter, bridge, hook, CLI, update, and release suites pass;
  CLI help, local Markdown links, status JSON, formatter, workspace tests,
  Clippy, diff checks, and zero retired-adapter gates pass.
- No persisted schema, production runtime, version, release, tag, push, or
  history change was made for Phase 15.
