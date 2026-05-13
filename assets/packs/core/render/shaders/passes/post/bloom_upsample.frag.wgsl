fn vv_bloom_upsample(base: vec3<f32>, bloom: vec3<f32>) -> vec3<f32> {
    return base + bloom * 0.18;
}

