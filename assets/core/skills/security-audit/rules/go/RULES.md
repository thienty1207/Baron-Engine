# Go security rules

Load this pack for Go evidence. Standard-library and third-party web frameworks
are selected only from imports or module evidence in the repository.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | `Query` assembled from request data, unsafe template output, or shell interpolation | Database, browser, and process | SQL placeholders, `html/template` contextual escaping, fixed argv | `Query` with placeholders; `text/template` for non-HTML output | Trace request data through transformations to sink and inspect template/query behavior |
| `authentication-authorization-ownership` | Handler uses caller-supplied id without principal or `tenantID` policy | User and tenant data | Middleware identity plus handler/service object check | Public read; authorization enforced in a verified service | Follow route registration and middleware order to the data operation |
| `secrets-session-cryptography` | Log or return a `token`, weak cookie flags, or custom crypto | Credentials and sessions | Redact structured logs; use maintained crypto and secure cookie defaults | Hash/fingerprint explicitly non-reversible and non-secret | Trace secret source to logs, responses, storage, and expiry/rotation code |
| `files-paths-upload-outbound-network` | `filepath.Join` with request path, unchecked upload, or arbitrary `http.Get` | Filesystem, archive, and internal network | Verify resolved path under root; cap and inspect uploads; allowlist host/redirect | `filepath.Join` of fixed config; client-side URL only | Show attacker control and verify canonical path/egress enforcement |
| `serialization-process-execution` | `exec.Command` runs a shell string or decoder accepts untrusted polymorphic data | OS process and parsed object graph | Invoke fixed executable with separate args; validate schema/type | Direct `exec.Command` with fixed executable and safe args | Trace user input to executable/args/decoder; inspect shell and type controls |
| `concurrency-resource-exhaustion` | Goroutine fan-out, unbounded channel, shared map mutation, or missing `context` deadline | Service capacity and shared state | Bound worker pools, synchronize maps, propagate deadlines/cancellation | Fixed small goroutine fan-out; sync.Map used with valid semantics | Establish concurrent reachability, input-driven cardinality, and release of resources |
| `dependency-build-deployment` | Stale module graph, unsafe replace, broad container privilege, or exposed CI secret | Dependency and runtime supply chain | Review go.sum, module sources, image user/capabilities, scoped secrets | Development-only tools omitted from production image | Inspect module resolution, build image, runtime config, and CI workflow path |

## Go-specific review notes

Review `net/http` handler ordering, Gin/Fiber/Echo/Chi middleware, request body
limits, `context.Context` propagation, TLS configuration, URL redirect policy,
and goroutine lifecycle. Import evidence identifies a framework, not correct
middleware execution.

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
