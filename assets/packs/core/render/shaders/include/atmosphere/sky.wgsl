// Studio-Ghibli inspired sky.
// Goals: painterly gradients, warm horizon bands, gentle dawn pastels,
// crisp stars at night. Cheap math, no procedural haze loops.

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

// Sparse, twinkly stars. Slow drift with sun azimuth.
fn vv_sky_star_field(uv: vec2<f32>, sun_xz: vec2<f32>) -> f32 {
    let grid = 230.0;
    let suv = uv + sun_xz * 0.35;
    let cell = floor(suv * grid);
    let frac_uv = fract(suv * grid);

    let star_pos = vv_sky_hash22(cell);
    let brightness = vv_sky_hash21(cell * 9.1);
    let twinkle = 0.85 + 0.15 * sin(global.render_params.x * 0.8 + brightness * 31.4);
    let size = 0.011 + brightness * 0.018;
    let d = length(frac_uv - star_pos);
    let star = smoothstep(size, 0.0, d) * brightness * twinkle;

    // Cull faint stars to keep the night clean.
    return select(0.0, star, brightness > 0.62);
}

// Painterly sky core. `uv.y == 0` is screen top, `uv.y == 1` is screen bottom.
fn vv_sky_color(uv: vec2<f32>) -> vec3<f32> {
    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_elev = sun_dir.y;

    // Horizon factor: 0 at zenith, 1 at horizon.
    let h = vv_sky_saturate(1.0 - uv.y);

    // Time-of-day weights.
    let day  = vv_sky_saturate(sun_elev * 3.2 + 0.22);
    let dawn = vv_sky_saturate(1.0 - abs(sun_elev) * 5.0)
             * vv_sky_saturate(sun_elev * 5.5 + 0.85);
    let night = vv_sky_saturate(-sun_elev * 4.0 - 0.10);

    // Painterly two-stop vertical gradient with a soft Ghibli curve.
    let curve = pow(h, mix(0.95, 0.55, day));
    var sky = mix(global.sky_zenith.rgb, global.sky_horizon.rgb, curve);

    // Warm cream band hugging the horizon during the day.
    let cream = vec3<f32>(1.00, 0.92, 0.78);
    let cream_band = vv_sky_smooth5((h - 0.55) * 2.2) * day * 0.18;
    sky = mix(sky, cream, cream_band);

    // Dawn / dusk: rosy gold horizon and lavender band higher up.
    let rose   = vec3<f32>(1.05, 0.55, 0.42);
    let lilac  = vec3<f32>(0.62, 0.42, 0.70);
    let rose_band  = vv_sky_smooth5((h - 0.45) * 2.0) * dawn;
    let lilac_band = vv_sky_smooth5((h - 0.15) * 1.8)
                   * (1.0 - vv_sky_smooth5((h - 0.55) * 2.0)) * dawn * 0.55;
    sky = mix(sky, rose,  rose_band  * 0.45);
    sky = mix(sky, lilac, lilac_band * 0.30);

    // Night: deep cobalt zenith + gentle moonlit horizon.
    if night > 0.01 {
        let night_zen  = vec3<f32>(0.018, 0.028, 0.055);
        let night_horiz = vec3<f32>(0.06, 0.075, 0.13);
        let night_sky = mix(night_zen, night_horiz, vv_sky_smooth5(h));
        sky = mix(sky, night_sky, night);

        let stars = vv_sky_star_field(uv, sun_dir.xz)
                  * vv_sky_smooth5(h * 1.7 - 0.18);
        sky += vec3<f32>(0.92, 0.95, 1.0) * stars * night * 1.4;
    }

    return sky;
}
