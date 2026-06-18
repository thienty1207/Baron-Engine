---
name: deprecation-and-migration
description: Use when migrating APIs, frameworks, schemas, data models, configs, feature flags, legacy behavior, compatibility layers, rollouts, rollback paths, dual-read, dual-write, or data migration work.
license: MIT-compatible Baron-owned local guidance; attribution lives in NOTICE.md
---

# Deprecation And Migration

This bundled optional domain skill is for changing old behavior safely. It is not a workflow skill. Superpowers remains the workflow core for planning, TDD, debugging, review, and verification.

## Baron Contract

- Use this skill only when Baron routing selects it for migration, deprecation, compatibility, rollout, rollback, or legacy retirement work.
- Do not replace Superpowers, Product Harness, Active Plan, proof, trace, or core quality gates.
- Preserve current behavior unless the user explicitly accepts a breaking change.
- Treat source of truth, compatibility window, rollback, data migration, and proof as first-class requirements.
- Mark unknown callers, unknown data owners, unknown migration volume, and unknown rollback safety as unknown.
- Never delete or transform data without a backup, dry-run, rollback, or explicit proof path appropriate to the risk lane.
- Keep project-specific migration facts in the current Vault capsule unless the user approves global promotion.
- Baron completion requires proof and trace; migration confidence cannot be inferred from silence.

## Use When

- The task changes legacy APIs, schemas, database migrations, config formats, feature flags, auth/session models, storage layout, provider integrations, or public behavior.
- The task deprecates old fields, routes, components, commands, workflows, packages, or runtime environments.
- The task needs dual-read, dual-write, backfill, expand/migrate/contract, compatibility window, shadow mode, or gradual rollout.
- The task has data migration or data-loss risk.
- The task moves a project from one architecture to another.
- The task asks whether it is safe to remove old code.

Do not use this skill for greenfield work with no compatibility, rollout, or migration risk.

## Migration Map

Before editing, identify:

- Current state: old behavior, old data, old API, old config, old dependency, or old workflow.
- Target state: exact new behavior and why it is needed.
- Source of truth: which system owns the data or decision during the transition.
- Consumers: UI, backend, mobile, workers, integrations, scripts, humans, or unknown.
- Compatibility window: how long old and new behavior coexist.
- Data migration: what changes, what can fail, and what must be reversible.
- Rollout control: feature flag, config switch, staged deploy, percentage rollout, or manual step.
- Rollback: exact path back to a safe state.
- Proof: commands, tests, dry-run, counts, checksums, shadow comparison, or smoke.

## Safe Patterns

### Expand / Migrate / Contract

- Expand schema or interface first with additive compatible fields.
- Migrate code to write/read the new shape.
- Backfill or convert old data in bounded batches.
- Verify counts and behavior.
- Contract only after old readers/writers are gone.

### Dual-Read

- Read from new source when available.
- Fall back to old source during the compatibility window.
- Emit safe diagnostics for fallback rate.
- Do not hide mismatches; record them as proof gaps.
- Remove fallback only after evidence shows it is no longer needed.

### Dual-Write

- Write to both old and new stores only when required and safe.
- Define what happens when one write fails.
- Use idempotency for retry-prone writes.
- Track mismatch counts and reconciliation.
- Keep rollback clear: which source remains authoritative?

### Feature Flag

- Flags must have owner, default, rollout plan, kill switch, and removal plan.
- Do not use flags to hide unverified data migration risk.
- Test both enabled and disabled paths while both exist.
- Remove stale flags after the compatibility window.

### Shadow Mode

- Run new behavior without affecting users.
- Compare output to old behavior.
- Log safe mismatch summaries.
- Promote only after mismatch risk is understood.
- Keep unknown mismatches as blockers for high-risk work.

## Data Migration Rules

- Back up before destructive changes.
- Prefer idempotent scripts.
- Process in bounded batches for large data.
- Store progress checkpoints.
- Verify source count, target count, failed count, skipped count, and checksum or invariant when practical.
- Keep raw PII and secrets out of logs and Vault memory.
- Do not run irreversible migration from an AI suggestion without explicit operator approval.
- For database migrations, verify rollback or explain why rollback is impossible and what compensating restore exists.

## Compatibility Window

- State start condition and end condition.
- State old-client support behavior.
- State user-visible changes.
- State operational monitoring during the window.
- State owner for removing old code.
- If the window is unknown, record it as unknown and do not claim deprecation complete.

## Rollback

- Rollback must be more than "revert the code" when data or external systems change.
- Define what happens to writes created during the new path.
- Define whether old code can read new data.
- Define provider/config rollback.
- Define feature flag or kill switch behavior.
- Verify rollback path in test or dry-run when risk is high.

## Verification

- Unit tests cover conversion logic, flags, compatibility helpers, and validation.
- Integration tests cover old path, new path, dual-read, dual-write, and rollback behavior where practical.
- Dry-run proves data migration plan without writing.
- Smoke tests prove user or API flow still works.
- Counts/checksums prove data migration completeness.
- Observability proves fallback/mismatch/error rates are visible.
- Security review is required when auth, permission, tenant, secrets, payments, or data exposure changes.
- Baron proof records commands/artifacts; Baron trace records current state, rollout state, rollback, and unknowns.

## Output Contract

Return migration guidance in this shape:

1. Current state and target state.
2. Risk lane and risk flags.
3. Compatibility window and consumer impact.
4. Migration plan: expand, migrate, contract; dual-read/dual-write if needed.
5. Rollout and rollback plan.
6. Data migration safety: backup, dry-run, batches, counts, checksums, PII.
7. Verification: exact tests, smoke, dry-run, or missing proof.
8. Baron trace/proof notes and unresolved unknowns.

## Red Flags

- Deleting old code before proving no callers remain.
- Running destructive SQL without backup or rollback.
- Migrating all data in one unbounded job.
- Relying on UI hiding instead of backend compatibility.
- Changing public errors or fields silently.
- Feature flag with no owner or removal plan.
- Dual-write with no failure policy.
- Backfill with no progress checkpoint.
- Rollback impossible but not stated.
- Completion claimed without proof.

## Baron Final Check

- High-risk migration requires detailed trace quality.
- If proof is weak, keep status partial or blocked.
- If the same scope needs correction, update the existing plan instead of creating vague v2 files.
- If the migration creates reusable learning, propose it as a candidate, not trusted global memory, until approved.
