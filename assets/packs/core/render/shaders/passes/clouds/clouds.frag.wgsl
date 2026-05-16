#include "include/camera/globals.wgsl"
#include "include/atmosphere/clouds_common.wgsl"

// Ghibli cloud pass: a single sharpened sample for low quality,
// a tiny 3-tap parallax stack for high quality. No long loops.

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let qbits = vv_quality_bits();
    let high_clouds = (qbits & 32u) != 0u;

    // Fade clouds out below the horizon and very close to it.
    let h = vv_cloud_saturate(1.0 - in.uv.y);
    let horizon_mask = vv_cloud_smooth5((h - 0.08) * 1.5);
    if horizon_mask <= 0.0 {
        return vec4<f32>(0.0);
    }

    var density: f32;
    if high_clouds {
        // 3 stacked layers create soft parallax / volumetric feeling
        // without a real raymarch.
        let d0 = vv_cloud_density(in.uv);
        let d1 = vv_cloud_density(in.uv + vec2<f32>(0.022, 0.008));
        let d2 = vv_cloud_density(in.uv + vec2<f32>(0.048, 0.020));
        density = (d0 * 0.55 + d1 * 0.30 + d2 * 0.15);
    } else {
        density = vv_cloud_density(in.uv);
    }

    let alpha = clamp(density * global.cloud_params.y * horizon_mask, 0.0, 0.78);
    let color = vv_cloud_light(density);

    return vec4<f32>(color, alpha);
}
