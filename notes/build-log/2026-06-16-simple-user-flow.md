# Simple User Flow Build Log

Date: 2026-06-16

## Goal

Make Baron easier for normal users without weakening the internal AI automation engine.

Normal user commands should center on:

- install Baron
- set the machine Vault
- init the agent surface
- choose the platform focus
- update managed adapter assets

Survey, context, memory, plan, harness, proof, trace, control-plane, certification, and automation commands remain available for AI/runtime/diagnostic use, but they should not dominate the main README or top-level help.

## Decisions

- Added `baron setup --vault`, defaulting to the current folder when no path is passed.
- Stored the machine Vault path in `~/.baron/config.toml`, or `BARON_HOME/config.toml` in tests.
- Let `baron init --codex`, `baron init --claude`, and `baron init --agent` use the machine Vault after setup.
- Added project platform focus flags for frontend, backend, fullstack, mobile, desktop, tool, library, data, cloud, and unknown.
- Supported shortcut init such as `baron init --codex --fullstack`.
- Hid internal automation command groups from top-level help while keeping the commands callable.
- Simplified README so users see the small front door first.

## Verification Log

- Targeted RED tests: failed before implementation as expected.
- Targeted setup/init/platform tests: passed.
- Config tests for machine Vault and platform persistence: passed.
- Context compiler platform focus test: passed.
- Survey regression test for ignoring generated agent asset directories: passed.
- Full workspace tests with `cargo test --workspace --all-targets`: passed.
- Format check with `cargo fmt --all -- --check`: passed after formatting.
- Clippy with `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Smoke setup/init/context flow using the compiled `baron.exe`: passed.
- Smoke verified that generated `.codex/skills` content is not treated as project risky surface.
