# Baron 3.1 Phase 24 - Public Trust Release Plan

Date: 2026-06-19

## Goal

Ship Baron `3.1.2` as a public-trust release: concise README, public demo,
certification snapshot, release/latest clarity, and synced
version/status metadata.

## Non-Negotiables

- Do not add new core engine behavior.
- Do not replace Superpowers or the three core quality gates.
- Do not hide automation commands from agents; keep them out of the public
  README command flow.
- Do not claim `releases/latest` is fixed until a GitHub release/tag exists.

## Checklist

- [x] Add RED tests for public-trust docs and status metadata.
- [x] Bump source release metadata to `3.1.2`.
- [x] Rewrite README as a concise public landing page.
- [x] Add `docs/demo/README.md`.
- [x] Keep public proof docs Baron-owned without external comparison links.
- [x] Add `docs/assessment/baron-3-public-certification.md`.
- [x] Update release docs for tag publication and `releases/latest`.
- [x] Update `docs/BARON_STATUS.md` and `docs/BARON_STATUS.json`.
- [x] Run full source verification.
- [x] Commit, tag, push, and verify GitHub release latest.
