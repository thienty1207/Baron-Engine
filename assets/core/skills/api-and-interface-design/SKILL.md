---
name: api-and-interface-design
description: Use when designing or changing APIs, public interfaces, SDK contracts, request or response shapes, versioning, compatibility, webhooks, idempotency, pagination, error schema, or auth boundary behavior.
license: MIT-compatible Baron-owned local guidance; attribution lives in NOTICE.md
---

# API And Interface Design

This bundled optional domain skill is for interface boundary work. It is not a workflow skill. Superpowers remains the workflow core for planning, TDD, debugging, review, and verification.

## Baron Contract

- Use this skill only after `.codex/skills/INDEX.md`, `.claude/skills/INDEX.md`, or Baron control-plane routing selects it.
- Do not replace Superpowers, `code-reviewer`, `security-auditor`, or `test-engineer`.
- Treat repository contracts, callers, tests, schemas, docs, and existing traffic examples as evidence.
- Mark unknown callers, unknown compatibility promises, unknown auth requirements, and unknown data owners as unknown.
- Prefer a small compatible change over a clever interface rewrite.
- Record proof and trace when an interface change affects users, clients, integrations, or public behavior.
- Keep project-specific interface facts inside the current Vault capsule unless the user approves global promotion.
- Do not invent consumers, SLAs, versions, or platform guarantees that are not present in repo evidence.

## Use When

- The task adds or changes REST, GraphQL, RPC, WebSocket, webhook, SDK, CLI, library, or service interface behavior.
- The task changes request shape, response shape, status codes, error schema, pagination, filtering, sorting, retries, or idempotency.
- The task changes a public model, DTO, serializer, validation boundary, or generated client contract.
- The task changes auth boundary behavior: who can call it, what roles are allowed, what tenant or resource scope is enforced.
- The task touches compatibility with old clients, mobile apps, partner integrations, background jobs, or external providers.
- The task asks whether an API is clean, stable, maintainable, or ready for product use.
- The task introduces versioning, migration, deprecation, or a new endpoint family.

Do not use this skill for implementation-only refactors that keep all interface behavior identical.

## Read First

- Existing route/controller/handler files for the changed boundary.
- Existing tests, schema files, generated clients, OpenAPI/GraphQL definitions, or typed SDK exports.
- Product Harness story and Active Plan state when the change is medium/high risk.
- Current auth, tenant, permission, and rate-limit behavior around the boundary.
- Call sites in frontend, backend, mobile, scripts, workers, or integration folders.
- README/API docs if they are used by real consumers.
- Recent trace/proof history when a previous attempt changed the same interface.

## Interface Map

Before proposing a contract, answer these from evidence:

- Who calls this interface today?
- Is it internal-only, user-facing, partner-facing, public SDK, or unknown?
- What data enters the boundary and what trust level does it have?
- What data leaves the boundary and who can observe it?
- What auth boundary protects it?
- What compatibility promise already exists?
- What behavior must remain stable for old clients?
- What error states are expected and how are they represented?
- What proof will show this interface works?

## Design Rules

- Names should match the domain language already used by the project.
- Required fields must be genuinely required; optional fields must define default, null, empty, and omission behavior.
- Error schema must be predictable and documented enough for callers to handle.
- Pagination must be stable: define limit, cursor or page strategy, sort order, and max bounds.
- Idempotency must be explicit for payments, webhooks, retries, imports, subscription updates, order creation, and destructive operations.
- Versioning must be planned before breaking clients.
- Compatibility must be preserved unless the user explicitly accepts a breaking change.
- Auth boundary must be enforced server-side, not only hidden in UI.
- Rate limits and abuse controls should be part of public or risky interfaces.
- Observability should expose safe request IDs or correlation IDs without leaking PII.

## Versioning

- Prefer additive changes when clients already exist.
- Use a compatibility window for breaking changes.
- Avoid ambiguous `v2` naming without stating what changed and who migrates.
- Do not silently change meaning of a field.
- For enum changes, define unknown-value handling when clients may lag.
- For response removals, use deprecation first when practical.
- For request validation tightening, check existing clients and data first.

## Pagination And Filtering

- List endpoints must not return unbounded records.
- Prefer cursor pagination for large or frequently changing collections.
- Define default and maximum limit.
- Define stable sorting.
- Document filter interactions and empty-state behavior.
- Verify query performance if the endpoint can touch large tables.
- Mark performance impact as unknown when no query or runtime evidence exists.

## Idempotency

- Require idempotency keys or dedupe records for retry-prone writes.
- Payments, subscription changes, imports, uploads, webhooks, and order creation need explicit duplicate handling.
- Make idempotent response behavior predictable: same response, conflict, or status lookup.
- Store enough evidence to prove dedupe happened without logging secrets.
- Include race-condition proof for high-risk writes.

## Error Schema

- Use consistent machine-readable error codes.
- Include human-readable messages only when safe.
- Avoid leaking stack traces, database names, file paths, provider secrets, or internal IDs.
- Preserve HTTP status semantics.
- Differentiate validation errors, auth failures, permission failures, not found, conflict, rate limit, and dependency failures.
- Ensure clients can recover or show a useful state.

## Auth Boundary

- State who can call the interface.
- State what resource scope, tenant, workspace, org, or ownership rule is enforced.
- Server-side checks are mandatory for protected behavior.
- Do not trust client-supplied role, tenant, plan, price, quota, or ownership fields.
- For admin/user boundaries, include direct API access proof, not just UI proof.
- For Supabase/RLS, verify policy and service-role boundaries if applicable.

## Verification

- Contract tests prove request, response, error schema, pagination, auth boundary, and compatibility behavior.
- Integration tests prove real handler + database/provider behavior when possible.
- Snapshot/schema checks can guard generated clients, but do not replace behavior tests.
- Smoke calls prove route wiring, auth, and serialization.
- Backward-compatibility proof should include an old-shape request or old client call when relevant.
- Negative tests should cover invalid input, missing auth, wrong permission, wrong tenant, and duplicate idempotent request when relevant.
- Baron proof must name exact commands or artifacts used.
- Baron trace must state what interface changed, who is affected, and what remains unknown.

## Output Contract

Return API/interface guidance in this shape:

1. Contract summary: endpoint/interface, caller, request, response, and boundary.
2. Compatibility: old behavior, new behavior, versioning, migration, and compatibility window.
3. Auth boundary: who can call it, what resource scope is enforced, and what is unknown.
4. Data contract: validation, pagination, idempotency, error schema, and privacy.
5. Risks: breaking changes, security risks, performance risks, and unclear ownership.
6. Recommendation: smallest compatible shape or change.
7. Verification: exact tests/smoke checks/proof required or already run.
8. Trace note: what Baron should record for future agents.

## Red Flags

- Endpoint added with no auth boundary.
- List endpoint with no limit or stable ordering.
- Public response silently removes or renames fields.
- Error schema differs per branch with no reason.
- Raw provider/database errors returned to clients.
- Client-supplied `isAdmin`, `tenantId`, `price`, `quota`, or ownership data trusted.
- Payment or webhook write without idempotency.
- API version introduced with no migration story.
- Compatibility risk called "low" without caller evidence.
- Test only covers happy path while boundary risk is high.

## Baron Final Check

- If evidence is thin, say exactly what is unknown.
- If the change is medium/high risk, require core quality gates and trace scoring.
- If security is involved, route `vibe-security-scan` and `security-auditor`.
- If performance is involved, route `performance-optimization` or `web-performance-auditor` as appropriate.
- Do not claim interface readiness until proof and trace support the claim.
