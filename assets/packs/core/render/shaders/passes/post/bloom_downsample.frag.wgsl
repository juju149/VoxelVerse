fn vv_bloom_threshold(color: vec3<f32>) -> vec3<f32> {
    return max(color - vec3<f32>(1.05), vec3<f32>(0.0));
}

