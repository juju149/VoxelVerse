fn vv_lod_alpha(local_params: vec4<f32>) -> f32 {
    return clamp(local_params.x, 0.0, 1.0);
}

