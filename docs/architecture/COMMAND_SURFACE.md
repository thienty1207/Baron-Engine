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
baron setup --vault [vault-path]
baron memory status [repo-path] --vault <vault-path>
baron memory index [repo-path] --vault <vault-path>
baron memory compact [repo-path] --vault <vault-path>
baron recall "<query>" [repo-path] --vault <vault-path>
```

`baron setup --vault` with no path uses the current directory as the machine
default Vault and stores it in `~/.baron/config.toml`. All Phase 2 memory
commands also accept `BARON_VAULT`. If neither `--vault`, `BARON_VAULT`, a
project-local `.baron/local.toml`, nor machine setup is available, Baron fails
clearly instead of guessing.

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
baron init --frontend
baron init --backend
baron init --fullstack
baron init --mobile
baron init --desktop
baron init --tool
baron init --library
baron init --data
baron init --cloud
baron init --codex --fullstack
baron update --codex
baron update --claude
baron update --agent
```

Implemented syntax accepts optional `[repo-path]`. `init` accepts
`--vault <vault-path>`, `BARON_VAULT`, or the machine Vault configured by
`baron setup --vault`; later commands may use `.baron/local.toml`. Platform
flags store focus in `.baron/project.toml` so AI agents prioritize the right
domain knowledge without creating new workflow ownership. `update` without an
adapter refreshes all registered adapters.

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

## Phase 8

Normal users install Baron through the checksum-verifying PowerShell or shell
installer documented in `docs/RELEASE.md`. Install, update, rollback, and
uninstall are installer actions rather than project commands.

Release automation uses two hidden maintainer commands:

```bash
baron release metadata <artifacts-dir> --release-version <version> --source-revision <git-sha>
baron release verify <artifacts-dir>
```

They generate and verify `SHA256SUMS` plus `release-manifest.json` for the
complete Windows, Linux, Intel macOS, and Apple Silicon macOS artifact set.

## Phase 9

```bash
baron automation status [repo-path]
baron automation reconcile [repo-path]
baron automation hook <session-start|prompt|checkpoint|context-compiled|plan-started|harness-started|proof-recorded|trace-scored|stop> [repo-path] --adapter <codex|claude|agent>
```

`automation hook` is the adapter-facing native entrypoint and reads the hook
payload from stdin. Normal users do not run it manually.

## Phase 10

```bash
baron memory import-sessions [repo-path] --vault <vault-path>
baron recall "<query>" [repo-path] --vault <vault-path>
baron context [repo-path] --task "<task>" --codex|--claude|--agent --vault <vault-path>
```

Context automatically imports a bounded batch for initialized projects.
`memory import-sessions` is the explicit inspection/recovery command.

## Phase 11

```bash
baron control-plane status [repo-path]
baron control-plane route "<task>" [repo-path] --risk <low|medium|high>
baron control-plane record-gate <agent> "<evidence summary>" [repo-path]
baron control-plane evidence [repo-path] --required <agent>
```

The control plane validates skill/agent contracts, protects Superpowers as the
only workflow owner, explains selected and skipped assets, and records evidence
before a mandatory quality gate counts as run.

## Phase 12

```bash
baron harness audit [repo-path]
baron harness verify-all [repo-path] [--limit <n>]
baron harness intervention "<summary>" [repo-path]
baron harness propose [repo-path]
baron harness outcome <proposal-id> "<actual outcome>" [repo-path]
```

The self-improving harness audits context reads, proof gaps, trace gaps,
documentation drift, interventions, friction patterns, and proposal outcomes.
It proposes improvements but does not rewrite core policy without human
approval.

## Phase 13

```bash
baron certify run [repo-path] --vault <vault-path> --profile <smoke|release|extreme>
baron certify status [repo-path]
```

Certification writes a Markdown and JSON report under
`docs/baron/certification/` and mirrors the Markdown report into the project
Vault capsule. It checks repository survey boundedness, memory-cache rebuild,
shared-Vault firewall behavior, compact context budget, automation readiness,
and release readiness.

## Phase 14

Baron 2.0 release hardening keeps the hidden maintainer release metadata
commands from Phase 8 and adds the public certification gate above. A release
claim is not trusted unless `baron certify run`, full tests, Clippy, release
metadata verification, and installer lifecycle tests pass.
