# Baron Engine

Baron is a Rust-first memory and harness engine for coding agents.

It turns an existing software repository into an agent-ready workspace for
Codex, Claude, Cursor-style agents, and other tools without making the user
learn a long command list.

Current version: `3.1.1`.

## What Baron Does

Baron helps an AI answer the questions that matter before it edits code:

- What project is this?
- What should be read first?
- What memory belongs to this project, and what memory is global?
- What task is active, what proof is required, and what trace should be left?
- Which skills, agents, and tools are safe to use for this work?

Under the hood, Baron combines a repository survey, Vault-backed memory, memory
firewall, context compiler, active plan state, Product Harness, proof gates,
trace quality, session replay, safe runtime policy, and strict skill/agent
routing. The normal user flow stays small.

## Quick Start

### 1. Install

Windows PowerShell:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
& $installer
baron --version
```

Linux or macOS:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh | sh
baron --version
```

### 2. Set The Vault

Stand inside the folder you want to use as Baron's long-term memory Vault:

```powershell
cd D:\work\AgentMemory
baron setup --vault
```

Or pass the Vault path directly:

```powershell
baron setup --vault "D:\work\AgentMemory"
```

### 3. Initialize A Project

Stand inside the project and choose the agent surface plus project focus:

```bash
baron init --codex --fullstack
baron init --claude --backend
baron init --agent --tool
```

Supported focus flags include `--frontend`, `--backend`, `--fullstack`,
`--mobile`, `--desktop`, `--tool`, `--library`, `--data`, and `--cloud`.

### 4. Update Later

```bash
baron update
```

## What The AI Runs Automatically

After init, the user normally does not run the deep engine commands by hand.
Baron installs adapter instructions and supported hooks so the AI can load
bounded context, route skills, check memory, track active work, record proof,
score traces, preserve continuity, and avoid unsafe completion claims when the
task requires it.

The full advanced command surface is documented in
[docs/architecture/COMMAND_SURFACE.md](docs/architecture/COMMAND_SURFACE.md).

## Demo

Read the public walkthrough:
[docs/demo/README.md](docs/demo/README.md).

It shows a simulated 10-year repository before and after Baron is installed,
including Codex, Claude, and generic-agent flows, project memory isolation,
proof gates, trace output, session replay, and safe runtime backend checks.

## Public Proof

- [Baron 3 public certification](docs/assessment/baron-3-public-certification.md)
  records the test, Clippy, smoke, install, shared Vault, and migration evidence
  used for the public-trust release.
- [Demo walkthrough](docs/demo/README.md) shows the public 10-year-repo flow
  without comparing Baron to another project.
- [Release guide](docs/RELEASE.md) documents install, update, rollback,
  checksum verification, and how `releases/latest` is produced.

## Source Of Truth

- Vault Markdown is the durable memory.
- SQLite and cache files are rebuildable accelerators.
- Superpowers remains the workflow core.
- The three mandatory quality gates are `code-reviewer`,
  `security-auditor`, and `test-engineer`.
- Agent-specific files are adapters, not separate brains.

## Release Safety

The installer verifies downloads against `SHA256SUMS` before replacing the
binary. Update keeps a rollback binary. Uninstall removes only Baron itself and
does not delete project files, adapters, `.baron/`, or Vault Markdown.

For current implementation status, read
[docs/BARON_STATUS.md](docs/BARON_STATUS.md).
