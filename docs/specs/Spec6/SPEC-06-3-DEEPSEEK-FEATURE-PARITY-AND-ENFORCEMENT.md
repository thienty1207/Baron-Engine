# SPEC-06.3 — DeepSeek Feature Parity and Evidence Enforcement

- **Status:** Optional future design; deferred. Implementation is not approved or scheduled. Begin only after the owner explicitly requests SPEC-06 implementation.
- **Parent scope:** SPEC-06, cross-model continuation between Codex and DeepSeek Harness.
- **Depends on:** SPEC-06.1 task identity/resume and SPEC-06.2 DSH adapter.

## Goal

Ensure a DSH session uses Baron Core's existing semantics for search, selected skills, logging, continuity, quality agents, proof, trace, and gates. Host-native prompts and model claims alone never count as execution evidence.

## Core authority

Baron Core remains the only authority for routing, trusted recall, project memory, Task State, plans, selected Core skills, required quality agents, proof, trace, and completion gates. The DSH adapter translates host events and Core responses; it does not implement parallel search, a second task ledger, or a second policy engine.

## Search and trusted context

- On task entry, prepare through Core before DSH performs a project mutation.
- Use Baron Core's trusted search/retrieval path for Baron memory and context. DSH filesystem search may help inspect source code but cannot substitute for a required Baron memory search.
- When Core requires retrieval, the adapter performs or requires that retrieval and records a bounded execution receipt. Search configuration, tool availability, prompt text, or a model's statement that it searched is not evidence.
- Respect `TrustedRecallPolicy::Current`; candidate, contested, superseded, expired, stale, or unrelated-project records cannot be injected as current truth.
- Keep injected context bounded to Core's selected resources and DSH/model context limits. Do not expose unrelated Vault projects or secrets.

## Skills and quality agents

- Load route-selected Core skills from `.baron/core/**` into DSH using its supported lifecycle/context extension point before the relevant model request. Mandatory Core skills must not depend on the model discovering or voluntarily calling an optional skill tool.
- Preserve user skills and agents. DSH's support for project `.agents/skills` may be used to discover user-owned skills, but Baron must not copy over or rewrite them.
- Keep the exactly three mandatory quality roles defined by Core: `code-reviewer`, `security-auditor`, and `test-engineer`. DSH-specific wrappers may translate invocation format but must retain Core role definitions and return bounded evidence to the parent task.
- A configured or visible skill/agent is not proof it ran. Core records actual invocation and result evidence when a policy requires execution.

## Logging and privacy

- Map DSH lifecycle and tool events into Baron-owned, project/task/session-correlated event records with stable idempotency keys.
- Capture the minimum bounded evidence needed for continuity, audit, and gates: event kind, host session, tool identity, safe path/operation metadata, result summary, and receipt references. Content capture follows the same redaction and trust boundaries as existing Baron adapters.
- Do not log provider keys, credentials, environment values, or raw secret-bearing configuration. Do not treat DSH's private session log as the cross-host source of truth.
- Retries, duplicate hooks, host restarts, and late events must not duplicate checkpoints or evidence. Event replay follows Core dedup/recovery rules.

## Tools, gates, and completion

- Use DSH lifecycle/tool extension points to inject prepare context, observe actual tool execution, and apply Core authorization before a protected operation.
- Route Baron search, checkpoint, recovery, proof, trace, and gate operations through structured Baron-owned interfaces. Never build shell commands by interpolating arbitrary task text.
- A mandatory gate is satisfied only by valid Core execution evidence. DSH hook/plugin presence, MCP registration, skill availability, a successful-looking assistant message, or a user-configured provider is not sufficient.
- Before DSH reports task completion, ask Core to reconcile required receipts, proof, trace, and blockers. If evidence is missing or stale, return the blocking reason and next safe action instead of success.
- If DSH cannot expose a required interception point, mark the capability as unsupported/degraded and fail closed for workflows that depend on it. Do not silently downgrade an enforced gate to prompt-only guidance.

## What “strict” means

The adapter can strictly enforce observable protocol conditions: prepare was run, required Core search returned, selected mandatory content was injected, protected tool calls passed Core policy, actual events were recorded, and required gates have valid receipts. It cannot prove a model internally understood or obeyed natural-language guidance. That assurance comes from controlled tool execution plus deterministic review/test/proof gates, not from the model's claim.

## Acceptance criteria

- A DSH task receives the same Core route, trusted context policy, selected skills, quality-agent roles, gates, blockers, and next action as an equivalent Codex task.
- A task requiring Baron search has a Core search execution record before the protected mutation; a task not requiring search does not gain unrelated retrieval noise.
- Selected mandatory skills are injected before model work. User-owned skills remain available and unchanged.
- DSH tool results and lifecycle events produce correctly correlated Baron log/checkpoint evidence, with retries idempotent and secrets redacted.
- Completion is blocked when a required search/skill invocation/quality-agent/proof/trace receipt is absent, stale, malformed, or bound to another project/task/session.
- A missing or degraded DSH hook cannot be represented as a passing gate based solely on configuration or prompt output.
- Codex and Claude behavior and their persisted evidence contracts remain unchanged by the DSH integration.

## Required verification cases

1. Compare equivalent Codex and DSH prepare packets for the same project/task and verify semantic parity in route, trusted context, required skills, gates, and next action.
2. Require a Core search, omit it, and verify mutation/completion is blocked; then run the search and verify only the valid receipt satisfies the gate.
3. Verify required skill injection occurs before DSH's model request even if the model never asks to load the skill.
4. Replay DSH lifecycle/tool events and verify logs/checkpoints remain idempotent; inject credential-like values into test fixtures and verify they do not appear in Baron logs or diagnostics.
5. Omit a required quality-agent, proof, trace, or tool execution receipt and verify Core reports a blocker rather than completion.
6. Disable or break the DSH plugin/hook and verify required workflows fail closed with a clear degraded-host status.
