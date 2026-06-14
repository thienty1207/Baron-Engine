---
id: CORS-MISCONFIG
severity_max: HIGH
applies_to: rust
---

# Rust CORS Misconfiguration

## Intent

Rust web services can expose sensitive APIs when CORS is configured to allow arbitrary origins, especially with credentials.

## Search patterns

```text
CorsLayer::permissive
Any
allow_origin
allow_credentials
actix_cors::Cors::permissive
rocket_cors
```

## Flag when

- wildcard/permissive origins are used on authenticated APIs
- credentials are allowed with broad or reflected origins
- production config accepts arbitrary origin headers

## Do not flag when

- public unauthenticated static/API resources intentionally allow wildcard CORS
- allowed origins are explicit per environment

## Fix recommendation

Use explicit production origins, avoid credentialed wildcard CORS, and keep local development permissiveness out of production config.
