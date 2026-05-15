# Render Architecture

VoxelVerse render ownership is Rust-first. The engine defines the render graph,
pipeline layouts, bind groups, render profiles and validation contracts in
`crates/vv-render`. The core pack supplies WGSL shader source at stable paths so
future packs can override visuals without redefining engine pipeline structure.

## Why no render `.ron`

The old `.ron` render manifests duplicated Rust pipeline decisions and made
shader layout validation depend on content files. The new rule is simpler:
content describes game content, while rendering architecture is typed engine
code. This removes a second source of truth for material families, profiles,
techniques, shader contracts and render graphs.

## Shader Structure

Shader source lives in `assets/packs/core/render/shaders`:

- `include/math`, `include/camera`, `include/lighting`, `include/atmosphere`,
  `include/material`, `include/voxel` contain reusable WGSL.
- `passes/*` contains entry points consumed by `vv-render::ShaderPath`.
- Includes use `#include "include/..."` and are expanded by the shader library
  before WGPU sees the source.

## Quality Presets

`PerfProfile` maps hardware tiers to `Potato`, `Balanced`, `High` and `Ultra`.
The packed quality flags control triplanar grain, PCF level, volumetric fog,
volumetric clouds, FXAA and bloom. Change presets in `crates/vv-render/src/perf_profile.rs`.

## Atmosphere Ownership

Runtime time belongs to `vv-world::WorldTime`; the renderer consumes it and does
not derive simulation time from its own startup clock. Artistic atmosphere
values live in `vv-render::AtmosphereConfig`:

- `SkyPalette` owns day, dawn, dusk and night sky colors.
- `FogConfig` owns distance and height fog strength.
- `CloudConfig` owns density, speed and shader step inputs.
- `PostProcessConfig` owns exposure.
- `WeatherConfig` owns current cloud coverage and fog multiplier.

Render pass files stay split by responsibility: sky, clouds, fog, post-process,
terrain, UI and world streaming are not bundled into one atmosphere catch-all.
Built-in atmosphere presets are tropical, desert, frozen, lunar, toxic, alien
and oceanic; each preset changes sky palette, fog, clouds, exposure and day
length as one coherent planet signature.

## Future Rules

- Add new passes by extending `ShaderPath` and the explicit Rust pipeline setup.
- Keep shader paths stable; modding will depend on them.
- Keep expensive full-screen effects profile-gated.
- Keep `GlobalUniform` layout changes synchronized between Rust and WGSL.
- Run Pack Doctor after shader structure changes; it rejects render `.ron` files
  and validates expected shaders and includes.

