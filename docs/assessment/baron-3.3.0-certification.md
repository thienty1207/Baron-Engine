# Baron 3.3.0 Source Certification

Date: 2026-07-14
Release type: verified source release

## What Baron 3.3 Certifies

- Request Authority separates inspection from authorized change work before durable automation state is written.
- Coherent State checks project config, local Vault binding, capsule schema, and project identity without silently repairing a mismatch.
- Completion Integrity treats edited `completed` text as untrusted unless verification, risk-appropriate proof, and a passing trace still exist.
- Memory and session query paths open existing SQLite indexes read-only and ask for an explicit rebuild when schema is incompatible.
- Release promotion binds every native asset to the requested version and exact source revision before any tag is created.

## Verification Evidence

| Evidence | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo test --workspace --all-targets` | passed with no skipped tests |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo build --release --locked -p baron-cli` | passed; binary reports `baron 3.3.0` |
| Installer lifecycle test | passed install, update, rollback, uninstall, same-terminal PATH, and checksum contracts |
| Real temporary project and Vault smoke | passed Codex/fullstack init, context, authority, plan, proof, trace, completion integrity, update, and custom asset preservation |
| Status JSON, release YAML, and static scans | passed |

Focused RED/GREEN evidence already covers authority classification, no-write
ambiguous/read-only handling, identity mismatch rejection, read-only cache
preservation, completion tampering, release identity mismatch, and proof-before-tag
workflow ordering.

## Release Boundary

- Source push to `origin/main`: pending final review, merge, and push.
- Binary GitHub Release: not published by this implementation task.
- The release workflow accepts a full commit SHA already on `main`, verifies all
  supported native builds and installer lifecycle, then creates the annotated
  tag and GitHub Release as the final promotion step.
- Existing tags and Releases are refused; published assets are never replaced.
