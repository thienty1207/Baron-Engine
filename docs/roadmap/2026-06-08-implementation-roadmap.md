# Baron Implementation Roadmap

Date: 2026-06-14
Status: phase 3 completed; phase 4 next

## Phase 0 - Foundation Skeleton

Goal: create a repo that cannot lose its direction.

Deliverables:

- Rust workspace skeleton
- product spec
- architecture docs
- roadmap
- temporary build notes
- core asset blueprints
- adapter blueprints
- first commit

Verification:

- `cargo test`
- `cargo run -p baron-cli -- --help`

## Phase 1 - Survey Engine

Goal: make Baron safely understand a repo before changing it.

Status: completed.

Commands:

```bash
baron survey [repo-path]
baron survey [repo-path] --json
baron init [repo-path] --codex --shadow
baron init [repo-path] --claude --shadow
baron init [repo-path] --agent --shadow
```

Deliverables:

- repo detection
- stack hints
- entrypoint detection
- risky surface detection
- test/build command discovery
- Project Atlas Markdown
- Project Atlas JSON
- no-write shadow mode

## Phase 2 - Vault And Memory Firewall

Goal: create durable memory without cross-project contamination.

Status: completed.

Commands:

```bash
baron memory status [repo-path] --vault <vault-path>
baron memory index [repo-path] --vault <vault-path>
baron memory compact [repo-path] --vault <vault-path>
baron recall "<query>" [repo-path] --vault <vault-path>
```

Deliverables:

- vault scaffold
- project capsule
- approved global memory
- global candidates
- confidence levels
- SQLite cache/index
- current-project priority
- cross-project blocking

Verification:

- `cargo test`
- memory CLI smoke
- multi-project firewall smoke
- `git diff --check`

## Phase 3 - Context Compiler

Goal: make AI read the right thing automatically.

Status: completed.

Commands:

```bash
baron context [repo-path] --codex --vault <vault-path>
baron context [repo-path] --claude --vault <vault-path>
baron context [repo-path] --agent --vault <vault-path>
baron context [repo-path] --codex --task "<task>" --vault <vault-path>
baron context [repo-path] --why --vault <vault-path>
```

Deliverables:

- compact context bundle
- adapter-specific formatting
- task/risk/phase-aware read selection
- bounded output
- skipped-context explanation

Verification:

- `cargo test`
- Codex, Claude, and generic context smoke
- task-risk smoke
- context-selection `--why` smoke
- `git diff --check`

## Phase 4 - Agent Adapters

Goal: support multiple agent tools without changing the Baron core.

Commands:

```bash
baron init --codex
baron init --claude
baron init --agent
baron update --codex
baron update --claude
baron update --agent
```

Deliverables:

- Codex adapter assets
- Claude adapter assets
- generic agent assets
- safe update/merge behavior
- user-owned content preservation

## Phase 5 - Plan, Harness, Proof, Trace

Goal: connect execution state with proof and trace quality.

Commands:

```bash
baron plan status
baron plan start "<title>"
baron harness intake "<title>"
baron proof status
baron trace record "<summary>"
baron trace score
```

Deliverables:

- active plan state
- product harness story packets
- risk flags
- proof requirements
- validation matrix
- trace quality tiers
- friction backlog

## Phase 6 - Migration From Agent Bootstrap

Goal: allow existing `agent-bootstrap` projects to move into Baron safely.

Commands:

```bash
baron migrate agent-bootstrap --dry-run
baron migrate agent-bootstrap
```

Deliverables:

- read existing `vault.config.json`
- preserve vault memory
- preserve plans/harness/traces
- preserve skills and agents
- generate Baron adapters
- keep rollback backups

## Phase 7 - Hardening And Release

Goal: ship Baron as a reliable tool.

Deliverables:

- Windows binary
- Linux binary
- macOS binary
- checksum verification
- smoke tests on old repos
- multi-project vault tests
- adapter output tests
- docs and install flow

## Release Rule

Every phase must end with:

- updated `notes/build-log/CURRENT.md`
- tests or smoke checks
- clear next action
- commit
