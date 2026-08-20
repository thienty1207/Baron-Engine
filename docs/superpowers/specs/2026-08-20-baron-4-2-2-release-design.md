# Baron 4.2.2 Release Design

Baron `4.2.2` is the public patch release for the completed multi-agent core
parity correction. It packages the existing shared `assets/core` behavior so
Codex, Claude, Reasonix, and generic adapters receive one Baron engine and one
project/Vault history while retaining native adapter views.

## Scope

- Bump the workspace and release identity from `4.2.1` to `4.2.2`.
- Publish the already-tested Reasonix shared-core materialization and
  Codex/Reasonix/Claude/generic switching behavior.
- Synchronize current README, changelog, release guide, status Markdown/JSON,
  certification target, Cargo lock metadata, and build log.
- Run local formatting, tests, Clippy, release build, metadata, and binary
  smoke gates before publishing.
- Push the exact source commit to `origin/main`, tag `v4.2.2`, and verify the
  immutable GitHub Release and `releases/latest`.

## Non-goals

- No new memory algorithm, model, Wiki, CodeGraph, fallback, or Vault schema.
- No overwrite of user-owned adapter files or project/Vault data.
- No rewriting historical `4.2.1` release records; they remain evidence for
  the previous public patch.

## Release gates

1. Cargo workspace version, lockfile, certification target, and current public
   metadata all resolve to `4.2.2`.
2. The complete Reasonix parity and cross-adapter round-trip tests pass.
3. Formatting, workspace tests, warnings-denied Clippy, locked release build,
   release metadata verification, and version smoke pass locally or are
   explicitly recorded as Windows environment-only exceptions.
4. The source commit is pushed to `origin/main` before `v4.2.2` is tagged.
5. GitHub Actions builds all four native targets, creates checksums/installers,
   and publishes an immutable `v4.2.2` Release without replacing an existing
   tag or release.
