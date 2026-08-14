# Baron 4.2 Phase 88 Audit

- Source: 545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d
- Contract: \\?\D:\Tools\CLI\Baron-Engine\docs\assessment\baron-4.2-contract.json
- Report: \\?\D:\Tools\CLI\Baron-Engine\docs\assessment\baron-4.2-benchmark.json
- Baseline 4.1: v4.1.0 public baseline; seeded Phase 86 evidence is retained but is not real-corpus correctness proof
- Baseline 4.0: BARON_ENGINE_GENERATION=4.0

## Known 4.1 gaps

- holdout identifiers were sealed but were not independently executed by the 4.1 runner
- Wiki and CodeGraph checks mostly proved non-empty hits/edges rather than answer correctness
- positive RRF rank could keep an irrelevant candidate without calibrated abstention
- session import and learning lacked task-level gold scoring and visible omission receipts
- fallback selection was primarily technical-error based rather than quality-gate based

## Hard requirements

- raw 4.2 is scored independently from fallback output
- zero cross-project leakage, false durable promotion, and fabricated evidence
- real redacted sessions plus an executable sealed holdout
- no version promotion before Phase 99 and Phase 100
