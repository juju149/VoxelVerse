param(
    [string]$PackPath = "assets/packs/core"
)

$ErrorActionPreference = "Stop"

cargo run -p vv-pack-doctor -- $PackPath
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

cargo test -p vv-pack-loader -p vv-pack-compiler
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

cargo test -p vv-worldgen
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "Pack V1 check passed."
