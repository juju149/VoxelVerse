param(
    [string]$ManifestPath = "Cargo.toml"
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
& (Join-Path $root "scripts/check_no_forbidden_deps.ps1") -ManifestPath $ManifestPath
