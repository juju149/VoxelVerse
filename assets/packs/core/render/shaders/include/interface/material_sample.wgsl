struct MaterialSample {
    albedo: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32,
    metallic: f32,
    alpha: f32,
}

fn vv_debug_material(vertex_color: vec3<f32>, geometry_normal: vec3<f32>) -> MaterialSample {
    return MaterialSample(
        clamp(vertex_color, vec3<f32>(0.02), vec3<f32>(1.0)),
        vv_safe_normalize(geometry_normal),
        0.82,
        0.0,
        1.0
    );
}

fn vv_material_layer(packed_tex_index: u32) -> u32 {
    return packed_tex_index & VV_MATERIAL_INDEX_MASK;
}