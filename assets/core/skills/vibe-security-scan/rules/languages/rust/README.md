# Rust Security Overlay

These rule files override generic rules when the primary language is Rust.

## Files in this folder

| File | Rule ID | What it specializes |
| --- | --- | --- |
| `02-sql-injection.md` | SQL-INJECTION | SQLx, Diesel raw SQL, SeaORM raw statements, `format!` query construction |
| `08-insecure-deserialization.md` | INSECURE-DESERIALIZATION | untrusted `bincode`, `serde_yaml`, `ron`, `postcard`, unsafe custom deserializers |
| `09-ssrf.md` | SSRF | `reqwest`, `hyper`, `ureq`, `awc`, webhook/proxy/image import URL fetches |
| `10-path-traversal.md` | PATH-TRAVERSAL | `PathBuf::push`, `std::fs`, upload/download paths, archive extraction |
| `15-cors-misconfig.md` | CORS-MISCONFIG | `tower-http`, Actix, Rocket, Axum permissive CORS |
| `17-verbose-error-debug-mode.md` | VERBOSE-ERROR-DEBUG-MODE | `RUST_BACKTRACE`, debug error responses, `anyhow`/`eyre` leaks |
| `21-command-injection.md` | COMMAND-INJECTION | `Command::new("sh")`, shell wrappers, user-controlled command/arg construction |

## Framework coverage

- Axum, Actix Web, Rocket, Warp
- SQLx, Diesel, SeaORM, raw `tokio-postgres`
- `reqwest`, `hyper`, `ureq`
- `tower-http` CORS

## Reasoning still applies

Rust memory safety does not remove appsec risk. Continue to trace L1 user input to sinks, read the full handler, and verify allowlists, parameter binding, canonical path checks, and safe command argument separation before reporting.
