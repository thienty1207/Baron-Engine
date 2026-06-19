# Baron Demo - 10-year repo

This demo shows how Baron should feel when a coding agent enters a large,
old repository.

The repo in this walkthrough is fictional, but the flow matches the tested
Baron CLI behavior.

## Starting Point

Imagine a 10-year full-stack product:

- old frontend routes mixed with new components
- backend auth, billing, and upload code
- stale architecture notes
- prior agent sessions with useful decisions
- multiple projects sharing the same long-term Vault

Without a harness, a fresh AI session usually guesses what to read, mixes old
memory with current work, and claims completion after a shallow test.

## Install Into The Project

```bash
baron --version
baron setup --vault "D:\work\AgentMemory"
cd D:\work\IT\Web\LegacyProduct
baron init --codex --fullstack
```

Equivalent generic agent surfaces:

```bash
baron init --claude --fullstack
baron init --agent --fullstack
```

## What Changes For Codex

Codex receives `AGENTS.md`, `.codex/`, Superpowers, Baron's core quality gates,
and adapter startup guidance.

At the start of real work, the AI is instructed to load a bounded context brief,
resume active work, route only matching skills, and record proof/trace evidence.

Expected shape:

```text
Baron Context Brief
- Project: legacyproduct
- Platform focus: fullstack
- Read first: product map, current plan, current harness story
- Memory firewall: current project preferred, cross-project memory blocked
- Session replay: bounded current-project hits only
- Runtime backend: safe providers with execution evidence only
```

## What Changes For Claude

Claude receives `CLAUDE.md` plus Baron-managed commands and skill/agent assets.
The same engine state is loaded, but the instructions use Claude-friendly
surface files.

Expected shape:

```text
Claude startup
- Load Baron context for this repository
- Use Superpowers as workflow core
- Use Baron quality agents as gates
- Do not mark complete without proof and trace evidence
```

## What Changes For A Generic Agent

Generic agents receive `AGENT.md`, portable context files, and `.baron/core`.
They can still follow the same contract without depending on Codex or Claude
specific behavior.

Expected shape:

```text
Generic agent startup
- Read AGENT.md
- Use baron-context.md/json
- Preserve local project instructions outside Baron markers
```

## Before And After

| Area | Before Baron | After Baron |
| --- | --- | --- |
| Read-first context | Agent guesses from file names | Baron gives a bounded project atlas and current work packet |
| Memory | Chat history or scattered notes | Vault-backed memory with project isolation |
| Shared Vault | Easy to mix projects | memory firewall blocks weak cross-project recall |
| Active work | Lost after interruption | continuity resume tells the next session where to continue |
| Product intent | Usually implicit | Product Harness tracks goal, risk, proof, and decisions |
| Completion | Agent may claim done after shallow work | proof gate and trace quality must support the claim |
| Old sessions | Hard to search | session replay searches bounded prior conversation messages |
| Tool checks | Tool exists might be treated as proof | safe runtime backend requires execution evidence |

## Example Agent Task

Human asks:

```text
Implement backend login hardening and make sure old auth decisions are respected.
```

Baron-backed agent behavior:

1. Load current project context without dumping the whole Vault.
2. Retrieve only relevant auth/security memory from the current project.
3. Start or resume an active plan.
4. Create or resume a high-risk Product Harness story.
5. Use security and test quality gates.
6. Record proof only after the relevant test command actually runs.
7. Leave a trace that links task, files, proof, and remaining risk.

The point is not that the user runs every internal command. The point is that
the AI has a stable operating harness instead of relying on memory from chat.

## What This Demo Proves

Baron is designed for large repositories where a simple prompt is not enough.
It does not replace the app, framework, or tests. It gives the AI a disciplined
way to understand the project, use the right memory, and leave evidence for the
next session.
