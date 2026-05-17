const VV_DEBUG_DISABLED: u32 = 0u;
const VV_DEBUG_VERTEX_COLOR: u32 = 1u;
const VV_DEBUG_WORLD_NORMAL: u32 = 2u;
const VV_DEBUG_MATERIAL_LAYER: u32 = 3u;
const VV_DEBUG_UV: u32 = 4u;
const VV_DEBUG_LOD_ALPHA: u32 = 5u;
const VV_DEBUG_CHUNK_KIND: u32 = 6u;
const VV_DEBUG_SHADOW_FACTOR: u32 = 7u;
const VV_DEBUG_DEPTH: u32 = 8u;
const VV_DEBUG_WORLD_POSITION_BANDS: u32 = 9u;

fn vv_debug_material_layer_color(layer: u32) -> vec3<f32> {
    let f = f32(layer % 997u);
    return fract(vec3<f32>(
        f * 0.1031,
        f * 0.1137 + 0.19,
        f * 0.1371 + 0.37
    ));
}

fn vv_debug_world_position_bands(world_pos: vec3<f32>) -> vec3<f32> {
    let bands = fract(abs(world_pos) * 0.05);
    return bands;
}