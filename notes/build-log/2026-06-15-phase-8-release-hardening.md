# Phase 8 Build Log - Release Hardening

Date: 2026-06-15

## Objective

Release Baron `1.0.0` as native, checksum-verified binaries with safe
install/update/rollback/uninstall behavior and cross-platform proof.

## Current State

- Branch: `codex/phase-8-release-hardening`
- Worktree: `C:\Users\Ho Thien Ty\.config\superpowers\worktrees\Baron-Engine\phase-8-release-hardening`
- Status: implementation and all local release gates pass; hosted native-runner
  proof is next.

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

Push `codex/phase-8-release-hardening`, inspect the four-platform GitHub Actions
matrix, fix any hosted-runner issue, then merge, tag `v1.0.0`, and verify the
published release assets before marking Baron 100%.
