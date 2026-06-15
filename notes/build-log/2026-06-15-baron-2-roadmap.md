# Baron 2.0 Roadmap Decision Log

Date: 2026-06-15

## Decision

Extend Baron with six phases, Phase 9 through Phase 14, and target release
`2.0.0`.

## Why

The `v1.0.0` release is a working foundation, but an adversarial audit found
gaps that conflict with the long-term promise:

- repositories with the same final folder name can share one Vault capsule
- project memory indexing stops after a fixed number of Markdown files
- repository survey stops after a fixed number of entries
- recall is lexical and weak across paraphrases or languages
- custom skill/agent routing text can be replaced during adapter update
- normal automation depends too heavily on an agent obeying instructions
- live Codex/Claude session ingestion is not yet a Baron-native subsystem
- the newest harness evolution loop is not yet implemented

## User Constraint

Do not require users to start agents through a Baron launcher. Users may work
through an IDE or open Codex/Claude directly.

## Architecture Response

Use native adapter hooks where available, adapter-managed instructions where
hooks are unavailable, and a Baron reconciliation/evidence layer that detects
missed lifecycle actions. Do not describe instruction-only behavior as
guaranteed automation.

## Remaining Phases

1. Phase 9 - Automation Runtime And Project Identity
2. Phase 10 - Massive Memory And Semantic Recall
3. Phase 11 - Skill And Agent Control Plane
4. Phase 12 - Self-Improving Harness
5. Phase 13 - Extreme Scale Certification
6. Phase 14 - Baron 2.0 Release Hardening

## Resume Point

Before implementation, write and approve the detailed Phase 9 design and
implementation plan. Start with regression tests for duplicate project names,
custom routing preservation, and observable IDE-compatible lifecycle
automation.
