---
id: VERBOSE-ERROR-DEBUG-MODE
severity_max: HIGH
applies_to: rust
---

# Rust Verbose Error / Debug Mode

## Intent

Debug traces and raw error chains can reveal paths, SQL, tokens, provider errors, env names, internal routes, or dependency versions.

## Search patterns

```text
RUST_BACKTRACE
debug_handler
format!\s*\(\s*".*\{:?\}"
eprintln!
tracing::debug!
anyhow::Error
eyre::Report
InternalServerError.*err
```

## Flag when

- stack traces or raw error chains are returned to users
- production config enables debug/backtrace output
- logs or responses include secrets, SQL strings, headers, cookies, or provider responses

## Do not flag when

- detailed errors are internal-only and redacted before user response
- debug output is compile-time or environment-gated away from production

## Fix recommendation

Return generic user-facing errors, log structured redacted details server-side, disable production backtraces, and scrub secrets from tracing fields.
