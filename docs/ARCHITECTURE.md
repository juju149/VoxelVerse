# VoxelVerse Architecture

VoxelVerse is a Rust workspace split by responsibility, not by future wishful layers.

## Current Crates

- `vv-voxel`: low-level voxel language. Owns `VoxelId`, voxel coordinates, chunk keys, LOD keys, chunk size, and compact override chunks.
- `vv-math`: pure geometry and camera math. It has no gameplay, rendering, pack, or world state knowledge.
- `vv-content-schema`: raw `.ron` structs only. It does not read files or assign runtime IDs.
- `vv-pack-loader`: filesystem pack loading and RON parsing. It derives content identity from pack namespace and file path.
- `vv-pack-compiler`: content validation, reference resolution, compact runtime registries, and PNG texture decoding.
- `vv-diagnostics`: shared diagnostics counters and startup/system reporting.
- `voxelverse-game`: runtime application. It owns windowing, gameplay wiring, world runtime, generation, meshing, renderer integration, input, and UI console.

## Deliberately Not Extracted Yet

- `vv-render`: not split yet because the renderer still depends directly on game UI, player state, world streaming, and meshing details. Extract it after renderer inputs become explicit data structs.
- `vv-core`: not created yet. There is no generic shared core concept worth a crate today; `VoxelId` belongs in `vv-voxel`.
- `vv-texture-gen` and `vv-preview-render`: planned for tooling/Studio, but not created as empty crates.

## Application Boundary

`apps/voxelverse-game` loads compiled content and renders/runs the game. It must not become an editor. Future tools, Studio, MCP integrations, and CLI commands should consume the same pack/schema/compiler crates instead of duplicating parsing or validation logic.

