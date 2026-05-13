fn vv_schlick_fresnel(cos_theta: f32, f0: f32) -> f32 {
    let c = 1.0 - clamp(cos_theta, 0.0, 1.0);
    return f0 + (1.0 - f0) * c * c * c * c * c;
}

