param(
    [string]$PackRoot = "assets/packs/core",
    [switch]$Apply
)

$ErrorActionPreference = "Stop"

function Join-RelPath {
    param([string[]]$Parts)
    $path = ""
    foreach ($part in $Parts) {
        if ([string]::IsNullOrWhiteSpace($part)) { continue }
        if ($path -eq "") { $path = $part } else { $path = Join-Path $path $part }
    }
    return $path
}

function Convert-SafeName {
    param([string]$Name)
    $extension = [System.IO.Path]::GetExtension($Name)
    $stem = [System.IO.Path]::GetFileNameWithoutExtension($Name)
    $safeStem = $stem.ToLowerInvariant()
    $safeStem = $safeStem -replace "[^a-z0-9_]+", "_"
    $safeStem = $safeStem -replace "_+", "_"
    $safeStem = $safeStem.Trim("_")
    if ([string]::IsNullOrWhiteSpace($safeStem)) {
        $safeStem = "unnamed"
    }
    if ($safeStem -match "^[0-9]+$") {
        $safeStem = "{0:D2}" -f [int]$safeStem
    }
    return "$safeStem$extension"
}

function Convert-SafeRelativePath {
    param([string]$RelativePath)
    $parts = $RelativePath -split "[\\/]+"
    $safe = New-Object System.Collections.Generic.List[string]
    for ($i = 0; $i -lt $parts.Count; $i++) {
        $part = $parts[$i]
        if ([string]::IsNullOrWhiteSpace($part)) { continue }
        $safeName = Convert-SafeName $part
        $stem = [System.IO.Path]::GetFileNameWithoutExtension($safeName)
        $extension = [System.IO.Path]::GetExtension($safeName)
        if ($i -eq ($parts.Count - 1) -and $stem -match "^[0-9]+$" -and $safe.Count -gt 0) {
            $prefix = $safe[$safe.Count - 1]
            $prefix = [System.IO.Path]::GetFileNameWithoutExtension($prefix)
            $safeName = "$prefix`_$stem$extension"
        }
        $safe.Add($safeName)
    }
    return (Join-RelPath $safe.ToArray())
}

function Get-RelativePath {
    param(
        [string]$Base,
        [string]$Path
    )
    $baseFull = [System.IO.Path]::GetFullPath($Base).TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    $pathFull = [System.IO.Path]::GetFullPath($Path)
    $baseUri = New-Object System.Uri($baseFull)
    $pathUri = New-Object System.Uri($pathFull)
    $relativeUri = $baseUri.MakeRelativeUri($pathUri)
    return [System.Uri]::UnescapeDataString($relativeUri.ToString()).Replace('/', [System.IO.Path]::DirectorySeparatorChar)
}

function Ensure-Dir {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path | Out-Null
    }
}

function Test-GitRepo {
    try {
        $inside = git rev-parse --is-inside-work-tree 2>$null
        return ($LASTEXITCODE -eq 0 -and $inside -eq "true")
    } catch {
        return $false
    }
}

function Get-GitRoot {
    try {
        $root = git rev-parse --show-toplevel 2>$null
        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($root)) {
            return [System.IO.Path]::GetFullPath($root)
        }
    } catch {
        return $null
    }
    return $null
}

function Test-GitTracked {
    param(
        [string]$GitRoot,
        [string]$Path
    )
    if ([string]::IsNullOrWhiteSpace($GitRoot)) {
        return $false
    }
    $rel = Get-RelativePath $GitRoot $Path
    git ls-files --error-unmatch -- "$rel" 2>$null | Out-Null
    return ($LASTEXITCODE -eq 0)
}

function Add-MapRow {
    param(
        [System.Collections.Generic.List[object]]$Rows,
        [string]$OldPath,
        [string]$NewPath,
        [string]$Category,
        [string]$Action,
        [string]$Confidence,
        [string]$Notes
    )
    $Rows.Add([pscustomobject]@{
        old_path = $OldPath
        new_path = $NewPath
        category = $Category
        action = $Action
        confidence = $Confidence
        notes = $Notes
    })
}

function Resolve-SpriteTarget {
    param([string]$RelativePath)
    $parts = $RelativePath -split "[\\/]+"
    $root = if ($parts.Count -gt 0) { $parts[0] } else { "" }
    $rest = if ($parts.Count -gt 1) { Join-RelPath $parts[1..($parts.Count - 1)] } else { "" }

    switch -Regex ($root) {
        "^chests$" { return @("media/voxel/props/containers", $rest, "prop_container", "high", "container sprite") }
        "^underwater_chests$" { return @("media/voxel/props/containers/underwater", $rest, "prop_container", "high", "underwater container sprite") }
        "^furniture$" { return @("media/voxel/props/furniture", $rest, "prop_furniture", "high", "furniture sprite") }
        "^crafting_station$" { return @("media/voxel/props/crafting_stations", $rest, "prop_crafting_station", "high", "crafting station sprite") }
        "^door$" { return @("media/voxel/props/doors", $rest, "prop_door", "high", "door sprite") }
        "^window$" { return @("media/voxel/props/structure_parts/windows", $rest, "prop_structure_part", "high", "window sprite") }
        "^lantern$" { return @("media/voxel/props/lights", $rest, "prop_light", "high", "placed light sprite") }
        "^camp$" { return @("media/voxel/props/lights/camp", $rest, "prop_light", "medium", "camp or fire sprite") }
        "^sign$" { return @("media/voxel/props/interactables/signs", $rest, "prop_interactable", "high", "sign sprite") }
        "^barricades_wood$" { return @("media/voxel/props/structure_parts/barricades_wood", $rest, "prop_structure_part", "high", "barricade sprite") }
        "^bars$" { return @("media/voxel/props/structure_parts/bars", $rest, "prop_structure_part", "high", "bar sprite") }
        "^castle$" { return @("media/voxel/props/structure_parts/castle", $rest, "prop_structure_part", "medium", "castle part sprite") }
        "^grave$" { return @("media/voxel/props/decoration/grave", $rest, "prop_decoration", "high", "grave decoration") }
        "^grass$|^ferns$|^junglefern$|^jungleredgrass$|^leafy_plant$|^lianas$|^reed$" { return @("media/voxel/vegetation/grass/$root", $rest, "vegetation_grass", "high", "grass-like vegetation") }
        "^flowers$" { return @("media/voxel/vegetation/flowers", $rest, "vegetation_flower", "high", "flower vegetation") }
        "^mushrooms$" { return @("media/voxel/vegetation/mushrooms", $rest, "vegetation_mushroom", "high", "mushroom vegetation") }
        "^blueberry$|^dead_bush$|^dead_plant$|^savanna_bush$|^snow_bush$|^welwitch$" { return @("media/voxel/vegetation/bushes/$root", $rest, "vegetation_bush", "medium", "bush-like vegetation") }
        "^cacti$" { return @("media/voxel/vegetation/desert/cacti", $rest, "vegetation_desert", "high", "desert vegetation") }
        "^underwater_" { return @("media/voxel/vegetation/underwater/$root", $rest, "vegetation_underwater", "high", "underwater vegetation") }
        "^algae$|^coral$|^lillypads$" { return @("media/voxel/vegetation/underwater/$root", $rest, "vegetation_underwater", "high", "aquatic vegetation") }
        "^carrot$|^corn$|^farms$|^flax$|^lettuce$|^pumpkin$|^radish$|^tomato$|^turnip$|^wheat_green$|^wheat_yellow$" { return @("media/voxel/vegetation/crops/$root", $rest, "vegetation_crop", "high", "crop vegetation") }
        "^potion$" { return @("media/voxel/items/consumables/potions", $rest, "item_consumable", "medium", "potion sprite") }
        "^mineral$" { return @("media/voxel/items/resources/mineral", $rest, "item_resource", "medium", "mineral sprite") }
        "^rocks$" { return @("media/voxel/props/decoration/rocks", $rest, "prop_decoration", "medium", "rock sprite") }
        default { return @("legacy_imports/needs_review/voxel/sprite/$root", $rest, "needs_review", "medium", "unclassified sprite") }
    }
}

function Resolve-VoxelTarget {
    param(
        [string]$VoxelRoot,
        [System.IO.FileInfo]$File
    )
    $relative = Get-RelativePath $VoxelRoot $File.FullName
    $parts = $relative -split "[\\/]+"
    $root = if ($parts.Count -gt 0) { $parts[0] } else { "" }
    $rest = if ($parts.Count -gt 1) { Join-RelPath $parts[1..($parts.Count - 1)] } else { "" }
    $safeRest = Convert-SafeRelativePath $rest
    $safeFile = Convert-SafeName $File.Name

    if ($File.Extension -eq ".ron" -and $File.Name -like "*_manifest.ron") {
        return @("legacy_imports/manifests", (Convert-SafeName $File.Name), "legacy_manifest", "quarantine", "high", "legacy manifest")
    }
    if ($File.Name -eq "README.md") {
        return @("legacy_imports/manifests", "readme_voxel_legacy.md", "legacy_manifest", "quarantine", "high", "legacy voxel README")
    }
    if ($relative -eq "char_template.vox" -or $relative -eq "not_found.vox") {
        return @("media/voxel/debug", $safeFile, "debug", "move", "high", "debug/template voxel")
    }
    if ($relative -eq "particle.vox") {
        return @("media/voxel/effects", $safeFile, "effect", "move", "high", "effect voxel")
    }

    switch ($root) {
        "armor" {
            if ($relative -match "admin|debug") {
                return @("media/voxel/debug/admin", $safeRest, "debug", "move", "medium", "admin armor asset")
            }
            return @("media/voxel/equipment/armor", $safeRest, "equipment_armor", "move", "high", "armor voxel")
        }
        "glider" { return @("media/voxel/equipment/gliders", $safeRest, "equipment_glider", "move", "high", "glider voxel") }
        "lantern" { return @("media/voxel/equipment/accessories/lanterns", $safeRest, "equipment_accessory", "move", "medium", "equipable lantern voxel") }
        "figure" { return @("media/voxel/characters/humanoids", $safeRest, "character_humanoid", "move", "medium", "humanoid figure voxel") }
        "npc" { return @("media/voxel/creatures/needs_review", $safeRest, "creature_needs_review", "move", "low", "needs entity taxonomy") }
        "object" { return @("media/voxel/props/interactables", $safeRest, "prop_interactable", "move", "medium", "object voxel") }
        "item" {
            $itemParts = $rest -split "[\\/]+"
            $itemRoot = if ($itemParts.Count -gt 0) { $itemParts[0] } else { "" }
            $itemRest = if ($itemParts.Count -gt 1) { Convert-SafeRelativePath (Join-RelPath $itemParts[1..($itemParts.Count - 1)]) } else { $safeFile }
            switch ($itemRoot) {
                "food" { return @("media/voxel/items/food", $itemRest, "item_food", "move", "high", "food item voxel") }
                "consumable" { return @("media/voxel/items/consumables", $itemRest, "item_consumable", "move", "high", "consumable item voxel") }
                "crafting" { return @("media/voxel/items/crafting", $itemRest, "item_crafting", "move", "high", "crafting item voxel") }
                "mineral" { return @("media/voxel/items/resources/mineral", $itemRest, "item_resource", "move", "high", "mineral item voxel") }
                default { return @("media/voxel/items/needs_review/$itemRoot", $itemRest, "item_needs_review", "move", "medium", "needs item taxonomy") }
            }
        }
        "weapon" {
            if ($rest -match "^debug_wand") {
                return @("media/voxel/debug/tools", $safeRest, "debug", "move", "high", "debug weapon")
            }
            $weaponParts = $rest -split "[\\/]+"
            $weaponRoot = if ($weaponParts.Count -gt 0) { $weaponParts[0] } else { "" }
            $weaponRest = if ($weaponParts.Count -gt 1) { Convert-SafeRelativePath (Join-RelPath $weaponParts[1..($weaponParts.Count - 1)]) } else { $safeFile }
            switch ($weaponRoot) {
                "projectile" { return @("media/voxel/projectiles", $weaponRest, "projectile", "move", "high", "projectile voxel") }
                "shield" { return @("media/voxel/equipment/shields", $weaponRest, "equipment_shield", "move", "high", "shield voxel") }
                "tool" { return @("media/voxel/equipment/tools", $weaponRest, "equipment_tool", "move", "high", "tool voxel") }
                default { return @("media/voxel/equipment/weapons/$weaponRoot", $weaponRest, "equipment_weapon", "move", "medium", "weapon voxel") }
            }
        }
        "sprite" {
            $target = Resolve-SpriteTarget $rest
            return @($target[0], (Convert-SafeRelativePath $target[1]), $target[2], "move", $target[3], $target[4])
        }
        default {
            return @("legacy_imports/needs_review/voxel/$root", $safeRest, "needs_review", "quarantine", "low", "unknown voxel root")
        }
    }
}

$packFull = [System.IO.Path]::GetFullPath($PackRoot)
$voxelRoot = Join-Path $packFull "voxel"
$diagnosticsDir = Join-Path $packFull "generated/diagnostics"
$mapPath = Join-Path $diagnosticsDir "voxel_asset_migration_map.csv"
$collisionPath = Join-Path $diagnosticsDir "voxel_asset_migration_collisions.csv"
$moveLogPath = Join-Path $diagnosticsDir "voxel_asset_migration_moves.csv"

if (-not (Test-Path -LiteralPath $packFull)) {
    throw "Pack root not found: $PackRoot"
}
if (-not (Test-Path -LiteralPath $voxelRoot)) {
    throw "Historical voxel root not found: $voxelRoot"
}

$targetDirs = @(
    "defs/blocks", "defs/materials", "defs/items", "defs/entities", "defs/props",
    "defs/vegetation", "defs/recipes", "defs/biomes", "defs/worldgen", "defs/loot",
    "defs/skeletons", "defs/animations", "defs/sounds", "defs/tags",
    "media/textures", "media/voxel", "media/audio", "media/icons", "media/particles", "media/ui",
    "media/voxel/characters/player", "media/voxel/characters/humanoids",
    "media/voxel/creatures/animals", "media/voxel/creatures/monsters", "media/voxel/creatures/bosses",
    "media/voxel/creatures/aquatic", "media/voxel/creatures/flying", "media/voxel/creatures/insects",
    "media/voxel/creatures/humanoids", "media/voxel/creatures/needs_review",
    "media/voxel/equipment/armor", "media/voxel/equipment/weapons", "media/voxel/equipment/tools",
    "media/voxel/equipment/shields", "media/voxel/equipment/gliders", "media/voxel/equipment/accessories",
    "media/voxel/items/resources", "media/voxel/items/food", "media/voxel/items/consumables",
    "media/voxel/items/keys", "media/voxel/items/crafting", "media/voxel/items/loot", "media/voxel/items/needs_review",
    "media/voxel/props/crafting_stations", "media/voxel/props/containers", "media/voxel/props/doors",
    "media/voxel/props/furniture", "media/voxel/props/lights", "media/voxel/props/decoration",
    "media/voxel/props/traps", "media/voxel/props/structure_parts", "media/voxel/props/interactables",
    "media/voxel/props/needs_review",
    "media/voxel/vegetation/grass", "media/voxel/vegetation/flowers", "media/voxel/vegetation/bushes",
    "media/voxel/vegetation/mushrooms", "media/voxel/vegetation/trees", "media/voxel/vegetation/crops",
    "media/voxel/vegetation/cave", "media/voxel/vegetation/desert", "media/voxel/vegetation/snow",
    "media/voxel/vegetation/underwater", "media/voxel/vegetation/needs_review",
    "media/voxel/projectiles", "media/voxel/effects", "media/voxel/debug",
    "source/voxel_raw", "source/texture_sources", "source/references",
    "generated/registries", "generated/atlases", "generated/mesh_cache", "generated/icons", "generated/diagnostics",
    "legacy_imports/manifests", "legacy_imports/voxel_raw", "legacy_imports/needs_review", "legacy_imports/deprecated"
)

foreach ($dir in $targetDirs) {
    Ensure-Dir (Join-Path $packFull $dir)
}

$rows = New-Object System.Collections.Generic.List[object]
$files = Get-ChildItem -LiteralPath $voxelRoot -Recurse -File | Sort-Object FullName

foreach ($file in $files) {
    $target = Resolve-VoxelTarget $voxelRoot $file
    $targetDir = Join-Path $packFull $target[0]
    $targetRel = $target[1]
    $newPath = Join-Path $targetDir $targetRel
    Add-MapRow $rows $file.FullName $newPath $target[2] $target[3] $target[4] $target[5]
}

$collisions = $rows |
    Group-Object new_path |
    Where-Object { $_.Count -gt 1 } |
    ForEach-Object {
        foreach ($row in $_.Group) {
            [pscustomobject]@{
                new_path = $_.Name
                old_path = $row.old_path
                category = $row.category
                notes = "multiple sources map to same destination"
            }
        }
    }

$rows | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $mapPath
$collisions | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $collisionPath

if ($collisions.Count -gt 0) {
    Write-Warning "Migration map has $($collisions.Count) collision rows. Review $collisionPath before applying."
    if ($Apply) {
        throw "Refusing to apply migration with destination collisions."
    }
}

$moveLog = New-Object System.Collections.Generic.List[object]
$useGit = Test-GitRepo
$gitRoot = if ($useGit) { Get-GitRoot } else { $null }

foreach ($row in $rows) {
    $old = $row.old_path
    $new = $row.new_path
    $status = "dry_run"
    $message = ""

    if ($Apply) {
        if (-not (Test-Path -LiteralPath $old)) {
            $status = "missing_source"
            $message = "source does not exist"
        } elseif (Test-Path -LiteralPath $new) {
            $status = "blocked_existing_destination"
            $message = "destination already exists"
        } else {
            Ensure-Dir ([System.IO.Path]::GetDirectoryName($new))
            if ($useGit -and (Test-GitTracked $gitRoot $old)) {
                git mv -- "$old" "$new"
                if ($LASTEXITCODE -ne 0) {
                    throw "git mv failed: $old -> $new"
                }
            } else {
                Move-Item -LiteralPath $old -Destination $new
            }
            $status = "moved"
        }
    }

    $moveLog.Add([pscustomobject]@{
        old_path = $old
        new_path = $new
        action = $row.action
        category = $row.category
        status = $status
        message = $message
    })
}

$moveLog | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $moveLogPath

if ($Apply) {
    Write-Output "Applied voxel asset migration. Move log: $moveLogPath"
} else {
    Write-Output "Dry-run complete. Migration map: $mapPath"
    Write-Output "Collision report: $collisionPath"
    Write-Output "Move log: $moveLogPath"
    Write-Output "Run again with -Apply only after reviewing diagnostics."
}
