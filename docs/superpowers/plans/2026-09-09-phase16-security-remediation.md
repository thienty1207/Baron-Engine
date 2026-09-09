# Baron Phase 16 Security Remediation

**Status:** `complete; superseded by the 5.0.0 release-candidate checkpoint`

**Scope:** remediate only SEC-03 release metadata authenticity and SEC-04
proof/gate evidence authority, then rerun the complete Phase 16 local release
gate. Keep the product at 4.2.2 and stop before version bump, commit, push,
tag, or publication.

## SEC-03

- Add a deterministic `ReleaseManifestV1` payload and signed envelope.
- Verify Ed25519 signatures against compiled public key IDs before consuming
  version, target, artifact, size, digest, identity, or compatibility fields.
- Keep artifact size and SHA-256 checks after authentication.
- Require the protected `BARON_RELEASE_SIGNING_KEY` workflow secret; never
  commit or embed private signing material.
- Cover canonicalization, duplicate/unknown fields, tampering, trust-root,
  platform, and mutation-safety cases.

## SEC-04

- Add exact task, operation, adapter, session, request, gate, and source
  identity to trusted execution receipts.
- Keep persisted unkeyed receipts diagnostic after process restart; only a
  receipt created by the current trusted runner can satisfy a gate.
- Make free-form gate/capability evidence diagnostic and require a bound
  receipt for authoritative status.
- Cover cross-task, cross-gate, stale-source, failed, replay, and child-text
  substitution cases.

## Verification

Run focused release and receipt tests first, then the complete workspace
formatter, test, Clippy, release-build, diff, retired-adapter, and Phase 16
adversarial matrix. Run a fresh native security review after the repaired
snapshot. Report TAC availability separately. The protected signing-key
contract is validated in workflow source without reading the production
secret on this host.

## Checkpoint result (2026-09-09)

- SEC-04 is closed for this checkpoint: free-form evidence is diagnostic only;
  current-operation receipts retain exact project/task/operation/adapter/
  session/request/gate/source bindings, and persisted receipts are diagnostic
  after restart. Cross-task, cross-gate, replay, source-change, and recomputed
  persisted-record tests pass.
- SEC-03 is closed: implementation tests pass for canonicalization,
  signature-first verification, pinned-key trust, tampering, exact release
  identity, platform/size/digest checks, and mutation safety. Both bootstrap
  installers verify the detached signature before parsing release fields or
  mutating the existing installation. The final native Codex Security scan
  (`450e2e04-14c0-40d5-985c-9579637e4a35`) found zero reportable findings and
  no snapshot-change warning. The remote bootstrap-script self-authentication
  limitation is documented explicitly; it does not weaken verification once a
  trusted installer copy is executing.
- The complete pre-bump workspace test matrix, formatter, Clippy with warnings
  denied, release build, 4.2.2 binary smoke, diff checks, and retired-adapter
  guards passed with zero unexpected failures. SEC-04 remains closed. TAC is
  unavailable on this host and is not represented as a passing scan.
- The follow-on Phase 16 release-candidate checkpoint then completed the clean
  target boundary, bumped the product to 5.0.0, rebuilt the release binary,
  and reran the complete post-bump matrix. Commit, push, hosted CI, tag, and
  GitHub Release remain outside this remediation checkpoint.
