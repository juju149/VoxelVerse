fn vv_hash11(x: f32) -> f32 {
    return fract(sin(x * 127.1) * 43758.5453123);
}

fn vv_hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453123);
}

fn vv_hash31(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453123);
}

fn vv_hash22(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        vv_hash21(p),
        fract(sin(dot(p, vec2<f32>(269.5, 183.3))) * 43758.5453123)
    );
}

fn vv_signed_hash31(p: vec3<f32>) -> f32 {
    return vv_hash31(p) * 2.0 - 1.0;
}
