# Baron Engine

Baron is a Rust-first engine for project memory, task context, routing,
continuity, and evidence-backed work. Baron Core is the brain. Codex and Claude
are the two supported native integrations over that Core.

```text
assets/core/**
      ↓
.baron/core/**
      ↓
Codex bridge / Claude bridge
```

Baron keeps one project identity, one Vault boundary, and one task history when
the same project is opened from either supported tool. The adapters provide the
host-native files and hooks; they do not own a second workflow or memory
system.

Current source version: `5.0.0`.
Current public release: [`v5.0.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v5.0.0).

## What Baron Does

Baron prepares a bounded, evidence-aware work packet before an agent changes a
repository. Core owns:

- project identity and Vault-backed trusted memory;
- task intent, constraints, plans, Task State, and recovery;
- profile-aware routing and work-shape decisions;
- Superpowers workflow and the three required quality agents;
- proof, trace, control-plane gates, and continuity;
- optional hooks, session replay, Wiki, and CodeGraph accelerators;
- Autopilot housekeeping and reviewable learning candidates.

Vault Markdown is durable source of truth. SQLite and other caches are
rebuildable accelerators. Candidates, stale records, and host-local notes never
silently become current project truth.

## Quick Start

### Install

Windows PowerShell:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
powershell.exe -NoProfile -ExecutionPolicy Bypass -File $installer
baron --version
```

Linux:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh | sh
baron --version
```

The expected output is `baron 5.0.0`. Installers authenticate the detached
release manifest, then verify the archive checksum
and the staged binary version before replacing an existing executable. See the
[release guide](docs/RELEASE.md) for rollback and offline installation.

The v5.0.0 release publishes Windows x64 and Linux x64 artifacts. macOS is
outside this release matrix.

The official release assets are authenticated by the detached Ed25519
`ReleaseManifestV1` signature for key `baron-release-2026` (fingerprint
`73a005a12cf79f1fa60612f0e359a13c83d2660806075b610b3d83f4f14c31b4`).

### Set up the Vault

Choose a folder for long-term memory and run:

```powershell
baron setup --vault "D:\work\AgentMemory"
```

Vault Markdown stays readable and portable. The project ID, rather than a
folder basename, separates projects that share one Vault.

### Initialize Baron

Run initialization from the project folder. Choose one or both supported native
surfaces as needed:

```bash
baron init --codex --fullstack
baron init --claude --backend
```

Initialization installs one canonical `.baron/core/**` runtime and a thin
host projection. Existing user text, custom skills, agents, settings, and
third-party hooks remain outside Baron-managed markers.

### Ask for work normally

After initialization, open Codex or Claude in the project and **ask Codex or Claude**
for the work in ordinary language. Baron and the native integration
prepare context, route the smallest useful set of Core resources, resume an
interrupted task, and collect proof as work proceeds. You do not need to learn
Baron's internal orchestration commands for normal work.

## Project profiles

The profile is a routing prior. Task intent, repository evidence, work shape,
risk, and current Task State still decide what loads.

| Profile | Main focus |
| --- | --- |
| `--frontend` | UI structure, accessibility, browser behavior, visual proof |
| `--backend` | services, APIs, auth, data boundaries, observability |
| `--fullstack` | cross-layer contracts and end-to-end verification |
| `--mobile` | lifecycle, offline state, storage, permissions, API, performance |
| `--desktop` | native lifecycle, packaging, permissions, release behavior |
| `--tool` / `--library` | public interfaces, compatibility, and consumer proof |
| `--data` | analytics, pipelines, transformations, and data quality |
| `--database` | schema, constraints, indexes, transactions, migrations, recovery |
| `--cloud` | deployment boundaries, infrastructure, secrets, and operations |

Data and Database are separate domains. A task with no changed files can still
route correctly from the intent, configured profile, repository survey, work
shape, risk, and current state.

## Updates and recovery

When a new Baron release is available, run the normal project update:

```bash
baron update
```

The update verifies the release, plans a preserve-first transaction, and writes
only Baron-managed files. A user edit or an uncertain baseline fails closed and
leaves a reviewable recovery record. Project source, Vault Markdown, custom
skills, and custom agents stay in the user's ownership boundary.

If work stops or a hook is unavailable, the next session can use the persisted
Task State and recovery packet: intent, constraints, plan, last successful
step, proof/trace evidence, affected files, blocker, and safe next action.
Native hooks are accelerators; the managed Codex or Claude contract remains the
fallback.

## Advanced diagnostics

The complete internal command catalog, structured prepare protocol, migration
receipts, and evidence rules are documented in
[COMMAND_SURFACE.md](docs/architecture/COMMAND_SURFACE.md). These commands are
for diagnostics, tests, migration, or maintainers. Normal project work stays
at the initialization and natural-language task layer.

Read the integration details in [Codex compatibility](docs/compatibility/CODEX.md)
and [Claude compatibility](docs/compatibility/CLAUDE.md). The architecture
pages explain Core ownership, memory trust, context tiers, hooks, Autopilot,
and update boundaries.

## Demo

The [public demo](docs/demo/README.md) walks through a long-running repository
with shared project memory, bounded context, recovery, and evidence gates.

## Public Proof

The [Baron 3 public certification](docs/assessment/baron-3-public-certification.md)
and the later assessment records document reproducible workspace, safety, and
release checks. The [build status](docs/BARON_STATUS.md) is the durable phase
dashboard; the [release guide](docs/RELEASE.md) is the source for installer and
rollback procedures.

## Safety and ownership

- User instructions and repository source have precedence over generated
  guidance.
- One live managed path has one owner: Core owns `.baron/core/**`; Codex and
  Claude own only their native bridges, wrappers, and managed entries.
- User text outside Baron markers, unknown JSON/TOML keys, custom skills and
  agents, and third-party hooks are preserved.
- Trusted memory is project-bound. Cross-project evidence needs an explicit
  firewall match, and unknown facts stay unknown.
- Proof requires execution evidence. A configured tool or a generated sentence
  is not proof that a check ran.
- Rollback and uninstall never delete project or Vault data.

Uninstall details and release lifecycle commands live in [RELEASE.md](docs/RELEASE.md).
