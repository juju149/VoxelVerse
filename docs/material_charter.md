# Terrain Material Charter

This charter defines the visual contract for core terrain materials. It is not
runtime data: authored materials still live in pack files and textures. The goal
is to keep the pack coherent before adding more premium rendering features.

## Global Direction

- Style: simple, sober, modern, readable at close range and stable at distance.
- Albedo: clear color identity, no photographic noise, no tiny high-contrast
  speckles.
- Normal: shallow voxel-friendly relief; avoid deep sculpted normals that fight
  the cube silhouette.
- Roughness: mostly matte. Terrain should read through color and lighting, not
  through glossy highlights.
- Mips: every terrain material must remain recognizable once mipmapped. Details
  below four pixels should support the broad color, not create shimmer.
- Macro variation: broad biome-scale color fields only. Do not bake noisy color
  variation into the texture to compensate for missing world variation.

## Material Families

| Family | Albedo direction | Normal direction | Roughness target | Notes |
| --- | --- | --- | --- | --- |
| grass | Saturated but natural green, warmer tops, cooler shadow side | Soft fiber and soil breakup, very shallow | 0.72-0.88 | Must stay readable as a surface blanket from far away. |
| dirt | Warm brown with restrained red/yellow bias | Fine packed clumps, no harsh cracks | 0.78-0.92 | Should support grass without becoming noisy. |
| stone | Neutral gray with slight cool or warm variants by biome | Broad chips and planes, no sharp photo detail | 0.64-0.82 | Main cliff material; silhouette and AO do most of the work. |
| sand | Light tan, low contrast, clean value | Soft grains implied, not literal speckle | 0.82-0.95 | Must not shimmer on beaches or dunes. |
| snow | Bright but not pure white, slight blue in shadow | Soft packed surface, almost flat | 0.70-0.90 | Preserve detail in sunlight; avoid blown-out albedo. |
| wood | Warm trunk color with clear growth direction | Directional grain, shallow | 0.58-0.78 | Ends and sides need distinct but compatible values. |
| leaves | Medium green with small hue variation | Minimal relief | 0.72-0.90 | Prefer cutout/foliage rules over noisy opaque terrain shading. |
| water | Data-driven material slot reserved for translucent rendering | Surface normals can be stronger than terrain | 0.02-0.18 | Must use a separate water path when transparency lands. |
| ore | Host stone plus readable mineral accents | Match host stone first | 0.55-0.78 | Accent coverage should stay low so ore reads as embedded, not painted. |

## Sampling Rules

- Close camera: preserve voxel-pixel crispness with nearest magnification.
- Far camera: use mipmapped minification to remove shimmer.
- Texture edges: author tiles as seamless; shader wrapping is per-face and uses
  material UVs, so edge pixels must not contain a one-pixel border color.
- High-frequency detail must survive mip reduction as a clean average. If a
  texture turns gray or dirty in mip previews, simplify the source texture.

## Contact And Transitions

- Ambient occlusion should explain block contact, not paint black creases.
- Rounded-edge hints stay subtle; they can soften the cube contact but must not
  make blocks look melted.
- Top and side variants should share hue family and roughness. Differences come
  from value, saturation, and biome tint rather than unrelated textures.
