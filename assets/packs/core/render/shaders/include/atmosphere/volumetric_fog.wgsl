fn vv_fog_saturate(v: f32) -> f32 {
    return clamp(v, 0.0, 1.0);
}

fn vv_fog_smooth5(t: f32) -> f32 {
    let c = vv_fog_saturate(t);
    return c * c * c * (c * (c * 6.0 - 15.0) + 10.0);
}

// Fullscreen fog veil.
// This is only a tiny cinematic layer. Real fog is handled in vv_aerial_fog.
// Keep it extremely subtle to avoid visible horizontal bands.
fn vv_fog_veil_alpha(uv: vec2<f32>) -> f32 {
    let qbits = vv_quality_bits();
    let enabled = (qbits & 16u) != 0u;

    if !enabled {
        return 0.0;
    }

    // Thin horizon-only veil.
    let enter = vv_fog_smooth5((uv.y - 0.34) * 6.0);
    let exit = 1.0 - vv_fog_smooth5((uv.y - 0.56) * 8.0);
    let horizon_band = enter * exit;

    // No fog at the bottom of the screen.
    let bottom_kill = 1.0 - vv_fog_smooth5((uv.y - 0.64) * 10.0);

    // Ultra subtle. Anything stronger becomes a visible UI-like stripe.
    let density = global.atmosphere_params.z * 0.010;

    return clamp(horizon_band * bottom_kill * density, 0.0, 0.022);
}
