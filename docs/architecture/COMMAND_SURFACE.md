# Baron Command Surface

This page separates the small normal user flow from the internal protocol used
by Core, native hooks, tests, migration, and release maintainers. It reflects
the current `baron --help` and subcommand help. Internal commands remain
available for evidence and recovery; they are not required for ordinary agent
work.

## Normal user commands

Install the native executable, choose a Vault, initialize a project, and then
open Codex or Claude normally:

```text
baron --version
baron setup --vault <vault-path>
baron init --codex [--shadow] [--<profile>]
baron init --claude [--shadow] [--<profile>]
baron update [repo-path]
```

Initialization accepts `--frontend`, `--backend`, `--fullstack`, `--mobile`,
`--desktop`, `--tool`, `--library`, `--data`, `--database`, `--cloud`, or
`--unknown`. `--data` and `--database` are separate profiles. The optional
`--vault <path>` selects the machine-local Vault routing for the project.

`--shadow` inventories a project without taking ownership of existing files.
The regular initialization transaction installs the canonical Core runtime and
the selected native bridge. Repeating initialization is safe and preserves
custom content.

`baron update` is the public update entry point. It verifies the release and
refreshes only Baron-managed project files. Dry-run, transaction, rollback, and
collision details are exposed in diagnostic output and receipts rather than
being part of the normal agent workflow.

## Adapter diagnostics

Codex and Claude are the only active adapters. The compatibility commands below
exist for inspection, migration, and repair of old project state:

```text
baron adapter status [repo-path]
baron adapter switch --to codex|claude [repo-path]
baron adapter switch --to codex|claude [repo-path] --dry-run
baron --codex
baron --claude
```

These commands do not change Core ownership or make a project-global adapter
preference authoritative. A normal user can open either initialized host
directly.

## Advanced / diagnostic commands

Normal users do not manually invoke Prepare. Native hooks or the managed
Codex/Claude contract send structured task input to Core. The high-level
protocol is documented here for integration tests and diagnostics:

```text
baron control-plane prepare --adapter codex --json
baron control-plane prepare --adapter claude --json
```

Task text is supplied as structured hook input or a safe transport; it is not
interpolated into a shell command. The returned `PreparePacketV1` includes
profile, work shape, risk, Task State, trusted context, selected resources,
verification, blockers, warnings, and next action.

The supported diagnostic groups are:

| Group | Purpose |
| --- | --- |
| `survey [--json]` | inspect repository identity, files, risks, and profile hints |
| `context` / `recall` | inspect bounded context and trusted memory retrieval |
| `memory` | index, compact, import sessions, inspect status, and resume evidence |
| `plan` | inspect or update start, progress, interruption, and completion state |
| `harness` | inspect intent, stories, decisions, friction, audits, and outcomes |
| `proof` / `trace` | execute or record evidence and score completion quality |
| `continuity` | checkpoint and recover interrupted task state |
| `capability` / `runtime` | inspect provider registration, presence, and safe backends |
| `automation` | inspect hook, reconciliation, and code-map observations |
| `autopilot` | inspect, review, approve, reject, or defer candidates |
| `control-plane` | inspect route, prepare, evidence, and gate receipts |
| `asset` | audit, quarantine, or propose a skill under ownership rules |
| `session-replay` | index, search, and replay bounded current-project messages |
| `migrate` | run or inspect transactional legacy migration and rollback |
| `certify` | run or inspect the release-profile certification |
| `release` | maintainer-only metadata and verification for a release package |

The exact forms are discoverable with `baron <group> --help`. The adapter
bridges load only route-selected Core resources; they do not ask the user to
run this catalog during a task.

## Ownership and safety contract

Core owns `.baron/core/**`, project identity, Task State, memory, receipts, and
workflow semantics. Codex owns its `AGENTS.md`, `.agents/`, and managed `.codex`
projection. Claude owns its `CLAUDE.md`, `.claude/` bridge, wrappers, and
managed entries. One live managed path has one owner.

Managed writes use safe replacement and the project lock. Missing, malformed,
changed, or ambiguous baselines fail closed and preserve user bytes. Update,
migration, rollback, and uninstall receipts remain recoverable and never remove
project source or Vault Markdown.

## Help as the contract

The root help intentionally keeps the public list short: `setup`, `init`,
`update`, `adapter`, and `help`, plus the Codex/Claude shortcuts. Deep groups
are still compiled into the executable so hooks, adapters, tests, and
maintainers share one observable Core protocol. Documentation examples should
be checked against `baron --help` and the relevant subgroup help after every
command-surface change.
