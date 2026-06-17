---
name: observability-and-instrumentation
description: Use when adding or reviewing logs, metrics, tracing, alerts, SLOs, audit events, production diagnostics, runbooks, or operational visibility.
license: MIT-inspired guidance; Baron-native adaptation
---

# Observability And Instrumentation

This bundled optional domain skill helps agents make running systems understandable. It is not a workflow skill; Superpowers still owns planning, implementation discipline, and verification.

## Use When

- adding logs, metrics, traces, spans, dashboards, alerts, SLOs, audit events, or runbooks
- debugging production-like failures where the project lacks visibility
- reviewing whether a feature has enough operational evidence for support

Do not use for pure UI polish, copy-only work, or test-only refactors with no runtime signal.

## Baron Contract

- Do not claim production behavior from logging code alone.
- Do not log secrets, tokens, PII, session data, private prompts, or raw vault memory.
- Prefer structured, low-cardinality signals that answer a real operator question.
- Keep evidence tied to the current project capsule; do not promote observations globally unless confirmed reusable.

## Signal Checklist

- What question must the operator answer?
- What event, metric, span, or log proves it?
- What labels are safe and bounded?
- What failure mode needs an alert?
- What dashboard/runbook entry helps the next agent or human?
- What test or smoke run confirms instrumentation fires?

## Output Contract

- Operational question
- Proposed signals
- Redaction/privacy notes
- Alert or SLO impact
- Verification evidence or missing proof
- Baron proof/trace updates needed

## Attribution

Inspired by MIT-licensed observability ideas from `addyosmani/agent-skills`, rewritten as Baron-native optional guidance.
