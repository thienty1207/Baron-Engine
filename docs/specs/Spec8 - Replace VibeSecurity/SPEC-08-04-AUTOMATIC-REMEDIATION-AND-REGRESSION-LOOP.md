# SPEC-08-04 — Automatic Remediation and Regression Loop

**Parent:** [SPEC-08-00](SPEC-08-00-REPLACEMENT-OVERVIEW.md)<br>
**Status:** Draft for owner review; implementation not started
**Depends on:** SPEC-08-03 and SPEC-08-05

## Goal

Give the user the requested end-to-end result: Baron detects a confirmed in-scope vulnerability during implementation, the AI applies a safe fix immediately, and Baron verifies that the fix works and closes the original path.

## Separation of responsibilities

Cloudflare’s audit is evidence gathering and reporting. It does not authorize edits to the target. Baron adds a separate remediation stage using the existing Superpowers/TDD implementation flow:

**audit candidate → independent validation → confirmed finding → bounded fix task → regression check → targeted re-audit → security-auditor gate → proof/trace**

The audit agent does not become the source-code editor, and the core security-auditor remains a review gate rather than an implementer.

## Automatic-fix policy

- Automatically fix confirmed vulnerabilities that are in the active repository scope and have a narrow, behavior-preserving correction.
- The implementation agent must read the affected source and surrounding data flow; it must not paste a generic fix based only on a rule name.
- Write a regression test or deterministic safe fixture that fails before the fix and passes after it, when the repository’s test surface permits.
- Preserve security controls. Do not “fix” a finding by disabling validation, authorization, CSRF, TLS, rate limits, audit logging, or safe defaults.
- After a fix, re-run the relevant targeted audit and verification. A successful patch without re-audit does not close the finding.
- When a candidate points to a directly connected path outside the initial focused scan, expand the audit scope within the same authorized repository before deciding whether a fix is needed. Findings still outside the authorized scope remain visible and prioritized; do not silently claim they were fixed.
- Never auto-fix rejected findings or treat needs_validation as confirmed. A safe, reversible hardening may be suggested separately, but it does not change the verdict.

## Stop for a product decision

Do not make an implicit product/security tradeoff when a correction requires a breaking public API, data-model migration/backfill, permission-policy change, irreversible user-data operation, production configuration, external account, or live system access. Explain the user impact in plain language, keep the finding open, and ask for the specific product decision needed.

The user’s request authorizes routine, local source fixes discovered within the active implementation task. It does not authorize destructive data changes, production actions, dependency installation, network probing, or release/merge actions.

## Bounded fix-and-recheck loop

- Use a bounded maximum of three fix/re-audit cycles for one finding in one task.
- Each cycle records the source revision/diff, regression check, audit result, and remaining blocker.
- If the finding remains or a new regression appears after the third cycle, stop automatic iteration, preserve the current evidence, mark the task blocked or incomplete, and report the next safe action.
- Group fixes only when they share a root cause and one regression test can prove the shared correction. Keep independent findings independently traceable.
- Do not close an issue because the code changed; close it only when the verifier observes that the vulnerable path is removed or adequately controlled.

## Completion criteria

A fixed finding is complete only when:

1. The original source path and boundary are rechecked.
2. A regression test or safe local proof passes.
3. The targeted re-audit no longer confirms the issue.
4. The independent security-auditor gate records its result.
5. Required Core proof and trace evidence are valid.

If any item is missing, retain the finding with an explicit state such as needs_validation, blocked, or incomplete; do not report the application or task as fully secure. The audit run may be complete with confirmed findings, but the application task security gate cannot pass until required in-scope fixable findings are closed and rechecked. If safe remediation is blocked by a product decision or missing capability, the task remains BLOCKED and cannot be reported as secure.
