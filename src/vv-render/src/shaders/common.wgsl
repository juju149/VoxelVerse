fn saturate(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

fn safe_normalize(v: vec3<f32>) -> vec3<f32> {
    return v / max(length(v), 1e-6);
}

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn dither_opacity(pos: vec4<f32>, alpha: f32) -> bool {
    let dither_threshold = dot(vec2<f32>(171.0, 231.0), pos.xy);
    return fract(dither_threshold / 71.0) > alpha;
}

fn hash13(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    let r = q + dot(q, q.yzx + 19.19);
    return fract((r.x + r.y) * r.z);
}

fn hash11(x: f32) -> f32 {
    return fract(sin(x * 17.123) * 43758.5453123);
}

fn smooth01(edge0: f32, edge1: f32, x: f32) -> f32 {
    return smoothstep(edge0, edge1, x);
}