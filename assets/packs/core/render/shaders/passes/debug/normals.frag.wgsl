#include "include/interface/terrain_io.wgsl"

@fragment
fn fs_main(in: TerrainVertexOut) -> @location(0) vec4<f32> {
    return vec4<f32>(normalize(in.world_normal) * 0.5 + vec3<f32>(0.5), 1.0);
}