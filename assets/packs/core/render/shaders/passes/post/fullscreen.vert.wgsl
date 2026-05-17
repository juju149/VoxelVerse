#include "include/interface/fullscreen_io.wgsl"

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOut {
    return vv_fullscreen_triangle(vertex_index);
}