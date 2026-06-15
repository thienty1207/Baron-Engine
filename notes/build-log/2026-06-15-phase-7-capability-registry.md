# Phase 7 Build Log - Baron Capability Registry

Date: 2026-06-15
Status: implementation_in_progress

## Goal

Make Baron capability-aware without confusing provider presence with execution
evidence.

## Approved Decisions

- `.baron/capabilities.toml` is the committed registry.
- `.baron/cache/capability-state.json` is rebuildable machine state.
- Supported kinds are CLI, binary, MCP, skill, HTTP, and agent adapter.
- Context reads only a bounded capability summary.
- Adapter startup contracts automatically refresh provider presence.
- Required missing providers weaken proof and trace.
- Provider presence never proves that a task verification ran.

## Baseline

- Branch: `codex/phase-7-capability-registry`
- Worktree: `C:\Users\Ho Thien Ty\.config\superpowers\worktrees\Baron-Engine\phase-7-capability-registry`
- `cargo build --workspace`: passed
- `cargo test --workspace --all-targets`: passed

## Current Step

Task 1: record the Phase 7 design, implementation plan, status, and recovery
checkpoint.

## Next Step

Write failing core registry tests before adding production code.

