#include "include/camera/globals.wgsl"

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) shadow_pos: vec3<f32>,
    @location(5) color: vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
    @location(7) @interpolate(flat) lod_alpha: f32,
}

fn vv_dither(pixel: vec4<f32>) -> f32 {
    let p = floor(pixel.xy);
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    if in.lod_alpha < 0.999 && vv_dither(in.clip_pos) > in.lod_alpha {
        discard;
    }

    let n = vv_safe_normalize(in.world_normal);
    let sun = vv_safe_normalize(global.sun_dir.xyz);

    let top_light = max(dot(n, sun) * 0.5 + 0.5, 0.0);
    let ambient = 0.68;
    let diffuse = top_light * 0.32;

    let base = clamp(in.color, vec3<f32>(0.02), vec3<f32>(1.0));
    let color = base * (ambient + diffuse);

    return vec4<f32>(color, 1.0);
}