@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> SkyOut {
    var out: SkyOut;
    var pos = vec2<f32>(-1.0, -3.0);

    switch vertex_index {
        case 0u: {
            pos = vec2<f32>(-1.0, -3.0);
        }
        case 1u: {
            pos = vec2<f32>(-1.0, 1.0);
        }
        default: {
            pos = vec2<f32>(3.0, 1.0);
        }
    }

    out.clip_pos = vec4<f32>(pos, 0.0, 1.0);
    out.ndc = pos;
    return out;
}

fn reconstruct_sky_ray(ndc: vec2<f32>) -> vec3<f32> {
    let clip = vec4<f32>(ndc, 1.0, 1.0);
    let world_far = global.inv_view_proj * clip;
    return safe_normalize(world_far.xyz / world_far.w - global.camera_pos.xyz);
}

fn atmospheric_sky_gradient(ray_dir: vec3<f32>, up: vec3<f32>, day_amount: f32) -> vec3<f32> {
    let raw_sky_t = saturate(dot(ray_dir, up));
    let horizon_power = max(global.atmosphere.sky_params.y, 0.1);
    let sky_t = pow(raw_sky_t, horizon_power);

    let horizon_color = global.atmosphere.horizon_glow_color.xyz;
    let mid_color = global.atmosphere.sky_color.xyz;
    let zenith_color = global.atmosphere.zenith_color.xyz;

    let mid_blend = smoothstep(0.035, 0.56, sky_t);
    let zenith_blend = smoothstep(0.34, 1.0, sky_t);

    var day_sky = mix(horizon_color, mid_color, mid_blend);
    day_sky = mix(day_sky, zenith_color, zenith_blend);

    let night_horizon = vec3<f32>(0.024, 0.030, 0.095);
    let night_mid = vec3<f32>(0.012, 0.020, 0.070);
    let night_top = vec3<f32>(0.004, 0.009, 0.038);

    var night_sky = mix(night_horizon, night_mid, mid_blend);
    night_sky = mix(night_sky, night_top, zenith_blend);

    var color = mix(night_sky, day_sky, day_amount);

    let horizon_band = horizon_amount_for_ray(ray_dir, up);
    color += horizon_color * horizon_band * mix(0.08, 0.26, day_amount);

    return max(color, vec3<f32>(0.0));
}

fn sun_glow_planetary(ray_dir: vec3<f32>, up: vec3<f32>, day_amount: f32) -> vec3<f32> {
    let sun_dir = sun_direction_world();
    let sun_cos = saturate(dot(ray_dir, sun_dir));
    let sun_height = local_sun_height(up);

    let sun_visibility = smoothstep(-0.08, 0.12, sun_height);

    let bloom = pow(sun_cos, 7.0) * 0.14;
    let halo = pow(sun_cos, 26.0) * 0.46;
    let disk = pow(sun_cos, 820.0) * 2.35;

    return global.atmosphere.sun_color.xyz
        * (bloom + halo + disk)
        * sun_visibility
        * mix(0.30, 1.0, day_amount);
}

fn twilight_glow_planetary(ray_dir: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let sun_dir = sun_direction_world();
    let horizon_mask = horizon_amount_for_ray(ray_dir, up);
    let twilight = local_twilight_amount(up);
    let sun_facing = pow(saturate(dot(ray_dir, sun_dir) * 0.5 + 0.5), 4.0);

    return global.atmosphere.horizon_glow_color.xyz
        * horizon_mask
        * twilight
        * sun_facing
        * 1.85;
}

fn moon_glow_planetary(ray_dir: vec3<f32>, night_amount: f32) -> vec3<f32> {
    let moon_dir = moon_direction_world();
    let moon_cos = saturate(dot(ray_dir, moon_dir));

    let disk = pow(moon_cos, 760.0) * 2.1;
    let halo = pow(moon_cos, 42.0) * 0.24;

    return global.atmosphere.moon_color.xyz * (disk + halo) * night_amount;
}

fn fs_atmosphere_sky(ray_dir: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let day_amount = local_day_amount(up);
    let night_amount = local_night_amount(up);

    var color = atmospheric_sky_gradient(ray_dir, up, day_amount);
    color += twilight_glow_planetary(ray_dir, up);
    color += sun_glow_planetary(ray_dir, up, day_amount);
    color += moon_glow_planetary(ray_dir, night_amount);
    color += star_field(ray_dir, night_amount) * 0.50;

    return color;
}

fn fs_space_sky(ray_dir: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    var color = space_background(ray_dir);
    color += sun_glow_planetary(ray_dir, up, 1.0) * 0.72;
    color += moon_glow_planetary(ray_dir, 1.0) * 0.70;
    return color;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
    let ray_dir = reconstruct_sky_ray(in.ndc);
    let up = camera_local_up();

    let atmosphere_amount = camera_atmosphere_amount();

    let atmosphere_sky = fs_atmosphere_sky(ray_dir, up);
    let space_sky = fs_space_sky(ray_dir, up);

    let color = mix(space_sky, atmosphere_sky, atmosphere_amount);

    return vec4<f32>(encode_final_color(color), 1.0);
}