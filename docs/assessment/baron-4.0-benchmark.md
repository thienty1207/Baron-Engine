# Baron 4.0 Intelligence Benchmark

- Report: `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`
- Source revision: `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`
- Fixture revision: `baron-4.0-fixtures-v1`
- Cross-project leakage: `0`

## Environment

- OS/arch: `windows/x86_64`
- Pointer width: `64`
- CPUs: `12`
- Profile: `debug`
- Cache rebuilt by runner: `true`
- Vault rebuilt by runner: `false`

## Baron 3.8 baseline scores

- memory: **100/100** (4 cases; metric floor `true`)
- wiki: **88/100** (4 cases; metric floor `false`)
- codegraph: **60/100** (4 cases; metric floor `false`)
- security: **100/100** (4 cases; metric floor `true`)
## Candidate scores

- Available: `true`
- Note: Candidate 4.0 engines ran independently for sixteen frozen cases across memory, Wiki, CodeGraph, and security routing; promotion still requires every score and hard gate to pass.
- memory: **100/100** (4 cases)
- wiki: **100/100** (4 cases)
- codegraph: **100/100** (4 cases)
- security: **100/100** (4 cases)

- Promotion-ready: `true`

## Case comparison

- `memory-resume-contract` (Memory): 3.8 `100` -> 4.0 `100`; expectations `4`/`4`
- `memory-trust-contract` (Memory): 3.8 `100` -> 4.0 `100`; expectations `4`/`4`
- `memory-vietnamese-handoff-contract` (Memory): 3.8 `100` -> 4.0 `100`; expectations `3`/`3`
- `memory-exact-path-contract` (Memory): 3.8 `100` -> 4.0 `100`; expectations `3`/`3`
- `wiki-citation-contract` (Wiki): 3.8 `100` -> 4.0 `100`; expectations `3`/`3`
- `wiki-freshness-contract` (Wiki): 3.8 `80` -> 4.0 `100`; expectations `5`/`5`
- `wiki-mixed-language-contract` (Wiki): 3.8 `75` -> 4.0 `100`; expectations `4`/`4`
- `wiki-injection-boundary-contract` (Wiki): 3.8 `100` -> 4.0 `100`; expectations `4`/`4`
- `codegraph-impact-contract` (CodeGraph): 3.8 `100` -> 4.0 `100`; expectations `2`/`2`
- `codegraph-call-contract` (CodeGraph): 3.8 `60` -> 4.0 `100`; expectations `5`/`5`
- `codegraph-symbol-span-contract` (CodeGraph): 3.8 `33` -> 4.0 `100`; expectations `3`/`3`
- `codegraph-project-isolation-contract` (CodeGraph): 3.8 `50` -> 4.0 `100`; expectations `2`/`2`
- `security-routing-contract` (Security): 3.8 `100` -> 4.0 `100`; expectations `3`/`3`
- `security-source-appsec-contract` (Security): 3.8 `100` -> 4.0 `100`; expectations `3`/`3`
- `security-oauth-source-contract` (Security): 3.8 `100` -> 4.0 `100`; expectations `4`/`4`
- `security-reverse-static-contract` (Security): 3.8 `100` -> 4.0 `100`; expectations `4`/`4`
