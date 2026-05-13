fn vv_planet_up(world_pos: vec3<f32>) -> vec3<f32> {
    return vv_safe_normalize(world_pos);
}

fn vv_hemisphere_ambient(normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);

    let sky_up = max(dot(normal, up), 0.0);
    let sky_side = vv_saturate(1.0 - sky_up * 1.35);
    let ground_up = max(dot(normal, -up), 0.0);

    let sky_top = global.sky_zenith.rgb * 0.44 * mix(0.82, 1.42, sky_up);
    let horizon = global.sky_horizon.rgb * 0.20;
    let ground = vec3<f32>(0.13, 0.10, 0.065);

    let day = vv_day_factor();
    let night = vv_night_factor();

    let day_ambient = sky_top * sky_up + horizon * sky_side + ground * ground_up;
    let night_ambient = vec3<f32>(0.025, 0.032, 0.060) * (0.55 + sky_up * 0.45);

    return mix(day_ambient, night_ambient, night * 0.92) * mix(0.55, 1.0, day);
}

fn vv_micro_bounce(normal: vec3<f32>, world_pos: vec3<f32>, albedo: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);
    let ground_facing = max(dot(normal, -up), 0.0);
    return albedo * vec3<f32>(0.055, 0.044, 0.030) * ground_facing;
}
