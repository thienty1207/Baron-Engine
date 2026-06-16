# Current Build Note

Date: 2026-06-15

## Current Phase

Phase 13-14 - Certification And Release (completed). Baron 2.0.0 source release
gates are implemented and locally verified.

## What Is Being Built

Baron 2.0 final batch delivered:

- extreme-scale certification
- hardened `v2.0.0` release

## Current Status

Phase 8 is merged to `main`, tagged `v1.0.0`, and published on GitHub. Phase 9
and Phase 10 are implemented in an isolated worktree with stored project
identity, native Codex/Claude hooks, lifecycle reconciliation, preserved custom
routing, incremental indexing, multilingual task-aware recall, and automatic
session import.

Baron 2.0 progress is 100% as a source release. Native GitHub release asset
publishing remains a separate operator action.

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
- duplicate-name project identity and move-preservation tests: passed
- legacy slug capsule migration without Markdown loss: passed
- Codex/Claude native hook merge and custom-hook preservation tests: passed
- automation journal, checkpoint throttle, and Stop reconciliation tests: passed
- custom skill/agent routing registration preservation tests: passed
- incremental 350-source memory index reuse/change/delete tests: passed
- risky surface detection beyond 6,000 repository entries: passed
- same-slug project-ID firewall isolation test: passed
- Vietnamese security query to Supabase RLS semantic recall test: passed
- task-focused context selection test: passed
- Codex/Claude session match, redaction, noise filtering, and dedupe tests: passed
- automatic context session import test: passed
- canonical Windows short-path and equivalent-path session matching regression: passed
- two same-name repositories produced two isolated Vault capsules: passed
- 1,008-source real Vault index smoke: passed
- Vietnamese customer-data security query to English RLS memory smoke: passed
- real session import redacted the secret and imported exactly once: passed
- real Codex SessionStart hook injected context and journaled execution: passed
- Phase 9-10 full `cargo test --workspace --all-targets`: passed
- Phase 9-10 `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 9-10 `git diff --check`: passed
- Phase 11 control-plane validation and conflict tests: passed
- Phase 11 explainable routing and gate-evidence tests: passed
- Phase 11 adapter contract hardening tests: passed
- Phase 12 context-read, drift, intervention, story verification, proposal, and outcome tests: passed
- Phase 12 CLI audit/verify/propose/outcome tests: passed
- Phase 11-12 compact context summary tests: passed
- Phase 11-12 route/gate/audit/propose/outcome/context smoke: passed
- Phase 11-12 `cargo fmt --all -- --check`: passed
- Phase 11-12 `cargo test --workspace --all-targets`: passed
- Phase 11-12 `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 11-12 `docs/BARON_STATUS.json` parse: passed
- Phase 11-12 `git diff --check`: passed
- Phase 13 certification core RED/GREEN tests: passed
- Phase 13 certification CLI RED/GREEN tests: passed
- Phase 14 release version RED/GREEN tests: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- temp repo smoke for init, memory index, context, control-plane route, harness audit, certification, certification status, release metadata, and release verification: passed

## Next Action

Commit docs/status, run final `git diff --check`, merge `codex/phase-13-14`
into `main`, and push `origin/main`.

## Phase 9-10 Feature Commits

- `7d602b9 feat: add collision resistant project identity`
- `8640e16 feat: add native automation and preserve routing`
- `3e0bbbe feat: add incremental large memory index`
- `e62c8a2 feat: add task aware multilingual recall`
- `25f434e feat: add automatic session memory ingestion`
- `aa089ee fix: match canonical session paths`
- Completion documentation is recorded in the Phase 9-10 git history.

## Phase 11-12 Feature Commits

- `1ab4d9b feat: add skill and agent control plane validation`
- `500eb4f feat: add explainable quality gate routing`
- `9b3d160 feat: harden adapter skill and agent contracts`
- `873a0fd feat: add self-improving harness audit`
- `b6fe5d6 feat: add harness improvement outcome loop`
- Completion documentation is recorded in the Phase 11-12 git history.

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
