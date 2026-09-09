# Release Metadata And Proof Authority

Baron release metadata is an authenticated `ReleaseManifestV1` plus a
`release-manifest.sig` signature record. The manifest is emitted as compact,
deterministic UTF-8 JSON from the fixed Rust field order. Baron signs those
exact bytes with the domain-separation marker
`baron-release-manifest-v1\0`; changing whitespace, line endings, ordering,
duplicate fields, or unknown fields changes the signed bytes or fails strict
parsing.

The updater resolves the key ID against the public key set compiled into the
Baron binary, verifies the Ed25519 signature, and only then validates release
version, source revision, platform, artifact names, sizes, digests, release
identity, and compatibility. It then checks the downloaded artifact size and
SHA-256. An unsigned or changed manifest cannot create staging state or mutate
the installed runtime. `SHA256SUMS` remains useful for manual checks and as a
second integrity comparison, but it is never an authority independent of the
authenticated manifest.

The current compiled production anchor is `baron-release-2026`. Its matching
private signing seed is not in this repository or in Baron binaries. Release
automation must inject the seed through the protected GitHub Actions secret
`BARON_RELEASE_SIGNING_KEY` for the metadata step; the workflow fails closed
when that secret is absent. Key rotation is an explicit source change that adds
the new public key and updates the protected secret and key ID together.

The Rust updater and both bootstrap installers use the same trust chain. The
installers require an HTTPS manifest and signature, verify Ed25519 with the
pinned key using OpenSSL, validate the selected platform artifact, then check
the signed size and SHA-256 before the first live-file mutation. If OpenSSL
with Ed25519 support is unavailable, installation fails closed with an
actionable error. `SHA256SUMS` is an integrity comparison only.

A remotely fetched installer cannot prove its own authenticity if the
distribution channel is compromised. Once a trusted copy of the installer is
executing, however, release metadata and binaries cannot be replaced without
the Baron signing private key.

Proof and control-plane gates use typed execution receipts from Baron's trusted
runner. A gate receipt carries project, task, operation, adapter, session,
request, gate, source fingerprint, command, result, and output digests. Gate
records must match all of those bindings and must be created by the current
Baron process. The persisted JSONL receipt is integrity checked for recovery
and diagnostics, but its unkeyed digest is not publisher authenticity; a
persisted receipt loaded after restart must be re-executed before it can satisfy
a security-sensitive gate. Free-form summaries, capability cache entries,
child-agent text, and Autopilot candidates remain diagnostic attachments.

See [RELEASE.md](../RELEASE.md) for the maintainer workflow. Do not publish a
release until a protected signing secret is configured and the authenticated
metadata, receipt binding, and complete release gates pass.
