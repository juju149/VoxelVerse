#include "include/math/constants.wgsl"

fn vv_fog_veil_alpha(uv: vec2<f32>) -> f32 {
    let qbits = vv_quality_bits();
    let enabled = (qbits & 16u) != 0u;
    if !enabled {
        return 0.0;
    }
    let horizon_band = vv_smooth5((uv.y - 0.42) * 2.1);
    let density = global.atmosphere_params.z * 0.12;
    return clamp(horizon_band * density, 0.0, 0.18);
}

