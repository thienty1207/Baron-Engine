# Baron Phase 8 Release Hardening Implementation Plan

**Goal:** Ship Baron `1.0.0` as checksum-verified native releases for Windows,
Linux, Intel macOS, and Apple Silicon macOS, with safe lifecycle scripts and a
cross-platform smoke matrix.

**Architecture:** Rust owns version and release metadata contracts. Native CI
runners build and smoke the CLI. Thin PowerShell and POSIX installers download
only matching archives, verify SHA-256 before replacement, and preserve one or
more rollback binaries. Vault and project data remain outside installer
ownership.

## Task 1: Lock Recovery State

- [x] Confirm Phase 7 main is clean and pushed.
- [x] Run the pre-Phase-8 full Rust baseline.
- [x] Write the Phase 8 design and implementation plan.
- [x] Mark Phase 8 `implementation_in_progress` in durable and temporary status.

## Task 2: Version And Release Metadata Contracts

- [x] Add failing tests for workspace version, CLI version, target artifact
  names, checksums, and deterministic release manifest.
- [x] Move all Baron crates to workspace version `1.0.0`.
- [x] Add `baron-core::release` models and checksum/manifest functions.
- [x] Expose `baron --version`.
- [x] Add package metadata required by release archives.

## Task 3: Checksum-Verified Lifecycle Scripts

- [x] Add failing static and lifecycle tests for PowerShell and shell installers.
- [x] Implement install/update with checksum verification before replacement.
- [x] Implement rollback from installer-owned backups.
- [x] Implement uninstall without deleting repo or Vault data.
- [x] Support explicit version, install directory, and release base URL.

## Task 4: Native CI And GitHub Release

- [x] Add CI for Windows, Linux, Intel macOS, and Apple Silicon macOS.
- [x] Add tag/version consistency checks.
- [x] Build and package on native runners.
- [x] Generate and re-verify `SHA256SUMS` and `release-manifest.json`.
- [x] Publish archives and installers as GitHub Release assets.

## Task 5: Release Smoke Matrix

- [x] Add fresh-project smoke.
- [x] Add old-project preservation smoke.
- [x] Add large-repository boundedness smoke.
- [x] Add shared-Vault memory-firewall smoke.
- [x] Add three-adapter lifecycle smoke.
- [x] Add optional capability degradation smoke and retain existing required
  capability completion-gate coverage.
- [ ] Run the matrix locally and on each GitHub-hosted operating system.

## Task 6: Documentation And Completion

- [x] Document install, update, rollback, uninstall, checksums, and data safety.
- [x] Align README, AGENTS, command surface, architecture, roadmap, and release
  notes.
- [x] Run format, full tests, Clippy, package smoke, installer smoke, and
  `git diff --check`.
- [ ] Push branch and verify the cross-platform CI matrix.
- [ ] Merge to main, tag `v1.0.0`, and verify release assets.
- [ ] Mark Phase 8 complete and overall status 100% only after all proof passes.
