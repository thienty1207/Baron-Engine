# C# security rules

Load this pack for C# and .NET project evidence. ASP.NET Core-specific checks
require evidence such as framework references or imports.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | `ExecuteSqlRaw` with user text, unsafe Razor output, or dynamic expression parsing | Database and browser output | Parameterized query, Razor encoding, constrained expression grammar | `ExecuteSqlRaw` with separate parameters | Trace user input into query/template and inspect provider binding or encoding |
| `authentication-authorization-ownership` | Endpoint reads data without `[Authorize]`, policy, or `tenantId` ownership guard | API identity, role, object, and tenant | Explicit endpoint/method policy plus service-level object authorization | Public anonymous endpoint; globally enforced policy verified in startup | Check effective middleware order, fallback policy, endpoint metadata, and data guard |
| `secrets-session-cryptography` | Log or return `token`, weak cookie settings, hard-coded key, or custom crypto | Secrets, cookies, and authenticated session | Secret provider, redaction, secure cookie options, maintained crypto API | Non-secret correlation value; development user-secrets not deployed | Trace credential source and all logs/responses; inspect effective environment settings |
| `files-paths-upload-outbound-network` | Combine request `path`, trust uploaded filename, or fetch arbitrary URL | Filesystem, storage, and internal network | Canonical containment, size/type validation, URL allowlist and redirect policy | Framework storage provider enforcing the boundary | Trace request data and verify canonical path, storage ACL, and egress guard |
| `serialization-process-execution` | `Process.Start` with shell-like string, unsafe polymorphic JSON, or untrusted formatter | OS process and deserialization | Argument list with fixed executable; explicit safe serializer/type policy | Fixed maintenance process; JSON parser with strict schema | Trace attacker-controlled input and prove process/serializer is reachable |
| `concurrency-resource-exhaustion` | Shared mutable state, unbounded channel, missing cancellation/deadline, or oversized body | Application state and service capacity | Correct locking/immutable state, bounded channels, cancellation and size caps | Immutable singleton; framework request limits configured | Establish concurrent reachability and verify actual cap, cancellation, or synchronization |
| `dependency-build-deployment` | Unsafe package source, stale lock, debug errors, permissive CORS, or broad service identity | Build and deployed application | Review lock/package source, production exception policy, explicit CORS, least privilege | Local-only developer exception page with enforced environment check | Inspect effective hosting configuration, dependency resolution, CI, and service identity |

## C#-specific review notes

Review ASP.NET Core authorization middleware and endpoint filters, model
validation, EF Core raw SQL, deserialization options, file upload names, XML
parsers, and `HttpClient` destination/redirect behavior. An attribute or
validation annotation is not proof until the active pipeline enforces it.

## Shared attack-class mapping

| Shared attack class | Rule rows |
|---|---|
| `trust_boundary_and_input_validation` | `input-injection-output` |
| `authentication_authorization_and_ownership` | `authentication-authorization-ownership` |
| `secrets_sessions_and_cryptography` | `secrets-session-cryptography` |
| `query_serialization_and_parsing` | `input-injection-output`, `serialization-process-execution` |
| `files_uploads_and_outbound_network` | `files-paths-upload-outbound-network` |
| `process_execution_and_information_disclosure` | `serialization-process-execution` |
| `concurrency_resources_and_supply_chain` | `concurrency-resource-exhaustion`, `dependency-build-deployment` |
| `deployment_and_security_configuration` | `dependency-build-deployment` |
