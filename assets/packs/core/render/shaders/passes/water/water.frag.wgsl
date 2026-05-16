#include "include/camera/globals.wgsl"
#include "include/math/constants.wgsl"
#include "include/lighting/sun.wgsl"
#include "include/lighting/volumetric_light.wgsl"
#include "include/atmosphere/fog.wgsl"
#include "include/atmosphere/aerial_perspective.wgsl"
#include "include/material/water_material.wgsl"

// Ghibli water: turquoise body, soft cyan sky reflection, painterly highlights.
// One vector reflection + Schlick fresnel — no waves, no textures.

struct WaterFragmentIn {
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
fn fs_main(in: WaterFragmentIn) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(global.camera_pos.xyz - in.world_pos);
    let sun_dir = normalize(global.sun_dir.xyz);

    let n_dot_v = max(dot(view_dir, normal), 0.0);

    // Body: stylized aquamarine, slightly desaturated when looking straight down.
    let deep    = vec3<f32>(0.030, 0.18, 0.32);
    let shallow = vec3<f32>(0.22,  0.62, 0.72);
    let body = mix(shallow, deep, pow(n_dot_v, 0.9));

    // Fresnel-driven sky reflection. Soft blue sky horizon.
    let f0 = 0.025;
    let fresnel = vv_schlick_fresnel(n_dot_v, f0) * global.water_params.x;
    let sky_reflect = mix(global.sky_horizon.rgb, global.sky_zenith.rgb, 0.35);

    var color = mix(body, sky_reflect, fresnel);

    // Painterly sun glint: tight, day-only.
    let half_dir = normalize(view_dir + sun_dir);
    let spec = pow(max(dot(normal, half_dir), 0.0), 92.0)
             * global.water_params.y
             * 0.55
             * vv_day_factor();
    let sun_tint = mix(vec3<f32>(1.05, 0.55, 0.30), vec3<f32>(1.05, 1.02, 0.92), vv_day_factor());
    color += sun_tint * spec;

    color = vv_apply_aerial_perspective(color, in.world_pos);

    let alpha = mix(global.water_params.z, 0.97, fresnel);
    return vec4<f32>(color, alpha);
}
