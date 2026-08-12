# Baron 3.8 Certification Record

Date: 2026-08-12  
Target: `3.8.0`  
Status: publicly released and verified

## Scope

This record covers the Baron 3.8 memory, knowledge, security, reverse-analysis,
documentation, and public-release work. It is deliberately evidence-first:
source presence, configured providers, or a workflow start are not release
proof by themselves.

## Local evidence

The following checks passed on the Windows workspace before publication:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --all-targets` (GitHub Actions run #20)
- `cargo test -p baron-core --test control_plane -- --nocapture`
- `cargo test -p baron-core knowledge --lib -- --nocapture`
- `cargo test -p baron-adapters --test adapter_lifecycle -- --nocapture`
- `cargo build --release --locked -p baron-cli`
- `target/release/baron.exe --version` returned `baron 3.8.0`
- Release-binary smoke passed for `memory resume`, `knowledge benchmark`,
  `wiki index/search`, and `knowledge codegraph-index/codegraph-query`.
- `docs/BARON_STATUS.json` parses with 65 phase records and Phase 53-64 are
  completed; the Baron 3.8 Markdown section has zero unchecked task/exit boxes.

The full workspace suite and native matrix passed in the hosted release
workflow; the local Windows all-target invocation exceeded the local command
window without emitting a failure and is not used as the authoritative result.

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

The exact source was pushed and the tag-triggered workflow completed
successfully:

- Repository: `https://github.com/thienty1207/Baron-Engine`
- Branch/source SHA: `main` / `57c5d94a5459e4597bd8a63bb1a6e65f7c197c23`
- GitHub Actions run: [#20](https://github.com/thienty1207/Baron-Engine/actions/runs/31603251123)
- Immutable tag: [`v3.8.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v3.8.0)
- Release URL: [`Baron 3.8.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v3.8.0)
- Assets/checksums: four native archives, four raw update candidates,
  `SHA256SUMS`, `release-manifest.json`, `install.ps1`, and `install.sh`.
- Fresh Windows `releases/latest` install: `baron 3.8.0`; `setup`,
  `init --codex --fullstack`, and `context` passed in a fresh project/Vault.

## Closure rule

Phase 64 is complete: the exact remote evidence is recorded above, README and
`releases/latest` agree on `3.8.0`, the tag and main resolve to the certified
source SHA, and the documentation closure commit is pushed.
