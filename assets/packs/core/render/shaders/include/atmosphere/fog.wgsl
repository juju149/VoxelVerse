#include "include/lighting/volumetric_light.wgsl"

fn vv_aerial_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let cam_pos = global.camera_pos.xyz;
    let dist = distance(cam_pos, world_pos);
    let density = global.atmosphere_params.x;
    let height_strength = global.atmosphere_params.y;
    let avg_r = mix(length(world_pos), length(cam_pos), 0.5);
    let height = max(avg_r - 1.0, 0.0) * 0.007;
    let dens = density * (1.0 + exp(-height) * height_strength);
    let fog_sq = dist * dens;
    let fog_f = clamp(1.0 - exp(-fog_sq * fog_sq * 0.52), 0.0, 1.0);
    let view_dir = normalize(world_pos - cam_pos);
    let scatter = vv_forward_scatter(view_dir);
    let fog_sun = vec3<f32>(1.05, 0.72, 0.34) * global.sky_zenith.w;
    let fog_col = mix(global.sky_horizon.rgb, fog_sun, scatter);
    return mix(color, fog_col, fog_f);
}

