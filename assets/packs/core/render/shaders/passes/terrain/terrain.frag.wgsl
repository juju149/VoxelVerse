#include "include/camera/globals.wgsl"
#include "include/math/constants.wgsl"
#include "include/math/random.wgsl"
#include "include/lighting/volumetric_light.wgsl"
#include "include/lighting/sun.wgsl"
#include "include/lighting/shadows.wgsl"
#include "include/lighting/ambient.wgsl"
#include "include/atmosphere/fog.wgsl"
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

fn vv_prepare_albedo(layer: u32, uv: vec2<f32>, vertex_color: vec3<f32>, world_pos: vec3<f32>, normal: vec3<f32>, color_only: bool, triplanar: bool) -> vec3<f32> {
    var albedo = vv_sample_voxel_albedo(layer, uv, vertex_color, color_only);

    if !color_only && layer != VV_VERTEX_COLOR_ONLY {
        albedo = vv_material_face_variation(albedo, world_pos, normal);
        albedo = vv_material_large_scale_variation(albedo, world_pos, normal);

        if triplanar {
            albedo *= 1.0 + vv_triplanar_grain(world_pos, normal);
        }
    }

    return max(albedo, vec3<f32>(0.0));
}

fn vv_direct_light(normal: vec3<f32>, world_pos: vec3<f32>, shadow_pos: vec3<f32>, roughness: f32) -> vec3<f32> {
    let sun_dir = vv_sun_direction();
    let ndotl = vv_sun_wrap_diffuse(normal, sun_dir);
    let shadow = mix(0.22, 1.0, vv_shadow(shadow_pos, ndotl, vv_shadow_pcf_level()));
    let sun = vv_sun_color();

    let wrap = sun * ndotl * shadow;
    let back = sun * vv_soft_backlight(normal, sun_dir);

    return wrap + back;
}

fn vv_apply_cinematic_depth(color: vec3<f32>, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);
    let sky_facing = max(dot(normal, up), 0.0);

    // Tiny lift for upward faces so plains read cleanly under atmospheric fog.
    let sky_lift = global.sky_zenith.rgb * sky_facing * 0.025 * vv_day_factor();

    // Gentle coolness on far side faces gives sculpted voxel relief.
    let side = vv_saturate(1.0 - abs(dot(normal, up)));
    let cool_side = vec3<f32>(0.965, 0.982, 1.015);

    return (color + sky_lift) * mix(vec3<f32>(1.0), cool_side, side * 0.045);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let layer = in.packed_tex_index & VV_MATERIAL_INDEX_MASK;
    let qbits = vv_quality_bits();

    let color_only = (qbits & VV_Q_COLOR_ONLY) != 0u;
    let triplanar = (qbits & VV_Q_TRIPLANAR) != 0u;

    let geometry_normal = vv_safe_normalize(in.world_normal);
    let view_dir = vv_safe_normalize(vv_camera_position() - in.world_pos);

    let roughness = vv_sample_voxel_roughness(layer, in.uv, color_only);
    let albedo = vv_prepare_albedo(layer, in.uv, in.color, in.world_pos, geometry_normal, color_only, triplanar);
    let normal = vv_sample_voxel_normal(layer, in.uv, geometry_normal, color_only);

    let ao = vv_vertex_ao(in.color) * vv_soft_contact_occlusion(geometry_normal, in.world_pos);

    let direct = vv_direct_light(normal, in.world_pos, in.shadow_pos, roughness);
    let ambient = vv_hemisphere_ambient(normal, in.world_pos) * ao;
    let bounce = vv_micro_bounce(normal, in.world_pos, albedo);

    let sun_dir = vv_sun_direction();
    let shadow_for_spec = vv_shadow(in.shadow_pos, max(dot(normal, sun_dir), 0.0), vv_shadow_pcf_level());
    let specular_strength = vv_specular_lite(normal, view_dir, sun_dir, roughness);
    let specular = vv_sun_color() * specular_strength * shadow_for_spec;

    var color = albedo * (direct + ambient + bounce) + specular;
    color = vv_apply_cinematic_depth(color, geometry_normal, in.world_pos);
    color = vv_apply_aerial_perspective(color, in.world_pos);

    return vec4<f32>(max(color, vec3<f32>(0.0)), 1.0);
}

