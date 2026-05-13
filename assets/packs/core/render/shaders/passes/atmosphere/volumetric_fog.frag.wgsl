#include "include/camera/globals.wgsl"
#include "include/atmosphere/volumetric_fog.wgsl"

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let alpha = vv_fog_veil_alpha(in.uv);
    let sun_dir = normalize(global.sun_dir.xyz);
    let warmth = clamp(sun_dir.y * 3.0 + 0.4, 0.0, 1.0);
    let fog_col = mix(global.sky_horizon.rgb, vec3<f32>(1.04, 0.74, 0.42), warmth * 0.22);
    return vec4<f32>(fog_col, alpha);
}

