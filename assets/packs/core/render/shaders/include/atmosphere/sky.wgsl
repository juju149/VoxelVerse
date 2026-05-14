fn vv_sky_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_sky_smooth5(t: f32) -> f32 {
    let c = vv_sky_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

fn vv_sky_hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vv_sky_hash22(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        vv_sky_hash21(p),
        fract(sin(dot(p, vec2<f32>(269.5, 183.3))) * 43758.5453)
    );
}

fn vv_sky_star_field(uv: vec2<f32>, sun_xz: vec2<f32>) -> f32 {
    let grid = 210.0;
    let suv = uv + sun_xz * 0.4;
    let cell = floor(suv * grid);
    let frac_uv = fract(suv * grid);

    let star_pos = vv_sky_hash22(cell);
    let brightness = vv_sky_hash21(cell * 9.1);
    let size = 0.014 + brightness * 0.016;

    return smoothstep(size, 0.0, length(frac_uv - star_pos)) * brightness;
}

fn vv_sky_color(uv: vec2<f32>) -> vec3<f32> {
    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_elev = sun_dir.y;

    let dawn_t = clamp(1.0 - abs(sun_elev) * 5.5, 0.0, 1.0)
        * clamp(sun_elev * 6.0 + 0.8, 0.0, 1.0);

    let night_t = clamp((-sun_elev - 0.10) * 7.0, 0.0, 1.0);
    let horiz_frac = clamp(1.0 - uv.y, 0.0, 1.0);

    var sky = mix(global.sky_horizon.rgb, global.sky_zenith.rgb, pow(horiz_frac, 0.55));

    let band = vv_sky_smooth5((horiz_frac - 0.15) * 3.0)
        * (1.0 - vv_sky_smooth5((horiz_frac - 0.65) * 2.5));

    sky = mix(sky, vec3<f32>(0.50, 0.22, 0.55), band * dawn_t * 0.50);

    let haze = pow(clamp((uv.y - 0.72) * 5.2, 0.0, 1.0), 2.2) * 0.10;
    sky = mix(
        sky,
        mix(global.sky_horizon.rgb * 0.95, vec3<f32>(0.85, 0.58, 0.32), dawn_t),
        haze
    );

    if night_t > 0.02 {
        let stars = vv_sky_star_field(uv, sun_dir.xz)
            * vv_sky_smooth5(horiz_frac * 1.6 - 0.15);

        sky += vec3<f32>(0.93, 0.95, 1.0) * stars * night_t * 1.6;
    }

    return sky;
}
