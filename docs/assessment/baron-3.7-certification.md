# Baron 3.7 Integrated Certification

Status: Phase 51 locally certified on the Baron `3.6.0` source baseline;
Phase 52 public promotion is in progress.

## Scope

This report records the local evidence for Phases 46-51. It does not claim a
public Baron 3.7 release until the immutable GitHub promotion, native runner
matrix, release assets, and a fresh `releases/latest` install smoke agree on
`3.7.0`.

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
- Regression: core and CLI all-target suites pass independently, Clippy passes
  with `-D warnings`, the locked release build passes, and the release binary
  reports `baron 3.6.0` before the Phase 52 bump.
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
commands completed; the combined timeout is retained as runner evidence and is
not presented as a test pass.

## Phase 52 handoff

The next safe action is to bump synchronized metadata from `3.6.0` to `3.7.0`,
run the release manifest/checksum/installer gates, push the exact certified
source, verify the native GitHub matrix and immutable Release, then install
from public `releases/latest` in a fresh Windows directory. Exact SHA, workflow
run, asset list, and public smoke output must be appended here before Phase 52
is marked complete.
