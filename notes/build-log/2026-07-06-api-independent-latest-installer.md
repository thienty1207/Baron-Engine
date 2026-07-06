# API-Independent Latest Installer Build Log

Date: 2026-07-06
Target: Baron 3.1.4
Status: completed and released in 3.1.4

## Trigger

The first `v3.1.3` release smoke reached the published installer but failed
while resolving latest because the shared anonymous GitHub API quota was
exhausted.

## Evidence

- `v3.1.3` CI and release workflows passed.
- HTTP `releases/latest` redirected correctly to `v3.1.3`.
- The installer failed only at its unnecessary `api.github.com` latest lookup.
- RED installer contract test failed before the fix.
- GREEN installer contract test passed after both installers switched to the
  published `release-manifest.json` asset.

## Current Checkpoint

- Root cause: confirmed.
- PowerShell installer fix: implemented.
- Linux/macOS shell installer fix: implemented.
- Manifest-based latest smoke during active API rate limiting: passed and
  installed Baron 3.1.3 from the published release manifest.
- Full lifecycle tests: passed, 5 tests.
- Full workspace tests: passed.
- Formatting and Clippy with warnings denied: passed.
- GitHub main CI `28797881851`: passed.
- GitHub release workflow `28797886356`: passed.
- Public latest installer update from 3.1.2 to 3.1.4: passed.
- Real `scanjob` init against the existing shared Vault: passed.
- Next action: none; wait for an explicitly approved future phase.
