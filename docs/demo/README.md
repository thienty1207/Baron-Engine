# Baron Demo - 10-year repo

This walkthrough shows how Baron feels when Codex or Claude enters a large,
old repository. The repository is fictional, but the flow matches the tested
Baron CLI and generated integration contracts.

## Starting point

Imagine a ten-year full-stack product with old frontend routes, backend auth
and billing, stale architecture notes, useful prior sessions, and multiple
projects sharing one Vault. Without a Core memory boundary, a fresh agent may
guess what to read, mix projects, and claim completion after a shallow check.

## Install and initialize

```bash
baron --version
baron setup --vault "D:\work\AgentMemory"
cd D:\work\IT\Web\LegacyProduct
baron init --codex --fullstack
```

The same project can also receive the Claude native projection:

```bash
baron init --claude --fullstack
```

Both projections consume the one canonical runtime under `.baron/core/**`.

## What the agent receives

After initialization, ask Codex or Claude for a task. Baron prepares a bounded
packet from the project survey, trusted memory, profile, Task State, route, and
proof requirements.

Expected shape:

```text
Baron Context Brief
- Project: legacyproduct (stable project ID)
- Platform focus: fullstack
- Intent and constraints: loaded from the current task
- Read first: project atlas, current plan, current harness story
- memory firewall: current project preferred, cross-project memory blocked
- Task State: last step, blocker, proof, recovery, next action
- Route: only the selected Core skills and quality agents
- Runtime backend: safe providers with execution evidence only
```

Native hooks can accelerate SessionStart, prompt, compaction, and stop events.
The managed host contract is the fallback when a hook is absent or untrusted.
Load Baron context for this repository through the host integration; a human
does not need to operate the internal command catalog.

The proof gate stays closed until execution evidence exists, and the safe runtime backend
check records what actually ran. Bounded session replay keeps
useful prior messages available without flooding the task.

## Before and after

| Area | Before Baron | After Baron |
| --- | --- | --- |
| Read-first context | Agent guesses from file names | Core gives a bounded project atlas and work packet |
| Memory | Chat history or scattered notes | Vault-backed trusted memory with project isolation |
| Shared Vault | Easy to mix projects | Memory firewall blocks weak cross-project recall |
| Active work | Lost after interruption | Task State and recovery preserve the next safe action |
| Product intent | Usually implicit | Harness and plan evidence carry intent and constraints |
| Completion | Shallow checks may be claimed | Proof and trace require execution evidence |
| Old sessions | Hard to search | Bounded current-project session replay |
| Tool checks | Presence treated as proof | Safe runtime backends require execution evidence |

## Example task

The human asks:

```text
Implement backend login hardening and make sure old auth decisions are respected.
```

The Baron-backed agent then:

1. loads current project context without dumping the Vault;
2. retrieves relevant trusted auth and security evidence;
3. starts or resumes the active plan and Task State;
4. creates or resumes a high-risk Harness story when required;
5. routes the security and test quality gates;
6. records proof only after the relevant check runs;
7. leaves a trace linking the task, files, evidence, and remaining risk.

The user asks for the outcome in ordinary language. Core handles routing,
memory, continuity, and evidence while Codex or Claude remains the native work
surface.

## What this demo proves

Baron is designed for long-running repositories where one prompt is not enough.
It does not replace the application, framework, or tests. It gives the agent a
stable Core, a trusted memory boundary, a route-aware task state, and a proof
trail that the next session can use.
