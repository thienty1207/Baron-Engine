# SPEC-05 CURRENT / latest authority inventory

Date: 2026-10-01. Audit baseline: `9bdb979`. Closure base: `ecaf362`.
The final production-source caller scan returned 239 matching lines across Core and CLI; exact intent/current callsites are classified below. Two independent reviews of `ecaf362..af65881` found three Important issues, which are being fixed and reverified. The earlier local counts are historical until integrated verification is rerun. This inventory is not closure acceptance.

## Search coverage

The initial closure-baseline scan found 320 tracked matching lines, 222 in executable-source extensions (including imports, definitions, comments, and tests). The final production-source scan used the same authority patterns over `crates/baron-core/src` and `crates/baron-cli/src` and found 239 matching lines; this count includes imports, declarations, comments, and callsites. Search patterns:

```text
CURRENT\.md|active_plan\(|active_plan_authority\(|active_plan_operation_binding\(|current_plan_title|current_plan_risk|current_risk|reconcile\(|latest_proof\(|latest_trace
active_plan_authority_for|active_plan_completion_evidence_status|record_trace\(|score_trace\(|record_proof|proof_for_operation\(|trace_for_operation\(|record_continuity|compile_task_state|gate_evidence_status|runtime_backend_report|record_lifecycle_event_for_operation
```

Each row below represents the body of a caller, not a classification inferred from its name. Baseline line numbers are provenance only and are not current line references. `BUG -> ...` identifies a repair being verified, not an assertion that verification has finished.

## Native hook payload contract checked against official references

Checked on 2026-10-01 against the [Codex Hooks reference](https://learn.chatgpt.com/docs/hooks) and [Claude Code Hooks reference](https://code.claude.com/docs/en/hooks). Codex exposes `session_id` on all hooks and event-scoped `turn_id` on `UserPromptSubmit`, `PreCompact`, and `Stop`; the prompt text is `prompt` on `UserPromptSubmit`, while Stop supplies `last_assistant_message` rather than the original prompt. Claude exposes common `session_id`; `prompt_id` is a common field from v2.1.196 onward and is absent until the first user input. Claude `UserPromptSubmit` carries `prompt`; `PreCompact` carries `trigger` and `custom_instructions`; Stop carries `stop_hook_active`, `last_assistant_message`, `background_tasks`, and `session_crons`, not the original prompt.

| Host | Event | Session key | Turn/prompt key | Task text in event |
| --- | --- | --- | --- | --- |
| Codex | SessionStart | `session_id` | absent | absent (`source` only) |
| Codex | UserPromptSubmit | `session_id` | `turn_id` | `prompt` |
| Codex | PreCompact / PostToolUse | `session_id` | `turn_id` | absent (`trigger` / tool fields) |
| Codex | Stop | `session_id` | `turn_id` | absent (`last_assistant_message`) |
| Claude Code | SessionStart | `session_id` | `prompt_id` may be absent before first input | absent (`source`, optional model/title) |
| Claude Code | UserPromptSubmit | `session_id` | `prompt_id` | `prompt` |
| Claude Code | PreCompact / PostToolUse | `session_id` | `prompt_id` when available | absent (`trigger` / tool fields) |
| Claude Code | Stop | `session_id` | `prompt_id` when available | absent (`last_assistant_message`, stop fields) |

The host-specific identifiers retain their native meanings: Claude `prompt_id` is not aliased to Codex `turn_id`; Baron `request_id` remains only a compatibility transport. These fields are characterized by the real-shaped fixtures in `crates/baron-core/tests/hook_identity.rs`.

## Production caller classifications

| Module / callers (baseline locations) | Classification and disposition |
| --- | --- |
| CLI proof record, trace record, score auto-selection (main 2945/3093/3116) | BUG -> EXACT_OPERATION_SCOPED with complete selectors and indexed ACTIVE authority; AMBIGUOUS_FAIL_CLOSED for no selector. Receipt tuple is its own exact selector and cannot use the legacy unindexed-plan fallback. |
| CLI refresh reconciliation report (main 3755) | PRESENTATION_ONLY; never marks completion. |
| plan authority / operation binding exports (227/230/244/247) | BUG -> LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED. Validated discovered operations are counted; missing CURRENT cannot hide multiplicity. |
| plan exact authority, completion, update, interrupt and completion wrappers (178/189/198/212/216/288/399/536/585/643/1330) | EXACT_OPERATION_SCOPED; former CURRENT integrity veto removed. |
| exact plan lookup and CURRENT sentinel (1057/1061/1093/1094/1098) | BUG -> EXACT_OPERATION_SCOPED through ACTIVE and canonical frontmatter; no CURRENT read. |
| identified start/resume compatibility check (394/403/405) | BUG -> EXACT_OPERATION_SCOPED; same-title anti-hijack checks scan managed canonical plans rather than CURRENT. |
| legacy start/resume (400) | BUG -> LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED, same selector as legacy mutation. |
| legacy update/interrupt/complete / selector (525/574/632/1341/1346/1350/1354/1364) | LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED. An existing compatibility projection is validated, never used to choose among identified operations. |
| legacy completion status precheck (254/259/263/265) | BUG -> AMBIGUOUS_FAIL_CLOSED; ACTIVE discovery occurs even without CURRENT. |
| ACTIVE discovery and linked plan validation (1063/1068/1090/1152/1393, 928/972/980/1532/1907) | EXACT_OPERATION_SCOPED / AMBIGUOUS_FAIL_CLOSED. Duplicate claims, unsafe paths, status, identity and canonical frontmatter conflicts reject. |
| completion proof, trace and gate evidence (729/762/781) | EXACT_OPERATION_SCOPED; exact proof and fresh trace for that proof, scoped trusted gate receipts. |
| plan status/completion-integrity presentation (801/806/843/859) | PRESENTATION_ONLY. CURRENT tamper diagnostics remain; identified context uses the new exact canonical plan view. |
| plan CURRENT projection publication (911/912), excluding CURRENT/INDEX/ACTIVE from discovery (1178) | PRESENTATION_ONLY publication; EXACT_OPERATION_SCOPED discovery exclusion. |
| automation reconcile / automation status (713/761) | LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED underneath PRESENTATION_ONLY status. Missing CURRENT no longer means no active work. |
| identified Stop / reconcile (542/735) | EXACT_OPERATION_SCOPED; durable host correlation precedes exact authority. Unknown, malformed, stale and ambiguous correlation fails closed. |
| Core unscoped trace recording (119/125/224/225/227/238/242) | BUG -> an identified active plan requires its own exact bound proof before an unscoped trace can be recorded; absent proof and multi-active ambiguity fail before writes. Unbound diagnostic traces remain available only when no identified operation claims implicit authority, and cannot satisfy identified completion. |
| bound trace publication/revalidation (161/203) | EXACT_OPERATION_SCOPED; authority and proof are rechecked after unlocked Git work. |
| bound trace story selection (229) | BUG -> EXACT_OPERATION_SCOPED association via validated canonical task ownership; unmatched story remains missing. |
| trace score/find_trace mutation (294/301/616/628) | Explicit ID: EXACT_OPERATION_SCOPED. Auto-score: LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED; no global newest operation fallback. |
| bound trace scoring (333/347/440) | EXACT_OPERATION_SCOPED. Unbound legacy scoring is diagnostic and cannot satisfy identified completion. |
| global latest score cache (473/474/479/482) | PRESENTATION_ONLY; operation authority uses fresh evaluation instead. |
| exact trace/proof selection (487/491/495/507/519) | EXACT_OPERATION_SCOPED, including proof ID and all lifecycle fields. |
| raw CURRENT title/risk helpers (738/739/746/748) | BUG -> removed from trace implementation. |
| proof complete binding / receipt recording / exact reader and validation projection (60/98/104/129/136/164/291/303) | BUG -> exact proof binding and plan risk; `TEST_MATRIX` promotion validates unique exact ACTIVE/story ownership before any proof/runtime write and updates only that operation. Unbound proof retains legacy current-story projection; incomplete operation context cannot touch a story. |
| deprecated unbound receipt/capability APIs (53/123) | AMBIGUOUS_FAIL_CLOSED; explicit binding required. |
| proof harness validation promotion (234/235 -> harness 203/206) | BUG -> EXACT_OPERATION_SCOPED association; proof A cannot promote B's current TEST_MATRIX story. |
| global latest proof/status (251/260) | PRESENTATION_ONLY. Identified consumers use exact operation proof. |
| operation/event continuity checkpoint ingress (132/143/152/164/180/181/194) | BUG -> EXACT_OPERATION_SCOPED; retain full identity in metadata and evidence selection. |
| checkpoint evidence composition (260/261/262/263/264) | BUG -> EXACT_OPERATION_SCOPED; no CURRENT plan/global proof/cached pass/unrelated journal event in A-labelled checkpoint. |
| shared recovery composition (347/348/349/350), continuity status (228/240) | PRESENTATION_ONLY legacy view; unbound/mismatched packets are excluded from identified resume authority. |
| Task State operation compilation (103/114/115/117/190/191/222/223/239/253) | BUG -> EXACT_OPERATION_SCOPED; full identity retained, unmatched intent/recovery remains unknown, and an identity-only checkpoint without an exact active plan is not resumed continuity. |
| Prepare continuity, resumed state and blockers (373/375/377/430/521/770) | BUG -> EXACT_OPERATION_SCOPED canonical plan / matched continuity. |
| identified intent producer / Prepare and Task State consumer | BUG -> EXACT_OPERATION_SCOPED mirrored `intent-operations/<operation_id>.md` Repo/Vault record; complete identity header validated, `CURRENT_INTENT.md` is projection only, legacy unbound producer remains display/intake-only for identified flows. Oversized/corrupt operation records fail closed. |
| Prepare verification proof/trace/gates/runtime (390/401/407/412) | EXACT_OPERATION_SCOPED; existing exact verification preserved. |
| identified context compile/state injection (110/146/147/172/318/334/576), operation context overload (237/247/252) | BUG -> EXACT_OPERATION_SCOPED; preserve identity and suppress unrelated shared resume/execution excerpts. |
| context “why loaded” presence checks (385/404) | PRESENTATION_ONLY. |
| context maintainer status/history/audit fallback (577/579/590/597/603/654) | PRESENTATION_ONLY / OUT_OF_SPEC diagnostics; not identified authority. |
| knowledge runtime resume brief (269/270/272/278/293 -> intelligence 569), candidate runtime (364/451) | BUG -> EXACT_OPERATION_SCOPED input for identified context; unscoped diagnostic API preserved. |
| intelligence benchmark runtime (802) | OUT_OF_SPEC; benchmark diagnostics do not authorize completion. |
| control-plane shared changed-file routing hints (772/773/792/369) | OUT_OF_SPEC routing hints, not completion authorization. |
| exact gate evidence (1540/1561/1645) | EXACT_OPERATION_SCOPED / AMBIGUOUS_FAIL_CLOSED; full identity and trusted receipt binding. |
| compatibility gate reports (1477/1488/1507/1577/1686; CLI 3514) | PRESENTATION_ONLY / OUT_OF_SPEC; not identified completion or Prepare verification. |
| capability required execution checks (427/515/527/612/653/866/887) | EXACT_OPERATION_SCOPED; capability/provider/verified receipt identity. |
| adapter-only capability/runtime reports (413/601/751; CLI 3693; certification 379) | PRESENTATION_ONLY / OUT_OF_SPEC; cannot satisfy operation execution evidence. |
| diagnostic runtime JSONL attachments (561/583; proof 220) | PRESENTATION_ONLY diagnostic_attachment; authoritative runtime checks use verified receipts. |
| generic lifecycle journal (651/680/686/701; CLI 2688/4095/4106/4111) | PRESENTATION_ONLY; never substitutes for durable exact hook correlation. |
| harness story projection/status (87/89/156) | PRESENTATION_ONLY. |
| shared harness title/risk/validation helpers (175/188/203/206) | LEGACY_SINGLE_ACTIVE_SAFE diagnostic helpers; identified proof/trace consumers use checked operation ownership instead. |
| harness improvement project-wide audit (51/53/70/74/75) | PRESENTATION_ONLY; trace existence is not passing operation evidence. |
| Autopilot status/provenance sources (557/1704/1706/1707) | PRESENTATION_ONLY; no CURRENT selection or silent truth promotion. |
| adapter update rollback_local_reconcile (875/883/1047/1075/1566) | OUT_OF_SPEC filesystem rollback, not plan/evidence reconciliation. |

Test fixtures, imports, declarations, comments and historical documentation are OUT_OF_SPEC as production authority callers. All production match groups are represented above. Context/replay/index/cache writers remain outside the SPEC-05 transaction scope; this repair removes their misuse as identified truth without introducing SPEC-06 machinery.

## Rulings and regressions

- Exact CURRENT veto is correctness authority even if it does not select the plan. Remove it from identified APIs; retain compatibility diagnostics and legacy tamper rejection.
- Two old tests that demanded exact B fail solely because CURRENT was forged now assert exact valid B succeeds while high-risk A still fails; canonical ACTIVE/frontmatter tamper tests are unchanged. This is the new presentation-only contract, not a weakening of managed plan authority.
- Preserve same-title anti-hijack behavior from canonical plan files, not from a latest projection. Session-only host delivery cannot prove a turn; it must never merge distinct native operations solely by text.
- Core no-selector trace APIs require their own guard; fixing CLI alone is insufficient. Regressions cover multi-active no-write ambiguity and sole A with newer completed B evidence.
- Explicit CLI receipt selectors require the exact indexed ACTIVE row; the legacy compatibility scanner may still read old plans but cannot authorize receipt-backed proof ingress.
- Bound proof publication uses the exact active plan risk and cannot update a different CURRENT story. Malformed owned story metadata is rejected before proof, receipt-use, or runtime evidence publication.
- `record_trace` remains fail-closed when one identified operation is active but lacks its own bound proof. Legacy unbound trace records are diagnostics only when no identified operation is implicitly selected; they cannot satisfy later identified completion.
- A matching identity-bearing continuity checkpoint without an exact active plan remains diagnostic and cannot make Task State claim the operation is resumed.
- Confirmed intent is correctness-sensitive too: identified Prepare no longer consumes the unbound legacy CURRENT intent. `record_intent_for_operation` writes a bounded, identity-header-validated record per operation in Repo and Vault; Prepare and Task State read that path only. The ordinary CURRENT remains latest presentation, and an unbound/mismatched intent cannot satisfy another operation's confirmation blocker.
- The `prepare_cli` UTF-8 panic was reproduced with a synthetic `é` fixture at baseline `ecaf362` and the same `String::truncate` boundary; current code uses a valid UTF-8 boundary and has a regression test. Baseline `prepare_cli` passes `5/5`, current `prepare_cli` passes `6/6`, and current `session_replay` passes `5/5`, all with nonexistent session roots.
- Host session roots are overridden to nonexistent temporary paths during local test execution: this isolates the host's real Codex/Claude transcript trees from test fixtures without changing product behavior. Normal context compilation still performs the existing bounded session import behavior.
- Final verification checkpoint: `cargo test --workspace --all-targets --no-fail-fast -j 1` exited `0` on the repaired source with isolated session roots and `BARON_TEST_POWERSHELL` set to the bundled PowerShell 7 executable. Current lifecycle scripts pass `7/7`; baseline `ecaf362` lifecycle scripts also pass `7/7` with the same host. Current Core/CLI/adapter suites are green, including hook identity `18/18`, concurrency `16/16`, config `16/16`, plan `51/51`, proof-trace `29/29`, operation-evidence CLI `9/9`, plan-identity CLI `4/4`, prepare CLI `6/6`, and adapter lifecycle `23/23`. `cargo fmt --all -- --check`, workspace Clippy `-D warnings`, locked release build, explicit ignored release smoke, and version check pass. Status JSON parses; maintained-doc integration checks and retired-adapter grep pass; `git diff ecaf362 --check` passes after this evidence update. Two fresh independent read-only reviews of the committed final range remain pending. No owned Critical/Important defect is currently known; closure remains pending review.
