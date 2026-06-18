# Baron Performance Skill Hardening

Date: 2026-06-18

## Trigger

The user compared Baron's `performance-optimization` skill against the much more detailed `addyosmani/agent-skills` performance skill and called out that Baron's version was too thin.

## Decision

Treat the feedback as correct. The previous Baron skill was only a routing contract. It did not teach an agent how to do performance work deeply enough for large, long-lived projects.

## Change

- Rewrote `assets/core/skills/performance-optimization/SKILL.md` as a Baron-native operational guide.
- Added a regression test requiring the skill to include measurement workflow, Core Web Vitals, common anti-patterns, budget guidance, and verification language.
- Kept the skill optional and non-core. Superpowers still owns workflow, and `web-performance-auditor` remains an optional advisory agent for web-specific performance audits.

## Verification

- RED: `cargo test -p baron-adapters --test adapter_lifecycle performance_optimization_skill_is_operationally_detailed -- --nocapture` failed before the rewrite.
- GREEN: the same targeted test passed after the rewrite.

## Next Action

Run format/full tests/Clippy, then commit and push if clean.
