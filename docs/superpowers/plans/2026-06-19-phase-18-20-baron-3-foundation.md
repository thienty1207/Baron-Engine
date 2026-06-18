# Baron 3.0 Phase 18-20 Implementation Plan

> Required workflow: use Superpowers as the workflow core, implement with RED/GREEN tests, and keep all new behavior Baron-native.

## Goal

Implement the first three Baron 3.0 phases:

- Phase 18: Asset Sovereignty And Skill/Agent Hardening
- Phase 19: Skill Lifecycle And Approval Engine
- Phase 20: Session Replay And Conversation Search

## Non-Negotiables

- Superpowers remains the workflow core.
- Runtime skill and agent instructions must be self-contained and local.
- Attribution and license details belong outside operational runtime guidance.
- Optional skills and optional agents remain lazy-routed and non-core.
- Session replay must stay bounded and project-safe.

## Tasks

### Task 1 - RED Tests

- [x] Add asset sovereignty tests for optional skills and agents.
- [x] Add skill lifecycle audit, quarantine, and staged proposal tests.
- [x] Add session replay indexing, search, and bounded replay tests.
- [x] Confirm targeted tests fail before implementation.

### Task 2 - Phase 18 Assets

- [x] Rewrite weak optional skills into self-contained Baron-native guidance.
- [x] Remove runtime external-link dependency language from operational files.
- [x] Deepen all bundled agents with scope, evidence, proof, trace, and anti-hallucination contracts.
- [x] Keep license/attribution in non-runtime files.

### Task 3 - Phase 19 Lifecycle Engine

- [x] Add asset quality audit.
- [x] Add quarantine for failing custom skills/agents.
- [x] Add staged skill update proposals with approval metadata.
- [x] Add hidden CLI inspection commands for AI/runtime use.

### Task 4 - Phase 20 Session Replay

- [x] Add local SQLite FTS session replay index.
- [x] Index imported session markdown from Vault project capsules.
- [x] Search exact prior messages with bounded surrounding context.
- [x] Include bounded relevant replay hits in context when a task is provided.

### Task 5 - Verification And Status

- [x] Update status/log/docs.
- [x] Run format, full tests, Clippy, and smoke.
- [ ] Commit and push.
