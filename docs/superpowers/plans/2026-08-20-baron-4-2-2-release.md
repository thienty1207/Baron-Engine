# Baron 4.2.2 Release Plan

- [x] Phase 118: release identity and current metadata
  - [x] Bump the Cargo workspace, lockfile, certification target, and current
    release metadata to `4.2.2`.
  - [x] Add the changelog entry and preserve historical `4.2.1` records.
  - [x] Update README/release/install guidance to point at `v4.2.2`.

- [x] Phase 119: local release proof
  - [x] Run formatting, adapter parity, CLI, public-doc, workspace, Clippy,
    locked release build, release metadata, and binary smoke gates.
  - [x] Record the two Windows-only archive-module blocks without weakening a
    gate; all parity, engine, memory, CLI, and release tests passed.
  - [x] Mark the completed Reasonix parity work as included in `4.2.2`.

- [x] Phase 120: immutable GitHub publication and handoff
  - [x] Stage only the intended `4.2.2` source, parity, documentation, and
    release files; commit the exact source.
  - [x] Push the exact source to `origin/main` and `agent/baron-4-0`, then
    publish the annotated tag `v4.2.2`.
  - [x] Verify GitHub Actions, native archives, checksums, installers,
    `releases/latest`, and the final README/status handoff.

## Verification record

Local proof is complete; the commit SHA, tag, workflow URL, release URL, asset
list, and final `baron --version` output are appended after publication.

- `cargo test -p baron-adapters --all-targets --no-fail-fast`: 3 unit, 30
  lifecycle, 15 planner, and 1 transaction test passed.
- `cargo test -p baron-cli --test reasonix_adapter_cli --no-fail-fast`: 6/6
  passed, including the full Codex -> Reasonix -> Claude -> Generic -> Codex
  round trip.
- `cargo test -p baron-core --test public_trust_docs --no-fail-fast`: 9/9
  passed.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets
  -- -D warnings`: passed.
- `cargo build --release --locked -p baron-cli`: passed; binary reports
  `baron 4.2.2`.
- `cargo test --workspace --all-targets --no-fail-fast`: all engine, memory,
  adapter, CLI, fallback, and release tests passed; two installer lifecycle
  tests could not run because this Windows PowerShell cannot load
  `Microsoft.PowerShell.Archive`.
- Release metadata fixture using `baron release metadata` and `baron release
  verify`: passed for `4.2.2` and the complete four-target artifact set.
- Isolated release binary smoke: Codex init -> `baron --reasonix` created the
  complete Reasonix core view; binary reported `baron 4.2.2`.

## Final publication evidence

- Commit: `80f4daa13e15b2aafb53bbefe0f54454130fcaba`.
- Branches: `origin/main` and `origin/agent/baron-4-0` both point to the exact
  release source; annotated tag `v4.2.2` points to the same commit.
- Baron CI: run
  `https://github.com/thienty1207/Baron-Engine/actions/runs/32347122499`
  passed format/Clippy and all four native test jobs.
- Baron Release: run
  `https://github.com/thienty1207/Baron-Engine/actions/runs/32347138548`
  passed exact-source verification, the four-target build matrix, checksums,
  release metadata, installer lifecycle, and immutable promotion.
- Public release: `https://github.com/thienty1207/Baron-Engine/releases/tag/v4.2.2`;
  `releases/latest` resolves to `v4.2.2` with 12 assets. The Windows raw
  candidate SHA-256 is
  `D26B0B7DF1708DC1669E7DAD215002A37929755E804AFE766B818B11A36E5A49` and the
  Windows archive SHA-256 is
  `B298E7007CC060C5AAD1B0B11AB4C618C461ABAC8473B81A5A1EE38029D55A4B`.
- Final version proof: the release binary reports `baron 4.2.2`; no release
  phase remains open. The next action is normal 4.2.2 maintenance.
