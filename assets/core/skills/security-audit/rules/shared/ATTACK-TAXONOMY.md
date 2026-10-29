# Shared security finding taxonomy

This taxonomy is shared by every language pack. A language pack adds source,
sink, and framework detail; it does not redefine finding categories or severity.
The classes below are review areas, not findings by themselves.

## Attack classes

| ID | Review area | Examples of boundaries to inspect |
|---|---|---|
| `trust_boundary_and_input_validation` | Input, validation, injection, and output encoding | Request and message input, templates, HTML, command/query arguments |
| `authentication_authorization_and_ownership` | Identity, permissions, ownership, and tenant separation | Server-side guards, object-level access, admin and service boundaries |
| `secrets_sessions_and_cryptography` | Credentials, sessions, tokens, cookies, and crypto | Secret storage, transport, rotation, signing, logging, and expiry |
| `query_serialization_and_parsing` | Queries, serialization, and unsafe parsing | SQL/NoSQL construction, parsers, object decoding, schema validation |
| `files_uploads_and_outbound_network` | Files, uploads, archives, URLs, and SSRF | Path containment, content checks, redirects, allowlists, egress |
| `process_execution_and_information_disclosure` | Process launch, diagnostics, logs, and errors | Shell/process boundaries, exception details, debug output, CORS/CSRF |
| `concurrency_resources_and_supply_chain` | Races, resource limits, and dependencies | Shared state, timeouts, size limits, lockfiles, build hooks |
| `deployment_and_security_configuration` | Security controls at build and deployment time | TLS, debug mode, CORS, permissions, CI secrets, hardened defaults |

## Severity policy

Use only the canonical severities `Critical`, `High`, `Medium`, `Low`, and
`Info`, consistent with the mandatory `security-auditor` gate.

- **Critical** — evidenced path enables broad compromise, arbitrary code
  execution, or similarly severe impact without a meaningful additional guard.
- **High** — an exploitable path exposes or alters sensitive data, crosses an
  authorization boundary, or compromises a production service.
- **Medium** — a real security weakness has a bounded or conditional impact and
  a specific affected boundary.
- **Low** — a concrete defense-in-depth issue has limited impact or requires
  uncommon preconditions.
- **Info** — evidence, hardening advice, or an unresolved coverage note that is
  not a confirmed vulnerability.

Do not infer severity from a rule name or keyword. State the source, transform,
sink or protected boundary, reachable conditions, and observed guard. If any
link is unknown, keep the item unconfirmed and describe the missing evidence.

## Confirmation evidence

A confirmed finding needs a repository path and a source-to-sink or broken-guard
chain. Record the relevant code locations, the trust boundary, the actual
control (or its absence), and safe verification evidence. A text match, loaded
rule, fixture presence, or scanner availability is not execution evidence and
does not confirm a finding. Keep unsupported code, unreadable files, skipped
paths, and unrun checks visible as gaps.
