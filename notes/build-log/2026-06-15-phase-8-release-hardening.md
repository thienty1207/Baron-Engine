# Phase 8 Build Log - Release Hardening

Date: 2026-06-15

## Objective

Release Baron `1.0.0` as native, checksum-verified binaries with safe
install/update/rollback/uninstall behavior and cross-platform proof.

## Current State

- Branch: `codex/phase-8-release-hardening`
- Worktree: `C:\Users\Ho Thien Ty\.config\superpowers\worktrees\Baron-Engine\phase-8-release-hardening`
- Status: completed, merged to `main`, tagged `v1.0.0`, and published.

## Baseline Proof

- `cargo test --workspace --all-targets`: passed
- `cargo fmt --all -- --check`: passed
- main commit: `4108c44 docs: complete Baron phase 7 capability registry`

## Decisions

- Workspace version is the only Baron release version.
- Four native artifacts cover Windows x64, Linux x64, macOS x64, and macOS
  arm64.
- Native GitHub runners provide platform proof; Baron does not label a
  cross-compiled artifact as native-tested.
- Installers verify SHA-256 before replacing the active binary.
- Installer rollback data is separate from project and Vault data.
- Phase 8 stays below 100% until hosted platform and release-asset proof pass.

## Implemented

- workspace version `1.0.0` and `baron --version`
- stable four-target archive contract
- Rust SHA-256 and release manifest generation/verification
- hidden maintainer release metadata commands
- checksum-verifying PowerShell and POSIX installers
- install, update, rollback, uninstall, offline source, and mirror support
- Windows, Linux, Intel macOS, and Apple Silicon macOS CI matrix
- tag/version validation and GitHub Release workflow
- fresh, old, large-repo, shared-Vault, multi-adapter, and degradation smoke
- install and release documentation

## Local Verification

- `cargo test --workspace --all-targets`: passed
- `cargo fmt --all -- --check`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: passed
- Windows installer install/update/rollback/uninstall: passed
- release manifest/checksum tamper regression: passed
- 2,000-file repository bounded survey/context smoke: passed
- shared-Vault cross-project isolation smoke: passed
- Codex/Claude/generic adapter smoke: passed
- optional capability degradation smoke: passed
- `docs/BARON_STATUS.json` parse: passed
- `git diff --check`: passed
- PowerShell parser check: passed
- local POSIX shell parser: unavailable because this Windows machine has no WSL
  distribution; the hosted Linux/macOS matrix is the required proof.

## Exact Resume Point

No Phase 8 implementation remains. Maintain `v1.0.0`, triage verified release
feedback, and create a new phase plan before changing released contracts.

## Hosted Runner Findings

- First CI run `27534169756`:
  - Linux x64: passed
  - macOS Apple Silicon: passed
  - Format and Clippy: passed
  - Windows x64: failed because the newest minimal Windows Server runner did
    not expose the `Get-FileHash` cmdlet
  - macOS Intel: still running when the Windows fix was prepared
- Fix: PowerShell installer now computes SHA-256 directly with
  `System.Security.Cryptography.SHA256`, removing the cmdlet dependency.
- Local installer lifecycle and PowerShell parser checks pass after the fix.
- Branch CI run `27534373035`: Windows x64, Linux x64, Intel macOS, Apple
  Silicon macOS, format, and Clippy all passed.
- Main CI run `27534650603`: passed after merge.
- Release workflow `27534923497`: version check, four native builds, asset
  verification, and GitHub Release publication passed.
- Published release:
  `https://github.com/thienty1207/Baron-Engine/releases/tag/v1.0.0`
- Eight release assets were verified: four native archives, two installers,
  `release-manifest.json`, and `SHA256SUMS`.
- The published PowerShell installer passed isolated install, update, rollback,
  and uninstall lifecycle verification without touching project or Vault data.
