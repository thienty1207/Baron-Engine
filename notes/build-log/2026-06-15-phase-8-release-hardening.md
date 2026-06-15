# Phase 8 Build Log - Release Hardening

Date: 2026-06-15

## Objective

Release Baron `1.0.0` as native, checksum-verified binaries with safe
install/update/rollback/uninstall behavior and cross-platform proof.

## Current State

- Branch: `codex/phase-8-release-hardening`
- Worktree: `C:\Users\Ho Thien Ty\.config\superpowers\worktrees\Baron-Engine\phase-8-release-hardening`
- Status: design locked; implementation starting with failing release-contract
  tests.

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

## Exact Resume Point

Add failing release-contract tests before implementation.
