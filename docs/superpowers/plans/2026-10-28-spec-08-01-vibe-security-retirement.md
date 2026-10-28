# SPEC-08-01 — Core Routing and Vibe Retirement

**Status:** Implemented and verified locally; pending owner review

## Objective

Replace the active `vibe-security-scan` route with the canonical Baron-owned
`security-audit` skill, automatically select it for meaningful application
code work, and retire old Core-managed files only when the existing ownership
baseline proves they are unchanged. Keep `security-auditor` as an independent
quality gate. This implements SPEC-08-01 only; it does not complete SPEC-08.

## Scope and design choice

The selected design follows the parent SPEC-08 architecture: Core owns the
route and asset identity, adapters only project Core decisions, and no upstream
runtime/package is installed. A superficial rename or reuse of the Vibe audit
corpus would misrepresent the intended replacement. A broad audit on every
request would violate the bounded-routing requirement. The first security-audit
asset will therefore identify the new Core contract and its honest capability
boundary; the seven-language corpus, full audit phases, sandboxed execution,
and remediation loop remain owned by SPEC-08-02 through SPEC-08-05.

Meaningful work is application source, build/dependency/CI configuration,
security policy, or a data-flow boundary. Documentation-only and cosmetic
work remain out of the focused route unless they touch a security boundary.
The prepare contract must distinguish routing from evidence that an audit
actually ran; selecting the skill is not a passed security check.

## Implementation sequence

1. Add Core routing regressions for routine application changes, security
   sensitive work, docs-only work, scope bounds, and preservation of all three
   mandatory gates when security is sensitive or high risk.
2. Add a backward-compatible prepare security-audit plan describing selected
   scope/mode, known languages, required validation, and unavailable
   capabilities without claiming execution or a clean result.
3. Replace current product routes, managed registries, adapter indexes,
   benchmark owners, and active guidance with `security-audit`; retain only
   exact legacy compatibility needed for safe migration.
4. Add update regressions for unchanged, modified, absent, and ambiguous legacy
   Vibe files; preserve user adapter assets/hooks/configuration; prove retries
   and failed publication cannot publish a route without its target asset or
   discard prior ownership/recovery evidence.
5. Reconcile the managed Core payload transaction so a removed upstream
   payload is not silently orphaned. Retire only exact-baseline bytes, preserve
   edited/unknown files with an explicit conflict, and make retries safe.
6. Run focused Core and adapter/CLI tests after each batch, then formatter,
   full workspace all-target tests, Clippy, CLI help, relative-link checks,
   status JSON parsing, and the tracked legacy-name gate. Use the supported
   PowerShell 7 test override on this Windows host.

## Files and flows to revalidate during implementation

- Core: `control_plane.rs`, `prepare.rs`, `asset_lifecycle.rs`,
  `migration.rs`, `platform.rs`, `intelligence.rs`.
- Adapter: `install.rs`, `update.rs`, generated Codex/Claude indexes and
  `phase4_core`, `phase14_update`, `update_planner`, and lifecycle tests.
- Runtime assets: `assets/core/skills/vibe-security-scan/**`,
  `assets/core/skills/api-and-interface-design/SKILL.md`,
  `assets/core/skills/apk-mobile-analysis/SKILL.md`.
- Core, adapter, and CLI routing/context/capability fixtures and tests.
- Maintained handoff/status/build log and current product benchmark claims.

The source-reading guide warns that this is a targeted flow inventory, not a
complete repository source pass. Final reporting will state exact coverage and
any uninspected paths.

## Checkpoint and persisted-state boundary

- Worktree branch: `codex/spec-08-01-vibe-retirement`.
- Base/source checkpoint: `origin/main` at
  `c1c182a187e62332a32ce1824f6658f261ce84c8`.
- At plan creation, only the six owner-provided Spec8 documents were copied
  into this isolated worktree. Implementation is now local and uncommitted.
- A new regression first reproduced docs-only false-positive routing when the
  repository contains an auth surface. Security-audit now routes from task or
  affected-path evidence, not repository-wide survey findings alone.
- Initial full workspace testing found a runtime skill contract mismatch; the
  exact required headings and explicit no-clean-result wording were added.
  Clippy then found test-only modules before later production items; the two
  modules were moved to the ends of their files. Final Windows verification
  passed formatting, all 112 workspace test targets (1,055 passed, 0 failed,
  4 ignored), and warnings-denied Clippy. The run used bundled PowerShell 7 via
  `BARON_TEST_POWERSHELL`.
- Focused reruns passed: Core control plane 22/22, security plan unit tests
  5/5, CLI prepare 8/8, adapter lifecycle 23/23, update planner 26/26, and
  adapter retirement I/O unit test 1/1. The docs-only regression on a repo
  containing an auth surface was observed RED before the routing fix and GREEN
  afterward.
- Source coverage remains targeted to SPEC-08-01 routing, PreparePacket,
  managed asset/update migration, related tests, and runtime assets. This is
  not a complete line-by-line source pass of the repository.
- Full Windows tests use the supported `BARON_TEST_POWERSHELL` override for
  bundled PowerShell 7 to avoid the host's Windows PowerShell 5.1 module
  autoload limitation.
- Proof and trace: no Baron project proof/trace; verification uses tests and
  disposable fixtures only.
- No project, Vault, `.env`, machine configuration, primary checkout, or
  original user data is in the write boundary.
- No commit, push, PR, merge, tag, version bump, or release is authorized.

## Safe next action

Documentation links, JSON parsing, and diff checks passed. Stop for owner
review; do not start another SPEC-08 child, commit, push, merge, or release.
