# 2026-07-24 - Superpowers 6.2 And Extension Decisions

## Scope

Refresh the bundled Superpowers workflow core to the pinned upstream `v6.2.0`
release and record the ownership boundaries for Graphify, Hallmark, Matt
Pocock's skills, and duplicate core assets.

This checkpoint does not implement Graphify, add a new optional skill, delete
`blueprints/core/`, bump Baron, or change the Phase 35-38 implementation order.

## Completed

- Audited the official Superpowers `v6.2.0` release.
- Added a RED adapter contract before replacing the bundled subtree.
- Vendored the complete upstream `skills/` snapshot at commit
  `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`.
- Preserved Baron-owned root routing in `SKILL.md` and `README.md`.
- Added `UPSTREAM.json`, `NOTICE.md`, and the upstream MIT license.
- Removed files retired by upstream 6.2.
- Verified the 50-file upstream subtree has no missing or extra file.
- Applied one recorded Baron hardening patch to the visual companion server:
  removed its remote brand image, telemetry signal, and live branding link.
- Verified Codex, Claude, and generic adapter installs receive the same pinned
  workflow tree.
- Pressure-tested plan isolation, review-fix behavior, and behavior-focused
  testing with baseline and skill-loaded agents.
- The first review-loop pressure test exposed that an agent could summarize the
  long SDD skill incorrectly. Added a short Baron-owned integration contract
  that makes the five-round breaker and escalation split explicit at the skill
  entry point, then added adapter assertions for that contract.
- Added an embedded-script mode contract. Shebang assets are installed as
  executable on Unix so the new SDD helpers work after Baron writes them from
  the binary; Windows remains unchanged.
- Accepted the one-owner extension contract in ADR 0002.

## Behavioral Test Notes

- Plan-isolation agents rejected a foreign flat ledger instead of treating
  matching task numbers as proof.
- Review agents returned findings to the original implementer and required
  independent re-review. One follow-up agent misreported the breaker, which is
  why the Baron-owned entry contract now repeats the exact rounds 1-3, rounds
  4-5, and post-round-5 rules.
- Test agents rejected grep-only proof and required an executable or observable
  behavior check with a mutation/failure case.

## Evidence

- Targeted adapter contract:
  `cargo test -p baron-adapters --test adapter_lifecycle superpowers_core_is_pinned_to_v6_2_and_installs_the_complete_workflow -- --exact`
- Embedded-script mode contract:
  `cargo test -p baron-adapters install::tests::embedded_shebang_assets_are_executable_but_docs_are_not -- --exact`
- Snapshot identity: 50 upstream files. Original upstream SHA-256 tree digest:
  `e76b5a985b27d626b7338028e292ecd944664ef5d85981c6fd05a8ae71b9291a`.
  Baron patched tree digest:
  `57dc7963c3610b035da7568ae5876577e3cd977baa9e239176044b09e08c8e82`.
- Decision record:
  `docs/decisions/0002-extension-ownership-and-code-graph.md`.

## Final Verification

- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `cargo test --workspace --all-targets`: passed with no failed or ignored
  Baron tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo build --release --locked -p baron-cli`: passed.
- Release binary smoke with a temporary Vault and
  `baron init --codex --fullstack`: installed the pinned 6.2 skill and the
  local-only visual server.
- Upstream visual-server tests after the Baron patch:
  - auth: 20 passed
  - browser launcher: 3 passed
  - helper: 15 passed
  - lifecycle: 13 passed
  - server: 31 passed, 2 Windows symlink-permission skips
  - WebSocket protocol: 32 passed
- Git Bash SDD semantic smoke: workspace isolation, task brief, review package,
  and git-ignore behavior passed.
- Upstream `find-polluter` and SessionStart shell suites passed.
- Three final quality-gate subagents were requested but exhausted their service
  quota before returning findings. Their incomplete runs were not counted as
  proof; the final local diff review and all commands above were completed
  independently.

No version bump, push, tag, release, Graphify integration, optional skill
addition, or `blueprints/core/` deletion is part of this checkpoint.

## Resume Point

Baron 3.4 remains at Phase 35. After this checkpoint is verified and committed,
resume `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`
from its first RED test.
