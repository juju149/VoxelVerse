fn vv_value_noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = vv_hash21(i + vec2<f32>(0.0, 0.0));
    let b = vv_hash21(i + vec2<f32>(1.0, 0.0));
    let c = vv_hash21(i + vec2<f32>(0.0, 1.0));
    let d = vv_hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn vv_fbm2d(p: vec2<f32>) -> f32 {
    var sum = 0.0;
    var amp = 0.56;
    var freq = 1.0;

    for (var i = 0u; i < 4u; i = i + 1u) {
        sum += vv_value_noise2d(p * freq) * amp;
        freq *= 2.17;
        amp *= 0.48;
    }

    return sum;
}

fn vv_cloud_fbm2d(p: vec2<f32>) -> f32 {
    let soft = vv_fbm2d(p);
    let detail = vv_value_noise2d(p * 6.3 + vec2<f32>(12.4, 7.1)) * 0.12;
    return clamp(soft + detail, 0.0, 1.0);
}
