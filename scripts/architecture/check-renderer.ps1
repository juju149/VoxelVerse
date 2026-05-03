$ErrorActionPreference = "Stop"

$frame = "src/vv-render/src/renderer/frame.rs"
$publicApi = "src/vv-render/src/renderer/public_api.rs"
$renderCargo = "src/vv-render/Cargo.toml"

$failed = $false

if ((Get-Content $publicApi -Raw) -match "inventory_slot_at|inventory_recipe_at") {
    Write-Host "Renderer public_api still owns inventory hit-testing. Move this to vv-interface/app." -ForegroundColor Red
    $failed = $true
}

if ((Get-Content $renderCargo -Raw) -match "vv-input") {
    Write-Host "vv-render still depends on vv-input. Renderer should consume render frame/camera data." -ForegroundColor Red
    $failed = $true
}

$frameLines = (Get-Content $frame).Count
if ($frameLines -gt 420) {
    Write-Host "renderer/frame.rs is still too large: $frameLines lines. Split render passes." -ForegroundColor Yellow
}

if ($failed) {
    exit 1
}

Write-Host "Renderer guard passed." -ForegroundColor Green