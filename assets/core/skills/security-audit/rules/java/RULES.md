# Java security rules

Load this pack for Java evidence. Spring/Jakarta supplements are selected only
when matching imports or dependency manifests are found.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | `executeQuery` with concatenated input, unsafe expression language, or unencoded output | Database, template, and browser | Prepared statements, parameter binding, context encoding | Bound JDBC placeholders; constant query | Trace request values into query/template and verify actual parameterization or encoding |
| `authentication-authorization-ownership` | Controller loads by id without principal, role, or tenant check | User, tenant, and administrator resources | Method/service authorization plus object-level ownership check | Public endpoints; policy in verified interceptor | Follow request mapping through service and repository for each sensitive operation |
| `secrets-session-cryptography` | Log a `token`, weak cookie/session flags, or custom password hashing | Credentials and authenticated session | Secret provider, redacted structured logs, secure cookie/session settings, maintained KDF | Non-secret correlation id; test profile isolated from production | Confirm secret value type and sink; inspect effective cookie and session configuration |
| `files-paths-upload-outbound-network` | Resolve a user `Path`, accept unchecked upload, or call arbitrary remote URL | Filesystem and internal network | Normalize and contain under root; validate size/type; destination allowlist | Framework storage API with verified containment | Trace user input through normalization, symlink behavior, redirects, and network policy |
| `serialization-process-execution` | `exec` with dynamic command, unsafe Java serialization, or untrusted object binding | Operating system and deserialized object graph | Fixed executable with separate arguments; schema-constrained data binding | Safe argument list; trusted internal serialization | Establish attacker control of arguments/data and inspect reachable deserialization or command path |
| `concurrency-resource-exhaustion` | Shared mutable state, unbounded executor/queue, or missing request limits | Session integrity and service capacity | Proper synchronization, bounded pools/queues, deadlines | Immutable singleton; bounded executor | Demonstrate cross-request race or identify unbounded reachable work |
| `dependency-build-deployment` | Debug mode, permissive CORS, vulnerable transitive dependency, or unsafe Maven/Gradle task | Production app and build chain | Secure profile defaults, reviewed lock/dependency reports, restricted build plugins | Test-only dependency not shipped | Inspect effective profile, dependency graph/lock, CI credentials, and artifact inclusion |

## Java-specific review notes

Review Spring Security filter/method rules, Jakarta filters, object binding,
expression languages, JNDI, reflection, Java serialization, and resource
ownership. Do not infer a security annotation is active without proving the
framework scans it on the reachable endpoint.

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
