# Phase 24 Public Trust Release Build Log

Date: 2026-06-19

## Scope

Ship Baron `3.1.1` as a public-trust cleanup release.

This phase does not add core memory, harness, proof, trace, skill, or agent
behavior. It packages the existing Baron 3 engine so a new GitHub reader can
understand, install, and trust it quickly without public comparison links to
external harness repositories.

## Decisions

- Keep README focused on four normal-user commands.
- Move deep automation explanation into docs instead of command dumping.
- Keep public proof focused on Baron-owned demo, certification, release, and
  smoke evidence.
- Treat GitHub `releases/latest` as a release workflow outcome, not a Cargo
  version claim.

## Progress

- 2026-06-19 - Added RED tests for public-trust docs/status metadata.
- 2026-06-19 - Rewrote README as a concise public landing page.
- 2026-06-19 - Added public demo and certification snapshot docs.
- 2026-06-19 - Updated release docs, source version, status Markdown, and status
  JSON for `3.1.1`.
- 2026-06-20 - Removed external harness repository comparison docs and added a
  regression test blocking those references from public files.

## Verification

- RED: `cargo test -p baron-core --test public_trust_docs` failed before docs
  and version updates because README/status were still `3.0.0` and demo docs
  did not exist.
- GREEN: `cargo test -p baron-core --test public_trust_docs` passed after the
  README, demo, certification, and status updates.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --all-targets`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- External harness reference RED/GREEN: `cargo test -p baron-core --test public_trust_docs` failed before cleanup, then passed after the comparison file and references were removed.
- Static scan for the removed external harness repo name, owner name, and old
  comparison file naming pattern: passed with no matches after cleanup.
- GitHub `main` push: passed at commit `6fa83a5`.
- Git tag `v3.1.1` push: passed.
- GitHub release workflow `27841880658`: passed.
- GitHub main CI `27841874570`: passed.
- GitHub `releases/latest`: passed; latest is `v3.1.1` with Windows, Linux,
  Intel macOS, Apple Silicon macOS, installer scripts, manifest, and checksums.
- Windows install smoke from `releases/latest`: passed with isolated
  `BARON_HOME`; `baron --version`, `baron setup --vault`, `baron init --codex
  --fullstack`, and `baron context --codex` all ran successfully.
