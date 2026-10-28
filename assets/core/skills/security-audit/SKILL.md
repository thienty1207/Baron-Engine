---
name: security-audit
description: Use for bounded security review planning on meaningful application changes and security-sensitive boundaries.
---

# Baron Security Audit

## Status and boundary

This managed skill is the canonical Baron source-code AppSec route. It replaces
the former Vibe route and remains distinct from the mandatory `security-auditor`
quality gate. Baron Core selects the route; Codex and Claude only receive its
projection.

This entrypoint currently defines the routing and reporting contract. The
seven-language coverage corpus in SPEC-08-02 and the complete audit workflow in
SPEC-08-03 are not yet implemented. Treat the matching
`audit_workflow_pending_spec_08_03` capability blocker in PreparePacket as
authoritative. Do not claim that an audit ran, passed, or found no issues while
that blocker remains.

## Use When

- Baron Core selects `security-audit` for a meaningful application code change.
- The task changes application source, build or dependency configuration, CI,
  security policy, or a data-flow boundary.
- The task or Core identifies a security-sensitive surface requiring a broader
  review.

Pure documentation and cosmetic work do not trigger a broad audit unless they
change a security policy or another security-sensitive boundary.

## Baron Contract

- Read the `security_audit` object from the current PreparePacket when present.
- Keep the selected mode and scope bounded to the task, changed paths, and
  connected trust boundaries.
- Before implementation, assess the planned scope and existing source for
  material trust-boundary risks. If a validated issue crosses into a connected
  in-repository boundary, expand the review scope before remediation.
- Treat `detected_languages` as evidence from named source paths, not proof that
  every language or repository file was covered.
- Treat a missing language, path, or capability as unknown and report it.
- Selecting this skill is routing evidence only. It is not execution evidence,
  proof, trace, or a passed quality gate.
- Keep `security-audit` as the methodology entrypoint and `security-auditor` as
  the independent security gate. Do not merge their responsibilities.
- Superpowers remains the workflow authority for planning, TDD, remediation,
  review, verification, proof, and trace.
- Do not invoke other subagents or claim that another agent executed a check.

## Safe operation

- Do not install scanners, packages, runtimes, or agent bundles.
- Do not run builds, tests, fixtures, scripts, or other target-controlled code
  unless Core confirms that the required operating-system sandbox is enforced.
- Do not probe live systems, exploit findings, change external accounts, or
  perform destructive availability tests.
- Keep source review read-only. Apply any confirmed fix only as a separately
  scoped Superpowers implementation task with regression coverage.
- Never promote an unverified candidate finding to trusted Vault memory.
- Never turn a missing check into a pass or a clean security claim.

## Required validation contract

When the full workflow is available, it must record evidence for each required
step before reporting a completed audit:

1. Check the planned scope and existing source before implementation.
2. Review the actual changed files and connected trust boundaries.
3. Expand the scope when a validated finding crosses into a connected
   in-repository boundary.
4. Validate each candidate finding with source evidence and a reproducible
   local check where safe capabilities permit it.
5. Label unresolved checks and language or scope gaps explicitly.
6. Re-audit after safe fixes and record the regression evidence.
7. Preserve open findings and blockers in the task state; an audit run may
   finish with findings, but an application task must not pass its required
   security completion gate while an in-scope fixable finding is unresolved.

Until SPEC-08-03 removes the corresponding capability blocker, this list is a
future contract, not a claim that these audit steps are implemented. Follow
the PreparePacket's blocker and report the unavailable workflow plainly.
There is no clean-audit result while the required workflow remains unavailable.

## Output Contract

For an unavailable or partial audit, report only:

- the requested mode and known scope;
- the detected language set, including an empty set when unknown;
- checks that actually ran, if any;
- unresolved coverage and capability blockers;
- whether any issue is confirmed, needs validation, or rejected;
- the safe next action.

Use `not_run` when no audit execution evidence exists. Never describe an
unavailable audit as clean, complete, or verified.

## Verification

Verification evidence must identify the exact files, trust boundary, and checks
that were inspected. A required gate counts only when Core records execution
evidence for the current operation. Proof and trace must preserve unknowns,
blockers, and the safe next action. Avoid duplicating sensitive source or
secret values in findings, logs, reports, or Vault notes.

## Provenance

SPEC-08 uses Cloudflare's `security-audit-skill` repository as a methodological
reference. This SPEC-08-01 entrypoint independently states Baron routing and
safety contracts; it does not copy upstream implementation text or install an
upstream runtime. Any later adapted material must retain its applicable
copyright and license notices.
