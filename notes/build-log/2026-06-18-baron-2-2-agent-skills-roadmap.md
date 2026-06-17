# Baron 2.2 Agent Skills Roadmap Log

Date: 2026-06-18

## Trigger

The user reviewed `addyosmani/agent-skills` and asked whether Baron can take useful ideas without making Baron weaker, noisier, or less automatic.

## Decision

Open Baron `2.2.0` as a two-phase additive program:

1. Phase 16 - Agent Skills Refinement
   - Refine the 3 Baron core quality agents.
   - Improve existing optional frontend/security skills instead of creating duplicate ownership.
   - Add only narrow optional domains that Baron is missing.
   - Keep Superpowers as the workflow core.
   - Keep optional routing lazy, explainable, and test-backed.

2. Phase 17 - Continuity Ledger And Resume Discipline
   - Productize the current status/build-log/plan/trace/journal habit into an explicit Baron resume contract.
   - Ensure every meaningful implementation has a checkpoint trail that survives network loss, quota exhaustion, shutdown, or session restart.
   - Keep the behavior automatic for AI agents; normal users should not need extra commands.

## Current Resume Point

- Phase 16 and Phase 17 implementation is complete in source.
- RED tests were added before implementation for optional skill/agent routing and continuity resume behavior.
- Phase 16 includes refined core agents, optional web performance agent, optional API/observability/performance/migration skills, and expanded control-plane routing.
- Phase 17 includes `continuity` module, hidden continuity CLI commands, hook-driven checkpoints, context resume output, and adapter startup rules.
- Full workspace verification passed: `cargo fmt --all -- --check`, `cargo test --workspace --all-targets`, and `cargo clippy --workspace --all-targets -- -D warnings`.
- Temp repo smoke passed for setup, init, optional routing, optional web performance agent, continuity checkpoint/status, and context resume.
- Commit and push are the only remaining actions in this turn.

## Existing Capability Check

Baron already has partial interruption recovery through:

- `docs/BARON_STATUS.md`
- `docs/BARON_STATUS.json`
- `notes/build-log/CURRENT.md`
- active plan state
- trace/proof records
- automation journal and lifecycle reconciliation

Gap being closed in Phase 17: these pieces existed, but there was not yet one explicit "Continuity Ledger" feature that tells every agent exactly where to resume after interrupted implementation.

## Guardrails

- Do not copy another repository as Baron's architecture.
- Do not add duplicate workflow skills that compete with Superpowers.
- Do not make optional skills or agents core.
- Do not expose internal automation command clutter to normal users.
- Do not claim Phase 16 or Phase 17 complete without tests and smoke verification.
