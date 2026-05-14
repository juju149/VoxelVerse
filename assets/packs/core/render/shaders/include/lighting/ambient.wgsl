fn vv_planet_up(world_pos: vec3<f32>) -> vec3<f32> {
    return vv_safe_normalize(world_pos);
}

fn vv_hemisphere_ambient(normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);

    let sky_up = max(dot(normal, up), 0.0);
    let sky_side = vv_saturate(1.0 - sky_up * 1.35);
    let ground_up = max(dot(normal, -up), 0.0);

    let sky_top = global.sky_zenith.rgb * 0.12 * mix(0.70, 1.05, sky_up);
    let horizon = global.sky_horizon.rgb * 0.045;
    let ground = vec3<f32>(0.035, 0.030, 0.024);

    let day = vv_day_factor();
    let night = vv_night_factor();

    let day_ambient = sky_top * sky_up + horizon * sky_side + ground * ground_up;
    let night_ambient = vec3<f32>(0.018, 0.024, 0.040) * (0.55 + sky_up * 0.35);

    return mix(day_ambient, night_ambient, night * 0.88) * mix(0.46, 0.68, day);
}

fn vv_micro_bounce(normal: vec3<f32>, world_pos: vec3<f32>, albedo: vec3<f32>) -> vec3<f32> {
    let up = vv_planet_up(world_pos);
    let ground_facing = max(dot(normal, -up), 0.0);
    return albedo * vec3<f32>(0.022, 0.018, 0.014) * ground_facing;
}
