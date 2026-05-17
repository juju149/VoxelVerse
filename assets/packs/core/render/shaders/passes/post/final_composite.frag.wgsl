#include "include/interface/fullscreen_io.wgsl"

@group(1) @binding(0) var t_scene: texture_2d<f32>;
@group(1) @binding(1) var s_scene: sampler;

@fragment
fn fs_main(in: FullscreenVertexOut) -> @location(0) vec4<f32> {
    let color = textureSample(t_scene, s_scene, clamp(in.uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
    return vec4<f32>(color, 1.0);
}