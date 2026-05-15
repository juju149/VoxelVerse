@group(2) @binding(0) var t_albedo: texture_2d_array<f32>;
@group(2) @binding(1) var t_normal: texture_2d_array<f32>;
@group(2) @binding(2) var t_roughness: texture_2d_array<f32>;
@group(2) @binding(3) var s_material: sampler;
@group(2) @binding(4) var<storage, read> material_colors: array<vec4<f32>>;

const VV_EDGE_MIN_U: u32 = 0x00010000u;
const VV_EDGE_MAX_U: u32 = 0x00020000u;
const VV_EDGE_MIN_V: u32 = 0x00040000u;
const VV_EDGE_MAX_V: u32 = 0x00080000u;

fn vv_material_uv(uv: vec2<f32>) -> vec2<f32> {
    return fract(uv);
}

fn vv_sample_voxel_albedo(layer: u32, uv: vec2<f32>, vertex_tint: vec3<f32>, color_only: bool) -> vec3<f32> {
    if layer == VV_VERTEX_COLOR_ONLY {
        return vertex_tint;
    }

    if color_only {
        return material_colors[layer].rgb * vertex_tint;
    }

    return textureSample(t_albedo, s_material, vv_material_uv(uv), i32(layer)).rgb * vertex_tint;
}

fn vv_sample_voxel_roughness(layer: u32, uv: vec2<f32>, color_only: bool) -> f32 {
    if layer == VV_VERTEX_COLOR_ONLY || color_only {
        return 0.74;
    }

    return clamp(textureSample(t_roughness, s_material, vv_material_uv(uv), i32(layer)).r, 0.32, 1.0);
}

fn vv_sample_voxel_normal(layer: u32, uv: vec2<f32>, surface_normal: vec3<f32>, color_only: bool) -> vec3<f32> {
    let n = vv_safe_normalize(surface_normal);
    if layer == VV_VERTEX_COLOR_ONLY || color_only {
        return n;
    }

    let tex_n = textureSample(t_normal, s_material, vv_material_uv(uv), i32(layer)).xyz * 2.0 - vec3<f32>(1.0);
    var reference = vec3<f32>(0.0, 1.0, 0.0);
    if abs(n.y) > 0.85 {
        reference = vec3<f32>(0.0, 0.0, 1.0);
    }

    let tangent = vv_safe_normalize(cross(reference, n));
    let bitangent = vv_safe_normalize(cross(n, tangent));
    let mapped = vv_safe_normalize(tangent * tex_n.x + bitangent * tex_n.y + n * max(tex_n.z, 0.18));

    return vv_safe_normalize(mix(n, mapped, 0.34));
}

fn vv_material_large_scale_variation(albedo: vec3<f32>, world_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let p = world_pos * 0.0075;
    let n = vv_safe_normalize(normal + vv_safe_normalize(world_pos) * 0.35);
    let broad_a = sin(dot(p, vec3<f32>(0.73, 0.31, 0.58)) + dot(n, vec3<f32>(1.7, 0.3, 0.9)));
    let broad_b = sin(dot(p, vec3<f32>(-0.22, 0.68, 0.41)) * 1.37 + dot(n, vec3<f32>(0.4, 1.1, 1.6)));
    let v = 0.5 + 0.25 * broad_a + 0.25 * broad_b;
    let warm = vec3<f32>(1.025, 1.010, 0.985);
    let cool = vec3<f32>(0.975, 0.990, 1.020);
    let tint = mix(cool, warm, vv_saturate(v));
    let strength = 0.020;
    return albedo * mix(vec3<f32>(1.0), tint, strength);
}

fn vv_material_face_variation(albedo: vec3<f32>, world_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let v = vv_face_variation(world_pos, normal);
    return albedo * (0.985 + v * 0.03);
}

fn vv_material_edge_mask(edge: bool, distance: f32) -> f32 {
    if edge {
        return 1.0 - smoothstep(0.0, 0.075, distance);
    }
    return 0.0;
}

fn vv_material_edge_contact(packed_tex_index: u32, uv: vec2<f32>) -> f32 {
    let t = vv_material_uv(uv);
    var contact = 0.0;
    contact = max(contact, vv_material_edge_mask((packed_tex_index & VV_EDGE_MIN_U) != 0u, t.x));
    contact = max(contact, vv_material_edge_mask((packed_tex_index & VV_EDGE_MAX_U) != 0u, 1.0 - t.x));
    contact = max(contact, vv_material_edge_mask((packed_tex_index & VV_EDGE_MIN_V) != 0u, t.y));
    contact = max(contact, vv_material_edge_mask((packed_tex_index & VV_EDGE_MAX_V) != 0u, 1.0 - t.y));
    return contact;
}

fn vv_material_apply_block_contact(albedo: vec3<f32>, packed_tex_index: u32, uv: vec2<f32>, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let contact = vv_material_edge_contact(packed_tex_index, uv);
    if contact <= 0.0 {
        return albedo;
    }

    let up = vv_planet_up(world_pos);
    let side = vv_saturate(1.0 - abs(dot(normal, up)));
    let shade = mix(0.984, 0.958, side) - contact * 0.018;
    return albedo * mix(1.0, shade, contact);
}
