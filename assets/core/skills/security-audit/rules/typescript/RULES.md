# TypeScript security rules

Load this pack only for TypeScript source or verified TypeScript project
evidence. Browser and Node.js boundaries remain distinct; a generic Node package
manifest alone is low-confidence TypeScript evidence.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | `query` string interpolation, `innerHTML`, unsafe URL, or untrusted template | Database, DOM, and rendered server output | Parameterized query, text APIs, context encoding, validated URL | `query` builder binds parameters; sanitized HTML with verified policy | Trace untrusted browser/server value to sink and inspect binding/encoding context |
| `authentication-authorization-ownership` | Route trusts client `tenantId`, hidden UI state, or token claims without server checks | API identity and tenant/object ownership | Verify signed identity and authorize object on server for each operation | UI role only changes presentation while server guard exists | Follow request to server-side policy and verify object/tenant binding |
| `secrets-session-cryptography` | Expose/log `token`, store secret in client bundle, or weaken cookie/session flags | Credential and session confidentiality | Server-only secret provider, redaction, secure cookies, maintained crypto | Public non-secret identifier; env substitution never bundled | Trace secret into build/client/logs and inspect effective session configuration |
| `files-paths-upload-outbound-network` | Construct a `path`, trust upload metadata, or fetch arbitrary URL | Node filesystem, browser origin, and internal network | Resolve under root; validate upload; allowlist server fetch targets | Browser navigation to user-approved external URL | Distinguish browser/server execution and trace path/URL to privileged sink |
| `serialization-process-execution` | `exec`/shell, dynamic evaluation, unsafe prototype merge, or permissive deserialization | OS process and JS object/runtime | Fixed executable with argv, schema validation, safe object construction | `exec` in test harness; JSON parse followed by validation | Establish production reachability and attacker control of command/object keys |
| `concurrency-resource-exhaustion` | Unbounded async fan-out, missing timeout, unbounded stream/body, or shared mutable state | Event loop and service capacity | Bound concurrency and payloads; use abort/deadline and backpressure | Bounded Promise batch; framework-enforced body limit | Trace remote cardinality and prove timeout/backpressure or its absence |
| `dependency-build-deployment` | Install scripts, floating dependency, unsafe CORS/CSRF config, exposed sourcemap secret | Supply chain, browser origin, and production deployment | Review lockfile/scripts; explicit origins/CSRF; secret-free bundles | Dev-only dependency excluded from production image | Inspect resolved lock, lifecycle scripts, deployment config, and emitted bundle evidence |

## TypeScript-specific review notes

Review runtime validation at type-erasure boundaries, SSR/client separation,
prototype pollution, JSX escaping, `postMessage` origin checks, CSRF, redirect
handling, and Node process/file APIs. Type annotations alone do not validate
untrusted runtime input.

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
