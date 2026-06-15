# Baron 2.0 Program Design

Date: 2026-06-15
Status: approved direction, implementation not started

## Goal

Baron 2.0 must become a durable, IDE-compatible agent harness that is
substantially stronger than both Agent Bootstrap and repository-harness in four
areas:

- automatic execution without requiring a dedicated Baron launcher
- large, isolated, meaning-aware memory
- strict and explainable skill/agent routing
- self-improving harness behavior backed by evidence

## Product Boundary

Users may open Codex, Claude, or another agent directly, including through an
IDE. Baron must not require `baron run` as the normal entrypoint.

Automation therefore uses a layered model:

1. native hooks or lifecycle integrations where an adapter supports them
2. adapter-managed startup/finalization instructions where hooks are absent
3. an idempotent reconciliation check that detects missed context, plan,
   harness, proof, trace, and memory actions before completion is accepted

Instructions alone are not considered guaranteed automation. Baron must report
which lifecycle events were actually observed and which were only requested.

## Program Structure

### Phase 9 - Automation Runtime And Project Identity

Build an IDE-compatible lifecycle runtime and replace basename-only project
identity with stable, collision-resistant project IDs. Fix update behavior so
Baron-managed refreshes cannot erase custom routing registrations.

### Phase 10 - Massive Memory And Semantic Recall

Replace fixed scan limits with deterministic incremental indexing. Add
task-aware hybrid recall, multilingual concept matching, session ingestion,
redaction, deduplication, recency, evidence confidence, and bounded context
selection. Local semantic providers are optional accelerators; Baron must
remain correct without a model, cloud service, or API key.

### Phase 11 - Skill And Agent Control Plane

Give every skill and agent a validated contract covering triggers, exclusions,
ownership, conflicts, dependencies, inputs, outputs, evidence, and completion
rules. Routing must be narrow, explainable, update-safe, and stronger than
folder discovery or prompt-only conventions.

### Phase 12 - Self-Improving Harness

Add context-read scoring, documentation drift audits, intervention records,
batch story verification, repeated-friction analysis, improvement proposals,
and measured backlog outcomes. Baron may propose improvements automatically,
but must not silently rewrite core policy or architecture.

### Phase 13 - Extreme Scale Certification

Prove behavior against old and very large repositories, hundreds of projects
sharing one Vault, large memory histories, interrupted sessions, corrupted
caches, project moves, project renames, duplicate names, and cross-platform
execution. Performance and isolation budgets become release gates.

### Phase 14 - Baron 2.0 Release Hardening

Provide safe v1-to-v2 migration, deterministic release assets, lifecycle
installers, rollback, compatibility tests, complete documentation, and native
CI proof. Baron 2.0 cannot be released while any identity, memory loss,
cross-project contamination, automation, routing, or evidence gate remains
unproven.

## Non-Negotiable Outcomes

- No two repositories share memory accidentally, even when their folder names
  are identical.
- No memory is silently dropped because a fixed file-count limit was reached.
- Context remains bounded even when the Vault is very large.
- Recall works across close meanings and common Vietnamese/English engineering
  language.
- Session memory is imported automatically where a supported adapter exposes
  history, with redaction and deduplication.
- Custom skill and agent files plus their routing registrations survive every
  update.
- Superpowers remains the workflow core.
- `code-reviewer`, `security-auditor`, and `test-engineer` remain the three core
  quality gates.
- Optional skills and agents are loaded only when their validated routing
  contracts match.
- Medium/high-risk completion requires proof that mandatory quality gates
  actually ran.
- Missing hooks, tools, memory, context, or proof are visible diagnostics, not
  silent success.

## Release Standard

Passing unit tests is necessary but insufficient. Baron 2.0 requires adversarial
stress tests, multi-project isolation tests, lifecycle interruption tests,
cross-platform release tests, and measured proof that automatic actions
occurred. Status documentation must distinguish completed implementation from
planned work and must never use `100%` for the 2.0 program before Phase 14
passes.
