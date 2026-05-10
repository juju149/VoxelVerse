# VoxelVerse Core Pack

This pack contains the built-in VoxelVerse content.

Current active runtime inputs:

- `blocks/`
- `worldgen/`
- `textures/`

Target authoring architecture:

- `defs/` for gameplay/content definitions.
- `media/` for final runtime media assets.
- `source/` for editable source assets and references.
- `generated/` for generated registries, atlases, caches and diagnostics.
- `legacy_imports/` for preserved imports that still need conversion.

The historical `voxel/` asset bank has been migrated out of the runtime root.
Most `.vox` files now live under `media/voxel/`; uncertain legacy sprite assets
and old manifests are quarantined under `legacy_imports/`.

`pack.toml` remains the current metadata file until a `pack.ron` manifest schema
is implemented and wired into the pack loader.
