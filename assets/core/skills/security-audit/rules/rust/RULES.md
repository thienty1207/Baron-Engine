# Rust security rules

Load this pack for Rust source or Cargo evidence. A safe Rust type is useful
evidence, but it does not establish safety at `unsafe`, FFI, authorization, or
external input boundaries.

| Family | Risky source or operation | Protected boundary | Safe pattern | Common false positives | Confirmation evidence |
|---|---|---|---|---|---|
| `input-injection-output` | `query` text built with `format!`, untrusted template/HTML, or unchecked parser output | Database and rendered output | Bind parameters, typed query APIs, validated templates and output encoding | `format!` used only for a log message or static SQL | Trace tainted data to the actual query/render sink and inspect binding semantics |
| `authentication-authorization-ownership` | Load by id without binding principal or `tenant_id` to the operation | User and tenant data | Derive identity from trusted auth context and enforce policy in service/query | Tenant id used for display only | Verify identity provenance and guard on every read/write path |
| `secrets-session-cryptography` | Expose/log `token`, misuse randomness, or roll custom crypto | Secrets, sessions, and key material | Use maintained crates, secret types/providers, redacted logs, explicit expiry | Public token fingerprint; test key unavailable in release | Trace secret lifetime and sink; inspect random source and crypto API use |
| `files-paths-upload-outbound-network` | Join an untrusted `path`, unpack archive entries, or fetch arbitrary URLs | Filesystem and internal network | Canonicalize within root, reject traversal/symlink escape, allowlist destinations | Path derived from a fixed enum | Prove path containment after canonicalization and outbound redirect restrictions |
| `serialization-process-execution` | `Command` invokes a shell, unsafe decoder accepts attacker-controlled data | OS process and parsed object graph | Fixed executable with separate args; bounded typed parser | `Command` with constant args; validated format | Trace input to shell/decoder and establish whether code or privileged data is reachable |
| `concurrency-resource-exhaustion` | Unbounded channels/tasks, lock poisoning ignored, or shared mutable state | Service capacity and synchronization | Bound queues/tasks, handle lock errors, use cancellation and deadlines | Bounded worker pool; immutable shared data | Show reachable task fan-out, blocking, or unsynchronized mutation |
| `dependency-build-deployment` | Unreviewed build script, feature drift, unsafe CI token, or permissive runtime config | Supply chain and deployed process | Review Cargo.lock/features/build.rs; least-privilege CI and runtime | Dev dependency excluded from shipped target | Establish dependency path to target and verify effective build/deploy settings |
| `unsafe-ffi-boundary` | `unsafe` pointer dereference, incorrect length, invalid ownership, or FFI aliasing | Memory safety across Rust/native boundary | Validate null/alignment/length/lifetime invariants and isolate a small unsafe block | Audited wrapper with documented invariant and regression coverage | Inspect caller/callee ABI, pointer provenance, ownership, length, and all safe callers |

## Rust-specific review notes

Inspect every unsafe block and FFI boundary in context. `unsafe` presence is a
review signal, not a finding. Check panic boundaries, `Send`/`Sync` assumptions,
unsafe trait implementations, parser bounds, and async task cancellation.

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
