---
id: SSRF
severity_max: CRITICAL
applies_to: go
---

# Go SSRF

## Intent

Go services often use `net/http`, resty, fasthttp, or Colly to fetch URLs. If L1 user input controls the URL without a strict allowlist, the server can be tricked into reaching metadata endpoints, localhost services, or private networks.

## Search patterns

```text
http\.Get\s*\(\s*\w+
http\.Post\s*\(\s*\w+
http\.Head\s*\(\s*\w+
http\.NewRequest\s*\(\s*[^,]+,\s*\w+
\.Do\s*\(\s*req\s*\)
colly\.NewCollector
\.Visit\s*\(\s*\w+
fasthttp\.Get
grpc\.Dial\s*\(\s*\w+
```

## Flag when

- request/query/body/header input reaches outbound URL fetches
- Colly visits user-controlled URLs without `AllowedDomains` or `URLFilters`
- validation only checks string containment or suffixes that attacker domains can bypass
- DNS resolution/private IP checks are absent for cloud or internal-network deployments

## Do not flag when

- destination host comes from a server-owned allowlist
- scheme is restricted, redirects are controlled, and private/link-local/loopback IPs are blocked after DNS resolution

## Fix recommendation

Use explicit host allowlists, restrict schemes to HTTPS, reject private/link-local/loopback IP ranges in a custom `DialContext`, configure Colly `AllowedDomains`, and set timeouts on outbound clients.
