# VoxelVerse Content Architecture

This document defines the target pack layout for scalable, moddable,
data-driven content.

## Goals

- One content source of truth per concept.
- Runtime consumes compiled registries and stable ids, not raw file paths.
- Visual assets do not define gameplay.
- Raw assets are never loaded directly by hot runtime paths.
- Mod packs can add or override content without editing engine code.
- Artists, designers, modders and engine code can navigate the pack without
  knowing historical import categories.

## Target Pack Layout

```text
assets/packs/core/
  pack.ron
  README.md
  defs/
    blocks/
    materials/
    items/
    entities/
    props/
    vegetation/
    recipes/
    biomes/
    worldgen/
    loot/
    skeletons/
    animations/
    sounds/
    tags/
  media/
    textures/
    voxel/
    audio/
    icons/
    particles/
    ui/
  source/
    voxel_raw/
    texture_sources/
    references/
  generated/
    registries/
    atlases/
    mesh_cache/
    icons/
    diagnostics/
  legacy_imports/
    manifests/
    voxel_raw/
    needs_review/
    deprecated/
```

## Target Voxel Media Layout

```text
media/voxel/
  characters/
    player/
    humanoids/
  creatures/
    animals/
    monsters/
    bosses/
    aquatic/
    flying/
    insects/
    humanoids/
    needs_review/
  equipment/
    armor/
    weapons/
    tools/
    shields/
    gliders/
    accessories/
  items/
    resources/
    food/
    consumables/
    keys/
    crafting/
    loot/
    needs_review/
  props/
    crafting_stations/
    containers/
    doors/
    furniture/
    lights/
    decoration/
    traps/
    structure_parts/
    interactables/
    needs_review/
  vegetation/
    grass/
    flowers/
    bushes/
    mushrooms/
    trees/
    crops/
    cave/
    desert/
    snow/
    underwater/
    needs_review/
  projectiles/
  effects/
  debug/
```

`media/voxel/debug/` is an editor/dev bucket and must not be included in a
shipping runtime registry.

## Definition Ownership

| Concept | Source of truth | Visual reference |
| --- | --- | --- |
| Block behavior | `defs/blocks/**/*.block.ron` | Material ids |
| Block material | `defs/materials/**/*.material.ron` | Texture ids |
| Item behavior | `defs/items/**/*.item.ron` | Icon/model ids |
| Entity behavior | `defs/entities/**/*.entity.ron` | Skeleton/model ids |
| Prop behavior | `defs/props/**/*.prop.ron` | Model ids |
| Vegetation placement | `defs/vegetation/**/*.vegetation.ron` | Model ids |
| Skeleton/rig | `defs/skeletons/**/*.skeleton.ron` | Part slots only |
| Animation | `defs/animations/**/*.animation.ron` | Skeleton ids |
| Loot | `defs/loot/**/*.loot.ron` | Item ids |
| Worldgen | `defs/worldgen/**/*.ron` | Block, biome, vegetation, spawn ids |
| Media file | `media/**` | No gameplay |
| Generated registry | `generated/registries/**` | Compiled output only |

## Naming Rules

Use only:

- `snake_case`
- lowercase ASCII
- padded numeric variants: `_00`, `_01`, `_02`
- descriptive names with role context

Do not use for new files:

- spaces
- uppercase
- hyphens
- bare numeric filenames
- ambiguous names like `male.vox`, `1.vox`, `object.vox`, `misc.vox`

Examples:

```text
wolf_head.vox
wolf_torso_front.vox
wolf_leg_front_r.vox
grass_large_00.vox
flower_blue_00.vox
iron_sword_1h.vox
wooden_pickaxe.vox
chest_wood_basic.vox
```

Stable ids use readable domains:

```text
core:block/terrain/grass
core:material/terrain/grass_top
core:item/weapon/iron_sword
core:entity/animal/wolf
core:prop/container/chest_wood
core:vegetation/grass_short
core:voxel/creatures/animals/wolf/head
```

Physical paths may change; ids should remain stable through registry
generation.

## Blocks And Materials

Terrain and construction blocks are not `.vox` models by default. They are
cube/cross-plane visuals using materials and texture atlases.

Target layout:

```text
defs/blocks/terrain/grass.block.ron
defs/materials/terrain/grass_top.material.ron
defs/materials/terrain/grass_side.material.ron
media/textures/blocks/grass/grass_top_albedo.png
media/textures/blocks/grass/grass_top_normal.png
media/textures/blocks/grass/grass_top_roughness.png
```

Target block example:

```ron
BlockDef(
    display_name: "Grass Block",
    category: "terrain",
    solid: true,
    opaque: true,
    hardness: 0.6,
    collision: full_cube,
    visual: CubeMaterial((
        top: "core:material/terrain/grass_top",
        sides: "core:material/terrain/grass_side",
        bottom: "core:material/terrain/dirt",
    )),
    gameplay: (
        tool: Some("core:tag/tool/shovel"),
        drops: "core:loot/blocks/grass",
        footstep_sound: "core:sound/step/grass",
    ),
)
```

Target material example:

```ron
MaterialDef(
    albedo: "core:texture/blocks/grass/grass_top_albedo",
    normal: Some("core:texture/blocks/grass/grass_top_normal"),
    roughness: Some("core:texture/blocks/grass/grass_top_roughness"),
    tint: Some(BiomeTint("grass")),
    render: opaque,
)
```

Current code still uses inline block texture sets. The material schema should
be introduced before moving active block texture references.

## Items And Equipment

Item definitions live in `defs/items/`. Visual `.vox` files live in
`media/voxel/items/` or `media/voxel/equipment/`.

Rule:

- A sword item owns gameplay in `defs/items/weapons/`.
- Its model lives in `media/voxel/equipment/weapons/`.
- Inventory icons live in `media/icons/items/`.

Target item example:

```ron
ItemDef(
    display_name: "Iron Sword",
    category: weapon,
    stack_size: 1,
    visual: (
        inventory_icon: "core:icon/items/weapons/iron_sword",
        world_model: "core:voxel/equipment/weapons/swords/iron_sword",
        hand_model: "core:voxel/equipment/weapons/swords/iron_sword",
    ),
    gameplay: Weapon((
        damage: 7,
        attack_speed: 1.2,
        durability: 240,
        tags: ["core:tag/item/sword", "core:tag/material/iron"],
    )),
)
```

## Entities And Skeletons

Living content is not a generic `npc/` bucket.

Definitions:

- `defs/entities/player/`
- `defs/entities/animals/`
- `defs/entities/monsters/`
- `defs/entities/bosses/`
- `defs/entities/aquatic/`
- `defs/entities/flying/`
- `defs/entities/insects/`
- `defs/entities/humanoids/`

Voxel media:

- `media/voxel/creatures/<category>/...`
- `media/voxel/characters/player/...`
- `media/voxel/characters/humanoids/...`

Skeleton definitions replace historical `*_central_manifest.ron` and
`*_lateral_manifest.ron` once converted.

Target skeleton files:

```text
defs/skeletons/biped_small.skeleton.ron
defs/skeletons/biped_large.skeleton.ron
defs/skeletons/quadruped_small.skeleton.ron
defs/skeletons/quadruped_medium.skeleton.ron
defs/skeletons/quadruped_large.skeleton.ron
defs/skeletons/bird_medium.skeleton.ron
defs/skeletons/dragon.skeleton.ron
defs/skeletons/arthropod.skeleton.ron
defs/skeletons/fish.skeleton.ron
defs/skeletons/golem.skeleton.ron
```

Legacy manifests stay in `legacy_imports/manifests/` until each family is
converted into explicit skeleton/entity defs.

## Props

Props are static or interactive objects that are not terrain blocks and not
living entities.

Examples:

| Visual role | Target media |
| --- | --- |
| Chest | `media/voxel/props/containers/` |
| Chair/table | `media/voxel/props/furniture/` |
| Anvil/workbench | `media/voxel/props/crafting_stations/` |
| Door/window | `media/voxel/props/doors/` |
| Lantern/campfire | `media/voxel/props/lights/` |
| Ladder/fence/bars | `media/voxel/props/structure_parts/` |
| Training dummy | `media/voxel/props/interactables/` |

Interactive props require `PropDef`.

## Vegetation

Vegetation models live in `media/voxel/vegetation/` and placement lives in
`defs/vegetation/` or worldgen spawn/distribution definitions.

Target definition example:

```ron
VegetationDef(
    display_name: "Short Grass",
    models: [
        "core:voxel/vegetation/grass/grass_short_00",
        "core:voxel/vegetation/grass/grass_short_01",
        "core:voxel/vegetation/grass/grass_short_02",
    ],
    placement: SurfaceOnly((
        allowed_blocks: ["core:block/terrain/grass"],
        density: 0.45,
        slope_max_degrees: 38,
        biome_tags: ["core:tag/biome/temperate", "core:tag/biome/plains"],
    )),
    render: instanced_voxel_prop,
    collision: none,
)
```

## Generated Outputs

Generated files are not hand-authored content:

```text
generated/registries/
generated/atlases/
generated/mesh_cache/
generated/icons/
generated/diagnostics/
```

Generated outputs may be deleted and regenerated by tools, but raw authored
assets must not be silently overwritten.

## Migration Policy

1. Create target directories.
2. Generate an exhaustive old -> new map.
3. Dry-run every move.
4. Resolve collisions.
5. Apply moves with `git mv` when Git is available.
6. Quarantine uncertain files in `legacy_imports/needs_review/`.
7. Keep manifests in `legacy_imports/manifests/`.
8. Convert manifests into first-class defs in later schema work.
9. Update Rust loaders only when active defs move from old paths to `defs/`.
10. Validate references before removing any legacy import.

No mass deletion is allowed during this migration.
