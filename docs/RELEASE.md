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
adds that directory to the user PATH. Open a new terminal, then run:

```powershell
baron --version
```

## Install On Linux Or macOS

```bash
curl -fsSL https://github.com/thienty1207/Baron-Engine/releases/latest/download/install.sh | sh
```

The default location is `~/.local/bin/baron`. If that directory is not already
on PATH, the installer prints the exact directory that needs to be added.

## Update

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

Update verifies the new archive, checks the downloaded binary version, saves
the current executable as a rollback copy, and only then replaces it.

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
Get-FileHash .\baron-v2.0.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

Compare that value with the matching line in `SHA256SUMS`.

## Offline Or Private Mirror Install

Download one native archive and `SHA256SUMS` into the same directory.

Windows:

```powershell
& .\install.ps1 -Version 2.0.0 -SourceDirectory D:\baron-release
```

Linux or macOS:

```bash
sh ./install.sh --version 2.0.0 --source-dir /path/to/baron-release
```

`BARON_RELEASE_BASE_URL` may point installers at a trusted GitHub-compatible
release mirror.

## Maintainer Release Contract

The Git tag must equal `v<workspace-version>`. A tag mismatch fails before
packaging. Native runners build and smoke each target. The release job then
assembles all four archives and runs:

```bash
baron release metadata release-assets --release-version 2.0.0 --source-revision <git-sha>
baron release verify release-assets
```

These maintainer commands are hidden from normal help because users do not need
them during project work.

Before publishing a `v2.0.0` release, also run:

```bash
baron certify run <repo-path> --vault <vault-path> --profile release
```

The certification report must pass before the release notes claim Baron is
healthy at scale.
