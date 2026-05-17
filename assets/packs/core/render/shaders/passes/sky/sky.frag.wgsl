#include "include/camera/globals.wgsl"

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let y = clamp(1.0 - in.uv.y, 0.0, 1.0);
    let horizon = max(global.sky_horizon.rgb, vec3<f32>(0.55, 0.62, 0.68));
    let zenith = max(global.sky_zenith.rgb, vec3<f32>(0.68, 0.76, 0.84));
    let sky = mix(horizon, zenith, y);
    return vec4<f32>(sky, 1.0);
}