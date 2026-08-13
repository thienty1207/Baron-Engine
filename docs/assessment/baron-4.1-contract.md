# Baron 4.1 Benchmark Contract

- Contract: `9a79b2dd385c814127453dabda5edfcf36af16b9ad563a26785fa76518fc3f5f`
- Source revision: `6c490703ad22d4d1b1bee6cb02e78ae34133e6d47e019bb54172e2ca07ff2be2`
- Fixture revision: `ebb642025a2867dc264395b8b217c94d6d8c41f933c1a31d1cde431408bf7883`
- Holdout hash: `8930a49663ca8f3ad52c87d2445e1eecefee26017c9a6288f196883f7035e73c`
- Tencent target: `TencentDB-Agent-Memory` `v2.0.0@0aff21a2d9f2b8a0354aaa80a2e586aab4054562 (surface baseline still required)`
- Surfaces: long_term_memory_l0_l3, semantic_retrieval_grounded_synthesis, automatic_session_learning, wiki, codegraph
- Context token budget: 8000
- Time budget: 10000 ms
- Peak memory budget: 536870912 bytes
- Cost normalized: `true`

The holdout is hash-sealed and must not be used for tuning. A missing Tencent runner or baseline is a hard `target_not_achieved` result.

## Development fixtures

- Cases: memory-l0-l3-resume (long_term_memory_l0_l3), semantic-vietnamese-english (semantic_retrieval_grounded_synthesis), session-evidence-candidates (automatic_session_learning), wiki-entities-links (wiki), codegraph-impact (codegraph)
- Sealed holdout cases: 5
