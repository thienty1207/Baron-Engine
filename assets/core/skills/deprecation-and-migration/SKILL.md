---
name: deprecation-and-migration
description: Use when migrating APIs, frameworks, schemas, data models, configs, feature flags, legacy behavior, compatibility layers, rollouts, or rollback paths.
license: MIT-inspired guidance; Baron-native adaptation
---

# Deprecation And Migration

This bundled optional domain skill is for changing old behavior safely. It is not a workflow owner; Superpowers still owns plan/TDD/debug/review/verification, and Baron proof/trace gates still decide completion.

## Use When

- moving from legacy code, old APIs, deprecated libraries, old schemas, old config, or compatibility layers
- changing public contracts where older callers may still exist
- planning rollout, rollback, dual-write, backfill, feature flag, or staged migration behavior

Do not use for greenfield work with no compatibility or migration risk.

## Baron Contract

- Preserve current behavior unless the user explicitly accepts a breaking change.
- Identify source of truth, rollback path, and data-loss risk before editing.
- Treat migration notes as project-specific unless the user confirms they are reusable global knowledge.
- Never mark migration work complete without proof appropriate to the risk lane.

## Migration Checklist

- What old behavior/data/interface exists today?
- Who depends on it?
- What compatibility window is needed?
- Is rollback possible without data loss?
- What gets migrated, backfilled, or cleaned up?
- What tests or smoke checks prove both old and new paths?

## Output Contract

- Current state
- Target state
- Compatibility and rollout plan
- Rollback/data-safety plan
- Verification evidence or missing proof
- Baron continuity checkpoint if the migration is interrupted

## Attribution

Inspired by MIT-licensed migration/deprecation ideas from `addyosmani/agent-skills`, rewritten as Baron-native optional guidance.
