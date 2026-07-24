# Baron Install And Release Guide

Baron ships as one native executable. It does not require Node.js, Python,
SQLite, Cargo, or a running server on the user's machine.

## Supported Release Binaries

- Windows x64
- Linux x64
- macOS Intel
- macOS Apple Silicon

Every GitHub Release contains the native archives, `SHA256SUMS`,
`release-manifest.json`, `install.ps1`, and `install.sh`.

## Install On Windows

Open PowerShell:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
& $installer
```

The default location is `%USERPROFILE%\.baron\bin\baron.exe`. The installer
adds that directory to the user PATH and refreshes PATH for the current
PowerShell session, so the next line works in the same copy-paste block:

```powershell
baron --version
```

## Install On Linux Or macOS

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh | sh
```

The default location is `~/.local/bin/baron`. If that directory is not already
on PATH, the installer prints the exact directory that needs to be added.

## Update A Project And Baron

From Baron 3.4 onward, stand in an initialized project and run:

```powershell
baron update
```

This is the normal update command. Baron downloads a release candidate from
the official release source, verifies its identity before use, refreshes only
the Baron-managed files for the project, and activates the runtime through a
recoverable transaction. Custom skills, custom agents, source code, plans,
and Vault Markdown are outside the update write set. If Baron finds an
ambiguous edit to a managed file, it leaves the live project and runtime
unchanged and reports the review location.

AI agents never run this public command. They may only repair already-installed
Baron-managed files locally through Baron’s internal automation contract; they
cannot download a release or replace the runtime.

On Windows, a verified runtime replacement can finish after the current Baron
process exits. Open a new terminal before checking the new `baron --version`.

## One-Time Upgrade From Baron 3.3 Or Older

The installer remains the safe way to cross into Baron 3.4 for the first time.
Run the matching command once, then use `baron update` for normal project and
runtime updates.

Windows:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
& $installer -Action update
```

Linux or macOS:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh |
  sh -s -- --action update
```

The installer verifies the new archive, checks the downloaded binary version,
saves the current executable as a rollback copy, and only then replaces it.
It also remains useful for a recovery install when a Baron executable is no
longer runnable.

## Roll Back

Windows:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
& $installer -Action rollback
```

Linux or macOS:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh |
  sh -s -- --action rollback
```

Rollback restores the newest installer-owned backup. It does not roll back
project files, adapters, plans, or Vault memory.

## Uninstall

Windows:

```powershell
$installer = Join-Path $env:TEMP "baron-install.ps1"
Invoke-WebRequest https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.ps1 -OutFile $installer
& $installer -Action uninstall
```

Linux or macOS:

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh |
  sh -s -- --action uninstall
```

Uninstall removes the Baron executable and install metadata only. These remain:

- every project repository
- every `.baron/` project configuration
- every generated agent adapter
- every Vault Markdown file
- every memory, plan, proof, trace, and story

## Checksum Safety

Both installers download the matching archive and `SHA256SUMS` into a temporary
directory. Baron is not extracted or installed unless SHA-256 verification
passes. The staged binary must also report the requested version before the
active binary is replaced.

For manual verification:

```bash
sha256sum -c SHA256SUMS
```

On Windows:

```powershell
Get-FileHash .\baron-v3.5.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

Compare that value with the matching line in `SHA256SUMS`.

## Offline Or Private Mirror Install

Download one native archive and `SHA256SUMS` into the same directory.

Windows:

```powershell
& .\install.ps1 -Version 3.5.0 -SourceDirectory D:\baron-release
```

Linux or macOS:

```bash
sh ./install.sh --version 3.5.0 --source-dir /path/to/baron-release
```

`BARON_RELEASE_BASE_URL` may point installers at a trusted GitHub-compatible
release mirror.

## Maintainer Release Contract

A release starts from an exact 40-character commit SHA already pushed as the
current `origin/main`. No release tag exists yet. The workflow checks that the
requested version matches Cargo, runs formatting, the full workspace tests and
Clippy, then builds and smokes every native target. The final promotion job
assembles all four archives and runs:

```bash
baron release metadata release-assets --release-version 3.5.0 --source-revision <40-character-git-sha>
baron release verify release-assets --expected-version 3.5.0 --expected-source-revision <40-character-git-sha>
```

These maintainer commands are hidden from normal help because users do not need
them during project work.

Before promoting a `v3.5.0` release, also run:

```bash
baron certify run <repo-path> --vault <vault-path> --profile release
```

The certification report must pass before the release notes claim Baron is
healthy at scale.

## Publishing `releases/latest`

`releases/latest` is controlled by GitHub Releases, not by `Cargo.toml` alone.
Push the verified source commit to `main`, copy its full SHA, and dispatch the
release workflow:

```bash
git push origin main
git rev-parse HEAD
gh workflow run release.yml -f release_version=3.5.0 -f source_revision=<40-character-git-sha>
```

The `Baron Release` workflow refuses an existing tag or Release, builds the
native archives from that exact SHA, verifies checksums and installer lifecycle,
and only then creates the annotated tag and immutable GitHub Release. Only the
final promotion job has repository write permission. When the workflow
finishes, `https://github.com/thienty1207/Baron-Engine/releases/latest` should
point at `v3.5.0`.

Public smoke after the workflow:

```bash
baron --version
baron setup --vault
baron init --codex --fullstack
```
