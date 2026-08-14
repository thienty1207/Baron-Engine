# Baron 4.2 Acceptance Record

Generated: 2026-08-14 (UTC)

## Verdict

`promote` for the bounded Baron 4.2 local release contract.

- Raw development repetitions: `100, 100, 100`
- Development gate: `passed`
- Sealed private holdout: `100/100` across 8 cases
- Holdout opened: once for contract `432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a`
- Promotion ready: `true`
- Hard failures: none
- Source revision: `545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d`
- Contract: [baron-4.2-contract.json](baron-4.2-contract.json)
- Development evidence: [baron-4.2-benchmark.json](baron-4.2-benchmark.json)
- Machine-readable acceptance: [baron-4.2-acceptance.json](baron-4.2-acceptance.json)

## What was actually gated

The evaluator exercised current-project provenance, negative/unknown answers,
calibrated semantic retrieval with abstention, task segmentation, duplicate
suppression, poisoning quarantine, temporal conflicts, Wiki citations and
freshness, directional CodeGraph evidence, and dynamic-call unknown behavior.
The private holdout was seeded and executed outside the repository and Vault;
its labels and raw case data are not committed or indexed by Baron runtime.

The release keeps `BARON_ENGINE_GENERATION=4.1` as a whole-engine rollback and
`BARON_ENGINE_GENERATION=4.0` as the explicit per-query safety fallback. A
fallback result never contributes points to the raw 4.2 score.

This is a bounded local correctness contract, not a claim of universal
correctness for unsupported languages, dynamic runtime behavior, or arbitrary
private corpora that were not supplied to the evaluator.
