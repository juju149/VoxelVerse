#include "include/camera/globals.wgsl"

struct Local {
    model: mat4x4<f32>,
    params: vec4<f32>,
}

@group(1) @binding(0) var<uniform> local: Local;

struct VertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec3<f32>,
    @location(4) tex_index: u32,
}

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

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    let world = local.model * vec4<f32>(in.pos, 1.0);
    let normal_mat = mat3x3<f32>(
        local.model[0].xyz,
        local.model[1].xyz,
        local.model[2].xyz
    );
    let world_normal = vv_safe_normalize(normal_mat * in.normal);
    let light_clip = global.light_view_proj * vec4<f32>(world.xyz + world_normal * 0.025, 1.0);

    var out: VertexOut;
    out.clip_pos = global.view_proj * world;
    out.uv = in.uv;
    out.world_normal = world_normal;
    out.world_pos = world.xyz;
    out.view_pos = global.camera_pos.xyz;
    out.shadow_pos = vec3<f32>(
        light_clip.x * 0.5 + 0.5,
        -light_clip.y * 0.5 + 0.5,
        light_clip.z
    );
    out.color = clamp(in.color, vec3<f32>(0.0), vec3<f32>(1.0));
    out.packed_tex_index = in.tex_index;
    out.lod_alpha = clamp(local.params.x, 0.0, 1.0);
    return out;
}