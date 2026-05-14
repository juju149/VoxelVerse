#include "include/lighting/volumetric_light.wgsl"

fn vv_luma(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Distance fog used by terrain / water.
// Goal: cinematic depth without a white horizontal band.
// The fog must darken and desaturate distance softly, not paint milk over the scene.
fn vv_aerial_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let cam_pos = global.camera_pos.xyz;
    let dist = distance(cam_pos, world_pos);

    // Raw density comes from the runtime profile.
    // We intentionally calm it down here because the old value reached full fog too early.
    let raw_density = max(global.atmosphere_params.x, 0.0);
    let density = raw_density * 0.16;

    let fog_distance = dist * density;

    // Slower exponential curve: keeps distant mountains visible.
    let distance_fog = 1.0 - exp(-(fog_distance * fog_distance) * 0.16);

    // Never let distance fog fully erase the world.
    let fog_f = clamp(distance_fog, 0.0, 0.30);

    let view_dir = normalize(world_pos - cam_pos);
    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_elev = sun_dir.y;

    let day = clamp(sun_elev * 4.0 + 0.20, 0.0, 1.0);
    let dawn = clamp(1.0 - abs(sun_elev) * 5.5, 0.0, 1.0)
        * clamp(sun_elev * 6.0 + 0.8, 0.0, 1.0);

    let scatter = clamp(vv_forward_scatter(view_dir) * 0.28, 0.0, 0.22);

    // Blue-grey atmospheric haze, deliberately not white.
    let night_haze = vec3<f32>(0.038, 0.046, 0.070);
    let day_haze = global.sky_horizon.rgb * vec3<f32>(0.30, 0.38, 0.48);
    let dawn_haze = vec3<f32>(0.78, 0.48, 0.28);

    var fog_col = mix(night_haze, day_haze, day);
    fog_col = mix(fog_col, dawn_haze, dawn * 0.22 + scatter);

    // Hard safety against the white-band effect.
    fog_col = min(fog_col, vec3<f32>(0.36, 0.46, 0.58));

    // Preserve terrain contrast: fog can soften, but not brighten the scene into a stripe.
    let source_luma = max(vv_luma(color), 0.025);
    let fog_luma = max(vv_luma(fog_col), 0.025);
    let luma_cap = min(1.0, (source_luma + 0.08) / fog_luma);
    fog_col *= luma_cap;

    return mix(color, fog_col, fog_f);
}
