---
name: vibe-security-scan
description: Use when reviewing defensive security risks in code or config, especially auth, API routes, server actions, secrets, .env, Supabase/RLS/storage, uploads, payments, subscriptions, quotas, dependencies, CORS, JWT, rate limits, access control, tenants, admin/user permissions, or production security readiness.
license: MIT; adapted from tanviet12/vbsec
---

# Vibe Security Scan

This bundled optional domain skill provides a defensive appsec scan for AI-assisted codebases. It is adapted from `tanviet12/vbsec` and tuned for Baron-managed projects.

Public source: https://github.com/tanviet12/vbsec
License: MIT, copyright Bui Tan Viet. See `LICENSE.txt`.

## Compatibility Guardrails

- Superpowers remains the workflow authority for planning, TDD, debugging, review, and verification.
- This skill is a security scan playbook, not a workflow skill and not a replacement for Superpowers.
- The core `security-auditor` remains the final independent security gate when the parent task needs review.
- Findings must feed Baron proof and trace quality gates before high-risk work can be called complete.
- Respect the Baron Memory Firewall: keep project findings inside the current project capsule and never promote unverified findings to global memory.
- Follow `AGENTS.md`, `.codex/INDEX.md`, `.codex/skills/INDEX.md`, and `.codex/agents/INDEX.md` first.
- Do not attack live systems, bypass authorization, exfiltrate data, or provide weaponized exploit steps.
- Never write secrets, private tokens, cookies, session data, or sensitive user data into reports or vault memory.
- Treat grep matches as leads, not findings. Confirm behavior from code and configuration before reporting.

## When To Use Automatically

Use this skill without waiting for the user to name it when the task touches:

- auth, login, signup, password reset, sessions, OAuth, JWT, cookies, roles, tenant isolation, admin/user permissions
- API routes, server actions, webhooks, backend handlers, Supabase RLS, storage buckets, service role keys
- uploads, file paths, external URLs, SSRF-prone fetch/proxy/import flows
- payment, subscription, quota, reward, order, wallet, or idempotency-sensitive flows
- `.env`, secrets, tokens, keys, logging, error handling, production readiness, dependency/package changes
- CORS, rate limits, brute-force protection, access control, SQL/ORM/raw query code, deserialization, command execution

Do not use this skill for purely visual UI work, copy changes, local-only refactors, or test-only edits unless they touch a security-sensitive boundary.

## Scan Modes

- Focused scan: use when the task touches a narrow surface. Load only the relevant rule files and nearby code.
- Full scan: use when the user asks for a security audit, production readiness review, or broad appsec scan. Load all generic rules, then language overlays for detected languages.
- Large scan: for large repos, read `workflows/large-review-sequential.md` and scan in bounded chunks. Do not create chunks from this skill.

## Language Routing

Always read `references/language-detection.md` for non-trivial scans.

Supported overlays:

- TypeScript/JavaScript: `rules/languages/typescript/`
- Python: `rules/languages/python/`
- PHP: `rules/languages/php/`
- Go: `rules/languages/go/`
- Rust: `rules/languages/rust/`

If no overlay fits, use `rules/generic/`.

## Canonical Rule Set

Use only these canonical rule IDs. If an issue is real but does not fit perfectly, map it to the closest canonical rule instead of inventing a new category.

| # | Rule ID | Max severity |
| --- | --- | --- |
| 1 | HARDCODED-SECRET | CRITICAL |
| 2 | SQL-INJECTION | CRITICAL |
| 3 | XSS | HIGH |
| 4 | IDOR | HIGH |
| 5 | SLOPSQUATTING | CRITICAL |
| 6 | BRUTE-FORCE | HIGH |
| 7 | MASS-ASSIGNMENT | CRITICAL |
| 8 | INSECURE-DESERIALIZATION | CRITICAL |
| 9 | SSRF | HIGH |
| 10 | PATH-TRAVERSAL | HIGH |
| 11 | CSRF | HIGH |
| 12 | BROKEN-ACCESS-CONTROL | CRITICAL |
| 13 | WEAK-PASSWORD-HASHING | CRITICAL |
| 14 | JWT-NONE-ALGORITHM | CRITICAL |
| 15 | CORS-MISCONFIG | HIGH |
| 16 | UNRESTRICTED-FILE-UPLOAD | CRITICAL |
| 17 | VERBOSE-ERROR-DEBUG-MODE | HIGH |
| 18 | MISSING-RATE-LIMIT | HIGH |
| 19 | RACE-CONDITION | HIGH |
| 20 | OUTDATED-DEPENDENCY | HIGH |
| 21 | COMMAND-INJECTION | CRITICAL |

## Evidence-First Workflow

1. Confirm the authorized local repo path and requested scope.
2. Map the attack surface: auth, authorization, API/server actions, database access, uploads, external URLs, background jobs, webhooks, dependencies, env/config, logging, and vault-sensitive memory.
3. Start from trust boundaries and reason with STRIDE before enumerating findings.
4. Detect languages and load the relevant generic rules plus overlays.
5. Search for leads with `rg`, then read surrounding code and data flow before calling anything a finding.
6. Classify data trust using `references/data-flow-classification.md`.
7. Check AI/LLM surfaces when present: prompt injection, tool permissions, secret/context leakage, model output into SQL/shell/HTML/file paths, recursion, and rate limits.
8. Check supply-chain risk when dependencies change: typosquats, postinstall scripts, lockfile drift, abandoned packages, and known CVEs when tooling is available.
9. Separate confirmed findings from unknowns, assumptions, skipped checks, and false positives.
10. Recommend fixes that preserve controls; never suggest disabling validation, auth, CSRF, rate limits, TLS, RLS, or audit logging as a shortcut.

## Output Contract

Write reports in Vietnamese unless the user asks otherwise.

Start with:

- `KET LUAN: DAT`, `KET LUAN: CAN SUA`, or `KET LUAN: KHONG DAT`
- Severity counts: Critical, High, Medium, Low, Info
- Scope reviewed and scope not reviewed

For each confirmed finding include:

- Severity
- Location
- Evidence
- Impact
- Safe abuse path, without weaponized payloads
- Recommended fix
- Verification step

End with:

- commands run
- files or surfaces reviewed
- unknowns and residual checks
- whether the core `security-auditor` should perform a final independent gate review
- Baron proof/trace gaps that block high-risk completion

## References

- `references/data-flow-classification.md`: trust levels and source/sink reasoning
- `references/language-detection.md`: language and overlay selection
- `references/output-format.md`: fuller report structure
- `workflows/small-review.md`: focused/small scan flow
- `workflows/large-review-sequential.md`: bounded large scan flow
- `rules/generic/`: canonical rules
- `rules/languages/`: language-specific overlays

## Additional Attribution

Security audit rubric ideas such as trust-boundary-first review, STRIDE framing, AI/LLM security checks, webhook/OAuth/SSRF, and supply-chain review are informed by MIT-licensed `addyosmani/agent-skills`, rewritten as Baron-native defensive guidance.
