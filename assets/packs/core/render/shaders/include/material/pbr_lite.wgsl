fn vv_specular_lite(normal: vec3<f32>, view_dir: vec3<f32>, sun_dir: vec3<f32>, roughness: f32) -> f32 {
    let half_v = vv_safe_normalize(sun_dir + view_dir);
    let ndoth = max(dot(normal, half_v), 0.0);
    let perceptual = clamp(roughness, 0.32, 1.0);
    let gloss = max(2.0, 44.0 * (1.0 - perceptual) * (1.0 - perceptual));
    let energy = (1.0 - perceptual) * 0.055;
    return pow(ndoth, gloss) * energy;
}

fn vv_fresnel_schlick(cos_theta: f32, f0: f32) -> f32 {
    let c = 1.0 - clamp(cos_theta, 0.0, 1.0);
    return f0 + (1.0 - f0) * c * c * c * c * c;
}
