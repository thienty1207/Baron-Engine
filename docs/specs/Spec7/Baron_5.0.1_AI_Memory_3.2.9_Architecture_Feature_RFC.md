# RFC — Baron Engine 5.0.1 × AI-Memory 3.2.9

- **Chủ đề:** So sánh kiến trúc, chọn feature có giá trị, quản trị rủi ro, phân rã workstream và kiểm chứng chống lỗi logic.
- **Trạng thái:** `REFERENCE ONLY / DEFERRED FUTURE DIRECTION` — RFC tham khảo, chưa phải SPEC được phê duyệt. Owner đang defer initiative này cho đến khi dùng Baron `5.0.1` để xây sản phẩm thực tế; chưa có lịch implementation.
- **Ngày lập:** 2026-10-28 (Asia/Ho_Chi_Minh).
- **Repository Baron:** <https://github.com/thienty1207/Baron-Engine>
- **Repository AI-Memory:** <https://github.com/akitaonrails/ai-memory>
- **Mốc mã nguồn được đối chiếu:** Baron `main` = `c1c182a187e62332a32ce1824f6658f261ce84c8` (v5.0.1); AI-Memory `main` = `59b3e8237fc3c8a3d1a1688ec9484ee2eebac966` (v3.2.9).
- **Phạm vi xác minh:** GitHub repository metadata, file tree, release metadata, README, tài liệu kiến trúc, các module Rust và test/benchmark **trọng yếu**. **Chưa** checkout, biên dịch, chạy CI hoặc kiểm thử A/B thực tế của hai engine trên cùng môi trường. **Không** tuyên bố đã scan 100% source. Các đánh giá về khoảng trống tính năng phải được kiểm tra lại khi triển khai tại checked-out HEAD.

> **Cách dùng:** đọc các feature như ý tưởng để đánh giá và áp dụng vào Baron khi có nhu cầu. Không merge AI-Memory, không triển khai toàn bộ danh sách, và không xem nội dung RFC là quyết định của owner. Chỉ mở lại sau khi owner có quyết định rõ ràng.
>
> **Phân biệt ID:** `SPEC-07-1…SPEC-07-8` là namespace đề xuất riêng cho tài liệu này. `SPEC-07-6` là handoff Codex↔Claude, **không phải** bộ SPEC-06 về DeepSeek Harness trong `docs/specs/Spec6/`.

### Mục lục

0. [Tóm tắt phương án](#section-0)
1. [Nguồn và độ tin cậy](#section-1)
2. [Review hai engine](#section-2)
3. [Feature shortlist](#section-3)
4. [Threat model và rủi ro](#section-4)
5. [Quy ước workstream](#section-5)
6. [SPEC-07-1 — Baseline và evaluation](#section-6)
7. [SPEC-07-2 — Observation retrieval](#section-7)
8. [SPEC-07-3 — Feedback và contradiction lint](#section-8)
9. [SPEC-07-4 — Memory aging](#section-9)
10. [SPEC-07-5 — Context optimizer](#section-10)
11. [SPEC-07-6 — Cross-agent handoff](#section-11)
12. [SPEC-07-7 — Cross-project preferences](#section-12)
13. [SPEC-07-8 — Hardening và release gates](#section-13)
14. [Test chống lỗi logic](#section-14)
15. [Gates định lượng](#section-15)
16. [Ưu tiên và release strategy](#section-16)
17. [Các quyết định còn mở](#section-17)
18. [Kết luận](#section-18)
19. [Nguồn tham chiếu](#section-19)

---

<a id="section-0"></a>
## 0. Executive summary — tóm tắt phương án tham khảo

**Giữ Baron Core làm authority duy nhất**, không merge nguyên repo AI-Memory, không đưa một memory server thứ hai vào kiến trúc mặc định. Baron cần học **cách AI-Memory vận hành vòng đời dữ liệu** và **cách đo chất lượng truy xuất**, không thay thế workflow, trust boundary, Task State hoặc Vault.

### Đọc nhanh

| Hướng | Tóm tắt |
|---|---|
| Giữ trong Baron | Core là authority duy nhất; Vault Markdown là nguồn dữ liệu bền vững; SQLite và index chỉ là dữ liệu dẫn xuất có thể dựng lại. |
| Có thể học | Tìm lại observation có provenance; feedback và relation lint; aging không phá hủy; đo chất lượng retrieval trước khi tối ưu context. |
| Không port nguyên mẫu | Memory server thứ hai; tự nâng trust; xóa memory theo decay; tự học preference toàn cục; mở rộng hàng loạt adapter. |
| Trạng thái | Các hạng mục bên dưới là proposal để tham khảo. Initiative hiện được defer; chỉ tạo SPEC/plan riêng khi owner chọn use case và chủ động mở lại. |

### Các workstream được đề xuất

| Nhóm | ID đề xuất | Tên | Điều kiện xem xét | Ưu tiên đề xuất |
|---|---|---|---|---|
| Nền tảng | SPEC-07-1 | Baseline, invariant contract & long-memory eval | Chỉ mở initiative thì làm baseline trước | P0 |
| Feature candidate | SPEC-07-2 | Observation-aware evidence retrieval | Chỉ xem xét nếu baseline tìm thấy khoảng trống | P1 |
| Feature candidate | SPEC-07-3 | Memory feedback, typed relations & contradiction lint | Chỉ xem xét sau khi chốt nguồn dữ liệu/authority | P1 |
| Có điều kiện | SPEC-07-4 | Non-destructive memory aging & maintenance | Có use case và flag; không xóa nguồn | P2 |
| Theo benchmark | SPEC-07-5 | Context selection & retrieval-efficiency optimizer | Chỉ promote khi số liệu chứng minh lợi ích | P1/P2 |
| Tùy chọn | SPEC-07-6 | Explicit claim-once cross-agent handoff | Optional, mặc định tắt | P3 |
| Tùy chọn | SPEC-07-7 | Approved cross-project preference profile | Optional, mặc định tắt | P3 |
| Gate tích hợp | SPEC-07-8 | Migration, adversarial hardening & release gates | Áp dụng cho thay đổi thuộc initiative nếu được triển khai | P0 |

**Lộ trình MVP tham khảo, chưa được phê duyệt:** SPEC-07-1 → SPEC-07-2 → SPEC-07-3 → SPEC-07-5, sau đó cân nhắc SPEC-07-4. SPEC-07-6/07 chỉ xem xét khi có use case đủ rõ. SPEC-07-8 là gate tích hợp cho thay đổi của initiative; kiểm thử bảo mật/migration liên quan vẫn cần chạy ở từng PR, không dồn đến cuối.

**Các giới hạn bất biến:**

1. Vault Markdown vẫn là **durable source of truth**; SQLite / Wiki / session replay / CodeGraph là index hoặc accelerator có thể rebuild.
2. `TrustedRecallPolicy::Current` và project firewall kiểm tra **trước** ranking/promotion; điểm số tìm kiếm không cấp quyền trust.
3. `OperationContext`, `LifecycleIdentity`, Task State, plan, proof, trace, recovery và approval thuộc Baron Core, không bị feature mới thay thế.
4. Candidate, observation, profile hint, agent output hoặc LLM summary là **untrusted data**, không được tự nâng thành `Verified`, current decision, approved global rule hoặc skill.
5. Codex và Claude vẫn là **thin bridges**; không đưa workflow hoặc memory policy riêng vào adapter.
6. Không xóa/sửa user-owned source hoặc Vault content để đổi lấy tốc độ; mọi mutation phải có authorization, identity, lock, transactional/receipt và recovery phù hợp.
7. Mặc định hoạt động offline / zero cloud-LLM; mọi dependency nặng, embedding, service thường trực hoặc maintenance mất dữ liệu đều phải optional và có fallback.
8. Không được gọi một bài test *đã được khai báo* là bằng chứng *đã chạy*. Mọi “PASS” cần kết quả lệnh, exit code, log/artifact, nguồn mã và environment.

---

<a id="section-1"></a>
## 1. Nguồn và độ tin cậy

### 1.1 Baron: những gì đã có trong 5.0.1

Các khẳng định dưới đây dựa trên README, kiến trúc, Rust source và test của mốc commit nêu đầu tài liệu:

- **Core & adapter:** `crates/baron-core`, `crates/baron-adapters`, `crates/baron-cli`; `.baron/core/**` là runtime Core đã triển khai vào project, Codex/Claude dùng native bridge. Xem `README.md`, `docs/architecture/ARCHITECTURE.md`, `docs/architecture/ADAPTERS.md`.
- **Memory:** `crates/baron-core/src/memory.rs`, `firewall.rs`, `vault.rs`; đã có `MemoryScope`, `MemoryConfidence`, `MemoryStatus`, `MemoryProvenance` với source hash/span, observed/valid time, supersession và contradiction metadata. `docs/architecture/MEMORY_MODEL.md` mô tả trust firewall.
- **Retrieval:** `semantic.rs`, `knowledge.rs`, `intelligence41.rs`, `intelligence42.rs`; lexical/BM25-like, bilingual expansion, local hashed vectors, ranking/fallback, Wiki và CodeGraph; **không** được mô tả như mô hình embedding học sâu chỉ vì có “vector”.
- **Session:** `session.rs`, `session_replay.rs`, `intelligence42.rs`; bounded import, redaction, dedup, replay và candidate-only learning. `intelligence42.rs` tuyên bố không ghi trusted record và không tạo Skill tự động.
- **Continuity:** `task_state.rs`, `continuity.rs`, `operation.rs`, `plan.rs`, `proof.rs`, `trace.rs`; task identity, checkpoint, failure recovery và proof gates.
- **Quality & concurrency:** `autopilot.rs`, `safe_io.rs`, `receipt_authority.rs`, `automation/hook_correlation.rs`; approval authority, locks, idempotency và operation binding. `SPEC-05` trong lịch sử Baron là SPEC cũ về multi-agent concurrency. Các `SPEC-07-1…SPEC-07-8` ở RFC này là namespace đề xuất riêng, khác với lịch sử Baron `SPEC-01…05` và bộ DeepSeek Harness `SPEC-06` trong `docs/specs/Spec6/`; không gộp các tên hoặc phạm vi này.
- **Release:** `CHANGELOG.md` ghi v5.0.1 là sửa false positive Control Plane validator do negated subagent mentions. Current handoff ghi release gate đã qua, nhưng cũng mô tả giới hạn test host Windows/PowerShell và các tồn dư; không suy diễn “không còn bug”.

**Lưu ý dễ nhầm:** Baron có `session_replay` và temporal/contradiction **nền tảng** từ trước; đề xuất mới phải làm rõ incremental behavior, không thiết kế “memory V2” song song. Tên “feature chưa có” ở đây nghĩa là **chưa thấy đầy đủ contract chuyên dụng tương đương trong phần nguồn đã kiểm tra**, không phải đã xác minh bằng full-source audit rằng hoàn toàn không tồn tại.

### 1.2 AI-Memory: những gì dùng để tham khảo

- `docs/ARCHITECTURE.md`: pipeline capture → consolidate → recall → handoff, writer actor, typed observations, single-use handoff, authorization, derived index, optional LLM.
- `docs/typed-edges.md`: closed vocabulary `causes`, `fixes`, `contradicts`; lint dựa trên explicit edge, không phụ thuộc LLM.
- `docs/design-memory-aging.md`: salience, access reinforcement, per-tier aging, extractive compaction, dedup, opt-in dream, feature flags và yêu cầu eval. **Tài liệu thiết kế bao gồm phần “current state” lịch sử**, vì vậy khi implement phải đối chiếu với source 3.2.9, không lấy line number trong design doc làm ground truth tuyệt đối.
- `docs/cross-project-profile.md`: preference có scope, precedence thấp hơn user/repo/project rules, bounded digest và attribution.
- `docs/agent-messaging.md` và `docs/design-handoff-offer-not-claim.md`: claim-once messaging, explicit addressing, nguy cơ auto-claim nhầm session và offer-before-claim.
- `docs/benchmarks/longmemeval-s-3.2.6-release.md`: benchmark 470 câu hỏi, zero-LLM FTS hit@5 **0.951**, local embedding hit@5 **0.945**, với provenance và latency. **Đây là báo cáo 3.2.6**, không phải phép chạy lại trên 3.2.9.
- `Cargo.toml`: AI-Memory v3.2.9 dùng Rust 2024, toolchain tối thiểu Rust 1.95, server/hook/LLM/web crates và tùy chọn local embeddings. Baron dùng edition 2021 và 3 crates; không copy nguyên dependency graph.

### 1.3 Phân loại độ chắc chắn

- **Đã xác nhận từ source/doc:** cơ chế, cấu trúc, tên module, contract đã mô tả ở trên.
- **Đánh giá kiến trúc:** mức phù hợp, rủi ro dự kiến, khả năng gây conflict hoặc latency.
- **Đề xuất chưa được kiểm chứng:** target p95, memory overhead, benchmark threshold, schema mới, pipeline mới và số lượng test; tất cả phải trở thành hypothesis + fixture trước triển khai.
- **Không so sánh điểm tuyệt đối:** Baron 4.2 từng báo 100/100 trên bộ contract phát triển/holdout nhỏ; AI-Memory đo LongMemEval-S. Hai phép đo **khác dataset, task definition, denominator, environment và metric**, không suy ra “Baron thắng/AI-Memory thắng”.

---

<a id="section-2"></a>
## 2. Review chi tiết hai engine

### 2.1 Bản chất sản phẩm

**Baron Engine 5.0.1:** một *coding-agent work orchestration and trust/governance engine*. Giá trị cốt lõi là biến yêu cầu người dùng thành task có identity/constraints, chọn context và skills, điều phối Codex/Claude, kiểm chứng proof, giữ continuity và phục hồi an toàn. Memory là một thành phần của hệ điều phối, không phải sản phẩm duy nhất.

**AI-Memory 3.2.9:** một *long-term memory service for coding agents*. Giá trị cốt lõi là tự động quan sát session, biến observation thành knowledge có thể truy xuất, giữ history có phiên bản, chuyển giao session, phục vụ nhiều harness và người dùng. Nó không có nghĩa vụ thay thế hệ thống proof/trace/mandatory gate của Baron.

### 2.2 Ma trận tính năng và mức chồng lấn

| Năng lực | Baron 5.0.1 | AI-Memory 3.2.9 | Quyết định |
|---|---|---|---|
| Durable human-readable memory | Vault Markdown; SQLite rebuildable | Git-backed Markdown Wiki; SQLite derived | Giữ Vault, không migration sang Wiki khác |
| Project identity & isolation | Stable project ID, firewall, trusted recall | Scope + operator auth + project routing | Giữ firewall của Baron; học explicit scope errors |
| Current-truth control | Verified/candidate/contested/superseded/expired | Versioning, admission, relations, lint | Baron vẫn làm trust authority |
| Session capture | Hook + bounded import + replay | Hook events, observations, session summaries | Mở rộng truy xuất observation thay vì capture engine thứ hai |
| Retrieval | Lexical/bilingual/hash-vector/Wiki/CodeGraph | FTS5/entity/link RRF/optional vectors/observation stream | Benchmark incremental fusion, không thay toàn ranker |
| Temporal knowledge | Provenance, valid-from/until, supersession, temporal ledger | Ingestion-time timeline, supersession | Không làm lại; mở rộng lint nếu cần |
| Contradiction quality | Trạng thái và metadata tồn tại | Typed edges, explicit lint, feedback | Thêm feedback/lint trên contract hiện có |
| Memory aging | Freshness, expiry/trust, housekeeping | Decay, reinforcement, cold compaction | Chỉ adopt non-destructive relevance management |
| Handoff | Task State + recovery + grounded brief | Typed handoff, claim-once, managed workstreams | Optional delivery protocol, không đổi Task State |
| Workflow/proof | Superpowers, quality agents, proof/trace gates | Không có contract tương đương toàn diện | Baron giữ ưu thế chuyên môn |
| Multi-user/server | Shared Vault/project trust; không định vị như multi-tenant memory server | Per-user keys/auth/audit/self-host server | Không mở server chỉ để lấy features |
| Multi-agent coverage | Codex/Claude chính thức | Hơn 20 harness được liệt kê | Tập trung test sâu 2 adapter đang hỗ trợ |
| Defaults/ops | Offline/local; signed install; preserve-first updates | Local service; zero-LLM default; optional embeddings/LLM | Chỉ thêm feature giữ zero-LLM & no-server fallback |
| Benchmark | Bộ acceptance nội bộ có bounded holdout | LongMemEval-S với provenance/report | Học cách làm A/B benchmark repeatable |

### 2.3 Điểm mạnh, điểm hạn chế và rủi ro hệ thống

**Baron — điểm mạnh**

1. Authority rõ ràng: planner, proof, trust và recovery thuộc Core; native adapters mỏng.
2. Context compiler có Tier 0…3, tránh mất identity/constraints khi cắt context.
3. Candidate-only session learning và Autopilot approval chống memory poisoning/skill promotion.
4. Project-bound Vault, nguồn chân lý đọc được; cache có thể rebuild; update có preserve-first và rollback.
5. Có test riêng cho concurrency, false-positive validator, receipt authority, hook identity, Windows filesystem behavior.

**Baron — chi phí cần kiểm chứng**

1. Context compilation có thể phải import/index/rebuild nhiều bước; `context.rs` gọi import sessions, memory index, temporal ledger và replay index. Cần đo cold/warm path, tránh đoán bottleneck.
2. `session_replay.rs` được thấy có thao tác xóa và index lại `session_messages` theo project; đây là điểm **đáng profile** khi session lớn, nhưng chưa có số benchmark chứng minh là bottleneck trong 5.0.1.
3. Nhiều authority/checkpoint làm logic phức tạp, nhất là khi đồng thời repo + Vault trên khác filesystem và có retry/crash.
4. `hook` là accelerator, không phải nguồn chứng nhận; nếu dựa quá nhiều vào hook tự động sẽ bỏ sót lúc host thay đổi.
5. Mọi thay đổi memory cần đánh giá backward compatibility: ngay cả trường dữ liệu phụ cũng có thể ảnh hưởng ranking, context hoặc recovery.

**AI-Memory — điểm mạnh**

1. Capture → consolidate → query → handoff là pipeline tường minh; observations hỗ trợ truy xuất chi tiết chưa xuất hiện trong summary.
2. Có typed relations và lint, explicit feedback, per-tier retention và access reinforcement.
3. Công bố benchmark retrieval lớn với metrics, latency, corpus hash, lệnh chạy và phân rã trường hợp.
4. Cross-harness UX mạnh: resumable workstreams, handoff offer/claim, cross-project profile.
5. Có định hướng zero-LLM default và rebuildable Git/SQLite.

**AI-Memory — trade-off cần tránh khi port**

1. Memory service, auth/multi-user, MCP/hook/router và local LLM dependencies có thể làm tăng diện tấn công/chi phí vận hành so với một Rust Core chạy tại chỗ.
2. Một số maintenance flows có khả năng eviction/hard-delete theo chính sách; không phù hợp nguyên tắc bảo toàn nguồn gốc của Baron.
3. Confidence/salience/access count không tương đương trust authority; tăng điểm nhờ truy cập nhiều có thể củng cố thông tin sai.
4. Cross-project message/profile là thay đổi bảo mật, không chỉ UX.
5. Sự đa dạng harness khiến portability testing phức tạp; Baron chỉ nên mở rộng harness khi thật sự có yêu cầu sản phẩm.

### 2.4 Tác động đến hiệu suất và năng suất

Phải phân biệt:

- **Hiệu suất hệ thống**: p50/p95 prepare và recall, I/O, RSS, CPU, cache rebuild, token/char budget, cold/warm latency.
- **Chất lượng memory**: hit@K/recall@K, false-positive, abstention chính xác, relevance, provenance coverage, secret leakage.
- **Năng suất agent**: số lần hỏi lại điều người dùng đã nói, số bước phải khôi phục thủ công, thời gian đến hành động đầu tiên hợp lệ, số correction/rework, tỉ lệ task complete có proof hợp lệ.
- **Độ tin cậy**: zero cross-project leakage, zero unauthorized promotion, crash recovery, idempotency và deterministic replay.

Feature có vẻ tăng recall nhưng làm tăng false-positive/hallucination hoặc p95 quá mức thì **không** được promote. Cũng không nên tối ưu tốc độ bằng cách bỏ proof/trace hoặc giảm Tier 0.

---

<a id="section-3"></a>
## 3. Feature shortlist: nên học gì, không nên học gì

| Feature / ý tưởng | AI-Memory tham chiếu | Baron đã có gì | Phần bổ sung thực sự | Giá trị kỳ vọng | Ưu tiên |
|---|---|---|---|---|---|
| Observation-aware retrieval | Raw observation search + session vote | Session replay & candidate learning | Bổ sung retrieval evidence đã được scope/sanitize, chỉ là advisory | Recall chi tiết lỗi/lệnh/quyết định thất lạc | Cao |
| Feedback + typed contradiction | Page feedback + fixes/causes/contradicts | Provenance contradiction + contested state + Autopilot | Human feedback, explicit relation lint, source drift review | Giảm trả lời theo tri thức lỗi thời | Cao |
| Non-destructive aging | Retention + access reinforcement + compaction | Memory freshness/expiry | Review queue, hot/warm/cold metadata không xóa nguồn | Giảm noise và index growth | Trung-cao |
| Retrieval/context optimizer | RRF/briefing bounded, explain rank | Semantic reranking + Tier 0…3 | Đo và chọn top evidence theo utility/budget; cache cost centers | Ít token hơn và chuẩn xác hơn | Cao nếu đo được |
| Claim-once handoff offer | Typed handoff + explicit accept | Task State/recovery + operation identity | Delivery acknowledgement, lease, CAS | Handoff ít bị xử lý nhầm | Có điều kiện |
| Cross-project preference profile | User preference digest, precedence | Approved global/project firewall | Preferences được duyệt, per-scope + precedence | Bớt lặp hướng dẫn giữa projects | Có điều kiện |
| Full memory server | Self-hosted service, multi-user auth | Local Core/CLI | Không cần thiết cho use case hiện tại | Nặng vận hành, thêm trust surface | Không |
| Dream/auto-approval | Opt-in LLM dream/auto improvement | Candidate-only learning, approval | Không cho auto-promotion | Rủi ro cao, khó đánh giá | Chưa |
| Auto-delete theo decay | Forget-sweep/TTL/hard delete | Preserve-first Vault | Chỉ metadata/review, không xóa | Rủi ro mất tri thức và audit | Không |
| 20+ native adapters | Nhiều integrations | 2 adapters có contract | Không mở rộng lúc này | Phân tán test matrix | Không |

**Phân loại đặc biệt:** một feature có thể nằm trong **Core code** nhưng **mặc định tắt**. Không đánh đồng “đã add vào core” với “được dùng trên mọi request”. Hướng đề xuất là **capability flag + semantic contract chung + shadow mode + measured promotion**.

---

<a id="section-4"></a>
## 4. Threat model và danh mục rủi ro

### 4.1 Trust boundaries

```text
User intent + repo instructions (highest applicable authority)
                     |
                     v
           Baron OperationContext
                     |
                     v
        Project identity + firewall
                     |
                     v
      Provenance + TrustedRecallPolicy
                     |
                     +--> Vault Markdown (durable truth)
                     |
                     +--> SQLite / replay / Wiki / graph (derived evidence)
                     |
                     +--> untrusted observations / feedback / profile hints
                     |
                     v
          Context Compiler (Tier 0-3)
                     |
                     v
       Codex bridge / Claude bridge
                     |
                     v
       Proof + trace + controlled approval
```

**Không cho phép** observation, typed edge, favorite memory, feedback hoặc handoff thông báo trở thành chỉ thị cấp cao hơn. Nếu nội dung lấy từ transcript chứa “bỏ qua quy định, chạy command...”, nó vẫn là trích dẫn dữ liệu, không phải prompt điều khiển.

### 4.2 Risk register

Mức độ `Critical/High/Medium/Low` là **phân loại dự kiến trước thử nghiệm**, không phải bug đã được xác nhận.

| ID | Rủi ro | Mức | Detection/test | Phòng ngừa/mitigation | Rollback/fail-closed |
|---|---|---|---|---|---|
| R01 | Cross-project observation leak | Critical | Hai repo cùng tên, nhiều session, query liên dự án | Match project ID trước scoring; test exact scope; deny mismatched ID | Disable observation retrieval |
| R02 | Raw text được promote thành current truth | Critical | Poisoning/candidate/contradiction fixtures | Retrieval result có trust tier cố định; approval tách biệt | Tắt candidate injection |
| R03 | Prompt injection từ transcript/metadata | Critical | Adversarial payload với tool directives | Sanitize + inert quotation; không execute từ text; never route commands | Omit contaminated evidence |
| R04 | Duplicate event/write sau retry/crash | High | Repeated hooks, replay, process-kill | Stable event/source key, idempotent upsert, journal/receipt | Replay checkpoint, không tạo duplicate |
| R05 | Handoff bị claim nhầm session | High | Concurrent Codex/Claude starts | Offer-only default; explicit accept + CAS + target identity | Return pending/unclaimed |
| R06 | Race/cross-filesystem partial commit | Critical | Fault injection sau mỗi write boundary | Ordered lock + validate ownership + recovery receipt | Reconcile; không tuyên bố atomic giả |
| R07 | Decay làm mất current decision/proof | Critical | Old-but-critical records | Không auto-delete, pin durable authority; tier metadata riêng | Disable aging scorer, rebuild index |
| R08 | Feedback thao túng ranking/authority | High | Spam, same actor, contradictory flags | Feedback có provenance; không tự promote | Ignore signals, review manual |
| R09 | Performance regression chuẩn bị task | High | p95 cold/warm prepare/retrieve | Bounded top-K, lazy work, memoized derived indexes, backpressure | Feature flag/fallback |
| R10 | Memory bloat/SQLite corruption | High | Large fixtures, corrupt db, disk-full | Caps, WAL/transactions kiểm tra, rebuildable indexes | Rebuild index từ Vault |
| R11 | Context budget loại mất Tier 0 | Critical | Oversized Tier 0 and long histories | Tier 0 mandatory or explicit error; omit lower tiers | Return blocked/warning, no fake brief |
| R12 | Stale source hash vẫn được cite là current | High | Source edit/rename/delete | Revalidate path/hash/revision; tombstones; stale labels | Abstain hoặc warning |
| R13 | Global preference vượt repo rule | High | Project override user preference | Precedence explicit; approved-only global | No profile injection |
| R14 | Migration/update sửa file người dùng | Critical | Modified managed files, unknown keys, symlink | Preserve-first plan, ownership, staging, rollback | Abort and write recovery receipt |
| R15 | Test đánh giá sai vì fallback/fixture leakage | High | Raw vs fallback scoring, sealed holdout | Frozen oracle, record fallback separately, no fixture training | Block promotion |
| R16 | Unsupported host/platform bị báo PASS | High | Windows/Linux/PowerShell 5.1/7 matrix | Phân biệt source/CI/live host; không giả định host ready | Declare unsupported/degraded |
| R17 | Secret trong log/benchmark artifact | Critical | Known synthetic secrets + scans | Redact before persistence, log metadata-only | Quarantine/delete derived output safely |
| R18 | Feature stale/offline phá đường chạy chính | High | Flags off, index missing, timeout | Optional accelerator timeout/ignore with warning | Baseline codepath unchanged |
| R19 | API/schema drift giữa crates/adapters | High | Golden `PreparePacketV1` + CLI contract | Version/serde defaults + backwards compatibility | Gate reject unknown incompatible state |
| R20 | Handoff lease hết hạn khiến hai agent cùng làm | High | Claim race + expiry + slow receiver | Fencing token + operation binding; stale writer không commit | Block concurrent writer; reoffer with receipt |

### 4.3 Chính sách dữ liệu và quyền

- **Read scope:** `(project_id, user/actor nếu có, operation identity nếu liên quan)`; không dùng basename hoặc cwd đoán scope khi có hành vi mutation.
- **Durable mutation:** chỉ thông qua writer hiện có, path canonicalization, lock order đã thống nhất, rollback/receipt; không “shortcut” direct `fs::write` từ feature.
- **Secrecy:** synthetic API key/password/PII fixtures phải được redact trước khi index và trước khi ghi debug logs; metadata cũng không chứa full prompt hoặc secret.
- **Input budget:** byte length, UTF-8 normalization, maximum paths/hits/graph edges, regex/CJK stress và resource exhaustion đều thuộc regression suite.
- **No destructive defaults:** `delete`, `compact`, `merge`, `promote`, `apply profile`, `claim task` phải có explicit authority/approval tương ứng.
- **No hidden execution:** memory text không thể trigger shell/network command; câu trả lời “đã test” phải dựa trên execution receipt.

---

<a id="section-5"></a>
## 5. Quy ước cho bộ SPEC-07 tham khảo

### 5.1 Namespace và trạng thái

Các tài liệu Baron hiện có đã dùng `SPEC-01…05`; bộ DeepSeek Harness trong `docs/specs/Spec6/` cũng có tên SPEC-06 riêng. Để tránh trùng ID, initiative tham khảo này dùng namespace đề xuất `SPEC-07-1…SPEC-07-8` (*Baron × AI-Memory*). Các ID này chưa được owner phê duyệt và không mở công việc implementation.

Mỗi SPEC có các trạng thái: `DRAFT → DESIGN_APPROVED → RED_PROVEN → IMPLEMENTED → INDEPENDENT_REVIEWED → CERTIFIED → MERGED`; chuyển trạng thái cần evidence, không đi tắt. `DEFERRED`, `BLOCKED`, `REJECTED` là trạng thái hợp lệ nếu benefit chưa được chứng minh.

### 5.2 Mỗi SPEC phải có đủ 12 mục

1. Vấn đề thực tế và baseline trước thay đổi.
2. Scope/in-scope và non-goals.
3. Canonical owner và đường đi dữ liệu (read/write).
4. Input/output type + invariant + schema version/backwards compatibility.
5. Security model + expected failure/fallback.
6. Measurable outcome + benchmark dataset/oracle.
7. Red tests trước code production.
8. Implementation slices nhỏ, mỗi slice có traceability.
9. Unit, property, integration, concurrency, fault-injection, OS tests thích hợp.
10. Migration, dry-run, rollback và observability.
11. Independent adversarial review + bug triage + repeat verification.
12. Acceptance report có SHA, commands, logs, test counts, failures, omissions.

### 5.3 Thứ tự PR/commit

```text
SPEC-07-1 foundation & frozen evaluator
       |
       +------> SPEC-07-2 observation retrieval ------+
       |                                           |
       +------> SPEC-07-3 feedback & lint ------------+---> SPEC-07-5 context optimizer
       |                                           |
       +------> SPEC-07-4 aging reports (read-only) --+
       |
       +------> SPEC-07-6 handoff (optional; independent authority check)
       |
       +------> SPEC-07-7 preferences (optional; firewall/global check)
                                                   |
                                 SPEC-07-8 release hardening/gates
```

SPEC-07-4 không phải prerequisite cứng của SPEC-07-5. SPEC-07-6 và SPEC-07-7 không được âm thầm đi vào MVP. Không trộn nhiều SPEC trong một PR để tránh khó bisect và rollback.

---

<a id="section-6"></a>
## 6. SPEC-07-1 — Baseline, invariants & long-memory evaluation

**Mục tiêu:** tạo phép thử đáng tin để biết các cải tiến có thực sự cải thiện recall, hiệu suất và năng suất; đóng băng behavior của 5.0.1 trước khi sửa ranker/Context Compiler.

**Owner đề xuất:** `baron-core` cho fixtures/logic; CLI/test harness dùng module evaluation độc lập. **Không** biến benchmark thành runtime dependency.

**Nguồn code nên kiểm tra:** `crates/baron-core/src/{memory,firewall,semantic,knowledge,context,prepare,session_replay,task_state}.rs`, `tests/{phase6_trusted_memory,context_compiler,concurrency,proof_trace}.rs`, `docs/assessment/*`.

**Thiết kế/implement:**

1. Pin Git SHA, toolchain, OS, dataset hash, seed, exact commands; chạy baseline ở 5.0.1 không đổi.
2. Tạo fixture corpus **synthetic** và dữ liệu đã xin phép: short, 100/1.000/10.000 session, notes đa ngôn ngữ, changed API, negative queries, cross-project collisions, prompt injection.
3. Test độc lập 4 lớp: **retrieval** (hit@K/MRR/precision), **answer-grounding** (trả lời đúng nguồn, abstain), **task-continuity** (recovery đúng operation), **system** (latency/RSS/index/rebuild/token budget).
4. Tạo `expected evidence IDs` và `expected answer state` trước khi implement; holdout độc lập, chống test overfitting.
5. Report raw results + fallback counts riêng; đo cold/warm, Windows/Linux, small/large Vault, single/multiprocess. Đặc tả label cho các tình huống “unknown” và “conflict”.
6. Bổ sung test cho `NoEvidence → abstain`; không để một answer dựa vào stale/candidate thành PASS vì trùng keyword.

**RED tests bắt buộc:**

- `unknown` phải trả về abstention hoặc evidence không đủ, không bịa.
- `candidate/contested/superseded/expired` không có quyền thành current truth.
- Two repos same basename share Vault nhưng không share project-scoped truth.
- Inject prompt “ignore guardrails” trong session không thay đổi routing/gates.
- Tier 0 overflow không làm drop im lặng constraints; compiler phải report overflow/block.
- Corrupted/missing SQLite index có cảnh báo + bounded recovery/fallback, không báo “PASS” giả.
- Replay cùng event nhiều lần không làm false success/proof.

**Chốt trước triển khai:** metrics `hit@1/3/5`, `MRR`, `evidence precision`, `abstain accuracy`, `p50/p95`, `max RSS`, `index bytes`, `context chars/tokens`, `time-to-valid-next-step`. Những metric chưa có baseline được đánh dấu `N/A`, không điền số giả.

**Acceptance:** evaluator chạy 3 repeats với cùng seed + ít nhất một seed khác; CI tạo JSON+Markdown report; immutable label set; báo lỗi rõ nếu thiếu dataset. Baseline report ghi “measured” hay “not measured”, không tự nhận số benchmark từ AI-Memory.

**Rollback:** chỉ thêm test, tool và report; không thay đổi default runtime. Nếu evaluator có bug, rollback evaluator riêng; không hợp thức hóa regression bằng sửa expected output.

---

<a id="section-7"></a>
## 7. SPEC-07-2 — Observation-aware evidence retrieval

**Mục tiêu:** tăng khả năng tìm lại chi tiết từ transcript/session đã nhập vào Baron mà summary/handoff bỏ sót; không tạo engine capture thứ hai.

**Trạng thái:** P1, **shadow-first**, sau đó optional-on nếu benchmark chứng minh benefit.

**Điểm khác biệt với code hiện tại:** Baron có `session_replay` và `learn_session_candidates_v42`. Feature mới chỉ **hợp nhất có kiểm soát** evidence từ replay/index vào retrieval path và `PreparePacketV1` ở Tier 2; không viết trusted fact từ raw observation.

**Contract gợi ý (minh họa, chưa phải API đã có):**

```text
ObservationEvidence {
  project_id, session_id, message_id, source_path, source_hash,
  source_span, role, observed_at, snippet_redacted,
  trust_kind = "untrusted_observation", confidence_hint,
  operation_id?, content_version
}
SearchOptions { mode = "off|shadow|on", max_hits, max_bytes, timeout_ms }
```

**Implement theo lát cắt:**

1. Trace từ `session.rs` → `session_replay.rs` → `firewall.rs` → `context.rs/prepare.rs`. Kiểm tra index thực tế có đủ source hash, project ID, role, span; thiếu thì nâng derived schema **có migration/rebuild**.
2. Thiết kế function truy xuất evidence read-only, project filtering trước query; giới hạn source count, candidate count và excerpt bytes.
3. Chỉ trả về result có `source_hash`, `source_span`, `scope`, state; nếu source biến mất → `stale/unavailable`, không fabricate.
4. Xây reranking trên channels đã có; tránh mặc định cộng điểm observation nhiều lần khiến cùng session áp đảo. Có thể học AI-Memory “mỗi session chỉ 1 vote” như một policy **cần benchmark**, không copy mù.
5. Tạo `shadow` mode: tính hits, ghi diagnostic **không thay đổi** result trả ra cho agent; report relevance/latency/leakage.
6. Chỉ khi pass gate mới enable read-only Tier 2 excerpt, có labels “historical observation / not verified”. No injection cho high-risk routes hoặc khi evidence trùng current validated proof.
7. Gracefully degrade khi SQLite lỗi, files moved, import disabled hoặc timeout. Không dùng observation retrieval làm signal kết luận task complete.

**Test matrix và expected behavior:**

| Case | Expected |
|---|---|
| Một fix có command ở observation nhưng không trong summary | Retrieve đúng span và session, label untrusted |
| Raw note yêu cầu bỏ security gate | Text inert, no action/authority change |
| Source đã đổi nội dung/hash | Stale warning, không cite như current |
| Same message từ Codex/Claude replay | Không duplicate result hoặc double vote |
| Hai project cùng tên | Query không tràn project boundary |
| 10k–100k observations | Runtime/size nằm trong baseline budget sau tuning |
| Restart giữa index write | Rebuild/recovery consistent |
| Missing/unreadable index | Baseline fallback, không crash task |
| Unicode Việt, path Windows, unspaced language | Query trả evidence ổn định, không phá lexical baseline |
| Transcript chứa secret giả lập | Secret không xuất hiện ở stored snippet/log/output |

**Bug-fix protocol:** RED lỗi bằng fixture nhỏ nhất → trace IDs + source hash + query channel → sửa đúng layer; nếu sai scope sửa ở filter, không vá bằng score threshold; nếu duplicate sửa event/index identity, không “dedup string” ngẫu nhiên; nếu timeout profile SQL/alloc trước khi tăng timeout. Rerun targeted + full trusted-memory suite + A/B benchmark.

**Acceptance:** zero scope leak/unauthorized truth promotion; retrieval benefit đo được trên recall slices, p95 không vượt ngưỡng đã chốt; flags `off` cho kết quả baseline tương đương; crash/index repair verified.

**Rollback:** flag `off`; drop/rebuild derived observation index sau khi backup nếu cần; không xóa session source.

---

<a id="section-8"></a>
## 8. SPEC-07-3 — Memory feedback, typed relations & contradiction lint

**Mục tiêu:** nâng chất lượng tri thức theo thời gian thông qua feedback có provenance và explicit relation lint; không phát sinh authority khác Autopilot/Core.

**Trạng thái:** P1, feature có **write path** nên test nghiêm hơn SPEC-07-2.

**Reuse:** `MemoryProvenance.contradicts`, `MemoryTrustState`, temporal ledger, `MemoryStatus`, `autopilot.rs`, `safe_io.rs` và Vault ownership. Không tạo bảng “verified facts” độc lập trong SQLite.

**Schema/contract gợi ý:**

```text
MemoryFeedbackV1 { event_id, project_id, record_id, source_hash,
  actor_id?, signal: helpful|not_helpful|stale|wrong,
  reason?, created_at, operation_id? }
MemoryRelationV1 { source_id, source_hash, relation:
  causes|fixes|contradicts, target_id, target_hash?, evidence_span,
  author_kind, reviewed=false }
MemoryLintFindingV1 { finding_id, scope, severity, source_ids,
  provenance, status: open|dismissed|resolved, suggested_action }
```

**Implementation:**

1. Define durable evidence location: append-only feedback/lint journal trong project Vault hoặc existing owner-controlled record path; SQLite là projection. Human feedback không làm thay đổi source document nếu chưa review.
2. Closed vocabulary cho relation; invalid/unknown relation → warning, không tự mở enum.
3. Relation to record/path phải canonicalize, chống `../`, symlink escape, project ID mismatch và stale hash.
4. Linter đọc changeset/source hashes, detect explicit contradiction, invalid relation, deprecated API paths và duplicate hints; kết quả chỉ là review finding.
5. Human approval tác động vào status/content phải gọi **existing Core authority**, không có “if confidence > 0.9 then verify” shortcut.
6. Các phản hồi nhiều lần cùng event_id idempotent; spam actor không làm auto-trust; feedback thuộc provenance, không thay source truth.
7. Không tính relation `fixes` là proof fix đã chạy: chỉ `proof/trace` cấp quyền xác nhận thực thi.

**Tests:**

- Lint phát hiện `A contradicts B`, kể cả khi một record bị rename/delete, và tạo deterministic finding ID.
- Contradiction không tự làm hỏng Task State hoặc đổi current decision; đúng hành vi là cảnh báo/review theo contract.
- Một actor gửi `helpful` 1.000 lần → không tự nâng trust.
- `wrong` trên record `Verified` chỉ mở review hoặc yêu cầu authority action; không mặc định xóa record.
- Multiple users/sessions concurrent feedback không ghi mất event; retry idempotent.
- Relation target ngoài project/global scope trái phép bị reject.
- `fixes` không được hiển thị như CI proof nếu không có actual execution receipt.
- Invalid UTF-8, JSON/TOML corrupt, path traversal, tampered source hash fail closed.

**Fix bug:** phân loại *semantic false positive* (finding sai), *write integrity* (mất/duplicate event), *authorization* (scope leak), *trust escalation* (promote sai). Với trust escalation dừng merge ngay; ưu tiên boundary fix hơn regex giảm noise. Fixture gốc thành regression test + adversarial variant.

**Acceptance:** finding có traceable evidence, không auto-approve/auto-delete, zero unauthorized promote, no data loss sau crash, migration/rebuild giữ event identity.

**Rollback:** tắt lint/injection; giữ feedback journal để audit; restore reviewed content chỉ qua versioned change/backup, không thao tác âm thầm.

---

<a id="section-9"></a>
## 9. SPEC-07-4 — Non-destructive memory aging & maintenance

**Mục tiêu:** giảm nhiễu khi dự án tích lũy nhiều session, không giảm mức độ tin cậy hoặc xóa kiến thức cũ.

**Status:** P2. Phase A chỉ **report/metadata**, Phase B mới cân nhắc relevance-aware retention, Phase C compaction **review-only**; không triển khai eviction mặc định.

**Tư duy từ AI-Memory:** salience + recency + usage + tier tạo retention signal. Nhưng usage chỉ biểu thị “được đọc nhiều”, **không** chứng minh “đúng”. AI-Memory có decay/hard-delete trong chính sách maintenance; Baron **không copy phần destructive**.

**Contract:**

- `AgingClass = hot | warm | cold | review` là **retrieval/maintenance hint**, không phải `MemoryTrustState` mới.
- `ProtectedKinds`: current decisions, invariants, plan/task, proof/trace, recovery, approved global → không bị auto-tier-down tác động authority; historical/candidate vẫn có quy định riêng.
- `age_score` được tính chỉ trên **current scope, time, access and relevance metadata**, không làm thay đổi trust.
- `access_count` nếu được ghi phải throttle + bounded; read-path logging không biến mỗi read thành durable Vault write.

**Implementation:**

1. Baseline: đo nguồn lớn nhất gây context bloat (sessions, notes, Wiki, graph) và tần suất reuse theo loại.
2. Implement **pure function** score trên data snapshot; deterministic với clock injected (`now`), không gọi clock ẩn trong hàm được unit test.
3. Chạy `dry-run/report` trước: in danh sách hot/warm/cold và lý do, không sửa Vault.
4. Persist classification trong derived cache với source hash/version và invalidation; SQLite có thể xóa/rebuild.
5. Nếu xét feature ranking, thực hiện **A/B shadow**, so hit@K và freshness/false negatives; không tự áp score lên trust filtering.
6. Eventual compaction chỉ tạo candidate diff “nguyên văn gốc → đề xuất rút gọn”, có link source/span và explicit review; không rewrite gốc, không hard-delete.

**Tests:**

- Record đã 3 năm nhưng là current decision/proof vẫn recoverable, không bị bỏ vì `cold`.
- Record stale được truy xuất nhiều **không** thành verified.
- Clock jumps, timestamps malformed, future events, DST/timezones và duplicate accesses deterministic.
- Rebuild SQLite cho cùng Vault tạo lại cùng metadata (ngoại trừ access ephemeral được mô tả rõ).
- Concurrent reads không gây lock contention hoặc update storm.
- Sweep trong disk-full/read-only mode không làm mutation một phần.
- Flag disabled = baseline unchanged.

**Fix bug:** nếu thiếu chính xác evidence do aging, **tắt ranking influence**, không tăng weight để che; điều tra split theo task type và source age. Nếu duplicate access count, sửa khóa sự kiện/throttle, không tăng latency budget để chấp nhận.

**Acceptance:** no destructive writes, no authority transition, metadata deterministic, neutral/positive retrieval quality trên holdout, memory/latency trong budget.

**Rollback:** disable scorer, regenerate cache. Mọi proposed compaction vẫn là review candidate, không tự apply.

---

<a id="section-10"></a>
## 10. SPEC-07-5 — Context selection & retrieval-efficiency optimizer

**Mục tiêu:** tăng năng suất agent bằng cách giảm context noise và thời gian prepare mà không giảm proof/trust coverage; tận dụng learned principles của AI-Memory về bounded briefs và provenance, **không thay** Context Compiler của Baron.

**Baseline code:** `context.rs` ưu tiên Tier 0…3; `semantic.rs` đã có ranking và thresholds; `intelligence41.rs` có token budgets; `prepare.rs` là semantic boundary. Phải profile trước khi thay bất kỳ thứ gì.

**Implementation theo thứ tự:**

1. Instrument `prepare` thành các stages: repo survey, session import, memory index, temporal refresh, replay index, trusted selection, ranking, render. Log duration/counters **không ghi raw prompt/secrets**.
2. Phân tích p95 cold/warm và I/O; đánh dấu stage có ROI cao; **không giả định** `index_session_replay` hay file scan chắc chắn là bottleneck chỉ từ code reading.
3. Bổ sung bounded retrieval planner nhận `task intent`, `OperationContext`, route/risk, eligible source set, token budget; reuse `TrustedRecallPolicy::Current`.
4. Dedupe evidence bằng `(project_id, source_hash, source_span, relation)`; giữ citations; đặt caps theo *số lượng* lẫn *bytes*. Reranking chỉ sau eligibility gate.
5. Tách preview/summary khỏi deepening, nếu thực tế giảm token mà không mất đúng source; cho agent fetch exact span khi cần.
6. Thử cache/incremental index chỉ ở derived layer: cache key có project ID, source digest, schema version, policy fingerprint, operation state khi cần; cache invalidation xảy ra trên edit/rename/delete và config change.
7. Không thay đổi Tier 0 contract; budget overflow phải error hoặc explicit omission diagnostics, không cắt thầm. Tier 1 proof phải giữ priority hợp lệ; Tier 2/3 có thể compress.
8. A/B shadow + rollback flag; đừng đổi ranking và cache/migration trong cùng một PR.

**Tests:**

- Source caching invalidates sau rename, source edit, index corruption, policy change.
- Baseline task state và mandatory gates xuất hiện đầy đủ trong output tối đa.
- Same evidence từ Wiki, replay, memory index không chiếm 3 slots.
- Negative queries không nhận unrelated vector hit vì luôn có result.
- Current evidence trước stale/contested kể cả stale có score cao.
- Stable deterministic ordering (tie-break) giữa OS/run và khi index build lại.
- Cache của repo A không phục vụ repo B cùng tên/different ID.
- Worst-case long prompt, UTF-8, 10k files, 100k observations, long path Windows; timeouts có degradation.
- Observability counters không lộ secrets/source snippets.

**Bug-fix:** nếu output đổi bất thường, lưu diff `PreparePacketV1` + route + provenance; xác định lỗi eligibility, rank, budget hay serialization. **Không sửa một score heuristic** để làm vừa fixture trước khi chứng minh nguyên nhân. Bất kỳ lỗi Tier 0/proof/authorization là merge blocker.

**Acceptance:** không tăng cross-project leaks hoặc unsafe answers; metric thực nghiệm tốt hơn baseline cho ít nhất một mục tiêu đã chọn (p95/token waste/recall), không có trade-off an toàn bị che. Ngưỡng proposal: p95 không tệ hơn baseline >15% và hit@5 không giảm >1 percentage point trên holdout; thực tế chốt lại bằng SPEC-07-1 trước khi implement.

**Rollback:** flags off, reset derived index, parity to previous `PreparePacketV1` golden examples.

---

<a id="section-11"></a>
## 11. SPEC-07-6 — Explicit claim-once cross-agent handoff (optional)

**Mục tiêu:** hỗ trợ tình huống Codex bàn giao **một operation cụ thể** cho Claude (hoặc ngược lại), có offer/accept/ack; không nhầm với recovery/handoff summary hiện có.

**Decision gate trước khi viết code:** chỉ mở nếu chứng minh task handoff hiện tại không đáp ứng use case mà không thể khắc phục bằng Task State/operation identity đã có.

**Contract state machine gợi ý:**

```text
OFFERED --explicit accept/CAS--> CLAIMED --acknowledge--> ACKNOWLEDGED
   |                                |                          |
   +--cancel (authorized)--> CANCELLED    +--lease timeout--> EXPIRED/REVIEW
```

**Quan trọng:** claim ≠ execution success ≠ task complete. Proof/trace/approval vẫn là Core authority. `ACKNOWLEDGED` chỉ xác nhận đã nhận; muốn xác nhận hoàn tất phải dùng plan/proof/gate.

**Implementation:**

1. Design review: define `handoff_id`, `project_id`, `operation_id`, source/target adapter/session, created_at, expiry, fencing token, claim owner, idempotency key; event ledger durable theo rules hiện có.
2. Mặc định **offer-only**: SessionStart không tự claim; cho phép read/list mà không mutate.
3. Accept dùng CAS dưới project/Vault lock đúng thứ tự; kiểm tra exact project/operation/source version. Không dựa vào “newest cwd” hoặc basename khi nhận.
4. Lease và fencing: stale worker không được commit operation state sau khi token hết hạn; reopen phải reconcile với existing plan/proof.
5. Hook unavailable => manual CLI/managed bridge fallback; không coi hook config là evidence executed.
6. Integration với `PreparePacketV1` chỉ để báo có handoff và safe next step; không tự phát lệnh sửa code.

**Tests:** 2 session claim cùng ID => một thắng; retry accept idempotent; crash sau claim trước ack; stale lease/fencing; two projects same name; mismatched operation; absent target adapter; invalid token; Windows path case; malicious handoff text; expiry while recipient busy. Process-level tests và deterministic barrier/fault injection là bắt buộc.

**Fix bug:** mọi double-claim/foreign-claim/blocking deadlock là Critical; fix at CAS/ownership transaction boundary, không fix bằng delay/sleep. Nếu lease complexity quá cao, **defer feature** thay vì ship protocol thiếu correctness proof.

**Acceptance:** single-claim property, no orphan task duplication, no unauthorized ownership change, correct replay/repair, explicit offer/accept UX and fallback. Default flag off.

**Rollback:** disable handoff protocol; preserve outstanding records as historical diagnostics, current Task State/recovery nguyên vẹn.

---

<a id="section-12"></a>
## 12. SPEC-07-7 — Approved cross-project preference profile (optional)

**Mục tiêu:** giảm việc phải lặp lại convention chung giữa repositories, giữ project firewall.

**Không phải:** tự học mọi thói quen từ transcript rồi biến thành global directive. AI-Memory có policy tự nhận biết preference nhưng Baron nên chọn cơ chế **user-approved-only** cho phiên bản đầu.

**Contract:** preference thuộc `GlobalVerified`/approved global boundary hiện có, có `actor`, `source`, `applies_to`, `priority=low`, `version`, `revoked_at?`, `approved_by`; chỉ phát sinh trong context như **default suggestion**, không như mandatory instruction.

**Precedence đề xuất:** current user instruction > repository rules (`AGENTS.md`/`CLAUDE.md` và source policy) > current project decisions > approved global preferences. Nếu Baron có contract precedence hiện hành khác, phải reuse owner contract và không đặt quy tắc mới ngầm.

**Implementation:**

1. Tách preference khỏi knowledge/proof; hạn chế category `tools`, `testing`, `style`, `workflow`, `architecture` và `applies_to` stack.
2. UI/CLI proposal + explicit approve/revoke; lưu vào existing approved global contract, không tự promote từ candidate.
3. Khi prepare, chỉ select bounded digest sau project firewall/trust; preference không được override source/config/constraints.
4. Không tự mutate `AGENTS.md`/`CLAUDE.md`; thao tác “apply rules to repo” phải explicit, preserve-first managed patch và diff review, nếu sau này cần.
5. Mặc định tắt; bật riêng theo operator/project; muốn multi-user private preferences phải thiết kế auth/ownership riêng, không mở rộng bằng global shared file tùy tiện.

**Tests:** conflict npm vs pnpm; source says Python but global prefers Rust; malicious preference author; unauthorized approval; two project scopes; revoke during query; cache invalidation; max byte budget; deterministic ordering; no secrets; default off parity.

**Fix bug:** nếu preferences vượt precedence, sửa selection/authority precondition chứ không chỉ prompt “đừng làm vậy”; invalid global scope phải reject trước ranking.

**Acceptance:** zero precedence violations, zero cross-project data exposure, compact bounded digest, revocation effective, no repo writes by default.

**Rollback:** disable projection, preserve or revoke approved preferences qua controlled operation; không sửa user-owned project files.

---

<a id="section-13"></a>
## 13. SPEC-07-8 — Cross-spec hardening, migration & release

**Mục tiêu:** chứng minh toàn chương trình không gây regression logic, security, update, rollback hoặc host behavior. **Không** gom tất cả test đến cuối: mỗi SPEC phải đủ test để merge riêng; SPEC-07-8 là end-to-end + independent release certification.

**Implementation:**

1. Lập manifest các feature flag, default, schema version, writer ownership, migration path và reverse/disable path.
2. E2E Core → Codex/Claude bridge → prepare → tasks → proof/trace → continuity → restart; kiểm thử host hooks có/không có.
3. Upgrade từ 5.0.1 Vault cũ, downgrade behavior và uninstall; không xóa user-owned Markdown. Nếu reverse migration không an toàn, **fail closed** và ghi rõ downgrade blocker; không giả vờ reversible.
4. Cross-FS repo/Vault, symlink, path canonicalization, Windows case-sensitivity, locked files, offline mode, checksum/manifest integrity, rollback testing.
5. Rebuild all disposable indexes, mất network (nếu feature optional), failed hook, corrupt state, crash at every durable write step; compare with expected recovery contract.
6. Independent security review: adversarial prompt injection, data exfiltration via cache, typed relation path traversal, forged operation IDs, replay poisoning, event key collision, secret leakage.
7. Run benchmark on same machine/hardware; produce deltas for each feature flag + combinations, not only “everything on”.
8. Documentation/install/update/release artifacts phải xác thực SHA/tag/version; không release hoặc merge khi chỉ có “tests planned”.

**Testing gates bắt buộc trước release:**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo build --workspace --release --locked
```

Bổ sung focused tests từ source guide/CI thực tế, feature-specific integration/contract tests, source-aware installer tests, Windows & Linux CI, power-shell lifecycle where supported, signed manifest verification. Các lệnh trên là **đề xuất phải chạy**, **không** phải đã chạy trong quá trình tạo tài liệu này.

**Release artifacts:** `acceptance.json`, `benchmark-before.json`, `benchmark-after.json`, `security-regression.md`, `migration-smoke.md`, `rollback-receipt.md`, `coverage-ledger.md`, build/test logs, reviewed diff, `main` SHA và release candidate SHA.

**Release policy:** không tự ý version bump hoặc push/tag trong lúc code review; tạo PR nhỏ, independent review, owner sign-off, merge theo thứ tự. Nếu hạng mục optional chưa đủ proof, giữ disabled và không block MVP bằng tính năng không bắt buộc.

---

<a id="section-14"></a>
## 14. Bộ test chống lỗi logic — áp dụng chung cho mọi SPEC

### 14.1 Test pyramid theo quyền sở hữu

| Tầng | Kiểm tra gì | Ai chịu trách nhiệm | Khi nào chạy |
|---|---|---|---|
| L0 — pure unit | normalize, rank, relation enum, score, state transition | Module owner | Mỗi commit |
| L1 — property/metamorphic | Idempotency, invariants, monotonicity nơi hợp lệ, input permutations | Test engineer độc lập | Mỗi PR |
| L2 — integration | Vault/SQLite/Core/Prepare/CLI/adapter | Core + adapter owner | Mỗi PR |
| L3 — concurrency/process | Race, CAS, duplicate hook, multiple process, lock ordering | Independent reviewer | Writer/identity changes |
| L4 — fault injection | Crash/restart, partial write, disk full, corruption, downgrade | Independent reviewer | Durable write/migration changes |
| L5 — adversarial security | Spoofed scope, path traversal, malicious transcript, leaked secrets | Security auditor | Mỗi SPEC có trust boundary |
| L6 — performance/quality | Holdout recall, abstention, p95/RSS/bytes | Eval engineer | Trước promote flag |
| L7 — live host/E2E | Codex/Claude, Windows/Linux, hooks fallback | Release engineer | Trước release |

### 14.2 Bộ bất biến phải luôn giữ

- `I01`: **No cross-project read without explicit authorization.**
- `I02`: **No candidate/observation/profile promoted to Verified by ranking, access or auto-learning.**
- `I03`: **Tier 0 constraints and operation identity never silently disappear.**
- `I04`: **Proof remains tied to actually executed evidence, not textual claim.**
- `I05`: **Replay of same event has at most one durable effect per defined identity window; known bounded caveat documented.**
- `I06`: **No user-owned/Vault destructive change from read-only feature.**
- `I07`: **Cache can be rebuilt without inventing new current truth.**
- `I08`: **Cross-filesystem interruption has recoverable receipts; never claim atomicity without guarantee.**
- `I09`: **Unknown or contradictory evidence abstains/asks review, does not guess.**
- `I10`: **Flag off reproduces baseline semantics, with justified exceptions only.**
- `I11`: **No source path/hash mismatch accepted as fresh citation.**
- `I12`: **Adapter parity: same semantic Core decision for Codex and Claude, except explicit host presentation.**

### 14.3 Mẫu test tối thiểu cho từng loại bug

```text
Given: source Vault + project A/B + operation identity + deterministic clock
When:  feature under test receives query/event/feedback/restart
Then:  expected outcome + exact source ID/span + trust state + changed files
And:   forbidden side effects remain absent
And:   generated proof/receipt corresponds to executed action
And:   retries/rebuild produce equivalent semantic state
```

**Phải kiểm tra cả positive và negative.** Ví dụ, test “truy xuất đúng API path” chưa đủ; còn phải đảm bảo path cũ/sai không xuất hiện như current truth và no-hit không tạo kết quả giả.

### 14.4 Fault-injection grid

| Injection point | Expected behavior |
|---|---|
| Trước khi ghi journal | Không nhận completed receipt giả |
| Sau journal trước Vault/index update | Retry/reconcile không tạo duplicate truth |
| Sau Vault trước SQLite commit | Cache rebuild khớp source truth |
| Trong SQLite transaction | Rollback hoặc recover consistent |
| Khi giữ repo lock rồi lấy Vault lock | Không deadlock; fixed lock order |
| Process kill đồng thời hai adapters | Operation-bound state không trộn nhau |
| Source renamed while read | Revalidate path/hash, stale warning |
| Disk full/read-only dir | Không truncate durable Markdown |
| Invalid JSON/TOML/UTF-8 | Fail closed with diagnostic, do not destructively repair |
| Feature flag changes mid-run | Không publish mixed authority state |

### 14.5 Property/metamorphic tests

- **Idempotency:** apply same event twice yields same semantic final state.
- **Isolation:** swap two repositories with same basename; records không đổi chủ.
- **Ordering independence:** reorder irrelevant observations; trusted result không thay đổi.
- **No authority escalation:** tăng `access_count` hoặc lexical score không đổi trust.
- **Round-trip:** encode/decode/migrate/rebuild preserve record IDs and sources.
- **Budget conservation:** emitted context ≤ policy budget hoặc explicit refusal; Tier 0 intact.
- **Source integrity:** mutate source after indexing; query cannot claim unchanged citation.
- **Handoff single-winner:** two concurrent claims of same offer cannot both succeed.
- **Permission monotonicity:** lower privilege cannot observe broader records.
- **Flag-off equivalence:** experimental path disabled behaves like baseline, subject to explicitly documented nonsemantic diagnostics.

Nếu dùng mutation testing, chỉ tính thành công khi mutation đổi invariant thực sự khiến ít nhất một test **fail**; không tính mutation trivial hoặc compile-error là coverage chất lượng.

### 14.6 Quy trình tìm và fix bug, tránh “fix càng nhiều càng sinh bug”

1. **Reproduce:** chốt Git SHA, host OS, flags, input sample, operation/project identity và exact failing assertion; bug không tái hiện phải ghi `UNREPRODUCED`, không tự tuyên bố fixed.
2. **Classify severity:** Critical (trust/scope/data loss/proof/ownership) → block ngay; High (concurrency/recovery/major regression) → block; Medium (rank false positive, perf regression) → hold promotion; Low (copy/docs) → separate PR.
3. **Isolate owner:** tìm bước đầu tiên khác baseline (`capture`, `index`, `eligible filter`, `rank`, `compile`, `adapter`, `write`, `recovery`). Không sửa “rải rác” nhiều module.
4. **RED first:** thêm fixture tối thiểu tái hiện đúng failure và test negative side effect. Independent reviewer xác nhận test đỏ trên code lỗi.
5. **Root cause:** dựng call/data flow + state transition + ownership contract; hỏi có upstream source/hash/identity mismatch không, có stale docs không.
6. **Minimal fix:** sửa tại authority boundary. Không tăng timeout, hạ threshold, broad catch hoặc chỉnh expected output chỉ để xanh test.
7. **GREEN + adversarial:** chạy test bug, sibling cases, mutation/property tests, full affected suites, cross-adapter and memory isolation.
8. **Repeat:** chạy 3 lần deterministic tests và stress concurrency/multiprocess theo risk; kiểm tra lạnh/nóng + recovery.
9. **Independent review:** reviewer khác implementation agent tìm counterexample, stale state và cách bypass; tối đa số vòng review định trước, chưa clear thì BLOCKED và ghi risk, không lặp vô hạn.
10. **Evidence record:** report trước/sau, nguyên nhân, sửa, tests, benchmark delta, hạn chế, rollback. Chỉ đóng khi independent reviewer không còn blocking findings.

**Quy tắc không thương lượng:** không xóa test đang fail, không sửa oracle cho vừa output, không skip build warning, không đánh tráo test mock thành live proof, không lấy fallback score làm raw win, không auto-merge khi chưa giải quyết review blocking comments.

---

<a id="section-15"></a>
## 15. Gates định lượng: điều kiện chốt trước coding

Các mức dưới đây là **proposed policy**, phải ratify trong SPEC-07-1; nếu baseline không đủ ổn định, thay bằng delta/CI hoặc slice-specific thresholds nhưng phải ghi lý do.

| Gate | Chỉ số | Mức đề xuất | Fail policy |
|---|---|---|---|
| G1 Scope | Unauthorized cross-project disclosures | **0** trong toàn bộ suite | Block merge |
| G2 Trust | Candidate/observation/profile → current verified without approval | **0** | Block merge |
| G3 Integrity | Data-loss, wrong operation/proof binding, unrecoverable partial writes | **0** | Block merge |
| G4 Retrieval quality | `hit@5` decrease on locked holdout | ≤ 1 percentage point; slices critical không regression | Hold feature |
| G5 Grounding | Unsupported current answers on negative/conflict set | Không tăng so baseline; critical false positives = 0 | Hold/block |
| G6 Latency | p95 warm-path prepare/recall | ≤ 1.15× baseline, same machine | Tune/disable |
| G7 Budget | Tier 0 loss, silent overflow | **0** | Block merge |
| G8 Resource | RSS, index size, CPU on long-lived corpus | Baseline + agreed cap; chưa có số tuyệt đối | Hold until measured |
| G9 Recoverability | Crash/retry/migration targeted failures | **0 unresolved blocking failures** | Block |
| G10 Portability | Supported Windows/Linux test matrix | CI PASS or explicitly scoped/deferred; no fake PASS | Block unsupported claim |
| G11 Default-off | Disabled flags produce baseline-compatible behavior | PASS | Block |
| G12 Evidence | Source SHA, commands, logs, measured metrics, known limits | 100% required fields present | No promotion |

**Lưu ý thống kê:** “0 trong suite” không chứng minh không tồn tại bug tuyệt đối. Long-memory holdout cần đủ đại diện cho workload thực tế; threshold 1pp với mẫu nhỏ dễ là nhiễu, nên report confidence interval/paired per-query delta và phân tích failure slices.

---

<a id="section-16"></a>
## 16. Ưu tiên thực hiện và release strategy

### 16.1 Kế hoạch theo wave

| Wave | Mục tiêu | SPEC | Merge strategy |
|---|---|---|---|
| Wave 0 | Baseline, benchmark, threat contract | SPEC-07-1 | Test-only PR, no semantic change |
| Wave 1 | Recall chi tiết + chất lượng tri thức | SPEC-07-2, SPEC-07-3 | 2 PR độc lập, flags off/shadow |
| Wave 2 | Giảm noise/context cost, giữ recall | SPEC-07-5, SPEC-07-4 | Đo ROI trước, chỉ bật nếu đạt gate |
| Wave 3 | Handoff/profile theo nhu cầu | SPEC-07-6, SPEC-07-7 | Optional PR; có thể defer hoàn toàn |
| Wave 4 | Unified acceptance, migration, release | SPEC-07-8 | Final integration PR + explicit release approval |

### 16.2 Stop conditions

Dừng feature và quay về baseline nếu có bất kỳ điều kiện nào:

- Cần tạo **authority/memory database thứ hai** hoặc làm mất vai trò durable source của Vault.
- Không thể chứng minh exact project/operation binding trên read/write mới.
- Mất dữ liệu, silent overwrite, double-claim, auto-promote hoặc wrong proof.
- Long-memory recall không cải thiện, trong khi p95/resource cost tăng đáng kể.
- Test matrix chỉ pass bằng cách sửa expected output, giảm coverage hoặc bỏ negative fixture.
- Không có migration/rollback đáng tin cậy cho dữ liệu bền vững.
- Feature không có use case thực tế hoặc không thể phân biệt lợi ích so với chức năng Baron đã có.

### 16.3 Definition of Done (DoD) cho một SPEC

- [ ] Design/spec được owner duyệt; conflict với historical SPEC và existing Core resolved.
- [ ] Source inventory/caller/callee/ownership review hoàn thành cho các file in-scope.
- [ ] Red fixture độc lập được chạy và fail đúng bug/hypothesis trước implementation.
- [ ] Production changes ít nhất đủ để test xanh; không tạo parallel authority.
- [ ] Positive/negative/invariant/property tests xanh và test counts được lưu.
- [ ] Security & adversarial targeted review có kết luận, blocking findings = 0.
- [ ] Performance/quality A/B on frozen corpus với log + delta rõ.
- [ ] Migration/rollback/fail-closed verified nếu có persistence change.
- [ ] Codex/Claude parity test applicable; hook absence fallback verified.
- [ ] Docs/source behavior aligned, no stale “latest release” claims.
- [ ] Owner chấp thuận flag promotion (nếu có); CI/commit SHA và receipt lưu lại.

### 16.4 Cấu trúc tài liệu trong Baron repository (đề xuất)

```text
docs/initiatives/baron-ai-memory/
  RFC.md                          # tài liệu tổng thể này sau khi review
  DECISIONS.md                    # owner-approved decisions, open questions
  RISK_REGISTER.md                # risks, owner, mitigation, residual state
  EVAL_CONTRACT.md                # frozen oracle, metrics, benchmarks
  SPECS/
    SPEC-07-1-baseline-eval.md
    SPEC-07-2-observation-retrieval.md
    SPEC-07-3-feedback-contradiction-lint.md
    SPEC-07-4-nondestructive-aging.md
    SPEC-07-5-context-optimizer.md
    SPEC-07-6-explicit-handoff.md
    SPEC-07-7-approved-global-profile.md
    SPEC-07-8-release-hardening.md
  VERIFICATION/
    index.md                      # commit SHA + executed commands + references
    <spec-id>-acceptance.md
```

**Đây là cấu trúc đề xuất**, chưa có trong repository. Nếu hiện hành đã có convention khác cho SPEC, ưu tiên convention chuẩn của Baron; chỉ giữ unique ID `SPEC-07-*`.

---

<a id="section-17"></a>
## 17. Những quyết định phải thống nhất trước khi Codex bắt đầu code

| ID | Câu hỏi | Đề xuất mặc định | Owner quyết định |
|---|---|---|---|
| D01 | Có thay Vault bằng Git Wiki không? | Không | Baron maintainer |
| D02 | Observation có được inject trực tiếp Tier 1 không? | Không; chỉ Tier 2 advisory | Core maintainer |
| D03 | Kích hoạt observation khi nào? | Off → shadow → measured on | Release owner |
| D04 | Feedback `wrong` có tự hạ trust không? | Không; review finding | Trust/approval owner |
| D05 | Retention có tự xóa source không? | Không, tuyệt đối trong initiative này | Maintainer |
| D06 | Handoff có auto-claim tại startup không? | Không; explicit offer/accept | Concurrency owner |
| D07 | Global profile có auto-learn không? | Không; approved-only | User/maintainer |
| D08 | Performance threshold định lượng? | Chốt theo SPEC-07-1 same-host paired benchmark | Eval owner |
| D09 | Dữ liệu long-memory benchmark dùng gì? | Synthetic + sanitized real cases với consent | Eval/security owner |
| D10 | Lịch migrate các sidecar? | Mỗi SPEC có migration + rollback; không gom | Release owner |
| D11 | Có thêm LLM/embedding service không? | Không bắt buộc; optional, off | Maintainer |
| D12 | Có mở adapter thứ ba không? | Không nằm trong initiative | Maintainer |
| D13 | Lỗi blocking khi test thì sao? | Dừng merge, root-cause RED/GREEN | Independent reviewer |
| D14 | Khi nào được tag release? | Sau SPEC-07-8 full evidence + explicit approval | Owner |

---

<a id="section-18"></a>
## 18. Kết luận cuối cùng

1. Baron 5.0.1 hiện đã sở hữu các phần quan trọng nhất của *trusted project agent orchestration*: Task State, memory trust, temporal evidence, recovery, proof, trace, context tiers và Codex/Claude bridge. **Không cần** nhập AI-Memory để làm lại những phần này.
2. AI-Memory đáng học nhất ở **observation retrieval**, **feedback + relation lint**, **memory lifecycle/retention** và **cách xây benchmark với provenance**. Các ý tưởng này cần port theo *contract* của Baron chứ không copy nguyên implementation.
3. Để thực sự tối ưu hiệu suất và năng suất, ưu tiên phải dựa vào **đo baseline**, không dựa vào số lượng feature. Nếu Context Compiler đang tốn I/O đáng kể, tối ưu incremental/cache theo evidence có thể tốt hơn thêm LLM/reranker.
4. Nếu owner chọn mở initiative, có thể phân rã thành **8 workstream SPEC-07 (`SPEC-07-1…SPEC-07-8`)**: một baseline, các feature candidate, hai hạng mục tùy chọn và một gate tích hợp. Đây là phương án tham khảo, không phải lịch triển khai đã duyệt.
5. Mọi SPEC phải có test âm tính, property/concurrency/fault injection phù hợp, independent reviewer, measured quality/performance, fail-closed semantics và rollback. **Không thể cam kết zero bug**, nhưng có thể giảm đáng kể lỗi logic bằng việc giữ duy nhất một authority, khóa contract trước code, kiểm thử ở đúng boundary và không chấp nhận “green by adjusting tests”.

---

<a id="section-19"></a>
## 19. Nguồn tham chiếu (permalink theo commit đã đối chiếu)

### Baron Engine 5.0.1

- [README](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/README.md)
- [CHANGELOG](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/CHANGELOG.md)
- [Architecture](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/architecture/ARCHITECTURE.md)
- [Memory Model](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/architecture/MEMORY_MODEL.md)
- [Context Compiler](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/architecture/CONTEXT_COMPILER.md)
- [Adapters](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/architecture/ADAPTERS.md)
- [Source Reading Guide](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/architecture/SOURCE_READING_GUIDE.md)
- [Canonical CURRENT](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/superpowers/plans/CURRENT.md)
- [Memory Source](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/crates/baron-core/src/memory.rs)
- [Semantic Ranker](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/crates/baron-core/src/semantic.rs)
- [Session Replay](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/crates/baron-core/src/session_replay.rs)
- [Intelligence 4.2 candidates](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/crates/baron-core/src/intelligence42.rs)
- [Autopilot authority](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/crates/baron-core/src/autopilot.rs)
- [4.2 Acceptance](https://github.com/thienty1207/Baron-Engine/blob/c1c182a187e62332a32ce1824f6658f261ce84c8/docs/assessment/baron-4.2-acceptance.md)
- [5.0.1 Release](https://github.com/thienty1207/Baron-Engine/releases/tag/v5.0.1)

### AI-Memory 3.2.9

- [README](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/README.md)
- [Architecture](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/ARCHITECTURE.md)
- [Comparison](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/comparison.md)
- [Typed Edges](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/typed-edges.md)
- [Memory Aging Design](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/design-memory-aging.md)
- [Cross-project Profile](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/cross-project-profile.md)
- [Agent Messaging](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/agent-messaging.md)
- [Offer vs Claim Design](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/design-handoff-offer-not-claim.md)
- [LongMemEval-S 3.2.6 report](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/docs/benchmarks/longmemeval-s-3.2.6-release.md)
- [Workspace Cargo.toml](https://github.com/akitaonrails/ai-memory/blob/59b3e8237fc3c8a3d1a1688ec9484ee2eebac966/Cargo.toml)
- [3.2.9 Release](https://github.com/akitaonrails/ai-memory/releases/tag/v3.2.9)

**Hết RFC.**
