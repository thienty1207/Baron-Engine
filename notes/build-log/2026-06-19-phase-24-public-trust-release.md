# Phase 24 Public Trust Release Build Log

Date: 2026-06-19

## Scope

Ship Baron `3.1.2` as a public-trust installer UX release.

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
  JSON for `3.1.2`.
- 2026-06-20 - Removed external harness repository comparison docs and added a
  regression test blocking those references from public files.
- 2026-06-21 - Fixed Windows installer PATH refresh so a one-block copy-paste
  install can run `baron --version` in the same PowerShell session.

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
- Windows installer same-session RED/GREEN: `cargo test -p baron-cli --test lifecycle_scripts powershell_installer_makes_baron_available_in_the_current_session` failed before the installer refreshed process PATH, then passed after the fix.
- Baron 3.1.2 installer UX fix full `cargo fmt --all -- --check`: passed.
- Baron 3.1.2 installer UX fix full `cargo test --workspace --all-targets`: passed.
- Baron 3.1.2 installer UX fix full `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Baron 3.1.2 installer UX fix stale-version scan, status JSON parse, and `git diff --check`: passed.
- GitHub `main` push: pending for `v3.1.2`.
- Git tag `v3.1.2` push: pending.
- GitHub release workflow: pending.
- GitHub `releases/latest`: pending for `v3.1.2`.
- Windows install smoke from `releases/latest`: pending for `v3.1.2`.
