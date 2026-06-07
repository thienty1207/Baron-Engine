# Baron Core Asset Blueprint

This folder describes the core assets Baron must preserve from
`agent-bootstrap`.

It is a blueprint in phase 0, not the final generated adapter output.

## Core Workflow

- `superpowers` is the only workflow core.

## Core Quality Agents

- `code-reviewer`
- `security-auditor`
- `test-engineer`

## Bundled Optional Domain Skills

- `frontend-design`
- `vibe-security-scan`

Optional skills are lazy-loaded only when routing matches the task. They must
not become workflow cores.
