fn vv_specular_lite(normal: vec3<f32>, view_dir: vec3<f32>, sun_dir: vec3<f32>, roughness: f32) -> f32 {
    let half_v = normalize(sun_dir + view_dir);
    let ndoth = max(dot(normal, half_v), 0.0);
    let gloss = max(34.0 * (1.0 - roughness * roughness), 1.0);
    return pow(ndoth, gloss) * (1.0 - roughness) * 0.35;
}

