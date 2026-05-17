#include "include/interface/terrain_io.wgsl"
#include "include/interface/material_atlas.wgsl"

@fragment
fn fs_main(in: TerrainVertexOut) -> @location(0) vec4<f32> {
    let material = vv_sample_material(
        select(VV_VERTEX_COLOR_ONLY, in.packed_tex_index, in.packed_tex_index != 0u),
        in.uv,
        in.color,
        vec3<f32>(0.0, 0.0, 1.0)
    );
    return vec4<f32>(material.albedo, material.alpha);
}
