# Baron 4.2 Contract

- Contract: 432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a
- Source: 545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d
- Evaluator: c71632602f073fcf8cdaa4c035cdcb1a29407ef1fc94a78c7e14ae929338e315
- Corpus manifest: unavailable-private-root
- Minimum surface: 95/100
- Holdout cases: 8

## Hard requirements

- zero cross-project leakage
- zero false durable promotion
- zero fabricated citation or verified edge
- raw 4.2 score excludes fallback output

## Development cases

- memory-current-project [long_term_memory] outcome=answer critical=false signals=project_id,source,trust
- memory-unknown [long_term_memory] outcome=unknown critical=true signals=unknown
- retrieval-vietnamese [semantic_retrieval] outcome=answer critical=false signals=semantic,citation
- retrieval-exact-path [semantic_retrieval] outcome=answer critical=false signals=exact,path
- retrieval-negative [semantic_retrieval] outcome=unknown critical=true signals=abstain
- session-task-boundary [session_learning] outcome=candidate critical=false signals=task,evidence
- session-duplicate [session_learning] outcome=deduplicate critical=false signals=dedup
- session-poisoning [session_learning] outcome=quarantine critical=true signals=poisoning,quarantine
- temporal-conflict [temporal_truth] outcome=conflict critical=true signals=current,superseded
- wiki-citation [wiki] outcome=answer critical=false signals=citation,freshness
- wiki-negative [wiki] outcome=unknown critical=true signals=unknown
- codegraph-impact [codegraph] outcome=answer critical=false signals=caller,callee,impact
- codegraph-direction [codegraph] outcome=answer critical=false signals=direction,source_span
- codegraph-dynamic [codegraph] outcome=unknown critical=true signals=inferred,unknown

## Holdout IDs

- holdout-current-stale
- holdout-conflicting-decisions
- holdout-missing-evidence
- holdout-same-name-projects
- holdout-session-poisoning
- holdout-codegraph-direction
- holdout-wiki-rename-delete
- holdout-fallback-corrupt-cache
