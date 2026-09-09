# Claude Code Compatibility

Verified 2026-09-08 against the current Baron source tree, generated payloads,
and the Claude bridge contract.

## Native project surface

`baron init --claude` installs one canonical Core runtime and a thin Claude
projection:

```text
.baron/core/**
      ↓ selected resources
CLAUDE.md
.claude/skills/baron-engine/SKILL.md
.claude/agents/{code-reviewer,security-auditor,test-engineer}.md
.claude/settings.json
.claude/skills/INDEX.md
.claude/agents/INDEX.md
.claude/commands/...       diagnostic conveniences where retained
```

The bridge is thin and sets `disable-model-invocation: true`. It does not copy
the Baron semantic library into `.claude/skills/**`; selected skills and agents
resolve relative to the canonical Core root. Claude wrappers are native views
of Core contracts, not separate workflow owners.

## Lifecycle and Task State

Claude supplies an explicit `OperationContext` and structured
`PrepareRequestV1`. Core applies `TrustedRecallPolicy::Current`, uses the
configured profile with task intent, repository survey, work shape, risk, and
current Task State, then returns `PreparePacketV1`. The packet contains bounded
Tier 0–3 context, selected canonical Core resources, verification, warnings,
blockers, and a safe next action.

Task State links the original intent, constraints, plan, last successful step,
affected files, proof/trace state, blocker, and recovery packet. It is shared
with Codex through the project ID and Vault boundary, so an interrupted task
can resume from evidence rather than a guessed host memory note.

Autopilot may perform bounded housekeeping and offer a reviewable candidate.
Candidates stay outside trusted memory, routing, policy, and Core assets until
an existing approval authority records a decision.

## Hooks, fallback, and host memory

`.claude/settings.json` merges Baron hook entries with third-party settings and
unknown keys. Claude hooks are optional accelerators over the canonical Core.
`CLAUDE.md` is the fallback when a hook is absent, untrusted, skipped, or
returns a soft failure. Hook event keys are deduplicated under the project
lock, and hook execution is never inferred from configuration alone.

Claude host auto memory remains host-local context. It is not Baron trusted
memory and cannot override Core intent, decisions, continuity, recovery, or
proof. Existing personal skills, commands, settings, and text outside Baron
markers are preserved.

## Update and runtime evidence

Core owns `.baron/core/**`; Claude owns only its native bridge and managed
projection. Changed or ambiguous baselines fail closed and leave a recoverable
receipt. The update path never silently replaces user-owned content.

The generated contract and layout tests prove source-level compatibility. A
live Claude completion claim requires a detected executable and an executed safe
task with evidence; generated files alone are not runtime proof.
