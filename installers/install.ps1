[CmdletBinding()]
param(
    [ValidateSet("install", "update", "rollback", "uninstall")]
    [string]$Action = "install",
    [string]$Version = "latest",
    [string]$InstallDir = (Join-Path $HOME ".baron\bin"),
    [string]$BaseUrl = "https://github.com/thienty1207/Baron-Engine/releases/download",
    [string]$SourceDirectory,
    [string]$StateDirectory,
    [switch]$NoPathUpdate
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($env:BARON_RELEASE_BASE_URL) {
    $BaseUrl = $env:BARON_RELEASE_BASE_URL
}

$stateRoot = if ($StateDirectory) {
    $StateDirectory
} elseif ($env:BARON_STATE_DIR) {
    $env:BARON_STATE_DIR
} else {
    Join-Path $HOME ".baron"
}
$backupDir = Join-Path $stateRoot "backups"
$metadataPath = Join-Path $stateRoot "install.json"
$binaryPath = Join-Path $InstallDir "baron.exe"

function Update-UserPath([bool]$Add) {
    if ($NoPathUpdate) {
        return
    }
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @($userPath -split ";" | Where-Object { $_ -and $_ -ne $InstallDir })
    if ($Add) {
        $parts += $InstallDir
    }
    [Environment]::SetEnvironmentVariable("Path", ($parts -join ";"), "User")
}

function Invoke-Rollback {
    New-Item -ItemType Directory -Force -Path $InstallDir, $backupDir | Out-Null
    $backup = Get-ChildItem -LiteralPath $backupDir -Filter "baron-*.exe" -File |
        Sort-Object LastWriteTimeUtc -Descending |
        Select-Object -First 1
    if (-not $backup) {
        throw "No Baron rollback binary is available."
    }
    if (Test-Path -LiteralPath $binaryPath) {
        $current = Join-Path $backupDir ("baron-rollback-current-{0}.exe" -f [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())
        Move-Item -LiteralPath $binaryPath -Destination $current -Force
    }
    Copy-Item -LiteralPath $backup.FullName -Destination $binaryPath -Force
    & $binaryPath --version | Out-Host
    Write-Host "Baron rollback completed."
}

function Invoke-Uninstall {
    if (Test-Path -LiteralPath $binaryPath) {
        Remove-Item -LiteralPath $binaryPath -Force
    }
    if (Test-Path -LiteralPath $metadataPath) {
        Remove-Item -LiteralPath $metadataPath -Force
    }
    Update-UserPath $false
    Write-Host "Baron executable removed. Project files and Vault memory were not touched."
}

if ($Action -eq "rollback") {
    Invoke-Rollback
    exit 0
}
if ($Action -eq "uninstall") {
    Invoke-Uninstall
    exit 0
}

if (-not [Environment]::Is64BitOperatingSystem) {
    throw "Baron currently publishes a 64-bit Windows binary."
}
if ($env:PROCESSOR_ARCHITECTURE -notin @("AMD64", "x86")) {
    throw "Unsupported Windows architecture: $env:PROCESSOR_ARCHITECTURE"
}

if ($Version -eq "latest") {
    if ($SourceDirectory) {
        throw "Offline installation requires an explicit -Version."
    }
    $release = Invoke-RestMethod `
        -Uri "https://api.github.com/repos/thienty1207/Baron-Engine/releases/latest" `
        -Headers @{ "User-Agent" = "Baron-Installer" }
    $Version = [string]$release.tag_name
    $Version = $Version.TrimStart("v")
}
if ($Version -notmatch "^\d+\.\d+\.\d+$") {
    throw "Baron version must use numeric major.minor.patch form."
}

$archiveName = "baron-v$Version-x86_64-pc-windows-msvc.zip"
$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("baron-install-" + [Guid]::NewGuid())
$archivePath = Join-Path $temporaryRoot $archiveName
$checksumsPath = Join-Path $temporaryRoot "SHA256SUMS"
$extractPath = Join-Path $temporaryRoot "extract"

try {
    New-Item -ItemType Directory -Force -Path $temporaryRoot, $extractPath, $InstallDir, $backupDir | Out-Null
    if ($SourceDirectory) {
        Copy-Item -LiteralPath (Join-Path $SourceDirectory $archiveName) -Destination $archivePath
        Copy-Item -LiteralPath (Join-Path $SourceDirectory "SHA256SUMS") -Destination $checksumsPath
    } else {
        $releaseBase = "$($BaseUrl.TrimEnd('/'))/v$Version"
        Invoke-WebRequest -Uri "$releaseBase/$archiveName" -OutFile $archivePath
        Invoke-WebRequest -Uri "$releaseBase/SHA256SUMS" -OutFile $checksumsPath
    }

    $checksumLine = Get-Content -LiteralPath $checksumsPath |
        Where-Object { $_ -match ("  " + [Regex]::Escape($archiveName) + "$") } |
        Select-Object -First 1
    if (-not $checksumLine) {
        throw "SHA256SUMS does not contain $archiveName."
    }
    $expectedChecksum = ($checksumLine -split "\s+")[0].ToLowerInvariant()
    $actualChecksum = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualChecksum -ne $expectedChecksum) {
        throw "Baron checksum verification failed for $archiveName."
    }

    Expand-Archive -LiteralPath $archivePath -DestinationPath $extractPath -Force
    $stagedBinary = Join-Path $extractPath "baron.exe"
    if (-not (Test-Path -LiteralPath $stagedBinary)) {
        throw "The Baron archive does not contain baron.exe."
    }
    $reportedVersion = (& $stagedBinary --version | Out-String).Trim()
    if ($reportedVersion -notmatch [Regex]::Escape($Version)) {
        throw "Downloaded Baron binary reported an unexpected version: $reportedVersion"
    }

    $backupPath = $null
    if (Test-Path -LiteralPath $binaryPath) {
        $backupPath = Join-Path $backupDir ("baron-{0}-{1}.exe" -f [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(), $Version)
        Move-Item -LiteralPath $binaryPath -Destination $backupPath -Force
    }
    try {
        Move-Item -LiteralPath $stagedBinary -Destination $binaryPath -Force
    } catch {
        if ($backupPath -and (Test-Path -LiteralPath $backupPath)) {
            Move-Item -LiteralPath $backupPath -Destination $binaryPath -Force
        }
        throw
    }

    @{
        version = $Version
        installed_at = [DateTimeOffset]::UtcNow.ToString("o")
        binary = $binaryPath
        checksum = $actualChecksum
    } | ConvertTo-Json | Set-Content -LiteralPath $metadataPath -Encoding UTF8
    Update-UserPath $true
    Write-Host "Baron $Version $Action completed at $binaryPath."
    if (-not $NoPathUpdate) {
        Write-Host "Open a new terminal before running baron."
    }
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}
