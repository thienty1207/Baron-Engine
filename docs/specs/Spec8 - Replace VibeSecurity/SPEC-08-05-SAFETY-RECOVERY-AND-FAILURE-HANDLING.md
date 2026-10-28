# SPEC-08-05 — Safety, Recovery, and Failure Handling

**Parent:** [SPEC-08-00](SPEC-08-00-REPLACEMENT-OVERVIEW.md)<br>
**Status:** Draft for owner review; implementation not started
**Depends on:** SPEC-08-01; required before audit or remediation execution

## Goal

Make the replacement safe and resumable across interruption, missing capabilities, malformed evidence, and host differences. A failure must produce a truthful partial/blocked result, never a false clean audit.

## Capability and execution gates

- Use Baron Core’s Capability Registry to distinguish a tool/agent that is registered, present, and actually executed.
- Static source reasoning remains available when optional tools are absent. No optional provider is installed automatically.
- Run target-controlled builds, tests, fixtures, fuzzers, browsers, or processes only if an OS-enforced sandbox provides: no external network; a sanitized allowlisted environment; a read-only source/toolchain boundary; writes restricted to assigned scratch; and bounded CPU, memory, process, disk, file-size, and time.
- If any required sandbox control is unavailable, do not run the target-controlled process. Continue only the source-only part and mark execution-dependent checks needs_validation.
- Never probe live deployments, external services, shared infrastructure, production identities, or user data. Never emit weaponized exploit instructions.
- Port the findings and coverage validation contract into deterministic Baron-native validation where practical. The normal user path must not unexpectedly require Node.js or install third-party tooling.

## Run state

The implementation must represent at least these distinct run states:

- in_progress
- complete
- incomplete
- blocked
- failed

Findings remain separately typed as confirmed, needs_validation, or rejected. A run may be complete when all required audit phases passed for the declared scope even if it contains confirmed findings; that run must not be described as clean. Application-task completion is separate and remains blocked while required in-scope findings await fixes or a product decision.

Each resumable checkpoint binds to project ID, exact source revision and dirty-state fingerprint, audit mode, scope, language set, completed phase, coverage ledger, candidate fingerprints, pending work, and safe next action. Resume must recheck the source identity and current content before trusting prior evidence.

## Failure-handling matrix

| Failure | Required behavior |
|---|---|
| OS sandbox absent or incomplete | Do not execute target code; keep source findings and report exact checks needing validation. |
| Independent verifier/subagent unavailable | Do not label the affected candidate independently confirmed; mark the run incomplete/blocked. |
| Language or framework unsupported | Use only defensible generic checks and record the unsupported surface as a coverage gap. |
| Source changes during a run | Invalidate evidence tied to changed paths, re-scope, and revalidate before completion. |
| User interrupts, usage limit, host crash, or agent failure | Persist the last safe checkpoint and next action; resume idempotently without duplicating findings or silently dropping units. |
| Coverage/finding JSON is malformed or schema validation fails | Do not advance to final complete status; preserve a bounded diagnostic and repair from trusted state. |
| Hunter/verifier disagreement | Keep the finding unresolved and request a fresh validation pass; never silently select the convenient verdict. |
| Target-controlled command fails or times out | Record exact command class and safe diagnostics; distinguish failure from a negative test result. |
| Fix or regression test fails | Keep the original finding open, retain the test failure, and stop or continue only within the bounded fix loop. |
| Report/artifact write is interrupted | Use atomic, recoverable publication; never expose a truncated report as final output. |
| Secret or personal data appears in evidence | Redact before report or memory promotion; never copy the ambient environment or secret values. |
| Agent budget/resource limit is reached | Stop launching work, retain unreviewed units as deferred/incomplete, and state the exact coverage loss. |

## Recovery and idempotency

- Use Baron Task State/continuity for task intent, phase, evidence references, blockers, and the next safe action.
- Writes to structured audit artifacts must be bounded, validated, and atomic or journaled. A retry must not duplicate findings, lose unrelated state, or overwrite an external edit.
- Source revision and project ID are mandatory evidence bindings; folder basename, stale run files, or a prior revision alone cannot bind findings to the current project.
- Prior-run findings and coverage are hints only. Recheck affected source before carrying evidence forward.
- Store detailed audit artifacts in an explicitly authorized task output location or the existing Baron project capsule contract. Do not create hidden directories in the application repository by default.
- Do not promote a candidate to trusted project or global memory until it has passed the current validation and Baron trust rules.

## Acceptance and fault-injection tests

- Inject interruption after each persisted phase boundary and prove safe resume, idempotent findings, and accurate coverage.
- Test source mutation, changed project identity, missing/malformed ledger, schema mismatch, output write interruption, verifier disagreement, resource exhaustion, and secret-redaction paths.
- Test all terminal states and ensure only complete runs with all required gates may make complete audit claims.
- Verify on Windows and Linux that sandbox/capability absence degrades safely rather than executing target code or claiming an audit pass.
- Test that a failed command is not interpreted as “no vulnerability found.”
