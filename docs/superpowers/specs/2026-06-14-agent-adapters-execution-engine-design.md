# Baron Phase 4-5 Design - Agent Adapters And Execution Engine

Date: 2026-06-14
Status: approved

## Goal

Turn Baron's compiled context into a real multi-agent harness, then connect that
harness to active plans, product intent, proof, and trace quality.

## Product Boundary

Baron remains one brain with multiple adapters:

- Rust core owns configuration, memory, context, plans, harness, proof, and trace.
- Codex, Claude, and generic-agent files only translate the same Baron contract.
- Vault Markdown remains durable memory.
- Repo Markdown remains current execution state.
- SQLite remains a rebuildable memory index.
- Superpowers remains the only workflow core.
- The three core quality agents remain `code-reviewer`, `security-auditor`, and
  `test-engineer`.

## Configuration

Baron uses two files:

```text
.baron/
  project.toml
  local.toml
  .gitignore
```

`project.toml` is safe to commit. It contains:

- schema version
- stable project slug
- registered adapters
- automatic startup, plan, harness, proof, and trace switches

`local.toml` is machine-local and ignored by `.baron/.gitignore`. It contains:

- absolute Vault path for this machine

Resolution order for Vault paths:

1. explicit `--vault`
2. `BARON_VAULT`
3. nearest ancestor `.baron/local.toml`

Baron never stores memory in either config file.

## Adapter Lifecycle

Commands:

```bash
baron init [repo-path] --codex --vault <vault-path>
baron init [repo-path] --claude --vault <vault-path>
baron init [repo-path] --agent --vault <vault-path>
baron update [repo-path]
baron update [repo-path] --codex
baron update [repo-path] --claude
baron update [repo-path] --agent
```

Repeated `init` adds an adapter to the registered adapter list. `update` with no
adapter refreshes every registered adapter.

Managed root instruction files use Baron markers:

```text
<!-- BARON:MANAGED:START -->
...
<!-- BARON:MANAGED:END -->
```

Text outside those markers is user-owned and must survive update. Managed asset
directories refresh only known Baron files. Unknown custom skills and agents
are never deleted.

### Codex

Generates:

- `AGENTS.md`
- `.codex/INDEX.md`
- `.codex/skills/INDEX.md`
- Superpowers workflow skill tree
- bundled optional `frontend-design` and `vibe-security-scan`
- `.codex/agents/INDEX.md`
- exactly three core agent TOML files

### Claude

Generates:

- `CLAUDE.md`
- `.claude/commands/baron-context.md`
- `.claude/commands/baron-status.md`
- `.claude/skills/` with the same routed workflow/domain assets
- `.claude/agents/` with Claude-readable forms of the three quality agents

### Generic Agent

Generates:

- `AGENT.md`
- `baron-context.md`
- `baron-context.json`
- `.baron/core/skills/`
- `.baron/core/agents/`

## Automatic Agent Contract

Every adapter tells the agent to:

1. run `baron context` at session start
2. inspect `baron plan status`
3. inspect `baron harness status`
4. start or resume a plan before meaningful implementation
5. create harness intake for medium/high-risk work
6. record proof after verification
7. record and score a trace before claiming completion
8. refuse a high-risk completion claim when proof or trace quality is weak

Users may inspect commands manually, but normal agent work is instruction-driven
and automatic.

## Execution State

Repo source of truth:

```text
docs/baron/
  plans/
    CURRENT.md
    INDEX.md
    YYYY-MM-DD/
  harness/
    CURRENT.md
    STORIES.md
    DECISIONS.md
    FRICTION.md
  proofs/
    INDEX.md
  traces/
    INDEX.md
    YYYY-MM-DD/
```

Vault mirror:

```text
Projects/<slug>/
  Plans/
  ProductHarness/
  Proofs/
  Traces/
```

Same-scope corrections update the active plan. Different work starts a new plan.
Silence or shutdown never means completed.

## Risk And Completion

Risk lanes:

- low: docs, copy, typo, small isolated text/config changes
- medium: ordinary frontend/backend/API/integration work
- high: auth, permissions, tenant/RLS, payment, migration, security, secrets,
  uploads, external providers, or destructive data work

Minimum proof:

- low: one concrete verification result
- medium: focused test/build/smoke evidence
- high: focused verification plus explicit security/data-impact evidence

Minimum trace:

- low: minimal
- medium: standard
- high: detailed

`plan complete` is blocked until the active plan has enough proof and a passing
trace score for its risk lane.

## Failure Handling

- Missing config: explain which init command creates it.
- Missing local Vault: explain `--vault`, `BARON_VAULT`, and `local.toml`.
- Malformed config: fail without rewriting it.
- Managed file without markers: append one managed block, preserve existing text.
- Failed Vault mirror: keep repo state, return an error, and do not claim success.
- Duplicate plan/intake title: resume the matching active item.
- Unknown facts: remain unknown.

## Testing

- config resolution from nested directories
- adapter init/update and preservation
- custom skill/agent survival
- multi-adapter registration
- plan interruption/resume/completion gate
- risk-aware harness intake
- proof recording and Vault mirror
- trace tiers and scoring
- high-risk completion rejection without proof
- automatic adapter instructions
- full CLI and smoke coverage

## Deferred

- Agent-bootstrap migration remains Phase 6.
- Release packaging remains Phase 7.
- Platform-specific executable hooks that require unstable external schemas are
  not hard-coded; adapters use durable instruction and command surfaces.
