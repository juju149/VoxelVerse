# VoxelVerse Render Shader Library

`render/` no longer contains `.ron` pipeline manifests. Pipelines, bind groups,
render profiles and pass order are defined in Rust in `vv-render`.

The pack owns shader source only:

- `shaders/include/` contains shared WGSL code.
- `shaders/passes/` contains pass entry points with stable paths.
- `#include "include/..."` is expanded by `vv-render` before WGPU compilation.

Stable pass shaders expected by Rust include terrain, sky, clouds, volumetric
fog, water, foliage, post, UI and debug shaders. Add a shader by placing a WGSL
file in the appropriate folder, then add or update the typed `ShaderPath` and
pipeline descriptor in `vv-render`.

Performance rules:

- Keep fragment loops bounded by quality profile.
- Put shared code in `include/` instead of copy-pasting.
- Avoid full-screen noise unless it is gated by profile flags.
- Keep individual shader files small enough to review; split near 400 lines.
- Do not add `.ron` files under `render/`.

