# VoxelVerse V1 Shader Pipeline Architecture

This document defines the technical architecture for the V1 shader pipeline.

The goal is not to add visual complexity first. The goal is to make the render
pipeline honest, inspectable, deterministic, performant, and safe to extend.

## Core rule

Do not add beautiful shader features before the pipeline can prove what it is
drawing.

Every visual feature must pass through:

1. a render pass declaration;
2. a pipeline descriptor;
3. a shader interface contract;
4. a debug view;
5. a performance budget.

## Layers

### Render Graph

Owns pass identity and stable shader paths.

Examples:

- ShadowDepth
- Sky
- Celestial
- Clouds
- TerrainOpaque
- VolumetricFog
- Precipitation
- FinalComposite
- Ui

The graph describes what exists.

### Render Schedule

Owns which passes are active for the current frame.

Examples:

- Clouds are active only if quality enables volumetric clouds and density is non-zero.
- Fog is active only if quality enables volumetric fog and strength is non-zero.
- Precipitation is active only if precipitation intensity is non-zero.

The schedule decides what runs.

### Pipeline Descriptors

Own the fixed technical contract of every pipeline:

- pipeline id;
- pass id;
- mesh or fullscreen;
- vertex shader;
- fragment shader;
- vertex layout;
- bind groups;
- render target;
- depth mode;
- blend mode.

The descriptor table is the blueprint. The future PipelineRegistry will use it
to create wgpu pipelines.

### Shader Library

Owns pack shader loading, include expansion, and WGSL validation.

Rules:

- no absolute include path;
- no include escaping shader root;
- active shaders must parse in tests;
- optional shaders must be promoted before production use.

### Shader Contract

Owns binary layouts and stable locations shared by Rust and WGSL:

- GlobalUniform = 304 bytes;
- LocalUniform = 80 bytes;
- Vertex = 48 bytes;
- vertex attribute locations;
- terrain output locations;
- quality bits;
- material sentinels.

WGSL must mirror this contract through include/interface files.

## Debug views required before beauty

The terrain pipeline must support the following debug modes before V1 beauty work:

- VertexColor;
- WorldNormal;
- MaterialLayer;
- Uv;
- LodAlpha;
- ChunkKind;
- ShadowFactor;
- Depth;
- WorldPositionBands.

If terrain is wrong, debug views must reveal whether the fault comes from:

- meshing;
- vertex colors;
- material ids;
- LOD overlap;
- depth conflict;
- shader math;
- post-process;
- atmosphere.

## V1 rebuild order

1. Architecture foundation.
2. Shader interface WGSL.
3. Debug terrain suite.
4. Stable flat terrain.
5. Texture albedo only.
6. Simple lighting.
7. Shadows.
8. Sky world-space.
9. Depth-aware fog.
10. Water pipeline.
11. Foliage pipeline.
12. Post-process.

## Forbidden for now

Until debug views exist, do not add:

- normal map material shading;
- roughness/specular;
- screen-space fog bands;
- heavy clouds;
- bloom;
- water transparency;
- foliage alpha sorting;
- triplanar noise;
- decorative shader variation.

V1 beauty must grow from a stable skeleton, not from shader fog.