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
    if ([System.IO.Path]::GetFileName($out) -eq "graphify-out") {
        [Console]::Error.WriteLine("--out must be the extraction root, not graphify-out")
        exit 11
    }
    $graphOut = Join-Path $out "graphify-out"
    New-Item -ItemType Directory -Force -Path $graphOut | Out-Null
    $graphPath = Join-Path $graphOut "graph.json"
    if ($mode -eq "malformed") {
        Set-Content -LiteralPath $graphPath -Value '{not-json' -NoNewline
        exit 0
    }
    if ($mode -eq "oversized-graph") {
        Set-Content -LiteralPath $graphPath -Value ("x" * 4096) -NoNewline
        exit 0
    }
    $sourceFile = "src/lib.rs"
    if ($mode -eq "foreign-source") {
        $sourceFile = "../outside.rs"
    }
    $duplicate = if ($mode -eq "duplicate-node") { ',{"id":"entry","label":"duplicate","source_file":"src/lib.rs","_origin":"ast"}' } else { "" }
    $graph = '{"nodes":[{"id":"entry","label":"entry","source_file":"' + $sourceFile + '","_origin":"ast"},{"id":"related","label":"related","source_file":"src/service.rs","_origin":"ast"}' + $duplicate + '],"edges":[{"source":"entry","target":"related","relation":"calls","confidence":"EXTRACTED","source_file":"src/lib.rs","source_location":"L1","weight":1.0}],"input_tokens":0,"output_tokens":0}'
    Set-Content -LiteralPath $graphPath -Value $graph -NoNewline
    Write-Output "extracted"
    exit 0
}

if ($command -eq "query") {
    [Console]::Error.WriteLine("Graphify 0.9.25 query is human-readable traversal, not a Baron JSON contract")
    exit 9
}

[Console]::Error.WriteLine("unexpected command")
exit 10
