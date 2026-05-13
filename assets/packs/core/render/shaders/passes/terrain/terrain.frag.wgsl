#include "include/camera/globals.wgsl"
#include "include/math/constants.wgsl"
#include "include/lighting/sun.wgsl"
#include "include/lighting/shadows.wgsl"
#include "include/lighting/ambient.wgsl"
#include "include/atmosphere/aerial_perspective.wgsl"
#include "include/material/pbr_lite.wgsl"
#include "include/material/voxel_material.wgsl"
#include "include/material/foliage_material.wgsl"
#include "include/voxel/face_variation.wgsl"
#include "include/voxel/triplanar.wgsl"
#include "include/voxel/ao.wgsl"
#include "include/camera/depth.wgsl"

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) shadow_pos: vec3<f32>,
    @location(5) color: vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let layer = in.packed_tex_index & VV_MATERIAL_INDEX_MASK;
    let qbits = vv_quality_bits();
    let color_only = (qbits & 8u) != 0u;
    let triplanar = (qbits & 1u) != 0u;
    let pcf_level = (qbits >> 1u) & 3u;

    var albedo = vv_sample_voxel_albedo(layer, in.uv, in.color, color_only);
    if !color_only && layer != VV_VERTEX_COLOR_ONLY {
        albedo *= 0.90 + vv_face_variation(in.world_pos, in.world_normal) * 0.20;
        if triplanar {
            albedo *= 1.0 + vv_triplanar_grain(in.world_pos, in.world_normal);
        }
    }

    let roughness = vv_sample_voxel_roughness(layer, in.uv, color_only);
    let normal = normalize(in.world_normal);
    let sun_dir = normalize(global.sun_dir.xyz);
    let ndotl = vv_sun_wrap(normal, sun_dir);
    let shadow = mix(0.23, 1.0, vv_shadow(in.shadow_pos, ndotl, pcf_level));
    let sun_col = vv_sun_color();
    let direct = sun_col * ndotl * shadow;
    let view_dir = normalize(global.camera_pos.xyz - in.world_pos);
    let specular = sun_col * vv_specular_lite(normal, view_dir, sun_dir, roughness) * shadow;
    let ambient = vv_hemisphere_ambient(normal, in.world_pos);

    var color = albedo * (direct + ambient) + specular;
    color = vv_apply_aerial_perspective(color, in.world_pos);
    return vec4<f32>(max(color, vec3<f32>(0.0)), 1.0);
}
