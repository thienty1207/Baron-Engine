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
