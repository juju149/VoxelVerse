#include "include/camera/globals.wgsl"
#include "include/interface/local.wgsl"
#include "include/interface/terrain_io.wgsl"

@vertex
fn vs_main(in: TerrainVertexIn) -> TerrainVertexOut {
    let world = local.model * vec4<f32>(in.pos, 1.0);
    let normal_mat = mat3x3<f32>(
        local.model[0].xyz,
        local.model[1].xyz,
        local.model[2].xyz
    );
    let world_normal = vv_safe_normalize(normal_mat * in.normal);
    let light_clip = global.light_view_proj * vec4<f32>(world.xyz + world_normal * 0.025, 1.0);

    var out: TerrainVertexOut;
    out.clip_pos = global.view_proj * world;
    out.uv = in.uv;
    out.world_normal = world_normal;
    out.world_pos = world.xyz;
    out.view_pos = vv_camera_position();
    out.shadow_pos = vec3<f32>(
        light_clip.x * 0.5 + 0.5,
        -light_clip.y * 0.5 + 0.5,
        light_clip.z
    );
    out.color = clamp(in.color, vec3<f32>(0.0), vec3<f32>(1.0));
    out.packed_tex_index = in.tex_index;
    out.lod_alpha = vv_local_opacity();
    return out;
}