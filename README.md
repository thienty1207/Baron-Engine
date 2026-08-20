# Baron Engine

Baron is a Rust-first memory and harness engine for coding agents. It turns an
existing repository into an agent-ready workspace for Codex, Claude, DeepSeek
Reasonix, and other agent tools while keeping the normal user flow small. All
adapters use the same Baron brain: one project identity, Vault, memory, and
session history.

Current source version: `4.2.2`.
Current public release: [`v4.2.2`](https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.2).

> **Download check:** install only from
> [`releases/latest`](https://github.com/thienty1207/Baron-Engine/releases/latest)
> and confirm that `baron --version` prints `baron 4.2.2`. If it prints an older
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

## What Baron 4.2 Adds

Baron 4.2 is the evidence-first intelligence release built on the 4.1
project firewall and the 4.0 recovery path:

- **Long-term memory:** L0 evidence through L3 project invariants are labelled
  separately from trust (`candidate`, `verified`, `contested`, `superseded`,
  `expired`, or `unknown`). Memory consolidation is read-only by default and
  writes only reviewable candidate proposals; it never silently promotes a
  model summary into truth.
- **Grounded handoff:** every new agent receives a bounded, project-bound
  Resume Brief with current work, decisions, proof, blockers, unknowns,
  affected files, and the next safe action. Sources and stale/contested labels
  remain visible.
- **Calibrated semantic retrieval:** exact/path, lexical, bilingual n-gram,
  local dense, temporal, Wiki, and CodeGraph channels are reranked only after
  project/trust eligibility. Low-confidence or negative queries abstain with a
  reason; a semantic score can never manufacture evidence.
- **Deep session learning:** sessions are split into task segments, noisy and
  duplicate events are removed, evidence spans and source hashes are retained,
  and prompt injection, destructive commands, secrets, forged output, and
  project mismatch are quarantined. All learned items remain candidate-only;
  Baron never creates a Skill from a conversation.
- **Bi-temporal truth:** facts and decisions carry observed/valid time,
  source-span lineage, supersession, expiry, conflict sets, tombstones,
  revalidation, backup, and rollback state. An as-of view never rewrites
  history into today's truth.
- **Wiki:** Markdown structure, citations, entities, typed links, freshness,
  and deletion/rename tombstones are indexed locally. Queries can follow a
  bounded two-hop link path without loading the whole documentation tree into
  the prompt.
- **CodeGraph:** the default local graph covers Rust, TypeScript, JavaScript,
  Python, and Go symbols, imports, references, calls, tests, source spans,
  directional relation confidence, impact paths, and deletion tombstones. The
  graph is project-isolated, disposable, and rebuilt from source when stale.
- **Bounded impact analysis:** Wiki and CodeGraph queries return source-linked,
  bounded results with relation and impact evidence instead of injecting the
  whole repository into the agent context.
- **Security:** `vibe-security-scan` keeps source AppSec ownership while
  defensive reverse-analysis routes cover static binary/APK/malware triage.
  Offensive or destructive requests, missing authorization, scope mismatch,
  path escape, and network escape fail closed. Baron does not execute samples or
  download security tools automatically.
- **Safe fallback:** 4.2 is the normal guarded path. A failed trust, temporal,
  cache, parser, identity, budget, or grounding gate returns `unknown` or the
  Baron 4.0 result for that query. Set `BARON_ENGINE_GENERATION=4.1` for the
  verified whole-engine 4.1 rollback, or `BARON_ENGINE_GENERATION=4.0` for
  the guarded per-query baseline. `3.8`/`baseline` remains the older recovery
  switch.

The 4.2 surfaces are local and deterministic; no paid embedding account,
network request, or model service is required for normal indexing and recall.
Optional providers remain lazy and degrade to the bounded local path.
Tencent comparison is not a Baron release gate.

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

The expected output is `baron 4.2.2`. The installers verify SHA-256 checksums
and the staged binary version before replacing an existing Baron executable.

The verified public release is [`v4.2.2`](https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.2).
Its native CI and release gates pass on Windows x64, Linux x64, Intel macOS,
and Apple Silicon; the immutable Release includes the archives, raw update
candidates, both installers, `release-manifest.json`, and `SHA256SUMS`. Baron
4.1 remains the whole-engine rollback path and Baron 4.0 remains the explicit
safe fallback. The exact source commit and workflow evidence are recorded in
the current [build status](docs/BARON_STATUS.md).

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
baron init --reasonix --fullstack
```

Supported focus flags include `--frontend`, `--backend`, `--fullstack`,
`--mobile`, `--desktop`, `--tool`, `--library`, `--data`, and `--cloud`.

### Switch between agent tools without splitting memory

Reasonix is a maintenance adapter on the 4.2 engine, packaged in the 4.2.2
release. It changes the agent
surface only: the project ID, Vault, memory, session history, Wiki, and
CodeGraph stay shared with Codex and Claude. Register another adapter once.
For daily switching, Baron finds the current project from the working
directory and keeps the long adapter commands out of the normal workflow:

```powershell
baron init --codex --fullstack
baron init --reasonix
baron --reasonix
baron --codex
```

The explicit diagnostics/preview commands remain available when a script or
troubleshooting session needs them:

```powershell
baron adapter status
baron adapter switch --to reasonix --dry-run
```

Reasonix receives the same Baron-managed core as Codex: the complete embedded
skill tree, the three mandatory quality agents plus optional agent contracts,
and their routing indexes are materialized under `.reasonix/skills` and
`.reasonix/agents`. Only the bridge files and hook format are Reasonix-specific;
the engine, project ID, Vault, memory, session history, Wiki, CodeGraph, plan,
proof, trace, and continuity state remain shared.

Reasonix installation is preserve-first. Existing unmarked `REASONIX.md`,
`.reasonix/INDEX.md`, skill/agent files, settings, and command files are never
silently overwritten; Baron reports preserved paths and conflicts for review.
Missing Baron-managed core assets can be restored by `baron --reasonix` or the
normal Baron local-reconciliation flow. The intelligence engine remains the Baron 4.2
engine; this maintenance correction is published as `4.2.2` and does not create
a `v4.3` release or change the memory engine.

### 4. Update later

```bash
baron update
```

Baron verifies the official release, refreshes only Baron-managed project
files, and keeps a recoverable transaction if a local edit needs review. It
never overwrites project source, Vault Markdown, custom skills, or custom
agents. On Windows, a verified binary replacement may finish after the current
process exits; open a new terminal before checking the version.

### Verify the 4.2 intelligence path

The normal commands use 4.2. To inspect the release gate without exposing
private sessions, run the local correctness contract:

```powershell
baron intelligence benchmark42 . --vault "D:\work\AgentMemory"
```

For a private owner-supplied holdout, keep the holdout directory outside the
repository and Vault, then pass it explicitly with `--holdout`. The runner
opens a holdout once, records hashes and every case, and never copies labels
into the Vault. `BARON_ENGINE_GENERATION=4.1` selects the whole-engine
rollback; `BARON_ENGINE_GENERATION=4.0` selects the legacy per-query fallback.

## Reinstall Windows safely

Before reinstalling Windows, copy these two things somewhere safe:

- your Vault folder, for example `D:\work\AgentMemory`;
- every project folder that uses Baron, including its hidden `.baron` folder.

After Windows is installed again, restore those folders and run:

1. the Windows install block above and confirm `baron --version` prints
  `baron 4.2.2`;
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

- [Baron 4.2 benchmark](docs/assessment/baron-4.2-benchmark.md) records the
  correctness contract, calibrated retrieval, task-segmented session learning,
  temporal conflict handling, Wiki citations, CodeGraph direction, and raw
  fallback behavior. The raw candidate score never counts a fallback result.
- [Baron 4.2 acceptance](docs/assessment/baron-4.2-acceptance.json) records
  three reproducible release-profile runs and the private sealed holdout
  result. Holdout labels stay outside Git and runtime indexes.
- [Baron 4.1 benchmark](docs/assessment/baron-4.1-benchmark.md) remains the
  whole-engine rollback evidence; Baron 4.0 remains the per-query recovery
  evidence.
- [Baron 4.1 Phase 86 acceptance](docs/assessment/baron-4.1-phase86-runner.md)
  records repeated release-binary runs, project isolation, and the exact
  internal acceptance result.

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
current `v4.2.2` install target.

## Source of truth and safety

- Vault Markdown is durable memory; caches can be deleted and rebuilt.
- `.baron/project.toml` stores project routing, never memory; local machine
  Vault routing stays in ignored `.baron/local.toml`.
- The memory firewall uses project identity rather than a folder name.
- Security analysis is defensive, bounded, and authorization-aware. Baron does
  not provide unrestricted offensive automation.
- Uninstall removes Baron itself and install metadata only; it does not delete
  project files, adapters, `.baron/`, or Vault Markdown.
