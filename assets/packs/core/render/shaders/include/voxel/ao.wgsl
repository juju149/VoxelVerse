fn vv_vertex_ao(vertex_color: vec3<f32>) -> f32 {
    return clamp(dot(vertex_color, vec3<f32>(0.333)), 0.35, 1.15);
}

