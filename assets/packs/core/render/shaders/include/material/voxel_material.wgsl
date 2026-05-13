@group(2) @binding(0) var t_albedo: texture_2d_array<f32>;
@group(2) @binding(1) var t_normal: texture_2d_array<f32>;
@group(2) @binding(2) var t_roughness: texture_2d_array<f32>;
@group(2) @binding(3) var s_material: sampler;
@group(2) @binding(4) var<storage, read> material_colors: array<vec4<f32>>;

fn vv_sample_voxel_albedo(layer: u32, uv: vec2<f32>, tint: vec3<f32>, color_only: bool) -> vec3<f32> {
    if layer == VV_VERTEX_COLOR_ONLY {
        return tint;
    }
    if color_only {
        return material_colors[layer].rgb * tint;
    }
    return textureSample(t_albedo, s_material, uv, i32(layer)).rgb * tint;
}

fn vv_sample_voxel_roughness(layer: u32, uv: vec2<f32>, color_only: bool) -> f32 {
    if layer == VV_VERTEX_COLOR_ONLY || color_only {
        return 0.72;
    }
    return textureSample(t_roughness, s_material, uv, i32(layer)).r;
}

