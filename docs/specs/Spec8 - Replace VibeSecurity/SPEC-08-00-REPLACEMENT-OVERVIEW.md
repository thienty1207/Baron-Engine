# SPEC-08 — Replace Vibe Security with Baron Security Audit

**Status:** SPEC-08-01 implemented and verified locally; SPEC-08 overall remains in progress and its remaining child specs are not started.<br>
**Date:** 2026-10-28
**Scope:** Complete runtime replacement of the Vibe security scan with a Baron-native security-audit workflow based on Cloudflare’s security-audit method.

## 1. User outcome

When the user asks an AI to build or change an application, Baron must automatically check the changed code and its nearby trust boundaries for security weaknesses. A confirmed, in-scope weakness that has a safe local fix should be fixed in the same work session, covered by a regression check, and audited again. Core starts with a risk check before implementation and rechecks the actual changed files and connected boundaries before task completion.

The initial required language set is:

- Python
- C++
- Java
- Rust
- Go
- TypeScript
- C#

The user should not have to choose scanners, agent roles, sandbox settings, audit profiles, or implementation details. Baron Core selects a safe bounded mode and explains any coverage gap in ordinary language.

## 2. Meaning of “replace 100%”

“Replace 100%” means:

1. New Baron installs and Core routing use security-audit; the Vibe security scan is no longer a selectable or loaded runtime skill.
2. Existing Baron-managed Vibe assets are migrated safely; user-owned or locally modified files are preserved.
3. The Cloudflare audit methodology becomes the only Baron source-code AppSec audit workflow.
4. The findings from that workflow feed a Baron-owned remediation, regression-test, and re-audit loop.

It does **not** mean that any AI skill can guarantee discovery of every vulnerability. An audit must report exact scope, coverage, unresolved checks, and uncertainty. Cloudflare’s own repository says repeated runs add coverage and that one test run found roughly half of the vulnerabilities found across repeated runs. See [Cloudflare security-audit](https://github.com/cloudflare/security-audit-skill) and its [license](https://github.com/cloudflare/security-audit-skill/blob/main/LICENSE).

## 3. Architecture decision

Port the Cloudflare workflow into Baron-owned Core assets and contracts. Do not install the upstream package as a separate runtime, expose its instructions as global host policy, or import its unrelated tool bootstrap and target-operations behavior.

The Cloudflare audit remains evidence-first and read-only. Baron’s existing Superpowers workflow owns edits: a validated finding becomes a bounded fix task, followed by a regression check and a new audit pass. Preserve the upstream MIT copyright and license notices for any material copied or adapted into Baron, and record attribution in the appropriate NOTICE file.

## 4. Baron invariants to preserve

- Core owns routing, project identity, Task State, durable memory, proof, trace, and quality-gate policy.
- Superpowers remains the workflow authority for planning, TDD, implementation, review, and verification.
- The exactly three mandatory agents remain code-reviewer, security-auditor, and test-engineer.
- security-auditor remains an independent final security gate. It does not become the audit orchestrator, implementation agent, or subagent coordinator.
- A security finding does not count as verified proof merely because a skill or tool is present. Core records evidence only for checks that actually ran.
- Codex and Claude adapters remain thin projections of Core behavior.
- Vault findings remain scoped to the current project capsule. Unverified findings are never promoted to trusted memory.
- User source files, custom skills/agents, unknown configuration keys, and third-party hooks remain preserved.
- Target-controlled builds, tests, or fixtures run only when the platform can enforce the required sandbox. Baron never installs tools automatically or probes live systems.

## 5. Five implementation specs

The five child specs are deliberately bounded by responsibility. They are not separate product alternatives.

| Order | Spec | Responsibility |
|---|---|---|
| 1 | [SPEC-08-01 — Core routing and Vibe retirement](SPEC-08-01-CORE-ROUTING-AND-VIBE-RETIREMENT.md) | Replace the route and migrate managed assets without data loss. |
| 2 | [SPEC-08-05 — Safety, recovery, and failure handling](SPEC-08-05-SAFETY-RECOVERY-AND-FAILURE-HANDLING.md) | Define capability gates, sandbox rules, resumable state, and honest partial results before execution work. |
| 3 | [SPEC-08-02 — Seven-language coverage](SPEC-08-02-SEVEN-LANGUAGE-COVERAGE.md) | Provide explicit detection, rules, and regression fixtures for the required languages. |
| 4 | [SPEC-08-03 — Audit workflow and finding quality](SPEC-08-03-AUDIT-WORKFLOW-AND-FINDING-QUALITY.md) | Implement the six-phase Cloudflare audit method using Baron-owned contracts. |
| 5 | [SPEC-08-04 — Automatic remediation loop](SPEC-08-04-AUTOMATIC-REMEDIATION-AND-REGRESSION-LOOP.md) | Fix confirmed safe findings, test them, and audit the fix. |

## 6. Global acceptance criteria

SPEC-08 is complete only when all are true:

1. Every meaningful application code-change task receives a bounded security check automatically; security-sensitive boundaries trigger the relevant deeper review.
2. All seven required languages are detected and have language-aware coverage tests. Mixed-language repositories account for each in-scope language separately.
3. A confirmed, safely fixable finding in the active task scope is repaired in that task, receives a regression test or equivalent reproducible check, and is re-audited.
4. Findings are distinguished as confirmed, needs_validation, or rejected; partial or blocked work is never reported as a clean complete audit. An audit run can complete with open findings, but the application task cannot pass its security completion gate until required in-scope fixable findings are fixed and rechecked. If a finding is blocked by a product decision or missing safe capability, the task remains BLOCKED and cannot be reported as secure.
5. The security-auditor gate, proof/trace requirements, Memory Firewall, and Superpowers ownership remain intact.
6. Existing installations update transactionally. Modified or ambiguous old assets are not deleted.
7. Current runtime, route, adapter-generated guidance, and tests contain no active Vibe security scan route or fallback. Historical provenance may retain the old name only when clearly labeled historical.
8. Upstream license and copyright notices are preserved for copied or adapted material.
9. Windows and Linux CI exercise route, asset migration, language detection, audit-state, remediation, and failure-recovery regressions.

## 7. Design evidence and limits

Target checkout requested by the user: D:\Working\Baron-Engine, branch main, HEAD cf319a12cfc19e2739303b1b26ee71357ae6c0cd. At inspection, the local origin/main tracking ref was nine commits ahead of this checkout; this was observed from local refs and was not freshly fetched from GitHub. The checkout already contains user changes and untracked files outside this new Spec8 folder. They are out of scope and must be preserved.

Targeted implementation inspection used the separate worktree at 04bda233e66b2b9bae96b23b7f8c6e202850db1f. It found the current skill selection in crates/baron-core/src/control_plane.rs, the managed-skill registry in crates/baron-core/src/asset_lifecycle.rs, compatibility names in crates/baron-core/src/migration.rs and platform.rs, adapter guidance in crates/baron-adapters/src/install.rs, plus routing, context, capability, survey, and lifecycle tests. This was a targeted design inspection, **not** a complete source-reading pass. Each implementation spec must revalidate its touchpoints against the then-current main branch before code changes.

The upstream workflow describes six phases: reconnaissance, coverage-led hunting, candidate validation, structured findings, independent record verification, and target-neutral reporting. Its full workflow requires tool-capable parallel agents, Node.js validators, and an OS-enforced sandbox for target-controlled execution. Baron must preserve the method while implementing equivalent Core-native validation and capability-aware behavior; a missing runtime capability must never be hidden as success. See the [upstream workflow](https://github.com/cloudflare/security-audit-skill/blob/main/skills/security-audit/SKILL.md).

## 8. Non-goals

- Claiming that a clean AI audit proves an application has no vulnerabilities.
- Unauthenticated or live-system penetration testing, exploitation, or availability testing.
- Automatically installing dependencies, static-analysis tools, or agent packages.
- Replacing Superpowers, security-auditor, test-engineer, proof, trace, or Baron routing.
- Supporting languages outside the initial seven before their coverage is defined and tested.

## Status checkpoint — 2026-10-28

SPEC-08-01 is implemented and verified locally on the isolated branch
`codex/spec-08-01-vibe-retirement`. Windows formatting, all 112 workspace test
targets (1,055 passed, 0 failed, 4 ignored), and warnings-denied Clippy passed.
SPEC-08-05, SPEC-08-02, SPEC-08-03, and SPEC-08-04 have not started. SPEC-08
as a whole is not complete. This is a targeted SPEC-08-01 source audit, not a
line-by-line repository-wide source pass. No commit, push, merge, version bump,
or release has been made for this work; the local result is pending owner
review.
