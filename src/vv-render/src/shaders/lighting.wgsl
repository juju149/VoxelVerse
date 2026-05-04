fn vv_cartoon_sky_color() -> vec3<f32> {
    return vec3<f32>(0.58, 0.76, 1.0);
}

fn vv_cartoon_ambient() -> vec3<f32> {
    return vec3<f32>(0.42, 0.52, 0.68);
}

fn vv_cartoon_sun_dir() -> vec3<f32> {
    return safe_normalize(vec3<f32>(-0.48, 0.82, -0.34));
}

fn vv_toon_tonemap(color: vec3<f32>) -> vec3<f32> {
    let mapped = color / (color + vec3<f32>(0.72));
    return clamp(mapped * 1.22, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn vv_cartoon_lighting(
    albedo: vec3<f32>,
    emission: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    ao: f32,
    roughness: f32,
) -> vec3<f32> {
    let n = safe_normalize(normal);
    let v = safe_normalize(view_dir);
    let l = vv_cartoon_sun_dir();

    let n_dot_l = saturate(dot(n, l));
    let topness = saturate(n.y * 0.5 + 0.5);

    let ambient = mix(vec3<f32>(0.30, 0.34, 0.42), vv_cartoon_ambient(), topness) * 0.78;
    let direct_band = smoothstep(0.04, 0.78, n_dot_l);
    let direct = vec3<f32>(1.0, 0.92, 0.76) * (0.20 + direct_band * 0.72);

    let rim = pow(1.0 - saturate(dot(n, v)), 2.6) * 0.075;
    let gloss = pow(1.0 - saturate(roughness), 2.0);
    let spec = pow(saturate(dot(reflect(-l, n), v)), mix(24.0, 90.0, gloss)) * gloss * 0.07;

    let clean_ao = mix(1.0, saturate(ao), 0.52);

    var lit = albedo * (ambient + direct) * clean_ao;
    lit += vv_cartoon_sky_color() * rim;
    lit += vec3<f32>(spec);
    lit += emission;

    lit = vv_saturate_color(lit, 1.22);
    return max(lit, vec3<f32>(0.0));
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
    return vv_cartoon_lighting(albedo, emission, normal, view_dir, ao, roughness);
}

fn apply_planetary_fog(color: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    return color;
}

fn encode_final_color(color: vec3<f32>) -> vec3<f32> {
    let mapped = vv_toon_tonemap(color);
    return pow(mapped, vec3<f32>(1.0 / 2.2));
}