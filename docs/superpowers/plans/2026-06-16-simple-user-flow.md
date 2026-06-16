# Simple User Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Make Baron feel as simple as Agent Bootstrap for normal users: install, setup Vault, init agent adapter, choose project platform.

**Architecture:** Keep the existing advanced commands for AI automation and diagnostics, but hide them from the main README flow. Add a machine-level Vault setup config so `baron init --codex` works after one `baron setup --vault`. Store platform focus in `.baron/project.toml` and surface it in context/adapter guidance so AI agents focus knowledge without adding conflicting skills.

**Tech Stack:** Rust CLI with Clap, TOML config, existing Baron Vault Markdown and adapter asset generation.

---

### Task 1: Test The Simple User Flow

**Files:**
- Modify: `crates/baron-cli/tests/adapter_cli.rs`
- Modify: `crates/baron-core/tests/config.rs`
- Modify: `crates/baron-cli/tests/cli.rs`

- [x] Add CLI tests for `baron setup --vault` using the current directory as Vault.
- [x] Add CLI tests for `baron init --codex` using the machine Vault created by setup.
- [x] Add CLI tests for `baron init --fullstack` updating platform on an existing project.
- [x] Add CLI tests for `baron init --codex --fullstack` setting adapter and platform in one command.
- [x] Run targeted tests and confirm they fail because the commands/options do not exist yet.

### Task 2: Implement Machine Vault Setup

**Files:**
- Modify: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/vault.rs`
- Modify: `crates/baron-cli/src/main.rs`

- [x] Add machine config at `~/.baron/config.toml`.
- [x] Add `baron setup --vault [path]`, defaulting to current directory when no path is given.
- [x] Create root Vault scaffold without requiring a project capsule.
- [x] Update Vault resolution so init can use the machine Vault after setup.

### Task 3: Implement Platform Focus

**Files:**
- Modify: `crates/baron-core/src/config.rs`
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-cli/src/main.rs`

- [x] Add platform enum values: frontend, backend, fullstack, mobile, desktop, tool, library, data, cloud, unknown.
- [x] Store optional platform focus in `.baron/project.toml`.
- [x] Add platform flags to `baron init`.
- [x] Let platform-only init update an existing project without reinstalling adapters.
- [x] Surface platform focus in context and adapter instructions.

### Task 4: Rewrite README Around Four User Actions

**Files:**
- Modify: `README.md`
- Modify: `docs/architecture/COMMAND_SURFACE.md`
- Modify: `AGENTS.md`

- [x] Make README main flow: install, setup Vault, init adapter, choose platform.
- [x] Move advanced command emphasis to docs, not README.
- [x] Keep wording generic so future skills/agents/rules do not conflict.

### Task 5: Verify And Release

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [x] Bump version for the new public user flow.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run targeted tests.
- [x] Run `cargo test --workspace --all-targets`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run smoke commands for setup/init/platform/context.
- [x] Commit, tag, and push the 2.1.0 source release; native GitHub assets remain workflow/operator follow-up.
