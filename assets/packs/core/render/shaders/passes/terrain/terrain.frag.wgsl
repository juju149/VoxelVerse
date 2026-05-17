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
#include "include/voxel/lod_fade.wgsl"
#include "include/camera/depth.wgsl"

// Ghibli-styled terrain fragment.
// Soft warm key light, tinted painterly shadows, gentle saturation lift.
// One sample per material map; no extra texture lookups vs the previous pass.

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

fn vv_prepare_albedo(
    layer: u32,
    packed_tex_index: u32,
    uv: vec2<f32>,
    vertex_color: vec3<f32>,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    color_only: bool,
    triplanar: bool,
) -> vec3<f32> {
    var albedo = vv_sample_voxel_albedo(layer, uv, vertex_color, color_only);

    if !color_only && layer != VV_VERTEX_COLOR_ONLY {
        albedo = vv_material_face_variation(albedo, world_pos, normal);
        albedo = vv_material_large_scale_variation(albedo, world_pos, normal);
        albedo = vv_material_apply_block_contact(albedo, packed_tex_index, uv, normal, world_pos);

        if triplanar {
            albedo *= 1.0 + vv_triplanar_grain(world_pos, normal);
        }
    }

    return max(albedo, vec3<f32>(0.0));
}

// Light contribution from the sun: wrap-around diffuse + soft backlight.
// Shadows are tinted warm at dawn and cool during the day for painterly depth.
// shadow: pre-computed shadow factor [0,1] shared with specular to avoid
// sampling the shadow map twice per fragment.
fn vv_ghibli_direct(
    normal: vec3<f32>,
    world_pos: vec3<f32>,
    shadow: f32,
) -> vec3<f32> {
    let sun_dir = vv_sun_direction();
    let ndotl = vv_sun_wrap_diffuse(normal, sun_dir);

    // Painterly shadow tint mixes sky + dawn warmth.
    let dawn = vv_dawn_factor();
    let shadow_cool = global.sky_zenith.rgb * 0.18;
    let shadow_warm = vec3<f32>(0.30, 0.18, 0.20);
    let shadow_floor = mix(shadow_cool, shadow_warm, dawn);

    let sun = vv_sun_color();
    let wrap = sun * ndotl;
    let back = sun * vv_soft_backlight(normal, sun_dir) * 1.4;

    // Lift shadows toward the sky tint instead of crushing to black.
    return wrap * shadow + shadow_floor * (1.0 - shadow * 0.75) + back;
}

// Subtle saturation boost: paintings push primaries without going neon.
fn vv_ghibli_paint(color: vec3<f32>) -> vec3<f32> {
    let luma = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(vec3<f32>(luma), color, 1.10);
}

// Gentle hemispheric tint: top of geometry catches sky, side faces stay neutral.
fn vv_ghibli_sky_lift(color: vec3<f32>, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);
    let sky_facing = max(dot(normal, up), 0.0);
    let lift = global.sky_zenith.rgb * sky_facing * 0.012 * vv_day_factor();
    return color + lift;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Dithered LOD fade: stochastically discard pixels when a chunk is
    // spawning in or dying out. Operates on the opaque depth pass so no
    // alpha-blending pipeline is needed.
    if in.lod_alpha < vv_dither_threshold(in.clip_pos) {
        discard;
    }

    let layer = in.packed_tex_index & VV_MATERIAL_INDEX_MASK;
    let qbits = vv_quality_bits();

    let color_only = (qbits & VV_Q_COLOR_ONLY) != 0u;
    let triplanar  = (qbits & VV_Q_TRIPLANAR)  != 0u;

    let geometry_normal = vv_safe_normalize(in.world_normal);
    let view_dir = vv_safe_normalize(vv_camera_position() - in.world_pos);

    let roughness = vv_sample_voxel_roughness(layer, in.uv, color_only);
    var albedo = vv_prepare_albedo(layer, in.packed_tex_index, in.uv, in.color, in.world_pos, geometry_normal, color_only, triplanar);
    albedo = vv_ghibli_paint(albedo);

    let normal = vv_sample_voxel_normal(layer, in.uv, geometry_normal, color_only);

    let ao = vv_vertex_ao(in.color) * vv_soft_contact_occlusion(geometry_normal, in.world_pos);

    // Compute shadow once and share between direct lighting and specular.
    // At PCF level 2 this saves up to 13 depth-compare samples per fragment.
    let sun_dir = vv_sun_direction();
    let ndotl = vv_sun_wrap_diffuse(normal, sun_dir);
    let shadow = vv_shadow(in.shadow_pos, ndotl, vv_shadow_pcf_level());

    let direct = vv_ghibli_direct(normal, in.world_pos, shadow);
    let ambient = vv_hemisphere_ambient(normal, in.world_pos) * ao;
    let bounce = vv_micro_bounce(normal, in.world_pos, albedo);

    // Crisp painterly highlight, gated by the same shadow.
    let specular = vv_sun_color() * vv_specular_lite(normal, view_dir, sun_dir, roughness) * shadow;

    var color = albedo * (direct + ambient + bounce) + specular;
    color = vv_ghibli_sky_lift(color, geometry_normal, in.world_pos);
    color = vv_apply_aerial_perspective(color, in.world_pos);

    return vec4<f32>(max(color, vec3<f32>(0.0)), 1.0);
}
