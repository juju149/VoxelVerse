#include "include/math/constants.wgsl"
#include "include/math/noise.wgsl"

fn vv_cloud_density(uv: vec2<f32>) -> f32 {
    let t = global.render_params.x * global.cloud_params.z;
    let p = uv * vec2<f32>(4.6, 2.2) + vec2<f32>(t, t * 0.18);
    let base = vv_fbm2d(p);
    let detail = vv_noise2d(p * 4.0 + vec2<f32>(9.2, 2.7)) * 0.18;
    let coverage = global.cloud_params.w;
    return clamp((base + detail - coverage) * 2.0, 0.0, 1.0);
}

fn vv_cloud_light(density: f32) -> vec3<f32> {
    let sun_elev = normalize(global.sun_dir.xyz).y;
    let day = clamp(sun_elev * 4.0 + 0.2, 0.0, 1.0);
    let dawn = clamp(1.0 - abs(sun_elev) * 5.5, 0.0, 1.0) * clamp(sun_elev * 6.0 + 0.8, 0.0, 1.0);
    let shade = mix(0.62, 1.0, 1.0 - density);
    let day_col = vec3<f32>(0.94, 0.96, 1.0);
    let dawn_col = vec3<f32>(1.0, 0.60, 0.36);
    let night_col = vec3<f32>(0.055, 0.065, 0.13);
    return mix(night_col, mix(day_col, dawn_col, dawn * 0.65), day) * shade;
}

