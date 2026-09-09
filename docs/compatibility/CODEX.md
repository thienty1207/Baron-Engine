# Codex Compatibility

Verified 2026-09-08 against the current Baron source tree, generated payloads,
and the Codex bridge contract.

## Native project surface

`baron init --codex` installs one canonical Core runtime and a thin Codex
projection:

```text
.baron/core/**
      ↓ selected resources
AGENTS.md
.agents/skills/baron-engine/SKILL.md
.agents/skills/baron-engine/agents/openai.yaml
.codex/agents/*.toml
.codex/INDEX.md
.codex/hooks.json
```

The semantic source for every skill and quality agent remains in the canonical
Core. The bridge is deliberately thin and does not create `.codex/skills/**`
copies. The three native TOML wrappers project the Core contracts for
`code-reviewer`, `security-auditor`, and `test-engineer`.

`openai.yaml` sets `policy.allow_implicit_invocation: false` so AGENTS.md can
own automatic routing without a second implicit lifecycle. The bridge may be
invoked explicitly for diagnostics and receives the same `PrepareRequestV1`
and `PreparePacketV1` semantics as the managed root contract.

## Lifecycle and task state

Codex supplies an explicit `OperationContext` with project/session identity,
task text, profile, work shape, risk, and current Task State. Core applies
`TrustedRecallPolicy::Current`, selects only the route-relevant canonical Core
resources, and returns bounded Tier 0–3 context, verification, blockers,
warnings, and next action. Task State carries intent, constraints, plan,
recovery, affected files, proof/trace state, and the last safe step across
Codex and Claude sessions.

Autopilot housekeeping and learning candidates stay behind the existing trust
and approval rules. A candidate never becomes current memory, routing policy,
or a skill simply because Codex saw it.

Codex host-local context is non-authoritative. It cannot override Core trusted
memory, intent, decisions, Task State, continuity, recovery, or proof.

## Hooks and fallback

`.codex/hooks.json` contains Baron-owned lifecycle entries merged with existing
third-party hooks and unrelated JSON keys. Native hooks are optional
accelerators. The managed `AGENTS.md` contract is the fallback when a hook is
missing, untrusted, skipped, or returns a soft failure. Event keys are
deduplicated under the project lock, so repeated initialization does not
duplicate entries or run two prepare operations for one event.

## Preservation and update boundary

User AGENTS text outside Baron markers, custom skills and agents, Codex settings,
unknown routing keys, and third-party hooks remain in place. A changed or
ambiguous Baron baseline fails closed and leaves a reviewable recovery receipt.
Core owns `.baron/core/**`; the Codex adapter owns only its native projection.

The host can open the project directly after initialization. Users do not need
to operate internal Baron commands or change a project preference before a
Codex session can consume the canonical Core.

## Runtime evidence

The generated contract and layout tests prove the source-level integration. A
live Codex completion claim requires a detected executable and an executed safe
task with evidence; generated files alone are not runtime proof.
