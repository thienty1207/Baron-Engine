# Current Build Note

Date: 2026-06-19

## Current Phase

Phase 24 - Baron 3.1 public trust release (published and verified).
Baron 3.0 Phase 18-23 are implemented and verified locally. The current task
is packaging that engine so GitHub readers can understand, install, trust, and
compare Baron quickly.

## What Is Being Built

Baron 3.1 public trust batch:

- Phase 24 keeps the engine unchanged and improves public trust.
- README becomes a concise landing page.
- Demo docs show a 10-year repo flow for Codex, Claude, and generic agents.
- Assessment docs explain Baron through Baron-owned certification evidence.
- Certification docs list concrete public proof and release/latest expectations.

## Current Status

Baron 3.1.1 is the current source release target. Phase 24 is implemented
locally with external harness comparison references removed. GitHub release
publication for `v3.1.1` is pending.

Current resume point:

- `docs/BARON_STATUS.md` marks Phase 24 as the Public Trust Release.
- `docs/BARON_STATUS.json` tracks stable/target release `3.1.1`, 100% completion, and zero remaining planned phases.
- `docs/demo/README.md` is the public demo.
- `docs/assessment/baron-3-public-certification.md` is the public proof snapshot.
- `notes/build-log/2026-06-19-baron-3-roadmap.md` records the trigger and non-negotiables.
- `notes/build-log/2026-06-19-phase-18-20-baron-3-foundation.md` records the implementation details.
- `docs/superpowers/plans/2026-06-19-phase-21-23-baron-3-release.md` is the active implementation plan.
- `notes/build-log/2026-06-19-phase-21-23-baron-3-release.md` is the active checkpoint log.
- Phase 21 autopilot core/CLI/context targeted tests are green.
- Phase 22 runtime policy core/CLI/context targeted tests are green.
- Phase 23 version/release/certification targeted tests are green.
- Public Trust docs/status targeted verification is green.
- Full workspace verification, Clippy, fmt check, status JSON parse, stale-release scan, and diff check are green.
- External harness reference cleanup test is green.
- Static scan for removed external harness references is green.
- Next implementation step: run full verification, commit, push, tag `v3.1.1`, and verify GitHub `releases/latest`.

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
- Phase 15 setup/init/platform RED/GREEN tests: passed
- Phase 15 top-level help simplification tests: passed
- Phase 15 platform context test: passed
- Phase 15 generated-agent-asset survey regression test: passed
- Phase 15 full `cargo test --workspace --all-targets`: passed
- Phase 15 `cargo fmt --all -- --check`: passed
- Phase 15 `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 15 setup/init/context smoke: passed
- `cargo fmt --all -- --check`: passed
- `cargo test --workspace --all-targets`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- temp repo smoke for init, memory index, context, control-plane route, harness audit, certification, certification status, release metadata, and release verification: passed
- Phase 16-17 targeted RED/GREEN tests for adapter lifecycle, control-plane routing, continuity, automation CLI, and context resume: passed
- Phase 16-17 full `cargo fmt --all -- --check`: passed
- Phase 16-17 full `cargo test --workspace --all-targets`: passed
- Phase 16-17 full `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 16-17 temp repo smoke for setup, init, optional skill routing, optional web performance agent routing, continuity checkpoint/status, and context resume: passed
- Performance skill hardening RED test before rewrite: failed as expected
- Performance skill hardening targeted GREEN test after rewrite: passed
- Performance skill hardening full `cargo test --workspace --all-targets`: passed
- Performance skill hardening full `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Performance skill hardening `cargo fmt --all -- --check`: passed
- Performance skill hardening `git diff --check`: passed
- Performance skill hardening temp repo smoke for init, installed skill content, performance routing, and optional web performance agent routing: passed
- Baron 3.0 roadmap status update `docs/BARON_STATUS.json` parse: passed
- Baron 3.0 roadmap status update `git diff --check`: passed
- Phase 18 asset sovereignty RED tests: failed as expected before implementation
- Phase 19 asset lifecycle RED tests: failed as expected before implementation
- Phase 20 session replay RED tests: failed as expected before implementation
- Phase 18 asset sovereignty targeted GREEN tests: passed
- Phase 19 asset lifecycle targeted GREEN tests: passed
- Phase 20 session replay targeted GREEN tests: passed
- Phase 20 context replay integration targeted test: passed
- Phase 19 hidden CLI help targeted test: passed
- Phase 18-20 `cargo fmt --all`: passed
- Phase 18-20 `cargo test --workspace --all-targets`: passed
- Phase 18-20 `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 18-20 temp repo smoke for setup, init, asset audit, session replay index/search, and task context replay: passed
- Phase 18-20 `docs/BARON_STATUS.json` parse: passed
- Phase 18-20 runtime optional skill/agent live-link scan: passed
- Phase 18-20 `git diff --check`: passed
- Phase 21 autopilot RED tests: failed as expected before implementation
- Phase 22 runtime policy RED tests: failed as expected before implementation
- Phase 21 autopilot targeted GREEN tests: passed
- Phase 22 runtime policy targeted GREEN tests: passed
- Phase 21-23 context, adapter lifecycle, release, lifecycle installer, and certification targeted tests: passed
- Phase 21-23 full `cargo fmt --all -- --check`: passed
- Phase 21-23 full `cargo test --workspace --all-targets`: passed
- Phase 21-23 full `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 21-23 temp repo smoke for setup, Codex/Claude/generic init, shared Vault memory index, runtime check, context, autopilot review/status, recall, certification, and Agent Bootstrap migration dry-run: passed
- Phase 21-23 `docs/BARON_STATUS.json` parse: passed
- Phase 21-23 stale-release static scan: passed
- Phase 21-23 `git diff --check`: passed
- Phase 24 public-trust docs RED test: failed as expected before implementation
- Phase 24 public-trust docs GREEN test: passed
- Phase 24 full `cargo fmt --all -- --check`: passed
- Phase 24 full `cargo test --workspace --all-targets`: passed
- Phase 24 full `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Phase 24 `git diff --check`: passed
- Phase 24 external harness reference cleanup targeted test: passed
- Phase 24 external harness reference static scan: passed

## Next Action

Baron 3.1.1 source cleanup is ready for final verification, commit, push, tag,
release workflow, and `releases/latest` smoke.

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
- Optional bundled domain skills include `frontend-design`, `vibe-security-scan`, `api-and-interface-design`, `observability-and-instrumentation`, `performance-optimization`, and `deprecation-and-migration`.
- Baron must support Codex, Claude, and generic agents through adapters.
- Shadow-first onboarding is mandatory for old repos.
- Migration imports data, not Agent Bootstrap architecture or runtime.
- Skills and agents must satisfy Baron-native contracts before activation.
- Missing capabilities must reduce confidence instead of producing false claims.
