# SPEC-08-05 — Safety, Recovery, and Failure Handling

**Parent:** [SPEC-08-00](SPEC-08-00-REPLACEMENT-OVERVIEW.md)<br>
**Status:** Fail-closed safety/recovery contracts and synthetic Windows checkpoint assessment at each saved phase boundary are implemented locally and verified. Automatic phase execution/continuation, Linux acceptance, and an OS-enforced sandbox backend remain unverified, so target execution stays blocked. Not committed or pushed.
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

## Local implementation checkpoint — 2026-10-29

The current worktree implements typed `in_progress`, `complete`, `incomplete`,
`blocked`, and `failed` audit states; failure classes that remain distinct from
negative findings; capability registration/presence/execution reporting; and a
PreparePacket projection that reports every required sandbox control as
unverified. No target command is launched because Baron has no trusted
OS-enforcement backend in this implementation.

Recovery checkpoints are bounded, strict-schema artifacts bound to the current
Baron Project ID, full Git revision, scoped dirty-state digest, exact scoped
file inventory/content hashes, run binding, phase, coverage, candidates,
pending work, and safe next action. Resume rechecks identity/revision/state and
is only an assessment hint; changes require a fresh audit. Checkpoint and
report publication is create-only and preserves conflicting user edits.
Reports redact common secret-like values and cannot claim a clean result,
proof, trace, or application-task completion.

Focused Windows regressions currently cover truthful states, failed and timed
out execution, capability evidence binding, fail-closed sandbox readiness,
checkpoint identity/revision/content/dirty-state validation, bounded source
inventory, malformed/schema-tampered checkpoints, idempotent create-only
publication, bounded reads for checkpoint inputs and conflicting output files, and report redaction/conflict preservation.
An added synthetic-repository CLI integration constructs interruption states
at 0–5 completed phase receipts plus the terminal six-receipt state, saves each
checkpoint, reloads it in separate CLI processes, and checks phase/status,
candidate fingerprints, coverage completeness, repeatable resume assessments,
and byte-for-byte checkpoint preservation. It proves resumability assessment
and state retention at each boundary; `resume-checkpoint` does not advance the
audit or demonstrate finding deduplication after continuation. Windows format,
the full workspace suite (117 targets; exit 0, no failures, 4 ignored),
warnings-denied Clippy, and focused Core/CLI/adapter tests passed. The
PreparePacket CLI suite passed 9/9 in 122.76 seconds. The source-controlled
target remains unexecuted. Linux CI and any automatic phase coordinator remain
acceptance work; SPEC-08 as a whole is still open, and SPEC-08-03/08-04 do not
provide an executable audit or remediation loop while the sandbox backend is
unverified.
