# Baron Engine

Baron is a Rust-first memory and harness engine for coding agents.

It turns an existing software repository into an agent-ready workspace for
Codex, Claude, Cursor-style agents, and other tools without making the user
learn a long command list.

Current source version: `3.5.0`.

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
routing. Baron now adds deep platform profiles, a non-destructive architecture
governor, evidence-backed reviewer closure, confirmed task intent, and an
actionable recovery point so a failed or interrupted session can resume
without guessing. Baron 3.3 also separates inspection from authorized changes,
refuses incoherent project/Vault identity, and rechecks completion against real
proof instead of trusting edited status text. The normal user flow stays small.

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

Baron generates a deep profile and architecture contract from the selected
focus plus evidence found in the repository. If the product later expands,
`baron init --mobile` adds mobile as an extension; it does not replace the
original fullstack foundation or rearrange existing code.

### 4. Update Later

```bash
baron update
```

From Baron 3.4 onward, this one command checks the official release,
verifies the candidate before use, refreshes only Baron-managed project files,
and keeps a recoverable transaction if a local edit needs review. It never
overwrites an ambiguous managed edit, custom skill, custom agent, project
source file, or Vault memory. On Windows, Baron may finish replacing its own
binary after the current process exits; the command reports that plainly.

If Baron 3.3 or an older version is already installed, use the installer once
to cross the 3.4 update boundary. The short recovery instructions are in
[docs/RELEASE.md](docs/RELEASE.md).

## What The AI Runs Automatically

After init, the user normally does not run the deep engine commands by hand.
Baron installs adapter instructions and supported hooks so the AI can load
bounded context, route skills, check memory, track active work, record proof,
score traces, confirm medium/high-risk intent before implementation, preserve
continuity, record an actionable recovery path after failure or interruption,
load task-relevant platform guidance, reconcile architecture safely, preserve
reviewer findings until fix proof exists, and avoid unsafe completion claims.

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
Release tags are created only after the exact source commit, all native builds,
checksums, and installer lifecycle have passed verification.

For current implementation status, read
[docs/BARON_STATUS.md](docs/BARON_STATUS.md).
