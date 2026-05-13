fn vv_forward_scatter(view_dir: vec3<f32>) -> f32 {
    let sun_dir = normalize(global.sun_dir.xyz);
    let sun_align = max(dot(view_dir, sun_dir), 0.0);

    let tight = pow(sun_align, 9.0) * 0.46;
    let wide = pow(sun_align, 2.2) * 0.12;

    return (tight + wide) * global.sky_zenith.w;
}

fn vv_horizon_scatter(view_dir: vec3<f32>, world_pos: vec3<f32>) -> f32 {
    let planet_up = normalize(world_pos);
    let horizon = pow(1.0 - abs(dot(view_dir, planet_up)), 2.0);
    let sun_elev = normalize(global.sun_dir.xyz).y;
    let day = clamp(sun_elev * 4.0 + 0.15, 0.0, 1.0);

    return horizon * 0.18 * day;
}
