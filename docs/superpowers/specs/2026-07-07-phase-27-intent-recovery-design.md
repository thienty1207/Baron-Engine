# Phase 27 Intent Clarity And Actionable Recovery Design

## Goal

Make Baron understand and persist the user's intended outcome before medium or
high-risk implementation, then preserve an actionable recovery path whenever
work fails, blocks, or is interrupted.

## Boundaries

- Superpowers remains the workflow owner.
- Product Harness owns durable intent and risk classification.
- Continuity owns interruption and recovery state.
- Vault Markdown remains the durable mirror and source of truth.
- Normal users do not need new commands; hidden commands exist for generated
  agent automation and inspection.
- Low-risk maintenance remains lightweight.

## Intent Contract

Baron stores one current intent brief in the repo and Vault. The brief records:

- title
- current behavior
- target behavior
- scope
- non-goals
- constraints
- decisions
- required proof
- remaining unknowns
- risk lane
- confirmation status and timestamp

Medium and high-risk Harness intake must refuse to start until a matching intent
brief is confirmed. Low-risk intake remains allowed without a formal confirmed
brief. Repeating the same confirmed intent updates the current pointer without
creating duplicate history.

## Recovery Contract

Baron stores every distinct failed, blocked, or interrupted attempt as a
separate recovery record and mirrors it to the Vault. Each packet records:

- stable recovery ID
- outcome
- root cause
- last successful step
- evidence
- affected files
- safe next action
- retry conditions
- linked plan, Harness story, proof, and trace state

The latest packet is copied to `CURRENT_RECOVERY.md` for bounded startup context.
Repeated identical recovery input resumes the current packet instead of creating
duplicates. A newer attempt never rewrites an older failed attempt as success.

## Context And Automation

Compact context loads bounded current intent and recovery summaries. Generated
Codex, Claude, and generic instructions require agents to inspect existing repo,
Vault, plan, decisions, and continuity before asking one missing question at a
time. They record confirmed intent before medium/high-risk intake and record
recovery before claiming an unfinished outcome.

## Verification

- core RED/GREEN tests for intent persistence, confirmation gate, low-risk path,
  recovery history, deduplication, Vault mirror, and bounded context
- CLI tests for hidden intent and recovery commands
- adapter contract tests for automatic behavior
- full workspace tests, formatting, Clippy, and a temp project/Vault smoke
