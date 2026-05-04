fn vv_cartoon_sky_color() -> vec3<f32> {
    return vec3<f32>(0.62, 0.80, 1.0);
}

fn vv_cartoon_ambient() -> vec3<f32> {
    return vec3<f32>(0.58, 0.66, 0.78);
}

fn vv_cartoon_sun_dir() -> vec3<f32> {
    return safe_normalize(vec3<f32>(-0.42, 0.88, -0.34));
}

fn vv_cartoon_lighting(
    albedo: vec3<f32>,
    emission: vec3<f32>,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    ao: f32,
    roughness: f32,
) -> vec3<f32> {
    let N = safe_normalize(normal);
    let V = safe_normalize(view_dir);
    let L = vv_cartoon_sun_dir();

    let n_dot_l = saturate(dot(N, L));
    let hemi = saturate(N.y * 0.5 + 0.5);

    let ambient = mix(vec3<f32>(0.42, 0.48, 0.56), vv_cartoon_ambient(), hemi);
    let soft_direct = smoothstep(0.02, 0.72, n_dot_l);
    let direct = vec3<f32>(1.0, 0.92, 0.76) * (0.52 + soft_direct * 0.72);

    let rim = pow(1.0 - saturate(dot(N, V)), 2.35) * 0.10;
    let gloss = pow(1.0 - saturate(roughness), 2.0);
    let spec = pow(saturate(dot(reflect(-L, N), V)), mix(18.0, 80.0, gloss)) * gloss * 0.08;

    let clean_ao = mix(1.0, saturate(ao), 0.62);
    let lit = albedo * (ambient + direct) * clean_ao + vv_cartoon_sky_color() * rim + spec + emission;

    // Mario-like candy color. Saturated, readable, not physically moody.
    let saturated = vv_saturate_color(lit, 1.18);
    return max(saturated, vec3<f32>(0.0));
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
    // Lightweight gamma-ish output. No filmic mood, no night blue soup.
    return pow(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
}