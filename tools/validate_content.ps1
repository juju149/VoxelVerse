param(
    [string]$PackRoot = "assets/packs/core"
)

$ErrorActionPreference = "Stop"
$errors = New-Object System.Collections.Generic.List[string]
$warnings = New-Object System.Collections.Generic.List[string]

function FullPath([string]$Path) {
    return [System.IO.Path]::GetFullPath($Path)
}

function Add-Error([string]$Message) {
    $script:errors.Add($Message)
}

function Add-Warning([string]$Message) {
    $script:warnings.Add($Message)
}

function RelPath([string]$Base, [string]$Path) {
    $baseFull = (FullPath $Base).TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    $pathFull = FullPath $Path
    $baseUri = New-Object System.Uri($baseFull)
    $pathUri = New-Object System.Uri($pathFull)
    return [System.Uri]::UnescapeDataString($baseUri.MakeRelativeUri($pathUri).ToString()).Replace('/', '\')
}

function Assert-Dir([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        Add-Error "Missing directory: $Path"
    }
}

function Assert-File([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Add-Error "Missing file: $Path"
    }
}

function Test-SafeName([string]$Name) {
    if ($Name -cmatch "[A-Z]") { return $false }
    if ($Name -match "\s") { return $false }
    if ($Name -match "-") { return $false }
    if ([System.IO.Path]::GetFileNameWithoutExtension($Name) -match "^[0-9]+$") { return $false }
    return $true
}

function Resolve-CoreRef([string]$Ref) {
    $parts = $Ref.Split(":", 2)
    if ($parts.Count -ne 2 -or $parts[0] -ne "core") { return $null }
    $path = $parts[1]

    if ($path.StartsWith("texture/")) {
        return Join-Path $PackRoot ("media/textures/" + $path.Substring("texture/".Length) + ".png")
    }
    if ($path.StartsWith("voxel/")) {
        $assetPath = Join-Path $PackRoot ("media/voxel/" + $path.Substring("voxel/".Length))
        $filePath = "$assetPath.vox"
        if (Test-Path -LiteralPath $filePath) { return $filePath }
        return $assetPath
    }
    if ($path.StartsWith("material/")) {
        return Join-Path $PackRoot ("defs/materials/" + $path.Substring("material/".Length) + ".material.ron")
    }
    if ($path.StartsWith("block/")) {
        return Join-Path $PackRoot ("defs/blocks/" + $path.Substring("block/".Length) + ".block.ron")
    }
    if ($path.StartsWith("block_model/")) {
        return Join-Path $PackRoot ("defs/block_models/" + $path.Substring("block_model/".Length) + ".block_model.ron")
    }
    if ($path.StartsWith("item/block/")) {
        return Join-Path $PackRoot ("defs/items/blocks/" + $path.Substring("item/block/".Length) + ".item.ron")
    }
    if ($path.StartsWith("item/resource/")) {
        return Join-Path $PackRoot ("defs/items/resources/" + $path.Substring("item/resource/".Length) + ".item.ron")
    }
    if ($path.StartsWith("item/tool/")) {
        return Join-Path $PackRoot ("defs/items/tools/" + $path.Substring("item/tool/".Length) + ".item.ron")
    }
    if ($path.StartsWith("item/weapon/")) {
        return Join-Path $PackRoot ("defs/items/weapons/" + $path.Substring("item/weapon/".Length) + ".item.ron")
    }
    if ($path.StartsWith("item/misc/")) {
        return Join-Path $PackRoot ("defs/items/misc/" + $path.Substring("item/misc/".Length) + ".item.ron")
    }
    if ($path.StartsWith("loot/")) {
        return Join-Path $PackRoot ("defs/loot/" + $path.Substring("loot/".Length) + ".loot.ron")
    }
    if ($path.StartsWith("entity/")) {
        return Join-Path $PackRoot ("defs/entities/" + $path.Substring("entity/".Length) + ".entity.ron")
    }
    if ($path.StartsWith("skeleton/")) {
        return Join-Path $PackRoot ("defs/skeletons/" + $path.Substring("skeleton/".Length) + ".skeleton.ron")
    }
    if ($path.StartsWith("field/")) {
        return Join-Path $PackRoot ("defs/worldgen/noise_fields/" + $path.Substring("field/".Length) + ".field.ron")
    }
    if ($path.StartsWith("climate/")) {
        return Join-Path $PackRoot ("defs/worldgen/climate_profiles/" + $path.Substring("climate/".Length) + ".climate.ron")
    }
    if ($path.StartsWith("biome_set/")) {
        return Join-Path $PackRoot ("defs/worldgen/biome_sets/" + $path.Substring("biome_set/".Length) + ".biome_set.ron")
    }
    if ($path.StartsWith("biome/")) {
        return Join-Path $PackRoot ("defs/worldgen/biomes/" + $path.Substring("biome/".Length) + ".biome.ron")
    }
    if ($path.StartsWith("terrain_layers/")) {
        return Join-Path $PackRoot ("defs/worldgen/terrain_layers/" + $path.Substring("terrain_layers/".Length) + ".terrain_layers.ron")
    }
    if ($path.StartsWith("cave/")) {
        return Join-Path $PackRoot ("defs/worldgen/caves/" + $path.Substring("cave/".Length) + ".cave.ron")
    }
    if ($path.StartsWith("ore/")) {
        return Join-Path $PackRoot ("defs/worldgen/ores/" + $path.Substring("ore/".Length) + ".ore.ron")
    }
    if ($path.StartsWith("vegetation/")) {
        return Join-Path $PackRoot ("defs/worldgen/vegetation/" + $path.Substring("vegetation/".Length) + ".vegetation.ron")
    }
    if ($path.StartsWith("structure/")) {
        return Join-Path $PackRoot ("defs/worldgen/structures/" + $path.Substring("structure/".Length) + ".structure.ron")
    }
    if ($path.StartsWith("spawn/")) {
        return Join-Path $PackRoot ("defs/worldgen/spawns/" + $path.Substring("spawn/".Length) + ".spawn.ron")
    }
    if ($path.StartsWith("prop_scatter/")) {
        return Join-Path $PackRoot ("defs/worldgen/prop_scatters/" + $path.Substring("prop_scatter/".Length) + ".prop_scatter.ron")
    }
    if ($path.StartsWith("visual_detail/")) {
        return Join-Path $PackRoot ("defs/worldgen/visual_details/" + $path.Substring("visual_detail/".Length) + ".visual_detail.ron")
    }
    return $null
}

$packFull = FullPath $PackRoot
if (-not (Test-Path -LiteralPath $packFull -PathType Container)) {
    throw "Pack root not found: $PackRoot"
}

Assert-File (Join-Path $PackRoot "pack.ron")
Assert-File (Join-Path $PackRoot "README.md")
foreach ($dir in @("defs", "media", "generated", "media/voxel", "media/textures", "generated/registries")) {
    Assert-Dir (Join-Path $PackRoot $dir)
}

foreach ($legacy in @("legacy_imports", "blocks", "worldgen", "textures", "items", "voxel", "pack.toml")) {
    $path = Join-Path $PackRoot $legacy
    if (Test-Path -LiteralPath $path) {
        Add-Error "Legacy path still exists: $path"
    }
}

$sourceFullPath = (FullPath (Join-Path $PackRoot "source"))
$emptyDirs = Get-ChildItem -LiteralPath $PackRoot -Directory -Recurse |
    Where-Object { -not $_.FullName.StartsWith($sourceFullPath, [System.StringComparison]::OrdinalIgnoreCase) } |
    Where-Object { -not (Get-ChildItem -LiteralPath $_.FullName -Force) }
foreach ($dir in $emptyDirs) {
    Add-Error "Empty directory remains: $(RelPath $PackRoot $dir.FullName)"
}

$invalidFiles = Get-ChildItem -LiteralPath $PackRoot -Recurse -File |
    Where-Object { -not $_.FullName.StartsWith($sourceFullPath, [System.StringComparison]::OrdinalIgnoreCase) } |
    Where-Object { $_.Extension -in @(".ron", ".vox", ".png") -and -not (Test-SafeName $_.Name) }
foreach ($file in $invalidFiles) {
    Add-Error "Invalid content filename: $(RelPath $PackRoot $file.FullName)"
}

$sourceRoot = (FullPath (Join-Path $PackRoot "source"))
$ronFiles = Get-ChildItem -LiteralPath $PackRoot -Recurse -File -Filter *.ron |
    Where-Object { -not $_.FullName.StartsWith($sourceRoot, [System.StringComparison]::OrdinalIgnoreCase) }
$allRefs = New-Object System.Collections.Generic.HashSet[string]
foreach ($file in $ronFiles) {
    $text = Get-Content -LiteralPath $file.FullName -Raw
    if ($text -match "legacy_imports|common\.items|voxel\.sprite|voxel\.npc") {
        Add-Error "Legacy reference found in RON: $(RelPath $PackRoot $file.FullName)"
    }
    $matches = [regex]::Matches($text, '"core:[a-z0-9_./-]+"')
    foreach ($match in $matches) {
        $ref = $match.Value.Trim('"')
        $null = $allRefs.Add($ref)
    }
}

foreach ($ref in $allRefs) {
    if ($ref -eq "core:item/none") {
        continue
    }
    if ($ref -match "^core:(tag|icon|sound|atlas|effect|movement|behavior|inventory|projectile|structure_template)/") {
        continue
    }
    $resolved = Resolve-CoreRef $ref
    if ($null -eq $resolved) {
        Add-Warning "No validator mapping for reference: $ref"
        continue
    }
    if (-not (Test-Path -LiteralPath $resolved)) {
        Add-Error "Broken reference: $ref -> $(RelPath $PackRoot $resolved)"
    }
}

$voxelRegistry = Join-Path $PackRoot "generated/registries/voxel_assets.ron"
Assert-File $voxelRegistry
if (Test-Path -LiteralPath $voxelRegistry) {
    $voxCount = (Get-ChildItem -LiteralPath (Join-Path $PackRoot "media/voxel") -Recurse -File -Filter *.vox).Count
    $registryText = Get-Content -LiteralPath $voxelRegistry -Raw
    if ($registryText -notmatch "asset_count:\s+$voxCount,") {
        Add-Error "Voxel registry asset_count does not match media/voxel count ($voxCount)."
    }
    if ($registryText -notmatch 'generated_from:\s+"media/voxel"') {
        Add-Error "Voxel registry generated_from must be media/voxel."
    }

    $assetMatches = [regex]::Matches($registryText, '\(id:\s+"([^"]+)",\s+path:\s+"([^"]+)",\s+kind:\s+voxel_model\)')
    if ($assetMatches.Count -ne $voxCount) {
        Add-Error "Voxel registry entry count ($($assetMatches.Count)) does not match media/voxel count ($voxCount)."
    }

    $assetIds = New-Object System.Collections.Generic.HashSet[string]
    foreach ($match in $assetMatches) {
        $id = $match.Groups[1].Value
        $path = $match.Groups[2].Value

        if (-not $id.StartsWith("core:voxel/")) {
            Add-Error "Invalid voxel asset id in registry: $id"
        }
        if (-not $assetIds.Add($id)) {
            Add-Error "Duplicate voxel asset id in registry: $id"
        }
        if (-not $path.StartsWith("media/voxel/") -or -not $path.EndsWith(".vox")) {
            Add-Error "Invalid voxel asset path in registry: $path"
            continue
        }

        $resolved = Join-Path $PackRoot $path
        if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
            Add-Error "Voxel registry points to missing file: $path"
        }
    }
}

Write-Output "Content validation warnings: $($warnings.Count)"
foreach ($warning in $warnings) {
    Write-Warning $warning
}

if ($errors.Count -gt 0) {
    Write-Output "Content validation errors: $($errors.Count)"
    foreach ($err in $errors) {
        Write-Output "ERROR: $err"
    }
    exit 1
}

Write-Output "Content validation passed."
