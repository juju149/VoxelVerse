struct FullscreenVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    var out: FullscreenVertexOut;
    out.clip_pos = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = out.clip_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return out;
}

