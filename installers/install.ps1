[CmdletBinding()]
param(
    [ValidateSet("install", "update", "rollback", "uninstall")]
    [string]$Action = "install",
    [string]$Version = "latest",
    [string]$InstallDir = (Join-Path $HOME ".baron\bin"),
    [string]$BaseUrl = "https://github.com/thienty1207/Baron-Engine/releases/download",
    [string]$LatestManifestUrl = "https://github.com/thienty1207/Baron-Engine/releases/latest/download/release-manifest.json",
    [string]$SourceDirectory,
    [string]$StateDirectory,
    [switch]$NoPathUpdate
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$TrustedReleaseKeyId = "baron-release-2026"
$TrustedReleasePublicKeyBase64 = "SBgrmtK5emhgD5e6UW664fXR3qogCjVPHtrxeVk2TcA="

if ($env:BARON_RELEASE_BASE_URL) {
    $BaseUrl = $env:BARON_RELEASE_BASE_URL
}
if ($env:BARON_RELEASE_LATEST_MANIFEST_URL) {
    $LatestManifestUrl = $env:BARON_RELEASE_LATEST_MANIFEST_URL
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

function Assert-HttpsUri([string]$UriValue) {
    $parsed = [Uri]$UriValue
    if ($parsed.Scheme -ne "https") {
        throw "Refusing insecure non-HTTPS Baron release URL: $UriValue"
    }
}

function Convert-HexToBytes([string]$HexValue) {
    if (-not $HexValue -or ($HexValue.Length % 2) -ne 0 -or $HexValue -notmatch "^[0-9a-fA-F]+$") {
        throw "Release signature is not valid hexadecimal data."
    }
    $bytes = New-Object byte[] ($HexValue.Length / 2)
    for ($index = 0; $index -lt $bytes.Length; $index++) {
        $bytes[$index] = [Convert]::ToByte($HexValue.Substring($index * 2, 2), 16)
    }
    return $bytes
}

function Write-TrustedPublicKeyPem([string]$Path) {
    $raw = [Convert]::FromBase64String($TrustedReleasePublicKeyBase64)
    if ($raw.Length -ne 32) {
        throw "Pinned Baron release public key must decode to 32 bytes."
    }
    $prefix = [byte[]](Convert-HexToBytes "302a300506032b6570032100")
    $der = New-Object byte[] ($prefix.Length + $raw.Length)
    [Buffer]::BlockCopy($prefix, 0, $der, 0, $prefix.Length)
    [Buffer]::BlockCopy($raw, 0, $der, $prefix.Length, $raw.Length)
    $body = [Convert]::ToBase64String($der)
    $pem = "-----BEGIN PUBLIC KEY-----`n$body`n-----END PUBLIC KEY-----`n"
    Set-Content -LiteralPath $Path -Value $pem -Encoding ascii -NoNewline
}

function Get-OpenSslPath {
    $command = Get-Command openssl -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }
    $candidates = @(
        (Join-Path ${env:ProgramFiles} "Git\usr\bin\openssl.exe"),
        (Join-Path ${env:ProgramFiles} "Git\mingw64\bin\openssl.exe"),
        (Join-Path ${env:ProgramFiles(x86)} "Git\usr\bin\openssl.exe")
    )
    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path -LiteralPath $candidate -PathType Leaf)) {
            return $candidate
        }
    }
    throw "OpenSSL with Ed25519 support is required to verify Baron metadata; refusing installation."
}

function Get-Sha256([string]$Path) {
    $stream = [System.IO.File]::OpenRead($Path)
    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = $sha256.ComputeHash($stream)
        return ([System.BitConverter]::ToString($bytes)).Replace("-", "").ToLowerInvariant()
    } finally {
        $sha256.Dispose()
        $stream.Dispose()
    }
}

function Verify-ReleaseManifest(
    [string]$ManifestPath,
    [string]$SignaturePath,
    [string]$Target,
    [string]$RequestedVersion,
    [string]$TemporaryRoot
) {
    $signature = Get-Content -LiteralPath $SignaturePath -Raw | ConvertFrom-Json
    $signatureNames = @($signature.PSObject.Properties.Name | Sort-Object)
    if (($signatureNames -join ",") -ne "key_id,schema_version,signature") {
        throw "Release signature metadata fields are invalid."
    }
    if ([int]$signature.schema_version -ne 1) {
        throw "Unsupported release signature schema."
    }
    if ([string]$signature.key_id -ne $TrustedReleaseKeyId) {
        throw "Release signature key ID is not Baron's trusted production key."
    }
    $signatureBytes = Convert-HexToBytes ([string]$signature.signature)
    if ($signatureBytes.Length -ne 64) {
        throw "Release signature must be exactly 64 bytes."
    }
    $signatureBinaryPath = Join-Path $TemporaryRoot "release-manifest.signature"
    [System.IO.File]::WriteAllBytes($signatureBinaryPath, $signatureBytes)

    $manifestBytes = [System.IO.File]::ReadAllBytes($ManifestPath)
    $prefix = [Text.Encoding]::UTF8.GetBytes("baron-release-manifest-v1")
    $message = New-Object byte[] ($prefix.Length + 1 + $manifestBytes.Length)
    [Buffer]::BlockCopy($prefix, 0, $message, 0, $prefix.Length)
    $message[$prefix.Length] = 0
    [Buffer]::BlockCopy($manifestBytes, 0, $message, $prefix.Length + 1, $manifestBytes.Length)
    $messagePath = Join-Path $TemporaryRoot "release-manifest.message"
    [System.IO.File]::WriteAllBytes($messagePath, $message)

    $publicKeyPath = Join-Path $TemporaryRoot "release-public-key.pem"
    Write-TrustedPublicKeyPem $publicKeyPath
    $openssl = Get-OpenSslPath
    $opensslArgs = @(
        "pkeyutl", "-verify", "-pubin", "-inkey", $publicKeyPath,
        "-rawin", "-in", $messagePath, "-sigfile", $signatureBinaryPath
    )
    $previousPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & $openssl @opensslArgs *> $null
    $opensslExit = $LASTEXITCODE
    $ErrorActionPreference = $previousPreference
    if ($opensslExit -ne 0) {
        throw "Baron release metadata signature verification failed; no files were changed."
    }

    $manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
    $manifestNames = @($manifest.PSObject.Properties.Name | Sort-Object)
    if (($manifestNames -join ",") -ne "artifacts,minimum_compatible_version,product,release_identity,schema_version,source_revision,update_candidates,version") {
        throw "Release manifest fields are invalid."
    }
    if ([int]$manifest.schema_version -notin @(1, 2) -or [string]$manifest.product -ne "Baron Engine") {
        throw "Release manifest identity is invalid."
    }
    if ([string]$manifest.version -notmatch "^\d+\.\d+\.\d+$") {
        throw "Release manifest version is invalid."
    }
    if ($RequestedVersion -ne "latest" -and [string]$manifest.version -ne $RequestedVersion) {
        throw "Release manifest version does not match the requested version."
    }
    if ([string]$manifest.release_identity -ne "github:thienty1207/Baron-Engine") {
        throw "Release manifest source identity is invalid."
    }
    if ([string]$manifest.minimum_compatible_version -notmatch "^\d+\.\d+\.\d+$") {
        throw "Release manifest compatibility version is invalid."
    }
    $artifacts = @($manifest.artifacts)
    $expectedArchiveName = "baron-v$($manifest.version)-$Target.zip"
    $matches = @($artifacts | Where-Object {
        [string]$_.target -eq $Target -and [string]$_.name -eq $ExpectedArchiveName
    })
    if ($matches.Count -ne 1 -or [string]$matches[0].binary -ne "baron.exe") {
        throw "Release manifest does not contain the expected platform artifact."
    }
    $artifact = $matches[0]
    if ([string]$artifact.sha256 -notmatch "^[0-9a-fA-F]{64}$") {
        throw "Release artifact digest is invalid."
    }
    $size = [Int64]$artifact.size_bytes
    if ($size -lt 0) {
        throw "Release artifact size is invalid."
    }
    return [pscustomobject]@{
        Version = [string]$manifest.version
        Sha256 = ([string]$artifact.sha256).ToLowerInvariant()
        SizeBytes = $size
    }
}

function Update-PathValue([string]$PathValue, [bool]$Add) {
    $parts = @($PathValue -split ";" | Where-Object { $_ -and $_ -ne $InstallDir })
    if ($Add) {
        return (@($InstallDir) + $parts) -join ";"
    }
    return $parts -join ";"
}

function Update-UserPath([bool]$Add) {
    if ($NoPathUpdate) {
        return
    }
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    [Environment]::SetEnvironmentVariable("Path", (Update-PathValue $userPath $Add), "User")
    $processPath = [Environment]::GetEnvironmentVariable("Path", "Process")
    $env:Path = Update-PathValue $processPath $Add
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
$target = "x86_64-pc-windows-msvc"
if ($Version -ne "latest" -and $Version -notmatch "^\d+\.\d+\.\d+$") {
    throw "Baron version must use numeric major.minor.patch form."
}

$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("baron-install-" + [Guid]::NewGuid())
$manifestPath = Join-Path $temporaryRoot "release-manifest.json"
$signaturePath = Join-Path $temporaryRoot "release-manifest.sig"
$checksumsPath = Join-Path $temporaryRoot "SHA256SUMS"
$extractPath = Join-Path $temporaryRoot "extract"
$archiveName = $null

try {
    New-Item -ItemType Directory -Force -Path $temporaryRoot, $extractPath | Out-Null
    if ($Version -eq "latest") {
        if ($SourceDirectory) {
            throw "Offline installation requires an explicit -Version."
        }
        Assert-HttpsUri $LatestManifestUrl
        Invoke-WebRequest -UseBasicParsing -Uri $LatestManifestUrl -OutFile $manifestPath
        $latestSignatureUrl = $LatestManifestUrl -replace "\.json$", ".sig"
        Assert-HttpsUri $latestSignatureUrl
        Invoke-WebRequest -UseBasicParsing -Uri $latestSignatureUrl -OutFile $signaturePath
    } elseif ($SourceDirectory) {
        Copy-Item -LiteralPath (Join-Path $SourceDirectory "release-manifest.json") -Destination $manifestPath
        Copy-Item -LiteralPath (Join-Path $SourceDirectory "release-manifest.sig") -Destination $signaturePath
    } else {
        $releaseBase = "$($BaseUrl.TrimEnd('/'))/v$Version"
        Assert-HttpsUri "$releaseBase/release-manifest.json"
        Assert-HttpsUri "$releaseBase/release-manifest.sig"
        Invoke-WebRequest -UseBasicParsing -Uri "$releaseBase/release-manifest.json" -OutFile $manifestPath
        Invoke-WebRequest -UseBasicParsing -Uri "$releaseBase/release-manifest.sig" -OutFile $signaturePath
    }

    $requestedVersion = $Version
    $verification = Verify-ReleaseManifest $manifestPath $signaturePath $target $requestedVersion $temporaryRoot
    $Version = $verification.Version
    $archiveName = "baron-v$Version-$target.zip"

    if ($SourceDirectory) {
        Copy-Item -LiteralPath (Join-Path $SourceDirectory $archiveName) -Destination (Join-Path $temporaryRoot $archiveName)
        Copy-Item -LiteralPath (Join-Path $SourceDirectory "SHA256SUMS") -Destination $checksumsPath
    } else {
        $releaseBase = "$($BaseUrl.TrimEnd('/'))/v$Version"
        Assert-HttpsUri "$releaseBase/$archiveName"
        Assert-HttpsUri "$releaseBase/SHA256SUMS"
        Invoke-WebRequest -UseBasicParsing -Uri "$releaseBase/$archiveName" -OutFile (Join-Path $temporaryRoot $archiveName)
        Invoke-WebRequest -UseBasicParsing -Uri "$releaseBase/SHA256SUMS" -OutFile $checksumsPath
    }

    $archivePath = Join-Path $temporaryRoot $archiveName
    $checksumLine = Get-Content -LiteralPath $checksumsPath |
        Where-Object { $_ -match ("  " + [Regex]::Escape($archiveName) + "$") } |
        Select-Object -First 1
    if (-not $checksumLine) {
        throw "SHA256SUMS does not contain $archiveName."
    }
    $expectedChecksumFromFile = ($checksumLine -split "\s+")[0].ToLowerInvariant()
    if ($expectedChecksumFromFile -ne $verification.Sha256) {
        throw "Unsigned SHA256SUMS does not match authenticated Baron metadata."
    }
    $actualChecksum = Get-Sha256 $archivePath
    $actualSize = (Get-Item -LiteralPath $archivePath).Length
    if ([Int64]$actualSize -ne [Int64]$verification.SizeBytes) {
        throw "Baron artifact size verification failed for $archiveName."
    }
    if ($actualChecksum -ne $verification.Sha256) {
        throw "Baron artifact checksum verification failed for $archiveName."
    }

    Expand-Archive -LiteralPath $archivePath -DestinationPath $extractPath -Force
    $stagedBinary = Join-Path $extractPath "baron.exe"
    if (-not (Test-Path -LiteralPath $stagedBinary -PathType Leaf)) {
        throw "The Baron archive does not contain baron.exe."
    }
    $reportedVersion = (& $stagedBinary --version | Out-String).Trim()
    if ($reportedVersion -ne "baron $Version") {
        throw "Downloaded Baron binary reported an unexpected version: $reportedVersion"
    }

    # Authentication, platform selection, size, and digest verification have
    # all completed before this first mutation of the existing installation.
    New-Item -ItemType Directory -Force -Path $InstallDir, $backupDir | Out-Null
    $backupPath = $null
    if (Test-Path -LiteralPath $binaryPath -PathType Leaf) {
        $backupPath = Join-Path $backupDir ("baron-{0}-{1}.exe" -f [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds(), $Version)
        Move-Item -LiteralPath $binaryPath -Destination $backupPath -Force
    }
    try {
        Move-Item -LiteralPath $stagedBinary -Destination $binaryPath -Force
    } catch {
        if ($backupPath -and (Test-Path -LiteralPath $backupPath -PathType Leaf)) {
            Move-Item -LiteralPath $backupPath -Destination $binaryPath -Force
        }
        throw
    }

    New-Item -ItemType Directory -Force -Path $stateRoot | Out-Null
    @{
        version = $Version
        installed_at = [DateTimeOffset]::UtcNow.ToString("o")
        binary = $binaryPath
        checksum = $actualChecksum
    } | ConvertTo-Json | Set-Content -LiteralPath $metadataPath -Encoding UTF8
    Update-UserPath $true
    Write-Host "Baron $Version $Action completed at $binaryPath."
    if (-not $NoPathUpdate) {
        Write-Host "You can run baron --version in this terminal now."
    }
} finally {
    if (Test-Path -LiteralPath $temporaryRoot) {
        Remove-Item -LiteralPath $temporaryRoot -Recurse -Force
    }
}
