struct LightingSample {
    direct: vec3<f32>,
    ambient: vec3<f32>,
    shadow: f32,
}

fn vv_basic_lighting(material: MaterialSample, world_pos: vec3<f32>) -> LightingSample {
    let sun_dir = vv_sun_direction();
    let ndotl = max(dot(material.normal, sun_dir), 0.0);

    let sun_color = vec3<f32>(1.0, 0.94, 0.82);
    let sky_color = max(global.sky_zenith.rgb, vec3<f32>(0.62, 0.70, 0.82));
    let ground_color = vec3<f32>(0.20, 0.18, 0.14);

    let up = vv_safe_normalize(world_pos);
    let sky_facing = max(dot(material.normal, up), 0.0);
    let ground_facing = max(dot(material.normal, -up), 0.0);

    let direct = sun_color * (0.18 + ndotl * 0.58);
    let ambient = sky_color * (0.18 + sky_facing * 0.12) + ground_color * ground_facing * 0.08;

    return LightingSample(direct, ambient, 1.0);
}

fn vv_apply_lighting(material: MaterialSample, lighting: LightingSample) -> vec3<f32> {
    return material.albedo * (lighting.direct + lighting.ambient);
}