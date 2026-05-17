#include "include/interface/terrain_io.wgsl"

@fragment
fn fs_main(in: TerrainVertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(clamp(in.color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}