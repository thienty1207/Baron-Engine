# C++ security rules

Load this pack only for in-scope C++ evidence. Header language comes from build
metadata, compilation databases, and consuming translation units; ambiguous
headers remain explicitly ambiguous.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | Concatenate request data into SQL, shell, format strings, or HTML | Database, process, logs, and rendered output | Prepared statements, typed APIs, bounded formatting, context encoding | Constant format string; bound SQL parameter | Trace untrusted bytes through conversions to the sink and inspect the actual API contract |
| `authentication-authorization-ownership` | Select or mutate an object using only `owner_id` or a caller-supplied identifier | Tenant/object ownership and privileged operations | Check authenticated identity and tenant at the service boundary | Id used only to locate a public record | Verify the caller context and ownership guard precede every sensitive read/write |
| `secrets-session-cryptography` | Expose a `token`, use custom crypto, or retain credentials in long-lived buffers | Secrets, sessions, and authenticated state | Use maintained crypto primitives, secret providers, and explicit zeroization where required | Opaque token identifier; test-only constant | Establish secret source, lifetime, copies, and any logging or network sink |
| `files-paths-upload-outbound-network` | Join a user `path`, trust upload metadata, or fetch a supplied URL | Filesystem, archive extraction, and internal network | Canonicalize under an allowlisted root; cap sizes; validate destinations | Fixed asset path; internal URL from trusted config | Verify canonical path remains inside root after symlink/normalization and redirects are constrained |
| `serialization-process-execution` | `system`, unsafe deserialization, or command string assembly | OS process and object representation | Use explicit argv, allowlisted operations, validated serialization formats | Fixed command with no attacker-controlled argument | Trace source to command/deserializer and prove the reachable control path |
| `concurrency-resource-exhaustion` | Unsynchronized shared state, unbounded buffers, unchecked sizes, or blocking locks | Memory safety and service availability | RAII locks, bounded containers, checked arithmetic, timeouts | Thread-local state; externally capped input | Identify concurrent access or resource source and inspect bound/lock lifetime |
| `dependency-build-deployment` | Unreviewed dependency, unsafe build script, permissive service configuration | Build chain, runtime privilege, and production settings | Review pinned lockfiles, sandbox build steps, least privilege | Development-only tool excluded from release | Link dependency/build configuration to a shipped artifact or privileged action |
| `memory-lifetime-bounds` | Raw pointer arithmetic, unchecked `memcpy`, lifetime escape, or integer-size mismatch | Process memory and object lifetime | Prefer spans/containers, validate lengths, use RAII and checked conversions | Proven fixed-size copy; bounds enforced at caller | Follow length and lifetime values to the access; establish reachable out-of-bounds/use-after-free behavior |

## C++-specific review notes

Review ownership transitions, raw buffers, integer conversions, format strings,
exception boundaries, and FFI. Treat `.h`, `.hh`, `.hpp`, and `.hxx` as headers;
never count a shared physical header twice. If C and C++ consumers disagree or
the build evidence is incomplete, report both interpretations and the gap.

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
