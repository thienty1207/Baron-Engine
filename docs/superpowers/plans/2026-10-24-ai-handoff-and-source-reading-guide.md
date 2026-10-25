# AI Handoff and Source Reading Guide Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `docs/superpowers/plans/CURRENT.md` Baron Engine's single, reliable AI handoff and add a repeatable guide for reading the complete source tree.

**Architecture:** Keep the existing current plan path as the canonical handoff. Keep the Markdown status dashboard, JSON status, and build log for their distinct release, machine-readable, and verification roles, but link them to the handoff and label historical snapshots clearly. Add a dynamic source-reading guide that derives file coverage from the checked-out tree.

**Tech Stack:** Markdown, JSON, PowerShell, Rust/Cargo verification.

**Spec:** `docs/superpowers/specs/2026-10-24-ai-handoff-and-source-reading-guide-design.md`

## Global Constraints

- Stable and downloadable release remain `5.0.0`; source candidate is `5.0.1` and remains unpublished.
- `docs/superpowers/plans/CURRENT.md` is the only canonical project handoff; do not create `docs/current/current.md`.
- Delete a documentation file only after proving it is a true duplicate, has no unique evidence, and its inbound references are safe to remove.
- Preserve the user's existing documents, historical plans, unique build evidence, and machine/Vault state.
- Commit and push the candidate branch as requested; do not tag, publish, merge, or create a release.

## Review Focus

- Older text in `CURRENT.md` or build logs must not override the current v5.0.1 candidate state.
- No status or README field may imply that v5.0.1 is already stable or downloadable.
- JSON `readFirst` must stop routing readers first to the old 4.2-era document list.
- The source guide must cover Core, CLI, adapters, canonical assets, tests, and CI while requiring explicit coverage accounting.
- Pushed-branch status must not imply that a PR or hosted CI run exists.

---

### Task 1: Establish the handoff and source-reading path

**Files:**
- Modify: `docs/superpowers/plans/CURRENT.md`
- Create: `docs/architecture/SOURCE_READING_GUIDE.md`
- Modify: `AGENTS.md`, `README.md`, `docs/superpowers/README.md`
- Modify: `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `notes/build-log/CURRENT.md`
- Modify: `docs/superpowers/plans/2026-10-24-v5-0-1-version-bump.md`
- Modify: `docs/superpowers/specs/2026-10-24-ai-handoff-and-source-reading-guide-design.md`

**Interfaces:**
- Consumes: current release truth from `docs/BARON_STATUS.json`, current branch evidence, `AGENTS.md` product invariants, and the actual source tree.
- Produces: one linked handoff path and a source guide with reproducible inventory and coverage steps.

- [x] **Step 1: Confirm document deletion candidates**

Run SHA-256 duplicate checks over `docs/` and all tracked/untracked Markdown documents; inspect inbound references for any exact match. Delete only a verified duplicate document that has no unique evidence. The initial `docs/` scan found no byte-identical files; the three identical Markdown files found under adapter test fixtures are fixtures, not documentation, and remain untouched.

Expected: no documentation deletion unless the expanded scan finds a genuine duplicate.

- [x] **Step 2: Write the canonical handoff and source-reading guide**

Rewrite the opening handoff in `docs/superpowers/plans/CURRENT.md` to state the current branch/base, 5.0.1 unpublished source-candidate boundary, project implementation/spec status, verification evidence, known host limitations, and safe next action. Label older checkpoint material as historical so it cannot appear active. The new source guide will cover the Core, CLI, adapter, `assets/core`, tests, and workflow paths; include commands that inventory the current checkout; define a reading order and per-file coverage ledger; and prohibit a 100% scan claim until every inventoried source file is accounted for.

- [x] **Step 3: Route repository readers to the canonical documents**

Update `AGENTS.md`, `README.md`, and `docs/superpowers/README.md` with links to the handoff and source guide. Keep the product invariants and command boundary unchanged.

- [x] **Step 4: Reconcile status and evidence roles**

Update the current sections of `docs/BARON_STATUS.md` and `notes/build-log/CURRENT.md` to link to the handoff and state that older checkpoints are historical. Update the JSON `readFirst` list to the current read order. Record that owner authorization now permits committing and pushing `codex/version-5-0-1`; preserve the 5.0.0 stable/downloadable values and state that no tag, publication, PR, or hosted CI run exists.

- [x] **Step 5: Run focused documentation verification**

Run the public trust docs test, parse the status JSON, validate repository-relative links in changed Markdown, and run `git diff --check`.

Expected: focused tests pass, JSON parses, every checked relative link resolves, and the diff check exits 0.

- [x] **Step 6: Review the complete candidate and commit**

Review the full diff, including the pending 5.0.1 version-bump work, and stage only the documented candidate files and plans. Commit with a message describing the 5.0.1 candidate and AI handoff update.

- [x] **Step 7: Record task completion**

Run `cargo test -p baron-core --test public_trust_docs` after the commit and record the result in the task ledger. The full repository-required verification suite is run separately below.

---

## Final Verification and Delivery

Run the repository-required format, full workspace tests, and warnings-denied Clippy. For documentation, also run focused Core and adapter contract suites, user-facing CLI help checks, link validation, status JSON parsing, the retired-adapter gate, and `git diff --check`. Obtain a fresh review of the complete local branch, address any Important findings, then push `codex/version-5-0-1` to `origin`. Do not open a PR unless separately requested.
