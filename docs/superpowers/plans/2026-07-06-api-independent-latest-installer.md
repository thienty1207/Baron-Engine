# Baron 3.1.4 API-Independent Latest Installer

Date: 2026-07-06
Status: source verified; release pending

## Goal

Keep the one-block Baron installer working even when the anonymous GitHub API
quota is exhausted.

## Root Cause

The installer downloaded its own script and release assets from normal GitHub
release URLs, but still called `api.github.com` to resolve the latest version.
That unnecessary API call could fail before any binary was downloaded.

## Plan

1. Add a RED contract test requiring both installers to avoid GitHub API quota.
2. Resolve latest through the published `release-manifest.json` asset.
3. Keep explicit version, checksum, staged binary version, rollback, and mirror
   behavior unchanged.
4. Verify PowerShell and shell installer contracts, full workspace tests,
   hosted release CI, and a real one-block latest install.

## Verification Record

- RED no-API installer contract: failed for the expected missing manifest path.
- GREEN no-API installer contract: passed for PowerShell and shell.
- Manifest-based latest smoke passed while anonymous GitHub API was rate-limited.
- Full installer lifecycle suite passed.
- Full workspace tests, formatting, and Clippy passed.

## Non-Negotiables

- Normal users still copy and paste one install block.
- Latest resolution must not require an authenticated API token.
- Mirror users can override the latest-manifest URL.
- Project files and Vault memory remain outside installer lifecycle changes.
