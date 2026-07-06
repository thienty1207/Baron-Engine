# Baron 3.1.3 Memory Index Resilience

Date: 2026-07-06
Status: released; installer follow-up moved to 3.1.4

## Fixed Behavior

Baron no longer aborts memory indexing when one Markdown source contains the
same heading and excerpt more than once. Identical records are deduplicated in
the rebuildable SQLite cache while the original Vault Markdown remains intact.

## Evidence

- The regression test failed before the fix with
  `UNIQUE constraint failed: records.id`.
- The same test passed after source-level record deduplication.
- The complete Vault Memory test suite passed.
- `cargo fmt --all -- --check` passed.
- `cargo test --workspace --all-targets` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- The real shared Vault rebuilt to 545 records from 31 sources.
- `scanjob` initialized successfully with `--codex --fullstack`.
- SHA-256 proof confirmed the imported session Markdown used for the smoke test
  was unchanged before and after indexing.
- GitHub main CI `28796795403` passed.
- GitHub release workflow `28796812011` passed and published `v3.1.3`.
- Release verification exposed a pre-existing anonymous API quota dependency in
  latest-version resolution; that installer concern is isolated in 3.1.4.

## External Harness Review Decision

The current external harness updates reinforce two useful future directions:

- require explicit shared understanding before medium or high-risk execution
- provide actionable recovery packets when automated work needs attention

Baron should consider those ideas in a separate approved phase. A dedicated
dashboard, mandatory local runner, or provider-specific unlimited execution
loop should not be added to this patch because they would duplicate Baron's
IDE-compatible adapters and complicate the simple user flow.
