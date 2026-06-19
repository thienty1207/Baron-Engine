# Phase 24 Public Trust Release Build Log

Date: 2026-06-19

## Scope

Ship Baron `3.1.0` as a public-trust release.

This phase does not add core memory, harness, proof, trace, skill, or agent
behavior. It packages the existing Baron 3 engine so a new GitHub reader can
understand, install, trust, and compare it quickly.

## Decisions

- Keep README focused on four normal-user commands.
- Move deep automation explanation into docs instead of command dumping.
- Compare against `repository-harness` honestly: it is simpler; Baron is deeper.
- Treat GitHub `releases/latest` as a release workflow outcome, not a Cargo
  version claim.

## Progress

- 2026-06-19 - Added RED tests for public-trust docs/status metadata.
- 2026-06-19 - Rewrote README as a concise public landing page.
- 2026-06-19 - Added public demo, repository-harness comparison, and
  certification snapshot docs.
- 2026-06-19 - Updated release docs, source version, status Markdown, and status
  JSON for `3.1.0`.

## Verification

- RED: `cargo test -p baron-core --test public_trust_docs` failed before docs
  and version updates because README/status were still `3.0.0` and demo docs
  did not exist.
- GREEN: `cargo test -p baron-core --test public_trust_docs` passed after the
  README, demo, comparison, certification, and status updates.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --all-targets`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- GitHub `main` push: passed at commit `7229481`.
- Git tag `v3.1.0` push: passed.
- GitHub release workflow `27839412902`: passed.
- GitHub `releases/latest`: passed; latest is `v3.1.0` with Windows, Linux,
  Intel macOS, Apple Silicon macOS, installer scripts, manifest, and checksums.
- Windows install smoke from `releases/latest`: passed with isolated
  `BARON_HOME`; `baron --version`, `baron setup --vault`, `baron init --codex
  --fullstack`, and `baron context --codex` all ran successfully.
