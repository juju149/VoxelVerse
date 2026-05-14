fn vv_fog_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_fog_smooth5(t: f32) -> f32 {
    let c = vv_fog_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

// Disabled for now.
// The previous UV-based fullscreen veil created a fake horizontal fog band.
// Keep atmosphere world-space only until depth-aware volumetric fog exists.
fn vv_fog_veil_alpha(uv: vec2<f32>) -> f32 {
    return 0.0;
}