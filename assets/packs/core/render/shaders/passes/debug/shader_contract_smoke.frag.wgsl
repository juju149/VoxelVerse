#include "include/interface/global.wgsl"
#include "include/interface/local.wgsl"
#include "include/interface/fullscreen_io.wgsl"
#include "include/interface/terrain_io.wgsl"
#include "include/interface/material_sample.wgsl"
#include "include/interface/lighting_sample.wgsl"
#include "include/interface/debug_modes.wgsl"
#include "include/voxel/dither.wgsl"

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let material = vv_debug_material(vec3<f32>(in.uv, 0.5), vec3<f32>(0.0, 1.0, 0.0));
    let lighting = vv_basic_planet_lighting(material, vv_camera_position() + vec3<f32>(0.0, 1.0, 0.0));
    let color = vv_apply_lighting(material, lighting);
    let layer_color = vv_debug_material_layer_color(u32(vv_dither_threshold(in.clip_pos) * 64.0));
    return vec4<f32>(mix(color, layer_color, 0.25), 1.0);
}