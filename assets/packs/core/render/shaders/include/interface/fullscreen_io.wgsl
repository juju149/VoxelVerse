struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

fn vv_fullscreen_triangle(vertex_index: u32) -> FullscreenVertexOut {
    var p = vec2<f32>(-1.0, 1.0);
    if vertex_index == 0u {
        p = vec2<f32>(-1.0, -3.0);
    } else if vertex_index == 1u {
        p = vec2<f32>(3.0, 1.0);
    }

    var out: FullscreenVertexOut;
    out.clip_pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);
    return out;
}
