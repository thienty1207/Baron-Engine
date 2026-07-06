# Memory Index Duplicate Record Fix Build Log

Date: 2026-07-06
Target: Baron 3.1.3
Status: source verified; release pending

## Trigger

`baron init --codex --fullstack` in `D:\work\IT\Tools\scanjob` failed with:

```text
error: UNIQUE constraint failed: records.id
```

## Evidence

- Installed Baron version: `3.1.2`.
- The failure reproduces consistently against the configured shared Vault.
- The conflicting records come from imported Codex session Markdown that
  repeats identical excerpts under the same heading.
- SQLite is failing after `parse_source` emits duplicate content-derived IDs
  for one source file.

## Current Checkpoint

- Root cause: confirmed.
- RED regression: failed with the expected SQLite primary-key error.
- GREEN regression: passed after source-level duplicate suppression.
- Full Vault Memory tests: passed, 13 tests.
- Full workspace tests: passed.
- Formatting and Clippy with warnings denied: passed.
- Real shared-Vault index: passed with 31 sources and 545 records.
- Real `scanjob` init: passed.
- Vault Markdown SHA-256 before and after indexing: unchanged.
- External harness review: complete and isolated from the bug fix.
- Review recommendation: consider a shared-understanding gate and actionable
  needs-attention recovery in a separate approved phase; do not add a mandatory
  dashboard, local runner, or provider-specific unlimited execution loop.
- Next action: commit, push, tag `v3.1.3`, run the release workflow, and smoke
  the installed latest binary against the real project/Vault.
