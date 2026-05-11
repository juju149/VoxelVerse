# VoxelVerse Studio Architecture Report

This report reflects the current content architecture. Older drafts that
referenced flat pack roots or runtime usage of raw voxel imports are obsolete.

## Current Pack Contract

The built-in pack is rooted at:

```txt
assets/packs/core/
```

Runtime-facing content is organized as:

```txt
pack.ron
defs/
media/
generated/
source/
```

Identity is path-derived. Data files do not repeat their own stable ID; the
loader derives IDs from pack namespace and path, for example:

```txt
defs/blocks/terrain/grass.block.ron -> core:block/terrain/grass
defs/materials/blocks/grass_block/grass_block_top.material.ron -> core:material/blocks/grass_block/grass_block_top
defs/items/blocks/grass.item.ron -> core:item/block/grass
defs/worldgen/biomes/temperate_forest.biome.ron -> core:biome/temperate_forest
```

## Data Ownership

Blocks define physical, visual, gameplay, audio, tags, and runtime role data.
Materials own texture references and render sampling. Items own inventory and
world visuals plus gameplay intent. Worldgen owns planet profiles, climate,
biome selection, terrain layers, ores, caves, vegetation, structures, spawns,
and visual details.

Voxel models live under:

```txt
assets/packs/core/media/voxel/
```

Block textures live under:

```txt
assets/packs/core/media/textures/blocks/
```

Generated registries live under:

```txt
assets/packs/core/generated/
```

## Rust Integration

Raw schemas are centralized in `crates/vv-content-schema`.

`vv-pack-loader` parses the current pack architecture recursively and derives
stable IDs from paths.

`vv-pack-compiler` resolves data into runtime registries. The runtime consumes
compiled IDs and tables, not fragile physical paths.

## Validation

Use:

```powershell
powershell -ExecutionPolicy Bypass -File tools\validate_content.ps1
powershell -ExecutionPolicy Bypass -File tools\pack_doctor.ps1
cargo test -p vv-pack-loader -p vv-pack-compiler
```

- `validate_content.ps1` is the fast filesystem-level validator.
- `pack_doctor.ps1` runs the deeper Rust validator (`vv-pack-doctor`),
  emitting `generated/reports/core_pack_report.json` and a sibling HTML
  report. See [`content_rules.md`](content_rules.md) and
  [`CONTENT_PIPELINE.md`](CONTENT_PIPELINE.md) for the rules Pack Doctor
  enforces.
- The loader test parses every core schema group. The compiler tests
  validate block materialization and texture registry loading from the
  current pack.

The future studio UI will sit on top of Pack Doctor's JSON report - see
[`voxelverse_studio_pack_dashboard.md`](voxelverse_studio_pack_dashboard.md).
