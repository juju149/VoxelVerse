# Core Content Architecture

VoxelVerse content is organized as authored definitions plus media assets.
Runtime code will be updated later to compile this layout into compact
registries.

## Pack Root

```text
assets/packs/core/
  pack.ron
  README.md
  defs/
  media/
  generated/
```

## Definitions

```text
defs/
  blocks/
  materials/
  items/
  entities/
  props/
  vegetation/
  loot/
  recipes/
  skeletons/
  sounds/
  tags/
  worldgen/
```

Rules:

- Raw ids are derived from namespace and path.
- Blocks reference materials.
- Materials reference textures.
- Items reference icons, voxel models or block models.
- Entities reference skeletons, body definitions, loot and spawn rules.
- Worldgen references block, biome, ore, cave, vegetation, structure and spawn ids.
- Media files never define gameplay.

## Media

```text
media/
  textures/
  voxel/
```

`media/voxel/` is organized by role:

```text
characters/
creatures/
equipment/
items/
props/
vegetation/
projectiles/
effects/
debug/
```

## Generated Data

```text
generated/
  registries/
```

Generated registries are derived data. They can be rebuilt and should not be
used as the design source of truth.

## Naming

- Lowercase `snake_case`.
- No spaces.
- No hyphens.
- No bare numeric filenames.
- Use contextual variants such as `flower_blue_00.vox`, not `00.vox`.

## Current Validation

Run:

```powershell
powershell -ExecutionPolicy Bypass -File tools\validate_content.ps1
```

The validator checks:

- No obsolete content roots remain.
- No empty directories remain.
- Content filenames are valid.
- `core:*` references resolve to a definition or media asset.
- The voxel asset registry count matches `media/voxel`.
