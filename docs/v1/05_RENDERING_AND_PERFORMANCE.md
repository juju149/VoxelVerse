# VoxelVerse V1 Rendering And Performance Contract

V1 rendering must make the world beautiful without turning the GPU into soup.
The goal is not maximum realism. The goal is a stable, premium, readable voxel planet.

## Rendering pillars

1. Stable horizon.
2. Beautiful atmosphere.
3. Readable voxel surfaces.
4. Smooth LOD transitions.
5. Satisfying first-person feedback.
6. Strong diagnostics.
7. Bounded budgets.

## Target frame budgets

V1 must provide at least three profiles:

| Profile | Target | Notes |
| --- | --- | --- |
| Low | 60 FPS on modest GPU | reduced shadows, fewer chunks, simple fog |
| High | 60 FPS on RTX-class GPU | default beauty target |
| Ultra | visual testing | not the baseline for gameplay |

Frame time targets:

```text
60 FPS = 16.67 ms
render target = under 10 ms
world/gameplay target = under 3 ms
streaming/upload spikes = under 6 ms per frame
```

## Streaming rules

Streaming must always be bounded by:

- maximum visible voxel chunks;
- maximum visible LOD tiles;
- maximum pending voxel mesh jobs;
- maximum pending LOD jobs;
- maximum GPU uploads per frame;
- maximum GPU upload bytes per frame;
- maximum upload time per frame.

No code path may dispatch unlimited jobs because the player turned quickly.

## LOD rules

The LOD system must guarantee coverage.

Rules:

- parent LOD stays visible until replacement child tiles are ready;
- near crosshair area gets priority;
- player chunk area gets priority;
- horizon coverage gets priority over invisible behind-camera detail;
- hysteresis prevents flickering split/merge;
- transition fade is short and subtle;
- no black holes, white bands, or empty terrain flashes.

## Chunk priorities

Priority should consider:

- distance to camera;
- view direction;
- crosshair/cursor focus;
- player current chunk;
- horizon importance;
- landmark importance;
- dirty chunk priority;
- missing coverage fallback.

## Meshing rules

Meshing must be deterministic and CPU-side.

Required V1 mesh features:

- greedy meshing for opaque voxels;
- correct face culling;
- ambient occlusion or equivalent depth readability;
- material layer IDs packed efficiently;
- block damage overlay support;
- prop baking within controlled radius;
- LOD mesh generation;
- seam-safe borders;
- no per-voxel allocation hot paths.

## Shader rules

WGSL shader code must be treated as production code.

Rules:

- shared functions must live in clear include/module strategy if supported by loader;
- missing helper functions are build-time failures;
- no duplicated noise helpers across shaders unless intentionally documented;
- debug shaders must be separate from production pass code;
- quality bits must be documented;
- uniform layout changes must have compile-time size checks.

## Beauty systems ranked by importance

For V1, visual quality comes mostly from these systems:

1. atmosphere and sky gradient;
2. fog and height fog tuned per planet;
3. stable shadows;
4. terrain color/texture palette;
5. biome silhouettes;
6. water if included;
7. first-person hands/tools;
8. prop placement and vegetation density;
9. particles;
10. post-processing.

Do not add particles before terrain, LOD, atmosphere, textures and shadows are stable.

## Water V1 rule

Water is allowed in V1 only if it is:

- visually decent;
- collision aware or clearly non-interactive;
- not breaking chunk meshing;
- not causing sorting artifacts everywhere;
- not destroying FPS.

If water cannot reach this gate, V1 may ship with limited ponds or no water.

## First-person rendering

The player must see actions.

V1 requires:

- idle hand/tool pose;
- swing animation;
- hit response;
- break response;
- place response;
- tool model or simple proxy;
- selected item preview for placeable blocks;
- no camera nausea.

The render layer animates. Gameplay decides events.

## Diagnostics overlay

The renderer must expose:

- FPS;
- frame time;
- render time;
- terrain draw time;
- draw calls;
- shadow draw calls;
- active voxel chunks;
- active LODs;
- pending voxel jobs;
- pending LOD jobs;
- uploaded voxel meshes this frame;
- uploaded LOD meshes this frame;
- upload bytes;
- upload time;
- LOD selection time;
- update view time;
- GPU memory estimate;
- quality profile.

## Performance anti-patterns

Forbidden:

- rebuilding all chunks for local edit;
- unbounded rayon spawn loops;
- GPU buffer recreation for unchanged UI every frame;
- string refs in hot paths;
- per-frame raw pack parsing;
- per-voxel heap allocations in meshing;
- loading all `.vox` files at startup;
- rendering distant props at full detail;
- hiding performance bugs with fog alone;
- shipping debug draw on by default.

## Render V1 gate

Rendering is V1-ready when:

- spawn view looks good;
- day/night or at least time-of-day lighting is stable;
- shadows do not shimmer badly;
- LOD has no visible holes;
- frame time is stable while moving and turning;
- mining crack overlay is readable;
- first-person tool feedback feels satisfying;
- UI remains readable over bright and dark scenes;
- quality profiles actually change budgets;
- diagnostics tell the truth.
