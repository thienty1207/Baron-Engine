# Baron 3.7 Quality And Trust Design

Date: 2026-08-12
Status: approved by the owner; implementation in progress
Current source baseline: `3.6.0`
Target public release: `3.7.0`

## Goal

Baron 3.7 strengthens trust in the existing engine without adding another
workflow core, another mandatory agent, or a new memory source. The release
makes lifecycle depth proportional to work, makes execution proof observable
and source-bound, makes Harness improvements measurable, and adds a
project-owned application runbook contract. The release is not complete until
the README, GitHub Release, `releases/latest`, and a fresh public install all
agree on `3.7.0`.

## Ownership Boundaries

- Superpowers remains the only workflow core.
- The mandatory gates remain `code-reviewer`, `security-auditor`, and
  `test-engineer`.
- Vault Markdown remains durable truth. SQLite, receipt indexes, and caches are
  rebuildable accelerators.
- Project ID remains the isolation boundary for receipts, gates, experiments,
  runbooks, session replay, and memory.
- Baron-owned execution runners are the only trusted receipt producers.
- Product repositories own application commands, readiness, state, interfaces,
  runtime evidence, and cleanup ownership.
- Autopilot proposes candidates only; human approval is required before an
  intervention changes guidance, policy, architecture, tools, or assets.
- Existing adapters translate the same core decisions; they do not fork policy.

## Release Shape

### Phase 46: Adaptive Work Shape

Introduce a read-only/change/ambiguous authority boundary and a work-shape
decision that considers mutation authority, durability, judgment, risk, proof,
and lifecycle depth independently. A small reversible one-session change may
use focused ephemeral state; risky, ambiguous, coordinated, or multi-session
work keeps intent, plan, recovery, proof, trace, and mandatory gates.

### Phase 47: Trusted Execution Receipts

Add a versioned receipt model containing project/source identity, task identity,
registered backend, executable and argv, cwd, timestamps, exit/result, bounded
output digests, and artifact digests. Receipts are emitted atomically by a
Baron-owned runner or registered verified backend. Handwritten, imported, or
agent-authored receipt-shaped data is reported evidence only. Vault Markdown is
the durable human-readable record; machine metadata is rebuildable and
integrity-checked.

### Phase 48: Gate And Completion Integrity

Bind quality-gate runs, Proof, reviewer closure, Trace, capability execution,
and Stop reconciliation to fresh trusted receipts and the current source
identity. Reported words, stale Markdown, failed/skipped/degraded checks, or
old source evidence cannot produce a false completion.

### Phase 49: Measured Harness Experiments

Keep Autopilot candidate-only. An approved intervention records a representative
baseline and a falsifiable hypothesis, then requires a comparable fresh-agent
rerun that records availability, retrieval, invocation, relevance, outcome,
proof, steering, retries, context cost, and maintenance cost before it is kept,
revised, removed, or left pending.

### Phase 50: Application Runbook

Add an optional project-owned runbook under `docs/baron/operations/`. Baron may
route it only for operate, reproduce, runtime-debug, end-to-end, and deployment
smoke tasks. It must distinguish observed facts from defaults, likely values,
stale values, and unknowns. It must not invent credentials, ports, readiness,
fixtures, or cleanup ownership, and must prove the real interface plus
correlated runtime evidence.

### Phase 51: Integrated Certification

Distill selected evidence-first lessons from the reviewed upstream harness
release into existing Baron owners: composition-root/artifact verification into tests,
honest automation failure into Proof, boundary validation into security, adapter
parity into review, and bounded cumulative state into performance. No live
dependency, fourth core agent, duplicate workflow, or universal engineering
wisdom runtime is introduced. Migrations preserve 3.6 projects and all user
assets, source, Vault, and history.

### Phase 52: Public Release

After Phases 46-51 pass, bump all version metadata to `3.7.0`, update README and
release docs, run the complete local certification, publish the exact verified
source through the immutable GitHub workflow, verify native assets/checksums,
then install from public `releases/latest` in a fresh Windows smoke. If any
publication step fails, the default branch must be restored to a truthful
install message before stopping. Only after public proof succeeds may the
stable status become `3.7.0`.

## Compatibility And Migration

- Existing free-form Proof, gate, experiment, and runbook text remains readable
  but is labelled reported/legacy evidence and cannot silently become trusted
  execution proof.
- Existing 3.6 projects receive additive metadata and safe fallback behavior;
  no project or Vault data is deleted.
- Missing receipts, providers, or runbooks degrade with explicit diagnostics;
  required missing proof blocks completion.
- Codex, Claude, and generic adapters receive equivalent routing and lifecycle
  behavior with bounded task context.

## Verification Contract

Every phase adds focused tests, adapter parity checks, and status/build-log
evidence. Release certification requires formatting, workspace tests, Clippy,
locked release build, manifest/version checks, installer lifecycle checks, and
fresh public `releases/latest` install/setup/init/context/update smoke.
