#include "include/camera/globals.wgsl"
#include "include/atmosphere/volumetric_fog.wgsl"

struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let alpha = vv_fog_veil_alpha(in.uv);

    if alpha <= 0.0001 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_elev = sun_dir.y;

    let day = clamp(sun_elev * 4.0 + 0.2, 0.0, 1.0);
    let dawn = clamp(1.0 - abs(sun_elev) * 5.5, 0.0, 1.0)
        * clamp(sun_elev * 6.0 + 0.8, 0.0, 1.0);

    // Darker, cinematic haze. No white fog.
    let night_haze = vec3<f32>(0.030, 0.040, 0.065);
    let day_haze = global.sky_horizon.rgb * vec3<f32>(0.52, 0.58, 0.68);
    let dawn_haze = vec3<f32>(0.78, 0.48, 0.28);

    var fog_col = mix(night_haze, day_haze, day);
    fog_col = mix(fog_col, dawn_haze, dawn * 0.18);
    fog_col = min(fog_col, vec3<f32>(0.50, 0.56, 0.66));

    return vec4<f32>(fog_col, alpha);
}
