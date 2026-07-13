# Baron 3.3 Trust And State Safety Design

## Goal

Make Baron distinguish read-only requests from repository-changing work, reject state mutations when project and Vault identity are incoherent, preserve proof-backed completion, and promote releases only after the exact source and every native artifact are verified.

## Scope

Baron 3.3 adds three tightly bounded safeguards:

1. A request-authority classifier that defaults ambiguous requests to read-only and keeps answer, explanation, review, diagnosis, planning, and status work from creating execution records.
2. A state-coherence guard for mutating AI commands, plus read-only SQLite access for query paths and explicit completion-integrity diagnostics.
3. An immutable release-candidate workflow that proves source identity, native assets, checksums, installer lifecycle, and upgrade compatibility before creating a tag or GitHub Release.

The public user flow remains install, Vault setup, project init, platform selection, and update. Superpowers remains workflow core. The three core quality agents and Vault Markdown source of truth do not change.

## Request Authority

Baron classifies a request as `read_only`, `change`, or `ambiguous`. Explicit change outcomes win over review words, so “review and apply fixes” is a change request. An ambiguous request does not authorize durable state mutation. Generated adapters run the authority check before plan, Harness, proof, trace, review, or Autopilot writes.

Read-only work may inspect the repository, Vault, status, and bounded context but must not create intake, plan, proof, trace, backlog, or learning records. This prevents automation itself from becoming noise.

## Coherent State And Completion

Mutating commands require a parseable supported project config, a resolvable local Vault, an existing project capsule, and matching project identity metadata. Missing or mismatched state returns a recovery instruction and leaves files unchanged. Init and update remain the repair paths because they own scaffold reconciliation.

SQLite-backed search opens indexes read-only. Query code cannot silently create a new empty database. Plan status reports completion integrity; a completed state without verification, proof, or a passing trace is reported as invalid rather than trusted.

## Immutable Release Promotion

Release preparation starts from an exact commit on `main`, not from an already-created tag. CI verifies source version, formatting, tests, Clippy, native builds, binary versions, checksums, release manifest identity, and installer lifecycle. Only the final promotion job receives write permission. It refuses an existing tag or release, creates one annotated tag for the proven commit, then publishes immutable assets without replacement.

## Error Handling

- Unknown request authority remains read-only and explains what is missing.
- Incoherent state names the conflicting path or identity and points to `baron update`.
- Missing SQLite indexes produce a clear rebuild instruction rather than creating empty state.
- Release identity drift, missing targets, changed checksums, existing tags, or existing releases stop promotion.

## Verification

Tests cover English and Vietnamese request authority, mixed review-and-fix prompts, no-write read-only behavior, capsule identity mismatch, missing state, read-only database access, completion tampering, release identity mismatch, workflow permissions, proof-before-tag ordering, installer lifecycle, and a real initialized project smoke.
