# Baron 4.0 Integrated Acceptance

- Project: `59b68ba36271a308126ddf49e5a8891fb6475665a0875b1a21fb094c990b46aa`
- Source revision: `85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69`
- Benchmark: `fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c`
- Score: **100/100**
- Passed: `true`

- `four-surface-score-floor` [memory/wiki/codegraph/security] **100/100** — passed
  - benchmark=fedeace68a4efd6e979a5a217f7b99676f5eaaaa5d58c3bf17c54d9da138e19c; leakage=0
- `security-route-regression` [security-routing] **100/100** — passed
  - cases=9; offensive/missing-auth/project-scope hard stops included
- `static-security-boundary` [defensive-static-appsec] **100/100** — passed
  - files=124; findings=0; dynamic_execution=false
- `memory-no-auto-promotion` [memory-consolidation] **100/100** — passed
  - records=0; candidate staging is reviewable and non-promoting
- `bounded-grounded-handoff` [resume-brief] **100/100** — passed
  - chars=902; project_id=59b68ba36271a308126ddf49e5a8891fb6475665a0875b1a21fb094c990b46aa
- `cache-source-identity` [wiki-codegraph-rebuildable-cache] **100/100** — passed
  - wiki_documents=81; graph_symbols=1699; source_revision=85e1ba7181924d58e207839ce5c0a70f96e3d334aea1739d98e4c06919205c69

Environment: `windows/x86_64` CPUs=12 profile=debug cache_rebuilt=true vault_rebuilt=false
