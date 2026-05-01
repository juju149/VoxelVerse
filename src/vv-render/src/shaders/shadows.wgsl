fn shadow_visibility(shadow_pos: vec3<f32>, n_dot_l: f32) -> f32 {
    if (
        shadow_pos.z > 1.0 ||
        shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
        shadow_pos.y < 0.0 || shadow_pos.y > 1.0
    ) {
        return 1.0;
    }

    let dim = vec2<f32>(textureDimensions(t_shadow));
    let texel = 1.0 / dim;

    let bias = max(0.00012, 0.00065 * (1.0 - n_dot_l));
    let depth = shadow_pos.z - bias;

    // Slightly wider at grazing angles.
    let radius = mix(1.0, 2.25, saturate(1.0 - n_dot_l));

    var sum = 0.0;
    var weight_sum = 0.0;

    for (var ix: i32 = -1; ix <= 1; ix = ix + 1) {
        for (var iy: i32 = -1; iy <= 1; iy = iy + 1) {
            let o = vec2<f32>(f32(ix), f32(iy));
            let dist2 = dot(o, o);
            let w = exp(-dist2 * 0.85);
            let sample_uv = shadow_pos.xy + o * texel * radius;
            let sample_value = textureSampleCompare(t_shadow, s_shadow, sample_uv, depth);

            sum = sum + sample_value * w;
            weight_sum = weight_sum + w;
        }
    }

    return sum / max(weight_sum, 1e-5);
}