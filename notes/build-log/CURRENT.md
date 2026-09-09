# Current Build Note

## Phase 16 v5.0.0 Release Candidate (2026-09-09)

- Current task: complete the verified Baron Engine 5.0.0 release through the
  protected hosted signing workflow.
- Last checkpoint: SEC-03 and SEC-04 are closed locally; fresh native Codex
  Security scan `6312e662-b7ba-45da-b42a-d02fba00193f` reported zero
  reportable findings on the final Windows/Linux snapshot. The clean Windows
  release binary is `baron.exe`, 17,526,272 bytes, SHA-256
  `45f301fc89fa527104d62d65938848b07d938bf6b68eec9396f186c44347c4c6`, and
  reports `baron 5.0.0`.
- Proof status: the post-bump formatter, complete workspace test matrix,
  Clippy, release build, ignored release smoke, final-binary four-way init,
  mixed Codex/Claude prepare, and diff checks pass with zero unexpected
  failures. The production signing secret remains only in GitHub Actions.
- Trace status: the source is an uncommitted 5.0.0 release candidate; no tag,
  GitHub Release, or hosted result is claimed yet.
- Persisted-state boundary: release staging has touched source, tests,
  installers, workflow, assets, and documentation only; project and Vault
  data remain outside the repository changes.
- Safe next action: inspect the final diff, commit the candidate, push `main`,
  observe required hosted CI and protected signing preflight, then stop if any
  hosted gate is red.

## Phase 16 SEC-03 Remediation In Progress (2026-09-09)

- Current task: close release metadata authenticity using the provisioned
  `baron-release-2026` public identity, then independently rerun Phase 16.
- Last checkpoint: production key pin, base64 seed decoding, detached manifest
  signature, authenticated Rust updater, and authenticated bootstrap installer
  paths are now implemented in the working tree. The product remains 4.2.2;
  no release action has occurred.
- Proof status: focused release, installer, self-update, and workflow contract
  suites pass after the remediation batch. SEC-04 remains closed.
- Trace status: fresh native security review and the complete Phase 16 matrix
  are still required before any version bump or release action.
- Persisted-state boundary: only release source, tests, installers, workflow,
  and documentation are being changed; project and Vault data are untouched.
- Safe next action: run the complete local matrix, inspect the final diff, run a
  fresh native security scan, then stop or proceed according to the closure
  criteria.

## Phase 16 Security Remediation Start (2026-09-09)

- Current task: remediate only release metadata authenticity (SEC-03) and
  proof/gate evidence authority (SEC-04) before reconsidering the blocked
  v5.0.0 release.
- Last checkpoint: Baron remains version 4.2.2 with no version bump, commit,
  push, tag, or publication. The signed manifest envelope, compiled trust
  anchor, current-operation receipt binding, and focused tests are in the
  working tree.
- Proof status: focused release, self-update, receipt, control-plane, plan,
  proof, runtime-policy, complete workspace tests, formatter, Clippy, release
  build, binary smoke, and diff checks pass with zero unexpected failures.
  SEC-04 is closed by typed current-operation receipt authority.
- Trace status: SEC-03 remains open. Fresh native Codex Security scan
  `40bbf799-a3dd-4fa3-a219-c0c6ede300c7` found one HIGH unsigned
  bootstrap-installer metadata path on the final snapshot, and protected
  signing-key provisioning is not configured or verifiable on this host.
- Safe next action: configure the protected signing identity and repair the
  bootstrap-installer trust path, then rerun a fresh security review and the
  complete Phase 16 gate without release staging.

## Phase 16 Final Release Decision (2026-09-09)

- Current task: close Phase 16 after the final adversarial and local release
  verification; publish v5.0.0 only if every gate is trustworthy and green.
- Last checkpoint: focused adversarial fixtures, the complete workspace test
  sweep, formatter, Clippy, release build, current-binary smoke, diff checks,
  and retired-adapter guards pass. Three ignored Phase 1 tests are obsolete
  historical behavior and fail only when explicitly forced; the ignored Phase
  14 release smoke passes.
- Proof status: local proof is green, but release eligibility is blocked by
  unresolved high-severity trust findings in unsigned release metadata and
  forgeable persisted capability/gate evidence. The native security scan was
  zero-finding for a pre-repair snapshot; TAC was unavailable.
- Final documentation compatibility check passes after retaining the existing
  Phase 16 status phrase while recording the blocked release note. Retired
  adapter content and filename guards also pass across the working tree.
- The release workflow now emits the accepted `Baron Engine v$version` tag
  message and GitHub Release title; the workflow contract suite passes.
- Trace status: no version bump, commit, push, tag, GitHub Release, or hosted
  CI claim. The current release binary remains 4.2.2.
- Safe next action: design the missing release-authenticity and proof-evidence
  trust boundary in a separately authorized phase, then rerun Phase 16. Do not
  publish v5.0.0 or begin Phase 17.

## Phase 16 Start / Independent Audit Checkpoint (2026-09-09)

- Current task: independently verify the completed Phase 1–15 Baron Core,
  Codex bridge, and Claude bridge implementation and release v5.0.0 only if
  every gate passes.
- Last checkpoint: the worktree is on `main` at the same commit as
  `origin/main`; cumulative Phase 1–15 changes are present and uncommitted;
  there are no conflict markers, tracked `target/` files, secrets found by the
  initial review, or retired-adapter content/filename matches.
- Proof status: formatter and the compatible Windows PowerShell workspace
  sweep pass with zero failures. Three ignored Phase 1 fixtures fail only when
  explicitly run because their historical behavior was hardened in Phase 2;
  the ignored release smoke passes against the existing 4.2.2 binary.
- Trace status: no release, version bump, commit, push, tag, or publication has
  occurred. Local evidence is source review, deterministic tests, and current
  binary smoke; hosted matrix evidence is still required.
- Persisted state: project, managed-state, memory, Task State, continuity,
  hook/dedup, Autopilot, PreparePacket, and protocol schemas remain unchanged
  until the release gate is green.
- Safe next action: run the adversarial verification matrix and high-risk
  mutation-path review; stop before version metadata if any blocker appears.

## Phase 16 Repair Checkpoint (2026-09-09)

- Current task: continue the final adversarial audit after a concrete
  link-escape repair.
- Last checkpoint: a Windows junction test first demonstrated that project
  initialization could write through linked `.baron` state before rejecting a
  later path. `ensure_directory_chain` now guards initialization; shared
  config, Vault, control-plane, proof, trace, plan, intent, harness, platform,
  session, architecture, and review writers use safe replacement. Config and
  Vault readers reject linked/reparse entries.
- Proof status: the focused config and Phase 16 adversarial suites pass,
  including the linked control-plane fixture. Full workspace gates remain to
  be rerun after this repair.
- Trace status: no version bump, release metadata update, commit, push, tag,
  or publication has occurred.
- Safe next action: run the full local test/Clippy/build matrix and finish the
  remaining release evidence before deciding whether v5.0.0 is eligible.

## Phase 15 Completion Checkpoint (2026-09-08)

- Current task: Phase 15 final public documentation, product model, generated
  contract audit, and command-surface alignment is complete.
- Last checkpoint: maintained docs now describe Baron Core as the canonical
  owner, Codex and Claude as thin projections, structured prepare, trusted
  memory/Task State tiers, profile routing, hook fallback, Autopilot trust, and
  preserve-first update/recovery. Historical audits and plans are labelled as
  provenance.
- Proof status: focused public-doc, link/framing, generated-contract, adapter,
  bridge, hook, CLI, update, release, workspace, format, Clippy, diff, and
  zero retired-adapter checks pass. CLI help matches documented examples.
- Trace status: no external model execution or release publication is claimed;
  evidence is source-backed documentation, generated payloads, CLI help, and
  deterministic tests.
- Persisted state: PreparePacket v1 and all project, managed-state, memory,
  task, continuity, hook, dedup, Autopilot, and release schemas are unchanged.
  No version bump, release, tag, push, publication, or history rewrite.
- Safe next action: prepare Phase 16 adversarial verification only. Do not
  begin it or publish a release in this phase.

## Phase 15 Start / Documentation Dependency Checkpoint (2026-09-08)

- Current task: align final public documentation, generated Codex/Claude
  contracts, and command positioning with the completed Phase 1–14 source.
- Last checkpoint: Phase 14 passed its compatibility, update/recovery, release
  binary, workspace, Clippy, formatter, diff, and retired-adapter gates. No
  Phase 15 documentation work had started before this checkpoint.
- Findings: README still presents adapter switching and deep engine commands as
  normal workflow; maintained architecture docs are chronological/internal;
  compatibility pages need final host-memory and fallback wording; the current
  changelog lacks an Unreleased section; generated contracts need invariant
  checks against the actual installer payloads.
- Proof status: no Phase 15 documentation proof yet. The first safe action is
  test-first documentation/generated-contract invariants.
- Trace status: no external model execution is claimed; evidence will come
  from source-backed docs checks and CLI/help verification.
- Persisted state: no schema, runtime, release, tag, push, version, or history
  work is authorized in this phase.
- Safe next action: add the focused documentation tests, then update only the
  maintained docs and generated-contract descriptions.

## Phase 14 Completion Checkpoint (2026-09-08)

- Current task: Phase 14 update, downgrade, migration, recovery, and local
  release-package hardening is complete.
- Last checkpoint: future project/managed writers now fail closed; state-guard
  errors retain actionable schema causes; update completion commits a runtime
  checkpoint before writing additive receipt evidence; receipt-boundary failure
  is recoverable; Core ownership, thin projections, task/continuity, hooks,
  dedup, and Autopilot state are preserved.
- Proof status: focused Phase 14 suites pass; the complete workspace matrix
  passes with zero failures under system Windows PowerShell; release build and
  Codex/Claude/Database binary smoke pass; formatter, Clippy, diff, and
  retired-adapter gates pass. The default Codex-bundled PowerShell runtime
  blocks two installer tests before assertions because its archive module
  cannot autoload.
- Trace status: no external model execution is claimed; release evidence is a
  local optimized binary and deterministic temporary-project smoke.
- Persisted state: project, managed-state, memory, transaction, PreparePacket,
  journal/dedup, continuity, and Autopilot schema versions remain unchanged.
  No tag, push, publication, version bump, or history rewrite occurred.
- Safe next action: prepare the Phase 15 documentation rewrite only; do not
  begin Phase 16 or release work.

## Phase 14 Regression Repair Checkpoint (2026-09-08)

- Current task: finish Phase 14 verification after update/recovery hardening.
- Last checkpoint: the workspace sweep exposed one regression in the existing
  state-guard schema test because the new future-project-schema refusal was
  wrapped without preserving its cause. The guard now includes the original
  actionable schema error while retaining the repair guidance.
- Proof status: the focused state-guard regression passes after the behavior-
  neutral error-preservation fix. The full workspace sweep must be rerun.
- Trace status: no external model execution is claimed; evidence remains local
  deterministic tests and build checks.
- Persisted state: no schema changes; no release, tag, push, version, or history
  work.
- Safe next action: rerun the complete workspace tests, then release/build,
  clippy, format, diff, and retired-adapter gates.

## Phase 14 Start / Dependency Map Checkpoint (2026-09-08)

- Current task: implement only Phase 14 update, downgrade, migration,
  recovery, and release-package hardening.
- Last checkpoint: Phase 13 Autopilot UX was accepted with no changes to the
  project, managed-state v2, memory, or PreparePacket v1 schemas. Source audit
  confirms managed-state migration, safe-I/O locking, append-safe journal,
  dedup refusal, and transaction rollback foundations are present.
- New findings: direct project writers do not reject a future project schema;
  completion receipts omit source/file/rollback/next-action evidence and are
  written before the terminal state commit; receipt failure has no explicit
  recoverable checkpoint. These are the first Phase 14 red-test targets.
- Proof status: no Phase 14 production proof yet. Tests must be written and run
  red before the minimal hardening edits.
- Trace status: no external model execution is claimed; evidence will come
  from deterministic fixtures and local build/test commands.
- Persisted state: preserve existing schemas unless a focused test proves an
  additive writer floor is necessary. No release/tag/push/version/history work.
- Safe next action: add the persisted compatibility and transaction fixtures;
  do not begin Phase 15.

## Phase 13 Archive Idempotency Checkpoint (2026-09-08)

- Current task: verify conversational response idempotency after resolved
  candidates are archived by safe housekeeping.
- Discovery: response fingerprints were searched only through active candidates,
  so a retry after archival could lose its no-op identity.
- Safe next action: add the archive retry fixture first, then search the
  existing bounded archive without changing the ledger schema.
- Scope guard: no Phase 14, release, tag, push, version, or history work.

## Phase 13 Final Verification Checkpoint (2026-09-08)

- Current task: Phase 13 Autopilot UX and safe candidate learning is complete.
- Last checkpoint: natural-language approval now fails closed when more than
  one unrelated candidate is pending; the deterministic Core fixture covers
  this ambiguity boundary alongside correlation, idempotency, conflict,
  suppression, correction, scope, recovery, memory, and child safety.
- Proof status: Phase 13 Core `14/14`, CLI `2/2`, full workspace tests, and
  workspace Clippy pass; three historical Phase 1 fixtures remain explicitly
  ignored. Formatter, diff, status JSON, and retired-adapter gates pass.
- Trace status: no external model execution is claimed; all evidence is from
  deterministic fixtures and repository verification commands.
- Persisted schema: Project, managed-state v2, memory, and PreparePacket v1
  remain unchanged. The additive Autopilot ledger remains schema 1 and old
  Markdown records remain untrusted evidence.
- Safe next action: prepare the Phase 14 plan only. Do not begin Phase 14,
  release, tag, push, version bump, or history rewrite.

## Phase 13 Clarification Safety Checkpoint (2026-09-08)

- Current task: close the final Phase 13 conversational approval edge case.
- Discovery: the resolver selected the newest item when multiple unrelated
  approvals were pending, which could guess at user intent.
- Safe next action: add a deterministic ambiguity regression first, then make
  the resolver fail closed when correlation does not select exactly one item.
- Scope guard: no Phase 14, release, tag, push, version, or history work.

## Phase 13 Completion Checkpoint (2026-09-08)

- Current task: complete only Autopilot UX, safe candidate learning,
  housekeeping, and conversational approval.
- Last checkpoint: Autopilot now keeps deterministic project/Vault candidate
  state with provenance, scope, impact, readiness, contradiction handling,
  bounded expiry/archive housekeeping, and explicit response correlation.
  Natural-language approval/correction/defer/rejection resolves without an ID;
  approved project decisions call the existing Product Harness authority.
  Pending and approved Autopilot files carry candidate frontmatter and remain
  outside trusted current memory. Stop hooks only run bounded housekeeping;
  resolved records archive without losing response idempotency.
- Proof status: Phase 13 Core `14/14` and CLI `2/2` pass. All prior relevant
  memory, routing, PreparePacket, continuity, hook, adapter, operation, and
  retired-adapter gates pass. The compatible system Windows PowerShell module
  path workspace sweep passes with zero failures and three intentionally
  ignored historical fixtures. Formatter, Clippy, and diff checks pass.
- Trace status: no external model execution is claimed; evidence is from
  deterministic fixtures and repository verification commands.
- Persisted schema: Project, managed-state v2, memory, and PreparePacket v1
  remain unchanged. Autopilot ledger schema 1 is additive and old-record
  tolerant; legacy Markdown candidates are imported as untrusted evidence.
- Safe next action: prepare the Phase 14 plan only. Do not begin Phase 14,
  release, tag, push, version bump, or history rewrite.

## Phase 13 Start Checkpoint (2026-09-08)

- Current task: implement only Autopilot UX, safe candidate learning,
  housekeeping, and conversational approval.
- Last checkpoint: Phase 12 native hook acceleration is complete and accepted.
  Source inspection shows timestamp candidate IDs, whole-file Autopilot writes,
  no project-scoped approval correlation, and no defer/correction/conflict or
  suppression lifecycle. PreparePacket v1 and existing decision/memory/firewall
  authorities are available for additive integration.
- Proof status: no Phase 13 production proof yet; deterministic red target tests
  are the next action.
- Trace status: no external model execution is claimed. Candidates must remain
  untrusted evidence until explicit approval reaches an existing authority.
- Persisted schema: preserve Project, managed-state v2, memory, and
  PreparePacket v1. Any Autopilot-only state must be additive and old-record
  tolerant.
- Safe next action: add Phase 13 target tests first. Do not begin Phase 14,
  release, tag, push, version bump, or history rewrite.

## Phase 12 Completion Checkpoint (2026-09-08)

- Current task: complete only Phase 12 native hook acceleration, lifecycle
  idempotency, parent/child boundaries, and reliable checkpoint events.
- Last checkpoint: normalized Codex/Claude lifecycle events now route through
  bounded Core handling; UserPromptSubmit reuses structured PreparePacketV1,
  SessionStart is bounded and idempotent, PreCompact is checkpoint-only, and
  Stop reconciles without claiming completion. Child evidence is bounded and
  parent-owned; recursion and duplicate delivery are guarded.
- Proof status: focused Phase 12 suites pass: Core `9/9`, adapters `3/3`, CLI
  `4/4`. Relevant prior Core, adapter, and CLI regression suites pass. The
  exact workspace command reports two host-only installer failures before
  assertions when the default Codex PowerShell runtime loads an incompatible
  `Microsoft.PowerShell.Archive`; with the system Windows PowerShell module
  path the full workspace sweep passes `568` tests, has `3` intentionally
  ignored historical fixtures, and has `0` failures, including lifecycle
  `5/5`.
- Trace status: no external model execution is claimed; evidence is from
  deterministic fixtures, focused/regression tests, formatter, Clippy, diff,
  status JSON, and retired-adapter grep gates.
- Persisted schema: project, managed-state v2, memory, and PreparePacket v1
  schemas remain unchanged. Journal/checkpoint fields are additive; the
  bounded restart dedup cache is `.baron/cache/automation-dedup.json` schema 1.
- Safe next action: prepare the Phase 13 plan only. Do not begin Phase 13,
  release, tag, push, version bump, or history rewrite.

## Phase 12 Start Checkpoint (2026-09-08)

- Current task: implement only native Codex/Claude hook acceleration,
  lifecycle idempotency, parent/child boundaries, and reliable checkpoint
  events.
- Last checkpoint: Phase 11 completed the Codex/Claude-only supported surface;
  this phase begins from the accepted working tree without resetting or
  rewriting earlier work.
- Proof status: Phase 11 evidence remains valid. Phase 12 target tests are not
  yet added or run.
- Trace status: no external model execution is claimed; this checkpoint is
  based on source inspection and the accepted Phase 11 test evidence.
- Persisted schema: preserve project schema, managed-state v2, memory schemas,
  and PreparePacket v1 unless an additive journal/checkpoint field is proven
  necessary.
- Safe next action: write deterministic Phase 12 target tests before changing
  production behavior. Do not begin Phases 13-16, release, tag, push, or
  history rewrite.

## Phase 11 Completion Checkpoint (2026-09-08)

- Current task: complete only final supported-adapter cleanup. Active runtime
  integrations are Codex and Claude; unsupported historical values are
  migration/diagnostic data only.
- Last checkpoint: Phase 11 removed unsupported runtime, CLI, context, hook,
  installer, update, test, asset, and current-documentation surfaces. The
  pre-cleanup inventory was 476 tracked content match lines and 3 tracked
  filenames containing the retired adapter token.
- Proof status: Phase 11 focused target tests, Core/adapter/CLI regressions,
  and all non-installer workspace targets pass. The default Codex PowerShell
  module path makes two installer tests fail before their assertions because
  `Microsoft.PowerShell.Archive` resolves from an incompatible runtime module.
  Re-running the same workspace command with the system module path reports
  568 passed, 0 failed, and 3 intentionally ignored historical fixtures; the
  focused installer suite is 5/5.
- Trace status: no external model execution is claimed; evidence is from
  deterministic fixtures, source checks, and test output.
- Persisted schema: PreparePacket v1, managed-state v2, memory schemas, and
  project schema are unchanged. Unsupported values remain in opaque legacy
  fields and cannot select an adapter.
- Safe next action: prepare Phase 12 only. Do not begin Phase 12, release,
  tag, push, or history rewrite.

## Phase 10 Final Checkpoint (2026-09-08)

- Current task: complete only Phase 10, removing project-global adapter
  correctness authority while retaining the serialized `active_adapter`
  compatibility field.
- Last checkpoint: explicit `OperationContext` identity is threaded through
  prepare, route/context, capability/runtime, proof, journal, continuity, and
  CLI correctness paths. Codex and Claude share semantic task state and retain
  their own session/request provenance. The legacy capability-proof wrapper
  fails closed instead of promoting cached adapter state. Switch/status remain
  compatibility UI.
- Proof status: Phase 10 Core `14/14`, CLI `8/8`, including Codex to Claude to
  Codex resume, all Core tests, all focused
  prepare/trusted-memory/context/profile-routing/Codex-bridge/Claude-bridge/
  proof/trace/automation/session/continuity/update/lifecycle targets passed.
  The required workspace sweep passes all product targets except two known
  Windows installer tests blocked by unavailable `Microsoft.PowerShell.Archive`.
- Trace status: no external model execution is claimed; the phase evidence is
  source-level and deterministic fixture/test evidence. `cargo fmt --all --
  --check`, workspace Clippy, `git diff --check`, and status JSON validation
  pass.
- Persisted schema: `PreparePacketV1` and managed-state v2 are unchanged;
  `active_adapter` still parses and remains available for compatibility/UI.
- Safe next action: prepare the Phase 11 plan only. Do not begin Phase 11,
  release work, or final documentation cleanup in this checkpoint.

## Phase 10 Start Checkpoint (2026-09-08)

- Current task: begin only Phase 10, removing project-global adapter
  correctness authority while preserving the serialized `active_adapter`
  compatibility field.
- Last checkpoint: Phase 9 Claude native thin bridge is complete and its
  focused/regression verification remains green. The Phase 10 plan is saved at
  `docs/superpowers/plans/2026-09-08-remove-global-adapter-authority-phase10.md`.
- Proof status: no Phase 10 production proof yet; target tests must be added and
  their current RED behavior classified first.
- Trace status: no Phase 10 trace claim yet.
- Safe next action: add the Phase 10 Core and CLI target fixtures/tests, run
  them before production edits, and stop if any baseline regression appears.

## Phase 9 Final Checkpoint (2026-09-08)

- Current task: complete only the Claude Code Native Thin Bridge after Phase 8
  acceptance.
- Last checkpoint: Claude projects a compact managed `CLAUDE.md` contract, an
  explicit disabled-invocation bridge skill, three native Core-provenance
  wrappers, retained diagnostic commands/indexes, and merged native settings
  hooks over the shared `.baron/core/**` runtime. Fresh installs do not create
  copied semantic Claude skill trees; ownership-safe writes preserve user,
  team, collision, and Codex content.
- Proof status: Phase 9 adapter bridge `8/8`, CLI bridge `5/5`, promoted target
  `1/1`, Phase 4 Core `9/9`, Phase 3 ownership `12/12`, Phase 2 safe I/O `8/8`,
  Phase 5 prepare `5/5`, Phase 6 trusted memory `4/4`, Phase 7 routing `19/19`,
  Phase 8 Codex bridge `6/6`, Phase 8 Codex CLI `5/5`, adapter lifecycle
  `30/30`, update planner `15/15`, CLI adapter `11/11`, profile CLI `2/2`,
  context compiler `19/19`, platform intelligence `5/5`, config `14/14`,
  continuity `6/6`, and update transaction `1/1` passed. `cargo fmt --all --
  --check`, the workspace sweep, workspace Clippy, `git diff --check`, and
  status JSON validation passed. The workspace sweep reports only the two
  known Windows installer failures caused by unavailable
  `Microsoft.PowerShell.Archive` support.
- Trace status: no external Claude model execution is claimed; only native
  file-format and CLI evidence is recorded. The two known Windows lifecycle
  installer failures remain separately classified as unavailable
  `Microsoft.PowerShell.Archive` host support.
- Auto-memory decision: Claude host auto memory is host-local and
  non-authoritative; it is not imported into trusted Baron memory, does not
  receive Baron durable state, and cannot override Baron intent, continuity,
  or recovery.
- RED evidence preserved: the new adapter target initially failed `5/8` for
  the absent Claude bridge, wrappers, compact contract, and collision-safe
  projection; the CLI target initially failed `3/5` for the absent bridge,
  contract path, and native snapshot. The Claude shadow preview also first
  lacked the new bridge/wrapper paths. These were feature RED results and all
  became green after the smallest Phase 9 projection changes.
- Safe next action: prepare the Phase 10 plan; do not implement Phase 10.

## Phase 9 Start Checkpoint (2026-09-08)

- Current task: implement only the Claude Code Native Thin Bridge after Phase
  8 acceptance.
- Last checkpoint: Codex has the Phase 8 thin bridge. Claude currently writes
  `CLAUDE.md`, diagnostic commands/indexes, and merged native settings hooks,
  but does not yet publish the Claude bridge skill or native Core-provenance
  subagent wrappers.
- Proof status: Phase 9 plan is saved; adapter and CLI target fixtures are next
  and must be observed RED before production edits.
- Trace status: no Phase 9 production proof yet. The two known Windows
  `Microsoft.PowerShell.Archive` lifecycle failures remain separately
  classified.
- Safe next action: write the Phase 9 Claude fixtures, run focused RED tests,
  then implement only the thin bridge and preservation seam.

## Phase 8 Final Checkpoint (2026-09-08)

- Current task: complete only the Codex Native Thin Bridge after Phase 7
  acceptance.
- Last checkpoint: Codex now projects a compact `AGENTS.md` contract, one
  `.agents/skills/baron-engine` bridge with `openai.yaml` metadata, three thin
  `.codex/agents` wrappers, `.codex/INDEX.md`, and merged native hooks over one
  `.baron/core/**` runtime. Fresh installs do not create `.codex/skills`; old
  or user-owned files at that path remain preserved input.
- Proof status: Phase 8 bridge `6/6`, CLI `5/5`, promoted target `1/1`, Phase 4
  Core `9/9`, Phase 3 ownership `12/12`, Phase 2 safe I/O `8/8`, Phase 5
  prepare `5/5`, Phase 6 trusted memory `4/4`, Phase 7 routing `19/19`,
  adapter lifecycle `30/30`, and update planner `15/15` passed. Full workspace
  verification passed every product target except the two known lifecycle
  installer tests blocked by the unavailable `Microsoft.PowerShell.Archive`
  module on this Windows host. `cargo fmt --all -- --check`, workspace
  Clippy, and `git diff --check` passed.
- Trace status: source-level Codex CLI `0.153.4` version/help evidence only;
  no external model execution or runtime completion claim. No release, tag, or
  GitHub push was created.
- Safe next action: prepare the Phase 9 plan; do not begin Phase 9 in this
  checkpoint.

## Phase 8 Start Checkpoint (2026-09-07)

- Current task: implement only the Codex Native Thin Bridge after Phase 7
  acceptance.
- Last checkpoint: Phase 7 profile-aware routing is complete; the workspace is
  still uncommitted and no release/tag was created.
- Proof status: Phase 8 fixtures and target tests are being added before any
  production projection change.
- Trace status: no Phase 8 production proof yet; known Windows PowerShell
  Archive lifecycle test limitations remain separate.
- Safe next action: run the new Codex bridge tests and confirm feature RED,
  then implement the smallest bridge/wrapper/preservation seam.

## Phase 7 Final Verification Checkpoint (2026-09-07)

- Current task: publish the Phase 7 report and stop before Phase 8.
- Last checkpoint: the exact final tree passed focused routing `19/19`,
  profile, context, prepare, canonical Core, config, promoted Phase 6,
  formatter, Clippy, diff, and status JSON checks. The full workspace sweep
  completed with no new product failures; normal APK release routing remains
  separate from APK reverse analysis.
- Proof status: Phase 7 is complete; only the two known installer tests are
  host-limited by unavailable `Microsoft.PowerShell.Archive`.
- Trace status: no release or tag.
- Safe next action: prepare the Phase 8 plan; do not implement Phase 8.

## Codex + Claude Core Consolidation - Phase 7 Completed (2026-09-07)

- Scope completed: bounded profile-aware routing, distinct Database/Data
  routing, canonical database-engineering and mobile-application-engineering
  Core skills, proportional agents, verification influence, and route
  explainability. Phase 8-12 bridge, adapter retirement, hook/idempotency, and
  release work remain deferred.
- Implementation: `baron_core::control_plane::route_task` remains the single
  authority. It consumes normalized task evidence, configured profile, bounded
  survey signals, work shape, current state/continuity markers, capabilities,
  and affected paths when available; fresh no-diff tasks remain routable.
- Canonical assets: `assets/core/skills/database-engineering/SKILL.md` and
  `assets/core/skills/mobile-application-engineering/SKILL.md` materialize once
  under `.baron/core/**`; adapter payloads remain integration-only.
- Focused proof: Phase 7 routing `19/19`, Database profile CLI `2/2`, platform
  intelligence `5/5`, context compiler `19/19`, Core prepare `5/5`, Core
  materialization `9/9`, config `14/14`, promoted Phase 6 targets `3/3`, and
  promoted Phase 7 targets `2/2`. `cargo fmt --all -- --check`, workspace
  Clippy, and `git diff --check` pass.
- Full workspace evidence: no new product regressions. The run reports only
  the two known Windows lifecycle installer failures caused by unavailable
  `Microsoft.PowerShell.Archive`:
  `native_installer_supports_install_update_rollback_and_uninstall` and
  `powershell_installer_makes_baron_available_in_the_current_session`.
- Safe next action: prepare the Phase 8 plan; do not begin Phase 8 in this
  checkpoint. No release or tag was created.
- Proof status: Phase 7 implementation complete; Phase 8 readiness review
  pending only.
- Trace status: no release or tag.

## Codex + Claude Core Consolidation - Phase 7 Started (2026-09-07)

- Scope: profile-aware control-plane routing, distinct Database profile and
  canonical database-engineering skill, plus mobile application engineering
  skill. Phase 8-12 bridge, adapter retirement, hook/idempotency, and release
  work remain deferred.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-07-profile-aware-routing-phase7.md`.
- Last checkpoint: Phase 6 trusted-memory, Task State, and priority-context
  targets are green and remain normal regression tests. Phase 7 Database
  profile, CLI flag, and profile-aware route tests are red against the current
  keyword router, as expected.
- Safe next action: implement the smallest Database config/profile and
  metadata-backed route extensions after the red fixture evidence.
- Proof status: Phase 7 tests added; production behavior unchanged so far.
- Trace status: implementation not complete; no release or tag.

## Codex + Claude Core Consolidation - Phase 6 Completed (2026-09-06)

- Scope: trusted current memory retrieval, priority-aware bounded context, and
  canonical Task State projection only. Profile-aware routing, adapter bridges,
  active-adapter redesign, adapter retirement, release work, and final docs
  remain deferred.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-06-trusted-memory-phase6.md`.
- Last checkpoint: Phase 5 added the bounded `control-plane prepare` façade;
  Phase 6 now routes context-critical memory, resume, grounded handoff, and
  direct recall through one named trusted policy and projects protected Task
  State before tiered context.
- Implementation: current eligibility is centralized across trust state,
  provenance, project scope, approved global scope, and temporal currentness;
  candidate/contested/superseded/expired/stale evidence stays out of trusted
  task context. Tier 0 preserves identity, intent, constraints, non-goals,
  plan, recovery, blockers, route, gates, and next action without a persisted
  task-state file.
- Focused proof: promoted Phase 6 targets `3/3`, dedicated Phase 6 tests
  `4/4`, context compiler `18/18`, Phase 1 current `8/8`, Phase 5 Core prepare
  `4/4`, and Phase 5 CLI prepare `5/5`. Workspace tests have no product
  regressions; two lifecycle installer tests are blocked only by the missing
  `Microsoft.PowerShell.Archive` module. Formatter, workspace Clippy, diff,
  and status JSON checks pass.
- Safe next action: prepare the Phase 7 profile-routing plan; do not begin
  Phase 7 implementation.
- Proof status: Phase 6 implementation complete; no release or tag.

## Codex + Claude Core Consolidation - Phase 5 Checkpoint (2026-09-06)

- Scope: high-level structured `control-plane prepare` only. Memory/context
  trust changes, profile-aware routing, final bridges, active-adapter redesign,
  Generic/retired-adapter cleanup, and final documentation remain deferred.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-06-control-plane-prepare-phase5.md`.
- Last checkpoint: Phase 4 installed one canonical `.baron/core/**` tree and
  separated Core semantics from Codex/Claude integration payloads.
- Implementation: `baron_core::prepare` now validates bounded version-1 JSON,
  composes existing Baron authorities into a deliberate packet, and projects
  canonical Core references. `baron control-plane prepare` transports the
  packet over bounded stdin/stdout with typed JSON errors and deterministic
  exit codes for explicit Codex or Claude adapters.
- Focused proof: Core prepare `4/4`, CLI prepare `5/5`, promoted Phase 1
  structured-input/error targets `2/2`, and Phase 4 Core `9/9`. Required
  formatter, workspace Clippy, diff, and status JSON checks pass. The full
  workspace sweep reports only the two known lifecycle installer failures
  caused by unavailable `Microsoft.PowerShell.Archive` on this Windows host.
- Safe next action: prepare the Phase 6 plan; do not begin Phase 6.
- Proof status: Phase 5 implementation complete; no release or tag.

## Codex + Claude Core Consolidation - Phase 4 Checkpoint (2026-09-06)

- Scope: canonical Core installation and Core/adapter payload separation only.
  Prepare, memory/context, routing, platform profiles, final bridges, Generic
  retirement, and retired-adapter cleanup remain deferred.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-06-codex-claude-core-phase4.md`.
- Last checkpoint: Phase 4 published one canonical embedded Core under
  `.baron/core/**`, separated Core and adapter payloads, reconciled Core-only
  updates without dropping adapter records, and preserved modified/ambiguous
  legacy files.
- Focused proof: Phase 4 Core `9/9`, promoted Core target assertions `3/3`,
  Phase 3 ownership `12/12`, adapter lifecycle `30/30`, update planner `15/15`,
  and the relevant CLI adapter/update suites pass. The full workspace sweep has
  no product regressions; two lifecycle installer tests are blocked only by the
  unavailable `Microsoft.PowerShell.Archive` module on this Windows machine.
- Safe next action: prepare the Phase 5 plan; do not begin Phase 5.
- Proof status: Phase 4 implementation complete; no release or tag.

## Codex + Claude Core Consolidation - Phase 3 Checkpoint (2026-09-06)

- Scope: explicit managed ownership, schema-2 publication, legacy ownership
  migration, ownership classification, and Codex/Claude preservation only.
  Canonical Core installation, thin bridges, prepare, memory/context, routing,
  adapter retirement, and retired-adapter cleanup remain deferred.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-06-codex-claude-core-phase3.md`.
- Last checkpoint: schema-1 legacy manifests now normalize to schema 2 with
  explicit `Core`, `Codex`, `Claude`, or generic legacy ownership. Verified
  unchanged adapter-local skills move to `.baron/core/skills`; modified former
  managed skills block; ambiguous and unregistered files remain untouched.
- Affected files: `crates/baron-adapters/src/update.rs`,
  `crates/baron-adapters/src/install.rs`, adapter ownership fixtures/tests,
  `crates/baron-cli/src/update_transaction.rs`, managed-state planner
  expectations, and Phase 3 status/plan records.
- Proof status: Phase 3 ownership `12/12`, promoted Phase 1 ownership targets
  `6/6`, promoted Phase 2 safety targets `5/5`, adapter lifecycle `30/30`,
  update planner `15/15`, and focused Safe I/O/transaction/migration targets
  pass. The required formatter and warnings-denied Clippy checks pass. The
  workspace test sweep has no source regressions; two lifecycle installer tests
  are blocked only by the unavailable `Microsoft.PowerShell.Archive` module. A
  deterministic publication temp collision restores the v1 state and retry
  succeeds.
- Trace status: no release, tag, or Phase 4 production work created.
- Safe next action: publish the Phase 3 report and stop before Phase 4.

## Codex + Claude Core Consolidation — Phase 2 Checkpoint (2026-09-06)

- Scope: Safe I/O and project-scoped locking only. Core ownership, canonical
  Core installation, bridges, prepare, memory/context, routing, and adapter
  cleanup remain deferred.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-06-codex-claude-core-phase2.md`.
- Implementation: `baron-core::safe_io` now owns explicit tri-state reads,
  preserve-first same-directory replacement, parent validation, platform-aware
  activation, and a bounded reentrant project lock at
  `.baron/.baron-mutation.lock`. Adapter managed/install/update paths, CLI
  update transactions, and migration publication use the shared primitives.
- Focused proof: `phase2_safe_io` `4/4`, adapter `phase2_current` `8/8`, update
  planner `15/15`, migration `8/8`, update transaction unit tests `8/8`, update
  recovery CLI `3/3`, and self-update CLI `1/1`. The five Phase 1 Safe I/O
  target tests are green when run as ignored tests; ownership/Core target tests
  remain intentionally red.
- Required verification: formatter check, workspace Clippy, and diff check
  pass. The full workspace test command reports only the two known lifecycle
  installer failures caused by `Microsoft.PowerShell.Archive` being unavailable
  in this Windows environment.
- Schema versions are unchanged. No Core ownership, bridge, prepare,
  memory/context, routing, Generic cleanup, or retired-adapter removal was
  started.
- Proof status: Phase 2 implementation complete with explicit environment
  limitation recorded. Trace status: no release or tag created.
- Safe next action: prepare the Phase 3 Core ownership plan and stop here.

## Codex + Claude Core Consolidation — Phase 1 Checkpoint (2026-09-06)

- Scope: freeze current behavior, migration inputs, and target architecture as
  executable fixtures. Production source remains unchanged; Phase 2 has not
  started.
- Active design/spec: `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`.
- Active plan: `docs/superpowers/plans/2026-09-06-codex-claude-core-phase1.md`.
- Fixture groups: current Codex/Claude/generic and multi-adapter surfaces;
  managed-state corruption, ownership, path, I/O, preservation, memory,
  continuity, profile, context-budget, and structured-input cases.
- Focused proof: adapter current `21/21`, core current `8/8`, CLI current
  `2/2`; target-red inventory is adapter `14`, core `7`, CLI `7`.
- Relevant regression proof: all adapter and core test targets passed. The CLI
  suite passed except two pre-existing lifecycle installer tests that cannot
  load `Microsoft.PowerShell.Archive` in this Windows environment; this is an
  environment limitation, not a Phase 1 source assertion.
- Proof status: current regression evidence is green; target architecture
  evidence is intentionally red; formatter, workspace Clippy, and status JSON
  checks are green; no hosted CI claim is made.
- Trace status: no product behavior or release state changed.
- Safe next action: publish the final Phase 1 report and stop before Phase 2.

## Baron 4.2.1 Patch Release Checkpoint (2026-08-19)

- The installed immutable `v4.2.0` artifact predates the already-merged
  legacy adapter commits. That is a packaging/version-boundary defect, not
  an intelligence-engine defect: the source checkout has the adapter, while
  the public binary still reports `4.2.0` and cannot expose the retired adapter flags.
- Owner requested `4.2.1`. Active design:
  `docs/superpowers/specs/2026-08-19-baron-4-2-1-patch-release.md`.
- Active plan:
  `docs/superpowers/plans/2026-08-19-baron-4-2-1-patch-release.md`.
- Current phase: Phases 109-112 are complete. Phase 112 published the
  immutable GitHub Release and closed the handoff.
- Proof status: `cargo fmt --all`, workspace check, workspace library tests
  (`40/40`), warnings-denied Clippy, locked release build, release identity/
  metadata tests, focused retired adapter CLI tests (`5/5`), and direct release-binary
  smoke all passed. The local release binary reports `baron 4.2.1`, exposes
  both the retired adapter shortcut and the retired adapter init path, and its local unsigned
  Windows SHA-256 is
  `97975CF1B0B0DDB07B92A7A7C7814D79B830E01577C2541E128A073D61204541`.
- Full local integration sweep caveat: five executable targets could not be
  treated as product failures in this Windows environment. Three were blocked
  by the personal WDAC policy (unsigned test binaries) and two installer tests
  failed because `Microsoft.PowerShell.Archive` could not autoload. Hosted
  native CI subsequently provided the clean four-platform proof.
- Hosted proof: CI run `32224005767` passed format/Clippy and native tests on
  Windows x64, Linux x64, Intel macOS, and Apple Silicon. Release run
  `32224022284` passed exact-source verification, all four native release
  builds, checksums, manifest, installer lifecycle, and immutable promotion.
  `releases/latest` is now `v4.2.1`.
- Public install proof: the downloaded Windows raw candidate reports
  `baron 4.2.1`, exposes the retired adapter shortcut and the retired adapter init path, and
  SHA-256 `21F4C84009E38951959F04EC1FADA20EE964661C261103849F0EEA23AC2CE942`
  matches the public `SHA256SUMS`. The new local helper
  `C:\Users\tytyb\Enable-Baron-4.2.1-retired adapter.ps1` and its narrowly scoped
  supplemental policy target this hash; deployment still requires an
  Administrator terminal.
- Trace status: the patch must preserve the shared-brain invariant. Codex and
  retired adapter use the same project ID, Vault, memory, Wiki, CodeGraph, plan,
  proof, trace, and continuity history; no `4.3` engine change is in scope.
- Windows policy note: the old personal WDAC supplemental exception remains
  hash-bound to the old unsigned `4.2.0` binary. If Windows blocks the staged
  `4.2.1` binary during installation, run the new helper as Administrator
  first; it allows only the public hash above and does not require the binary
  to be installed yet. Do not broaden the policy or disable Device Guard.
- Safe next action: normal `4.2.1` maintenance. Keep Codex and retired adapter on the
  shared project/Vault brain and use the new helper only for the exact public
  binary hash.

## Baron retired adapter Adapter Track Checkpoint (2026-08-19)

- Owner approved an adapter-only retired adapter compatibility track; Baron source
  and public version remain `4.2.0`. No intelligence engine or 4.3 bump is in
  scope.
- Active design:
  `docs/superpowers/specs/2026-08-19-retired adapter-adapter-design.md`.
- Active plan:
  `docs/superpowers/plans/2026-08-19-retired adapter-adapter-program.md`.
- Current phase: Phase 108 complete; Phases 101-108 are checked in the
  plan/status dashboard.
- Proof status: adapter lifecycle 27/27, core all-targets, focused retired adapter
  CLI 3/3, formatter check, workspace Clippy, locked release build, and
  release `baron 4.2.0` smoke passed. The broader CLI suite has two unrelated
  Windows installer failures because `Microsoft.PowerShell.Archive` cannot
  autoload in this environment; the retired adapter tests themselves pass.
- Trace status: switch smoke recorded a shared checkpoint journal entry with
  adapter provenance. Adapter commit `bd6a5a08da5ebda0b743dc542b17d36f6094427a`
  and closure commit `e6312f265b12de54125f606476939a7e0905458d` were pushed
  and verified on both `origin/agent/baron-4-0` and `origin/main`; the worktree
  is clean.
- Shared-brain invariant: Codex and retired adapter must keep the same project ID,
  Vault, memory, Wiki, CodeGraph, plan, proof, trace, and continuity history.
- Safe next action: normal `4.2.0` maintenance. Keep the shared-brain
  retired adapter/Codex adapter path and preserve-first conflict policy; do not create
  a 4.3 tag or Release for this adapter track.
- Phase 108 UX proof: root the retired adapter shortcut and `baron --codex` shortcuts
  resolve the current project, switch the active adapter, preserve the shared
  project/Vault namespace, and keep `baron adapter status/switch` for explicit
  diagnostics. Focused CLI shortcut tests passed `5/5`; CLI Clippy and format
  checks passed; source version remains `4.2.0`.
- Retry condition: any conflict, identity mismatch, or unrelated worktree
  change must be preserved and recorded before continuing.

## Baron 4.2 Final Public Release Checkpoint (2026-08-14)

- Owner approved the thirteen-phase Baron 4.2 program (Phases 88-100) and
  authorized implementation, private local evaluation, testing, GitHub
  publication, README synchronization, and final reinstall/rollback proof.
- Active design:
  `docs/superpowers/specs/2026-08-14-baron-4-2-practical-perfection-design.md`.
- Active plan:
  `docs/superpowers/plans/2026-08-14-baron-4-2-program.md`.
- Current phase: Phase 100 complete. Local and public source/binary are
  `4.2.0`; public `releases/latest` resolves to the immutable `v4.2.0` Release.
- Continuity checkpoint: `notes/build-log/2026-08-14-baron-4-2-program.md`.
- Proof status: Phases 88-100 are checked in the status/plan and accepted by the
  bounded contract. Raw development runs are `100,100,100`; private holdout is
  `100/100` over 8 cases; promotion_ready is `true`.
- Acceptance artifacts: `docs/assessment/baron-4.2-acceptance.{json,md}`,
  contract `432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a`,
  source revision `545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d`.
- Verification status: `cargo fmt --check`, workspace check, warnings-denied
  Clippy, full CLI/core tests, and release build all passed. Trusted gate
  receipts for code-reviewer, security-auditor, and test-engineer passed in the
  disposable gate fixture outside the repository.
- The first Phase 100 native workflow attempt was correctly rejected by the
  public trust-doc contract because `remainingPhaseCount` still described the
  old release-state convention. The status JSON was aligned with the current
  4.2 target before promotion.
- The next native diagnostic exposed two Unix-only update-recovery fixture
  failures: the shell delegate candidate was created without executable mode.
  The fixture now sets mode `0755`, matching a real Unix raw candidate; this is
  a cross-platform test correction, not a gate relaxation.
- The first executable-mode patch kept a moved `PathBuf` behind the Unix-only
  cfg block, so Linux/macOS compilation caught it while Windows could not. The
  fixture now borrows for the write and moves only after permissions are set.
- The final exact-source run passed after the Unix fixture preserved executable
  mode, borrowed the candidate path correctly, avoided rewriting ELF ABI version
  symbols, and scoped the Windows-only patch helper for hosted Clippy.
- Public CI run `31771633229` passed native tests on Windows x64, Linux x64,
  Intel macOS, and Apple Silicon plus format/Clippy. Public release run
  `31771646989` passed exact-source verification, checksums, manifest, installer
  lifecycle, and immutable promotion. Tag/source commit:
  `af42a2d3fcf37f315c6a24c5cebbef59ee6a4bc0`.
- Independent `releases/latest` verification downloaded the manifest and
  checksums, matched the Windows archive/raw candidate hashes, and installed
  `baron 4.2.0` without a PATH change. A separate 4.1 -> 4.2 -> 4.1 smoke
  forced 4.0, restored 4.1, and preserved project/Vault sentinel hashes.
- Safe resume point: normal maintenance. Baron 4.1 remains the whole-engine
  rollback and `BARON_ENGINE_GENERATION=4.0` remains the per-query/legacy
  recovery path.

## Baron 4.1 Release Checkpoint (2026-08-14)

- Owner approved implementation of the eleven-phase Baron 4.1 program after
  removing automatic Skill creation/distillation from scope.
- Owner authorized the Baron-only 4.1 release: Tencent comparison is optional
  reference material and no longer blocks promotion. Baron 4.0 remains the
  explicit fallback.
- Active design:
  `docs/superpowers/specs/2026-08-13-baron-4-1-intelligence-evaluation-design.md`.
- Active plan:
  `docs/superpowers/plans/2026-08-13-baron-4-1-program.md`.
- Last successful checkpoint: Baron 4.1 source `6bea181` is public as tag
  `v4.1.0`; Baron CI #56 and Baron Release #24 passed exact-source,
  cross-platform, checksum, manifest, installer, and immutable-promotion gates.
- Current work: reconcile final public-release evidence in status files, retain
  the 4.0 fallback, and perform the requested disposable cache cleanup.
- Latest proof batch: `cargo check --workspace`, workspace Clippy with
  warnings denied, Baron core library tests, CLI integration tests, and the
  focused Vault/CodeGraph/context suites passed. The full workspace/all-target
  test command passed locally after the semantic candidate filter fix and also
  passed in the hosted Ubuntu release verifier.
- The latest complete `cargo test -p baron-cli --tests --no-fail-fast` run also
  passed all CLI integration groups (23 test binaries/groups, including
  lifecycle, memory, context, CodeGraph, release, and recovery coverage).
- The complete CLI suite and 28 Baron core tests also passed; the hosted native
  matrix covered Windows x64, Linux x64, Intel macOS, and Apple Silicon.
- The isolated seeded development-fixture test passes all five local surfaces
  at `100/100` without mutating a user's Vault; external comparison and
  confidence artifacts are non-blocking diagnostics.
- The frozen-contract Phase 86 runner now repeats the release binary three
  times, verifies the contract/source hashes, records raw failures, and accepts
  the Baron-only local gate without inventing Tencent data. Artifact:
  `docs/assessment/baron-4.1-phase86-runner.*`.
- Implemented candidate surfaces include deterministic lexical/vector/RRF
  retrieval, bounded session-learning candidates with no Skill creation,
  temporal ledger refresh/rollback, grounded handoff citations, semantic Wiki
  and CodeGraph ranking, poisoning quarantine, token/cost measurements, stale
  cache rebuild, and a default `BARON_ENGINE_GENERATION=4.1` path with
  explicit 4.0 and 3.8/baseline fallbacks.
- Latest clean benchmark report: `docs/assessment/baron-4.1-benchmark.json`
  (report `bbc04a89ef0102aa584f2e77710f5ea1029032a77873a578109ed77f7263ee9b`,
  contract `86054c9a45c7d61df91b8b1468ed13347ef96a66091f69a7c404c646dab62af2`).
  The seeded development fixture scored Memory `100/100`, Semantic/grounded
  synthesis `100/100`, Session learning `100/100`, Wiki `100/100`, and
  CodeGraph `100/100` on the release binary. Tencent and external confidence
  are optional and were not used as release gates. Total runtime was `4533 ms`
  (`2189 ms` indexing, `2343 ms` query), estimated 367 handoff tokens,
  `44,155,262` cache/artifact bytes, and `168,239,104` peak working-set bytes;
  the 10,000 ms query and 512 MiB memory budgets passed.
- Proof status: Baron-only Phase 86 and public Phase 87 are complete; a fresh
  public `install.ps1` smoke returned `baron 4.1.0` in an isolated directory.
  Broader real-corpus/scale hardening remains a non-blocking follow-up.
- Safe resume point: use the published `v4.1.0` baseline, retain the explicit
  `BARON_ENGINE_GENERATION=4.0` fallback, and clean only rebuildable/disposable
  caches. Optional Tencent diagnostics are not a release claim.
- Tencent inspection evidence is recorded in
  `docs/assessment/baron-4.1-tencent-v2.0.0-inspection.*`: tag `v2.0.0`
  resolves to commit `0aff21a2d9f2b8a0354aaa80a2e586aab4054562`, but the public
  README only exposes PersonaMem `48% -> 76%`, not the five per-surface scores
  needed for a same-corpus gate.

Date: 2026-08-14
Target: Baron 4.1.0 public release
Program target: Baron 4.1.0

Public release evidence:

- Source/tag: `6bea181044fa0d6f4a74195b8c7455eaa09fdf62` / `v4.1.0`
- Release: https://github.com/thienty1207/Baron-Engine/releases/tag/v4.1.0
- CI: https://github.com/thienty1207/Baron-Engine/actions/runs/31723285579
- Release workflow: https://github.com/thienty1207/Baron-Engine/actions/runs/31723297751

## Final Public Release Checkpoint (2026-08-13)

- Baron `v4.0.0` is public at
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v4.0.0`.
- Release source is exact commit
  `041564d7387e88f5894cb0f9cb3eecb1d328c5b7`, tagged `v4.0.0`; the release
  workflow is
  `https://github.com/thienty1207/Baron-Engine/actions/runs/31680499831`.
  The preceding native CI matrix is
  `https://github.com/thienty1207/Baron-Engine/actions/runs/31679762576`.
- Native CI passed Format/Clippy, full tests, release build, and smoke on
  Windows x64, Linux x64, Intel macOS, and Apple Silicon. The release job then
  passed exact-source verification, archive builds, SHA-256 checksums,
  `release-manifest.json`, installer lifecycle smoke, and immutable promotion.
- Public assets include four native archives and raw binaries, `SHA256SUMS`,
  `release-manifest.json`, `install.ps1`, and `install.sh`. The public
  `releases/latest` endpoint resolves to `v4.0.0`; the downloaded manifest
  reports version `4.0.0` and source `041564d…`, and the public Windows archive
  SHA-256 matches `SHA256SUMS`.
- Fresh Windows public install downloaded `install.ps1` from `releases/latest`,
  verified the archive checksum, installed `baron 4.0.0`, and passed `setup`,
  real `init --codex --fullstack`, `context`, memory index/recall, Wiki
  index/search, and CodeGraph index/query. A reinstall into the same temporary
  directory preserved the project marker and all Vault files (`14` before and
  `14` after). The installed runtime's same-version `baron update --codex`
  attempt refused the redundant `4.0.0` candidate without changing project or
  Vault state.
- Release scope is complete. Real-repository/large-corpus scale, durable
  temporal compaction, full adapter parity, and dynamic lab execution remain
  explicit follow-up limits; no release claim relies on those unimplemented
  paths.
- Final documentation synchronization commit `764baaced3275ac57d6b72a6c85c9e627620de84`
  is pushed to `origin/main`. Its follow-up CI run
  `https://github.com/thienty1207/Baron-Engine/actions/runs/31682195497` passed
  Format/Clippy and all four native runners. Documentation trust tests are 9/9,
  status JSON parses, and the working tree is clean.
- Next action: normal maintenance from the published `v4.0.0` baseline.

## Baron 4.0 Release Checkpoint

- Owner approval: explicit approval to implement Phases 65-76, run the full
  benchmark/test/certification program, publish `v4.0.0` to GitHub, and update
  the README with the newest install path.
- Baseline: Baron `3.8.0` remains the explicit recovery fallback; public stable
  and source release are now `4.0.0`.
- Release source `041564d7387e88f5894cb0f9cb3eecb1d328c5b7` is pushed to
  `origin/main`; the working branch retains the same source while final docs
  synchronization is prepared.
- Active design: `docs/superpowers/specs/2026-08-13-baron-4-0-intelligence-security-design.md`.
- Active plan: `docs/superpowers/plans/2026-08-13-baron-4-0-program.md`.
- Phase 65 starts with a frozen score contract and baseline; no 4.0 result may
  replace the 3.8 path without per-query comparison and hard-gate evidence.
- Current proof: `cargo fmt --all -- --check`, workspace check, workspace
  Clippy with warnings denied, full workspace/all-target tests (exit 0), all
  Baron core/CLI focused tests, guarded context smoke, the independent
  sixteen-case A/B benchmark (all four surfaces 100/100, zero leakage), the
  nine-case security routing regression (100/100), the bounded static scan
  (100/100, 0 findings), and integrated acceptance (100/100) pass. Reports are in
  `docs/assessment/baron-4.0-benchmark.*` and
  `docs/assessment/baron-4.0-security-regression.*` and
  `docs/assessment/baron-4.0-certification.*`. The expanded clean benchmark
  report id is `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`
  with source fingerprint
  `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`.
- Candidate behavior: released 4.0 memory/Wiki/CodeGraph paths are default;
  `BARON_ENGINE_GENERATION=3.8` or `baseline` forces the active 3.8 recovery
  path, and unknown values fail closed.
  Wiki links/citations, CodeGraph imports/spans/calls/references, and security
  fail-closed/allowlist checks are implemented as local bounded candidates.
- Phase 67 now has a read-only `baron memory consolidate` analyzer that stages
  duplicate/conflict/supersession candidates without writing or promoting any
  Vault record; durable compaction, temporal authority, and rollback are still
  open.
- All public release gates are now closed: exact-source push, native matrix,
  immutable tag/Release, checksums/manifest, and fresh public Windows recovery
  install passed. Broader real-repository/large-scale corpus and durable
  temporal compaction remain explicit follow-up limits and are not silently
  claimed as completed.
- Safe next action: commit/push the final truthful README, status, and build-log
  synchronization, run the documentation trust tests, and leave the tree clean.

## Pre-Publication Evidence (All Gates Passed)

- Source version is `4.0.0`; `cargo fmt --all -- --check`, `cargo check
  --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo build --release --locked -p baron-cli` pass. The optimized binary
  reports `baron 4.0.0`.
- `cargo test --workspace --all-targets --no-fail-fast` passes after the README,
  status, and fingerprint-boundary updates. The final `public_trust_docs` suite
  is 9/9, and the full workspace run exits 0.
- Final clean acceptance report: `docs/assessment/baron-4.0-certification.*`,
  score `100/100`, benchmark id
  `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`, source
  fingerprint
  `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`.
  Memory/Wiki/CodeGraph/Security each score 100/100; leakage is 0; security
  routing is 9/9; static scan is 100/100 with 0 findings; bounded handoff is
  902 characters and project-grounded.
- CodeGraph source fingerprints now exclude generated assessment/status/build
  metadata and release README prose, so evidence references cannot invalidate
  their own code identity; Wiki remains the documentation index/source path.
- Optimized binary smoke passes setup, Codex fullstack init, default 4.0
  context/Resume Brief, Wiki index/search, CodeGraph index/query, and explicit
  `BARON_ENGINE_GENERATION=3.8` recovery context.
- Public release gate complete; only final documentation synchronization remains
  in this build note. No source or release artifact change is required.

## Current Phase

Baron 4.0 Phase 76 exact-source publication and recovery install (`completed`).

## Baron 3.8 Implementation Checkpoint

- Owner approval: explicit approval to implement all planned Baron 3.8 phases,
  update README, publish the newest Windows-downloadable release, and mark each
  completed phase/task in `docs/BARON_STATUS.md`.
- Phase 53-56 implementation evidence now exists in the `knowledge` module:
  bounded Resume Brief, deterministic local hybrid recall, layered memory
  labels, redaction, benchmark output, and Continuity/context integration.
- Phase 57-59 implementation evidence now exists for project-bound incremental
  Wiki citations and a disposable local CodeGraph fallback with symbol/edge
  queries.
- Phase 60-63 implementation evidence now includes project identity checks,
  secret redaction, local cache boundaries, three Baron-owned optional reverse
  skills, narrow reverse routing, and adapter asset lifecycle integration.
- Current proof: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, knowledge tests, control-plane tests, adapter
  lifecycle tests, locked release build, exact `baron 3.8.0`, status JSON parse,
  and new CLI Wiki/CodeGraph/Resume smoke all pass. GitHub Actions run
  `31603251123` also passed the full workspace suite, Clippy, all four native
  targets, release metadata/checksums, installer lifecycle, and immutable
  `v3.8.0` promotion. Fresh `releases/latest` Windows setup/init/context smoke
  returned `baron 3.8.0`.
- Follow-up CI run `31604514726` exposed an OS-dependent assertion in the
  bounded preservation-preview test: macOS directory enumeration can stop
  before one particular unlisted file while still satisfying the contract.
  The test now checks only its actual guarantees (bounded diagnostic and
  dependency-artifact exclusion); the preceding planner test retains the
  explicit user-file preservation assertion. Local `update_planner` coverage
  is 15/15 after this contract-alignment fix, with no release artifact change.
- GitHub Actions CI run `31605317869` (`#48`) then passed format/clippy and
  all four native targets, including the formerly failing Apple Silicon job;
  `main` is green at commit `01c1051991ea772da59bae4a622d98cdeff90dad`.
- Final status-audit commit `97015aef29f60bb2a7a5c170130ece82913944cf`
  leaves every checklist item in `BARON_STATUS.md` checked (658 checked,
  0 unchecked). GitHub Actions CI run `31607104564` (`#50`) passed format,
  Clippy, Windows, Linux, macOS Intel, and Apple Silicon for that exact
  revision.
- Safe next action: normal maintenance from the published `v3.8.0` baseline;
  re-run the native release workflow only for a future version bump.

## Final Baron 3.7 Public Release Checkpoint

- Baron 3.7 is complete and public. Source revision
  `cc14c222130ac2047d36b3b752d9140521d3538e` is pushed to `origin/main` and
  was accepted by GitHub Actions release run `31582187832`.
- Exact-source verification, Ubuntu full workspace tests, Clippy, Windows
  x64, Linux x64, macOS Intel, Apple Silicon, checksum/manifest generation,
  installer lifecycle smoke, annotated tag, and immutable Release promotion
  all passed.
- Public Release: `v3.7.0` at
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v3.7.0`. Assets
  include all four native targets, raw update candidates, `SHA256SUMS`,
  `release-manifest.json`, `install.ps1`, and `install.sh`.
- Fresh Windows public smoke downloaded `install.ps1` from
  `releases/latest`, installed `baron 3.7.0`, passed `setup`,
  `init --codex --fullstack`, and `context`; the same-version update guard
  rejected a redundant update without changing the user-owned marker.
- README, `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, the certification
  report, the executable 3.7 plan, and the design completion record were
  synchronized in final documentation commit `2e637495d3d8c639e5920633ab21e9e4c3b7e9ee`.
- A follow-up README-only commit `1fb041768c7a8937dbe6632f2d66df85e9f947b5`
  makes the Windows installer command work under the host's default execution
  policy by invoking the downloaded script with `-ExecutionPolicy Bypass`.
- Proof status is complete; `origin/main` equals `1fb0417` and the local
  working tree is clean and synchronized.

## Status Checklist Repair

- GitHub review caught that the Phase 46-51 detailed checklists in
  `docs/BARON_STATUS.md` still displayed `[ ]` even though the phase summary,
  status JSON, tests, and release evidence were complete.
- Every actual completion item and exit-gate item under Phases 46-51 is now
  checked `[x]`. Phase 52's remaining unchecked bullets are explicitly labeled
  permanent hard-stop policy rules, not unfinished work.
- The Phase 51 status text now describes the certified 3.7 release source;
  historical 3.6 baseline references remain only where they document the
  pre-bump certification boundary.

## Checkpoint After Phase 46-50 And Phase 48 Hardening

- Phase 46 now has a write-free work-shape decision and hidden CLI surface;
  Phase 47 has source/project-bound receipts from the Baron-owned runner;
  Phase 48 now requires current receipts for medium/high plan proof and all
  three quality gates; Phase 49 has approval plus comparable-rerun experiment
  records; Phase 50 routes a bounded project runbook only to runtime tasks.
- Focused proof passed: core receipt (including large-output/no-deadlock,
  source-change, and tamper invalidation), trusted proof, high-risk plan with
  receipt-backed gates, proof/trace, control plane, operations, harness
  experiments, and CLI command suites.
- A first large-output test exposed that the bounded reader stopped draining a
  pipe; the runner now drains concurrently and retains only bounded excerpts.
  A second issue exposed Baron-managed Markdown changing source fingerprints;
  `docs/baron`, `.baron`, and adapter-managed runtime trees are excluded from
  the product source fingerprint while project source contents are hashed.
- Proof status: focused implementation evidence is passing; integrated Phase
  51 certification is not yet complete. Source/version remains `3.6.0`.
- Trace status: this implementation batch has no public release trace yet;
  release trace must include exact source SHA, native matrix, assets, and
  fresh public installer smoke.
- Safe next action: run the full workspace suite and Clippy, then close any
  certification gaps before changing release metadata.

## Phase 51 Certification Checkpoint Before Version Bump

- Phase 51 local certification is complete on the source version `3.6.0`.
- Passing evidence: `cargo fmt --all -- --check`; all `baron-core` and
  `baron-cli` targets with single-threaded integration execution; Graphify
  local fixture with a bounded 3-second Windows startup budget; Clippy with
  `-D warnings`; locked release build; release binary `baron 3.6.0`; release
  profile certification; adapter/preservation/scale/migration/interruption/
  redaction/public-doc regression gates.
- The combined workspace command was attempted and timed out on this Windows
  host after emitting passing results; the independent package all-target
  suites completed without assertion failures. The timeout is recorded as a
  test-runner scheduling limitation, not a release pass.
- Proof status: Phases 46-51 are certified locally; public 3.7 proof does not
  exist yet. Trace status: release trace is pending exact candidate SHA.
- Safe next action: bump workspace/package/lock/docs metadata to `3.7.0`, keep
  README claims aligned with the public release only after promotion, then run
  the release manifest and native GitHub workflow gates.

## Phase 52 Candidate Checkpoint After Version Bump

- Workspace/package/lockfile, certification target, dynamic version assertions,
  release guide, and source-version README line now agree on `3.7.0`.
- The README deliberately still identifies public `releases/latest` as
  `v3.6.0` until the immutable 3.7 Release exists; it contains the candidate
  version and the exact post-promotion check without making a false download
  claim.
- Candidate proof passed: formatting check, Clippy, focused core/CLI release
  tests, locked optimized build, and release binary `baron 3.7.0`.
- Public proof status: no 3.7 GitHub tag, Release, native matrix, assets, or
  latest-install smoke exists yet. Trace status is pending the exact pushed
  candidate SHA and workflow run.
- Safe next action: inspect and commit this candidate, push the exact source to
  `origin/main`, dispatch the immutable release workflow, and wait for all
  native/public checks before changing the README public-release block.

## Phase 52 Workflow Repair Checkpoint

- Release workflow `31581443056` verified the exact candidate SHA, formatting,
  and the beginning of the full workspace suite, then failed only in the
  Linux self-update fixture because its hard-coded `3.6.1` candidate is older
  than the now-running `3.7.0` binary.
- The fixture now derives its candidate from `CARGO_PKG_VERSION` and increments
  the patch number, preserving the intended staged-candidate assertion without
  weakening the production older-version rejection.
- Local proof passed after the repair: formatting check, the Linux-sensitive
  self-update integration test, and Baron CLI Clippy with `-D warnings`.
- Current proof status: release workflow must be re-dispatched from the repair
  commit; no tag or public 3.7 Release is claimed until that run passes.
- Safe next action: commit this focused test repair, push `origin/main`,
  dispatch with the exact new SHA, and inspect all native/public release jobs.

## Phase 52 Clippy Repair Checkpoint

- Release workflow `31581885211` passed exact-source verification, formatting,
  and the full Ubuntu workspace test, then exposed four Unix-only unused imports
  in Windows-gated receipt tests during workspace Clippy.
- Those imports are now gated with `cfg(windows)`, preserving the Windows test
  coverage while making the all-target lint clean on Ubuntu.
- Local proof passed after the repair: `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings`.
- Current proof status: no 3.7 tag or public Release exists; the workflow must
  be re-dispatched from this lint-clean commit.
- Safe next action: commit, push, dispatch the exact SHA, and monitor the full
  native matrix plus immutable publication.

## Continuity Checkpoint Before Source Edits

- User approval: explicit approval to implement and publish the full Baron 3.7
  program, including README, GitHub push, tag, Release, and public installer
  smoke.
- Last successful step: baseline audit confirmed `v3.6.0` is the current public
  release; only the pre-existing Baron 3.7 plan edit was dirty.
- Proof status: no Baron 3.7 implementation proof yet; source remains `3.6.0`.
- Trace status: no active 3.7 execution trace yet.
- Affected files so far: new 3.7 design/plan and this build note, plus the
  approved 3.7 status plan in `docs/BARON_STATUS.md`.
- Safe next action: implement Phase 46 work-shape and Phase 47 trusted receipt
  primitives, keeping all release metadata at `3.6.0` until certification.
- Retry condition: if any batch fails, preserve the failing test/output and
  record the exact source files and next safe repair in this file before
  continuing.

## Final Release Checkpoint

- The user explicitly authorized publishing the immutable `v3.6.0` GitHub
  Release after asking for a reinstall-ready README.
- README now provides a one-block Windows installer, a strict `baron 3.6.0`
  version check, Vault restore guidance, and project refresh guidance.
- `docs/BARON_STATUS.md` has a visible final-release checklist. It is checked
  only after GitHub native builds, immutable tag/assets, and `releases/latest`
  smoke have completed.
- Fresh pre-publish verification passed: `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo build --release --locked -p baron-cli`, YAML lint for
  `.github/workflows/release.yml`, and a release-profile Vault/project
  certification smoke using `baron 3.6.0`.
- The initial immutable promotion run `30243740873` stopped in its Ubuntu
  verification job before any native build, tag, or GitHub Release was made.
  The error was a missing `CandidateBinaryInspector` trait import in an
  Unix-only runtime-activation branch. Windows local verification did not
  compile that branch, which is why the failure first appeared in GitHub.
- A RED/GREEN regression test now covers exact activated-runtime version
  matching. The repair moves that check into one shared helper so both the
  trait contract and the mismatch rejection are compiled on every platform.
- Fresh repair evidence: the exact-version regression, full workspace suite,
  Clippy, and `cargo build --release --locked -p baron-cli` passed. A local
  `x86_64-unknown-linux-gnu` cross-check is blocked only because this Windows
  machine lacks `x86_64-linux-gnu-gcc` for `ring` and bundled SQLite; it is not
  accepted as Linux proof. GitHub Ubuntu must compile the repaired exact source
  before Baron can create a tag or release.
- The repaired release binary completed a fresh `setup --vault`,
  `init --codex --fullstack`, and `certify run --profile release` smoke. It
  produced the expected Vault, Codex adapter, project configuration, and all
  release certification checks passed.
- The second immutable promotion run `30244634765` also stopped before any
  tag, native artifact, or GitHub Release. Its Linux-only self-update fixture
  copied a debug binary larger than the real 128 MB candidate ceiling, so the
  fixture failed before its intended version-mismatch assertion. The fix uses
  a tiny executable Unix wrapper that delegates to the running test binary;
  production candidate limits remain enforced. The fresh full suite, Clippy,
  locked release build, and workflow YAML check passed after this fixture fix.
- An incorrect-SHA dispatch `30245161318` was cancelled before any build or
  promotion after its mismatch was detected. The next dispatch always derives
  `source_revision` directly from `origin/main`, not from copied text.
- The third immutable promotion run `30245233369` also stopped before any tag,
  native artifact, or GitHub Release. The remaining Unix update-transaction
  fixture staged an oversized patched debug binary. It now stages a bounded
  wrapper that delegates to the patched backing binary, preserving the expected
  candidate protocol and the real production size ceiling. The complete local
  test suite, Clippy, locked release build, and workflow YAML gate passed after
  the complete fixture audit.
- The fourth immutable promotion run `30245849879` passed its complete Ubuntu
  test suite, then stopped before tag creation during Clippy. The error set was
  limited to an empty Unix-only placeholder test plus two Windows-only runtime
  owners compiled on Linux. The placeholder test is removed and the runtime
  state path/helper are now explicitly Windows-scoped; the release workflow and
  every production safety ceiling remain unchanged.
- Fresh fourth-repair local evidence passed: `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast --quiet`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo build --release --locked -p baron-cli`, workflow YAML lint, status JSON
  parsing, and a release binary `setup --vault`, `init --codex --fullstack`,
  `certify run --profile release`, and `--version` smoke. The smoke returned
  `baron 3.6.0` and passed every release certification check.
- Repaired source commit `36413199d39c547664d1a2500e8a4444219d858e` is now
  pushed to `origin/main`. The next immutable promotion derives its exact source
  SHA from the current remote branch instead of using copied text.
- Final public proof: GitHub Actions run `30246729740` passed exact-source
  verification, Windows, Linux, macOS Intel, macOS Apple Silicon, and immutable
  promotion. Tag `v3.6.0` resolves to
  `c89486694d9a4431e04106274d0c9f997db42683`; the release contains native
  archives, `SHA256SUMS`, `release-manifest.json`, `install.ps1`, and
  `install.sh`.
- Public Windows smoke: downloaded `install.ps1` from `releases/latest` into a
  new temporary directory, installed with `-Version 3.6.0 -NoPathUpdate`, then
  verified `baron 3.6.0`, `setup --vault`, `init --codex --fullstack`, and
  `context` in a fresh project.
- No release action remains. For a Windows reinstall, follow the short README
  flow: restore the Vault and project folders, install from `releases/latest`,
  run `baron setup --vault`, then run `baron update` in each project.

## Verified Checkpoint Before Phase 35

The bundled Superpowers workflow core has been refreshed to the pinned upstream
`v6.2.0` skill tree without changing Baron's core ownership:

- exact upstream commit:
  `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`
- upstream subtree: 50 files with no missing or extra file; one recorded Baron
  patch removes the visual companion's remote branding/telemetry request
- Baron-owned root routing and adapter integration remain local
- Codex, Claude, and generic adapter contract test passed
- provenance, MIT license, and offline runtime ownership are recorded
- Unix installs preserve executable mode for embedded shebang scripts
- a short Baron-owned entry contract protects the SDD review breaker and
  behavior-test rules from long-context summarization drift
- extension boundaries for code graphs and optional skill sources are accepted
  in `docs/decisions/0002-extension-ownership-and-code-graph.md`

This checkpoint starts Phase 35 only. It does not bump the Baron version, add
Graphify, or change the 3.4 phase order.

## Blueprint Audit

The two apparent core trees are not equal maintained sources:

- `assets/core/` has 145 runtime asset files and is embedded by
  `baron-adapters`.
- `blueprints/core/` has 7 stale historical files.
- Six paths overlap and all six have different content.
- No Rust crate, installer, manifest, test, or workflow reads
  `blueprints/core/`.

Phase 35 completed the source-of-truth contract, deletion of
`blueprints/core/`, adapter parity proof, managed baseline, conservative
three-way planner, malformed-marker refusal, and no-write CLI preview. The
RED test failed on the stale directory; the same focused test passed after
removal. A fresh install now records only repository-relative Baron-owned
content under `.baron/managed-state/`; custom assets and project state remain
outside its write set.

## What Is Being Built

- Phase 35 has removed stale blueprints, proved `assets/core/` is the only
  runtime source, and recorded the managed baseline plus safe read-only
  three-way preview.
- Phase 36 resolved and verified the exact native release candidate before project activation.
- Phase 37 applies project/runtime changes transactionally, stages conflicts, and recovers interrupted updates; its focused transaction/recovery suites now pass.
- Phase 38 separates human update authority from AI local repair and certifies Baron 3.4; the full certification gate has passed and `3.4.0` is the current stable source baseline.
- Phases 39-41 produced certified Baron 3.5 by distilling selected frontend,
  deep-module, and domain-language techniques into existing Baron owners, with
  no second workflow or frontend skill. Reconcile now restores missing managed
  Domain Language documents without overwriting user content.
- Phases 42-45 produced Baron 3.6 with an optional project-scoped Graphify
  code-only provider, bounded context, source verification, strict isolation,
  and Survey fallback. The source baseline is now `3.6.0`.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`.
3. Read
   `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`.
4. Execute
   `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`.
5. Phase 39 completed: local frontend guidance now has a brief fingerprint,
   anti-template gates, responsive/state proof, three-adapter installation
   proof, narrow routing proof, and no operational live dependency.
6. Phase 40 completed: local deep-module guidance and project-scoped domain
   language now preserve user content, avoid generic memory leakage, and
   withhold divergent repo/Vault terms from trusted context.
7. Phase 41 completed: local assets, narrow routing, adapter parity, user
   preservation, public flow, and the Baron 3.5 source gate passed.
8. Phase 42 completed: provider-neutral graph state, source fingerprint,
   project identity, cache path, junction, query-bound, and optional-capability
   contracts passed without provider invocation.
9. Phase 43 completed: the deterministic fake Graphify adapter accepts only
   `0.9.25`, permits only local probe/code-only extraction/bounded query calls,
   disables provider query logging, uses bounded subprocess files and atomic
   cache promotion, and keeps the last good graph on every tested failure.
10. Phase 44 completed: task-matched context never invokes Graphify, caches at
   most eight local hints, and routes AI to hidden refresh/query only when
   needed. Every graph result is checked against current repo source; inferred
   results remain advisory.
11. Phase 45 completed: same-name graph isolation, Vault exclusion, stale and
   corrupt fallback, 6,100+ source fingerprint coverage, and hook/instruction
   preservation passed.
12. Baron 3.6 source/docs/lock/certification are synchronized. Read
   `docs/assessment/baron-3.6.0-code-graph-certification.md` before any later
   code-map change.
13. `v3.6.0` is publicly released after immutable GitHub proof. No active
   implementation or release action remains.

## Verified Baseline

- Isolated branch: `codex/baron-3-4-to-3-6`.
- Planning baseline commit: `b6e619b`.
- Baseline `cargo test --workspace --all-targets`: passed on 2026-07-23.
- Source and stable-source version are now `3.6.0` after Phase 45's full gate.
- Baron 3.4 and 3.5 have source certification; no GitHub Release/tag was made.
- Baron 3.6 Phases 42-45 have complete source/test evidence.
- Baron `v3.6.0` is the latest downloadable release, built and promoted by
  GitHub Actions run `30246729740`.
- The program has 0 planned phases remaining.
- Superpowers `v6.2.0` adapter contract: passed on 2026-07-24.
- Full post-refresh workspace tests, Clippy, release build, adapter smoke, SDD
  semantic smoke, and visual-server behavior tests: passed on 2026-07-24.

## Non-Negotiables

- Superpowers remains workflow core.
- Core agents remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains source of truth.
- Normal users keep the simple install/setup/init/update flow.
- AI never silently downloads or activates a release.
- Conflicting managed edits never overwrite live project content.
- No release tag or GitHub Release exists before its source and artifacts pass proof.
- `assets/core/` is the only runtime source for bundled skills and agents.
- `blueprints/core/` must be removed before any managed baseline is trusted.
- Optional providers and skills cannot become a second workflow, memory, or
  instruction owner.
- Baron 3.5 may deepen existing owners but must not create Hallmark or Matt
  workflow skills.
- Baron 3.6 may use Graphify only as a local optional code map; it cannot own
  hooks, instructions, Vault memory, global context, or workflow state.

## Multi-Agent Core Parity Checkpoint (2026-08-20)

- User approval: explicit instruction to implement the original Baron contract:
  one shared core for Codex and Claude; unsupported historical values remain
  opaque migration input.
- Finding: `assets/core/` is embedded as the source of truth, but
  `managed_payloads_for_adapter` and `install_retired adapter` currently install only
  retired adapter bridge/command/settings files; Codex receives the complete embedded
  skills and agents. The existing test name
  `retired adapter_adapter_installs_shared_brain_assets_without_engine_assets` records
  the incomplete behavior.
- Proof status: baseline source is clean before this track; no parity
  implementation proof exists yet.
- Trace status: no completion trace for this track exists yet.
- Affected files planned: `crates/baron-adapters/src/install.rs`, adapter
  lifecycle tests, adapter architecture/README, status Markdown/JSON, and the
  new parity design/plan.
- Safe next action: implement Phase 113-115 with the existing managed-baseline
  and preservation boundaries, then add RED/GREEN parity tests in Phase 116.
- Retry condition: if a write or test fails, preserve the exact failing output,
  affected paths, and next safe repair here before continuing.

## Multi-Agent Core Parity Completion Checkpoint (2026-08-20)

- Phase status: Phases 113-117 completed and every task is marked `[x]` in the
  active plan and `docs/BARON_STATUS.md`.
- Implementation: the historical adapter payload once received the same
  embedded `assets/core` skill/agent inventory as Codex. Its legacy projection
  included an index, skill/agent indexes, the complete skill tree, and the
  complete agent tree while retaining its native commands/settings/hooks bridge.
- Preservation: initial retired adapter materialization writes only missing or byte-
  identical core assets. Changed/unmarked core files are preserved and listed
  as conflicts; routing files keep custom text through Baron routing markers.
- Proof status: the full adapter target set passed (3 unit, 30 lifecycle, 15
  planner, and 1 transaction test), retired adapter CLI 6/6 including the full
  Codex -> retired adapter -> Claude -> Generic -> Codex round trip, public docs 9/9,
  Clippy warnings-denied, locked release build, release `baron 4.2.1` smoke,
  and an isolated Codex -> retired adapter core materialization smoke all passed.
- Full workspace note: all relevant engine, memory, adapter, CLI, fallback,
  and release tests passed. Four pre-existing Windows environment gates could
  not execute: two PowerShell archive tests lack `Microsoft.PowerShell.Archive`,
  one update-recovery candidate executable was blocked by WDAC, and one
  `work_shape` test executable was blocked by WDAC. No parity-related test
  failed; these gates remain explicitly environment-only.
- Trace status: implementation trace is complete for the local maintenance
  track; no GitHub publication or version bump was authorized or performed.
- Safe next action: normal `4.2.1` maintenance. Future releases must preserve
  the shared core contract and rerun the parity/preservation suite.

## Multi-Agent Core Parity Handoff Checkpoint (2026-08-20)

- Current task: hand off the completed shared-core correction for Codex,
  Claude, retired adapter, and generic adapters.
- Last checkpoint: the final CLI regression now covers the complete
  Codex -> retired adapter -> Claude -> Generic -> Codex round trip; the generic CLI
  spelling `agent` correctly persists as the internal `generic` adapter kind.
- Proof status: latest `cargo fmt --all -- --check`, public-trust docs 9/9,
  warnings-denied Clippy, and legacy adapter CLI 6/6 passed. The isolated
  release smoke remains `baron 4.2.1` with the complete retired adapter core view.
- Trace status: complete for this local maintenance track. No source release,
  global install, GitHub push, or tag was performed.
- Resume point: future work starts in normal `4.2.1` maintenance; rerun the
  parity/preservation suite before changing any adapter payload boundary.

## Baron 4.2.2 Release Checkpoint (2026-08-20)

- User approval: publish the completed multi-agent core parity correction as
  Baron `4.2.2` on GitHub.
- Current task: bump release identity and current public metadata, run the
  release gates, then push `origin/main` and tag `v4.2.2`.
- Last successful step: retired adapter parity implementation and the complete
  four-adapter round-trip test passed on the `4.2.2` source line.
- Proof status: Phase 118 and 119 local proof passed: adapter target set,
  retired adapter CLI 6/6, public docs 9/9, format, Clippy, locked release build,
  release metadata fixture, and isolated `baron 4.2.2` parity smoke. The full
  workspace sweep passed all engine, memory, adapter, CLI, fallback, and
  release tests; two installer lifecycle tests remain blocked only because
  this Windows PowerShell cannot load `Microsoft.PowerShell.Archive`.
- Trace status: release trace pending; no source, project, or Vault data may
  be rewritten outside the intended release files.
- Safe next action: inspect the complete release diff, commit only the intended
  `4.2.2` files, push `origin/main`, and publish/tag `v4.2.2` only after the
  staged scope is confirmed.

## Baron 4.2.2 Public Release Completion Checkpoint (2026-08-20)

- Current task: close the authorized Baron 4.2.2 publication and leave a
  reproducible install target for a future Windows reinstall.
- Release source: commit
  `80f4daa13e15b2aafb53bbefe0f54454130fcaba` is pushed to both
  `origin/main` and `origin/agent/baron-4-0`; annotated tag `v4.2.2` points to
  the same source.
- Remote proof: Baron CI run
  `32347122499` passed format/Clippy and all four native test jobs. Baron
  Release run `32347138548` passed exact-source verification, the Windows,
  Linux, Intel macOS, and Apple Silicon matrix, release metadata/checksums,
  installer lifecycle, and immutable promotion.
- Public proof: Release
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.2` is not a
  draft or prerelease, contains 12 assets, and `releases/latest` resolves to
  `v4.2.2`. The Windows raw candidate SHA-256 is
  `D26B0B7DF1708DC1669E7DAD215002A37929755E804AFE766B818B11A36E5A49`; the
  Windows archive SHA-256 is
  `B298E7007CC060C5AAD1B0B11AB4C618C461ABAC8473B81A5A1EE38029D55A4B`.
- Version proof: hosted release assets and the local locked release binary
  report `baron 4.2.2`. README, status Markdown/JSON, active plan, and this
  build log now agree; Phases 118, 119, and 120 are all complete and checked.
- Trace status: complete for this release. No project or Vault data was
  rewritten. The next action is normal Baron 4.2.2 maintenance.
