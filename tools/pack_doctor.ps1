param(
    [string]$PackRoot = "assets/packs/core"
)

$ErrorActionPreference = "Stop"

$reportsDir = Join-Path $PackRoot "generated/reports"
if (-not (Test-Path -LiteralPath $reportsDir)) {
    New-Item -ItemType Directory -Path $reportsDir | Out-Null
}

$jsonPath = Join-Path $reportsDir "core_pack_report.json"
$htmlPath = Join-Path $reportsDir "core_pack_report.html"

cargo run --quiet -p vv-pack-doctor -- $PackRoot --json $jsonPath --html $htmlPath
