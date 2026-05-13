#include "include/math/constants.wgsl"

fn vv_sun_color() -> vec3<f32> {
    let sun_elev = normalize(global.sun_dir.xyz).y;
    return mix(
        vec3<f32>(1.10, 0.55, 0.18),
        vec3<f32>(1.24, 1.17, 0.96),
        vv_saturate(sun_elev * 2.6 + 0.32)
    ) * global.sky_zenith.w;
}

fn vv_sun_wrap(normal: vec3<f32>, sun_dir: vec3<f32>) -> f32 {
    return max(dot(normal, sun_dir) * 0.86 + 0.14, 0.0);
}

