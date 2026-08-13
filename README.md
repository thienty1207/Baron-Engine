# Baron Engine

Baron is a Rust-first memory and harness engine for coding agents. It turns an
existing repository into an agent-ready workspace for Codex, Claude, and other
agent tools while keeping the normal user flow small.

Current source version: `4.0.0`.
Public promotion: `v4.0.0` release workflow in progress; the last stable,
downloadable release is [`v3.8.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v3.8.0)
until the immutable 4.0.0 Release is published.

> **Download check:** after the 4.0.0 Release workflow completes, install only
> from [`releases/latest`](https://github.com/thienty1207/Baron-Engine/releases/latest)
> and confirm that `baron --version` prints `baron 4.0.0`. If it prints an older
> version, stop and refresh the
> [Releases page](https://github.com/thienty1207/Baron-Engine/releases).

## What Baron Does

Baron helps an AI answer the questions that matter before it edits code:

- What project is this, and which memory belongs to it?
- What is being built, where did work stop, and what decision is current?
- Which source, proof, blocker, unknown, and next safe action should be carried
  into a new agent session?
- Which Wiki pages, symbols, imports, calls, and impact paths are relevant?
- Which security route is safe, authorized, and evidence-backed?

Baron combines repository survey, Vault-backed memory, a project firewall,
bounded context compilation, plans, Product Harness, proof and trace gates,
session replay, safe runtime policy, and strict skill/agent routing. Superpowers
remains the workflow core, and the mandatory quality gates remain
`code-reviewer`, `security-auditor`, and `test-engineer`.

## What Baron 4.0 Adds

Baron 4.0 is the intelligence and defensive-security release:

- **Long-term memory:** L0 evidence through L3 project invariants are labelled
  separately from trust (`candidate`, `verified`, `contested`, `superseded`,
  `expired`, or `unknown`). Memory consolidation is read-only by default and
  writes only reviewable candidate proposals; it never silently promotes a
  model summary into truth.
- **Grounded handoff:** every new agent receives a bounded, project-bound
  Resume Brief with current work, decisions, proof, blockers, unknowns,
  affected files, and the next safe action. Sources and stale/contested labels
  remain visible.
- **Wiki:** Markdown structure, citations, freshness, and explicit links are
  indexed locally. Queries can follow a bounded two-hop link path without
  loading the whole documentation tree into the prompt.
- **CodeGraph:** the default local graph covers Rust, TypeScript, JavaScript,
  Python, and Go symbols, imports, references, calls, and source spans. The
  graph is project-isolated, disposable, and rebuilt from source when stale.
- **Security:** `vibe-security-scan` keeps source AppSec ownership while
  defensive reverse-analysis routes cover static binary/APK/malware triage.
  Offensive or destructive requests, missing authorization, scope mismatch,
  path escape, and network escape fail closed. Baron does not execute samples or
  download security tools automatically.
- **Safe fallback:** 4.0 is the normal guarded path. If a candidate build,
  cache, project identity, or structural check fails, Baron falls back to the
  proven 3.8 result. To force the baseline during incident recovery, set
  `BARON_ENGINE_GENERATION=3.8` (or `baseline`).

The 4.0 surfaces are local and deterministic; no paid embedding account,
network request, or model service is required for normal indexing and recall.
Optional providers remain lazy and degrade to the bounded local path.

## Quick Start

### 1. Install

Windows PowerShell:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
powershell.exe -NoProfile -ExecutionPolicy Bypass -File $installer
baron --version
```

Linux or macOS:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh | sh
baron --version
```

After the public promotion completes, the expected output is `baron 4.0.0`.
During promotion, `releases/latest` may still return the previous stable
binary. The installers verify SHA-256 checksums and the staged binary version
before replacing an existing Baron executable.

### 2. Set the Vault

Stand inside the folder you want to use as Baron's long-term memory Vault:

```powershell
cd D:\work\AgentMemory
baron setup --vault
```

Or pass the Vault path directly:

```powershell
baron setup --vault "D:\work\AgentMemory"
```

Vault Markdown is the durable source of truth. SQLite, search indexes, Wiki,
and CodeGraph caches are rebuildable accelerators.

### 3. Initialize a project

Stand inside the project and choose the agent surface plus project focus:

```bash
baron init --codex --fullstack
baron init --claude --backend
baron init --agent --tool
```

Supported focus flags include `--frontend`, `--backend`, `--fullstack`,
`--mobile`, `--desktop`, `--tool`, `--library`, `--data`, and `--cloud`.

### 4. Update later

```bash
baron update
```

Baron verifies the official release, refreshes only Baron-managed project
files, and keeps a recoverable transaction if a local edit needs review. It
never overwrites project source, Vault Markdown, custom skills, or custom
agents. On Windows, a verified binary replacement may finish after the current
process exits; open a new terminal before checking the version.

## Reinstall Windows safely

Before reinstalling Windows, copy these two things somewhere safe:

- your Vault folder, for example `D:\work\AgentMemory`;
- every project folder that uses Baron, including its hidden `.baron` folder.

After Windows is installed again, restore those folders and run:

1. the Windows install block above; after public promotion, confirm
   `baron --version` prints `baron 4.0.0`;
2. `baron setup --vault "D:\work\AgentMemory"`;
3. `baron update` inside each restored Baron project.

This reconnects the long-term memory and refreshes Baron-managed adapter files;
it does not erase project code or Vault memory.

## What the AI runs automatically

After `init`, users normally do not run the deep engine commands by hand.
Baron installs adapter instructions and supported hooks so the AI can load
bounded context, check capability/runtime safety, recall project memory, route
skills, track work, record proof, score traces, preserve continuity, recover
from interruption, consult the relevant Wiki/CodeGraph slices, and avoid
unsafe completion claims. Hook absence degrades to reconciliation; it never
pretends that instruction-only behavior executed.

The complete advanced command surface is documented in
[docs/architecture/COMMAND_SURFACE.md](docs/architecture/COMMAND_SURFACE.md).

## Demo

Read the public walkthrough:
[docs/demo/README.md](docs/demo/README.md). It shows a simulated long-running
repository before and after Baron is installed, including memory isolation,
proof gates, trace output, and adapter flows.

## Public Proof

- [Baron 4.0 integrated acceptance](docs/assessment/baron-4.0-certification.md)
  records the six hard checks, bounded handoff, zero leakage, security routing,
  static AppSec boundary, and cache/source identity evidence.
- [Baron 4.0 benchmark](docs/assessment/baron-4.0-benchmark.md) records the
  independent 3.8 baseline and 4.0 candidate cases for Memory, Wiki, CodeGraph,
  and security routing.
- [Release guide](docs/RELEASE.md) documents install, update, rollback,
  checksum verification, and the public `releases/latest` path.
- [Build status](docs/BARON_STATUS.md) is the durable phase dashboard.

The historical [Baron 3 public certification](docs/assessment/baron-3-public-certification.md)
and prior release records remain available for audit; they do not change the
current `v4.0.0` install target.

## Source of truth and safety

- Vault Markdown is durable memory; caches can be deleted and rebuilt.
- `.baron/project.toml` stores project routing, never memory; local machine
  Vault routing stays in ignored `.baron/local.toml`.
- The memory firewall uses project identity rather than a folder name.
- Security analysis is defensive, bounded, and authorization-aware. Baron does
  not provide unrestricted offensive automation.
- Uninstall removes Baron itself and install metadata only; it does not delete
  project files, adapters, `.baron/`, or Vault Markdown.
