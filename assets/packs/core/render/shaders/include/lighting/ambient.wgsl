fn vv_planet_up(world_pos: vec3<f32>) -> vec3<f32> {
    return vv_safe_normalize(world_pos);
}

fn vv_hemisphere_ambient(normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);

    let sky_up = max(dot(normal, up), 0.0);
    let sky_side = vv_saturate(1.0 - sky_up * 1.35);
    let ground_up = max(dot(normal, -up), 0.0);

    let sky_top = global.sky_zenith.rgb * 0.28 * mix(0.76, 1.20, sky_up);
    let horizon = global.sky_horizon.rgb * 0.13;
    let ground = vec3<f32>(0.090, 0.070, 0.045);

    let day = vv_day_factor();
    let night = vv_night_factor();

    let day_ambient = sky_top * sky_up + horizon * sky_side + ground * ground_up;
    let night_ambient = vec3<f32>(0.035, 0.042, 0.066) * (0.60 + sky_up * 0.40);

    return mix(day_ambient, night_ambient, night * 0.88) * mix(0.58, 0.86, day);
}

fn vv_micro_bounce(normal: vec3<f32>, world_pos: vec3<f32>, albedo: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);
    let ground_facing = max(dot(normal, -up), 0.0);
    return albedo * vec3<f32>(0.055, 0.044, 0.030) * ground_facing;
}
