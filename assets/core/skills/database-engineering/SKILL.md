---
name: database-engineering
description: Use for relational modeling, constraints, indexes, query plans, transactions, concurrency, migrations, backfills, rollback, and database integrity.
routing_triggers: relational schema,foreign key,constraint,primary key,nullability,uniqueness,referential integrity,normalization,denormalization,index,query plan,n+1,transaction,isolation,lock,deadlock,migrate,migration,backfill,rollback,orm usage,repository data access,persistence
routing_exclusions: csv transform,kafka ingestion,analytics quality,frontend page,pure http handler,incidental sql
profile_affinities: database,backend,fullstack
routing_dependencies: superpowers
routing_conflicts: mobile-application-engineering,apk-mobile-analysis
evidence_requirements: schema or query evidence,affected data boundary,migration or integrity impact
verification_hints: schema constraints,query plan,index,transaction concurrency,rollback or integrity check
---

# Database Engineering

## Baron Contract

Use when the task changes persistence semantics, relational integrity, query
behavior, transaction boundaries, or recoverability. Use this skill for
database engineering decisions in those areas. Superpowers remains the
workflow core. Keep the route narrow:
load this skill when task evidence or a configured Database profile supports a
real database concern, not merely because a file contains a SQL string.

## Modeling

- Identify entities, relationships, ownership, and lifecycle before changing a
  schema.
- Give each relation an intentional primary key and document key stability.
- Use foreign keys, uniqueness, nullability, and check constraints to make
  invariants executable where the database owns the invariant.
- Preserve referential integrity and define delete/update behavior explicitly.
- Prefer normalized models until a measured access pattern justifies
  intentional denormalization and its consistency plan.
- Record assumptions about cardinality, optionality, tenant boundaries, and
  sensitive fields as evidence, not guesses.

## Query Design

- Start from the access pattern and expected result shape; avoid accidental
  over-fetching and unbounded result sets.
- Choose indexes from selectivity, ordering, filter combinations, and write
  cost. Validate composite-index column order against real predicates.
- Inspect an execution or query plan for slow paths and record the evidence.
- Bound pagination and make ordering stable; do not hide an N+1 query behind a
  repository or ORM abstraction.
- Keep query, pool, timeout, and retry behavior within known resource bounds.
- Label performance claims as measured, estimated, or unknown.

## Transactions and Concurrency

- State the transaction owner and keep atomic work inside one deliberate
  boundary.
- Select isolation and locking for the anomaly being prevented; consider lost
  updates, write skew, phantom reads, and deadlocks.
- Keep lock order consistent and define bounded retry behavior for transient
  conflicts. Make retries idempotent where a write can be repeated.
- Test concurrent behavior when the task changes a race, isolation level, or
  lock boundary. Do not claim concurrency safety from a unit test alone.

## Migration and Backfill

- Prefer additive, compatible changes before destructive changes. Use an
  expand/contract window when old and new application versions overlap.
- Separate schema migration from data backfill when that improves restart and
  rollback safety. Make backfills bounded, resumable, and observable.
- Check nullability, defaults, indexes, foreign keys, and lock duration before
  applying a migration to a large table.
- Define compatibility windows, rollback or forward-fix behavior, and the
  validation query before execution.
- Treat destructive schema changes as high-risk: require confirmation, a
  recovery path, and proof that protected data is not silently lost.

## Application Boundary

- Keep ORM models, repositories, raw SQL, and transaction ownership aligned
  with the declared domain boundary.
- Use raw SQL when it makes a critical query or lock behavior explicit, and
  cover it with contract or integration evidence.
- Keep connection and pool lifecycle visible. Do not open hidden transactions
  across unrelated service boundaries.
- Preserve API compatibility when persistence changes alter serialized or
  externally visible behavior.

## Integrity and Recovery

- Validate constraints, representative data, and migration state before and
  after a change.
- Protect destructive operations with a dry-run or bounded preview whenever
  the platform supports it.
- Record backup/recovery expectations when data loss or irreversible migration
  is possible; do not invent backup success.
- Keep failed or interrupted migration state actionable with the last safe
  step, affected objects, blocker, and next recovery action.

## Output Contract

Report the affected schema or query boundary, evidence used, invariants,
performance/concurrency risks, migration and rollback plan, and exact
verification. Mark unknown database facts explicitly. Include proof and trace
status for high-risk work. Do not execute destructive SQL, live exploitation,
or irreversible data changes without the existing Baron authority and gates.
