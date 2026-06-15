# Baron Implementation Roadmap

Date: 2026-06-14
Status: phases 0 through 7 completed; phase 8 next

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

Status: completed.

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

Verification:

- managed-block and custom-asset preservation tests
- Codex, Claude, and generic adapter tests
- multi-adapter init/update CLI tests
- nested-directory automatic config smoke

## Phase 5 - Plan, Harness, Proof, Trace

Goal: connect execution state with proof and trace quality.

Status: completed.

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
- validation matrix linking each story to proof status and evidence
- trace quality tiers
- friction backlog

Verification:

- plan lifecycle and interruption tests
- risk-aware harness tests
- proof and trace tier tests
- high-risk completion rejection tests
- repo/Vault mirror tests
- execution CLI smoke

## Phase 6 - Native Migration And Legacy Retirement

Status: completed on 2026-06-15.

Goal: recover useful data from legacy Agent Bootstrap projects, convert it into
Baron-native structures, verify the conversion, and retire the legacy runtime.
Baron must not carry Agent Bootstrap architecture forward.

Commands:

```bash
baron migrate agent-bootstrap --dry-run
baron migrate agent-bootstrap
```

Deliverables:

- read legacy config and inventory old managed assets in dry-run mode
- back up the pre-migration project inside Vault migration artifacts
- convert useful memory, plans, harness records, proof, and traces
- validate custom skills and agents against Baron contracts
- quarantine invalid or conflicting custom assets
- generate Baron-native core assets, adapters, config, and indexes
- verify imported counts and hashes before cleanup
- remove Agent Bootstrap managed files and runtime after successful verification
- support rollback without depending on Agent Bootstrap

Verification:

- no-write inventory tests
- representative legacy project fixtures
- record-count and content-hash parity tests
- invalid skill/agent quarantine tests
- cleanup allowlist tests
- rollback tests
- zero Agent Bootstrap runtime dependency scan

Delivered:

- transactional dry-run/apply/status/rollback lifecycle
- Vault-contained backup manifest, receipt, and failure record
- memory, plans, Product Harness, research, notes, questions, handoff, and
  session import
- strict custom skill/agent validation and quarantine
- Baron-native adapter/core asset regeneration
- content-hash and imported-content verification before cleanup
- automatic rollback on failed install or verification

## Phase 7 - Baron Capability Registry

Status: completed on 2026-06-15.

Goal: let Baron know which tools are available, what capability each tool
provides, whether the active agent can use it, and how missing tools affect
proof confidence.

Commands:

```bash
baron capability register
baron capability check
baron capability list
baron capability remove
```

Deliverables:

- capability-based provider registry
- provider kinds for CLI, binary, MCP, skill, HTTP, and adapter
- `present`, `missing`, and `unknown` presence states
- active-adapter compatibility
- clean fallback behavior for optional capabilities
- bounded context summary
- Proof/Trace confidence integration
- execution evidence requirement before a tool-backed claim is accepted

Verification:

- provider-kind and presence-probe tests
- capability lookup and adapter compatibility tests
- graceful degradation tests
- false tool-execution claim regression tests
- bounded context and shared-Vault smoke tests

Delivered:

- committed `.baron/capabilities.toml` registry and rebuildable machine cache
- CLI, binary, MCP, skill, HTTP, and agent-adapter provider kinds
- adapter-specific presence and compatibility observations
- automatic checks in Codex, Claude, and generic startup contracts
- bounded context summary with cache isolation between adapters
- optional-provider degradation and required-provider diagnostics
- structured capability execution evidence in Proof
- Trace failure when required capability evidence is absent

## Phase 8 - Hardening And Release

Goal: ship Baron as a reliable tool.

Deliverables:

- Windows binary
- Linux binary
- macOS binary
- checksum verification
- smoke tests on old repos
- smoke tests on very large repos
- multi-project vault tests
- adapter output tests
- capability degradation tests
- docs and install flow

## Release Rule

Every phase must end with:

- updated `notes/build-log/CURRENT.md`
- tests or smoke checks
- clear next action
- commit
