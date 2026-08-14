# Baron 4.2 Development Benchmark

- Report: 38ba0e0f325c466156d3e97f5c12d182cbdab4ca9c77af0d9bfeaacfb7b177ed
- Source: 545cf6ca6ef8a92886c09dc2bfd38c101edf43408a121a87eba408fde81a562d
- Contract: 432846e9aa5088a87a3c8ba2785cfc6c21a8afc12c6536fc8a78447c92b5b39a
- Raw candidate: true
- Holdout opened: false
- Score: 100/100
- Passed cases: 14/14
- Fallback cases: 4

## Case results

- [x] memory-current-project: expected=answer observed=answer signals=3/3 fallback=false evidence=citation=Projects/repo--b45a817d5302/Facts.md source=Projects/repo--b45a817d5302/Facts.md trust=verified provenance=Projects/repo--b45a817d5302/Facts.md#L7 project_id=b45a817d53029864d8e1b268460a176594ceae5cac04330f764cc45d39e31676 excerpt=Verified proof: current project uses semantic retrieval and a safe next action.
- [x] memory-unknown: expected=unknown observed=unknown signals=1/1 fallback=true evidence=abstain:unknown | Baron 4.2 abstained because the query requires unknown evidence: `fact not present in this repository`
- [x] retrieval-vietnamese: expected=answer observed=answer signals=2/2 fallback=false evidence=semantic:calibrated | citation=Projects/repo--b45a817d5302/Facts.md trust=verified provenance=Projects/repo--b45a817d5302/Facts.md#L8 excerpt=Tìm kiếm ngữ nghĩa memory phải có citation and current trust.
- [x] retrieval-exact-path: expected=answer observed=answer signals=2/2 fallback=false evidence=exact:path | citation=crates/baron-core/src/semantic.rs:1 source_span=L1-L1:C4-C9 | citation=crates/baron-core/src/semantic.rs:2 source_span=L2-L2:C4-C9
- [x] retrieval-negative: expected=unknown observed=unknown signals=1/1 fallback=true evidence=abstain:unknown | Baron 4.2 abstained because the query requires unknown evidence: `unrelated quantum database that does not exist`
- [x] session-task-boundary: expected=candidate observed=candidate signals=2/2 fallback=false evidence=task-segments:1 | dedup:1 | noise:0 | candidate-only:true | evidence-spans:3
- [x] session-duplicate: expected=deduplicate observed=deduplicate signals=1/1 fallback=false evidence=task-segments:1 | dedup:1 | noise:0 | candidate-only:true | evidence-spans:3
- [x] session-poisoning: expected=quarantine observed=quarantine signals=2/2 fallback=false evidence=task-segments:1 | dedup:1 | noise:0 | candidate-only:true | evidence-spans:3 | poisoning-quarantine
- [x] temporal-conflict: expected=conflict observed=conflict signals=2/2 fallback=false evidence=current:7 | superseded:0 | contested:2
- [x] wiki-citation: expected=answer observed=answer signals=2/2 fallback=false evidence=citation:docs/ARCHITECTURE.md#L5-L6 | docs/ARCHITECTURE.md#L5-L6 | semantic-confidence:0.823;channels:lexical,ngram | freshness:current | citation:docs/MEMORY.md#L1-L3 | docs/MEMORY.md#L1-L3 | semantic-confidence:0.565;channels:lexical,ngram | freshness:current | citation:docs/ARCHITECTURE.md#L1-L4 | docs/ARCHITECTURE.md#L1-L4 | semantic-confidence:0.605;channels:lexical,ngram | freshness:current
- [x] wiki-negative: expected=unknown observed=unknown signals=1/1 fallback=true evidence=
- [x] codegraph-impact: expected=answer observed=answer signals=3/3 fallback=false evidence=source_span:crates/baron-core/src/semantic.rs:L2-L2:C4-C9 | references -> callee@crates/baron-core/src/semantic.rs:L1-L1:C4-C9 [inferred] | calls -> callee@crates/baron-core/src/semantic.rs:L1-L1:C4-C9 [syntax-evidence] | v4 lexical/fuzzy symbol match; relations remain advisory until current source verification; calibrated score 6.947 with lexical/vector evidence fusion; callee-edge; impact-path; v42-directional-edge-contract | source_span:crates/baron-core/src/semantic.rs:L1-L1:C4-C9 | references <- caller@crates/baron-core/src/semantic.rs:L2-L2:C4-C9 [inferred] | calls <- caller@crates/baron-core/src/semantic.rs:L2-L2:C4-C9 [syntax-evidence] | v4 lexical/fuzzy symbol match; relations remain advisory until current source verification; calibrated score 6.947 with lexical/vector evidence fusion; caller-edge; impact-path; v42-directional-edge-contract
- [x] codegraph-direction: expected=answer observed=answer signals=2/2 fallback=false evidence=source_span:crates/baron-core/src/semantic.rs:L2-L2:C4-C9 | references -> callee@crates/baron-core/src/semantic.rs:L1-L1:C4-C9 [inferred] | calls -> callee@crates/baron-core/src/semantic.rs:L1-L1:C4-C9 [syntax-evidence] | v4 lexical/fuzzy symbol match; relations remain advisory until current source verification; calibrated score 8.641 with lexical/vector evidence fusion; callee-edge; impact-path; v42-directional-edge-contract | source_span:crates/baron-core/src/semantic.rs:L1-L1:C4-C9 | references <- caller@crates/baron-core/src/semantic.rs:L2-L2:C4-C9 [inferred] | calls <- caller@crates/baron-core/src/semantic.rs:L2-L2:C4-C9 [syntax-evidence] | v4 lexical/fuzzy symbol match; relations remain advisory until current source verification; calibrated score 7.894 with lexical/vector evidence fusion; caller-edge; impact-path; v42-directional-edge-contract
- [x] codegraph-dynamic: expected=unknown observed=unknown signals=2/2 fallback=true evidence=inferred:unknown

## Hard failures

- none
