param(
    [string]$PackRoot = "assets/packs/core",
    [switch]$RunRustContentTests
)

$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
$warnings = New-Object System.Collections.Generic.List[string]

function Add-Error {
    param([string]$Message)
    $script:errors.Add($Message)
}

function Add-Warning {
    param([string]$Message)
    $script:warnings.Add($Message)
}

function Assert-Dir {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        Add-Error "Missing directory: $Path"
    }
}

function Test-SafeContentName {
    param([string]$Name)
    if ($Name -cmatch "[A-Z]") { return $false }
    if ($Name -match "\s") { return $false }
    if ($Name -match "-") { return $false }
    if ([System.IO.Path]::GetFileNameWithoutExtension($Name) -match "^[0-9]+$") { return $false }
    return $true
}

function Get-RelativePath {
    param([string]$Base, [string]$Path)
    $baseFull = [System.IO.Path]::GetFullPath($Base).TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    $pathFull = [System.IO.Path]::GetFullPath($Path)
    $baseUri = New-Object System.Uri($baseFull)
    $pathUri = New-Object System.Uri($pathFull)
    $relativeUri = $baseUri.MakeRelativeUri($pathUri)
    return [System.Uri]::UnescapeDataString($relativeUri.ToString()).Replace('/', [System.IO.Path]::DirectorySeparatorChar)
}

$packFull = [System.IO.Path]::GetFullPath($PackRoot)
if (-not (Test-Path -LiteralPath $packFull -PathType Container)) {
    throw "Pack root not found: $PackRoot"
}

$requiredDirs = @(
    "blocks",
    "worldgen",
    "textures",
    "generated",
    "defs",
    "media",
    "source",
    "legacy_imports",
    "media/voxel",
    "generated/diagnostics"
)

foreach ($dir in $requiredDirs) {
    Assert-Dir (Join-Path $packFull $dir)
}

$blockDir = Join-Path $packFull "blocks"
$textureDir = Join-Path $packFull "textures"
$legacyDir = Join-Path $packFull "legacy_imports"
$mediaVoxelDir = Join-Path $packFull "media/voxel"

if (Test-Path -LiteralPath $blockDir) {
    $blockFiles = Get-ChildItem -LiteralPath $blockDir -File -Filter *.ron | Sort-Object Name
    if ($blockFiles.Count -eq 0) {
        Add-Error "No block RON files found in active loader path: $blockDir"
    }

    $stems = $blockFiles | ForEach-Object { $_.BaseName }
    $duplicateStems = $stems | Group-Object | Where-Object { $_.Count -gt 1 }
    foreach ($dup in $duplicateStems) {
        Add-Error "Duplicate block id stem in active loader path: $($dup.Name)"
    }

    foreach ($file in $blockFiles) {
        $text = Get-Content -LiteralPath $file.FullName -Raw
        $matches = [regex]::Matches($text, '"([a-z0-9_]+):([a-z0-9_./-]+)"')
        foreach ($match in $matches) {
            $namespace = $match.Groups[1].Value
            $path = $match.Groups[2].Value
            if ($namespace -ne "core") {
                continue
            }
            if ($path -match "(albedo|normal|roughness)$") {
                $texturePath = Join-Path $textureDir ($path + ".png")
                if (-not (Test-Path -LiteralPath $texturePath -PathType Leaf)) {
                    Add-Error "Missing texture referenced by $($file.FullName): core:$path -> $texturePath"
                }
            }
        }
    }
}

if (Test-Path -LiteralPath $mediaVoxelDir) {
    $invalidNames = Get-ChildItem -LiteralPath $mediaVoxelDir -Recurse -File |
        Where-Object { -not (Test-SafeContentName $_.Name) } |
        Select-Object -First 100

    foreach ($file in $invalidNames) {
        $rel = Get-RelativePath $packFull $file.FullName
        Add-Warning "Invalid target media filename: $rel"
    }
}

if (Test-Path -LiteralPath $legacyDir) {
    $activeRonRoots = @("defs", "blocks", "items", "worldgen")
    foreach ($root in $activeRonRoots) {
        $dir = Join-Path $packFull $root
        if (-not (Test-Path -LiteralPath $dir)) { continue }
        $ronFiles = Get-ChildItem -LiteralPath $dir -Recurse -File -Filter *.ron
        foreach ($file in $ronFiles) {
            $text = Get-Content -LiteralPath $file.FullName -Raw
            if ($text -match "legacy_imports") {
                Add-Error "Runtime definition references legacy_imports: $($file.FullName)"
            }
        }
    }
}

$manifestDir = Join-Path $legacyDir "manifests"
if (Test-Path -LiteralPath $manifestDir) {
    $manifestCount = (Get-ChildItem -LiteralPath $manifestDir -File -Filter *.ron).Count
    if ($manifestCount -gt 0) {
        Add-Warning "Legacy manifests are quarantined but not converted yet: $manifestCount file(s)"
    }
}

if ($RunRustContentTests) {
    cargo test -p vv-pack-compiler
    if ($LASTEXITCODE -ne 0) {
        Add-Error "Rust content tests failed: cargo test -p vv-pack-compiler"
    }
}

Write-Output "Content validation warnings: $($warnings.Count)"
foreach ($warning in $warnings) {
    Write-Warning $warning
}

if ($errors.Count -gt 0) {
    Write-Output "Content validation errors: $($errors.Count)"
    foreach ($err in $errors) {
        Write-Error $err
    }
    exit 1
}

Write-Output "Content validation passed."
