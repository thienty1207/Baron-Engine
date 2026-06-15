# Phase 11-12 Control Plane And Self-Improving Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Baron route skills/agents through validated contracts, require evidence for mandatory quality gates, and let the Product Harness audit itself without silently rewriting core policy.

**Architecture:** Add a Baron-native control plane that validates skill/agent assets, explains routing, records quality-gate evidence, and preserves custom routing. Add a self-improving harness layer that scores context-read behavior, audits drift/friction/proof gaps, records interventions, proposes improvements, and tracks outcomes behind a human approval gate.

**Tech Stack:** Rust 2021, Clap, Serde/TOML/Markdown parsing, existing Vault Markdown mirror, existing adapter managed blocks.

---

## File Structure

- Create `crates/baron-core/src/control_plane.rs`: skill/agent contract model, validation, routing, conflict diagnostics, gate evidence recording/status, and repo/Vault mirror.
- Create `crates/baron-core/src/harness_improvement.rs`: context-read score, drift audit, intervention log, story verification, friction grouping, improvement proposals, and outcome tracking.
- Modify `crates/baron-core/src/lib.rs`: expose Phase 12 identity and new modules.
- Modify `crates/baron-adapters/src/install.rs`: strengthen generated startup contracts and routing indexes with explicit contract requirements.
- Modify `crates/baron-cli/src/main.rs`: add `control-plane` commands and extend `harness` with `audit`, `verify-all`, `intervention`, `propose`, and `outcome`.
- Modify `crates/baron-core/src/context.rs`: include bounded control-plane and harness-improvement summaries.
- Modify status, roadmap, architecture docs, README, AGENTS, and build logs.

### Task 1: Phase 11 Control Plane Validation

**Files:**
- Create: `crates/baron-core/src/control_plane.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Test: `crates/baron-core/tests/control_plane.rs`

- [x] Write failing tests proving core skills and agents have validated contracts.
- [x] Write failing tests proving Superpowers is the only workflow owner.
- [x] Write failing tests proving duplicate workflow ownership and recursive subagent orchestration are diagnosed.
- [x] Implement contract parsing from installed repo assets plus embedded Baron assumptions.
- [x] Implement `validate_control_plane(repo_root)` with diagnostics and pass/fail status.
- [x] Run focused tests until green.
- [x] Commit `feat: add skill and agent control plane validation`.

### Task 2: Phase 11 Narrow Routing And Gate Evidence

**Files:**
- Modify: `crates/baron-core/src/control_plane.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-core/tests/control_plane.rs`
- Test: `crates/baron-cli/tests/control_plane_cli.rs`

- [x] Write failing tests proving frontend tasks route Superpowers plus `frontend-design`, `code-reviewer`, and `test-engineer`.
- [x] Write failing tests proving auth/security tasks route Superpowers plus `vibe-security-scan`, `security-auditor`, `code-reviewer`, and `test-engineer`.
- [x] Write failing tests proving routing explains selected and skipped assets.
- [x] Write failing tests proving medium/high-risk completion cannot count mandatory agents without gate evidence.
- [x] Implement `baron control-plane status|route|record-gate|evidence`.
- [x] Mirror gate evidence to Vault Markdown.
- [x] Run focused tests until green.
- [x] Commit `feat: add explainable quality gate routing`.

### Task 3: Phase 11 Adapter Contract Hardening

**Files:**
- Modify: `crates/baron-adapters/src/install.rs`
- Test: `crates/baron-adapters/tests/adapter_lifecycle.rs`

- [x] Write failing tests proving generated indexes include ownership, trigger, exclusion, evidence, and conflict rules.
- [x] Write failing tests proving custom routing survives while managed contract text refreshes.
- [x] Strengthen Codex, Claude, and generic startup contracts to call `baron control-plane route` and record gate evidence.
- [x] Run adapter tests until green.
- [x] Commit `feat: harden adapter skill and agent contracts`.

### Task 4: Phase 12 Harness Audit And Intervention Records

**Files:**
- Create: `crates/baron-core/src/harness_improvement.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Test: `crates/baron-core/tests/harness_improvement.rs`

- [x] Write failing tests proving context-read score reports observed/missing startup actions.
- [x] Write failing tests proving documentation drift and contradiction diagnostics are bounded.
- [x] Write failing tests proving intervention records are mirrored to Vault.
- [x] Implement `audit_harness`, `record_intervention`, and drift/friction/proof-gap summaries.
- [x] Run focused tests until green.
- [x] Commit `feat: add self-improving harness audit`.

### Task 5: Phase 12 Story Verification, Proposals, And Outcomes

**Files:**
- Modify: `crates/baron-core/src/harness_improvement.rs`
- Modify: `crates/baron-cli/src/main.rs`
- Test: `crates/baron-core/tests/harness_improvement.rs`
- Test: `crates/baron-cli/tests/harness_improvement_cli.rs`

- [x] Write failing tests proving open stories and proof gaps are verified in bounded batches.
- [x] Write failing tests proving repeated friction creates an improvement proposal.
- [x] Write failing tests proving proposals are not applied to core policy without human approval.
- [x] Write failing tests proving predicted impact and actual outcome can be recorded.
- [x] Implement `baron harness audit|verify-all|intervention|propose|outcome`.
- [x] Run focused tests until green.
- [x] Commit `feat: add harness improvement outcome loop`.

### Task 6: Context, Docs, Status, And Closure

**Files:**
- Modify: `crates/baron-core/src/context.rs`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/roadmap/2026-06-08-implementation-roadmap.md`
- Modify: `docs/architecture/ADAPTERS.md`
- Modify: `docs/architecture/COMMAND_SURFACE.md`
- Modify: `docs/architecture/ARCHITECTURE.md`
- Modify: `notes/build-log/CURRENT.md`
- Create: `notes/build-log/2026-06-16-phase-11-12-control-plane-harness.md`

- [x] Write failing context/docs tests proving compact context contains bounded control-plane and harness audit summaries.
- [x] Update docs to mark Phase 11 and Phase 12 complete only after proof passes.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Smoke a temp repo: init, route security task, record three gates, audit, intervention, propose, outcome, context.
- [x] Run `git diff --check`.
- [x] Commit `docs: complete Baron phases 11 and 12`.

## Self-Review

- Phase 11 maps to validated contracts, narrow routing, conflict diagnostics, custom preservation, and mandatory gate evidence.
- Phase 12 maps to context-read score, drift audit, interventions, story verification, friction proposals, outcome loop, and human approval gate.
- Superpowers remains workflow core; Baron control plane is routing/evidence infrastructure, not a new workflow brain.
- Product Harness may propose improvements but must not rewrite AGENTS, architecture, or core policy automatically.
