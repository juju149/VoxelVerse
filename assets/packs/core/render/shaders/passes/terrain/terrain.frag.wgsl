#include "include/camera/globals.wgsl"
#include "include/interface/terrain_io.wgsl"
#include "include/interface/material_sample.wgsl"
#include "include/interface/lighting_sample.wgsl"
#include "include/voxel/dither.wgsl"

@fragment
fn fs_main(in: TerrainVertexOut) -> @location(0) vec4<f32> {
    if in.lod_alpha < 0.999 && vv_dither_threshold(in.clip_pos) > in.lod_alpha {
        discard;
    }

    let geometry_normal = vv_safe_normalize(in.world_normal);
    let material = vv_debug_material(in.color, geometry_normal);
    let lighting = vv_basic_lighting(material, in.world_pos);
    let color = vv_apply_lighting(material, lighting);

    return vec4<f32>(max(color, vec3<f32>(0.0)), material.alpha);
}