#include "include/camera/globals.wgsl"
#include "include/math/constants.wgsl"
#include "include/lighting/sun.wgsl"
#include "include/lighting/volumetric_light.wgsl"
#include "include/atmosphere/fog.wgsl"
#include "include/atmosphere/aerial_perspective.wgsl"
#include "include/material/water_material.wgsl"

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

    let fresnel = vv_schlick_fresnel(max(dot(view_dir, normal), 0.0), 0.04) * global.water_params.x;

    let half_dir = normalize(view_dir + sun_dir);
    let spec = pow(max(dot(normal, half_dir), 0.0), 88.0) * global.water_params.y * global.sky_zenith.w;

    let deep = vec3<f32>(0.025, 0.12, 0.22);
    let shallow = vec3<f32>(0.12, 0.45, 0.66);
    let body = mix(deep, shallow, 0.35);

    var color = mix(body, global.sky_horizon.rgb, fresnel);
    color += vec3<f32>(1.05, 1.00, 0.92) * spec;

    color = vv_apply_aerial_perspective(color, in.world_pos);

    return vec4<f32>(color, mix(global.water_params.z, 0.96, fresnel));
}

