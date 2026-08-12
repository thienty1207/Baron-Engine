# Baron 3.8 Certification Record

Date: 2026-08-12  
Target: `3.8.0`  
Status: release verification in progress

## Scope

This record covers the Baron 3.8 memory, knowledge, security, reverse-analysis,
documentation, and public-release work. It is deliberately evidence-first:
source presence, configured providers, or a workflow start are not release
proof by themselves.

## Local evidence

The following checks passed on the Windows workspace before publication:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p baron-core --test control_plane -- --nocapture`
- `cargo test -p baron-core knowledge --lib -- --nocapture`
- `cargo test -p baron-adapters --test adapter_lifecycle -- --nocapture`
- `cargo build --release --locked -p baron-cli`
- `target/release/baron.exe --version` returned `baron 3.8.0`
- Release-binary smoke passed for `memory resume`, `knowledge benchmark`,
  `wiki index/search`, and `knowledge codegraph-index/codegraph-query`.
- `docs/BARON_STATUS.json` parses with 65 phase records and Phase 53-64 are
  completed; the Baron 3.8 Markdown section has zero unchecked task/exit boxes.

The full Windows `cargo test -p baron-core --all-targets --no-fail-fast`
invocation exceeded the local 120-second command window without emitting a
failure. It is not counted as a local pass; the hosted release workflow is the
authoritative full-suite and native-matrix gate.

## Security and asset evidence

- `vibe-security-scan` small and large workflows are read-only by default,
  bounded, path-scoped, and do not create implicit report/temp directories or
  use raw recursive deletion.
- Reverse analysis is three lazy, Baron-owned, defensive assets:
  `binary-reverse-analysis`, `apk-mobile-analysis`, and `malware-triage`.
- Control Plane routes reverse tasks narrowly while preserving
  `vibe-security-scan`, `security-auditor`, and the other two core quality gates.
- Wiki and CodeGraph data stay in disposable `.baron/cache/` files; Vault
  Markdown and repository source remain authoritative.

## Public release evidence

The following fields are filled only after the exact source is pushed and the
tag-triggered workflow completes successfully:

- Repository: `https://github.com/thienty1207/Baron-Engine`
- Branch/source SHA: `pending publication`
- GitHub Actions run: `pending publication`
- Immutable tag: `v3.8.0` (pending publication)
- Release URL: `pending publication`
- Asset inventory/checksums: `pending publication`
- Fresh Windows `releases/latest` install: `pending publication`

## Closure rule

Phase 64 is not complete until the pending fields above contain the exact
remote evidence, README and `releases/latest` agree on `3.8.0`, the local and
remote SHA match, and the final commit is pushed without an unrecorded change.
