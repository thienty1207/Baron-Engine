# Baron 4.1 Benchmark Contract

- Contract: `86054c9a45c7d61df91b8b1468ed13347ef96a66091f69a7c404c646dab62af2`
- Source revision: `cd0f21bc916bd9e8c607069442eeccfab52d1c700956ec613c1846b07831141d`
- Fixture revision: `ebb642025a2867dc264395b8b217c94d6d8c41f933c1a31d1cde431408bf7883`
- Holdout hash: `8930a49663ca8f3ad52c87d2445e1eecefee26017c9a6288f196883f7035e73c`
- External comparison: `optional-reference-only` `not-compared-by-4.1-release-gate`
- Surfaces: long_term_memory_l0_l3, semantic_retrieval_grounded_synthesis, automatic_session_learning, wiki, codegraph
- Context token budget: 8000
- Time budget: 10000 ms
- Peak memory budget: 536870912 bytes
- Cost normalized: `true`

The holdout is hash-sealed and must not be used for tuning. Baron 4.1 release acceptance is based on the five local surfaces and resource gates; external comparisons are optional and non-blocking.

## Development fixtures

- Cases: memory-l0-l3-resume (long_term_memory_l0_l3), semantic-vietnamese-english (semantic_retrieval_grounded_synthesis), session-evidence-candidates (automatic_session_learning), wiki-entities-links (wiki), codegraph-impact (codegraph)
- Sealed holdout cases: 5
