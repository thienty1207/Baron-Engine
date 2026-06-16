# Baron 2.0 Final Audit

Date: 2026-06-16

## Verdict

Baron 2.0 is designed to be stronger than both Agent Bootstrap and
repository-harness because it combines three layers in one Rust-first engine:

- repository harnessing: product intent, story, proof, trace, and validation
- durable memory: shared Vault Markdown, SQLite cache, firewall, recall, and sessions
- agent control: adapters, automation evidence, skill/agent routing, capability proof, and certification

This is not a claim that Baron replaces every future idea from those projects.
It means Baron now has a broader, stricter foundation for long-lived and large
projects.

## Compared With Agent Bootstrap

Agent Bootstrap is strong at Obsidian-first memory and Codex setup. Baron goes
further by making the engine native and multi-agent:

- supports Codex, Claude, and generic agents instead of one primary tool shape
- uses stable project identity so same-name projects cannot share memory by accident
- keeps SQLite as a rebuildable accelerator while Vault Markdown stays source of truth
- has a control plane that validates skill/agent contracts instead of trusting folder presence
- records proof, trace quality, mandatory gate evidence, and automation events
- migrates useful Agent Bootstrap data but does not depend on its runtime

## Compared With repository-harness

repository-harness is excellent at making a repo explain itself to agents:
AGENTS, product contracts, story packets, validation matrix, and decisions.
Baron keeps that harness idea, then adds the missing long-term engine layer:

- shared Vault memory across many projects
- memory firewall to stop cross-project contamination
- task-aware multilingual recall
- automatic Codex/Claude session import with redaction and dedupe
- adapter-specific install/update for Codex, Claude, and generic agents
- capability registry so tool presence is not confused with tool execution
- certification reports for scale, cache recovery, context budget, and release readiness

## What Makes Baron Harder To Fool

- Unknown facts remain unknown instead of being guessed.
- Global memory candidates are not loaded as facts.
- Cross-project memory requires an explicit match.
- Medium/high-risk completion requires proof and trace quality.
- Mandatory quality agents do not count unless gate evidence is recorded.
- Missing tools lower proof confidence instead of silently passing.
- Certification fails the release claim if core checks fail.

## Proof Pointers

- Survey and bounded repo tests: `crates/baron-core/tests/survey.rs`
- Vault and memory firewall tests: `crates/baron-core/tests/vault_memory.rs`
- Context compiler tests: `crates/baron-core/tests/context_compiler.rs`
- Adapter lifecycle tests: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Plan/harness/proof/trace tests: `crates/baron-core/tests/plan.rs`, `crates/baron-core/tests/harness.rs`, `crates/baron-core/tests/proof_trace.rs`
- Migration tests: `crates/baron-core/tests/migration.rs`
- Capability tests: `crates/baron-core/tests/capability.rs`
- Control-plane tests: `crates/baron-core/tests/control_plane.rs`
- Self-improving harness tests: `crates/baron-core/tests/harness_improvement.rs`
- Certification tests: `crates/baron-core/tests/certification.rs`, `crates/baron-cli/tests/certification_cli.rs`
- Release tests: `crates/baron-core/tests/release.rs`, `crates/baron-cli/tests/lifecycle_scripts.rs`, `crates/baron-cli/tests/release_cli.rs`, `crates/baron-cli/tests/release_smoke.rs`

## Honest Boundary

Baron 2.0 is release-ready only after local full verification passes and the
branch is merged and pushed. Publishing native GitHub release assets is a
separate operator action unless explicitly requested.
