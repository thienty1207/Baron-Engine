# Current Baron Plan State

Last updated: 2026-06-08

## Current Focus

- Phase: 1 - Survey Engine
- Goal: implement read-only repo survey, Project Atlas stdout, JSON stdout, and no-write shadow init previews.
- Status: completed
- Verification: `cargo fmt --all`, `cargo test`, `baron survey .`, `baron survey . --json`, `baron init . --codex --shadow`, and `git diff --check` passed
- Commit message: `feat: add Baron survey engine`

## Rules

- Do not build the full engine before the spec and roadmap are clear.
- Keep build notes updated during long work.
- Do not start phase 2 until Phase 1 is committed and reviewed.
