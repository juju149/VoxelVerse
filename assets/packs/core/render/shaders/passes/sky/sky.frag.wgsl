#include "include/interface/global.wgsl"
#include "include/interface/fullscreen_io.wgsl"

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let h = clamp(1.0 - in.uv.y, 0.0, 1.0);
    let horizon = max(global.sky_horizon.rgb, vec3<f32>(0.56, 0.64, 0.70));
    let zenith = max(global.sky_zenith.rgb, vec3<f32>(0.70, 0.78, 0.88));
    let sky = mix(horizon, zenith, smoothstep(0.0, 1.0, h));
    return vec4<f32>(sky, 1.0);
}
