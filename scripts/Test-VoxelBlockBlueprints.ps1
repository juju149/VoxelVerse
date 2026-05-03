param(
    [string]$GeneratedRoot = "assets\packs\voxelverse_core\defs\blocks\generated\blueprints",
    [switch]$RunCargoChecks
)

$ErrorActionPreference = "Stop"

function Find-VoxelVerseRoot {
    $dir = (Get-Location).Path
    while ($true) {
        if ((Test-Path (Join-Path $dir "Cargo.toml")) -and (Test-Path (Join-Path $dir "assets"))) {
            return (Resolve-Path $dir).Path
        }
        $parent = Split-Path $dir -Parent
        if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $dir) { throw "Repo root not found." }
        $dir = $parent
    }
}

$Root = Find-VoxelVerseRoot
Set-Location $Root

$rootPath = Join-Path $Root $GeneratedRoot
if (!(Test-Path $rootPath)) { throw "Generated blueprint output missing: $rootPath" }

$files = Get-ChildItem -Path $rootPath -Filter "*.ron" -Recurse
if ($files.Count -lt 12) { throw "Expected at least 12 generated blueprint blocks, found $($files.Count)." }

$seenDisplayKeys = @{}
$seenNames = @{}

foreach ($file in $files) {
    $text = Get-Content -Path $file.FullName -Raw

    foreach ($marker in @("render:", "material:", "shape:", "program:", "meshing:", "physics:", "drops:")) {
        if (!$text.Contains($marker)) { throw "Missing marker '$marker' in $($file.FullName)" }
    }

    $displayMatch = [regex]::Match($text, 'display_key:\s*Some\("([^"]+)"\)')
    if (!$displayMatch.Success) { throw "Missing display_key in $($file.FullName)" }

    $display = $displayMatch.Groups[1].Value
    if ($seenDisplayKeys.ContainsKey($display)) {
        throw "Duplicate display_key '$display' in $($file.FullName) and $($seenDisplayKeys[$display])"
    }
    $seenDisplayKeys[$display] = $file.FullName

    $name = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
    if ($seenNames.ContainsKey($name)) {
        throw "Duplicate generated block filename '$name' in $($file.FullName) and $($seenNames[$name])"
    }
    $seenNames[$name] = $file.FullName
}

$all = ($files | ForEach-Object { Get-Content $_.FullName -Raw }) -join "`n"

foreach ($marker in @(
    'type: "patterned"',
    'pattern: "natural_cells"',
    'pattern: "layered_surface"',
    'pattern: "rings"',
    'details: [',
    'kind: pebble',
    'kind: leaf_lobe',
    'kind: grain',
    'kind: crack'
)) {
    if (!$all.Contains($marker)) { throw "Blueprint audit failed, missing marker: $marker" }
}

Write-Host "Blueprint audit passed: $($files.Count) generated block files." -ForegroundColor Green

if ($RunCargoChecks) {
    cargo fmt
    cargo test -p vv-compiler --test compile_voxelverse_core
    cargo check -p vv-schema
    cargo check -p vv-registry
    cargo check -p vv-compiler
    cargo check -p vv-render
}