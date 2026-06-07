# Baron Product Spec 1.0

Date: 2026-06-08
Status: draft foundation spec

## One-Line Definition

Baron is a Rust-first multi-agent memory operating system that turns a repo,
a vault, execution plans, product harness data, proof, and trace history into
the exact context an AI agent needs before it acts.

## Why Baron Exists

Existing kits solve parts of the problem:

- `agent-bootstrap-obsidian-cli` gives Codex a strong Obsidian vault bridge,
  Superpowers workflow, active plan state, Product Harness, trace quality, and
  memory recall.
- `repository-harness` makes repos more agent-ready with feature intake, story
  packets, validation matrix, decision records, and a Rust CLI.

Baron must stand above both. It must be durable for new projects, old projects,
single-agent workflows, and multi-agent toolchains.

## Product Position

```text
repository-harness makes a repo ready for agents.
agent-bootstrap gives Codex a vault-backed memory workflow.
Baron gives many agent tools a shared long-term brain that knows what to read,
what to trust, what proof is required, and what to remember next.
```

## Target Users

- Developers using Codex, Claude Code, Cursor, or other coding agents.
- Builders with many projects sharing one long-term vault.
- Teams adopting agents inside old or messy repositories.
- Solo builders who want AI sessions to resume without losing context.

## Core Outcomes

Baron succeeds when an agent can enter a repo and know:

- what the product is
- where implementation surfaces live
- what memory is relevant
- which facts are verified
- which facts are stale or unknown
- what the active task is
- what proof is required
- how to leave a useful trace
- which adapter format to use

## Non-Goals

- Baron is not a chatbot.
- Baron is not an IDE.
- Baron is not a replacement for tests.
- Baron is not a replacement for Superpowers.
- Baron is not a central server.
- Baron does not treat SQLite as source of truth.
- Baron does not assume a repo is safe to rewrite on first install.

## Design Principles

1. Shadow first.
   Baron observes old repos before changing them.

2. Vault first.
   Markdown in the vault is durable memory. Indexes and databases accelerate it
   but do not replace it.

3. Context is compiled, not dumped.
   Baron sends the smallest useful context bundle for the current task, phase,
   risk, and agent tool.

4. Memory has confidence.
   Verified, likely, stale, cross-project, and unknown memory must be separated.

5. Agent tools are adapters.
   Codex, Claude, Cursor, and generic agents receive different file shapes from
   the same Baron core.

6. Proof gates completion.
   High-risk work cannot be called done without evidence.

7. Trace makes the next session smarter.
   Meaningful work must leave a bounded trace that explains outcome, proof,
   files touched, friction, and next action.

## Core Systems

### 1. Survey Engine

Reads a repo in shadow mode and produces a Project Atlas:

- project type
- stack hints
- entrypoints
- test/build commands
- risky surfaces
- docs that may be stale
- files and directories to read first
- unknowns that require confirmation

### 2. Vault Memory Engine

Stores durable memory in Markdown and maintains accelerated indexes:

- current project memory
- verified global memory
- global candidates
- cross-project references
- imported session summaries
- plan, harness, proof, and trace records

### 3. Memory Firewall

Prevents a shared vault from polluting the current project:

- current project memory wins
- verified global memory is allowed when relevant
- cross-project memory is blocked unless explicitly matched
- stale memory is downgraded
- unknown facts remain unknown

### 4. Context Compiler

Builds adapter-specific context bundles:

- `baron context --codex`
- `baron context --claude`
- `baron context --agent`

The compiler chooses content by:

- task query
- active plan
- current story
- risk lane
- changed files
- proof requirements
- relevant memory
- adapter constraints

### 5. Plan State

Tracks active implementation state:

- current focus
- status
- last known state
- verification state
- next action
- interruption recovery

### 6. Product Harness

Tracks feature intent and product contract:

- input type
- story packet
- risk flags
- proof checklist
- validation matrix
- decisions
- friction backlog

### 7. Proof Engine

Connects work to validation:

- known test commands
- proof ladder
- risk-based proof requirements
- proof history
- missing proof warnings

### 8. Trace Quality Engine

Scores task traces:

- incomplete
- minimal
- standard
- detailed

High-risk work must leave detailed traces.

### 9. Adapter Engine

Generates and refreshes tool-specific assets:

- Codex: `AGENTS.md`, `.codex/skills`, `.codex/agents`
- Claude: `CLAUDE.md`, command/import guidance
- Generic: portable `AGENT.md`/JSON/Markdown contracts

## Core Assets

Baron keeps these core concepts from `agent-bootstrap`:

- Superpowers as the only workflow core.
- Three core quality agents:
  - `code-reviewer`
  - `security-auditor`
  - `test-engineer`
- Bundled optional domain skills:
  - `frontend-design`
  - `vibe-security-scan`

Optional custom skills and optional custom agents must be registered and routed.
They must not become core by accident.

## First Release Boundary

The first useful Baron release should do less than the final vision but be real:

- `baron survey`
- `baron init --codex`
- `baron init --claude`
- `baron init --agent`
- `baron context`
- basic Project Atlas
- basic Vault layout
- basic adapter output
- basic memory index
- basic status command

It should not attempt advanced semantic recall, full proof automation, or
complete migration until the foundation is proven.
