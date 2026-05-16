# VoxelVerse V1 Content And Modding Contract

This document defines the V1 content model.
It supersedes any older folder split that separates blocks, items, recipes and loot into unrelated definitions.

## Founding rule

One gameplay object stays together. World placement rules stay separate.

An object is a thing the player can know, hold, place, use, mine, craft, eat, wear, fight with, store in, or interact with.

A world rule is a rule for where, when, how often, or under which conditions objects appear.

## Canonical pack layout

```text
assets/packs/<namespace>/
  pack.ron
  README.md
  defs/
    objects/**/*.object.ron
    voxel_models/**/*.voxel_model.ron
    skeletons/**/*.skeleton.ron
    sounds/**/*.sound.ron
    world/
      planets/**/*.planet.ron
      noise/**/*.field.ron
      climate/**/*.climate.ron
      biome_sets/**/*.biome_set.ron
      biomes/**/*.biome.ron
      terrain_layers/**/*.terrain_layers.ron
      caves/**/*.cave.ron
      ores/**/*.ore.ron
      vegetation/**/*.vegetation.ron
      vegetation/**/*.prop_scatter.ron
      structures/**/*.structure.ron
      fauna/**/*.fauna.ron
  media/
    textures/**/*.png
    voxel/**/*.vox
    audio/**/*
  generated/
    registries/**
    reports/**
  render/
    shaders/**
```

## Identity

Identity is derived from namespace and path.

```text
defs/objects/terrain/stone.object.ron
-> core:object/terrain/stone

defs/world/biomes/forest/birch_forest.biome.ron
-> core:biome/forest/birch_forest
```

The file must not repeat its own id.

## Reference rules

All references must be explicit.

Valid:

```ron
"core:object/terrain/stone"
"core:biome/forest/birch_forest"
"#core:tag/tool/pickaxe"
"core:voxel_model/items/wooden_pickaxe"
```

Forbidden:

```ron
"stone"
"pickaxe"
"#station.construction"
"../textures/stone.png"
```

## Object file anatomy

A `.object.ron` may contain these sections:

```ron
(
  format_version: 1,
  name: "Stone",
  description: Some("Bloc naturel robuste."),
  tags: ["#core:tag/block/natural", "#core:tag/material/stone"],

  block: Some((...)),
  item: Some((...)),
  mining: Some((...)),
  tool: None,
  weapon: None,
  food: None,
  effect: None,
  station: None,
  storage: None,
  light: None,
  fuel: None,
  entity: None,
  loot: None,
  recipes: [],
)
```

A block that can be held must have both `block` and `item`.
A tool must have `item` and `tool`.
A station block must have `block`, `item`, `station`, and usually `storage` or processing slots.

## Forbidden content patterns

Forbidden:

- block defined in one file and its item in another file;
- recipe output defined far away from the produced object;
- loot rule hidden in runtime code;
- worldgen rule inside object file;
- direct media path from gameplay references;
- unknown fields accepted silently;
- old file layout kept as compatibility;
- `legacy`, `old`, `new`, `v2` fields;
- duplicate tag spelling.

## Tags

Tags are semantic contracts.

Examples:

```text
#core:tag/block/natural
#core:tag/block/building
#core:tag/material/wood
#core:tag/tool/pickaxe
#core:tag/station/construction
#core:tag/biome/spawns/common
```

Tags must be declared or derived by compiler in one canonical place.
Pack Doctor must detect:

- unused tags;
- missing tag namespace;
- invalid tag domain;
- duplicate meaning;
- tag used by recipe but no station owns it.

## Recipes

Recipes live in the object they produce.

```ron
recipes: [(
  station: Some("#core:tag/station/construction"),
  kind: shaped((
    pattern: ["SSS", " F ", " F "],
    legend: { "S": "core:object/resources/stone", "F": "core:object/resources/stick" },
  )),
  output: (item: "core:object/tools/stone_pickaxe", count: 1),
  group: Some("tools"),
)]
```

Rules:

- output must match the enclosing object;
- ingredients must resolve;
- station tag must resolve;
- recipe must be reachable;
- recipe must not duplicate another recipe with same station and inputs;
- recipe cannot produce hidden debug items unless item category is `debug`.

## Mining and drops

Mining is a contract between:

- block hardness;
- required tool tag;
- required tool tier;
- strike damage;
- persistent block damage;
- drops.

V1 mining must support:

- hit-based damage, not invisible timer mining;
- persistent cracks stored in world state;
- tool speed and tier;
- no drops when wrong tool tier if configured;
- clear UI/audio feedback when blocked;
- block break producing inventory drops or world drops.

A breakable block must explicitly define drops or explicitly define empty drops.
Silent assumptions are pack errors.

## Voxel models

`.vox` files are raw media. Gameplay references voxel model manifests.

```text
defs/voxel_models/items/wooden_pickaxe.voxel_model.ron
media/voxel/items/wooden_pickaxe.vox
```

V1 manifest must define:

- source media path;
- usage: item, prop, entity, hand, structure;
- scale;
- pivot;
- forward/up axes;
- bounds;
- collision mode;
- material strategy;
- LOD behavior if relevant.

## World rules

World rules live under `defs/world` and reference objects.

Examples:

- biome says which terrain layers and vegetation rules are active;
- ore rule says where `core:object/ores/copper_ore` appears;
- vegetation rule says where tree objects or props appear;
- fauna rule says where animal entity objects spawn;
- structure rule says where a structure appears.

Objects do not know where they spawn. The world knows that.

## Pack Doctor V1 requirements

Pack Doctor must fail on:

- unresolved refs;
- unused authored object unless explicitly allowed;
- duplicate display names inside same category;
- invalid file suffix;
- unknown RON fields;
- hidden item with recipe output;
- recipe cycle with no natural source;
- item with no source and no debug category;
- block with missing texture;
- texture file missing PBR-lite map if required;
- station tag without station object;
- station object with no useful recipe and no storage behavior;
- worldgen rule referencing missing biome or object;
- media file referenced directly by gameplay.

## Modder experience target

A modder adds a block by:

1. creating one `.object.ron`;
2. adding textures or pointing to existing texture keys;
3. optionally adding world rule;
4. running Pack Doctor;
5. reading precise errors;
6. launching game.

No engine code should be needed.
