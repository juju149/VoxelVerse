param(
    [ValidateSet("debug", "release")]
    [string]$BuildProfile = "release",

    [ValidateSet("normal", "high", "verbose")]
    [string]$DiagnosticsProfile = "verbose",

    [string]$LogDir = "logs/diagnostics"
)

$ErrorActionPreference = "Stop"

$resolvedLogDir = Join-Path (Get-Location) $LogDir
New-Item -ItemType Directory -Force -Path $resolvedLogDir | Out-Null

$env:VOXELVERSE_DEV = "1"
$env:VV_DIAGNOSTICS = $DiagnosticsProfile
$env:VV_DIAGNOSTICS_DIR = $resolvedLogDir

$gameLog = Join-Path $resolvedLogDir "game.log"
$cargoArgs = @("run", "-p", "voxelverse")
if ($BuildProfile -eq "release") {
    $cargoArgs += "--release"
}

Write-Host "[diagnostics] profile=$DiagnosticsProfile"
Write-Host "[diagnostics] dir=$resolvedLogDir"
Write-Host "[diagnostics] game log=$gameLog"
Write-Host "[diagnostics] cargo $($cargoArgs -join ' ')"

& cargo @cargoArgs 2>&1 | Tee-Object -FilePath $gameLog
exit $LASTEXITCODE
