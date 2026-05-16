# VoxelVerse Pack V1

This is the official authored pack contract for VoxelVerse V1.

Founding rule:

- If a modder can easily make a silent mistake, the schema is wrong.
- If the compiler has to guess, the pack is wrong.
- If the runtime has to repair content, the architecture is wrong.

## Layout

```text
assets/packs/<namespace>/
  pack.ron
  README.md
  defs/
    objects/**/*.object.ron
    voxel_models/**/*.voxel_model.ron
    skeletons/**/*.skeleton.ron
    world/
      planets/**/*.profile.ron
      climate/**/*.climate.ron
      noise/**/*.field.ron
      biome_sets/**/*.biome_set.ron
      biomes/**/*.biome.ron
      terrain_layers/**/*.terrain_layers.ron
      caves/**/*.cave.ron
      ores/**/*.ore.ron
      vegetation/**/*.vegetation.ron
      vegetation/**/*.prop_scatter.ron
      structures/**/*.structure.ron
  media/
    textures/**/*.png
    voxel/**/*.vox
  generated/
    registries/
    reports/
  render/
```

`defs/` is human-authored truth. `media/` is raw files. `generated/` is compiler output. Runtime code must load compiled registries, not raw `.ron` source.

## Identity

Identity is path-derived inside the namespace.

`defs/objects/terrain/stone.object.ron` is `core:object/terrain/stone`.

Files must not repeat their own id. References must be fully qualified:

- valid object ref: `core:object/terrain/stone`
- invalid object ref: `stone`
- valid tag ref: `#core:tag/material/wood`
- invalid tag ref: `wood` or `#station.construction`

## Objects

One `.object.ron` file defines one gameplay identity. It may contain sections such as `block`, `item`, `tool`, `weapon`, `food`, `station`, `storage`, `light`, `fuel`, `entity`, `loot`, and `recipes`.

Objects must not contain worldgen rules. Spawn, ore placement, vegetation placement, biome selection and structure placement live under `defs/world/`.

Unknown fields are errors. Legacy fields are errors. `recipe:` is forbidden; only `recipes:` is accepted.

## Items

Every visible inventory item must declare:

```ron
item: (
    stack: 64,
    category: resource,
    visible_in_inventory: true,
    inventory_icon: texture("core:texture/items/resources/coal"),
)
```

Valid categories are `block`, `resource`, `food`, `tool`, `weapon`, `armor`, `station`, `utility`, `material`, `quest`, and `debug`.

Valid icon strategies are:

- `texture("core:texture/items/...")`
- `block`
- `voxel_model("core:voxel_model/...")`
- `auto_generated`

The old `icon:` field is forbidden.

## Voxel Models

Gameplay never references `media/voxel/**/*.vox` directly. It references a manifest:

```ron
world_model: "core:voxel_model/items/apple"
```

The manifest lives at `defs/voxel_models/items/apple.voxel_model.ron` and declares source media, usage, scale, pivot, orientation, bounds, collision, render behavior and runtime uses.

## Recipes

Recipes are embedded in the object they produce. Each recipe output must be the enclosing object item.

```ron
recipes: [(
    station: "#core:tag/station/construction",
    kind: shapeless((
        ingredients: ["core:object/trees/oak_log", "core:object/resources/plant_fiber"],
    )),
    output: (item: "core:object/tools/wooden_axe", count: 1),
)]
```

Ingredients and outputs must be full object references. Missing ingredients are errors. Unresolved ingredients are compiler errors, not ignored recipes.

## Stations

A station is one object with a `station` section and exactly one owner for each station tag:

```ron
tags: ["#core:tag/station/construction", "#core:tag/block/furniture"]
station: (type: workbench)
```

Recipes target stations through strict tags, for example `#core:tag/station/furnace`.

## Blocks And Mining

Blocks define visuals and physical behavior. Mining rules are explicit. A cassable block must either declare drops or intentionally declare `drops: []`. Silent self-drop fallback is not a V1 authoring pattern for new content.

Negative hardness for unbreakable blocks is legacy and should be migrated to an explicit durability/mining contract in the next mining schema pass.

## Worldgen

Worldgen is separate from gameplay objects. The V1 conceptual pipeline is:

1. planet
2. shape
3. climate
4. noise fields
5. terrain layers
6. water and hydrology
7. biome selection
8. biomes
9. caves
10. ores
11. vegetation
12. props
13. structures
14. animals
15. ambience
16. runtime budgets

Vegetation species, animal objects and structures are not placement rules. Placement and spawn rules belong under `defs/world/`.

## Pack Doctor

Run:

```powershell
cargo run -p vv-pack-doctor -- assets/packs/core
```

Core V1 must report `0 errors`, `0 warnings`, and score `100/100`.

Errors must say where the file is, which field is wrong, why it is dangerous, and how to fix it.

## Compiler

The compiler pipeline is:

1. read source files
2. check format
3. check paths
4. check fields
5. derive ids
6. resolve references
7. validate rules
8. normalize content
9. generate registries
10. generate atlas/previews
11. generate reports
12. expose compiled files to runtime

The runtime must not compensate for a broken pack.
