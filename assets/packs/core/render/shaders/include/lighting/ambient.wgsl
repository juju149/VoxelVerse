fn vv_hemisphere_ambient(normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let planet_up = normalize(world_pos);
    let sky_up = max(dot(normal, planet_up), 0.0);
    let sky_side = clamp(1.0 - sky_up * 1.35, 0.0, 1.0);
    let sky_bot = max(dot(normal, -planet_up), 0.0);
    let sky = global.sky_zenith.rgb * 0.46 * mix(0.82, 1.42, sky_up);
    let horizon = global.sky_horizon.rgb * 0.20;
    let ground = vec3<f32>(0.13, 0.10, 0.065);
    return sky * sky_up + horizon * sky_side + ground * sky_bot;
}

