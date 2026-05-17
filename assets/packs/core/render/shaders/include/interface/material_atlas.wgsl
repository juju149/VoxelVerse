#include "include/interface/global.wgsl"
#include "include/interface/material_sample.wgsl"

@group(2) @binding(0) var vv_material_albedo: texture_2d_array<f32>;
@group(2) @binding(1) var vv_material_normal: texture_2d_array<f32>;
@group(2) @binding(2) var vv_material_roughness: texture_2d_array<f32>;
@group(2) @binding(3) var vv_material_sampler: sampler;
@group(2) @binding(4) var<storage, read> vv_material_flat_colors: array<vec4<f32>>;

fn vv_material_uv(uv: vec2<f32>) -> vec2<f32> {
    return uv;
}

fn vv_material_flat_color(layer: u32) -> vec3<f32> {
    if layer == VV_VERTEX_COLOR_ONLY {
        return vec3<f32>(1.0);
    }
    return vv_material_flat_colors[layer].rgb;
}

fn vv_material_normal_from_texture(sampled: vec3<f32>, geometry_normal: vec3<f32>) -> vec3<f32> {
    let tangent_space = sampled * 2.0 - vec3<f32>(1.0);
    let n = vv_safe_normalize(geometry_normal);
    let reference = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n.y) > 0.92);
    let tangent = vv_safe_normalize(cross(reference, n));
    let bitangent = vv_safe_normalize(cross(n, tangent));
    return vv_safe_normalize(tangent * tangent_space.x + bitangent * tangent_space.y + n * tangent_space.z);
}

fn vv_sample_material(
    packed_tex_index: u32,
    uv: vec2<f32>,
    vertex_color: vec3<f32>,
    geometry_normal: vec3<f32>,
) -> MaterialSample {
    let layer = vv_material_layer(packed_tex_index);
    if layer == VV_VERTEX_COLOR_ONLY {
        return vv_debug_material(vertex_color, geometry_normal);
    }

    if vv_has_quality_flag(VV_Q_COLOR_ONLY) {
        return vv_make_material_sample(
            clamp(vertex_color * vv_material_flat_color(layer), vec3<f32>(0.0), vec3<f32>(1.0)),
            geometry_normal,
            0.82,
            0.0,
            1.0
        );
    }

    let atlas_uv = vv_material_uv(uv);
    let layer_index = i32(layer);
    let albedo = textureSample(vv_material_albedo, vv_material_sampler, atlas_uv, layer_index);
    let normal_texel = textureSample(vv_material_normal, vv_material_sampler, atlas_uv, layer_index).rgb;
    let roughness = textureSample(vv_material_roughness, vv_material_sampler, atlas_uv, layer_index).r;

    return vv_make_material_sample(
        albedo.rgb * clamp(vertex_color, vec3<f32>(0.0), vec3<f32>(2.0)),
        vv_material_normal_from_texture(normal_texel, geometry_normal),
        roughness,
        0.0,
        albedo.a
    );
}
