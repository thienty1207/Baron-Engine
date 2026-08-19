# Baron 4.2.1 Patch Release Program

Reasonix was implemented after the immutable `v4.2.0` tag. This four-phase
patch program packages that existing adapter in a real downloadable binary;
it does not create Baron 4.3 or change the intelligence engine.

- [x] Phase 109: release identity and source truth
  - [x] Bump the workspace release to exactly `4.2.1`.
  - [x] Keep the existing Reasonix CLI/adapter implementation unchanged.
  - [x] Add the patch-release decision and root-cause record.
- [x] Phase 110: documentation, status, and release metadata
  - [x] Update README, release guide, changelog, architecture, status
    Markdown/JSON, and the active build log to `4.2.1` where they describe the
    current downloadable release.
  - [x] Record that 4.2.0 remains the prior engine release and 4.2.1 is the
    adapter-packaging patch.
- [ ] Phase 111: verification and binary proof
  - [x] Run format, workspace check/library tests, Clippy, locked release
    build, CLI help, Reasonix adapter tests, and release identity/metadata
    verification. The remaining full integration exceptions are recorded in
    `docs/BARON_STATUS.md` and are delegated to hosted native CI.
  - [x] Prove the release binary reports `baron 4.2.1` and exposes both
    `baron --reasonix` and `baron init --reasonix`.
  - [x] Record the local Windows release-binary SHA-256 hash for the narrowly
    scoped WDAC exception; the public raw asset hash will be recorded after
    GitHub promotion.
- [ ] Phase 112: immutable GitHub publication and handoff
  - [ ] Commit and push the exact source to `origin/main`.
  - [ ] Push `v4.2.1` only after local gates pass; let the immutable release
    workflow build all native assets and installers.
  - [ ] Verify `releases/latest`, README install output, checksums, and the
    final release URL; mark every task `[x]`.
