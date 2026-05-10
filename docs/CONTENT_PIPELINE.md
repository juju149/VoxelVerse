# Content Pipeline

The permanent content flow is:

```text
assets/packs/*/defs
assets/packs/*/media
-> validation
-> generated registries and atlases
-> compact runtime registries
-> game runtime, worldgen, meshing, renderer
```

## Identity

Content identity is path-as-identity inside a pack namespace. Raw `.ron` files
should not repeat their own ID when the namespace and path already define it.

Examples:

```text
assets/packs/core/defs/blocks/terrain/grass.block.ron
-> core:block/terrain/grass

assets/packs/core/defs/materials/blocks/grass_block/grass_block_top.material.ron
-> core:material/blocks/grass_block/grass_block_top
```

## Pack Layout

The core pack uses:

```text
assets/packs/core/
  pack.ron
  defs/
  media/
  generated/
```

## Runtime Rules

- Runtime code must not parse raw media files directly.
- Runtime code should consume generated registries.
- Voxels store compact runtime IDs, never strings.
- Blocks reference materials.
- Materials reference texture IDs.
- Media files never define gameplay.
