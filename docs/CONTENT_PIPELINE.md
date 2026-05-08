# Content Pipeline

The permanent content flow is:

```text
assets/packs/* raw files
-> vv-pack-loader
-> vv-content-schema raw structs
-> vv-pack-compiler validation and reference resolution
-> compact runtime registries
-> game runtime, worldgen, meshing, renderer
```

## Identity

Content identity is path-as-identity:

```text
assets/packs/core/blocks/dirt.ron -> core:dirt
```

Raw `.ron` files should not repeat their own ID when the namespace and path already define it.

## Pack Layout

The core pack currently uses:

```text
assets/packs/core/
  blocks/
  items/
  textures/
  texture_recipes/
  worldgen/
  generated/
```

`worldgen/` contains the current procedural planet data: planets, fields, climates, biome sets, biomes, terrain layers, ores, caves, vegetation, structures, fauna, and visual details.

## Runtime Rules

- Runtime code does not parse raw pack files directly.
- Voxels store compact `VoxelId`, never strings.
- Missing required pack roots and missing `blocks/` directories are hard errors.
- Optional future categories may be absent until their systems exist.
- Renderer PNG assets remain final assets under `textures/`; generated or intermediate outputs belong under `generated/` or future tool-specific folders.

