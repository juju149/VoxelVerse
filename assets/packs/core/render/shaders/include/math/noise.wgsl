#include "include/math/random.wgsl"

fn vv_noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(vv_hash21(i), vv_hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(vv_hash21(i + vec2<f32>(0.0, 1.0)), vv_hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

fn vv_fbm2d(p: vec2<f32>) -> f32 {
    return vv_noise2d(p) * 0.58
        + vv_noise2d(p * 2.23 + vec2<f32>(4.1, 1.7)) * 0.29
        + vv_noise2d(p * 5.11 + vec2<f32>(1.3, 7.2)) * 0.13;
}

