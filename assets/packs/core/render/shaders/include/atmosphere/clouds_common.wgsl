fn vv_cloud_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_cloud_smooth5(t: f32) -> f32 {
    let c = vv_cloud_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

fn vv_cloud_hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vv_cloud_noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = vv_cloud_hash21(i);
    let b = vv_cloud_hash21(i + vec2<f32>(1.0, 0.0));
    let c = vv_cloud_hash21(i + vec2<f32>(0.0, 1.0));
    let d = vv_cloud_hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn vv_cloud_fbm2d(p: vec2<f32>) -> f32 {
    return vv_cloud_noise2d(p) * 0.58
        + vv_cloud_noise2d(p * 2.23 + vec2<f32>(4.1, 1.7)) * 0.29
        + vv_cloud_noise2d(p * 5.11 + vec2<f32>(1.3, 7.2)) * 0.13;
}

fn vv_cloud_density(uv: vec2<f32>) -> f32 {
    let t = global.render_params.x * global.cloud_params.z;
    let p = uv * vec2<f32>(4.6, 2.2) + vec2<f32>(t, t * 0.18);

    let base = vv_cloud_fbm2d(p);
    let detail = vv_cloud_noise2d(p * 4.0 + vec2<f32>(9.2, 2.7)) * 0.18;
    let coverage = global.cloud_params.w;

    return clamp((base + detail - coverage) * 2.0, 0.0, 1.0);
}

fn vv_cloud_light(density: f32) -> vec3<f32> {
    let sun_elev = normalize(global.sun_dir.xyz).y;
    let day = clamp(sun_elev * 4.0 + 0.2, 0.0, 1.0);
    let dawn = clamp(1.0 - abs(sun_elev) * 5.5, 0.0, 1.0)
        * clamp(sun_elev * 6.0 + 0.8, 0.0, 1.0);

    let shade = mix(0.52, 0.82, 1.0 - density);

    let day_col = vec3<f32>(0.72, 0.78, 0.86);
    let dawn_col = vec3<f32>(1.0, 0.60, 0.36);
    let night_col = vec3<f32>(0.055, 0.065, 0.13);

    return mix(night_col, mix(day_col, dawn_col, dawn * 0.65), day) * shade;
}
