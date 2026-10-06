# SPEC-05 CURRENT / latest authority inventory

Date: 2026-10-06. Audit baseline: `9bdb979`; latest reviewed WIP: `4a3efc0`; closure base: `ecaf362`.
The earlier Core/CLI scan returned 258 matching lines before the latest repairs. Two fresh independent reviews of `ecaf362..4a3efc0` found three Important findings; all three now have observed RED regressions and focused GREEN changes. Focused evidence: harness scoping `13/13`, proof/trace `33/33` before the last scorer-tampering assertion (that test passes alone), shared-Vault hook journal publication `1/1`. The production caller audit and full verification have not yet been rerun over the current repair snapshot, so this document is an interim audit checkpoint, not readiness or acceptance. Interim WIP source commit `6bdf96d` was uploaded at the user's request and matched the remote branch SHA; the prior reviews are not acceptance. Public version remains `baron 5.0.0`.

## Search coverage

The initial closure-baseline scan found 320 tracked matching lines, 222 in executable-source extensions (including imports, definitions, comments, and tests). The current source-tree scan uses the authority patterns over `crates/baron-core/src` and `crates/baron-cli/src` and finds 258 matching lines; this count includes inline tests as well as imports, declarations, comments, and callsites. Search patterns:

```text
CURRENT\.md|active_plan\(|active_plan_authority\(|active_plan_operation_binding\(|current_plan_title|current_plan_risk|current_risk|reconcile\(|latest_proof\(|latest_trace
active_plan_authority_for|active_plan_completion_evidence_status|record_trace\(|score_trace\(|record_proof|proof_for_operation\(|trace_for_operation\(|record_continuity|compile_task_state|gate_evidence_status|runtime_backend_report|record_lifecycle_event_for_operation
```

Each row below represents the body of a caller, not a classification inferred from its name. Baseline line numbers are provenance only and are not current line references. All correctness-sensitive rows below have a final disposition; `BUG` is intentionally absent.

## Native hook payload contract checked against official references

Rechecked on 2026-10-05 against the [Codex Hooks reference](https://developers.openai.com/codex/hooks/) and [Claude Code Hooks reference](https://code.claude.com/docs/en/hooks). Codex exposes `session_id` on all hooks and event-scoped `turn_id` on `UserPromptSubmit`, `PreCompact`, `PostToolUse`, and `Stop`; the prompt text is `prompt` on `UserPromptSubmit`, while Stop supplies `last_assistant_message` rather than the original prompt. Claude exposes common `session_id`; `prompt_id` is a common field from v2.1.196 onward and is absent until the first user input. Claude `UserPromptSubmit` carries `prompt`; `PreCompact` carries `trigger` and `custom_instructions`; Stop carries `stop_hook_active`, `last_assistant_message`, `background_tasks`, and `session_crons`, not the original prompt. Claude has no host turn key in the documented PostToolUse payload; its optional common `prompt_id` remains the correlation key when present.

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
| CLI proof record, trace record, score auto-selection (main 2945/3093/3116) | EXACT_OPERATION_SCOPED with complete selectors and indexed ACTIVE authority; AMBIGUOUS_FAIL_CLOSED for no selector. Receipt tuple is its own exact selector and cannot use the legacy unindexed-plan fallback. |
| CLI refresh reconciliation report (main 3755) | PRESENTATION_ONLY; never marks completion. |
| plan authority / operation binding exports (227/230/244/247) | EXACT_OPERATION_SCOPED for a sole identified plan; AMBIGUOUS_FAIL_CLOSED for multiple managed active plans. Unbound legacy plans retain CURRENT-linked compatibility. |
| plan exact authority, completion, update, interrupt and completion wrappers (178/189/198/212/216/288/399/536/585/643/1330) | EXACT_OPERATION_SCOPED; identified authority and completion ignore CURRENT, while unbound legacy integrity still validates its linked projection. |
| exact plan lookup and CURRENT sentinel (1057/1061/1093/1094/1098) | EXACT_OPERATION_SCOPED through ACTIVE and canonical frontmatter; CURRENT is not read for identified selection. |
| identified start/resume compatibility check (394/403/405) | EXACT_OPERATION_SCOPED; same-title anti-hijack checks scan managed canonical plans rather than CURRENT. |
| legacy start/resume (400) | LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED. A sole identified operation comes from ACTIVE/frontmatter; unbound legacy resume still needs its linked CURRENT. |
| legacy update/interrupt/complete / selector (525/574/632/1341/1346/1350/1354/1364) | LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED. Count managed active plans before reading CURRENT; a sole identified plan ignores stale/malformed CURRENT, multiple plans fail closed, and unbound legacy plans require an exact compatible pointer. |
| legacy completion status precheck (254/259/263/265) | EXACT_OPERATION_SCOPED for a sole identified plan; AMBIGUOUS_FAIL_CLOSED for multiple plans. CURRENT is only consulted for unbound legacy authority. |
| ACTIVE discovery and linked plan validation (1063/1068/1090/1152/1393, 928/972/980/1532/1907) | EXACT_OPERATION_SCOPED / AMBIGUOUS_FAIL_CLOSED. Duplicate claims, unsafe paths, status, identity and canonical frontmatter conflicts reject. |
| completion proof, trace and gate evidence (729/762/781) | EXACT_OPERATION_SCOPED; exact proof and fresh trace for that proof, scoped trusted gate receipts. |
| plan status/completion-integrity presentation (801/806/843/859) | PRESENTATION_ONLY. CURRENT tamper diagnostics remain; identified context uses the new exact canonical plan view. |
| plan CURRENT projection publication (911/912), excluding CURRENT/INDEX/ACTIVE from discovery (1178) | PRESENTATION_ONLY publication; EXACT_OPERATION_SCOPED discovery exclusion. |
| automation reconcile / automation status (713/761) | EXACT_OPERATION_SCOPED / AMBIGUOUS_FAIL_CLOSED underneath PRESENTATION_ONLY status for identified work; unbound legacy status remains projection-linked. Missing CURRENT does not hide active identified work. |
| identified Stop / reconcile (542/735) | EXACT_OPERATION_SCOPED; durable host correlation precedes exact authority. Unknown, malformed, stale and ambiguous correlation fails closed. Session-only Stop also checks all validated active bindings for its adapter/session; a concurrent active plan without a correlation row makes the selection ambiguous. Final response publication revalidates the exact identity and recomputes proof/trace completion evidence under the project lock, so evidence changed after the initial reconciliation cannot leave stale passing metadata. |
| Core unscoped trace recording (119/125/224/225/227/238/242) | EXACT_OPERATION_SCOPED for a sole identified plan with its own exact bound proof; absent proof and multi-active ambiguity fail before writes. Unbound diagnostic traces remain available only when no identified operation claims implicit authority, and cannot satisfy identified completion. |
| bound trace publication/revalidation (161/203) | EXACT_OPERATION_SCOPED; authority and proof are rechecked after unlocked Git work. |
| bound trace story selection (229) | EXACT_OPERATION_SCOPED association through the exact ACTIVE plan's validated task/title and one unique managed story; does not read harness CURRENT. Missing, duplicate, malformed-risk, or linked story-tree state fails closed / remains missing. |
| trace score/find_trace mutation (294/301/616/628) | Explicit ID: EXACT_OPERATION_SCOPED. Auto-score: LEGACY_SINGLE_ACTIVE_SAFE / AMBIGUOUS_FAIL_CLOSED; no global newest operation fallback. |
| bound trace scoring (333/347/440) | EXACT_OPERATION_SCOPED. Unbound legacy scoring is diagnostic and cannot satisfy identified completion. |
| global latest score cache (473/474/479/482) | PRESENTATION_ONLY; operation authority uses fresh evaluation instead. |
| exact trace/proof selection (487/491/495/507/519) | EXACT_OPERATION_SCOPED, including proof ID and all lifecycle fields. |
| raw CURRENT title/risk helpers (738/739/746/748) | PRESENTATION_ONLY; removed from the trace correctness path. |
| proof complete binding / receipt recording / exact reader and validation projection (60/98/104/129/136/164/291/303) | EXACT_OPERATION_SCOPED; `TEST_MATRIX` promotion validates unique exact ACTIVE/story ownership before any proof/runtime write and updates only that operation. Unbound proof retains legacy current-story projection; incomplete operation context cannot touch a story. |
| deprecated unbound receipt/capability APIs (53/123) | AMBIGUOUS_FAIL_CLOSED; explicit binding required. |
| proof harness validation promotion (234/235 -> harness 203/206) | EXACT_OPERATION_SCOPED association; proof A cannot promote B's current TEST_MATRIX story. |
| global latest proof/status (251/260) | PRESENTATION_ONLY. Identified consumers use exact operation proof. |
| operation/event continuity checkpoint ingress (132/143/152/164/180/181/194) | EXACT_OPERATION_SCOPED; retain full identity in metadata and evidence selection. |
| checkpoint evidence composition (260/261/262/263/264) | EXACT_OPERATION_SCOPED; no CURRENT plan/global proof/cached pass/unrelated journal event in A-labelled checkpoint. |
| recovery ingress/composition | EXACT_OPERATION_SCOPED with additive identity selector and validated ACTIVE/frontmatter; linked plan/proof/trace belong to that identity, and operation-local RECOVERY is durable. No-selector safe canonical singleton is LEGACY_SINGLE_ACTIVE_SAFE; multiple managed active plans are AMBIGUOUS_FAIL_CLOSED before writes. The former actionable mixed CURRENT/global-latest composition was BUG, not presentation, and is repaired. Legacy unbound packets remain diagnostics when no identified active authority exists. |
| continuity status | PRESENTATION_ONLY shared latest view. Identified checkpoint/Task State/Prepare use immutable operation/event archives and operation-local CHECKPOINT/RECOVERY records, not shared CURRENT selection. |
| Task State operation compilation (103/114/115/117/190/191/222/223/239/253) | EXACT_OPERATION_SCOPED; full identity retained, unmatched intent/recovery remains unknown, and an identity-only checkpoint without an exact active plan is not resumed continuity. |
| Prepare continuity, resumed state and blockers (373/375/377/430/521/770) | EXACT_OPERATION_SCOPED canonical plan / matched continuity. |
| identified intent producer / Prepare and Task State consumer | EXACT_OPERATION_SCOPED mirrored `intent-operations/<operation_id>.md` Repo/Vault record; complete identity header validated, `CURRENT_INTENT.md` is projection only, legacy unbound producer remains display/intake-only for identified flows. Oversized/corrupt operation records fail closed. |
| Prepare verification proof/trace/gates/runtime (390/401/407/412) | EXACT_OPERATION_SCOPED; existing exact verification preserved. |
| identified context compile/state injection (110/146/147/172/318/334/576), operation context overload (237/247/252) | EXACT_OPERATION_SCOPED; preserve identity and suppress unrelated shared resume/execution excerpts. |
| context “why loaded” presence checks (385/404) | PRESENTATION_ONLY. |
| context maintainer status/history/audit fallback (577/579/590/597/603/654) | PRESENTATION_ONLY / OUT_OF_SPEC diagnostics; not identified authority. |
| knowledge runtime resume brief (269/270/272/278/293 -> intelligence 569), candidate runtime (364/451) | EXACT_OPERATION_SCOPED input for identified context; unscoped diagnostic API preserved. |
| intelligence benchmark runtime (802) | OUT_OF_SPEC; benchmark diagnostics do not authorize completion. |
| control-plane changed-file routing hints (772/773/792/369) | Generic route and adapter-only compatibility state may use shared projections as advisory routing hints (`PRESENTATION_ONLY` / `OUT_OF_SPEC` for completion). Identified `route_task_for_operation` validates full LifecycleIdentity and reads only exact operation intent/checkpoint/recovery; it ignores shared CURRENT paths. `identified_routing_does_not_consume_another_operations_current_paths` proves operation A keeps its database route while CURRENT=B advertises frontend paths. Routing remains advisory, never proof/completion authorization. |
| exact gate evidence (1540/1561/1645) | EXACT_OPERATION_SCOPED / AMBIGUOUS_FAIL_CLOSED; full identity and trusted receipt binding. |
| compatibility gate reports (1477/1488/1507/1577/1686; CLI 3514) | PRESENTATION_ONLY / OUT_OF_SPEC; not identified completion or Prepare verification. |
| capability required execution checks (427/515/527/612/653/866/887) | EXACT_OPERATION_SCOPED; capability/provider/verified receipt identity. |
| adapter-only capability/runtime reports (413/601/751; CLI 3693; certification 379) | PRESENTATION_ONLY / OUT_OF_SPEC; cannot satisfy operation execution evidence. |
| diagnostic runtime JSONL attachments (561/583; proof 220) | PRESENTATION_ONLY diagnostic_attachment; authoritative runtime checks use verified receipts. |
| generic lifecycle journal (651/680/686/701; CLI 2688/4095/4106/4111) | PRESENTATION_ONLY; never substitutes for durable exact hook correlation. |
| harness story projection/status (87/89/156) | PRESENTATION_ONLY. |
| raw shared harness title/risk helpers (175/188) | PRESENTATION_ONLY and legacy diagnostics only. |
| identified harness story/validation helper (203/206) | EXACT_OPERATION_SCOPED through the exact active plan, unique managed story matching its canonical task, and unique same-task ACTIVE ownership; ignores harness CURRENT and rejects linked/duplicate story trees. |
| hook response journal read/dedup/append (automation 471/997/1028) | EXACT_OPERATION_SCOPED shared resource mutation; all journal lookup and publication paths acquire checkout then shared Vault capsule lock. Local per-checkout dedup remains fenced by its claim token. |
| harness improvement project-wide audit (51/53/70/74/75) | EXACT_OPERATION_SCOPED through `active_plan_completion_evidence_status` for a sole identified ACTIVE plan; multiple active plans return explicit failure diagnostics, not a CURRENT/latest-proof guess. Unbound legacy audit remains a diagnostic compatibility path. `harness_audit_does_not_treat_current_b_evidence_as_unambiguous_for_a_and_b` covers A+B active with only B passing. |
| Autopilot status/provenance sources (557/1704/1706/1707) | PRESENTATION_ONLY; no CURRENT selection or silent truth promotion. |
| adapter update rollback_local_reconcile (875/883/1047/1075/1566) | OUT_OF_SPEC filesystem rollback, not plan/evidence reconciliation. |

Test fixtures, imports, declarations, comments and historical documentation are OUT_OF_SPEC as production authority callers. All production match groups are represented above. Context/replay/index/cache writers remain outside the SPEC-05 transaction scope; this repair removes their misuse as identified truth without introducing SPEC-06 machinery.

## Latest fresh review findings and focused repairs (2026-10-06)

- `FIX REQUIRED`: two fresh read-only reviews of `ecaf362..4a3efc0` found three Important issues; those verdicts are not acceptance. The complete source-tree caller scan and full verification must be repeated after the current changes.
- Harness story authority: RED `identified_operations_resolve_their_own_story_after_b_updates_current` returned no story for A after B updated harness CURRENT. GREEN resolver uses the exact identified ACTIVE plan title, requires unique same-task active ownership, searches managed story artifacts by canonical title, and ignores CURRENT; duplicates/linked story tree/risk conflicts fail closed. Proof promotion and bound trace regression verifies A still uses A story while B is CURRENT.
- Legacy trace proof provenance: RED `legacy_trace_cannot_borrow_a_completed_operations_bound_proof` attached A's bound proof ID to unbound B's trace. GREEN legacy trace creation only selects an unbound proof; score evaluation resolves the proof ID and rejects any proof with operation binding. The regression additionally rewrites B's trace to reference A's proof and confirms scoring fails with `unbound proof binding`.
- Shared hook response journal: RED `hook_response_publication_serializes_shared_vault_journal_across_checkouts` let checkout B publish/deduplicate while checkout A held the shared capsule lock. GREEN journal lookup, append helper and hook response publication acquire checkout then shared Vault lock across the journal read/dedup/append decision; the test confirms one shared event row and returns A's existing response after lock release.
- Focused green results: harness scoping `13/13`; proof/trace `33/33` before the final tampering assertion, which passed alone afterward; shared-Vault hook journal publication `1/1`. Full Core/CLI/adapters/workspace, Clippy, release, and complete CURRENT caller scan are not yet verified on this repair.

## Rulings and regressions

- CURRENT is a human-facing latest projection, not a correctness authority for identified operations. Identified authority and completion-status readers derive from validated ACTIVE/frontmatter; no-selector legacy APIs count managed active plans first, use the sole identified plan, and fail ambiguous on multiplicity. Unbound legacy paths continue to require a compatible CURRENT link.
- Fresh review finding fixed: a completed lifecycle identity could be started again, replacing its completed ACTIVE row and reviving old hook/evidence authority. `completed_operation_identity_cannot_reopen_a_plan` now fails before any ACTIVE, plan-file, or plan-count mutation; new lifecycle work must use a fresh canonical identity.
- Fresh review finding fixed: a stale/missing CURRENT path could veto a sole identified operation in authority/completion reads and no-selector mutation. `sole_active_operation_ignores_stale_current_for_binding_and_authority` first reproduced the failure in authority selection and again in legacy `update_plan`; after repair it proves exact A remains selected, completion/trace do not borrow CURRENT, and the legacy update targets A. CURRENT metadata mismatch and unsafe-pointer regressions confirm projection text cannot replace ACTIVE/frontmatter authority; canonical ACTIVE/frontmatter tamper tests remain fail-closed.
- Preserve same-title anti-hijack behavior from canonical plan files, not from a latest projection. Session-only host delivery cannot prove a turn; it must never merge distinct native operations solely by text.
- Fresh review finding fixed: an id-less Claude Stop could match the only retained completed mapping for A while a distinct active plan B for the same adapter/session had no correlation row. Stop returned the benign completed-A response and skipped reconciliation. `old_claude_session_only_stop_cannot_hide_unmapped_active_operation` reproduced this (RED: `continue`, `operation_already_completed=true`); resolution now enumerates validated active plan bindings for that adapter/session and fails closed if any other operation is active. The test passes in hook identity and all-target runs.
- Reassessed prior Minor: plan mutation entry points now hold checkout→Vault locks across shared `Plans/INDEX.md` reads/writes; a held shared-Vault lock fails before writes (`plan_mutation_fails_without_writes_when_shared_vault_lock_is_held`). The old concurrent Vault-index lost-update concern is therefore no longer present in current code.
- Deferred Minor from independent review: journal response lookup reads the full JSONL before limiting the reverse scan; dedup capacity trimming can evict a live in-flight claim (token fencing prevents stale publication, while a retry can still repeat pre-publication work); trace scoring scans the archive while holding repo/Vault locks (bounded lock timeout remains fail-closed). Keep these as Minor unless final reviewers provide evidence that raises their impact.
- Core no-selector trace APIs require their own guard; fixing CLI alone is insufficient. Regressions cover multi-active no-write ambiguity and sole A with newer completed B evidence.
- Explicit CLI receipt selectors require the exact indexed ACTIVE row; the legacy compatibility scanner may still read old plans but cannot authorize receipt-backed proof ingress.
- Bound proof publication uses the exact active plan risk and cannot update a different CURRENT story. Malformed owned story metadata is rejected before proof, receipt-use, or runtime evidence publication.
- `record_trace` remains fail-closed when one identified operation is active but lacks its own bound proof. Legacy unbound trace records are diagnostics only when no identified operation is implicitly selected; they cannot satisfy later identified completion.
- A matching identity-bearing continuity checkpoint without an exact active plan remains diagnostic and cannot make Task State claim the operation is resumed.
- Confirmed intent is correctness-sensitive too: identified Prepare no longer consumes the unbound legacy CURRENT intent. `record_intent_for_operation` writes a bounded, identity-header-validated record per operation in Repo and Vault; Prepare and Task State read that path only. The ordinary CURRENT remains latest presentation, and an unbound/mismatched intent cannot satisfy another operation's confirmation blocker.
- Baseline `ecaf362` `session_replay::one_line` calls `String::truncate(limit - 3)` without checking a UTF-8 boundary. With the synthetic 216-ASCII-plus-`é` fixture, that exact pre-fix operation panics at byte 217 (source-equivalent reproduction exit 101). The regression test now lives in `session_replay`: baseline suite `4/4`, current suite `5/5`; current `session_replay_rendering_never_splits_a_multibyte_character` passes. Separately, `prepare_cli` passes `5/5` at baseline and `6/6` currently. All project tests use nonexistent session roots.
- Host session roots are overridden to nonexistent temporary paths during local test execution: this isolates the host's real Codex/Claude transcript trees from test fixtures without changing product behavior. Normal context compilation still performs the existing bounded session import behavior.
- Historical final local verification checkpoint (2026-10-04): Core 584/0 across 60 result suites; CLI 190/0 across 38 (one release-only test ignored); adapters 124/0 across 14 (three ignored); workspace 898/0 across 112 (four ignored). The explicit ignored release smoke passed separately. This is historical evidence superseded by the 2026-10-05 counts above. Test runs isolate real Codex/Claude session roots with unique nonexistent temporary overrides and use the bundled PowerShell 7 host. Exact-range independent reviews remain pending; no closure or GitHub CI result is claimed.
