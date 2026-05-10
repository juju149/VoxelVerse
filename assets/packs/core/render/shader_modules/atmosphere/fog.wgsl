fn vv_apply_curvature_fog(color: vec3<f32>, camera_pos: vec3<f32>, world_pos: vec3<f32>, fog_density: f32) -> vec3<f32> {
    let dist = distance(camera_pos, world_pos);
    let fog_factor = 1.0 - exp(-(dist * fog_density) * (dist * fog_density * 0.5));
    let fog_color = mix(vec3<f32>(0.25, 0.46, 0.86), vec3<f32>(0.72, 0.82, 1.0), 0.25);
    return mix(color, fog_color, clamp(fog_factor, 0.0, 1.0));
}

