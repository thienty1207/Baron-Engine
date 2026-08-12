# Baron 3.7 Integrated Certification

Status: Baron `3.7.0` publicly certified and released.

## Scope

This report records the local and public evidence for Phases 46-52. The
immutable GitHub promotion, native runner matrix, release assets, and fresh
`releases/latest` install smoke all agree on `3.7.0`.

## Passing evidence

- Work shape: read-only, focused-ephemeral, durable, high-risk, and
  English/Vietnamese ambiguity fixtures pass without unrelated lifecycle writes.
- Trusted receipts: the Baron-owned runner binds project identity, source
  content fingerprint, argv, provider, result, bounded output, redaction, and
  integrity; large output drains without pipe deadlock; stale, changed-source,
  tampered, failed, and wrong-project receipts cannot satisfy proof.
- Completion integrity: medium/high plans require a current receipt-backed proof
  and current receipts for `code-reviewer`, `security-auditor`, and
  `test-engineer`; edited Markdown alone is insufficient.
- Harness experiments: approval, baseline/hypothesis, fresh rerun, and
  keep/revise/remove/pending outcome lifecycle pass in core and CLI tests.
- Application runbook: project-owned bounded runbook fields route only to
  runtime-relevant tasks; unknown application facts stay unknown and Baron does
  not invent credentials, readiness, ports, interfaces, or cleanup ownership.
- Regression: the full Ubuntu workspace all-target suite and Clippy pass with
  `-D warnings`; local core/CLI suites, the locked release build, and the
  release binary report `baron 3.7.0`.
- Release profile: `baron certify run --profile release` passes on a fresh
  project/Vault; setup, adapter init, cache recovery, firewall, bounded context,
  optional code-map fallback, automation, autopilot, runtime policy, and the
  four-target release readiness checks pass.
- Public documentation: README/status JSON consistency, no external workflow
  dependency in public files, and the current 3.7 status contract pass.

## Known boundary

The combined `cargo test --workspace --all-targets --no-fail-fast` invocation
was attempted on Windows but timed out after emitting passing results from the
individual targets. The independent `baron-core` and `baron-cli` all-target
commands completed, and the authoritative Ubuntu release workflow then passed
the full workspace suite and Clippy. The Windows timeout is retained as runner
evidence and is not presented as a Windows full-suite pass.

## Public release evidence

- Source revision: `cc14c222130ac2047d36b3b752d9140521d3538e`
- GitHub Actions release run: [`31582187832`](https://github.com/thienty1207/Baron-Engine/actions/runs/31582187832)
- Immutable tag and Release: [`v3.7.0`](https://github.com/thienty1207/Baron-Engine/releases/tag/v3.7.0)
- Release assets: Windows x64, Linux x64, macOS Intel, macOS Apple Silicon
  archives and raw candidates, `SHA256SUMS`, `release-manifest.json`,
  `install.ps1`, and `install.sh`.
- Native proof: exact-source verification, Ubuntu full workspace tests,
  Clippy, all four native builds, exact binary version smoke, checksums,
  manifest verification, installer lifecycle, and immutable promotion passed.
- Fresh Windows public smoke directory:
  `C:\Users\tytyb\AppData\Local\Temp\baron-3-7-public-smoke-ec0b95dd0c56419cbb93f87292510cd0`.
  `releases/latest/download/install.ps1` installed `baron 3.7.0`; `setup`,
  `init --codex --fullstack`, `context`, same-version update protection, and
  user-owned marker preservation passed.
