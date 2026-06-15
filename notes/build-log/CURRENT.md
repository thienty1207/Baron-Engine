# Current Build Note

Date: 2026-06-15

## Current Phase

Phase 9 - Automation Runtime And Project Identity (planned; implementation not started).

## What Is Being Built

Baron 2.0 is a six-phase continuation:

- IDE-compatible automatic lifecycle behavior without requiring `baron run`
- collision-resistant project identity
- massive and meaning-aware memory
- strict skill and agent control plane
- self-improving Product Harness
- extreme-scale certification and a hardened `v2.0.0` release

## Current Status

Phase 8 is merged to `main`, tagged `v1.0.0`, and published on GitHub. Baron
2.0 implementation has not started. The program design and Phase 9-14 roadmap
are now the source of truth for remaining work.

The roadmap was extended after an adversarial audit confirmed duplicate-name
project identity collisions, fixed scan limits, lexical-only recall, custom
routing loss during update, instruction-heavy automation, missing live session
ingestion, and an incomplete harness improvement loop.

## Verification

- Phase 3 baseline `cargo build --quiet`: passed
- Phase 3 baseline `cargo test --quiet`: passed
- configuration tests: passed
- adapter lifecycle and preservation tests: passed
- plan and harness tests: passed
- proof and trace tests: passed
- weak-proof and managed-state false-positive regressions: passed
- execution CLI tests: passed
- full `cargo test --quiet`: passed
- three-adapter/custom-preservation/high-risk lifecycle smoke: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- fresh three-adapter/custom-preservation/high-risk lifecycle smoke: passed
- `docs/BARON_STATUS.json` parse: passed
- `git diff --check`: passed after normalizing vendored trailing whitespace
- Phase 4-5 merged and pushed to `origin/main`: passed
- Phase 6-8 roadmap/status synchronization: passed
- `docs/BARON_STATUS.json` parse after roadmap extension: passed
- static phase-reference scan after roadmap extension: passed
- Phase 6 migration core tests: passed
- Phase 6 migration CLI tests: passed
- Baron-native core agent/skill contract tests: passed
- separate source/destination Vault migration: passed
- automatic rollback after injected install failure: passed
- malicious manifest path traversal and unsafe slug regressions: passed
- file-granular rollback preserves post-migration plans and memory: passed
- manual dry-run/apply/recall/status/rollback smoke: passed
- Phase 6 full `cargo test --workspace --all-targets`: passed
- Phase 6 `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 7 registry core tests: passed
- Phase 7 provider-kind and HTTP/MCP/adapter probe tests: passed
- Phase 7 capability CLI tests: passed
- Phase 7 context bounding and adapter cache-isolation tests: passed
- Phase 7 Proof/Trace false-claim regression tests: passed
- Phase 7 Codex/Claude/generic automatic startup tests: passed
- Vault environment race regression: passed 5 consecutive runs
- Phase 7 full `cargo test --workspace --all-targets`: passed
- Phase 7 `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 7 `cargo fmt --all -- --check`: passed
- Phase 7 positive register/check/context/proof/trace/complete smoke: passed
- Phase 7 missing-required-provider negative smoke: passed and blocked completion
- `docs/BARON_STATUS.json` parse: passed
- `git diff --check`: passed
- Phase 8 branch CI `27534373035`: passed on all four native platforms
- Phase 8 main CI `27534650603`: passed
- Phase 8 release workflow `27534923497`: passed
- Published `v1.0.0` install/update/rollback/uninstall lifecycle: passed

## Next Action

Write and approve the detailed Phase 9 design and implementation plan. Begin
implementation with failing regressions for duplicate project names, custom
routing preservation, and observable IDE-compatible lifecycle automation.

## Phase 4-5 Feature Commits

- `07c09db feat: add Baron project configuration`
- `737a4e8 feat: add managed multi-agent adapters`
- `68df801 feat: add adapter init and update lifecycle`
- `0f555e9 feat: add risk-aware product harness`
- `055b0b8 feat: add proof and trace quality gates`
- `bd058e2 feat: add gated active plan state`
- `217233f feat: expose Baron execution state commands`
- `3ee93c5 feat: connect execution state to context and memory`
- `094891e feat: tighten Baron execution quality gates`

## Roadmap Extension

- Phase 6: Native Migration And Legacy Retirement
- Phase 7: Baron Capability Registry
- Phase 8: Release Hardening
- Decision log: `notes/build-log/2026-06-15-phase-6-8-roadmap.md`
- Phase 6 design: `docs/superpowers/specs/2026-06-15-native-migration-legacy-retirement-design.md`
- Phase 6 plan: `docs/superpowers/plans/2026-06-15-phase-6-native-migration.md`
- Phase 6 build log: `notes/build-log/2026-06-15-phase-6-native-migration.md`
- Phase 7 design: `docs/superpowers/specs/2026-06-15-baron-capability-registry-design.md`
- Phase 7 plan: `docs/superpowers/plans/2026-06-15-phase-7-capability-registry.md`
- Phase 7 build log: `notes/build-log/2026-06-15-phase-7-capability-registry.md`
- Phase 8 design: `docs/superpowers/specs/2026-06-15-release-hardening-design.md`
- Phase 8 plan: `docs/superpowers/plans/2026-06-15-phase-8-release-hardening.md`
- Phase 8 build log: `notes/build-log/2026-06-15-phase-8-release-hardening.md`

## Baron 2.0 Roadmap Extension

- Phase 9: Automation Runtime And Project Identity
- Phase 10: Massive Memory And Semantic Recall
- Phase 11: Skill And Agent Control Plane
- Phase 12: Self-Improving Harness
- Phase 13: Extreme Scale Certification
- Phase 14: Baron 2.0 Release Hardening
- Program design: `docs/superpowers/specs/2026-06-15-baron-2-program-design.md`
- Decision log: `notes/build-log/2026-06-15-baron-2-roadmap.md`

## Completed Phase 4-5 Work

- Branch was merged and deleted.
- Design: `docs/superpowers/specs/2026-06-14-agent-adapters-execution-engine-design.md`
- Plan: `docs/superpowers/plans/2026-06-14-phase-4-5-adapters-execution-engine.md`

## Phase 3 Commit

- Commit: `c192baf feat: add Baron context compiler`

## Phase 2 Commit

- Commit: `c6fc469 feat: add vault memory firewall`

## Phase 1 Commit

- Commit message: `feat: add Baron survey engine`
- Use `git log -1 --oneline` after commit for the exact hash.

## Phase 0 Commit

- Commit message: `chore: create Baron phase 0 foundation`
- Use `git log -1 --oneline` for the exact hash.

## Do Not Forget

- Superpowers remains the workflow core.
- Core agents are `code-reviewer`, `security-auditor`, `test-engineer`.
- Optional bundled domain skills are `frontend-design` and `vibe-security-scan`.
- Baron must support Codex, Claude, and generic agents through adapters.
- Shadow-first onboarding is mandatory for old repos.
- Migration imports data, not Agent Bootstrap architecture or runtime.
- Skills and agents must satisfy Baron-native contracts before activation.
- Missing capabilities must reduce confidence instead of producing false claims.
