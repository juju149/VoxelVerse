// Studio-Ghibli clouds: soft cotton shapes, painterly rims.
// Cheap value-noise FBM with a clean coverage cut.

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

// 3-octave FBM. Weights normalized so output stays in [0,1].
fn vv_cloud_fbm2d(p: vec2<f32>) -> f32 {
    return vv_cloud_noise2d(p)                                * 0.55
         + vv_cloud_noise2d(p * 2.17 + vec2<f32>( 4.1, 1.7))  * 0.30
         + vv_cloud_noise2d(p * 4.83 + vec2<f32>( 1.3, 7.2))  * 0.15;
}

// Returns shaped density in [0,1]. Single coverage parameter drives sky cover.
fn vv_cloud_density(uv: vec2<f32>) -> f32 {
    let t = global.render_params.x * global.cloud_params.z;
    // Stretch horizontally so clouds look broader near the zenith.
    let p = uv * vec2<f32>(4.4, 2.1) + vec2<f32>(t, t * 0.16);

    let base = vv_cloud_fbm2d(p);
    let coverage = vv_cloud_saturate(global.cloud_params.w);

    // Map (base - coverage) to a soft puffy mass.
    // Multiplier 2.2 gives clean edges without going chunky.
    let shaped = vv_cloud_smooth5((base - coverage + 0.05) * 2.2);
    return shaped;
}

// Painterly lighting: bright top, soft warm rim, gentle blue underside.
fn vv_cloud_light(density: f32) -> vec3<f32> {
    let sun_elev = normalize(global.sun_dir.xyz).y;
    let day  = vv_cloud_saturate(sun_elev * 3.5 + 0.18);
    let dawn = vv_cloud_saturate(1.0 - abs(sun_elev) * 5.0)
             * vv_cloud_saturate(sun_elev * 5.5 + 0.85);

    // Inner shadow self-attenuates with density; rim stays bright.
    let lit = mix(0.80, 1.05, 1.0 - density);
    let rim = pow(1.0 - density, 3.0) * 0.35;

    let day_col  = vec3<f32>(0.97, 0.96, 0.94);
    let dawn_col = vec3<f32>(1.05, 0.72, 0.52);
    let dusk_warm = mix(day_col, dawn_col, dawn * 0.85);
    let night_col = vec3<f32>(0.10, 0.12, 0.18);

    let base = mix(night_col, dusk_warm * lit, day);
    // Bright rim picks up the sun colour, gives the cotton-puff feeling.
    return base + dusk_warm * rim * day;
}
