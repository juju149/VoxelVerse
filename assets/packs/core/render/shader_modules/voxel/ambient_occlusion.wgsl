fn vv_voxel_ao(vertex_color: vec3<f32>) -> f32 {
    return clamp(dot(vertex_color, vec3<f32>(0.3333)), 0.25, 1.25);
}

