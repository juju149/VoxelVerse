fn vv_hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vv_hash31(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

fn vv_hash22(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        vv_hash21(p),
        fract(sin(dot(p, vec2<f32>(269.5, 183.3))) * 43758.5453)
    );
}

