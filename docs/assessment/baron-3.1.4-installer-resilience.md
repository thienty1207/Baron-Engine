# Baron 3.1.4 Installer Resilience

Date: 2026-07-06
Status: released and verified

## Outcome

Baron's normal one-block latest installer no longer depends on anonymous GitHub
API quota. Both Windows and Linux/macOS installers resolve the release version
from the checksum-verified bundle's published manifest path.

## Evidence

- No-API installer contract failed before the fix and passed after it.
- Manifest-based latest install passed while the anonymous API was rate-limited.
- Installer lifecycle suite passed with install, update, rollback, uninstall,
  same-session PATH, unsafe-version rejection, and latest-resolution coverage.
- Full workspace tests, formatting, and Clippy passed.
- GitHub main CI `28797881851` passed on all four native platforms.
- GitHub release workflow `28797886356` passed and published `v3.1.4`.
- The public installer updated the real machine from Baron 3.1.2 to 3.1.4.
- `baron init --codex --fullstack` passed in the real `scanjob` project against
  the existing shared Vault.
