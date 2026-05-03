struct UiVertexIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>,
}

struct UiVertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
}

@vertex
fn vs_ui(in: UiVertexIn) -> UiVertexOut {
    var out: UiVertexOut;
    out.clip_pos = vec4<f32>(in.pos, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.params = in.params;
    return out;
}

@fragment
fn fs_ui(in: UiVertexOut) -> @location(0) vec4<f32> {
    return in.color;
}