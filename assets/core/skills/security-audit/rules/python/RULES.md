# Python security rules

Load this pack only when Python source or a Python project manifest is in the
current coverage inventory. Framework supplements require matching repository
evidence. Rules identify review paths; they do not confirm findings by name.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | `execute` built from request text, dynamic template input, or unescaped HTML | Database query, template context, or browser output | Bound query parameters, framework autoescaping, context-specific encoding | Parameterized `execute` calls; trusted constant fragments | Trace the input to the query/template sink and inspect parameter binding or encoding |
| `authentication-authorization-ownership` | Lookup by object id without checking owner, tenant, or role | User, tenant, and administrator data | Enforce server-side object policy before read/write; include tenant in query | Public records; policy enforced in a verified shared dependency | Follow route to service/query and verify the guard runs for every operation |
| `secrets-session-cryptography` | Log or return a `token`; weak signing, cookie, or password settings | Credentials, sessions, and user identity | Secret manager/env injection, redacted logs, secure cookie flags, maintained crypto API | Test-only tokens; redaction helper names without sensitive output | Confirm secret provenance and reachable sink; inspect cookie/signature configuration |
| `files-paths-upload-outbound-network` | Open a user-selected `path`, extract archives, accept uploads, or fetch arbitrary URLs | Filesystem and internal network | Resolve beneath an approved root; validate size/type; use URL allowlists and redirect checks | `path` used only for a fixed resource; framework-protected upload | Trace path/URL from untrusted input and verify canonical containment or egress policy |
| `serialization-process-execution` | `subprocess` with `shell=True`, dynamic import/eval, or unsafe object deserialization | OS process, Python runtime, and parsed object graph | Pass fixed executable and argument vector; use schema-constrained formats | Constant administrative command; safe JSON decoding | Trace attacker-controlled values into process, import, or decoder and establish reachability |
| `concurrency-resource-exhaustion` | Unbounded work, missing `lock`, unbounded queues, or blocking request work | Shared state and service capacity | Bound inputs/timeouts/queues; use correct synchronization and cancellation | Small bounded batch; immutable state | Show reachable concurrency and missing cap/lock, or demonstrate race in local test evidence |
| `dependency-build-deployment` | Insecure `DEBUG`, permissive CORS, unsafe build hooks, or floating dependencies | Production configuration and software supply chain | Secure defaults, pinned reviewed dependencies, scoped CI credentials | Local-only debug configuration with enforced environment separation | Inspect effective deployment config, lockfile, build scripts, and CI secret exposure |

## Python-specific review notes

Review `pickle`, unsafe YAML loaders, `eval`/`exec`, dynamic module loading,
subprocess APIs, template autoescape overrides, Django ORM raw methods, Flask
session configuration, and FastAPI dependency authorization. Confirm behavior
from source and configuration; do not treat a matching API name as a finding.

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
