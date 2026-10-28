# SPEC-08-03 — Audit Workflow and Finding Quality

**Parent:** [SPEC-08-00](SPEC-08-00-REPLACEMENT-OVERVIEW.md)<br>
**Status:** Draft for owner review; implementation not started
**Depends on:** SPEC-08-01, SPEC-08-02, and SPEC-08-05

## Goal

Adopt the six-phase, coverage-led, adversarially validated security-audit method as Baron’s only source-code AppSec audit workflow, while expressing run state and proof through Baron Core.

## Audit phases

1. **Reconnaissance:** pin repository identity, source revision, dirty state, authorized paths, languages/frameworks, trust boundaries, and relevant prior evidence.
2. **Coverage-led hunting:** create a deterministic ledger before reviewing findings; assign bounded units by subsystem, boundary, lifecycle, language, and attack class; reconcile discovered gaps.
3. **Candidate validation:** send each unique candidate to a fresh verifier that tries to disprove it using current source evidence. The finding author cannot verify their own candidate.
4. **Structured output:** record confirmed, needs_validation, and rejected verdicts in a machine-readable, versioned schema. Validate both findings and the coverage ledger before proceeding.
5. **Independent record verification:** independently re-read every record that will be presented as confirmed and reconcile material corrections through validation again.
6. **Target-neutral reporting:** derive the user-facing report from the validated findings and ledger, including covered, skipped, deferred, and out-of-scope surfaces.

The implementation must revalidate the current upstream phase contract before coding. See [Cloudflare’s audit workflow](https://github.com/cloudflare/security-audit-skill/blob/main/skills/security-audit/SKILL.md) and [repository overview](https://github.com/cloudflare/security-audit-skill).

## Baron execution modes

- **Focused (automatic during implementation):** perform a pre-implementation risk check against the planned paths and existing neighboring flows. Before completion, inspect the actual changed application paths and directly connected trust boundaries using relevant language overlays. Report findings and gaps before the task can be called complete.
- **Standard:** bounded subsystem or project audit when the user requests a broad review or Core detects a material security boundary.
- **Deep:** high-stakes or large audit with stronger coverage redundancy and independent validation.
- Scope and mode change breadth, not the evidence bar. Focused mode is not represented as full-repository coverage.
- If evidence points beyond a focused unit, expand through the connected boundary inside the authorized repository scope; if expansion is not safe or the path is outside scope, record the exact gap and blocker before completion.

## Finding contract

A confirmed finding requires all of:

- a named affected principal or attacker capability;
- a protected resource or control boundary;
- a current-source path from input/state to the broken control or sensitive sink;
- a concrete security outcome and impact supported by evidence;
- a minimal recommended correction and a reproducible safe verification.

needs_validation identifies an exact unresolved fact and blocker. It has no severity and is not counted as confirmed. rejected records the candidate claim and the evidence that disproved it.

Severity reflects demonstrated impact and preconditions, not checklist priority. Pattern matches, missing defense-in-depth layers, scanner output, or suspicious names alone are leads, not findings.

Audit completion describes whether all required audit phases and validations ran for the declared scope. It may coexist with confirmed findings. Application-task completion is a separate Baron gate and cannot pass while required in-scope findings remain unfixed or unblocked.

## Core state and evidence integration

- Every run binds to Baron project identity, exact source revision, scope, language set, profile, and current Task State.
- Run progress and blockers are represented in Baron Task State/continuity; audit outputs do not become trusted Vault memory by default.
- Coverage/finding schemas and their validators are Baron-owned, deterministic, versioned, and testable without invoking an AI model. Preserve Cloudflare’s evidence semantics while avoiding an unnecessary Node runtime requirement in Baron.
- A proof receipt can say a check executed only when it records the exact scope/revision, command or review operation, outcome, timestamp, and artifact reference required by Core.
- Codex and Claude receive the same Core selection and evidence contract through their current adapter projections.

## Orchestration boundary

- A host-capable parent may use isolated hunter and verifier roles required by the audit profile.
- Do not change the core security-auditor contract to call subagents or implement fixes.
- If the host cannot provide the required independent validation for a full audit, keep the affected candidates unresolved and mark the run incomplete/blocked. Do not simulate independent review by asking the same agent to repeat its conclusion without a fresh context.
- The test-engineer and security-auditor gates remain independent Baron gates, not aliases for the hunter/verifier roles.

## Acceptance tests

- Phase transitions cannot skip reconnaissance, candidate validation, schema validation, independent record verification, or final reporting.
- Duplicate candidates are consolidated deterministically without hiding distinct root causes.
- Stale findings are revalidated against current source; prior coverage never implies current safety.
- Invalid/missing ledger or finding fields prevent a complete result.
- needs_validation never receives severity or a clean/confirmed label.
- Reports state exact scope, language coverage, unreviewed paths, prior-run limitations, validation mode, and residual checks.
- High-risk completion remains blocked when its required proof, gate receipt, or passing trace is missing.
