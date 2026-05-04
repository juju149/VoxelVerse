fn vv_cartoon_sky_color() -> vec3<f32> {
    return vec3<f32>(0.50, 0.68, 0.98);
}

fn vv_cartoon_ambient() -> vec3<f32> {
    return vec3<f32>(0.34, 0.42, 0.58);
}

fn vv_cartoon_sun_dir() -> vec3<f32> {
    return safe_normalize(vec3<f32>(-0.52, 0.84, -0.30));
}

fn vv_toon_tonemap(color: vec3<f32>) -> vec3<f32> {
    let mapped = color / (color + vec3<f32>(0.82));
    return clamp(mapped * 1.34, vec3<f32>(0.0), vec3<f32>(1.0));
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
    let side_shadow = saturate(1.0 - topness);

    let ambient = mix(vec3<f32>(0.22, 0.24, 0.30), vv_cartoon_ambient(), topness) * 0.70;
    let direct_band = smoothstep(0.08, 0.82, n_dot_l);
    let direct = vec3<f32>(1.0, 0.88, 0.68) * (0.16 + direct_band * 0.78);

    let rim = pow(1.0 - saturate(dot(n, v)), 2.8) * 0.055;
    let gloss = pow(1.0 - saturate(roughness), 2.0);
    let spec = pow(saturate(dot(reflect(-l, n), v)), mix(28.0, 96.0, gloss)) * gloss * 0.055;

    let clean_ao = mix(1.0, saturate(ao), 0.68);

    var lit = albedo * (ambient + direct) * clean_ao;
    lit *= mix(1.0, 0.86, side_shadow * 0.18);
    lit += vv_cartoon_sky_color() * rim;
    lit += vec3<f32>(spec);
    lit += emission;

    lit = vv_saturate_color(lit, 1.34);
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