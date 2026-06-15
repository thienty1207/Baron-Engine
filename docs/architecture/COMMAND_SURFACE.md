# Baron Command Surface

This document tracks the target CLI contract.

## Phase 1

```bash
baron survey [repo-path]
baron survey [repo-path] --json
baron init [repo-path] --codex --shadow
baron init [repo-path] --claude --shadow
baron init [repo-path] --agent --shadow
```

## Phase 2

```bash
baron memory status [repo-path] --vault <vault-path>
baron memory index [repo-path] --vault <vault-path>
baron memory compact [repo-path] --vault <vault-path>
baron recall "<query>" [repo-path] --vault <vault-path>
```

All Phase 2 memory commands also accept `BARON_VAULT`. If neither `--vault` nor
`BARON_VAULT` is provided, Baron fails clearly instead of guessing.

## Phase 3

```bash
baron context [repo-path] --codex --vault <vault-path>
baron context [repo-path] --claude --vault <vault-path>
baron context [repo-path] --agent --vault <vault-path>
baron context [repo-path] --codex --task "<task>" --vault <vault-path>
baron context [repo-path] --why --vault <vault-path>
```

Context commands also accept `BARON_VAULT`. Normal context requires exactly one
adapter target. `--why` defaults to generic-agent reasoning when no adapter is
specified. Phase 3 prints bounded context to stdout and does not generate
adapter files.

## Phase 4

```bash
baron init --codex
baron init --claude
baron init --agent
baron update --codex
baron update --claude
baron update --agent
```

Implemented syntax accepts optional `[repo-path]`. `init` accepts
`--vault <vault-path>` or `BARON_VAULT`; later commands may use
`.baron/local.toml`. `update` without an adapter refreshes all registered
adapters.

## Phase 5

```bash
baron plan status
baron plan start "<title>"
baron plan update "<note>"
baron plan complete "<proof>"
baron plan interrupt "<state>"
baron harness status
baron harness intake "<title>"
baron proof status
baron trace record "<summary>"
baron trace score
```

Implemented Phase 5 surface:

```bash
baron plan status [repo-path]
baron plan start "<title>" [repo-path]
baron plan update "<note>" [repo-path]
baron plan interrupt "<state>" [repo-path]
baron plan complete "<verification>" [repo-path]
baron harness status [repo-path]
baron harness intake "<title>" [repo-path]
baron harness decision "<summary>" [repo-path]
baron harness friction "<summary>" [repo-path]
baron proof status [repo-path]
baron proof record "<verification>" [repo-path]
baron trace record "<summary>" [repo-path] --outcome <completed|partial|blocked|failed>
baron trace score [repo-path] [--id <trace-id>]
```

`baron trace score` exits unsuccessfully when the trace does not meet the
risk-required tier. This makes the quality gate enforceable by agent runtimes
instead of being a passive report.
Detailed traces require a changed project file; Baron-managed config, adapter,
plan, harness, proof, and trace files are excluded from that evidence.

## Phase 7

```bash
baron capability register "<capability>" [repo-path] --name <provider> --kind <cli|binary|mcp|skill|http|agent-adapter> [--required] [--command <command>] [--scan <target>] [--adapter <codex|claude|agent>]... --description "<description>"
baron capability check [capability] [repo-path] [--adapter <codex|claude|agent>] [--json]
baron capability list [repo-path] [--adapter <codex|claude|agent>] [--json]
baron capability remove "<capability>" [repo-path] --name <provider>
baron proof record "<verification>" [repo-path] --capability-evidence "<capability>|<provider>|<result summary>"
```

Capability definitions are committed in `.baron/capabilities.toml`. Presence
checks write only `.baron/cache/capability-state.json`. Presence and adapter
compatibility do not count as task execution; structured proof evidence is
required before a registered required capability can support completion.
