# Baron Command Surface

This document tracks the target CLI contract.

## Phase 1

```bash
baron survey [repo-path]
baron survey [repo-path] --json
baron init [repo-path] --codex --shadow
baron init [repo-path] --claude --shadow
baron init [repo-path] --agent --shadow
baron init [repo-path] --reasonix --shadow
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
baron context [repo-path] --reasonix --vault <vault-path>
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
baron init --reasonix
baron update [repo-path]
```

Implemented syntax accepts optional `[repo-path]`. `init` accepts
`--vault <vault-path>`, `BARON_VAULT`, or the machine Vault configured by
`baron setup --vault`; later commands may use `.baron/local.toml`. Platform
flags store focus in `.baron/project.toml` so AI agents prioritize the right
domain knowledge without creating new workflow ownership. `baron update` is a
human-authorized verified release transaction: the candidate process renders
managed assets, ambiguous edits stage for review, and custom assets/Vault
memory remain outside its write set. AI agents use local-only automation
reconciliation and cannot authorize a download or runtime replacement.

## Phase 101-107: Reasonix adapter maintenance on 4.2.0

```bash
baron adapter status [repo-path]
baron adapter switch --to <codex|claude|agent|reasonix> [repo-path]
baron adapter switch --to reasonix [repo-path] --dry-run
```

The Reasonix adapter is a frontend compatibility track only. It keeps the
existing project ID, Vault path, Vault Markdown, memory index, session journal,
Wiki, and CodeGraph shared with the other adapters; the active adapter is
routing metadata, not a new memory namespace. Installation is preserve-first:
unmarked user-owned Reasonix files are left unchanged and listed as conflicts.
This track does not change the engine or create a Baron `4.3` release; the
public/source version remains `4.2.0`.

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
baron capability register "<capability>" [repo-path] --name <provider> --kind <cli|binary|mcp|skill|http|agent-adapter> [--required] [--command <command>] [--scan <target>] [--adapter <codex|claude|agent|reasonix>]... --description "<description>"
baron capability check [capability] [repo-path] [--adapter <codex|claude|agent|reasonix>] [--json]
baron capability list [repo-path] [--adapter <codex|claude|agent|reasonix>] [--json]
baron capability remove "<capability>" [repo-path] --name <provider>
baron proof record "<verification>" [repo-path] --capability-evidence "<capability>|<provider>|<result summary>"
```

Capability definitions are committed in `.baron/capabilities.toml`. Presence
checks write only `.baron/cache/capability-state.json`. Presence and adapter
compatibility do not count as task execution; structured proof evidence is
required before a registered required capability can support completion.

## Phase 8

Normal users install Baron through the checksum-verifying PowerShell or shell
installer documented in `docs/RELEASE.md`. The installer provides install,
one-time pre-3.4 upgrade, rollback, and uninstall. From Baron 3.4 onward,
normal project/runtime updates use `baron update`.

Release automation uses two hidden maintainer commands:

```bash
baron release metadata <artifacts-dir> --release-version <version> --source-revision <git-sha>
baron release verify <artifacts-dir> --expected-version <version> --expected-source-revision <40-character-git-sha>
```

They generate and verify `SHA256SUMS` plus `release-manifest.json` for the
complete Windows, Linux, Intel macOS, and Apple Silicon macOS artifact set.

## Phase 38

`baron update` is the sole normal-user update command. It verifies an official
target-specific candidate, lets that verified candidate plan and activate only
Baron-managed project assets, stages conflicts outside live files, and records
a recoverable runtime handoff. Hidden continuation, abort, and finalizer flags
exist only for the verified candidate protocol and are deliberately absent from
normal help. `baron automation reconcile` has a narrower AI-facing contract:
it uses the currently installed embedded assets only and cannot use the network
or replace the Baron executable.

## Phase 9

```bash
baron automation status [repo-path]
baron automation reconcile [repo-path]
baron automation hook <session-start|prompt|checkpoint|context-compiled|plan-started|harness-started|proof-recorded|trace-scored|stop> [repo-path] --adapter <codex|claude|agent|reasonix>
```

`automation hook` is the adapter-facing native entrypoint and reads the hook
payload from stdin. Normal users do not run it manually.

## Phase 10

```bash
baron memory import-sessions [repo-path] --vault <vault-path>
baron recall "<query>" [repo-path] --vault <vault-path>
baron context [repo-path] --task "<task>" --codex|--claude|--agent|--reasonix --vault <vault-path>
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

## Phase 16

No new normal-user command is introduced. Phase 16 expands control-plane routing
so `baron control-plane route "<task>"` can select narrow optional skills for
API/interface design, observability, performance, migration/deprecation, and an
optional web performance auditor. Superpowers remains the workflow core, and the
mandatory quality agents remain `code-reviewer`, `security-auditor`, and
`test-engineer`.

## Phase 17

```bash
baron continuity status [repo-path]
baron continuity checkpoint "<current state and next action>" [repo-path]
```

These commands are hidden from top-level help and are meant for AI automation,
adapter instructions, diagnostics, and recovery. Native hooks also refresh the
continuity packet automatically. Normal users should not need to run them during
ordinary work.
It proposes improvements but does not rewrite core policy without human
approval.

## Phase 27

```bash
baron harness intent-status [repo-path]
baron harness intent "<title>" [repo-path] --current "<observed state>" --target "<confirmed target>" --scope "<scope>" --proof "<required proof>" [--non-goal "<item>"]... [--constraint "<item>"]... [--decision "<item>"]... [--unknown "<item>"]... [--confirmed]
baron continuity recover "<root cause>" [repo-path] --outcome <failed|blocked|interrupted> --last-success "<last successful step>" --next-action "<safe next action>" [--evidence "<evidence>"]... [--affected-file "<path>"]... [--retry-condition "<condition>"]...
```

These commands are hidden AI-automation surfaces. Medium/high-risk Harness
intake requires a matching confirmed intent brief. `--confirmed` records
explicit shared understanding; adapters must not infer confirmation from
silence. Recovery packets preserve failed or interrupted attempts, mirror to
the Vault, and give the next session a safe evidence-backed resume point.
Low-risk maintenance does not require a formal intent brief.

## Phase 28-31

Normal users keep using the same commands:

```bash
baron init --codex --fullstack
baron init --mobile
baron update
```

The first platform becomes primary. Later platform init adds an extension and
regenerates evidence-based profile and architecture contracts without moving
existing code. AI adapters use these hidden review commands when needed:

```bash
baron review status [repo-path]
baron review finding "<summary>" [repo-path] --severity <level> --evidence "<evidence>" [--affected-file "<path>"]...
baron review close <finding-id> [repo-path] --fix-evidence "<change>" --verification "<command/result>"
```

Review closure requires both fix evidence and verification. These commands are
not part of the normal user workflow.

## Phase 18

No new normal-user command is introduced. Phase 18 hardens Baron-managed runtime
skills and agents so operational instructions are self-contained local assets.
Attribution, license notes, and upstream inspiration live outside runtime
`SKILL.md` files. Superpowers remains the workflow core, and optional skills stay
lazy-routed.

## Phase 19

```bash
baron asset audit [repo-path]
baron asset quarantine [repo-path]
baron asset propose-skill <skill> "<reason>" <content-path> [repo-path]
```

These commands are hidden from top-level help and are meant for AI automation,
diagnostics, migration, and maintenance. Audit scores local runtime assets for
external runtime links, thin contracts, missing proof/trace language, duplicate
workflow ownership, and recursive subagent orchestration. Quarantine moves
failing custom assets out of routing while skipping managed Baron assets. Skill
proposals are staged as reviewable diffs with approval metadata and never
overwrite runtime guidance silently.

## Phase 20

```bash
baron session-replay index [repo-path] --vault <vault-path>
baron session-replay search "<query>" [repo-path] --vault <vault-path>
baron session-replay replay <message-id> [repo-path] --vault <vault-path> --radius <n>
```

These commands are hidden from top-level help and are meant for AI automation,
diagnostics, and recovery. Context automatically refreshes the current project's
session replay index and includes only bounded matching messages when a task is
provided. Search and replay are filtered by project identity so shared Vaults do
not leak weak cross-project session history into the active project.

## Phase 21

```bash
baron autopilot status [repo-path]
baron autopilot review "<summary>" [repo-path]
baron autopilot approve <candidate-id> [repo-path]
baron autopilot reject <candidate-id> [repo-path]
```

These commands are hidden from top-level help and are meant for AI automation,
diagnostics, and review. Autopilot proposes learning candidates after meaningful
work, surfaces continuity resume state, records observed automation, and keeps
uncertain learning separate from trusted facts until approved.

## Phase 22

```bash
baron runtime check [repo-path] [--adapter <codex|claude|agent|reasonix>] [--json]
```

This command is hidden from top-level help and is meant for AI automation and
diagnostics. Runtime checks distinguish provider availability from real
execution evidence, flag unsafe backends, warn on degraded optional tools, and
block required unsafe or unverified tool-backed proof claims.

## Baron 3.7 Quality And Trust

The following AI-facing commands are hidden from the normal top-level help.
They make work shape, execution evidence, Harness experiments, and application
runbooks explicit without adding another workflow core:

```bash
baron work-shape "<task>" [repo-path] --json
baron proof execute --capability <capability> --provider <provider> [repo-path] -- <command> [args...]
baron proof record [verification] [repo-path] --receipt <receipt-id>
baron control-plane record-gate <agent> "<summary>" [repo-path] --receipt <receipt-id>
baron control-plane evidence [repo-path] --required <agent>
baron harness experiment-start [repo-path] --baseline "<baseline>" --hypothesis "<hypothesis>" --intervention "<intervention>" --approved
baron harness experiment-rerun <experiment-id> [repo-path] --available --retrieved --invoked --relevant --outcome "<outcome>"
baron harness experiment-outcome <experiment-id> <keep|revise|remove|pending> "<decision>" [repo-path]
```

`baron proof execute` is the only trusted receipt producer. Receipts bind the
project, current source fingerprint, command, provider, result, bounded output
digests, and integrity hash. Medium/high-risk plan completion requires a fresh
receipt and all three core quality gates to reference current receipt IDs;
legacy Markdown remains readable but cannot satisfy that completion gate.

An optional project-owned `docs/baron/operations/RUNBOOK.md` is routed only to
runtime, reproduce, end-to-end, and deployment-smoke tasks. Its start command,
readiness, interface, runtime evidence, and cleanup ownership are reported as
bounded project facts; unknown values stay unknown until observed.

## Phase 23

Baron 3.0 certification uses the existing release and certification commands:

```bash
baron certify run [repo-path] --vault <vault-path> --profile <smoke|release|extreme>
baron certify status [repo-path]
baron release metadata <artifacts-dir> --release-version <version> --source-revision <git-sha>
baron release verify <artifacts-dir> --expected-version <version> --expected-source-revision <40-character-git-sha>
```

Certification includes memory firewall, context budget, observable automation,
autopilot readiness, runtime backend policy, and release metadata gates.

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

## Baron 3.8 Knowledge

```bash
baron memory resume [repo-path] --vault <vault-path> [--task "<task>"] [--json]
baron wiki index [repo-path]
baron wiki search "<query>" [repo-path] [--limit <n>]
baron wiki status [repo-path]
baron knowledge resume [repo-path] --vault <vault-path> [--task "<task>"] [--json]
baron knowledge benchmark [repo-path] --vault <vault-path> [--task "<task>"] [--json]
baron knowledge codegraph-index [repo-path]
baron knowledge codegraph-query "<query>" [repo-path] [--limit <n>] [--json]
```

These are hidden AI-facing commands. Resume output is bounded and project-ID
isolated. Wiki and local CodeGraph write only disposable `.baron/cache/` JSON;
they never replace Vault Markdown or current repository source. Missing,
stale, corrupt, unsupported, or oversized derived data degrades to lexical
recall or Survey instead of blocking normal coding or satisfying proof.

The optional reverse skills are routed by `baron control-plane route` only for
binary, APK/mobile, malware, or firmware analysis. They are static/read-only by
default and do not add a router, case lifecycle, global bootstrap, or quality
gate.

## Baron 4.1 Intelligence And Security

The following hidden AI-facing commands expose guarded 4.1 intelligence without
changing the normal user surface. Baron 4.1 is the released default;
`BARON_ENGINE_GENERATION=4.0` selects the guarded fallback and
`BARON_ENGINE_GENERATION=3.8` (or `baseline`) selects the older recovery
baseline. Every 4.1 path automatically falls back when structural checks fail.

```bash
baron memory consolidate [repo-path] --vault <vault-path> [--json] [--stage]
baron intelligence benchmark [repo-path] --vault <vault-path> [--json]
baron intelligence security-route "<task>" [repo-path] [--json]
baron intelligence security-regression [repo-path] --vault <vault-path> [--json]
baron intelligence static-scan [repo-path] --vault <vault-path> [--json]
baron intelligence acceptance [repo-path] --vault <vault-path> [--json]
```

`memory consolidate` is deliberately non-promoting: it reports duplicate,
conflicting, superseded, stale, and unknown candidates but never promotes or
rewrites Vault facts. Optional `--stage` writes one atomic, reviewable proposal
under the project capsule's `Artifacts/` folder; it still does not change
memory records. The benchmark writes only rebuildable assessment logs,
and the security regression command writes deterministic evidence under
`docs/assessment/`. `static-scan` is a bounded source-only AppSec pass: it
never executes targets and every finding remains advisory until the
`security-auditor` gate validates it. Wiki and CodeGraph candidates remain available through the
existing hidden `baron wiki` and `baron knowledge` commands; their links,
imports, spans, and advisory relations are bounded cache evidence, not durable
truth.
