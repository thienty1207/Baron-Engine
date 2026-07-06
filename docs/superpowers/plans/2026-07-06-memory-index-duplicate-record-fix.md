# Baron 3.1.3 Memory Index Duplicate Record Fix

Date: 2026-07-06
Status: source verified; release pending

## Goal

Make `baron init` resilient when imported session Markdown contains repeated
lines that currently generate the same SQLite `records.id`.

## Root Cause

- Memory record IDs are derived from source path, scope, project identity,
  heading, and excerpt.
- Imported session notes can legitimately repeat the same excerpt under the
  same heading.
- `parse_source` emits both records, then `replace_source` inserts the same
  primary key twice and aborts the index transaction.

## Plan

1. Add a regression test with repeated imported-session excerpts and observe
   the current UNIQUE constraint failure.
2. Deduplicate identical records inside each Markdown source before SQLite
   insertion without deleting or rewriting Vault Markdown.
3. Verify the targeted test, full workspace, Clippy, real `scanjob` init, and
   memory index rebuild.
4. Review the latest public external harness changes separately and record
   only evidence-backed recommendations; do not vendor or copy its runtime.
5. Publish a patch release only after verification passes.

## Verification Record

- RED duplicate-record regression: passed by failing for the expected reason.
- GREEN duplicate-record regression: passed.
- Full Vault Memory suite: passed.
- Full workspace tests, formatting, and Clippy: passed.
- Real shared-Vault index and `scanjob` init: passed.
- Vault Markdown content hash remained unchanged.

## Non-Negotiables

- Vault Markdown remains untouched and remains the source of truth.
- SQLite stays a disposable cache.
- Duplicate content must not crash init.
- The fix must not weaken project identity or Memory Firewall isolation.
