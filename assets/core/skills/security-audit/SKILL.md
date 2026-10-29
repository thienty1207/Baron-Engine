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

SPEC-08-02 supplies bounded repository inventory plus local language rules and
static regression fixtures for Python, C++, Java, Rust, Go, TypeScript, and
C#. SPEC-08-03 adds the versioned six-phase run/finding contract, deterministic
Core validator, independent-review rules, and target-neutral report model.
SPEC-08-05 implements fail-closed capability reporting, checkpoint
revalidation, and safe report publication. The operating-system sandbox
backend is still unverified, so target audit execution remains `not_run`.
Inventory, schemas, CLI validation, and corpus presence do not mean an audit
ran, passed, or found no issues.

The current execution blocker is `sandbox_backend_unverified`. Preserve it and
the individual missing-control codes in the host projection; do not translate
them into a pass or an audit result.

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
- Bind a run to the exact project ID, full source revision, dirty-state digest
  when dirty, authorized scope, in-scope language set, profile, task ID,
  operation ID, and digest of the current Task State. Repeat reconnaissance if
  any bound source or task state changes. Bind every candidate and independent
  record review to the SHA-256 digest of that complete run identity; the same
  Git commit with different dirty files or Task State is a different review
  context.
- Follow the six required phases in order: reconnaissance, coverage-led
  hunting, fresh candidate validation, structured output validation,
  independent record verification, and target-neutral reporting. Never skip,
  reorder, or represent an incomplete phase as complete.
- Create and validate the deterministic coverage ledger before submitting any
  finding candidate. Each unit identifies subsystem, boundary, lifecycle,
  language, attack class, scope path, disposition, and evidence or reason.
- Use `baron security-audit validate-run` with versioned run JSON on stdin
  before accepting structured output. This command checks schema and workflow
  invariants only; it does not verify artifact existence, execute source, or
  create proof/gate/trace evidence. Run IDs are canonical UUIDs, and CLI or
  continuity output contains only the run ID digest.
- Keep candidate verifiers distinct from every candidate author. Deep mode
  requires two distinct fresh verifiers per candidate. A separate fresh
  verifier must re-read each confirmed record. Material corrections invalidate
  later phase receipts and require candidate validation again.
- A confirmed review must keep its source lines and evidence within the
  candidate's cited range. A rejection must preserve the exact candidate claim
  and use disproof evidence linked to the candidate and within authorized
  scope.
- Use Baron continuity checkpointing for progress, bound to the exact active
  operation. Candidate claims and source excerpts do not become trusted Vault
  memory.
- Before implementation, assess the planned scope and existing source for
  material trust-boundary risks. If a validated issue crosses into a connected
  in-repository boundary, expand the review scope before remediation.
- Enter remediation only for an in-scope finding whose final candidate verdict
  is `confirmed` and whose independent record review is accepted without a
  correction requiring revalidation. `rejected` and `needs_validation` findings
  never authorize a fix.
- Keep the audit role read-only. The implementation agent must inspect the
  affected source and connected data flow, then use the existing Superpowers
  TDD flow for a narrow fix and a regression check that fails before the fix
  when the repository permits it. Do not paste a generic rule-name fix or
  weaken an existing security control.
- After a regression check passes, target re-audit the exact post-fix source and
  boundary, then use the separate `security-auditor` gate and current-operation
  Core proof and trace. A changed file or serialized pass claim does not close
  the finding. Run no more than three fix/re-audit cycles per finding; if it
  remains confirmed, a regression fails, or a later check is unresolved, stop
  and retain an explicit blocked or incomplete state.
- Stop for a product/security decision when the fix changes a public contract,
  data model, permission policy, production setting, external account, or
  performs an irreversible user-data action. Explain the impact and leave the
  finding open until the decision is supplied.
- `baron security-audit validate-remediation` checks bounded schema and ordering
  only. Its result is not a test, re-audit, independent gate, proof, trace, edit
  authorization, or finding closure.
- Read `language_coverage` for per-language confidence, scope, source paths,
  evidence, rule-pack path, planned attack classes, and gaps. `detected_languages`
  is repository inventory and may include languages outside the requested
  scope.
- Read `scope_exclusions`, `stack_signals`, `unsupported_evidence`, and
  `coverage_gaps` before describing coverage. An exclusion is valid only when
  Core records its path and reason; an ambiguous header or unsupported source
  remains a gap.
- Load only the `rules/<language>/RULES.md` pack for each in-scope language.
  Load `FRAMEWORKS.md` only when a matching stack signal is present. The shared
  `rules/shared/ATTACK-TAXONOMY.md` is authoritative for attack categories and
  severity; language packs may not invent a severity scale.
- The framework signal catalog is intentionally non-exhaustive. Preserve the
  `framework_detection_catalog_non_exhaustive_requires_validation` gap for
  each in-scope detected language. A missing known signal does not prove that
  the repository has no framework. Record framework or construct evidence
  without a reliable matching rule as `needs_validation`; never silently use a
  weaker language-only rule pack as a substitute.
- Files under `fixtures/` are static examples for regression contracts. Never
  execute them, treat their presence as a finding, or use them as proof that a
  real repository is clean.
- Fixture `expected_outcome` labels define the future candidate-validation
  contract. SPEC-08-02 checks these labels and fixture structure as data only;
  it does not execute a vulnerability classifier or prove that a model obeys
  them. Do not claim safe-pattern behavior until the audit workflow validates
  actual candidate output against these cases.
- Treat `checks_run` as execution evidence only when Core records a check and
  its evidence. An empty list means no audit checks ran.
- Treat a missing language, path, or capability as unknown and report it.
- Preserve low-confidence Node manifest-only TypeScript evidence as a gap; do
  not silently upgrade it to confirmed TypeScript source.
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
- While `sandbox_backend_unverified` is present, preserve remediation and target
  execution as `not_run`: do not edit target source or run target builds, tests,
  fixtures, scanners, or scripts. Keep every finding open and report the
  missing controls. Core contracts and checkpoints do not establish that an
  operating-system sandbox is active.
- Never promote an unverified candidate finding to trusted Vault memory.
- Never turn a missing check into a pass or a clean security claim.

## Required validation contract

The workflow contract records each required phase before reporting a completed
audit:

1. Reconnaissance pins source, dirty state, identity, scope, languages, profile,
   Task State, and prior-run limitations.
2. Coverage-led hunting reviews ledger units and records explicit covered,
   skipped, deferred, or out-of-scope results before findings are finalized.
3. Candidate validation asks a fresh verifier to challenge each candidate.
4. Structured output validates the versioned finding records and ledger.
5. Independent record verification re-reads each proposed confirmed record;
   material corrections return through candidate validation.
6. Target-neutral reporting derives findings, coverage, and residual checks
   from the validated records.

Automatic repair, regression checks, and re-audit after fixes belong to
SPEC-08-04. Until that spec is implemented, keep application-task completion
blocked when its security gate requires an unresolved in-scope fix.

The contract is implemented locally but not yet merged. Target-controlled
execution is unavailable because no sandbox backend has verified every
required control. Follow PreparePacket's blocker and report unavailable
execution plainly. There is no clean audit result while execution remains
unavailable.

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
