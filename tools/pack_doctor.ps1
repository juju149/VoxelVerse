#requires -Version 5.1
<#
.SYNOPSIS
    Run the VoxelVerse Pack Doctor against a content pack.

.DESCRIPTION
    This is the canonical entrypoint for content validation. It builds the
    `vv-pack-doctor` crate in release mode (so CI runs are fast on a warm
    cache) and writes both a JSON and an HTML report next to the pack.

    Exit code:
        0 -- no errors, only warnings or clean
        1 -- one or more errors found
        2 -- pipeline itself failed to run (e.g. pack not found)

    Suitable for use both interactively and from CI. No arguments are
    required when the working directory is the workspace root.

.PARAMETER PackRoot
    Path to the pack directory. Defaults to assets/packs/core.

.PARAMETER Json
    Override the JSON report destination.

.PARAMETER Html
    Override the HTML report destination.

.PARAMETER Release
    Build the doctor in release mode (default). Set $false for debug.

.EXAMPLE
    pwsh tools/pack_doctor.ps1
    pwsh tools/pack_doctor.ps1 -PackRoot assets/packs/my_mod
#>
param(
    [string]$PackRoot = "assets/packs/core",
    [string]$Json = "",
    [string]$Html = "",
    [switch]$Debug
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $PackRoot -PathType Container)) {
    Write-Error "Pack root not found: $PackRoot"
    exit 2
}

$reportsDir = Join-Path $PackRoot "generated/reports"
if (-not (Test-Path -LiteralPath $reportsDir)) {
    New-Item -ItemType Directory -Path $reportsDir -Force | Out-Null
}

if ([string]::IsNullOrEmpty($Json)) {
    $Json = Join-Path $reportsDir "pack_doctor_report.json"
}
if ([string]::IsNullOrEmpty($Html)) {
    $Html = Join-Path $reportsDir "pack_doctor_report.html"
}

$cargoArgs = @("run", "--quiet")
if (-not $Debug) {
    $cargoArgs += "--release"
}
$cargoArgs += @("-p", "vv-pack-doctor", "--", $PackRoot, "--json", $Json, "--html", $Html)

& cargo @cargoArgs
$code = $LASTEXITCODE

Write-Host ""
Write-Host "JSON report: $Json"
Write-Host "HTML report: $Html"
exit $code
