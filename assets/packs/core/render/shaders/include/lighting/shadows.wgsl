@group(0) @binding(1) var t_shadow: texture_depth_2d;
@group(0) @binding(2) var s_shadow: sampler_comparison;

fn vv_shadow(shadow_pos: vec3<f32>, ndotl: f32, pcf_level: u32) -> f32 {
    if shadow_pos.z > 1.0 || shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
       shadow_pos.y < 0.0 || shadow_pos.y > 1.0 {
        return 1.0;
    }
    let bias = max(0.00035 * (1.0 - ndotl), 0.00006);
    let uv = shadow_pos.xy;
    let z = shadow_pos.z - bias;
    if pcf_level == 0u {
        return textureSampleCompare(t_shadow, s_shadow, uv, z);
    }
    let ts = 1.5 / vec2<f32>(textureDimensions(t_shadow));
    var s = textureSampleCompare(t_shadow, s_shadow, uv, z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>( ts.x, 0.0), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>(-ts.x, 0.0), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>(0.0,  ts.y), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + vec2<f32>(0.0, -ts.y), z);
    if pcf_level == 1u {
        return s * 0.2;
    }
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-1.7,  0.7), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 1.7,  0.7), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>(-0.7,  1.7), z);
    s += textureSampleCompare(t_shadow, s_shadow, uv + ts * vec2<f32>( 0.7, -1.7), z);
    return s / 9.0;
}

