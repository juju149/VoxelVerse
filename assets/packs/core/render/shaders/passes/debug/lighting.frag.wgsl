#include "include/camera/globals.wgsl"

struct DebugIn {
    @builtin(position) clip_pos: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
}

@fragment
fn fs_main(in: DebugIn) -> @location(0) vec4<f32> {
    let ndotl = max(dot(normalize(in.world_normal), normalize(global.sun_dir.xyz)), 0.0);
    return vec4<f32>(vec3<f32>(ndotl), 1.0);
}

