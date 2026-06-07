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
baron memory status
baron memory index
baron memory compact
baron recall "<query>"
```

## Phase 3

```bash
baron context --codex
baron context --claude
baron context --agent
baron context --why
```

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
