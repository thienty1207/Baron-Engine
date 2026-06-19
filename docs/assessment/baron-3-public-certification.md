# Baron 3.1.0 Public Trust Certification

This snapshot records the public-trust evidence for Baron 3.1.0.

The goal of this release is not to add more core engine behavior. The goal is
to make the existing Baron 3 engine easier to inspect, install, explain, and
compare publicly.

## Certification Scope

- README is short enough for a new user to understand the normal flow.
- Demo docs show how a 10-year repo changes after Baron is installed.
- Public comparison explains Baron vs repository-harness without attacking the
  other project.
- Release docs explain how `releases/latest` is produced and verified.
- Source release version, CLI version, tests, and status files agree.

## Required Evidence

These checks are required for the public-trust release:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Public release evidence:

- GitHub release latest points to the current Baron source release after tag
  publication.
- Installer smoke verifies `baron --version`.
- Fresh Vault smoke verifies setup with `baron setup --vault`.
- Fresh project smoke verifies `baron init --codex --fullstack`.
- Context smoke verifies the agent can load the project brief after init.
- Shared Vault smoke verifies project memory isolation.
- Migration smoke verifies Agent Bootstrap dry-run/apply/rollback paths remain
  safe.

## Current Evidence Record

Baron 3.1.0 source certification passed locally on 2026-06-19:

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- public-trust README/demo/comparison/certification tests
- installer lifecycle smoke inside the release smoke test suite
- shared Vault, migration, context, proof, trace, automation, and memory tests

GitHub release evidence is still a publication step:

- push `main`
- push tag `v3.1.0`
- let the release workflow publish native assets
- verify `releases/latest` points at the new Baron release

## Interpretation

Passing this certification means Baron is public-release ready. It does not mean
Baron is a desktop app, chat gateway, hosted service, or Hermes-style agent
platform. Baron remains a local Rust engine for repository memory, harness,
proof, and adapter discipline.
