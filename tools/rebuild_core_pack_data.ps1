param(
    [string]$PackRoot = "assets/packs/core"
)

$ErrorActionPreference = "Stop"

function FullPath([string]$Path) {
    return [System.IO.Path]::GetFullPath($Path)
}

function Assert-InPack([string]$Path) {
    $pack = (FullPath $PackRoot).TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    $full = FullPath $Path
    $packRootFull = $pack.TrimEnd('\', '/')
    if (
        -not $full.Equals($packRootFull, [System.StringComparison]::OrdinalIgnoreCase) -and
        -not $full.StartsWith($pack, [System.StringComparison]::OrdinalIgnoreCase)
    ) {
        throw "Refusing to touch path outside pack root: $full"
    }
    return $full
}

function Ensure-Dir([string]$Path) {
    $full = Assert-InPack $Path
    if (-not (Test-Path -LiteralPath $full)) {
        New-Item -ItemType Directory -Path $full | Out-Null
    }
}

function Write-Text([string]$Path, [string]$Text) {
    $full = Assert-InPack $Path
    Ensure-Dir ([System.IO.Path]::GetDirectoryName($full))
    Set-Content -LiteralPath $full -Value $Text.TrimStart() -Encoding UTF8
}

function Move-FileSafe([string]$Source, [string]$Destination) {
    $src = Assert-InPack $Source
    $dst = Assert-InPack $Destination
    if (-not (Test-Path -LiteralPath $src)) { return }
    if (Test-Path -LiteralPath $dst) { throw "Destination already exists: $dst" }
    Ensure-Dir ([System.IO.Path]::GetDirectoryName($dst))
    Move-Item -LiteralPath $src -Destination $dst
}

function Move-TreeContents([string]$Source, [string]$Destination) {
    $src = Assert-InPack $Source
    $dst = Assert-InPack $Destination
    if (-not (Test-Path -LiteralPath $src)) { return }
    Ensure-Dir $dst
    Get-ChildItem -LiteralPath $src -Force | ForEach-Object {
        $target = Join-Path $dst $_.Name
        if (Test-Path -LiteralPath $target) { throw "Destination already exists: $target" }
        Move-Item -LiteralPath $_.FullName -Destination $target
    }
}

function Remove-TreeSafe([string]$Path) {
    $full = Assert-InPack $Path
    if (Test-Path -LiteralPath $full) {
        Remove-Item -LiteralPath $full -Recurse -Force
    }
}

function Remove-EmptyDirs([string]$Root) {
    $full = Assert-InPack $Root
    if (-not (Test-Path -LiteralPath $full)) { return }
    Get-ChildItem -LiteralPath $full -Directory -Recurse |
        Sort-Object FullName -Descending |
        Where-Object { -not (Get-ChildItem -LiteralPath $_.FullName -Force) } |
        Remove-Item -Force
}

function Normalize-BareNumericFiles([string]$Root) {
    $full = Assert-InPack $Root
    if (-not (Test-Path -LiteralPath $full)) { return }
    $files = Get-ChildItem -LiteralPath $full -Recurse -File |
        Where-Object { $_.BaseName -match "^[0-9]+$" } |
        Sort-Object FullName
    foreach ($file in $files) {
        $parent = Split-Path -Leaf $file.DirectoryName
        $safeParent = $parent.ToLowerInvariant() -replace "[^a-z0-9_]+", "_"
        $safeParent = ($safeParent -replace "_+", "_").Trim("_")
        $newName = "$safeParent`_$($file.BaseName)$($file.Extension)"
        $target = Join-Path $file.DirectoryName $newName
        if (Test-Path -LiteralPath $target) { throw "Filename normalization collision: $target" }
        Move-Item -LiteralPath $file.FullName -Destination $target
    }
}

function To-PackPath([string]$Path) {
    $pack = (FullPath $PackRoot).TrimEnd('\', '/') + [System.IO.Path]::DirectorySeparatorChar
    $full = FullPath $Path
    return $full.Substring($pack.Length).Replace('\', '/')
}

function VoxelIdFromPath([string]$Path) {
    $rel = To-PackPath $Path
    $rel = $rel -replace "^media/voxel/", ""
    $rel = $rel -replace "\.vox$", ""
    return "core:voxel/$rel"
}

$pack = Assert-InPack $PackRoot

$dirs = @(
    "defs/assets",
    "defs/blocks/air",
    "defs/blocks/terrain",
    "defs/blocks/natural/logs",
    "defs/blocks/natural/leaves",
    "defs/blocks/flora",
    "defs/blocks/ores",
    "defs/items/blocks",
    "defs/items/resources",
    "defs/items/tools",
    "defs/items/weapons",
    "defs/items/food",
    "defs/items/consumables",
    "defs/materials/blocks",
    "defs/entities/player",
    "defs/entities/animals",
    "defs/props",
    "defs/vegetation",
    "defs/loot/blocks",
    "defs/loot/entities",
    "defs/recipes/crafting",
    "defs/skeletons",
    "defs/sounds",
    "defs/tags/blocks",
    "defs/tags/items",
    "defs/tags/biomes",
    "defs/worldgen/planet_profiles",
    "defs/worldgen/climate_profiles",
    "defs/worldgen/biome_sets",
    "defs/worldgen/biomes",
    "defs/worldgen/noise_fields",
    "defs/worldgen/terrain_layers",
    "defs/worldgen/ores",
    "defs/worldgen/caves",
    "defs/worldgen/vegetation",
    "defs/worldgen/structures",
    "defs/worldgen/spawns",
    "defs/worldgen/visual_details",
    "generated/registries"
)
foreach ($dir in $dirs) { Ensure-Dir (Join-Path $PackRoot $dir) }

Move-TreeContents (Join-Path $PackRoot "textures") (Join-Path $PackRoot "media/textures")

$legacy = Join-Path $PackRoot "legacy_imports/needs_review/voxel/sprite"
if (Test-Path -LiteralPath (Assert-InPack $legacy)) {
    Move-TreeContents "$legacy/beehive" (Join-Path $PackRoot "media/voxel/props/interactables/beehives")
    Move-TreeContents "$legacy/bomb" (Join-Path $PackRoot "media/voxel/props/traps/bombs")
    Move-TreeContents "$legacy/bone" (Join-Path $PackRoot "media/voxel/props/decoration/bones")
    Move-TreeContents "$legacy/cave" (Join-Path $PackRoot "media/voxel/vegetation/cave")
    Move-TreeContents "$legacy/cavern" (Join-Path $PackRoot "media/voxel/vegetation/underwater/lillypads")
    Move-TreeContents "$legacy/crystal" (Join-Path $PackRoot "media/voxel/props/decoration/crystals")
    Move-TreeContents "$legacy/ember" (Join-Path $PackRoot "media/voxel/effects/embers")
    Move-TreeContents "$legacy/fruit" (Join-Path $PackRoot "media/voxel/items/food/fruit")
    Move-TreeContents "$legacy/hay" (Join-Path $PackRoot "media/voxel/props/decoration/hay")
    Move-TreeContents "$legacy/lingonberry" (Join-Path $PackRoot "media/voxel/vegetation/bushes/lingonberry")
    Move-TreeContents "$legacy/sea_urchin" (Join-Path $PackRoot "media/voxel/vegetation/underwater/sea_urchins")
    Move-TreeContents "$legacy/seashells" (Join-Path $PackRoot "media/voxel/props/decoration/seashells")
    Move-TreeContents "$legacy/snow_pebbles" (Join-Path $PackRoot "media/voxel/props/decoration/snow_pebbles")
    Move-TreeContents "$legacy/twigs" (Join-Path $PackRoot "media/voxel/items/resources/twigs")
    Move-TreeContents "$legacy/wizard" (Join-Path $PackRoot "media/voxel/effects/magic")
    Move-TreeContents "$legacy/wood" (Join-Path $PackRoot "media/voxel/items/resources/wood")

    Move-FileSafe "$legacy/misc/bell.vox" (Join-Path $PackRoot "media/voxel/props/interactables/bells/bell.vox")
    Move-FileSafe "$legacy/misc/crystal_ball.vox" (Join-Path $PackRoot "media/voxel/props/interactables/magic/crystal_ball.vox")
    Move-FileSafe "$legacy/misc/ensnaring_vines.vox" (Join-Path $PackRoot "media/voxel/props/traps/ensnaring_vines.vox")
    Move-FileSafe "$legacy/misc/ensnaring_web.vox" (Join-Path $PackRoot "media/voxel/props/traps/ensnaring_web.vox")
    Move-FileSafe "$legacy/misc/fireblock.vox" (Join-Path $PackRoot "media/voxel/effects/fire/fireblock.vox")
    Move-FileSafe "$legacy/misc/floating_diamond.vox" (Join-Path $PackRoot "media/voxel/props/decoration/crystals/floating_diamond.vox")
    Move-FileSafe "$legacy/misc/icespike.vox" (Join-Path $PackRoot "media/voxel/props/traps/spikes/icespike.vox")
    Move-FileSafe "$legacy/misc/iron_spike.vox" (Join-Path $PackRoot "media/voxel/props/traps/spikes/iron_spike.vox")
    Move-FileSafe "$legacy/misc/ladder.vox" (Join-Path $PackRoot "media/voxel/props/structure_parts/ladders/ladder.vox")
    Move-FileSafe "$legacy/misc/lantern_ground_open.vox" (Join-Path $PackRoot "media/voxel/props/lights/lantern_ground_open.vox")
    Move-FileSafe "$legacy/misc/metal_chain.vox" (Join-Path $PackRoot "media/voxel/props/structure_parts/chains/metal_chain.vox")
    Move-FileSafe "$legacy/misc/rope.vox" (Join-Path $PackRoot "media/voxel/props/structure_parts/ropes/rope.vox")
    Move-FileSafe "$legacy/misc/scarecrow.vox" (Join-Path $PackRoot "media/voxel/props/decoration/scarecrows/scarecrow.vox")
    Move-FileSafe "$legacy/misc/street_lamp.vox" (Join-Path $PackRoot "media/voxel/props/lights/street_lamp.vox")
    Move-FileSafe "$legacy/misc/tent.vox" (Join-Path $PackRoot "media/voxel/props/interactables/camps/tent.vox")

    Get-ChildItem -LiteralPath "$legacy/misc" -File -Filter "sea_decor*.vox" -ErrorAction SilentlyContinue |
        ForEach-Object { Move-FileSafe $_.FullName (Join-Path $PackRoot "media/voxel/props/structure_parts/sea_decor/$($_.Name)") }
    Get-ChildItem -LiteralPath "$legacy/misc" -File -Filter "terracotta_block*.vox" -ErrorAction SilentlyContinue |
        ForEach-Object { Move-FileSafe $_.FullName (Join-Path $PackRoot "media/voxel/props/structure_parts/terracotta/$($_.Name)") }
}

Write-Text (Join-Path $PackRoot "pack.ron") @"
PackManifest(
    format_version: 1,
    namespace: "core",
    display_name: "VoxelVerse Core",
    version: "0.1.0",
    kind: builtin,
    description: "Built-in foundation content for VoxelVerse.",
    authors: ["VoxelVerse Team"],
    license: "project-internal",
    load_priority: 0,
    dependencies: [],
    features: [
        "blocks",
        "materials",
        "items",
        "entities",
        "props",
        "vegetation",
        "worldgen",
        "loot",
        "recipes",
        "tags",
    ],
    content_roots: (
        definitions: "defs",
        media: "media",
        generated: "generated",
    ),
    rules: (
        identity: path_derived,
        id_style: "namespace:domain/category/name",
        runtime_loads_raw_files: false,
    ),
)
"@

function Write-MaterialDefs {
    $textureRoot = Join-Path $PackRoot "media/textures/blocks"
    if (-not (Test-Path -LiteralPath (Assert-InPack $textureRoot))) { return }
    $albedos = Get-ChildItem -LiteralPath $textureRoot -Recurse -File -Filter "*_albedo.png" | Sort-Object FullName
    foreach ($albedo in $albedos) {
        $stem = $albedo.BaseName -replace "_albedo$", ""
        $dir = $albedo.DirectoryName
        $relDir = (To-PackPath $dir) -replace "^media/textures/blocks/", ""
        $normal = Join-Path $dir "$($stem)_normal.png"
        $roughness = Join-Path $dir "$($stem)_roughness.png"
        $materialPath = Join-Path $PackRoot "defs/materials/blocks/$relDir/$stem.material.ron"
        $tint = if ($stem -match "grass.*top|leaves|foliage") { "Some(BiomeTint(""foliage_or_grass""))" } else { "None" }
        $normalRef = if (Test-Path -LiteralPath $normal) { "Some(""core:texture/blocks/$relDir/$($stem)_normal"")" } else { "None" }
        $roughRef = if (Test-Path -LiteralPath $roughness) { "Some(""core:texture/blocks/$relDir/$($stem)_roughness"")" } else { "None" }
        Write-Text $materialPath @"
MaterialDef(
    display_name: "$($stem -replace "_", " ")",
    category: block_surface,
    albedo: "core:texture/blocks/$relDir/$($stem)_albedo",
    normal: $normalRef,
    roughness: $roughRef,
    tint: $tint,
    render: opaque,
    sampling: pixel_art_nearest,
    atlas: "core:atlas/blocks/main",
    authoring: (
        source: "media/textures/blocks/$relDir",
        generated_by: "tools/rebuild_core_pack_data.ps1",
    ),
)
"@
    }
}

Write-MaterialDefs

function BlockDef(
    [string]$Path, [string]$Display, [string]$Category, [string]$Shape, [string]$Collision,
    [string]$Render, [string]$Hardness, [string]$Tool, [string]$Drop, [string]$Materials,
    [string]$Tags, [string]$Sound = "stone", [string]$Extra = ""
) {
    Write-Text (Join-Path $PackRoot $Path) @"
BlockDef(
    display_name: "$Display",
    category: "$Category",
    physical: (
        solid: $($Collision -ne "none" -and $Shape -ne "none" | ForEach-Object { $_.ToString().ToLowerInvariant() }),
        opaque: $($Render -eq "opaque" | ForEach-Object { $_.ToString().ToLowerInvariant() }),
        collision: $Collision,
        hardness: $Hardness,
        blast_resistance: $([double]$Hardness * 3.0),
        friction: 0.82,
        restitution: 0.0,
    ),
    visual: (
        shape: $Shape,
        render: $Render,
        materials: $Materials,
        ambient_occlusion: true,
        casts_shadow: true,
    ),
    gameplay: (
        preferred_tool: $Tool,
        drops: "$Drop",
        placement: grid_aligned,
        replaceable: false,
    ),
    audio: (
        footstep: "core:sound/step/$Sound",
        break: "core:sound/break/$Sound",
        place: "core:sound/place/$Sound",
    ),
    tags: $Tags,
    $Extra
)
"@
}

BlockDef "defs/blocks/air/air.block.ron" "Air" "air" "none" "none" "invisible" "0.0" "None" "core:loot/blocks/air" "None" "[]" "none" "runtime: (reserved_id: 0, can_target: false, blocks_light: false),"
BlockDef "defs/blocks/terrain/core.block.ron" "Planet Core" "terrain" "cube" "full_cube" "opaque" "999.0" "None" "core:loot/blocks/empty" 'All("core:material/blocks/bedrock/bedrock_all")' '["core:tag/block/unbreakable", "core:tag/block/planet_core"]' "stone" "runtime: (role: planet_core),"
BlockDef "defs/blocks/terrain/bedrock.block.ron" "Bedrock" "terrain" "cube" "full_cube" "opaque" "999.0" "None" "core:loot/blocks/empty" 'All("core:material/blocks/bedrock/bedrock_all")' '["core:tag/block/unbreakable"]'
BlockDef "defs/blocks/terrain/grass.block.ron" "Grass Block" "terrain" "cube" "full_cube" "opaque" "0.6" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/grass" '(top: "core:material/blocks/grass_block/grass_block_top", sides: "core:material/blocks/grass_block/grass_block_side", bottom: "core:material/blocks/grass_block/grass_block_bottom")' '["core:tag/block/terrain", "core:tag/block/soil", "core:tag/block/supports_surface_vegetation"]' "grass" "runtime: (role: default_place),"
BlockDef "defs/blocks/terrain/dirt.block.ron" "Dirt" "terrain" "cube" "full_cube" "opaque" "0.5" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/dirt" 'All("core:material/blocks/dirt/dirt_all")' '["core:tag/block/terrain", "core:tag/block/soil"]' "dirt"
BlockDef "defs/blocks/terrain/coarse_dirt.block.ron" "Coarse Dirt" "terrain" "cube" "full_cube" "opaque" "0.65" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/coarse_dirt" 'All("core:material/blocks/dirt/dirt_all")' '["core:tag/block/terrain", "core:tag/block/soil", "core:tag/block/dry_soil"]' "dirt"
BlockDef "defs/blocks/terrain/podzol.block.ron" "Podzol" "terrain" "cube" "full_cube" "opaque" "0.6" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/podzol" '(top: "core:material/blocks/grass_block/grass_block_top", sides: "core:material/blocks/dirt/dirt_all", bottom: "core:material/blocks/dirt/dirt_all")' '["core:tag/block/terrain", "core:tag/block/soil", "core:tag/block/forest_floor"]' "grass"
BlockDef "defs/blocks/terrain/stone.block.ron" "Smooth Stone" "terrain" "cube" "full_cube" "opaque" "1.5" 'Some("core:tag/item/tool/pickaxe")' "core:loot/blocks/stone" 'All("core:material/blocks/smooth_stone/smooth_stone_all")' '["core:tag/block/terrain", "core:tag/block/stone"]'
BlockDef "defs/blocks/terrain/andesite.block.ron" "Andesite" "terrain" "cube" "full_cube" "opaque" "1.6" 'Some("core:tag/item/tool/pickaxe")' "core:loot/blocks/andesite" 'All("core:material/blocks/andesite/andesite_all")' '["core:tag/block/terrain", "core:tag/block/stone"]'
BlockDef "defs/blocks/terrain/deepslate.block.ron" "Deepslate" "terrain" "cube" "full_cube" "opaque" "2.3" 'Some("core:tag/item/tool/pickaxe")' "core:loot/blocks/deepslate" '(top: "core:material/blocks/deepslate/deepslate_top", sides: "core:material/blocks/deepslate/deepslate_side", bottom: "core:material/blocks/deepslate/deepslate_top")' '["core:tag/block/terrain", "core:tag/block/stone", "core:tag/block/deep_layer"]'
BlockDef "defs/blocks/terrain/red_sand.block.ron" "Red Sand" "terrain" "cube" "full_cube" "opaque" "0.5" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/red_sand" 'All("core:material/blocks/red_sand/red_sand_all")' '["core:tag/block/terrain", "core:tag/block/sand", "core:tag/block/desert"]' "sand"
BlockDef "defs/blocks/terrain/snow.block.ron" "Snow Block" "terrain" "cube" "full_cube" "opaque" "0.4" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/snow" 'All("core:material/blocks/powder_snow/powder_snow_all")' '["core:tag/block/terrain", "core:tag/block/snow"]' "snow"
BlockDef "defs/blocks/terrain/powder_snow.block.ron" "Powder Snow" "terrain" "cube" "soft_cube" "alpha_test" "0.2" 'Some("core:tag/item/tool/shovel")' "core:loot/blocks/powder_snow" 'All("core:material/blocks/powder_snow/powder_snow_all")' '["core:tag/block/terrain", "core:tag/block/snow", "core:tag/block/soft"]' "snow"

foreach ($wood in @("oak", "birch", "spruce", "acacia", "dark_oak", "jungle")) {
    BlockDef "defs/blocks/natural/logs/$wood`_log.block.ron" "$((Get-Culture).TextInfo.ToTitleCase($wood.Replace('_',' '))) Log" "natural/log" "cube" "full_cube" "opaque" "1.5" 'Some("core:tag/item/tool/axe")' "core:loot/blocks/$wood`_log" '(top: "core:material/blocks/oak_log/oak_log_top", sides: "core:material/blocks/oak_log/oak_log_side", bottom: "core:material/blocks/oak_log/oak_log_top")' '["core:tag/block/log", "core:tag/block/wood"]' "wood"
    BlockDef "defs/blocks/natural/leaves/$wood`_leaves.block.ron" "$((Get-Culture).TextInfo.ToTitleCase($wood.Replace('_',' '))) Leaves" "natural/leaves" "cube" "leaf_volume" "alpha_test" "0.25" 'Some("core:tag/item/tool/shears")' "core:loot/blocks/$wood`_leaves" 'All("core:material/blocks/oak_leaves/oak_leaves_all")' '["core:tag/block/leaves", "core:tag/block/foliage"]' "leaves" "simulation: (decays_without_log: true, supports_biome_tint: true),"
}

BlockDef "defs/blocks/flora/dandelion.block.ron" "Dandelion" "flora" "cross_plane" "none" "alpha_test" "0.0" "None" "core:loot/blocks/dandelion" 'All("core:material/blocks/dandelion/dandelion_all")' '["core:tag/block/flora", "core:tag/block/replaceable", "core:tag/block/surface_detail"]' "grass" "simulation: (surface_attached: true, breaks_when_support_removed: true),"
BlockDef "defs/blocks/flora/short_grass.block.ron" "Short Grass" "flora" "cross_plane" "none" "alpha_test" "0.0" "None" "core:loot/blocks/short_grass" 'All("core:material/blocks/short_grass/short_grass_all")' '["core:tag/block/flora", "core:tag/block/replaceable", "core:tag/block/surface_detail"]' "grass" "simulation: (surface_attached: true, breaks_when_support_removed: true),"
BlockDef "defs/blocks/ores/coal_ore.block.ron" "Coal Ore" "ore" "cube" "full_cube" "opaque" "2.0" 'Some("core:tag/item/tool/pickaxe")' "core:loot/blocks/coal_ore" 'All("core:material/blocks/coal_ore/coal_ore_all")' '["core:tag/block/ore", "core:tag/resource/coal"]'
BlockDef "defs/blocks/ores/iron_ore.block.ron" "Iron Ore" "ore" "cube" "full_cube" "opaque" "3.0" 'Some("core:tag/item/tool/pickaxe")' "core:loot/blocks/iron_ore" 'All("core:material/blocks/iron_ore/iron_ore_all")' '["core:tag/block/ore", "core:tag/resource/iron"]'

function SimpleLoot([string]$Path, [string]$Drop, [string]$Count = "1") {
    Write-Text (Join-Path $PackRoot $Path) @"
LootTableDef(
    rolls: 1,
    entries: [
        (item: "$Drop", count: ($Count, $Count), chance: 1.0),
    ],
)
"@
}

SimpleLoot "defs/loot/blocks/empty.loot.ron" "core:item/none" "0"
SimpleLoot "defs/loot/blocks/air.loot.ron" "core:item/none" "0"
SimpleLoot "defs/loot/blocks/coal_ore.loot.ron" "core:item/resource/coal" "2"
SimpleLoot "defs/loot/blocks/iron_ore.loot.ron" "core:item/resource/iron_ore_chunk"

foreach ($block in @(
    "grass", "dirt", "coarse_dirt", "podzol", "stone", "andesite", "deepslate", "red_sand", "snow", "powder_snow",
    "oak_log", "birch_log", "spruce_log", "acacia_log", "dark_oak_log", "jungle_log",
    "oak_leaves", "birch_leaves", "spruce_leaves", "acacia_leaves", "dark_oak_leaves", "jungle_leaves",
    "dandelion", "short_grass"
)) {
    SimpleLoot "defs/loot/blocks/$block.loot.ron" "core:item/block/$block"
}

function ItemDef([string]$Path, [string]$Display, [string]$Category, [string]$Stack, [string]$Visual, [string]$Gameplay, [string]$Tags) {
    Write-Text (Join-Path $PackRoot $Path) @"
ItemDef(
    display_name: "$Display",
    category: "$Category",
    stack_size: $Stack,
    visual: $Visual,
    gameplay: $Gameplay,
    tags: $Tags,
)
"@
}

foreach ($block in @(
    "grass", "dirt", "coarse_dirt", "podzol", "stone", "andesite", "deepslate", "red_sand", "snow", "powder_snow",
    "oak_log", "birch_log", "spruce_log", "acacia_log", "dark_oak_log", "jungle_log",
    "oak_leaves", "birch_leaves", "spruce_leaves", "acacia_leaves", "dark_oak_leaves", "jungle_leaves",
    "dandelion", "short_grass", "coal_ore", "iron_ore"
)) {
    $blockRef = if ($block -match "_ore$") { "core:block/ores/$block" } elseif ($block -match "_log$") { "core:block/natural/logs/$block" } elseif ($block -match "_leaves$") { "core:block/natural/leaves/$block" } elseif ($block -in @("dandelion", "short_grass")) { "core:block/flora/$block" } else { "core:block/terrain/$block" }
    ItemDef "defs/items/blocks/$block.item.ron" "$((Get-Culture).TextInfo.ToTitleCase($block.Replace('_',' ')))" "block" "99" "(inventory_icon: ""core:icon/blocks/$block"", world_model: BlockItem(""$blockRef""))" "PlaceBlock(""$blockRef"")" '["core:tag/item/block"]'
}
ItemDef "defs/items/resources/coal.item.ron" "Coal" "resource" "99" '(inventory_icon: "core:icon/items/resources/coal", world_model: "core:voxel/items/resources/mineral/deposit/coal_1")' "CraftingIngredient((fuel_value: Some(1600)))" '["core:tag/item/resource", "core:tag/resource/coal", "core:tag/item/fuel"]'
ItemDef "defs/items/resources/iron_ore_chunk.item.ron" "Iron Ore Chunk" "resource" "99" '(inventory_icon: "core:icon/items/resources/iron_ore_chunk", world_model: "core:voxel/items/resources/mineral/deposit/iron_1")' "CraftingIngredient((smelts_to: Some(\"core:item/resource/iron_ingot\")))" '["core:tag/item/resource", "core:tag/resource/iron"]'
ItemDef "defs/items/resources/iron_ingot.item.ron" "Iron Ingot" "resource" "99" '(inventory_icon: "core:icon/items/resources/iron_ingot", world_model: None)' "CraftingIngredient(())" '["core:tag/item/resource", "core:tag/material/iron"]'
ItemDef "defs/items/resources/plant_fiber.item.ron" "Plant Fiber" "resource" "99" '(inventory_icon: "core:icon/items/resources/plant_fiber", world_model: "core:voxel/items/crafting/plant_fiber")' "CraftingIngredient(())" '["core:tag/item/resource", "core:tag/material/fiber"]'
ItemDef "defs/items/tools/stone_pickaxe.item.ron" "Stone Pickaxe" "tool" "1" '(inventory_icon: "core:icon/items/tools/stone_pickaxe", world_model: "core:voxel/equipment/tools/pickaxe_stone", hand_model: "core:voxel/equipment/tools/pickaxe_stone")' "Tool((tool_tags: [\"core:tag/item/tool/pickaxe\"], tier: 1, mining_speed: 4.0, durability: 132))" '["core:tag/item/tool", "core:tag/item/tool/pickaxe"]'
ItemDef "defs/items/weapons/wood_bow.item.ron" "Wood Bow" "weapon" "1" '(inventory_icon: "core:icon/items/weapons/wood_bow", world_model: "core:voxel/equipment/weapons/bow/bow/wood", hand_model: "core:voxel/equipment/weapons/bow/bow/wood")' "Weapon((class: bow, damage: 5.0, attack_speed: 0.8, durability: 180, projectile: \"core:projectile/arrow\"))" '["core:tag/item/weapon", "core:tag/item/weapon/bow"]'
ItemDef "defs/items/food/apple.item.ron" "Apple" "food" "16" '(inventory_icon: "core:icon/items/food/apple", world_model: "core:voxel/items/food/fruit/apple")' "Food((nutrition: 4, saturation: 1.2, eat_seconds: 1.0))" '["core:tag/item/food", "core:tag/item/plant_food"]'
ItemDef "defs/items/consumables/health_potion.item.ron" "Health Potion" "consumable" "8" '(inventory_icon: "core:icon/items/consumables/health_potion", world_model: "core:voxel/items/consumables/potions/potion_red")' "Consumable((effect: \"core:effect/heal\", magnitude: 8.0, use_seconds: 1.2))" '["core:tag/item/consumable", "core:tag/item/potion"]'

SimpleLoot "defs/loot/entities/rabbit.loot.ron" "core:item/resource/plant_fiber"

Write-Text (Join-Path $PackRoot "defs/entities/player/player.entity.ron") @"
EntityDef(
    display_name: "Player",
    category: player,
    body: CharacterBody((skeleton: "core:skeleton/humanoid_player", model_root: "core:voxel/characters/humanoids")),
    gameplay: (
        health: 20,
        movement: "core:movement/player_planet_gravity",
        inventory: "core:inventory/player_survival",
        interaction_reach_voxels: 7.0,
    ),
    tags: ["core:tag/entity/player"],
)
"@

Write-Text (Join-Path $PackRoot "defs/entities/animals/rabbit.entity.ron") @"
EntityDef(
    display_name: "Rabbit",
    category: animal,
    body: ModularVoxelBody((
        skeleton: "core:skeleton/quadruped_small",
        model_root: "core:voxel/creatures/needs_review/rabbit",
    )),
    gameplay: (
        health: 6,
        movement: "core:movement/quadruped_small",
        behavior: "core:behavior/passive_grazer",
        drops: "core:loot/entities/rabbit",
    ),
    spawn: "core:spawn/rabbit",
    tags: ["core:tag/entity/animal", "core:tag/entity/passive"],
)
"@

foreach ($skel in @("humanoid_player","biped_small","biped_large","quadruped_small","quadruped_medium","quadruped_large","bird_medium","dragon","arthropod","fish","golem")) {
    Write-Text (Join-Path $PackRoot "defs/skeletons/$skel.skeleton.ron") @"
SkeletonDef(
    display_name: "$($skel.Replace('_', ' '))",
    coordinate_space: voxel_model_local,
    scale: 1.0,
    slots: [],
    animation_sets: [],
    notes: "Clean replacement for imported body manifests; slots will be expanded when animation code is wired.",
)
"@
}

Write-Text (Join-Path $PackRoot "defs/worldgen/planet_profiles/default.profile.ron") @"
PlanetProfileDef(
    display_name: "Default Living Planet",
    seed: 2,
    shape: SphericalVoxelPlanet((
        resolution: 10000,
        surface_layer: 5000,
        voxel_size_meters: 0.5,
        edge_rounding_radius_voxels: 0.16,
        core_layers: 6,
        sea_level_offset: -6,
        max_terrain_offset: 180,
    )),
    climate: "core:climate/earthlike",
    biome_set: "core:biome_set/earthlike",
    terrain_layers: "core:terrain_layers/earth_crust",
    caves: ["core:cave/default"],
    ores: ["core:ore/coal", "core:ore/iron"],
    vegetation: ["core:vegetation/oak_tree", "core:vegetation/birch_tree", "core:vegetation/spruce_tree"],
    structures: ["core:structure/small_ruin"],
    spawns: ["core:spawn/rabbit"],
    visual_details: ["core:visual_detail/flower_scatter"],
    streaming: (
        near_voxel_lod_radius: 12,
        far_surface_lod_radius: 192,
        upload_budget_chunks_per_frame: 8,
    ),
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/climate_profiles/earthlike.climate.ron") @"
ClimateProfileDef(
    display_name: "Earthlike",
    fields: (
        temperature: "core:field/temperature_jitter",
        humidity: "core:field/humidity",
        continentality: "core:field/continentality",
        erosion: "core:field/erosion",
        weirdness: "core:field/weirdness",
    ),
    atmosphere: (
        fog_color: (0.64, 0.76, 0.92),
        horizon_fog_density: 0.018,
        sky_scatter_strength: 0.72,
    ),
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/biome_sets/earthlike.biome_set.ron") @"
BiomeSetDef(
    display_name: "Earthlike Planet Biomes",
    selection: ClimateMap((
        entries: [
            (biome: "core:biome/temperate_forest", temperature: (0.30, 0.72), humidity: (0.35, 0.85), weight: 1.0),
            (biome: "core:biome/plains", temperature: (0.35, 0.80), humidity: (0.20, 0.55), weight: 0.85),
            (biome: "core:biome/desert", temperature: (0.65, 1.00), humidity: (0.00, 0.28), weight: 0.75),
            (biome: "core:biome/snowy_taiga", temperature: (0.00, 0.32), humidity: (0.35, 0.90), weight: 0.8),
            (biome: "core:biome/alpine", temperature: (0.00, 0.48), humidity: (0.15, 0.80), elevation: (0.70, 1.00), weight: 0.9),
        ],
    )),
)
"@

function Biome([string]$Name, [string]$Display, [string]$Top, [string]$Under, [string]$Tags, [string]$Grass, [string]$Foliage) {
    Write-Text (Join-Path $PackRoot "defs/worldgen/biomes/$Name.biome.ron") @"
BiomeDef(
    display_name: "$Display",
    surface: (
        top: "$Top",
        under: "$Under",
        depth_voxels: (2, 6),
        slope_override: Some((above_degrees: 42, top: "core:block/terrain/stone")),
    ),
    terrain: (
        base_height: 0.02,
        amplitude: 0.40,
        flatness: 0.46,
        hill_field: "core:field/rolling_hills",
        ridge_field: None,
        terrace_strength: 0.03,
    ),
    palette: (
        grass: $Grass,
        foliage: $Foliage,
        fog_bias: (0.0, 0.0, 0.0),
    ),
    placement: (
        vegetation_tags: $Tags,
        fauna_tags: ["core:tag/fauna/small_herbivore"],
        structure_tags: ["core:tag/structure/small"],
    ),
    tags: $Tags,
)
"@
}

Biome "temperate_forest" "Temperate Forest" "core:block/terrain/grass" "core:block/terrain/dirt" '["core:tag/biome/temperate", "core:tag/biome/forest"]' "(0.36, 0.66, 0.28)" "(0.22, 0.52, 0.22)"
Biome "plains" "Open Plains" "core:block/terrain/grass" "core:block/terrain/dirt" '["core:tag/biome/temperate", "core:tag/biome/plains"]' "(0.46, 0.72, 0.30)" "(0.30, 0.58, 0.24)"
Biome "desert" "Red Sand Desert" "core:block/terrain/red_sand" "core:block/terrain/red_sand" '["core:tag/biome/desert", "core:tag/biome/dry"]' "(0.70, 0.58, 0.25)" "(0.52, 0.46, 0.20)"
Biome "snowy_taiga" "Snowy Taiga" "core:block/terrain/snow" "core:block/terrain/dirt" '["core:tag/biome/cold", "core:tag/biome/forest"]' "(0.62, 0.72, 0.62)" "(0.36, 0.50, 0.40)"
Biome "alpine" "Alpine Heights" "core:block/terrain/stone" "core:block/terrain/deepslate" '["core:tag/biome/cold", "core:tag/biome/mountain"]' "(0.50, 0.62, 0.45)" "(0.32, 0.44, 0.34)"

foreach ($field in @("rolling_hills","mountain_ridges","soft_hills","plateau_ridges","dune_waves","warp_large","humidity","temperature_jitter","continentality","erosion","weirdness","coal_veins","iron_veins","oak_scatter","birch_scatter","spruce_scatter","jungle_scatter","acacia_scatter","dark_oak_scatter","flower_noise","cave_worms","cave_chambers")) {
    Write-Text (Join-Path $PackRoot "defs/worldgen/noise_fields/$field.field.ron") @"
NoiseFieldDef(
    kind: perlin,
    frequency: 1.0,
    amplitude: 1.0,
    octaves: 5,
    persistence: 0.50,
    lacunarity: 2.05,
    seed_salt: "$field",
    domain_warp: Some((field: "core:field/warp_large", strength: 0.12)),
)
"@
}

Write-Text (Join-Path $PackRoot "defs/worldgen/terrain_layers/earth_crust.terrain_layers.ron") @"
TerrainLayerSetDef(
    display_name: "Earth Crust",
    layers: [
        (range: surface, block: "core:block/terrain/grass"),
        (range: subsurface, block: "core:block/terrain/dirt", thickness: (2, 6)),
        (range: crust, block: "core:block/terrain/stone"),
        (range: deep_crust, block: "core:block/terrain/deepslate"),
        (range: core, block: "core:block/terrain/core"),
    ],
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/ores/coal.ore.ron") @"
OreDistributionDef(
    block: "core:block/ores/coal_ore",
    replace: ["core:block/terrain/stone", "core:block/terrain/deepslate", "core:block/terrain/andesite"],
    depth_voxels: (12, 180),
    density: 0.065,
    vein_size: (4, 18),
    field: "core:field/coal_veins",
    biome_tags: ["core:tag/biome/any"],
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/ores/iron.ore.ron") @"
OreDistributionDef(
    block: "core:block/ores/iron_ore",
    replace: ["core:block/terrain/stone", "core:block/terrain/deepslate", "core:block/terrain/andesite"],
    depth_voxels: (24, 220),
    density: 0.045,
    vein_size: (3, 12),
    field: "core:field/iron_veins",
    biome_tags: ["core:tag/biome/any"],
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/caves/default.cave.ron") @"
CaveSystemDef(
    display_name: "Default Cave Network",
    fields: ["core:field/cave_worms", "core:field/cave_chambers"],
    carve: (
        min_depth_voxels: 18,
        max_depth_voxels: 420,
        tunnel_radius: (2.0, 7.0),
        chamber_radius: (6.0, 24.0),
        air_block: "core:block/air/air",
    ),
)
"@

foreach ($tree in @("oak","birch","spruce","jungle","acacia","dark_oak")) {
    Write-Text (Join-Path $PackRoot "defs/worldgen/vegetation/$tree`_tree.vegetation.ron") @"
VegetationPlacementDef(
    display_name: "$((Get-Culture).TextInfo.ToTitleCase($tree.Replace('_', ' '))) Tree",
    kind: procedural_tree,
    placement: (
        allowed_surface_tags: ["core:tag/block/soil"],
        biome_tags: ["core:tag/biome/forest"],
        density: 0.03,
        slope_max_degrees: 34,
        scatter_field: "core:field/$tree`_scatter",
    ),
    stamp: (
        trunk: "core:block/natural/logs/$tree`_log",
        leaves: "core:block/natural/leaves/$tree`_leaves",
        height: (5, 9),
        canopy_radius: (2, 4),
        canopy_density: 0.65,
    ),
)
"@
}

Write-Text (Join-Path $PackRoot "defs/worldgen/spawns/rabbit.spawn.ron") @"
SpawnRuleDef(
    entity: "core:entity/animals/rabbit",
    biome_tags: ["core:tag/biome/plains", "core:tag/biome/forest"],
    density: 0.025,
    group_size: (1, 3),
    light: daylight,
    surface_tags: ["core:tag/block/soil"],
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/structures/small_ruin.structure.ron") @"
StructureDistributionDef(
    display_name: "Small Ruin",
    structure: "core:structure_template/small_ruin",
    biome_tags: ["core:tag/biome/temperate", "core:tag/biome/plains"],
    spacing_voxels: (450, 900),
    slope_max_degrees: 18,
    rarity: uncommon,
)
"@

Write-Text (Join-Path $PackRoot "defs/worldgen/visual_details/flower_scatter.visual_detail.ron") @"
VisualDetailDef(
    display_name: "Flower Scatter",
    models: ["core:voxel/vegetation/flowers"],
    biome_tags: ["core:tag/biome/temperate", "core:tag/biome/plains"],
    density: 0.12,
    render: instanced_voxel_prop,
    collision: none,
)
"@

Write-Text (Join-Path $PackRoot "defs/props/core_props.prop_collection.ron") @"
PropCollectionDef(
    props: [
        (id_hint: "camp_tent", model: "core:voxel/props/interactables/camps/tent", collision: voxel_bounds, interaction: shelter),
        (id_hint: "street_lamp", model: "core:voxel/props/lights/street_lamp", collision: voxel_bounds, interaction: light_source),
        (id_hint: "bell", model: "core:voxel/props/interactables/bells/bell", collision: voxel_bounds, interaction: signal),
        (id_hint: "ladder", model: "core:voxel/props/structure_parts/ladders/ladder", collision: ladder, interaction: climbable),
    ],
)
"@

Write-Text (Join-Path $PackRoot "defs/vegetation/core_surface_details.vegetation.ron") @"
VegetationDef(
    groups: [
        (id_hint: "grass", models: ["core:voxel/vegetation/grass"], placement: surface_only, render: instanced_voxel_prop),
        (id_hint: "flowers", models: ["core:voxel/vegetation/flowers"], placement: surface_only, render: instanced_voxel_prop),
        (id_hint: "cave_flora", models: ["core:voxel/vegetation/cave"], placement: cave_surface_or_ceiling, render: instanced_voxel_prop),
        (id_hint: "underwater", models: ["core:voxel/vegetation/underwater"], placement: underwater_surface, render: instanced_voxel_prop),
    ],
)
"@

Write-Text (Join-Path $PackRoot "defs/tags/blocks/core_block_tags.ron") @"
TagSetDef(
    tags: [
        (id_hint: "terrain", values: ["core:block/terrain/grass", "core:block/terrain/dirt", "core:block/terrain/stone"]),
        (id_hint: "soil", values: ["core:block/terrain/grass", "core:block/terrain/dirt", "core:block/terrain/coarse_dirt", "core:block/terrain/podzol"]),
        (id_hint: "stone", values: ["core:block/terrain/stone", "core:block/terrain/andesite", "core:block/terrain/deepslate"]),
        (id_hint: "ore", values: ["core:block/ores/coal_ore", "core:block/ores/iron_ore"]),
    ],
)
"@

Normalize-BareNumericFiles (Join-Path $PackRoot "media/voxel")

$voxels = Get-ChildItem -LiteralPath (Join-Path $PackRoot "media/voxel") -Recurse -File -Filter *.vox | Sort-Object FullName
$voxelEntries = $voxels | ForEach-Object {
    "        (id: ""$(VoxelIdFromPath $_.FullName)"", path: ""$(To-PackPath $_.FullName)"", kind: voxel_model),"
}
Write-Text (Join-Path $PackRoot "generated/registries/voxel_assets.ron") @"
VoxelAssetRegistry(
    generated_from: "media/voxel",
    asset_count: $($voxels.Count),
    assets: [
$($voxelEntries -join "`n")
    ],
)
"@

Remove-TreeSafe (Join-Path $PackRoot "legacy_imports")
Remove-TreeSafe (Join-Path $PackRoot "blocks")
Remove-TreeSafe (Join-Path $PackRoot "worldgen")
Remove-TreeSafe (Join-Path $PackRoot "items")
Remove-TreeSafe (Join-Path $PackRoot "voxel")
$packToml = Join-Path $PackRoot "pack.toml"
if (Test-Path -LiteralPath (Assert-InPack $packToml)) {
    Remove-Item -LiteralPath (Assert-InPack $packToml) -Force
}

Remove-EmptyDirs $PackRoot

Write-Output "Rebuilt core pack data architecture."
