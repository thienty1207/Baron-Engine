# Baron Phase 8 Release Hardening Design

## Goal

Ship Baron as one reproducible `1.0.0` release contract across Windows, Linux,
and macOS without changing Baron's architecture or requiring a package manager.

The release layer packages the existing Rust CLI. It does not become another
engine, memory source, adapter, or workflow core.

## Release Contract

Baron publishes native archives with deterministic names:

- `baron-v1.0.0-x86_64-pc-windows-msvc.zip`
- `baron-v1.0.0-x86_64-unknown-linux-gnu.tar.gz`
- `baron-v1.0.0-x86_64-apple-darwin.tar.gz`
- `baron-v1.0.0-aarch64-apple-darwin.tar.gz`

Every release also publishes:

- `SHA256SUMS`
- `release-manifest.json`
- `install.ps1`
- `install.sh`

The manifest records the Baron version, source revision, archive names, target
triples, SHA-256 values, and binary names. Artifact names and manifest ordering
must be stable so the release can be reproduced and audited.

## Version Source Of Truth

The workspace package version is the release version. All Baron crates inherit
it, the CLI exposes it through `baron --version`, and release automation rejects
a Git tag that does not match `v<workspace-version>`.

## Installer Safety

Installers follow the same transaction:

1. Resolve an explicit version or the latest GitHub release.
2. Select the native archive for the current operating system and CPU.
3. Download the archive and `SHA256SUMS` into a temporary directory.
4. Verify the archive checksum before extraction.
5. Run the downloaded binary's version check.
6. Move the previous binary into Baron's backup directory.
7. Atomically replace the active binary.

Update uses the same transaction. Rollback restores the newest valid backup.
Uninstall removes only the executable and installer-owned metadata; it never
deletes project files, Vault Markdown, `.baron/`, or user memory.

The scripts accept a release base URL override so tests and private mirrors can
exercise exactly the same path without contacting GitHub.

## CI And Release Automation

Normal CI runs the full workspace tests on Windows, Linux, Intel macOS, and
Apple Silicon macOS. It also runs formatting, Clippy, release-contract tests,
and the cross-platform release smoke suite.

Tag workflow:

1. Validate that `vX.Y.Z` equals the workspace version.
2. Build each native binary on its native GitHub runner.
3. Package one deterministic archive per target.
4. Combine artifacts in one release job.
5. Generate `SHA256SUMS` and `release-manifest.json`.
6. Verify every checksum and manifest entry.
7. Publish installers and all release assets to one GitHub Release.

No cross-compiled binary is presented as native proof. Each operating system
builds and smokes its own executable.

## Release Smoke Matrix

The Rust release smoke suite covers:

- a fresh project
- an old project with existing user instructions
- a large repository with bounded survey/context behavior
- two projects sharing one Vault with memory isolation
- Codex, Claude, and generic adapters together
- optional capability degradation
- required capability failure blocking false completion

This suite runs on every supported GitHub runner. Local Windows verification is
necessary but is not used as proof that macOS or Linux binaries work.

## Completion Gate

Phase 8 reaches 100% only when:

- local Rust tests, format, Clippy, release packaging, and Windows installer
  smoke pass
- the branch CI matrix passes on all supported runners
- a `v1.0.0` tag produces all named release assets
- downloaded release assets pass checksum and install lifecycle verification
- status, architecture, command, install, update, rollback, and uninstall docs
  agree with the shipped behavior

If GitHub-hosted platform proof is unavailable, Baron remains below 100% and
the status files must say exactly what is missing.
