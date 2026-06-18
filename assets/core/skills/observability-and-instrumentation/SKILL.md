---
name: observability-and-instrumentation
description: Use when adding or reviewing logs, metrics, traces, correlation id propagation, alerts, SLOs, audit events, production diagnostics, runbooks, or operational visibility.
license: MIT-compatible Baron-owned local guidance; attribution lives in NOTICE.md
---

# Observability And Instrumentation

This bundled optional domain skill helps AI make running systems understandable. It is not a workflow skill. Superpowers remains the workflow core for planning, TDD, debugging, review, and verification.

## Baron Contract

- Use this skill only when Baron routing selects it for operational visibility work.
- Do not replace Superpowers, proof, trace, `code-reviewer`, `security-auditor`, or `test-engineer`.
- Never log secrets, tokens, passwords, session cookies, private prompts, raw Vault memory, payment details, or unnecessary PII.
- Treat logs, metrics, traces, alerts, SLOs, and audit events as product evidence only when they can be verified.
- Mark production behavior as unknown when no runtime, test, dashboard, trace, or log evidence exists.
- Prefer a few useful signals over noisy instrumentation that future agents cannot interpret.
- Keep signal names, labels, and event fields stable enough for dashboards and alerts.
- Use Baron proof and trace to record what signal was added or verified.

## Use When

- The task asks for logs, structured logging, metrics, traces, spans, correlation id, request id, alert, dashboard, SLO, audit event, runbook, or diagnostics.
- The code handles auth, payments, uploads, background jobs, queues, external providers, webhooks, tenant isolation, or production incident risk.
- The user asks why something failed and the repo lacks enough evidence to debug it next time.
- A Product Harness story has proof gaps around operations, support, monitoring, or production readiness.
- A feature needs safe auditability: who did what, when, to which resource, and with what result.
- A backend/API flow needs latency, error, retry, or dependency visibility.

Do not use this skill for pure UI copy, local-only refactors, or tests that have no runtime visibility impact.

## Operator Questions

Every signal must answer at least one real question:

- Did the request enter the system?
- Which user, tenant, org, job, or resource was involved, if safe to log?
- What dependency was called?
- How long did it take?
- Did it fail, retry, time out, or degrade?
- Was the failure user-caused, system-caused, provider-caused, or unknown?
- What proof helps the next agent reproduce or verify the issue?

If no operator question exists, do not add instrumentation just for volume.

## Signal Types

### Logs

- Prefer structured fields over unparseable strings.
- Include correlation id or request id when available.
- Use safe identifiers, not raw secrets or sensitive payloads.
- Use levels consistently: debug for local detail, info for meaningful lifecycle events, warn for recoverable risk, error for failed outcomes.
- Avoid duplicate logs in hot loops.
- Avoid logging whole request bodies unless the project has explicit safe redaction.

### Metrics

- Metrics should be low-cardinality and aggregatable.
- Counters track counts such as requests, failures, retries, and jobs processed.
- Histograms track latency, size, duration, and queue wait.
- Gauges track current depth, active workers, open connections, or cache size.
- Avoid labels like raw user id, email, token, URL with query, full path, or unbounded error text.

### Traces

- Traces connect request, service, database, provider, job, and downstream spans.
- Use span names that describe behavior, not implementation trivia.
- Add bounded attributes: tenant id only if safe, operation type, provider name, status, retry count, cache hit, and correlation id.
- Do not put PII or secrets in span attributes.
- Trace sampling should not hide critical errors completely.

### SLOs And Alerts

- SLOs define what "healthy enough" means for a user or operator.
- Alerts must be actionable; every alert needs owner, cause hints, and first response steps.
- Alert on symptoms before causes when possible.
- Use burn-rate or sustained thresholds for noisy metrics.
- Do not page humans for informational events.

### Audit Events

- Audit events should be append-only or tamper-evident when the domain requires it.
- Record actor, action, resource, result, timestamp, and correlation id.
- Record permission changes, admin actions, billing changes, destructive actions, and security-sensitive events.
- Do not store secrets or full sensitive payloads in audit records.

## Correlation Id Rules

- Generate or accept a correlation id at the trust boundary.
- Propagate it through logs, traces, job payloads, and outbound provider calls when safe.
- Return a public-safe request id in API errors when useful.
- Do not trust arbitrary client correlation ids for security decisions.
- Do not use correlation id as authentication, authorization, or tenant proof.

## Privacy And PII

- Treat PII as sensitive by default.
- Prefer stable opaque IDs over names, emails, phone numbers, addresses, cookies, tokens, prompts, or payment data.
- Redact before writing logs, traces, metrics, audit events, or Vault memory.
- If redaction behavior is unknown, say unknown and avoid recommending raw logging.
- The `security-auditor` gate should review observability changes that touch secrets, auth, tenant, or payment data.

## Verification

- Unit tests can assert formatter, redaction, event construction, and label allowlists.
- Integration tests can assert logs/metrics/traces are emitted for success and failure paths.
- Smoke checks can run a route/job and inspect captured output.
- For alerts/SLOs, verify config syntax and state the missing production proof if live dashboards cannot be checked.
- For correlation id, verify propagation across request, job, and outbound calls where practical.
- For PII, verify sensitive fields are absent or redacted.
- Baron proof should include command output, captured log, metric name, trace span, alert config, or audit record sample.
- Baron trace should state the operational question the signal answers.

## Output Contract

Return observability guidance in this shape:

1. Operational question: what the signal proves.
2. Signal plan: logs, metrics, traces, audit events, alerts, SLOs, or runbook changes.
3. Safety: PII, secrets, labels, cardinality, sampling, and retention risks.
4. Implementation notes: exact files/surfaces and smallest useful change.
5. Verification: commands, captured output, or missing proof.
6. Baron evidence: proof/trace summary and any required quality gate.
7. Unknowns: production dashboard, live traffic, provider behavior, or runtime facts not verified.

## Red Flags

- Logging full request bodies or raw errors from auth/payment/provider flows.
- Labels with user email, token, raw URL, stack trace, or unbounded exception message.
- Metrics that cannot be aggregated because every label value is unique.
- Alerts with no action or owner.
- SLOs copied from another system without product relevance.
- Trace spans that expose PII or secrets.
- Audit logs that miss actor, resource, action, result, or timestamp.
- Claiming "observable" without a test, smoke, or captured evidence.

## Baron Final Check

- If the task is high risk, require proof and trace before completion.
- If the signal could expose sensitive data, route security review.
- If the signal supports product readiness, update Product Harness proof gaps.
- If evidence is missing, keep the finding as unknown instead of pretending production visibility exists.
