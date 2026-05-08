# Modding Notes

Modding source files are `.ron` files under `assets/packs/<namespace>/`.

## Current Supported Content

- Blocks: `blocks/*.ron`
- Block texture references: `textures/**/*.png`
- Procedural worldgen: `worldgen/**`

The pack namespace is the folder name. A file path becomes the content key, for example `assets/packs/core/blocks/air.ron` becomes `core:air`.

## Validation Expectations

Pack errors should be explicit. A broken pack should fail in loading or compilation with a useful diagnostic rather than silently falling back.

Current required block rules:

- One block key ending in `:air` must exist and becomes `VoxelId(0)`.
- Exactly one block must declare `role = "planet_core"`.
- Solid blocks should have all material faces resolved when texture data is present.

## Future Studio/CLI Contract

VoxelVerse Studio and `vv-cli` should call:

- `vv-content-schema` to author raw data shapes.
- `vv-pack-loader` to discover and parse packs.
- `vv-pack-compiler` to validate and build runtime registries.

They should not implement separate pack parsers.

