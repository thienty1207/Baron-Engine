# Baron 3.1.1 Public Trust Certification

This snapshot records the public-trust evidence for Baron 3.1.1.

The goal of this release is not to add more core engine behavior. The goal is
to make the existing Baron 3 engine easier to inspect, install, explain, and
compare publicly.

## Certification Scope

- README is short enough for a new user to understand the normal flow.
- Demo docs show how a 10-year repo changes after Baron is installed.
- Public docs explain Baron through Baron-owned demo and certification evidence.
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

Baron 3.1.1 source certification passed locally on 2026-06-19:

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- public-trust README/demo/certification tests
- installer lifecycle smoke inside the release smoke test suite
- shared Vault, migration, context, proof, trace, automation, and memory tests

GitHub release evidence passed on 2026-06-20:

- `main` was pushed at commit `6fa83a5`.
- tag `v3.1.1` was pushed.
- GitHub release workflow `27841880658` completed successfully.
- GitHub main CI `27841874570` completed successfully.
- `releases/latest` points at `v3.1.1`.
- The release contains Windows, Linux, Intel macOS, Apple Silicon macOS,
  `install.ps1`, `install.sh`, `release-manifest.json`, and `SHA256SUMS`.
- Windows install smoke from `releases/latest` passed with isolated
  `BARON_HOME`: `baron --version`, `baron setup --vault`, `baron init --codex
  --fullstack`, and `baron context --codex` ran successfully.

## Interpretation

Passing this certification means Baron is public-release ready. It does not mean
Baron is a desktop app, chat gateway, hosted service, or Hermes-style agent
platform. Baron remains a local Rust engine for repository memory, harness,
proof, and adapter discipline.
