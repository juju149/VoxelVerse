struct TerrainVertexIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec3<f32>,
    @location(4) tex_index: u32,
}

struct TerrainVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) view_pos: vec3<f32>,
    @location(4) shadow_pos: vec3<f32>,
    @location(5) color: vec3<f32>,
    @location(6) @interpolate(flat) packed_tex_index: u32,
    @location(7) @interpolate(flat) lod_alpha: f32,
}

fn vv_material_layer_from_vertex(in: TerrainVertexOut) -> u32 {
    return in.packed_tex_index & VV_MATERIAL_INDEX_MASK;
}