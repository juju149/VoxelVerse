#include "include/camera/globals.wgsl"
#include "include/atmosphere/clouds_common.wgsl"

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let qbits = vv_quality_bits();
    let high_clouds = (qbits & 32u) != 0u;
    let steps = max(global.cloud_params.x, 1.0);
    let horizon_mask = vv_smooth5((1.0 - in.uv.y) * 1.8 - 0.12);
    var density = 0.0;
    if high_clouds {
        for (var i = 0u; i < 14u; i = i + 1u) {
            if f32(i) >= steps {
                break;
            }
            let layer = f32(i) / max(steps - 1.0, 1.0);
            density += vv_cloud_density(in.uv + vec2<f32>(layer * 0.045, layer * 0.018)) / steps;
        }
    } else {
        density = vv_cloud_density(in.uv);
    }
    let alpha = clamp(density * global.cloud_params.y * horizon_mask, 0.0, 0.72);
    let color = vv_cloud_light(density);
    return vec4<f32>(color, alpha);
}

