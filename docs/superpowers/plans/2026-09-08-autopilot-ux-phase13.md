# Phase 13: Autopilot UX, Safe Candidate Learning, and Conversational Approval

## Scope

Implement only the Phase 13 Autopilot candidate lifecycle. Candidates remain
untrusted observations until an explicit, project-scoped approval is correlated
to the pending proposal. Approval may create an evidence-backed project
decision through the existing decision authority, while runtime, skill, memory,
workflow, routing, proof, and managed-asset changes continue to require their
existing authorities. Phase 14 update/release hardening, Phase 15 documentation
rewrite, Phase 16 final verification/release, version changes, tags, pushes, and
history rewrites are deferred.

## Repository evidence

- `crates/baron-core/src/autopilot.rs` currently generates timestamp-based IDs,
  rewrites Markdown candidate files, records no scope/provenance/readiness or
  suppression state, and only changes a status line for approve/reject.
- The current Vault candidate file is discovered under `ProductHarness`; it
  must carry explicit candidate frontmatter or an equivalent firewall rule so
  pending learning cannot look like active memory during indexing.
- `PreparePacketV1` already has bounded `blockers`, `warnings`, and `unknowns`;
  approval status can be projected through those existing fields without a
  schema bump.
- Existing `record_decision`, intent, memory/firewall, managed transaction,
  project lock, and Phase 12 lifecycle authorities are the only promotion and
  persistence authorities to reuse.
- Phase 12 hooks can maintain bounded candidate state after meaningful events,
  but Stop must not create a candidate or promote one automatically.

## Execution order

- [x] Record the Phase 13 start checkpoint and add deterministic red tests for
      safety, lifecycle, UX, scope, suppression, conflicts, recovery, routing,
      memory, child boundaries, and cross-adapter behavior.
- [x] Define an additive, backward-tolerant Autopilot ledger representation in
      the existing repo/Vault Autopilot area. Preserve old Markdown entries and
      treat unknown or malformed records as untrusted/ambiguous.
- [x] Add stable candidate identity, deterministic observation deduplication,
      provenance, scope, impact, readiness, contradiction, and bounded
      housekeeping. Retain provenance while compacting duplicate evidence.
- [x] Keep pending candidates outside `TrustedRecallPolicy::Current`, Tier-0,
      task truth, routing authority, proof authority, and context truth; expose
      only a bounded approval-needed warning/status in existing lifecycle data.
- [x] Add conversational response parsing for approve, reject, defer, and
      correction without requiring internal IDs in normal UX. Correlate the
      response to the current project/task/session/request and persist the
      result idempotently across retries and adapters.
- [x] Route approved project decisions through `record_decision` (or the
      existing explicit authority for a selected destination). Never mutate
      Core assets, managed projections, policies, or trusted memory directly
      from an Autopilot candidate.
- [x] Add rejection/defer cooldown and suppression, explicit conflict surfacing,
      correction replacement, interruption recovery, and child-agent evidence
      boundaries. Explicit current intent remains authoritative.
- [x] Integrate bounded housekeeping with lifecycle-safe call sites and ensure
      Codex/Claude hooks remain accelerators with no popup/auto-promotion loop.
- [x] Run focused Phase 13 tests, relevant prior-phase regressions, and the
      required formatter/workspace test/Clippy/diff checks with the compatible
      Windows PowerShell module path when needed.
- [x] Update status, build log, and this plan with evidence; stop before Phase
      14.

## Proposed contracts

- Candidate identity is a deterministic hash of project ID, normalized
  semantic proposal, and explicit scope. Adapter/session/request provenance is
  evidence, not identity authority.
- Candidate records include status (`candidate`, `ready`, `approved`,
  `rejected`, `deferred`, `conflict`, `expired`), scope, impact, source
  provenance, evidence count, contradiction state, readiness explanation,
  correlation fields, and suppression/cooldown metadata. Missing fields in old
  records default to untrusted candidate state.
- Equivalent observations merge deterministically and retain distinct source
  provenance. Repetition, confidence, hook delivery, or Codex/Claude agreement
  never implies approval.
- Conversational responses accept yes/no/defer/correction forms and resolve
  against a bounded current pending approval. Ambiguous responses fail safe and
  do not change state. Correction records a replacement proposal and preserves
  the original history.
- Repeated approval/rejection/defer delivery with the same correlation is a
  byte-stable no-op. A pending decision survives interruption and can be
  completed from either supported adapter.
- Safe housekeeping may deduplicate metadata, expire weak stale candidates,
  prune bounded cache state, and archive resolved history without consent.

## Completion evidence

The final Phase 13 report must separate candidate-safety invariants from the
approval UX and list focused tests plus the full required verification commands.
It must report files, fixtures, persisted schema changes, PreparePacket
compatibility, retired-adapter gates, unexpected failures, discoveries, spec
adjustment, and Phase 14 readiness. No release, tag, push, or Phase 14 work is
allowed.

## Completion evidence (2026-09-08)

Implemented the additive `docs/baron/autopilot/STATE.json` ledger mirrored to
the project Vault, deterministic project-scoped candidate IDs, provenance and
readiness, duplicate observation merging, stale expiry and resolved archival,
bounded context/Prepare warnings, natural-language approval/correction/defer/
rejection, response idempotency, conflict and intent-authority handling, and
Codex/Claude cross-adapter correlation. Candidate and approval Markdown carries
candidate frontmatter so it cannot enter current trusted memory. Approved
project decisions use the existing Product Harness `record_decision` authority;
Core assets, routing, task truth, proof policy, and managed projections are not
mutated by Autopilot.

Focused Phase 13 Core tests pass `14/14` and CLI tests pass `2/2`. The complete
workspace command under the compatible system Windows PowerShell module path
passes with zero failures and three intentionally ignored historical fixtures.
Formatter, workspace Clippy, diff checks, prior trusted-memory/routing/
prepare/continuity/hooks/adapter/operation gates, and both retired-adapter zero
gates pass. No Project, managed-state v2, memory, or PreparePacket v1 schema was
changed. Phase 14 has not started.
