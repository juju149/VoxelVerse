fn vv_sun_lighting(normal: vec3<f32>, sun_dir: vec3<f32>, roughness: f32) -> vec3<f32> {
    let sun_color = vec3<f32>(1.25, 1.12, 0.82);
    let ndotl = max(dot(normal, normalize(sun_dir)) * 0.82 + 0.18, 0.0);
    return sun_color * ndotl * mix(1.05, 0.62, roughness);
}

