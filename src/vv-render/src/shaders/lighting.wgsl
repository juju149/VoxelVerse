fn vv_wrapped_lambert(n_dot_l: f32, wrap: f32) -> f32 {
    return saturate((n_dot_l + wrap) / (1.0 + wrap));
}

fn vv_fresnel(n_dot_v: f32, power: f32) -> f32 {
    return pow(1.0 - saturate(n_dot_v), power);
}

fn vv_saturate_color(color: vec3<f32>, amount: f32) -> vec3<f32> {
    let luma = luminance(color);
    return mix(vec3<f32>(luma), color, amount);
}

fn apply_planetary_lighting(
    albedo: vec3<f32>,
    emission: vec3<f32>,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    shadow_pos: vec3<f32>,
    ao: f32,
    ao_strength: f32,
    roughness: f32,
    surface_response: f32,
    specular_strength: f32,
) -> vec3<f32> {
    let N = safe_normalize(normal);
    let V = safe_normalize(view_dir);
    let up = local_up_at(world_pos);

    let L = sun_direction_world();
    let M = moon_direction_world();

    let sun_height = local_sun_height(up);
    let day_amount = local_day_amount(up);
    let night_amount = local_night_amount(up);
    let twilight_amount = local_twilight_amount(up);

    let n_dot_l = saturate(dot(N, L));
    let n_dot_m = saturate(dot(N, M));
    let n_dot_v = saturate(dot(N, V));

    let ao_direct = mix(1.0, ao, ao_strength);
    let ao_ambient = mix(ao_direct, ao_direct * ao_direct, 0.42);

    let shadow = shadow_visibility(shadow_pos, n_dot_l);
    let shadow_soft = mix(shadow, 1.0, twilight_amount * 0.22);

    let hemi = dot(N, up) * 0.5 + 0.5;

    let day_sky = vv_saturate_color(global.atmosphere.sky_color.xyz, 1.18) * 0.92;
    let day_ground = vv_saturate_color(global.atmosphere.ground_ambient_color.xyz, 1.08) * 1.05;

    let night_sky = vec3<f32>(0.012, 0.020, 0.075) + global.atmosphere.moon_color.xyz * 0.15;
    let night_ground = vec3<f32>(0.006, 0.009, 0.030) + global.atmosphere.moon_color.xyz * 0.045;

    let sky_ambient = mix(night_sky, day_sky, day_amount);
    let ground_ambient = mix(night_ground, day_ground, day_amount);

    var ambient = mix(ground_ambient, sky_ambient, hemi);
    ambient = ambient * (0.72 + surface_response * 0.20) * ao_ambient;

    let twilight_color = vv_saturate_color(global.atmosphere.horizon_glow_color.xyz, 1.25);
    ambient = ambient + twilight_color * twilight_amount * 0.24 * ao_ambient;

    let sun_wrap = mix(0.46, 0.24, day_amount);
    let soft_sun = vv_wrapped_lambert(n_dot_l, sun_wrap);

    let sun_gate = smoothstep(-0.08, 0.10, sun_height);

    let sun_direct = global.atmosphere.sun_color.xyz
        * soft_sun
        * shadow_soft
        * day_amount
        * sun_gate
        * ao_direct;

    let moon_direct = global.atmosphere.moon_color.xyz
        * vv_wrapped_lambert(n_dot_m, 0.28)
        * night_amount
        * 0.30
        * ao_direct;

    let shadow_fill = global.atmosphere.shadow_tint_color.xyz
        * (1.0 - shadow)
        * day_amount
        * sun_gate
        * (0.22 + n_dot_l * 0.32);

    let twilight_fill = twilight_color
        * twilight_amount
        * saturate(1.0 - n_dot_l)
        * 0.22;

    let back_sun = saturate(-dot(N, L));
    let rim_day = global.atmosphere.sky_color.xyz
        * vv_fresnel(n_dot_v, 2.25)
        * mix(0.055, 0.20, back_sun)
        * day_amount;

    let rim_twilight = twilight_color
        * vv_fresnel(n_dot_v, 2.0)
        * twilight_amount
        * 0.11;

    let rim_night = global.atmosphere.moon_color.xyz
        * vv_fresnel(n_dot_v, 2.7)
        * night_amount
        * 0.065;

    let gloss = pow(1.0 - saturate(roughness), 2.0);
    let R = reflect(-L, N);

    let specular = global.atmosphere.sun_color.xyz
        * shadow_soft
        * day_amount
        * sun_gate
        * pow(saturate(dot(R, V)), mix(14.0, 110.0, gloss))
        * gloss
        * specular_strength;

    let lighting =
        sun_direct
        + moon_direct
        + ambient
        + shadow_fill
        + twilight_fill
        + rim_day
        + rim_twilight
        + rim_night;

    let lit = albedo * lighting + specular + emission;

    return max(lit, vec3<f32>(0.0));
}

fn apply_planetary_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let dist = distance(global.camera_pos.xyz, world_pos);
    let fog_start = max(global.atmosphere.sky_params.x, 0.0);
    let fog_density = max(global.atmosphere.fog_color_density.w, 0.0);

    let fog_range = max(dist - fog_start, 0.0);
    var fog_factor = 1.0 - exp(-(fog_range * fog_density) * (fog_range * fog_density * 0.45));

    let camera_air = camera_atmosphere_amount();
    let pixel_altitude = max(distance(world_pos, planet_center()) - planet_radius_m(), 0.0);
    let pixel_air = atmosphere_amount_at_altitude(pixel_altitude);
    let air_amount = min(camera_air, pixel_air);

    let up = local_up_at(world_pos);
    let day_amount = local_day_amount(up);
    let night_amount = local_night_amount(up);
    let twilight_amount = local_twilight_amount(up);

    fog_factor = fog_factor * air_amount;
    fog_factor = fog_factor * mix(0.70, 1.28, twilight_amount);

    let day_fog = global.atmosphere.fog_color_density.xyz;
    let night_fog = vec3<f32>(0.010, 0.014, 0.050);
    let twilight_fog = global.atmosphere.horizon_glow_color.xyz;

    var fog_color = mix(night_fog, day_fog, day_amount);
    fog_color = mix(fog_color, twilight_fog, twilight_amount * 0.42);
    fog_color = fog_color + global.atmosphere.moon_color.xyz * night_amount * 0.035;

    return mix(color, fog_color, saturate(fog_factor));
}