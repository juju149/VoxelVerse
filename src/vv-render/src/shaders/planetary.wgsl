fn planet_center() -> vec3<f32> {
    return global.atmosphere.planet_center_radius.xyz;
}

fn planet_radius_m() -> f32 {
    return max(global.atmosphere.planet_center_radius.w, 1.0);
}

fn sun_direction_world() -> vec3<f32> {
    return safe_normalize(global.atmosphere.sun_direction.xyz);
}

fn moon_direction_world() -> vec3<f32> {
    return safe_normalize(global.atmosphere.moon_direction.xyz);
}

fn local_up_at(world_pos: vec3<f32>) -> vec3<f32> {
    return safe_normalize(world_pos - planet_center());
}

fn camera_local_up() -> vec3<f32> {
    return local_up_at(global.camera_pos.xyz);
}

fn camera_altitude_m() -> f32 {
    return max(distance(global.camera_pos.xyz, planet_center()) - planet_radius_m(), 0.0);
}

fn atmosphere_height_m() -> f32 {
    return max(global.atmosphere.atmosphere_params.x, 1.0);
}

fn atmosphere_amount_at_altitude(altitude_m: f32) -> f32 {
    let fade_start = max(global.atmosphere.atmosphere_params.y, 0.0);
    let fade_end = max(global.atmosphere.atmosphere_params.z, fade_start + 1.0);
    return 1.0 - smoothstep(fade_start, fade_end, altitude_m);
}

fn camera_atmosphere_amount() -> f32 {
    return atmosphere_amount_at_altitude(camera_altitude_m());
}

fn terminator_softness() -> f32 {
    return max(global.atmosphere.atmosphere_params.w, 0.01);
}

fn local_sun_height(up: vec3<f32>) -> f32 {
    return dot(up, sun_direction_world());
}

fn local_day_amount(up: vec3<f32>) -> f32 {
    let h = local_sun_height(up);
    let softness = terminator_softness();
    return smoothstep(-softness, softness, h);
}

fn local_night_amount(up: vec3<f32>) -> f32 {
    return 1.0 - local_day_amount(up);
}

fn local_twilight_amount(up: vec3<f32>) -> f32 {
    let h = abs(local_sun_height(up));
    let softness = terminator_softness();
    return 1.0 - smoothstep(softness, softness * 5.0, h);
}

fn horizon_amount_for_ray(ray_dir: vec3<f32>, up: vec3<f32>) -> f32 {
    let h = saturate(dot(ray_dir, up));
    return pow(1.0 - h, 2.35);
}

fn star_field(ray_dir: vec3<f32>, intensity: f32) -> vec3<f32> {
    let p = floor(ray_dir * 1200.0);
    let h = hash13(p);

    let star_mask = select(0.0, 1.0, h > 0.9968);
    let twinkle = mix(0.35, 1.0, hash13(p + vec3<f32>(17.0, 43.0, 91.0)));

    let tiny_p = floor(ray_dir * 2400.0);
    let tiny_h = hash13(tiny_p + vec3<f32>(31.0, 11.0, 73.0));
    let tiny_mask = select(0.0, 1.0, tiny_h > 0.9987);

    let star = star_mask * twinkle;
    let dust = tiny_mask * 0.35;

    return vec3<f32>(star + dust) * global.atmosphere.sky_params.z * intensity;
}

fn space_background(ray_dir: vec3<f32>) -> vec3<f32> {
    let base = vec3<f32>(0.0015, 0.0025, 0.008);
    return base + star_field(ray_dir, 1.0);
}

fn planet_atmosphere_height_m() -> f32 {
    return atmosphere_height_m();
}

fn altitude_m(world_pos: vec3<f32>) -> f32 {
    return max(distance(world_pos, planet_center()) - planet_radius_m(), 0.0);
}