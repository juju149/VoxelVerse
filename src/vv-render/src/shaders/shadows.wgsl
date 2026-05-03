// Shadow mode lives in global.planet.z:
//   0.0 = off (no shadows)
//   1.0 = stable (small kernel, stronger bias — minimises acne/flicker)
//   2.0 = high (wide PCF kernel, sharper contact shadows)
// See vv_config::ShadowMode and the VV_SHADOWS env var.

fn shadow_visibility(shadow_pos: vec3<f32>, n_dot_l: f32) -> f32 {
    let mode = global.planet.z;

    if (mode < 0.5) {
        return 1.0;
    }

    if (n_dot_l <= 0.012) {
        return 1.0;
    }

    if (
        shadow_pos.z > 1.0 ||
        shadow_pos.x < 0.0 || shadow_pos.x > 1.0 ||
        shadow_pos.y < 0.0 || shadow_pos.y > 1.0
    ) {
        return 1.0;
    }

    let dim = vec2<f32>(textureDimensions(t_shadow));
    let texel = 1.0 / dim;

    let grazing = saturate(1.0 - n_dot_l);

    // Bias scales with grazing^2 so beveled near-grazing faces (where
    // microfacets cross the depth quantum) get extra slack without
    // destroying contact shadows on flat-lit surfaces.
    let bevel_term = grazing * grazing;
    let bias = max(0.00018, 0.00040 + 0.00120 * bevel_term);
    let depth = shadow_pos.z - bias;

    var visibility = 1.0;

    if (mode < 1.5) {
        // Stable mode: 5-tap cross. Cheap, no flicker on bevels.
        let r = 1.30;
        let center = textureSampleCompare(t_shadow, s_shadow, shadow_pos.xy, depth);
        let east   = textureSampleCompare(t_shadow, s_shadow, shadow_pos.xy + vec2<f32>( r, 0.0) * texel, depth);
        let west   = textureSampleCompare(t_shadow, s_shadow, shadow_pos.xy + vec2<f32>(-r, 0.0) * texel, depth);
        let north  = textureSampleCompare(t_shadow, s_shadow, shadow_pos.xy + vec2<f32>(0.0,  r) * texel, depth);
        let south  = textureSampleCompare(t_shadow, s_shadow, shadow_pos.xy + vec2<f32>(0.0, -r) * texel, depth);
        visibility = (center * 2.0 + east + west + north + south) / 6.0;
    } else {
        // High mode: 25-tap soft Gaussian PCF.
        let radius = mix(0.95, 2.65, grazing);

        var sum = 0.0;
        var weight_sum = 0.0;
        for (var ix: i32 = -2; ix <= 2; ix = ix + 1) {
            for (var iy: i32 = -2; iy <= 2; iy = iy + 1) {
                let o = vec2<f32>(f32(ix), f32(iy));
                let dist2 = dot(o, o);
                let w = exp(-dist2 * 0.42);
                let sample_uv = shadow_pos.xy + o * texel * radius;
                let sample_value = textureSampleCompare(t_shadow, s_shadow, sample_uv, depth);
                sum = sum + sample_value * w;
                weight_sum = weight_sum + w;
            }
        }
        visibility = sum / max(weight_sum, 1e-5);
    }

    // Avoid pitch-black crawl on grazing faces.
    visibility = mix(visibility, 1.0, grazing * 0.08);

    return saturate(visibility);
}
