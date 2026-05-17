fn vv_lod_alpha(local_params: vec4<f32>) -> f32 {
    return clamp(local_params.x, 0.0, 1.0);
}

// Ordered Bayer 4×4 dither threshold for alpha-tested LOD fade.
// Returns a per-pixel threshold in (0.03125, 0.96875).
// Discard the fragment if lod_alpha < vv_dither_threshold(clip_pos).
fn vv_dither_threshold(clip_pos: vec4<f32>) -> f32 {
    // Convert clip-space position to window pixel coords before dithering.
    let ndc = clip_pos.xy / clip_pos.w;
    let res = vv_viewport_size();
    let window = (ndc * 0.5 + vec2<f32>(0.5)) * res;

    // Pixel indices modulo 4.
    let ix = u32(floor(window.x)) & 3u;
    let iy = u32(floor(window.y)) & 3u;

    // Decompose into low/high bits and compute Bayer value without
    // indexing a constant array (avoid static-index restriction).
    let x0 = ix & 1u;
    let x1 = (ix >> 1u) & 1u;
    let y0 = iy & 1u;
    let y1 = (iy >> 1u) & 1u;

    var d2_low: u32;
    if (y0 == 0u) {
        if (x0 == 0u) {
            d2_low = 0u;
        } else {
            d2_low = 2u;
        }
    } else {
        if (x0 == 0u) {
            d2_low = 3u;
        } else {
            d2_low = 1u;
        }
    }

    var d2_high: u32;
    if (y1 == 0u) {
        if (x1 == 0u) {
            d2_high = 0u;
        } else {
            d2_high = 2u;
        }
    } else {
        if (x1 == 0u) {
            d2_high = 3u;
        } else {
            d2_high = 1u;
        }
    }

    let k = 4u * d2_low + d2_high;
    return (f32(k) + 0.5) / 16.0;
}

