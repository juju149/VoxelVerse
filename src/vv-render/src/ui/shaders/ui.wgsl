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

fn rounded_rect_alpha(uv: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let safe_size = max(size, vec2<f32>(1.0, 1.0));
    let r = clamp(radius, 0.0, min(safe_size.x, safe_size.y) * 0.5);

    if (r <= 0.5) {
        return 1.0;
    }

    let local = (uv - vec2<f32>(0.5, 0.5)) * safe_size;
    let half_inner = safe_size * 0.5 - vec2<f32>(r, r);
    let q = abs(local) - half_inner;
    let outside = length(max(q, vec2<f32>(0.0, 0.0)));
    let inside = min(max(q.x, q.y), 0.0);
    let distance = outside + inside - r;

    return 1.0 - smoothstep(0.0, 1.35, distance);
}

@fragment
fn fs_ui(in: UiVertexOut) -> @location(0) vec4<f32> {
    let radius = in.params.x;
    let size = in.params.yz;
    let alpha = rounded_rect_alpha(in.uv, size, radius);

    if (alpha <= 0.001) {
        discard;
    }

    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}