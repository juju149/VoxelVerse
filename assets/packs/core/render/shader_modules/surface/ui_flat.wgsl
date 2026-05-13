@group(2) @binding(0) var t_albedo:   texture_2d_array<f32>;
@group(2) @binding(3) var s_material: sampler;

const MATERIAL_INDEX_MASK: u32 = 0x0000FFFFu;
const VERTEX_COLOR_ONLY:   u32 = 0x0000FFFFu;

struct UiVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) shadow_pos: vec3<f32>,
    @location(5) color: vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
}

@fragment
fn fs_main(in: UiVertexOut) -> @location(0) vec4<f32> {
    let mat_idx = in.packed_tex_index & MATERIAL_INDEX_MASK;
    // mat_idx == 0 → no material assigned (fallback to vertex color)
    // mat_idx == 0xFFFF → explicit vertex-color sentinel
    if mat_idx == 0u || mat_idx == VERTEX_COLOR_ONLY {
        return vec4<f32>(in.color, 1.0);
    }
    // Atlas layers are 1-based; convert to 0-based array index.
    let tex = textureSample(t_albedo, s_material, in.uv, i32(mat_idx) - 1).rgb;
    // Multiply by vertex color so the per-face brightness shading is applied.
    return vec4<f32>(tex * in.color, 1.0);
}
