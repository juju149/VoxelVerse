@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

fn vv_shadow_sample(uv: vec2<f32>, z: f32) -> f32 {
    return textureSampleCompare(t_shadow, s_shadow, uv, z);
}

fn vv_shadow(shadow_pos: vec3<f32>, ndotl: f32, pcf_level: u32) -> f32 {
    if shadow_pos.z > 1.0 ||
       shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
       shadow_pos.y < 0.0 || shadow_pos.y > 1.0 {
        return 1.0;
    }

    let bias = max(0.00036 * (1.0 - ndotl), 0.000055);
    let uv = shadow_pos.xy;
    let z = shadow_pos.z - bias;

    if pcf_level == 0u {
        return vv_shadow_sample(uv, z);
    }

    let texel = 1.35 / vec2<f32>(textureDimensions(t_shadow));

    var sum = 0.0;
    sum += vv_shadow_sample(uv, z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 1.0,  0.0), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>(-1.0,  0.0), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 0.0,  1.0), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 0.0, -1.0), z);

    if pcf_level == 1u {
        return sum / 5.0;
    }

    sum += vv_shadow_sample(uv + texel * vec2<f32>( 1.25,  1.25), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>(-1.25,  1.25), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 1.25, -1.25), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>(-1.25, -1.25), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 2.15,  0.45), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>(-2.15,  0.45), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 0.45,  2.15), z);
    sum += vv_shadow_sample(uv + texel * vec2<f32>( 0.45, -2.15), z);

    return sum / 13.0;
}
