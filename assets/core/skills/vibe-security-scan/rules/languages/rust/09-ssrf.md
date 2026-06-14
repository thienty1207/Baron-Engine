---
id: SSRF
severity_max: HIGH
applies_to: rust
---

# Rust SSRF

## Intent

Server-side Rust HTTP clients can fetch attacker-controlled URLs and expose internal services, metadata endpoints, or private networks.

## Search patterns

```text
reqwest::get
Client::new\(\).*\.get
hyper::Client
ureq::get
awc::Client
Url::parse
```

## Flag when

- L1 input controls URL, host, scheme, redirect target, webhook callback, image import, or proxy destination
- there is no scheme, host, DNS/IP, private-network, or redirect allowlist
- code can reach cloud metadata endpoints or internal services

## Do not flag when

- destination is selected from a server-owned allowlist
- host is fixed and only safe path/query pieces are user-controlled and encoded

## Fix recommendation

Allowlist schemes and hosts, block private/link-local IP ranges after DNS resolution, disable or validate redirects, and use fixed outbound destinations for webhooks/imports.
