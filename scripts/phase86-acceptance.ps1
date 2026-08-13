<#
.SYNOPSIS
  Run the Baron 4.1 Phase 86 acceptance preflight without inventing a Tencent score.

.DESCRIPTION
  This runner is deliberately an evidence collector, not a score fixer. It
  reuses the frozen Baron contract, runs the release binary repeatedly, checks
  that every report is bound to the same contract/source revision, and requires
  an explicit reviewed Tencent v2.0.0 baseline plus independent confidence
  evidence before it can return `passed`.

  Tencent's public repository does not expose the five surface scores needed by
  Baron. Pass -TencentBaselinePath and -ConfidenceEvidencePath only when those
  artifacts were produced by a separately reviewed, same-corpus runner.
#>

[CmdletBinding()]
param(
    [string]$RepoPath = (Get-Location).Path,
    [string]$BaronBinary,
    [string]$ContractPath,
    [string]$TencentBaselinePath,
    [string]$ConfidenceEvidencePath,
    [string]$VaultRoot,
    [string]$OutputPath,
    [switch]$SeedDevelopmentFixture,
    [ValidateRange(3, 20)]
    [int]$Runs = 3
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = (Resolve-Path -LiteralPath $RepoPath).Path
if (-not $ContractPath) { $ContractPath = Join-Path $repo 'docs/assessment/baron-4.1-contract.json' }
if (-not $OutputPath) { $OutputPath = Join-Path $repo 'docs/assessment/baron-4.1-phase86-runner.json' }
$markdownPath = [System.IO.Path]::ChangeExtension($OutputPath, '.md')

if (-not $BaronBinary) {
    $release = Join-Path $repo 'target/release/baron.exe'
    $debug = Join-Path $repo 'target/debug/baron.exe'
    if (Test-Path -LiteralPath $release) { $BaronBinary = $release }
    elseif (Test-Path -LiteralPath $debug) { $BaronBinary = $debug }
    else { throw 'No Baron binary found. Build target/release/baron.exe first.' }
}
$BaronBinary = (Resolve-Path -LiteralPath $BaronBinary).Path
$contractFile = (Resolve-Path -LiteralPath $ContractPath).Path
$contract = Get-Content -LiteralPath $contractFile -Raw | ConvertFrom-Json

if (-not $VaultRoot) {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $VaultRoot = Join-Path $repo (Join-Path '.tmp' ("phase86-run-$stamp"))
}
if ($SeedDevelopmentFixture -and (Test-Path -LiteralPath $VaultRoot)) {
    $existingEntries = @(Get-ChildItem -LiteralPath $VaultRoot -Force -Recurse -ErrorAction SilentlyContinue)
    if ($existingEntries.Count -gt 0) {
        throw "Refusing to seed a non-empty Vault path: $VaultRoot. Use a new disposable path."
    }
}
New-Item -ItemType Directory -Force -Path $VaultRoot | Out-Null

function Seed-DevelopmentFixture {
    # Create the normal Vault capsule first; this does not touch the repository
    # and keeps the fixture outside the user's configured Vault.
    & $BaronBinary memory index $repo --vault $VaultRoot | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not initialize the isolated Phase 86 Vault fixture.' }
    $capsules = @(Get-ChildItem -LiteralPath (Join-Path $VaultRoot 'Projects') -Directory)
    if ($capsules.Count -ne 1) { throw "Expected one current-project capsule, found $($capsules.Count)." }
    $capsule = $capsules[0].FullName
    New-Item -ItemType Directory -Force -Path (Join-Path $capsule 'Sessions/Imported') | Out-Null
    @'
# Verified Memory

- The current Baron project is building a local memory engine.
- The current checkpoint is the Phase 86 acceptance runner and proof state.
- The confirmed next safe action is to inspect the recorded evidence before changing source.
- Tìm kiếm ngữ nghĩa / semantic retrieval uses bounded evidence and citations.
'@ | Set-Content -LiteralPath (Join-Path $capsule 'Facts.md') -Encoding UTF8
    @'
# Confirmed Decisions

- Vault Markdown remains durable truth; indexes are disposable accelerators.
- The acceptance gate must preserve project isolation and label unknowns.
'@ | Set-Content -LiteralPath (Join-Path $capsule 'Decisions.md') -Encoding UTF8
    @'
### User

We decided the current work is the Phase 86 acceptance runner and the next safe action is to inspect proof.

### Assistant

The proof passed for the bounded local path; session candidates stay evidence-linked and no Skill is created.
'@ | Set-Content -LiteralPath (Join-Path $capsule 'Sessions/Imported/phase86-session.md') -Encoding UTF8

    # A second capsule carries the same words under a different project ID. A
    # correct firewall must count it as blocked, never return it as current truth.
    $other = Join-Path $VaultRoot 'Projects/phase86-other-project--000000000000'
    New-Item -ItemType Directory -Force -Path $other | Out-Null
    @'
{"schemaVersion":2,"projectId":"phase86-other-project","projectSlug":"phase86-other-project"}
'@ | Set-Content -LiteralPath (Join-Path $other '.baron-project.json') -Encoding ASCII
    @'
# Other Project

- The current Baron project is building a local memory engine.
'@ | Set-Content -LiteralPath (Join-Path $other 'Facts.md') -Encoding UTF8
}

if ($SeedDevelopmentFixture) { Seed-DevelopmentFixture }

function Get-FileSha256([string]$Path) {
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Invoke-BaronBenchmark([int]$RunNumber) {
    $raw = & $BaronBinary intelligence benchmark41 $repo --vault $VaultRoot --json 2>&1
    $exitCode = $LASTEXITCODE
    $text = ($raw -join "`n").Trim()
    if ($exitCode -ne 0) {
        return [pscustomobject]@{
            run = $RunNumber
            ok = $false
            exit_code = $exitCode
            error = $text
        }
    }
    try {
        $report = $text | ConvertFrom-Json
    } catch {
        return [pscustomobject]@{
            run = $RunNumber
            ok = $false
            exit_code = $exitCode
            error = "Benchmark output was not JSON: $text"
        }
    }
    return [pscustomobject]@{
        run = $RunNumber
        ok = $true
        exit_code = $exitCode
        report_id = $report.report_id
        contract_id = $report.contract.contract_id
        source_revision = $report.source_revision
        target_achieved = $report.target_achieved
        same_corpus_win = $report.same_corpus_win
        statistical_confidence_95 = $report.statistical_confidence_95
        repetitions = $report.repetitions
        baron_scores = $report.baron_scores
        tencent = $report.tencent
        metrics = $report.metrics
        hard_failures = $report.hard_failures
    }
}

$oldTencent = $env:BARON_TENCENT_BASELINE_JSON
$oldConfidence = $env:BARON_41_CONFIDENCE_EVIDENCE_JSON
$runResults = @()
try {
    if ($TencentBaselinePath) {
        $env:BARON_TENCENT_BASELINE_JSON = (Resolve-Path -LiteralPath $TencentBaselinePath).Path
    }
    if ($ConfidenceEvidencePath) {
        $env:BARON_41_CONFIDENCE_EVIDENCE_JSON = (Resolve-Path -LiteralPath $ConfidenceEvidencePath).Path
    }
    for ($i = 1; $i -le $Runs; $i++) {
        $runResults += Invoke-BaronBenchmark $i
    }
} finally {
    if ($null -eq $oldTencent) { Remove-Item Env:BARON_TENCENT_BASELINE_JSON -ErrorAction SilentlyContinue }
    else { $env:BARON_TENCENT_BASELINE_JSON = $oldTencent }
    if ($null -eq $oldConfidence) { Remove-Item Env:BARON_41_CONFIDENCE_EVIDENCE_JSON -ErrorAction SilentlyContinue }
    else { $env:BARON_41_CONFIDENCE_EVIDENCE_JSON = $oldConfidence }
}

$failures = [System.Collections.Generic.List[string]]::new()
if ($runResults.Count -ne $Runs) { $failures.Add("Expected $Runs runs but collected $($runResults.Count)") }
$successful = @($runResults | Where-Object { $_.ok })
if ($successful.Count -ne $Runs) { $failures.Add('One or more Baron benchmark runs failed') }
foreach ($result in $successful) {
    if ($result.contract_id -ne $contract.contract_id) {
        $failures.Add("Run $($result.run) contract mismatch: $($result.contract_id)")
    }
    if ($result.source_revision -ne $contract.source_revision) {
        $failures.Add("Run $($result.run) source revision mismatch: $($result.source_revision)")
    }
    foreach ($surface in @('long_term_memory_l0_l3','semantic_retrieval_grounded_synthesis','automatic_session_learning','wiki','codegraph')) {
        $score = @($result.baron_scores | Where-Object { $_.surface -eq $surface })
        if ($score.Count -ne 1 -or [int]$score[0].score -lt 95) {
            $value = if ($score.Count -eq 1) { $score[0].score } else { 'missing' }
            $failures.Add("Run $($result.run) $surface score is $value; required 95")
        }
    }
}

$latest = if ($successful.Count -gt 0) { $successful[-1] } else { $null }
$externalBaselineAvailable = $false
$confidenceAvailable = $false
if ($latest) {
    $externalBaselineAvailable = $latest.tencent.status -eq 'available' -and $latest.tencent.same_corpus -and $latest.tencent.confidence_95
    $confidenceAvailable = [bool]$latest.statistical_confidence_95 -and [int]$latest.repetitions -ge 3
}
if (-not $externalBaselineAvailable) { $failures.Add('Reviewed same-corpus Tencent v2.0.0 baseline is unavailable or rejected') }
if (-not $confidenceAvailable) { $failures.Add('Independent 95% confidence evidence with at least three repetitions is unavailable or rejected') }

$status = if ($failures.Count -eq 0) { 'passed' }
          elseif (-not $externalBaselineAvailable -or -not $confidenceAvailable) { 'blocked_external_evidence' }
          else { 'failed' }

$artifact = [ordered]@{
    schema_version = 1
    generated_at = (Get-Date).ToUniversalTime().ToString('o')
    status = $status
    repo = $repo
    baron_binary = $BaronBinary
    baron_binary_sha256 = Get-FileSha256 $BaronBinary
    contract_path = $contractFile
    contract_sha256 = Get-FileSha256 $contractFile
    contract_id = $contract.contract_id
    source_revision = $contract.source_revision
    fixture_revision = $contract.fixture_revision
    holdout_hash = $contract.holdout_hash
    seed_development_fixture = [bool]$SeedDevelopmentFixture
    vault_root = $VaultRoot
    requested_runs = $Runs
    collected_runs = $runResults.Count
    external_tencent_baseline_available = $externalBaselineAvailable
    independent_confidence_available = $confidenceAvailable
    failures = @($failures)
    runs = @($runResults)
    next_action = if ($status -eq 'passed') { 'Owner review may authorize Phase 87 release preparation.' } else { 'Provide a reviewed same-corpus Tencent runner/baseline and independent repeated confidence artifact, then rerun this script.' }
}
$artifact | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

$md = @(
    '# Baron 4.1 Phase 86 Acceptance Runner',
    '',
    ('- Status: **{0}**' -f $status),
    ('- Contract: `{0}`' -f $contract.contract_id),
    ('- Contract SHA-256: `{0}`' -f (Get-FileSha256 $contractFile)),
    ('- Source revision: `{0}`' -f $contract.source_revision),
    ('- Runs: {0}/{1}' -f $runResults.Count, $Runs),
    ('- Tencent same-corpus baseline accepted: `{0}`' -f $externalBaselineAvailable),
    ('- Independent 95% confidence accepted: `{0}`' -f $confidenceAvailable),
    ''
)
$md += '## Result'
$md += ''
if ($failures.Count -eq 0) { $md += '- All Phase 86 gates passed.' }
else { foreach ($failure in $failures) { $md += "- **Open gate:** $failure" } }
$md += ''
$md += 'The runner never converts Tencent public marketing/PersonaMem numbers into a five-surface baseline. A missing or rejected external artifact keeps the result blocked and preserves `v4.0.0` as stable.'
$md -join "`n" | Set-Content -LiteralPath $markdownPath -Encoding UTF8

Write-Output ($artifact | ConvertTo-Json -Depth 20)
if ($status -ne 'passed') { exit 2 }
