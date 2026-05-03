# Data Driven Block Details - Phase 2

Phase 2 adds reusable procedural details to block visuals without creating block-specific code.

Generic detail kernels:

- pebble
- root
- leaf_lobe
- grain
- speckle
- stain
- crack

## Runtime layout

Each RuntimeBlockDetail uses:

color  = rgba detail color
params = density, min_size, max_size, slope_bias
meta   = kind, face_mask, seed, reserved

RuntimeBlockVisual.procedural.z stores the compiled detail count.

## Authoring example

details: [
    (
        kind: pebble,
        color: "#A66A3DA0",
        density: 0.18,
        min_size: 0.035,
        max_size: 0.075,
        slope_bias: 0.25,
        faces: [side],
        seed: 4201,
    ),
]

## Next

Phase 3 should introduce visual presets so blocks can inherit from reusable .ron visual recipes.