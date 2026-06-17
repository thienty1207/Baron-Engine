---
name: api-and-interface-design
description: Use when designing or changing APIs, public interfaces, SDK contracts, request or response shapes, versioning, compatibility, webhooks, or boundary behavior.
license: MIT-inspired guidance; Baron-native adaptation
---

# API And Interface Design

This bundled optional domain skill is for API and public-interface work only. Superpowers remains the workflow core, and Baron proof/trace gates still decide completion.

## Use When

- adding or changing REST, GraphQL, RPC, webhook, SDK, CLI, library, or internal service contracts
- choosing request/response shapes, error models, pagination, idempotency, versioning, or compatibility rules
- reviewing whether a boundary is stable enough for callers

Do not use for implementation-only tasks that do not change an interface.

## Baron Contract

- Read repo conventions and current callers before proposing a new contract.
- Mark unknown consumers, compatibility promises, or data ownership as unknown.
- Keep interface decisions local to the project unless the user confirms they are global.
- Do not replace Superpowers, `code-reviewer`, `security-auditor`, or `test-engineer`.

## Review Checklist

- Who calls this interface, and what breaks if it changes?
- Are names, status codes, errors, pagination, retries, idempotency, and auth behavior explicit?
- Are optional fields, nulls, empty values, limits, and validation failures specified?
- Is the migration path clear for old clients?
- Is the verification plan concrete: contract tests, integration tests, schema checks, or smoke calls?

## Output Contract

- Contract summary
- Caller/consumer impact
- Compatibility risks
- Security/data boundary risks
- Recommended shape or change
- Verification evidence or missing proof

## Attribution

Inspired by MIT-licensed API/interface design ideas from `addyosmani/agent-skills`, rewritten as Baron-native optional guidance.
