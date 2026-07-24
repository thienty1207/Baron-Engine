# Deep Module Boundaries

Use this reference only when a task changes a module, API, service seam, or
public abstraction. It helps make the boundary easier for callers to use and
harder to misuse without creating a second workflow or architecture owner.

## Boundary Decisions

1. State the public behavior callers need, using verified project domain terms
   where they exist.
2. Keep the public surface smaller than the hidden implementation complexity.
3. Put invariants at the owning boundary. A caller should not need to remember
   validation, ownership, transaction, or compatibility rules that the module
   can enforce itself.
4. Define one clear entry point and explicit dependency direction. Avoid a
   pass-through module that only renames another API.
5. Design seams that can be tested through public behavior, not private
   implementation details.
6. Preserve compatibility and migration evidence for existing callers before
   changing a proven boundary.

## Reject Weak Abstractions

Do not add a new layer merely because it looks architecturally tidy. Reject it
when it does not hide meaningful complexity, reduce a real duplication, enforce
an invariant, or make behavior easier to test and migrate.

## Evidence Record

Before final response, record:

```text
Public surface: <caller-facing behavior>
Owned invariants: <rules enforced at this boundary>
Dependency direction: <who may depend on whom>
Compatibility: <existing callers and migration evidence>
Domain terms: <verified terms with evidence, or ambiguous/unknown>
Verification: <public behavior tests, smoke checks, or missing proof>
```

Unknown callers, terms, and compatibility promises remain unknown. Do not turn
a disputed domain word into a verified fact solely to make an abstraction sound
cleaner.
