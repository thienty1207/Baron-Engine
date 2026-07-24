$ErrorActionPreference = "Stop"

if ($env:FAKE_GRAPHIFY_LOG) {
    ($args -join " ") | Add-Content -LiteralPath $env:FAKE_GRAPHIFY_LOG
    ("query_log_disable=" + $env:GRAPHIFY_QUERY_LOG_DISABLE) | Add-Content -LiteralPath $env:FAKE_GRAPHIFY_LOG
    ("graphify_api_key_present=" + [bool]$env:GRAPHIFY_API_KEY) | Add-Content -LiteralPath $env:FAKE_GRAPHIFY_LOG
}

$mode = $env:FAKE_GRAPHIFY_MODE
$command = if ($args.Count -gt 0) { $args[0] } else { "" }

if ($command -eq "--version") {
    if ($mode -eq "wrong-version") {
        Write-Output "graphify 0.9.24"
    } else {
        Write-Output "graphify 0.9.25"
    }
    exit 0
}

if ($mode -eq "timeout") {
    Start-Sleep -Seconds 5
}

if ($mode -eq "nonzero") {
    [Console]::Error.WriteLine("fake provider failure")
    exit 7
}

if ($command -eq "extract") {
    if ($args.Count -ne 6 -or $args[2] -ne "--code-only" -or $args[3] -ne "--out" -or $args[5] -ne "--no-cluster") {
        [Console]::Error.WriteLine("unexpected extract command")
        exit 8
    }
    $out = $args[4]
    New-Item -ItemType Directory -Force -Path $out | Out-Null
    if ($mode -eq "oversized-graph") {
        Set-Content -LiteralPath (Join-Path $out "graph.json") -Value ("x" * 4096) -NoNewline
        exit 0
    }
    Set-Content -LiteralPath (Join-Path $out "graph.json") -Value '{"nodes":[],"edges":[]}' -NoNewline
    Write-Output "extracted"
    exit 0
}

if ($command -eq "query") {
    if ($args.Count -ne 7 -or $args[2] -ne "--graph" -or $args[4] -ne "--json" -or $args[5] -ne "--budget") {
        [Console]::Error.WriteLine("unexpected query command")
        exit 9
    }
    if ($mode -eq "oversized") {
        Write-Output ("x" * 4096)
        exit 0
    }
    if ($mode -eq "malformed") {
        Write-Output "{not-json"
        exit 0
    }
    Write-Output '[{"node_id":"entry","label":"entry","source_file":"src/lib.rs","relation":"calls","confidence":"EXTRACTED","explanation":"entry calls service","score":0.9},{"node_id":"related","label":"related","source_file":"src/service.rs","relation":"depends_on","confidence":"INFERRED","explanation":"provider inferred relation","score":0.4}]'
    exit 0
}

[Console]::Error.WriteLine("unexpected command")
exit 10
