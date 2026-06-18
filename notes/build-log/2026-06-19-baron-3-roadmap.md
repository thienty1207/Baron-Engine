# Baron 3.0 Roadmap Log

Date: 2026-06-19

## Trigger

The user identified two problems that must be handled before Baron can be treated as a long-term top-tier engine:

1. Some bundled optional skills were too thin and contained runtime-facing external attribution/link language instead of self-contained Baron-native guidance.
2. `nousresearch/hermes-agent` shows strong ideas around self-improving agents, session replay, skill lifecycle, memory write approval, gateway/runtime awareness, and tool backends that Baron should learn from without replacing Superpowers.

## Decision

Open Baron `3.0.0` as a six-phase program:

- Phase 18: Asset Sovereignty And Skill/Agent Hardening
- Phase 19: Skill Lifecycle And Approval Engine
- Phase 20: Session Replay And Conversation Search
- Phase 21: Background Learning And Continuity Autopilot
- Phase 22: Capability Runtime And Safe Tool Backends
- Phase 23: Baron 3.0 Release Certification

## Non-Negotiables

- Superpowers remains the workflow core.
- Baron must not clone Hermes or become a Hermes wrapper.
- Baron runtime skills and agents must be self-contained local assets.
- External attribution belongs in `NOTICE.md` or `LICENSE.txt`, not operational instructions.
- Optional skills and optional agents stay lazy-routed and never become core.
- Vault Markdown remains the durable source of truth.

## Current Resume Point

- `docs/BARON_STATUS.md` declares Baron 3.0 direction and Phase 18-23.
- `docs/BARON_STATUS.json` tracks target release `3.0.0`, 65% completion, and Phase 21 as next.
- Phase 18-20 are implemented in `notes/build-log/2026-06-19-phase-18-20-baron-3-foundation.md`.
- Next implementation work should start with Phase 21 background learning and continuity autopilot.

## Next Action

Implement Phase 21 with RED tests first:

- candidate memory/skill/harness learning must not become trusted fact automatically
- approval gates must protect sensitive or runtime-affecting writes
- interrupted work must resume from continuity, session replay, plan, harness, proof, and trace state
- automation must record what actually ran instead of assuming compliance
